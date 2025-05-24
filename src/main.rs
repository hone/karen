use crate::heroku_mia::{
    Client,
    agents::{AgentTool, AgentToolType},
};
use serenity::{all::ApplicationId, model::prelude::GuildId, prelude::*};
use std::{collections::HashMap, env, sync::Arc};
use tracing::instrument;
use tracing_subscriber::{self, EnvFilter};

mod discord;
mod heroku_mia;

#[tokio::main]
#[instrument]
async fn main() -> anyhow::Result<()> {
    let filter = if env::var("RUST_LOG").is_ok() {
        EnvFilter::from_default_env()
    } else {
        EnvFilter::new("info")
    };

    tracing_subscriber::fmt().with_env_filter(filter).init();

    let inference_url = env::var("INFERENCE_URL").expect("INFERENCE_URL not set");
    let inference_key = env::var("INFERENCE_KEY").expect("INFERENCE_KEY not set");
    let inference_model_id = env::var("INFERENCE_MODEL_ID").expect("INFERENCE_MODEL_ID not set");

    tracing::info!("Environment variables loaded");
    tracing::info!("INFERENCE_URL: {}", inference_url);
    tracing::info!("INFERENCE_MODEL_ID: {}", inference_model_id);

    let discord_token = env::var("DISCORD_TOKEN").expect("Expected env variable: DISCORD_TOKEN");
    let guild_id = GuildId::new(
        env::var("DISCORD_GUILD_ID")
            .expect("Expected env variable: DISCORD_GUILD_ID")
            .parse()
            .expect("DISCORD_GUILD_ID must be an integer"),
    );
    let application_id = ApplicationId::new(
        env::var("DISCORD_APPLICATION_ID")
            .expect("Expected environment variable: DISCORD_APPLICATION_ID")
            .parse()
            .expect("application id is not a valid id"),
    );

    let conversation_history = Arc::new(RwLock::new(HashMap::new()));

    let heroku_mia_client = Client::new(inference_url, inference_key);
    let tools: Vec<AgentTool>;

    match heroku_mia_client.list_mcp_servers().await {
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
            tracing::error!("Heroku MIA Error listing MCP servers: {e}");
            return Err(e)?;
        }
    }

    let mut discord_client =
        serenity::Client::builder(discord_token, GatewayIntents::GUILD_MESSAGES)
            .event_handler(discord::Handler {})
            .await
            .expect("Error creating discord client.");
    {
        let mut data = discord_client.data.write().await;
        data.insert::<discord::type_map_keys::ConversationHistory>(conversation_history);
        data.insert::<discord::type_map_keys::GuildId>(guild_id);
        data.insert::<discord::type_map_keys::HerokuMiaClient>(heroku_mia_client);
        data.insert::<discord::type_map_keys::InferenceModelId>(inference_model_id);
        data.insert::<discord::type_map_keys::AgentTools>(tools);
    }

    if let Err(err) = discord_client.start().await {
        tracing::error!("Discord Client Error: {err}");
    }

    Ok(())
}
