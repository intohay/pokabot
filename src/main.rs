use dotenv::dotenv;
use std::env;
use std::collections::HashMap;
use crate::scraper::Scraper;
use crate::twitter::Twitter;
use crate::chatgpt::ChatGPT;
use chrono::Local;
use url::Url;
mod twitter;
mod chatgpt;
mod scraper;
pub mod helper;


fn extract_path(url_or_path: &str) -> String {
    // 相対パスの場合、適当なドメインを追加して完全なURLを作成
    let url = if !url_or_path.starts_with("http://") && !url_or_path.starts_with("https://") {
        format!("https://example.com{}", url_or_path)
    } else {
        url_or_path.to_string()
    };

    let parsed_url = Url::parse(&url).unwrap();

    // URLからパス部分を取得して返す
    parsed_url.path().to_string()
}

async fn tweet_latest_post(twitter: &Twitter, chatgpt: &ChatGPT, scraper: &Scraper){
    let url = scraper.scrape_latest_url().await;
    let previous_url = scraper.load_url();

    if extract_path(&url) != extract_path(&previous_url) {
        tweet_both(&url, &twitter, &chatgpt, &scraper).await;
        scraper.save_url(&url);
    } else {
        println!("[{}] Nothing to scrape", Local::now().format("%Y-%m-%d %Hh%Mm%Ss %Z"));
    }
    
    
}

async fn tweet_until_latest_post(twitter: &Twitter, chatgpt: &ChatGPT, scraper: &Scraper){

    let url = scraper.scrape_latest_url().await;
    let previous_url = scraper.load_url();

    let latest_post_id = helper::extract_post_id(&url).unwrap();
    let previous_post_id = helper::extract_post_id(&previous_url).unwrap();

    for id in previous_post_id..=latest_post_id{
        let target_url = format!("https://www.hinatazaka46.com/s/official/diary/detail/{}?ima=0000&cd=member", id);
        if scraper.page_exists(&target_url).await {
            tweet_both(&target_url, &twitter, &chatgpt, &scraper).await;
            scraper.save_url(&target_url);
        }
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

    let name = scraper.scrape_name(post_url).await;

    let prompt_eng = if name == "ポカ" {
        "---\n Pretend to be the writer of the blog above and make a promotional tweet about it within 150 characters in English briefly."
    } else {
         "---\nRead the idol's blog above and tweet your comment to it casually as one of her fans within 150 characters in English briefly."
    };

    let res_eng = chatgpt.get_response(format!("{}\n {}", blog, prompt_eng)).await.unwrap();
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
    let name = scraper.scrape_name(post_url).await;

    let prompt = if name == "ポカ" {
        "---\n上記のブログを書いた本人になりきって、日本語50字以内で短めに、ブログの宣伝ツイートをしてください。"
    } else {
         "---\n上記のアイドルのブログを読んだ感想を、彼女のファンになったつもりで、カジュアルな口調で、日本語50字以内で短めにツイートしなさい。"
    };

    let res = chatgpt.get_response(format!("{}\n {}", blog, prompt)).await.unwrap();
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

    tweet_until_latest_post(&twitter, &chatgpt, &scraper).await;

}
