use serde_json::json;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

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
    

    pub async fn get_response(&self, prompt: &str) -> reqwest::Result<String> {
        let client = reqwest::Client::new();
        let post_body = json!({
            "model" : "gpt-3.5-turbo",
            "messages" : [{"role": "user", "content": prompt}],
            "temperature": 0.7
            });
        let res = client.post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(&self.api_key)
        .header("Content-Type", "application/json")
        .json(&post_body)
        .send()
        .await?
        .text()
        .await?;
        

        let deserialized: Response = serde_json::from_str(&res).unwrap();
        let response= &deserialized.choices[0].message.content;

        

        

         



        Ok(response.to_string())

    }
}
