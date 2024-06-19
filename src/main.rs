use dotenv::dotenv;
use std::env;
use std::mem;
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
use std::collections::HashMap;
use anyhow::{Context, Result};
use log::{debug, info};
use env_logger;


#[derive(Debug, Deserialize)]
struct Person {
    full_name: String,
    nickname: String,
    hashtag: String,
}




async fn tweet_news(news_id: &str ,twitter: &Twitter, chatgpt: &ChatGPT, scraper: &Scraper,connection: &mut SqliteConnection, member_info: &HashMap<String, Person> ) -> Result<()> {

    let news = scraper.scrape_news(news_id).await?;
    let now = Local::now();
    let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

    save_news(news_id, news.posted_at(), connection);
    
    let prompt = {
        format!("以下は、日向坂46というアイドルグループに関するニュースです。
        ファンになったつもりで、ニュースの内容を要約し、カジュアルな日本語40字以内で短めにツイートしなさい。
        現在時刻は{}です。
        文章中にメンバーの名前が現れたときは下のあだ名リストを使ってあだ名に置き換えてください。\n
        [あだ名リスト] \n {} \n 
        
        出力形式: \n
        {{'output': 'Tweet text'}} \n
        ", now_str, member_info.iter().map(|(name, person)| format!("{}: {}", name, person.nickname)).collect::<Vec<String>>().join("\n"))
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
            println!("paased tweet limit");
            twitter.post_thread(&text, &images).await?;
            println!("posted");
            break;
        }
    }

    


    Ok(())
}

