use commands::query::get_original_message_id;
use futures::StreamExt;
use serenity::{
    all::{
        CommandDataOptionValue, Context, EventHandler, Interaction, Message as SerenityMessage,
        Ready,
    },
    async_trait,
};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

use crate::heroku_mia::{self, types::Message as HerokuMiaMessage};

mod commands;
pub(crate) mod type_map_keys;

#[derive(Error, Debug)]
pub(crate) enum DiscordError {
    #[error("No Such Command {0}")]
    NoSuchCommand(String),
    #[error("Invalid argument")]
    InvalidArgument,
    #[error("Serenity Error {0}")]
    SerinityError(#[from] serenity::Error),
    #[error("Heroku MIA Error {0}")]
    HerokuMiaError(#[from] heroku_mia::client::HerokuMiaError),
}

pub(crate) struct Handler {}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("{} is connected!", ready.user.name);

        let guild_id = type_map_keys::GuildId::get(&ctx.data).await;

        let commands = guild_id
            .set_commands(&ctx.http, vec![commands::query::register()])
            .await;

        tracing::info!(
            "The following guild slash commands have been registered: {}",
            commands
                .as_ref()
                .unwrap()
                .iter()
                .map(|command| command.name.as_str())
                .collect::<Vec<&str>>()
                .join(", ")
        );

        if let Err(e) = commands {
            tracing::error!("Failed to register slash command: {:?}", e);
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let result = match command.data.name.as_str() {
                "query" => {
                    if let Some(option) = command.data.options.get(0) {
                        if let CommandDataOptionValue::String(prompt) = &option.value {
                            commands::query::run(&ctx, &command, &prompt)
                                .await
                                .map_err(|e| DiscordError::SerinityError(e))
                        } else {
                            Err(DiscordError::InvalidArgument)
                        }
                    } else {
                        Err(DiscordError::InvalidArgument)
                    }
                }
                _ => Err(DiscordError::NoSuchCommand(
                    command.data.name.as_str().to_string(),
                )),
            };

            if let Err(err) = result {
                tracing::error!("Error with the slash command: {}", err);
            }
        }
    }

    async fn message(&self, ctx: Context, msg: SerenityMessage) {
        if msg.author.bot {
            return;
        }

        if let Some(referenced_message) = &msg.referenced_message {
            if referenced_message.author.id == ctx.cache.current_user().id {
                tracing::info!("Query Reply");
                let conversations_lock = type_map_keys::ConversationHistory::get(&ctx.data).await;
                let mut conversations = conversations_lock.write().await;
                let original_message_id = get_original_message_id(&ctx, &msg).await.unwrap();

                tracing::info!("Query Reply {original_message_id}");
                if let Some(conversation_vec) = conversations.get_mut(&original_message_id.get()) {
                    tracing::info!("Query Reply {original_message_id}: Found conversation history");
                    conversation_vec.push(HerokuMiaMessage::User {
                        content: msg.content.clone(),
                    });

                    let conversation_arc = Arc::new(Mutex::new(conversation_vec.clone()));
                    tracing::debug!("Query Reply {original_message_id}: {:?}", conversation_arc);

                    let mut stream = commands::query::agents_call(
                        &type_map_keys::HerokuMiaClient::get(&ctx.data).await,
                        type_map_keys::AgentTools::get(&ctx.data).await,
                        &type_map_keys::InferenceModelId::get(&ctx.data).await,
                        Arc::clone(&conversation_arc),
                    )
                    .await;

                    let mut last_message = msg;

                    while let Some(message_result) = stream.next().await {
                        match message_result {
                            Ok(message) => {
                                tracing::info!(
                                    "Query Reply {original_message_id}: Received streamed message"
                                );
                                if !message.is_empty() {
                                    match last_message.reply(&ctx.http, message).await {
                                        Ok(new_msg) => last_message = new_msg,
                                        Err(e) => tracing::error!(
                                            "Query Reply {original_message_id}: Error sending message: {:?}",
                                            e
                                        ),
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Query Reply {original_message_id}: Heroku MIA Error during agent call: {:?}",
                                    e
                                );
                                if let Err(e) = last_message
                                    .reply(&ctx.http, "Error communicating with inference service.")
                                    .await
                                {
                                    tracing::error!(
                                        "Query Reply {original_message_id}: Error sending error message: {:?}",
                                        e
                                    );
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}
