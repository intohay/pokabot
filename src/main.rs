use dotenv::dotenv;
use std::env;
use std::collections::HashMap;

mod twitter;
mod chatgpt;
mod scraper;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set.");
    // let consummer_key = env::var("CK").expect("CK must be set.");
    // let consummer_secret = env::var("CS").expect("CS must be set.");
    // let access_token_key = env::var("AT").expect("AT must be set.");
    // let access_token_secret = env::var("AS").expect("AS must be set.");
    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID must be set.");
    let client_secret = env::var("CLIENT_SECRET").expect("CLIENT_SECRET must be set.");


    // let twitter_url = env::var("TWITTER_URL").expect("TWITTER_URL must be set.");

    let twitter = twitter::Twitter::new(
        client_id, client_secret
    );


    

    let chatgpt = chatgpt::ChatGPT::new(api_key);

   

    

    let base = "https://www.hinatazaka46.com";
    let url = "https://www.hinatazaka46.com/s/official/diary/member?ima=0000";
    
    let scraper = scraper::Scraper::new(base, url);

    let url = scraper.scrape_latest_url().await;
    let previous_url = scraper.load_url();

    if url != previous_url {
        scraper.save_url(&url);
        let text = scraper.scrape_text(&url).await;
        println!("{}",text);

        // let pre_prompt = "
        // Read the idol's blog below and tweet your comment to it casually as one of her fans within 50 words in Japanese\n";

        let pre_prompt = "以下のアイドルのブログを読んだ感想を、カジュアルかつキモくオタクのように、日本語40字以内で短めにツイートしなさい。\n";
        let res = chatgpt.get_response(&(pre_prompt.to_owned() + &text)).await.unwrap();

        println!("{}", res.replace("私", "ポカ"));
        let text = res.replace("私", "ポカ");
        // let mut params = HashMap::new();
        // params.insert("text", "hello");

        twitter.post(format!("{}{}{}",text, base, url)).await.unwrap();
        println!("{:?}", res);
        
        // println!("{}",text);
    } else {
        println!("Nothing to scrape");
    }
    
    

 

    // let id = res["data"]["id"].as_str().unwrap();
    // println!("https://twitter.com/scienceboy_jp/status/{}", id);

    // let res = twitter.delete(&format!("tweets/{}", id), HashMap::new()).unwrap();
    // println!("{:?}", res);

    //let res = twitter.post(&format!("statuses/destory/{}", res["id"].as_str().unwrap()), HashMap::new());
    //println!("{:?}", res);

    //let res = twitter.destroy_status(&res.id_str);
    //println!("{}/status/{}", twitter_url, res.id_str);
}