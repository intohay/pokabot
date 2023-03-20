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
use crate::scraper::News;


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

async fn tweet_news_in_both_lang(twitter: &Twitter, chatgpt: &ChatGPT, scraper: &Scraper){
    let jp_prompt = "以下は、日向坂46というアイドルグループに関するニュースです。ファンになったつもりで、ニュースの内容を要約し、カジュアルな日本語60字以内で短めにツイートしなさい。";
    let eng_prompt = "Below is news on Hinatazaka46, Japanese idol group. Tweet summary of the news casually within 150 words in English.";

    let news_list = scraper.scrape_until_latest_news().await;

    tweet_until_latest_news(eng_prompt,&news_list, twitter, chatgpt, scraper).await;
    tweet_until_latest_news(jp_prompt, &news_list, twitter, chatgpt, scraper).await;

}
async fn tweet_until_latest_news(prompt: &str, news_list: &Vec<News>, twitter: &Twitter, chatgpt: &ChatGPT, scraper: &Scraper) {

    
    

    for news in news_list {
        loop {
            let body = chatgpt.get_response(format!("{}\n {}", prompt, news.get_body())).await.unwrap();
            let news_url = news.get_url();
            let images = news.get_images();

            let text = if news_url.contains("https") {
                format!("{} \n{}", body, news_url)
            } else {
                format!("{} \n{}{}",body, scraper.get_base(), news_url)
            };

            if helper::is_within_twitter_limit(&text) {
                twitter.post(&text, &images).await.unwrap();
                break;
            }
        }
    }

}


async fn tweet_latest_post(twitter: &Twitter, chatgpt: &ChatGPT, scraper: &Scraper){
    let url = scraper.scrape_latest_url().await;
    let previous_url = scraper.load_url("last_blog.txt");

    if extract_path(&url) != extract_path(&previous_url) {
        tweet_blog_in_both_lang(&url, &twitter, &chatgpt, &scraper).await;
        scraper.save_url(&url, "last_blog.txt");
    } else {
        println!("[{}] Nothing to scrape", Local::now().format("%Y-%m-%d %Hh%Mm%Ss %Z"));
    }
    
    
}

async fn tweet_until_latest_post(twitter: &Twitter, chatgpt: &ChatGPT, scraper: &Scraper){

    let url = scraper.scrape_latest_url().await;
    let previous_url = scraper.load_url("last_blog.txt");


    let latest_post_id = scraper.extract_post_id(&url).unwrap();
    let previous_post_id = scraper.extract_post_id(&previous_url).unwrap();

    if latest_post_id == previous_post_id {
        return;
    }


    for id in (previous_post_id+1)..=(latest_post_id+5){
        let target_url = format!("https://www.hinatazaka46.com/s/official/diary/detail/{}?ima=0000&cd=member", id);
        if scraper.page_exists(&target_url).await {
            tweet_blog_in_both_lang(&target_url, &twitter, &chatgpt, &scraper).await;
            scraper.save_url(&target_url, "last_blog.txt");
        }
    }
    

}



async fn tweet_blog_in_both_lang(post_url: &str, twitter: &Twitter, chatgpt: &ChatGPT, scraper: &Scraper){
    let name = scraper.scrape_name(post_url).await;

    let eng_prompt = if name == "ポカ" {
        "---\n Act as the writer of the blog below and make a promotional tweet about it within 150 characters in English briefly."
    } else {
         "---\nRead the idol's blog below and tweet your comment to it casually as one of her fans within 150 characters in English briefly."
    };

    let jp_prompt = if name == "ポカ" {
        "---\n以下のブログを書いた本人になりきって、日本語50字以内で短めに、ブログの宣伝ツイートをしてください。"
    } else {
         "---\n以下のアイドルのブログを読んだ感想を、彼女のファンになったつもりで、カジュアルな口調で、日本語50字以内で短めにツイートしなさい。"
    };

    tweet_blog(post_url, eng_prompt, twitter, chatgpt, scraper).await;
    tweet_blog(post_url, jp_prompt, twitter, chatgpt, scraper).await;
    


}


async fn tweet_blog(post_url: &str, prompt: &str, twitter: &Twitter, chatgpt: &ChatGPT, scraper: &Scraper){

    let blog = scraper.scrape_text(post_url).await;

    println!("{}",blog);

    let images = scraper.scrape_blog_images(post_url).await;


    loop {
        let body = chatgpt.get_response(format!("{}\n {}",prompt, blog )).await.unwrap();

        let text = if post_url.contains("https") {
            format!("{} \n{}", body, post_url)
        } else {
            format!("{} \n{}{}",body, scraper.get_base(), post_url)
        };

        if helper::is_within_twitter_limit(&text) {
             twitter.post(&text, &images).await.unwrap();
             break;
        }
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
    let blog_url = "https://www.hinatazaka46.com/s/official/diary/member?ima=0000";
    let news_url = "https://www.hinatazaka46.com/s/official/news/list";
    let scraper = scraper::Scraper::new(base, blog_url, news_url);

    tweet_until_latest_post(&twitter, &chatgpt, &scraper).await;
    tweet_news_in_both_lang(&twitter, &chatgpt, &scraper).await;

}
