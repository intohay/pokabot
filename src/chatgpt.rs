use serde_json::json;
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
    

    pub async fn get_response(&self, prompt: String) -> anyhow::Result<String> {
        let client = reqwest::Client::new();
        let post_body = json!({
            "model" : "gpt-4-1106-preview",
            "messages" : [{"role": "user", "content": prompt}],
            "temperature": 0.7,
            });

    
    
        let res = client.post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(&self.api_key)
        .header("Content-Type", "application/json")
        .json(&post_body)
        .send()
        .await?
        .text()
        .await?;
        
        println!("{}",res);
        let deserialized: Response = serde_json::from_str(&res)?;
        let response = deserialized.choices[0].message.content.clone();


        Ok(response)

    }
}