async fn tweet_until_latest_news(twitter: &Twitter, chatgpt: &ChatGPT, scraper: &Scraper, connection: &mut SqliteConnection, member_info: &HashMap<String, Person>) -> Result<()> {

    let news_ids = scraper.scrape_news_ids().await?;

    for news_id in news_ids.into_iter().rev() {
        match is_news_tweeted(&news_id, connection)? {
            true => continue,
            false => { 
                tweet_news(&news_id, &twitter, &chatgpt, &scraper, connection, member_info).await?;
                println!("Tweeted successfully!");
            }
        }

    }
    

    Ok(())

    

}
async fn tweet_until_latest_post(
    twitter: &Twitter,
    chatgpt: &ChatGPT,
    scraper: &Scraper,
    connection: &mut SqliteConnection,
    member_info: &HashMap<String, Person>,
) -> Result<()> {
    let post_ids = scraper.scrape_post_ids().await?;

    for post_id in post_ids.into_iter().rev() {
        match is_post_tweeted(post_id, connection)? {
            true => continue,
            false => {
                tweet_blog(post_id, &twitter, &chatgpt, &scraper, connection, member_info).await?;
                println!("Tweeted successfully!");
            }
        }
    }

    Ok(())
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


async fn tweet_blog(post_id: i32 ,twitter: &Twitter, chatgpt: &ChatGPT, scraper: &Scraper, connection: &mut SqliteConnection, member_info: &HashMap<String, Person>) -> Result<()>{

    
    let post_url = format!("https://www.hinatazaka46.com/s/official/diary/detail/{}?ima=0000&cd=member", post_id);

    let blog = scraper.scrape_blog(post_id).await?;
    let max_length = 3800;

    let name = blog.name();
    let title = blog.title();
    let images = blog.images();
    let body = truncate_string(blog.body(), max_length);

    let posted_at = blog.posted_at();
    save_blog(post_id, name, posted_at, connection);

    let now = Local::now();
    let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

    
    let default_hashtag = "#日向坂46";
    let person = member_info.get(name);
    let mut hashtag = "";
    let mut nickname = "";
    if let Some(person) = person {
        hashtag = &person.hashtag;
        nickname = &person.nickname;
    } else {
        hashtag = default_hashtag;
    }

    

    let prompt = 
        if name == "ポカ" {
            format!("
            以下のブログを書いた本人になりきって、短い一文で、ブログの宣伝ツイートをしてください。ただし、必ずTwitterの文字数制限を遵守しなさい。現在時刻は{}です。
            メンバーの名前が文章中に現れたときは、下のあだ名リストを使ってあだ名に置き換えてっください。またハッシュタグは必ず{}を含めるようにしてください。\n
            [あだ名リスト] \n {} \n
            [タイトル] {} \n
            [投稿者] {} \n
            [本文] \n {} \n

            出力形式: \n
            {{'output': 'Tweet text'}} \n
            ", 
            now_str,"#pokablog", member_info.iter().map(|(name, person)| format!("{}: {}", name, person.nickname)).collect::<Vec<String>>().join("\n"), title, name, body)
        } else {
            format!("
            あなたはアイドルオタクです。以下は、日向坂46という日本の女性アイドルグループのメンバーのブログです。このブログ内の何か一つの話題を取り上げ、それに関してあなたが思ったことや考えたことを短い一文でツイートしなさい。ただし、必ずTwitterの文字数制限を遵守しなさい。現在時刻は{}です。
            メンバーの名前が文章中に現れたときは、下のあだ名リストを使ってあだ名に置き換えてっください。またハッシュタグは必ず{}を含めるようにしてください。
            [あだ名リスト] \n {} \n
            [タイトル] {} \n
            [投稿者] {} (あだ名: {}) \n
            [本文] \n {} \n

            出力形式: \n
            {{'output': 'Tweet text'}} \n
            ", now_str, hashtag, member_info.iter().map(|(name, person)| format!("{}: {}", name, person.nickname)).collect::<Vec<String>>().join("\n"), title, name, nickname, body)
        };
   
    

    loop {
        let body = chatgpt.get_response(format!("{}",prompt)).await?;

        
        let text = format!("{} \n{}", &body, post_url);

        if helper::is_within_twitter_limit(&text) {
            twitter.post_thread(&text, &images).await?;
            break;
        }
    }

    // save_blog(post_id, name, posted_at, connection);

    

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

fn is_post_tweeted(post_id: i32, connection: &mut SqliteConnection) -> Result<bool> {
    use pokabot::schema::blogs::dsl::*;
    use pokabot::models::Blog;
    let blog = blogs.filter(id.eq(post_id)).first::<Blog>(connection)?;

    return Ok(blog.jp_tweeted);
}

fn is_news_tweeted(n_id: &str, connection: &mut SqliteConnection) -> Result<bool> {
    use pokabot::schema::news::dsl::*;
    use pokabot::models::News;
    let n = news.filter(news_id.eq(n_id)).first::<News>(connection)?;

    return Ok(n.jp_tweeted);

}


fn save_blog(post_id: i32, name: &str, posted_at: &NaiveDateTime, connection: &mut SqliteConnection) {
    use pokabot::schema::blogs;

    if is_new_post(post_id, connection) {
        let new_blog = NewBlog {
            id: post_id,
            name: name,
            posted_at: posted_at.clone(), // 修正
            jp_tweeted: true,
            eng_tweeted: false,
        };

         diesel::insert_into(blogs::table)
        .values(&new_blog)
        .execute(connection)
        .expect("Error saving new post");

    } else {
        use pokabot::schema::blogs::dsl::*;
        let target = blogs.filter(pokabot::schema::blogs::id.eq(post_id));

        diesel::update(target)
            .set(pokabot::schema::blogs::jp_tweeted.eq(true))
            .execute(connection).expect("Error updating the post");
       
    }

   
}

fn save_news(news_id: &str, posted_at: &NaiveDateTime, connection: &mut SqliteConnection){
    use pokabot::schema::news;

    if is_new_news(news_id, connection) {
        println!("new news");
        let new_news = NewNews {
            news_id: news_id,
            posted_at: posted_at.clone(), // 修正
            jp_tweeted: true,
            eng_tweeted: false,
        };

         diesel::insert_into(news::table)
        .values(&new_news)
        .execute(connection)
        .expect("Error saving new post");
    } else {
        println!("update news");
        use pokabot::schema::news::dsl::*;
        let target = news.filter(pokabot::schema::news::news_id.eq(news_id));
        
        diesel::update(target)
            .set(pokabot::schema::news::jp_tweeted.eq(true))
            .execute(connection).expect("Error updating the post");
    }

}

fn read_csv(file_path: &str) -> Result<Vec<Person>> {
    let mut rdr = csv::Reader::from_path(file_path)?;
    let mut people = Vec::new();
    for result in rdr.deserialize() {
        let person: Person = result?;
        people.push(person);
    }
    Ok(people)
}

fn generate_hashmap(people: Vec<Person>) -> HashMap<String, Person> {
    let mut map = HashMap::new();
    for person in people {
        map.insert(person.full_name.clone(), person);
    }

    map
}
#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

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

    // data/members.csvからメンバーの情報を読み込む
    let data = read_csv("data/members.csv")?;
    let member_info = generate_hashmap(data);
    
    // print member_info
    for (name, person) in &member_info {
        println!("{}: {}", name, person.nickname);
    }

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
            tweet_until_latest_post(&twitter, &chatgpt, &scraper, connection, &member_info).await?;
            tweet_until_latest_news(&twitter, &chatgpt, &scraper, connection, &member_info).await?;
           

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
    

    

    

}
