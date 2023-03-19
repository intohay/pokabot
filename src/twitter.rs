use base64;
use chrono::Utc;
use reqwest;
use reqwest::multipart;
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use percent_encoding::{utf8_percent_encode, AsciiSet};
use serde::de::value;
use std::collections::HashMap;
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::io::{BufReader,Write, BufWriter};
use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};
use serde_json::json;
use bytes::Bytes;

#[derive(Serialize, Deserialize, Debug)]
struct Token {
    access_token: String,
    expires_in: i64,
    timestamp: i64,
    refresh_token: String
}

#[derive(Serialize, Deserialize, Debug)]
struct ResponseToken {
    access_token: String,
    expires_in: i64,
    refresh_token: String
}

#[derive(Serialize, Deserialize, Debug)]
struct TweetResponse {
    data : TweetInfo
}

#[derive(Serialize, Deserialize, Debug)]
struct TweetInfo {
    id : String,
    text: String
}
// レスポンスで必要な部分だけ記述
// これを戻り値にせずserde_json::Valueで全部取得してもよい

// Twitterの認証関連と一部ラッパー実装
pub struct Twitter {
    client_id: String,
    client_secret: String,
    consummer_key: String,
    consummer_secret: String,
    access_token_key: String,
    access_token_secret: String,
    user_id: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    media_id_string : String
}

impl Twitter {
    const FRAGMENT: AsciiSet = percent_encoding::NON_ALPHANUMERIC
        .remove(b'*')
        .remove(b'-')
        .remove(b'.')
        .remove(b'_');


    // インスタンス生成
    pub fn new(
        client_id: String, client_secret: String, consummer_key: String, consummer_secret: String, access_token_key: String, access_token_secret: String, user_id: String)
        -> Twitter {
        Twitter {
            client_id, client_secret, consummer_key, consummer_secret, access_token_key, access_token_secret, user_id
        }
    }
    fn get_request_header(&self, method: &str, endpoint: &str) -> String {
        let nonce = format!("nonce{}", Utc::now().timestamp());
        let timestamp = format!("{}", Utc::now().timestamp());
        // oauth_*パラメータ
        let mut oauth_params: HashMap<&str, &str> = HashMap::new();
        oauth_params.insert("oauth_consumer_key", &self.consummer_key);
        oauth_params.insert("oauth_nonce", &nonce);
        oauth_params.insert("oauth_signature_method", "HMAC-SHA1");
        oauth_params.insert("oauth_timestamp", &timestamp);
        oauth_params.insert("oauth_token", &self.access_token_key);
        oauth_params.insert("oauth_version", "1.0");

        // シグネチャを計算
        let oauth_signature = self.get_oauth_signature(
            method, endpoint,
            &self.consummer_secret, &self.access_token_secret,
            &oauth_params);

        // シグネチャをoauth_*パラメータに追加
        oauth_params.insert("oauth_signature", &oauth_signature);

        // ヘッダを返す
        format!(
            "OAuth {}",
            oauth_params
                .into_iter()
                .map(|(key, value)| {
                    format!(r#"{}="{}""#,
                            utf8_percent_encode(key, &Self::FRAGMENT),
                            utf8_percent_encode(value, &Self::FRAGMENT))
                })
                .collect::<Vec<String>>()
                .join(", ")
            )
    }
    
    fn get_oauth_signature(
        &self, method: &str, endpoint: &str,
        consummer_secret: &str, access_token_secret: &str,
        params: &HashMap<&str, &str>
        ) -> String {

        let key: String = format!("{}&{}",
                                  utf8_percent_encode(consummer_secret, &Self::FRAGMENT),
                                  utf8_percent_encode(access_token_secret, &Self::FRAGMENT));

        let mut params: Vec<(&&str, &&str)> = params.into_iter().collect();
        params.sort();

        let param_string = params
            .into_iter()
            .map(|(key, value)| {
                format!("{}={}",
                        utf8_percent_encode(key, &Self::FRAGMENT),
                        utf8_percent_encode(value, &Self::FRAGMENT))
            })
            .collect::<Vec<String>>()
            .join("&");

        let data = format!("{}&{}&{}",
                           utf8_percent_encode(method, &Self::FRAGMENT),
                           utf8_percent_encode(endpoint, &Self::FRAGMENT),
                           utf8_percent_encode(&param_string, &Self::FRAGMENT));

        let hash = hmacsha1::hmac_sha1(key.as_bytes(), data.as_bytes());

        base64::encode(&hash)
    }

    

