use heroku_mia::agents::AgentRequest;

use crate::heroku_mia::{
    Client,
    agents::{AgentTool, AgentToolType},
    chat_completion::ChatCompletionRequest,
    types::Message,
};
use std::env;
use tracing_subscriber;

mod heroku_mia;

use tracing::{instrument, level_filters::LevelFilter};

#[tokio::main]
#[instrument]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(LevelFilter::INFO.into()),
        )
        .init();

    let inference_url = env::var("INFERENCE_URL").expect("INFERENCE_URL not set");
    let inference_key = env::var("INFERENCE_KEY").expect("INFERENCE_KEY not set");
    let inference_model_id = env::var("INFERENCE_MODEL_ID").expect("INFERENCE_MODEL_ID not set");

    tracing::info!("Environment variables loaded");
    tracing::info!("INFERENCE_URL: {}", inference_url);
    tracing::info!("INFERENCE_MODEL_ID: {}", inference_model_id);

    let client = Client::new(inference_url, inference_key);

    let tools: Vec<AgentTool>;

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
            tracing::error!("Heroku MIA Error listing MCP servers: {}", e);
            return Err(e)?;
        }
    }

    {
        let messages = vec![Message::User {
            content: "What all the heroes with 14 or greater hit points?".to_string(),
        }];
        let request = AgentRequest::builder(&inference_model_id, messages)
            .tools(tools)
            .build();
        tracing::debug!("Making agent call with request: {:?}", request);
        match client.agents_call(&request).await {
            Ok(response) => {
                tracing::info!("Agent call successful");
                for message in response {
                    if let Some(choice) = message.choices.get(0) {
                        match &choice.message {
                            Message::Assistant { content, .. } => {
                                tracing::info!("Agent response (Assistant): {}", content);
                            }
                            Message::System { content, .. } => {
                                tracing::info!("Agent response (System): {}", content);
                            }
                            Message::Tool { content, .. } => {
                                tracing::debug!("Agent response (Tool): {}", content);
                            }
                            Message::User { content, .. } => {
                                tracing::info!("Agent response (User): {}", content);
                            }
                        }
                    } else {
                        tracing::warn!("Agent call successful, but no choices returned.");
                    }
                }
            }
            Err(e) => {
                tracing::error!("Heroku MIA Error during agent call: {:?}", e);
            }
        }
    }

    {
        let prompt = "Write a short story about a robot learning to love.";
        let messages = vec![Message::User {
            content: prompt.to_string(),
        }];

        let request = ChatCompletionRequest::builder(&inference_model_id, messages).build();
        tracing::debug!("Making chat completion call with request: {:?}", request);

        match client.chat_completion(&request).await {
            Ok(response) => {
                tracing::info!("Chat completion call successful");
                if let Some(choice) = response.choices.get(0) {
                    match &choice.message {
                        Message::Assistant { content, .. } => {
                            tracing::info!("Chat completion response (Assistant): {}", content);
                        }
                        Message::System { content, .. } => {
                            tracing::info!("Chat completion response (System): {}", content);
                        }
                        Message::Tool { content, .. } => {
                            tracing::info!("Chat completion response (Tool): {}", content);
                        }
                        Message::User { content, .. } => {
                            tracing::info!("Chat completion response (User): {}", content);
                        }
                    }
                } else {
                    tracing::warn!("Chat completion call successful, but no choices returned.");
                }
            }
            Err(e) => {
                tracing::error!("Heroku MIA Error during chat completion call: {:?}", e);
            }
        }
    }

    Ok(())
}
