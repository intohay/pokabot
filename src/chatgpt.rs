use serde_json::json;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::time;

use crate::helper::is_within_twitter_limit;

pub struct ChatGPT {
    api_key: String
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    choices : Vec<Choices>
}
#[derive(Serialize, Deserialize, Debug)]
struct Choices {
    message: Message
}
#[derive(Serialize, Deserialize, Debug)]
struct Message {
    content: String
}
impl ChatGPT {
    pub fn new(api_key: String) -> ChatGPT {
        ChatGPT {
            api_key
        }
    }
    

    pub async fn get_response(&self, prompt: String) -> reqwest::Result<String> {
        let client = reqwest::Client::new();
        let post_body = json!({
            "model" : "gpt-3.5-turbo",
            "messages" : [{"role": "user", "content": prompt}],
            "temperature": 0.7,
            });

        let mut result = String::new();
        let mut found = false;
        loop {

        
            let res = client.post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .header("Content-Type", "application/json")
            .json(&post_body)
            .send()
            .await?
            .text()
            .await?;
            
            println!("{}",res);
            let deserialized: Response = serde_json::from_str(&res).unwrap();
            let response: Vec<_> = deserialized.choices.iter().map(|x| &x.message.content).collect();

            
            
            for r in response{
                println!("{}: {}", r.chars().count(), r);
                if is_within_twitter_limit(r) {
                    result = r.to_string();
                    found = true;
                    break;
                }
            }

            if found {
                break;
            }

            time::sleep(time::Duration::from_millis(1000)).await;

          
            
        
        }



        
        Ok(result)

    }
}