    pub async fn post(&self, text: String, images: &Vec<Bytes>) -> reqwest::Result<()> {

        let mut media_ids: Vec<String> = vec![];
        for image in images{
            media_ids.push(self.upload_image(image).await);
        }

    
        
        let client = reqwest::Client::new();
        let bearer_token = self.get_access_token().await.unwrap();

        let mut post_data = json!({ "text" : text });
        

       
        if images.len() != 0 {
            post_data = 
            json!({ 
                "text" : text,
                "media" : {
                    "media_ids" : media_ids[0..1]
                } 
            });
        }
        

     
        let res = client.post("https://api.twitter.com/2/tweets")
            .bearer_auth(&bearer_token)
            .header("Content-Type","application/json")
            .json(&post_data)
            .send()
            .await
            .unwrap().text().await.unwrap();

        println!("{}",res);

        let mut tweet : TweetResponse = serde_json::from_str(&res).unwrap();

        let rest = media_ids.iter().skip(1).cloned().collect::<Vec<_>>();

        for chunk in rest.chunks(4) {

            let ids_str = chunk.join(",");

            post_data = 
                json!({ 
                    "media" : {
                        "media_ids" : ids_str
                    },
                    "reply" : {
                        "in_reply_to_tweet_id": &tweet.data.id
                    }
                });

            let res = client.post("https://api.twitter.com/2/tweets")
            .bearer_auth(&bearer_token)
            .header("Content-Type","application/json")
            .json(&post_data)
            .send()
            .await
            .unwrap().text().await.unwrap();

            println!("{}",res);

            tweet = serde_json::from_str(&res).unwrap();
        }


        Ok(())


    }

    // image : base64 encode
    async fn upload_image(&self, image: &Bytes) -> String {
        let client = reqwest::Client::new();
       
        // let bearer_token = self.get_access_token().await.unwrap();
        let endpoint = "https://upload.twitter.com/1.1/media/upload.json".to_string();

        let header_auth = self.get_request_header("POST", &endpoint);
        // println!("{}", header_auth);


        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, header_auth.parse().unwrap());
        // headers.insert(CONTENT_TYPE, "multipart/form-data".parse().unwrap());
        

        // let mut file = File::open("miku.jpg").unwrap();
        // let mut buffer:Vec<u8> = Vec::new();
        // file.read_to_end(&mut buffer).unwrap();

        let part = multipart::Part::bytes(image.to_vec())
        .file_name("image.jpg")
        .mime_str("image/jpeg").unwrap();

        // let part = multipart::Part::bytes(buffer).file_name("image.png");

        let form = reqwest::multipart::Form::new()
            .text("additional_owners", self.user_id.clone())
            .part("media", part);

        let res = client.post(&endpoint)
            // .bearer_auth(bearer_token)
            .headers(headers)
            .multipart(form)
            .send()
            .await
            .unwrap().text().await.unwrap();

        
            
        

        println!("{}", res);
        
        let image: Image = serde_json::from_str(&res).unwrap();

        return image.media_id_string;
    }

    pub async fn get_user_id(&self, screen_name: &str) -> String {
        let client = reqwest::Client::new();
        let bearer_token = self.get_access_token().await.unwrap();
        let endpoint = format!("https://api.twitter.com/1.1/users/show.json?screen_name={}",screen_name);
        let header_auth = self.get_request_header("GET", &endpoint);

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, header_auth.parse().unwrap());


        let res = client.get(endpoint)
            .bearer_auth("AAAAAAAAAAAAAAAAAAAAAPyjmAEAAAAA7Xe%2BTE0Ep%2FDgM4CN0kOk%2F%2BSYu5A%3D0UqBlGrtq5IHADx4jkvj0KEYE8oZkgzvSgHCSI7ea5eftLeGPS")
            // .headers(headers)
            .send()
            .await
            .unwrap().text().await.unwrap();

        println!("{}",res);
        return String::new();

    }
    #[allow(dead_code)]
    pub async fn get_access_token(&self) -> reqwest::Result<String> {
       
        let file = File::open("token.json").unwrap();
        let reader = BufReader::new(file);

        let deserialized_token: Token = serde_json::from_reader(reader).unwrap();
        
        let previous_timestamp = deserialized_token.timestamp;
        let dt: DateTime<Local> = Local::now();
        let timestamp: i64 = dt.timestamp();
        if timestamp - previous_timestamp < deserialized_token.expires_in {
            return Ok(deserialized_token.access_token);
        } else {
            let endpoint = "https://api.twitter.com/2/oauth2/token";
            let client = reqwest::Client::new();

            let mut params = HashMap::new();
            let refresh_token : &str = &(deserialized_token.refresh_token);
         
            params.insert("refresh_token", refresh_token);
            params.insert("grant_type", "refresh_token");
            params.insert("client_id", &self.client_id);

            let res = client.post(endpoint)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&params)
            .send()
            .await?
            .text()
            .await?;
            
            println!("{:?}", res);

            let deserialized_res: ResponseToken = serde_json::from_str(&res).unwrap();
            let tokens = Token {
                access_token : deserialized_res.access_token, 
                expires_in: deserialized_res.expires_in,
                timestamp: timestamp,
                refresh_token: deserialized_res.refresh_token
            };

            println!("new access_token: {}\n new refresh_token: {}", tokens.access_token, tokens.refresh_token);

            let serialized_res: String = serde_json::to_string(&tokens).unwrap();
            {
                let mut fout = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open("token.json")
                .unwrap();
                fout.write_all(serialized_res.as_bytes());
            }

            return Ok(tokens.access_token);
            

        }
        
        

        
    }
}