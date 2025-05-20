use heroku_mia::agents::AgentRequest;

use crate::heroku_mia::{
    Client,
    agents::{AgentTool, AgentToolType},
    chat_completion::ChatCompletionRequest,
    types::Message,
};
use std::env;

mod heroku_mia;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let inference_url = env::var("INFERENCE_URL").expect("INFERENCE_URL not set");
    let inference_key = env::var("INFERENCE_KEY").expect("INFERENCE_KEY not set");
    let inference_model_id = env::var("INFERENCE_MODEL_ID").expect("INFERENCE_MODEL_ID not set");

    let client = Client::new(inference_url, inference_key);

    let mut tools: Vec<AgentTool> = Vec::new();

    match client.list_mcp_servers().await {
        Ok(servers) => {
            tools = servers
                .into_iter()
                .flat_map(|server| {
                    server.tools.into_iter().map(|tool| {
                        AgentTool::builder(AgentToolType::Mcp, tool.namespaced_name).build()
                    })
                })
                .collect();
        }
        Err(e) => {
            panic!("Heroku MIA Error listing MCP servers: {}", e);
        }
    }

    {
        let messages = vec![Message::User {
            content: "What all the heroes with 14 or greater hit points?".to_string(),
        }];
        let request = AgentRequest::builder(&inference_model_id, messages).build();
        match client.agents_call(&request).await {
            Ok(response) => {
                for message in response {
                    if let Some(choice) = message.choices.get(0) {
                        match &choice.message {
                            Message::Assistant { content, .. } => {
                                println!("Generated Text:\n{}", content)
                            }
                            Message::System { content, .. } => {
                                println!("Generated Text:\n{}", content)
                            }
                            Message::Tool { content, .. } => {
                                println!("Generated Text:\n{}", content)
                            }
                            Message::User { content, .. } => {
                                println!("Generated Texet:\n{}", content)
                            }
                        }
                    } else {
                        println!("API call successful, but no choices returned.");
                    }
                }
            }
            Err(e) => {
                eprintln!("Heroku MIA Error: {}", e);
            }
        }
    }

    {
        let prompt = "Write a short story about a robot learning to love.";
        let messages = vec![Message::User {
            content: prompt.to_string(),
        }];

        let request = ChatCompletionRequest::builder(&inference_model_id, messages).build();

        match client.chat_completion(&request).await {
            Ok(response) => {
                if let Some(choice) = response.choices.get(0) {
                    match &choice.message {
                        Message::Assistant { content, .. } => {
                            println!("Generated Text:\n{}", content)
                        }
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
    }

    Ok(())
}
