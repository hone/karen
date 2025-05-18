use crate::api::{ApiError, Client, Message, Role};
use std::env;

mod api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let inference_url = env::var("INFERENCE_URL").expect("INFERENCE_URL not set");
    let inference_key = env::var("INFERENCE_KEY").expect("INFERENCE_KEY not set");
    let inference_model_id = env::var("INFERENCE_MODEL_ID").expect("INFERENCE_MODEL_ID not set");

    let client = Client::new(inference_url, inference_key, inference_model_id);

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
            eprintln!("API Error listing MCP servers: {}", e);
        }
    }

    let prompt = "Write a short story about a robot learning to love.";
    let messages = vec![Message {
        role: Role::User,
        content: prompt.to_string(),
    }];

    match client.chat_completion(messages).await {
        Ok(response) => {
            if let Some(choice) = response.choices.get(0) {
                println!("Generated Text:\n{}", choice.message.content);
            } else {
                println!("API call successful, but no choices returned.");
            }
        }
        Err(e) => {
            eprintln!("API Error: {}", e);
        }
    }

    Ok(())
}
