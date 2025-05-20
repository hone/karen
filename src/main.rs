use crate::heroku_mia::{Client, chat_completion::ChatCompletionRequest, types::Message};
use std::env;

mod heroku_mia;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let inference_url = env::var("INFERENCE_URL").expect("INFERENCE_URL not set");
    let inference_key = env::var("INFERENCE_KEY").expect("INFERENCE_KEY not set");
    let inference_model_id = env::var("INFERENCE_MODEL_ID").expect("INFERENCE_MODEL_ID not set");

    let client = Client::new(inference_url, inference_key);

    match client.list_mcp_servers().await {
        Ok(servers) => {
            println!("Available MCP Tools:");
            for server in servers {
                println!("Server: {}", server.namespace);
                for tool in server.tools {
                    println!("  - {}", tool.namespaced_name);
                }
            }
        }
        Err(e) => {
            eprintln!("Heroku MIA Error listing MCP servers: {}", e);
        }
    }

    let prompt = "Write a short story about a robot learning to love.";
    let messages = vec![Message::User {
        content: prompt.to_string(),
    }];

    let request = ChatCompletionRequest::builder(inference_model_id, messages).build();

    match client.chat_completion(&request).await {
        Ok(response) => {
            if let Some(choice) = response.choices.get(0) {
                match &choice.message {
                    Message::Assistant { content, .. } => println!("Generated Text:\n{}", content),
                    Message::System { content, .. } => println!("Generated Text:\n{}", content),
                    Message::Tool { content, .. } => println!("Generated Text:\n{}", content),
                    Message::User { content, .. } => println!("Generated Texet:\n{}", content),
                }
            } else {
                println!("API call successful, but no choices returned.");
            }
        }
        Err(e) => {
            eprintln!("Heroku MIA Error: {}", e);
        }
    }

    Ok(())
}
