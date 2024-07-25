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

#[derive(Serialize, Deserialize, Debug)]
struct Output {
    output: String
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
            "model" : "gpt-4o-mini",
            "messages" : [
                {
                    "role": "system",
                    "content": "出力はすべてJSON形式で返してください。"
                },
                {
                    "role": "user", 
                    "content": prompt
                }],
            "temperature": 0.7,
            "response_format": {"type": "json_object"}
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
        
        // interpret response as json
        let output: Output = serde_json::from_str(response.as_str())?;
        


        Ok(output.output)

    }
}
