use dotenv::dotenv;
use std::env;
use std::collections::HashMap;
use crate::scraper::Scraper;
use crate::twitter::Twitter;
use crate::chatgpt::ChatGPT;

mod twitter;
mod chatgpt;
mod scraper;



async fn tweet_latest_post(twitter: &Twitter, chatgpt: &ChatGPT, scraper: &Scraper){
    let url = scraper.scrape_latest_url().await;
    let previous_url = scraper.load_url();

    if url.contains(&previous_url) || previous_url.contains(&url) {
        tweet_both(&url, &twitter, &chatgpt, &scraper).await;
        scraper.save_url(&url);
    } else {
        println!("Nothing to scrape");
    }
    
    
}

async fn tweet_both(post_url: &str, twitter: &Twitter, chatgpt: &ChatGPT, scraper: &Scraper){
    tweet_eng_post(post_url, &twitter, &chatgpt, &scraper).await;
    tweet_jp_post(post_url, &twitter, &chatgpt, &scraper).await;
}

async fn tweet_eng_post(post_url: &str, twitter: &Twitter, chatgpt: &ChatGPT, scraper: &Scraper){

    let blog = scraper.scrape_text(post_url).await;

    println!("{}",blog);

    let images = scraper.scrape_images(post_url).await;

    let prompt_eng = "---\nRead the idol's blog above and tweet your comment to it casually as one of her fans within 150 characters in English briefly.";

    let res_eng = chatgpt.get_response(format!("{}\n {}", blog, prompt_eng), 280 - 23).await.unwrap();
    println!("{}", res_eng);

    if post_url.contains("https") {
        twitter.post(format!("{} \n{}",res_eng, post_url), &images).await.unwrap();
    } else {
        twitter.post(format!("{} \n{}{}",res_eng, scraper.get_base(), post_url), &images).await.unwrap();
    }
    
}


async fn tweet_jp_post(post_url: &str, twitter: &Twitter, chatgpt: &ChatGPT, scraper: &Scraper) {
    let blog = scraper.scrape_text(post_url).await;

    println!("{}",blog);

    let images = scraper.scrape_images(post_url).await;

    let prompt = "---\n上記のアイドルのブログを読んだ感想を、カジュアルかつキモくオタクのように、日本語50字以内で短めにツイートしなさい。";

    let res = chatgpt.get_response(format!("{}\n {}", blog, prompt), 140 - 23).await.unwrap();
    println!("{}", res);
    
    if post_url.contains("https") {
        twitter.post(format!("{} \n{}",res, post_url), &images).await.unwrap();
    } else {
        twitter.post(format!("{} \n{}{}",res, scraper.get_base(), post_url), &images).await.unwrap();
    }
    
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set.");
    let consummer_key = env::var("CK").expect("CK must be set.");
    let consummer_secret = env::var("CS").expect("CS must be set.");
    let access_token_key = env::var("AT").expect("AT must be set.");
    let access_token_secret = env::var("AS").expect("AS must be set.");
    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID must be set.");
    let client_secret = env::var("CLIENT_SECRET").expect("CLIENT_SECRET must be set.");
    let user_id = env::var("USER_ID").expect("USER_ID must be set.");

    // let twitter_url = env::var("TWITTER_URL").expect("TWITTER_URL must be set.");

    let twitter = twitter::Twitter::new(
        client_id, client_secret, consummer_key, consummer_secret, access_token_key, access_token_secret, user_id
    );
    let chatgpt = chatgpt::ChatGPT::new(api_key);

    let base = "https://www.hinatazaka46.com";
    let url = "https://www.hinatazaka46.com/s/official/diary/member?ima=0000";
    let scraper = scraper::Scraper::new(base, url);

    tweet_latest_post(&twitter, &chatgpt, &scraper).await;

}