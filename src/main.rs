use dotenv::dotenv;
use std::env;
pub mod scraper;
use crate::scraper::scraper::Scraper;
use crate::twitter::Twitter;
use crate::chatgpt::ChatGPT;
use chrono::prelude::*;
mod twitter;
mod chatgpt;
mod instagram;
pub mod helper;
use std::fs;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::Read;
use bytes::Bytes;
use diesel::prelude::*;
use diesel::SqliteConnection;
use pokabot::models::{NewBlog, NewNews};
use fs2::FileExt;
use std::fs::OpenOptions;
use std::path::Path;
use std::io::{self, ErrorKind};
extern crate chrono;
use chrono::Local;

// fn extract_path(url_or_path: &str) -> String {
//     // 相対パスの場合、適当なドメインを追加して完全なURLを作成
//     let url = if !url_or_path.starts_with("http://") && !url_or_path.starts_with("https://") {
//         format!("https://example.com{}", url_or_path)
//     } else {
//         url_or_path.to_string()
//     };

//     let parsed_url = Url::parse(&url).unwrap();

//     // URLからパス部分を取得して返す
//     parsed_url.path().to_string()
// }


async fn tweet_news(news_id: &str ,twitter: &Twitter, chatgpt: &ChatGPT, scraper: &Scraper, lang: &str, connection: &mut SqliteConnection) -> anyhow::Result<()> {

    let news = scraper.scrape_news(news_id).await?;
    let now = Local::now();
    let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let name2nickname = [
        ("加藤史帆", "かとし"),
        ("齊藤京子", "きょんこ"),
        ("佐々木久美", "くみてん"),
        ("佐々木美玲", "みーぱん"),
        ("高瀬愛奈", "まなふぃ"),
        ("高本彩花", "おたけ"),
        ("東村芽依", "めいめい"),
        ("金村美玖", "みくちゃん"),
        ("河田陽菜", "ひなちゃん"),
        ("小坂菜緒", "こさかな"),
        ("富田鈴花", "すーじー"),
        ("丹生明里", "にぶちゃん"),
        ("濱岸ひより", "ひよたん"),
        ("松田好花", "このちゃん"),
        ("上村ひなの", "ひなのちゃん"),
        ("髙橋未来虹", "みくにん"),
        ("森本茉莉", "まりぃ"),
        ("山口陽世", "ぱる"),
        ("石塚瑶季", "たまちゃん"),
        ("小西夏菜実", "こにしん"),
        ("清水理央","りおちゃん"),
        ("正源司陽子", "よーこ"),
        ("竹内希来里", "きらりん"),
        ("平尾帆夏", "ひらほー"),
        ("平岡海月","みっちゃん"),
        ("藤嶌果歩","かほりん"),
        ("宮地すみれ","すみれちゃん"),
        ("山下葉留花","はるはる"),
        ("渡辺莉奈","りなし")];


    let prompt = {
        format!("以下は、日向坂46というアイドルグループに関するニュースです。
        ファンになったつもりで、ニュースの内容を要約し、カジュアルな日本語40字以内で短めにツイートしなさい。
        現在時刻は{}です。
        あだ名リストは適宜使ってください。
        [あだ名リスト] \n {}", now_str, name2nickname.iter().map(|(name, nickname)| format!("{}: {}", name, nickname)).collect::<Vec<String>>().join("\n"))
    };

    loop {
        let body = chatgpt.get_response(format!("{}\n [ニュース] \n {}", prompt, news.body())).await?;
        let news_url = news.url();
        let images = news.images();

        let text = if news_url.contains("https") {
            format!("{} \n{}", body, news_url)
        } else {
            format!("{} \n{}{}",body, scraper.get_base(), news_url)
        };

        if helper::is_within_twitter_limit(&text) {
            twitter.post_thread(&text, &images).await?;
            break;
        }
    }

    save_news(news_id, news.posted_at(), lang, connection);


    Ok(())
}

async fn tweet_until_latest_news(twitter: &Twitter, chatgpt: &ChatGPT, scraper: &Scraper, lang: &str, connection: &mut SqliteConnection) {

   
    if let Ok(news_ids) = scraper.scrape_news_ids().await {
        for news_id in news_ids.into_iter().rev() {
            let is_tweeted = is_news_tweeted(&news_id, lang, connection);

            match is_tweeted {
                Ok(true) => continue,
                Ok(false) | Err(_) => { 
                    let result = tweet_news(&news_id, &twitter, &chatgpt, &scraper, lang, connection).await;
                    match result {
                        Ok(_) => println!("Tweeted successfully!"),
                        Err(error) => eprintln!("{:?}", error)
                    }
                }
            }

        }
    }

    

}

async fn tweet_until_latest_post(twitter: &Twitter, chatgpt: &ChatGPT, scraper: &Scraper, lang: &str, connection: &mut SqliteConnection){


    if let Ok(post_ids) = scraper.scrape_post_ids().await {
        for post_id in post_ids.into_iter().rev() {

            let is_tweeted = is_post_tweeted(post_id, lang, connection);

            match is_tweeted {
                Ok(true) => continue,
                Ok(false) | Err(_) => {
                    let result = tweet_blog(post_id, &twitter, &chatgpt, &scraper, lang, connection).await;
                    match result {
                        Ok(_) => println!("Tweeted successfully!"),
                        Err(error) => eprintln!("{:?}", error)
                    }
                }
            }
        }
    }

    

}


fn truncate_string(input: &str, length: usize) -> String {
    let mut truncated = String::new();
    let mut char_count = 0;

    for c in input.chars() {
        if char_count >= length {
            break;
        }
        truncated.push(c);
        char_count += 1;
    }

    truncated
}

fn trim_outer_quotes(s: &str) -> &str {
    if s.starts_with('"') && s.ends_with('"') {
        &s[1..s.len() - 1]
    } else if s.starts_with('「') && s.ends_with('」') {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

async fn tweet_blog(post_id: i32 ,twitter: &Twitter, chatgpt: &ChatGPT, scraper: &Scraper, lang: &str, connection: &mut SqliteConnection) -> anyhow::Result<()>{

    
    let post_url = format!("https://www.hinatazaka46.com/s/official/diary/detail/{}?ima=0000&cd=member", post_id);

    let blog = scraper.scrape_blog(post_id).await?;
    let max_length = 3800;

    let name = blog.name();
    let title = blog.title();
    let images = blog.images();
    let body = truncate_string(blog.body(), max_length);

    let posted_at = blog.posted_at();
    save_blog(post_id, name, posted_at, "none", connection);

    let now = Local::now();
    let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let name2nickname = [
        ("加藤史帆", "かとし"),
        ("齊藤京子", "きょんこ"),
        ("佐々木久美", "くみてん"),
        ("佐々木美玲", "みーぱん"),
        ("高瀬愛奈", "まなふぃ"),
        ("高本彩花", "おたけ"),
        ("東村芽依", "めいめい"),
        ("金村美玖", "みくちゃん"),
        ("河田陽菜", "ひなちゃん"),
        ("小坂菜緒", "こさかな"),
        ("富田鈴花", "すーじー"),
        ("丹生明里", "にぶちゃん"),
        ("濱岸ひより", "ひよたん"),
        ("松田好花", "このちゃん"),
        ("上村ひなの", "ひなのちゃん"),
        ("髙橋未来虹", "みくにん"),
        ("森本茉莉", "まりぃ"),
        ("山口陽世", "ぱる"),
        ("石塚瑶季", "たまちゃん"),
        ("小西夏菜実", "こにしん"),
        ("清水理央","りおちゃん"),
        ("正源司陽子", "よーこ"),
        ("竹内希来里", "きらりん"),
        ("平尾帆夏", "ひらほー"),
        ("平岡海月","みっちゃん"),
        ("藤嶌果歩","かほりん"),
        ("宮地すみれ","すみれちゃん"),
        ("山下葉留花","はるはる"),
        ("渡辺莉奈","りなし")];


    let prompt = if lang == "jp" {  
        if name == "ポカ" {
            format!("
            以下のブログを書いた本人になりきって、短い一文(日本語20字程度)で、ブログの宣伝ツイートをしてください。ただし、必ずTwitterの文字数制限を遵守しなさい。現在時刻は{}です。
            あだ名リストは適宜使ってください。
            [あだ名リスト] \n {} \n
            [タイトル] {} \n
            [投稿者] {} \n
            [本文] {}
            ", now_str, name2nickname.iter().map(|(name, nickname)| format!("{}: {}", name, nickname)).collect::<Vec<String>>().join("\n"), title, name, body)
        } else {
            format!("
            あなたはアイドルオタクです。以下は、日向坂46という日本の女性アイドルグループのメンバーのブログです。このブログ内の何か一つ話題を取り上げ、それに関してあなたが思ったことや考えたことを短い一文(日本語30字程度)でツイートしなさい。ただし、必ずTwitterの文字数制限を遵守しなさい。現在時刻は{}です。
            あだ名リストは適宜使ってください。
            [あだ名リスト] \n {} \n
            [タイトル] {} \n
            [投稿者] {} \n
            [本文] {}
            ", now_str, name2nickname.iter().map(|(name, nickname)| format!("{}: {}", name, nickname)).collect::<Vec<String>>().join("\n"), title, name, body)
        }
    } else {
        if name == "ポカ" {
            format!("---\n Act as the writer of the blog below and make a promotional tweet about it within 150 characters in English briefly.")
        } else {
            format!("---\nRead the idol's blog below and tweet your comment to it casually as one of her fans within 150 characters in English briefly.")
        }
    };

    

    loop {
        let body = chatgpt.get_response(format!("{}",prompt)).await?;
        

        
        let text = format!("{} \n{}", trim_outer_quotes(&body), post_url);

        if helper::is_within_twitter_limit(&text) {
            twitter.post_thread(&text, &images).await?;
            break;
        }
    }

    save_blog(post_id, name, posted_at, lang, connection);

    

    Ok(())
    
}

#[derive(Serialize, Deserialize, Debug)]
struct UserInfo {
    username: String,
    nickname: String,
    timestamp: i64
}

#[derive(Serialize, Deserialize, Debug)]
struct PostInfo {
    node : Node
}

#[derive(Serialize, Deserialize, Debug)]
struct Node {
    shortcode: String
}

// struct Post {
//     username: String,
//     nickname: String,
//     media : Vec<Bytes>,
//     url: String, 
//     text: String,
// }





// async fn tweet_instagram(twitter: &Twitter, chatgpt: &ChatGPT){
    
//     let json_file = "instagram_users.json";
//     let json_data = fs::read_to_string(json_file).unwrap();

//     // Parse the JSON data into a serde_json::Value
//     let users:Vec<UserInfo> = serde_json::from_str(&json_data).unwrap();
    

//     for user in users {

//         let username = user.username;
//         let nickname = user.nickname;
//         let entries = fs::read_dir(&username).unwrap();
        
//         let mut new_txt_files:Vec<String> = vec![];

//         for entry in entries {
//             if let Ok(entry) = entry {
//                 let filename = String::from(entry.file_name().to_str().unwrap());
//                 if let Ok(dt) =  Utc.datetime_from_str(&filename, "%Y-%m-%d_%H-%M-%S_UTC.txt") {
//                     let timestamp = dt.timestamp();
//                     let prev_timestamp = user.timestamp;
//                     if prev_timestamp < timestamp {
//                         new_txt_files.push(filename);
//                     }
                    
//                 }
//             }
//         }

//         let mut posts : Vec<Post> = vec![];


//         for new_txt_file in new_txt_files {
//             let mut f = File::open(format!("{}/{}",username, new_txt_file)).expect("file not found");
//             let mut text = String::new();
//             f.read_to_string(&mut text).unwrap();

//             let mut i = 1;

//             let mut media : Vec<Bytes> = vec![];

//             loop {
//                 let path = format!("{}/{}_{}.jpg",username, new_txt_file.replace(".txt", ""), i);
//                 if let Ok(mut file) = File::open(path) {
//                     let mut buffer = Vec::new();
//                     file.read_to_end(&mut buffer).unwrap();
//                     media.push(Bytes::from(buffer));
//                     i += 1;
//                 } else {
//                     break;
//                 }
//             }  

//             let json_file = new_txt_file.replace(".txt", ".json");
//             let json_data = fs::read_to_string(format!("{}/{}",username, json_file)).unwrap();

//             // Parse the JSON data into a serde_json::Value
//             let info:PostInfo = serde_json::from_str(&json_data).unwrap();

//             let post = Post {
//                 username: username.clone(),
//                 nickname: nickname.clone(),
//                 media: media,
//                 text: text,
//                 url: format!("https://www.instagram.com/p/{}",info.node.shortcode)
//             };

//             posts.push(post);

//         }


            

//         for post in posts {
//             loop {
//                 let prompt = format!("以下は、日本の日向坂46というアイドルグループのメンバーである、{}のInstagramの投稿に、本人が一緒に載せた文章です。投稿された画像をあなたに見せることはできませんが、この文章に対して、違和感のない感想を、日本語40字以内で短めに、そしてカジュアルにツイートしてください。\n {}", nickname, post.text);

//                 let text = chatgpt.get_response(prompt).await.unwrap();
//                 let text = format!("{} \n {}",text, post.url);
//                 if helper::is_within_twitter_limit(&text) {
//                     twitter.post_thread(&text, &post.media).await.unwrap();
//                     break;
//                 }
        
//             }
//         }
//     }
    
// }

fn establish_connection() -> SqliteConnection {
    let database_url = "pokabot.db";
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}


fn is_new_post(post_id: i32, connection: &mut SqliteConnection) -> bool {
    use pokabot::schema::blogs::dsl::*;
    use pokabot::models::Blog;
    match blogs.filter(id.eq(post_id)).first::<Blog>(connection) {
            Ok(_) => false,
            Err(_) => true,
    }
}

fn is_new_news(news_id_str: &str, connection: &mut SqliteConnection) -> bool {
    use pokabot::schema::news::dsl::*;
    use pokabot::models::News;
    match news.filter(news_id.eq(news_id_str)).first::<News>(connection) {
            Ok(_) => false,
            Err(_) => true,
    }
}

fn is_post_tweeted(post_id: i32, lang: &str, connection: &mut SqliteConnection) -> anyhow::Result<bool> {
    use pokabot::schema::blogs::dsl::*;
    use pokabot::models::Blog;
    let blog = blogs.filter(id.eq(post_id)).first::<Blog>(connection)?;
    if lang == "jp" {
        return Ok(blog.jp_tweeted);
    } else {
        return Ok(blog.eng_tweeted);
    }
}

fn is_news_tweeted(n_id: &str, lang: &str, connection: &mut SqliteConnection) -> anyhow::Result<bool> {
    use pokabot::schema::news::dsl::*;
    use pokabot::models::News;
    let n = news.filter(news_id.eq(n_id)).first::<News>(connection)?;
    if lang == "jp" {
        return Ok(n.jp_tweeted);
    } else {
        return Ok(n.eng_tweeted);
    }
}


fn save_blog(post_id: i32, name: &str, posted_at: &NaiveDateTime, lang: &str, connection: &mut SqliteConnection) {
    use pokabot::schema::blogs;

    if is_new_post(post_id, connection) {
        let new_blog = NewBlog {
            id: post_id,
            name: name,
            posted_at: posted_at.clone(), // 修正
            jp_tweeted: if lang == "jp" {true} else {false},
            eng_tweeted: if lang == "eng" {true} else {false},
        };

         diesel::insert_into(blogs::table)
        .values(&new_blog)
        .execute(connection)
        .expect("Error saving new post");

    } else {
        use pokabot::schema::blogs::dsl::*;
        let target = blogs.filter(pokabot::schema::blogs::id.eq(post_id));

        if lang == "jp" {
            diesel::update(target)
            .set(pokabot::schema::blogs::jp_tweeted.eq(true))
            .execute(connection).expect("Error updating the post");
        } else {
            diesel::update(target)
            .set(pokabot::schema::blogs::eng_tweeted.eq(true))
            .execute(connection).expect("Error updating the post");
        }
       
    }

   
}

fn save_news(news_id: &str, posted_at: &NaiveDateTime, lang: &str, connection: &mut SqliteConnection){
    use pokabot::schema::news;

    if is_new_news(news_id, connection) {
        println!("new news");
        let new_news = NewNews {
            news_id: news_id,
            posted_at: posted_at.clone(), // 修正
            jp_tweeted: if lang == "jp" {true} else {false},
            eng_tweeted: if lang == "eng" {true} else {false},
        };

         diesel::insert_into(news::table)
        .values(&new_news)
        .execute(connection)
        .expect("Error saving new post");
    } else {
        println!("update news");
        use pokabot::schema::news::dsl::*;
        let target = news.filter(pokabot::schema::news::news_id.eq(news_id));

        if lang == "jp" {
            diesel::update(target)
            .set(pokabot::schema::news::jp_tweeted.eq(true))
            .execute(connection).expect("Error updating the post");
        } else {
            diesel::update(target)
            .set(pokabot::schema::news::eng_tweeted.eq(true))
            .execute(connection).expect("Error updating the post");
        }
    }

}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let gpt_api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set.");
    let consummer_key = env::var("CK").expect("CK must be set.");
    let consummer_secret = env::var("CS").expect("CS must be set.");
    let access_token_key = env::var("AT").expect("AT must be set.");
    let access_token_secret = env::var("AS").expect("AS must be set.");
    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID must be set.");
    let client_secret = env::var("CLIENT_SECRET").expect("CLIENT_SECRET must be set.");
    let user_id = env::var("USER_ID").expect("USER_ID must be set.");
    // let insta_access_token = env::var("INSTA_ACCESS_TOKEN").expect("INSTA_ACCESS_TOKEN must be set.");
    

    let twitter = twitter::Twitter::new(
        client_id, client_secret, consummer_key, consummer_secret, access_token_key, access_token_secret, user_id
    );
    let chatgpt = chatgpt::ChatGPT::new(gpt_api_key);

   
    let connection = &mut establish_connection();
    let base = "https://www.hinatazaka46.com";
    let blog_url = "https://www.hinatazaka46.com/s/official/diary/member?ima=0000";
    let news_url = "https://www.hinatazaka46.com/s/official/?ima=0000";
    let scraper = Scraper::new(base, blog_url, news_url);

    let args: Vec<String> = env::args().collect();

    let lang = args.get(1).expect("Specify an argument.");

    
    let lock_file_path = Path::new("app.lock");

    // ロックファイルを作成または開く
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&lock_file_path)?;


    match file.try_lock_exclusive() {
        Ok(_) => {
            // ロックが取得できた場合、プログラムを実行
            println!("Lock acquired, running program...");
            tweet_until_latest_post(&twitter, &chatgpt, &scraper, lang, connection).await;
            tweet_until_latest_news(&twitter, &chatgpt, &scraper, lang, connection).await;

            // ロックを解除
            file.unlock()?;
            println!("Program finished, lock released.");
        }
        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
            // ロックが既に取得されている場合、エラーメッセージを表示
            eprintln!("Another instance of the program is already running.");
        }
        Err(e) => {
            // その他のエラーが発生した場合、エラーメッセージを表示
            eprintln!("An error occurred while trying to lock the file: {}", e);
        }
    }

    

    Ok(())
    // tweet_instagram(&twitter, &chatgpt ).await;

    

    

}
