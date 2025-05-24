use commands::query::get_original_message_id;
use serenity::{
    all::{
        CommandDataOptionValue, Context, EventHandler, Interaction, Message as SerenityMessage,
        Ready,
    },
    async_trait,
};
use thiserror::Error;

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

        get_original_message_id(&ctx, &msg).await.unwrap();

        if let Some(referenced_message) = &msg.referenced_message {
            if referenced_message.author.id == ctx.cache.current_user().id {
                tracing::info!("Query Reply");
                let conversations_lock = type_map_keys::ConversationHistory::get(&ctx.data).await;
                let mut conversations = conversations_lock.write().await;

                let original_message_id = commands::query::get_original_message_id(&ctx, &msg)
                    .await
                    .unwrap();
                tracing::info!("Query Reply {original_message_id}");
                if let Some(conversation) = conversations.get_mut(&original_message_id.get()) {
                    tracing::info!("Query Reply {original_message_id}: Found conversation history");
                    conversation.push(HerokuMiaMessage::User {
                        content: msg.content.clone(),
                    });

                    tracing::debug!("Query Reply {original_message_id}: {:?}", conversation);

                    match commands::query::agents_call(
                        &type_map_keys::HerokuMiaClient::get(&ctx.data).await,
                        type_map_keys::AgentTools::get(&ctx.data).await,
                        &type_map_keys::InferenceModelId::get(&ctx.data).await,
                        conversation,
                    )
                    .await
                    {
                        Ok(messages) => {
                            tracing::info!(
                                "Query Reply {original_message_id}: Agents call successful: {}",
                                messages.len()
                            );
                            for message in messages {
                                if !message.is_empty() {
                                    if let Err(e) = msg.reply(&ctx.http, message).await {
                                        tracing::error!(
                                            "Query Reply {original_message_id}: Error sending error message: {:?}",
                                            e
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!(
                                "Query Reply {original_message_id}: Heroku MIA Error during agent call: {:?}",
                                e
                            );
                            if let Err(e) = msg
                                .reply(&ctx.http, "Error communicating with inference service.")
                                .await
                            {
                                tracing::error!(
                                    "Query Reply {original_message_id}: Error sending error message: {:?}",
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
