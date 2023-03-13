use base64;
use chrono::Utc;
use reqwest;
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use percent_encoding::{utf8_percent_encode, AsciiSet};
use std::collections::HashMap;
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader,Write, BufWriter};
use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};
use serde_json::json;

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
// レスポンスで必要な部分だけ記述
// これを戻り値にせずserde_json::Valueで全部取得してもよい

// Twitterの認証関連と一部ラッパー実装
pub struct Twitter {
    client_id: String,
    client_secret: String
}

impl Twitter {
   
    // インスタンス生成
    pub fn new(
        client_id: String, client_secret: String)
        -> Twitter {
        Twitter {
            client_id, client_secret
        }
    }

   

    

    pub async fn post(&self, text: String) -> reqwest::Result<()> {
        let client = reqwest::Client::new();
        let bearer_token = self.get_access_token().await.unwrap();

        let post_data = json!({ "text" : text });

     
        let res = client.post("https://api.twitter.com/2/tweets")
            .bearer_auth(bearer_token)
            .header("Content-Type","application/json")
            .json(&post_data)
            .send()
            .await
            .unwrap();

        println!("{:?}",res);

        Ok(())


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
        
        

        
        



        return Ok(String::new());
    }
}