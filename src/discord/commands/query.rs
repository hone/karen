use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateInteractionResponseMessage, Message as SerenityMessage, MessageId,
};

use crate::{
    discord::{DiscordError, type_map_keys},
    heroku_mia::{
        Client,
        agents::{AgentRequest, AgentTool},
        types::Message as HerokuMiaMessage,
    },
};

pub async fn run(
    ctx: &Context,
    command: &CommandInteraction,
    prompt: &str,
) -> Result<(), serenity::Error> {
    command
        .create_response(
            &ctx.http,
            serenity::all::CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("Starting a new conversation. Reply to this message to continue."),
            ),
        )
        .await?;
    let mut last_message = command.get_response(&ctx.http).await?;
    let conversation_key = last_message.id.get();
    tracing::info!("Query {conversation_key}...");

    let mut messages = bootstrap_messages();
    messages.push(HerokuMiaMessage::User {
        content: prompt.to_string(),
    });

    match agents_call(
        &type_map_keys::HerokuMiaClient::get(&ctx.data).await,
        type_map_keys::AgentTools::get(&ctx.data).await,
        &type_map_keys::InferenceModelId::get(&ctx.data).await,
        &mut messages,
    )
    .await
    {
        Ok(messages) => {
            tracing::info!("Query {conversation_key}: Message Count {}", messages.len());
            for message in messages {
                match last_message.reply(&ctx.http, message).await {
                    Ok(message) => last_message = message,
                    Err(e) => tracing::error!("Error sending error message: {:?}", e),
                }
            }
        }
        Err(e) => {
            tracing::error!("Heroku MIA Error during agent call: {:?}", e);
            if let Err(e) = last_message
                .reply(&ctx.http, "Error communicating with inference service.")
                .await
            {
                tracing::error!("Error sending error message: {:?}", e);
            }
        }
    }

    let msg = command.get_response(&ctx.http).await?;

    {
        let conversations_lock = type_map_keys::ConversationHistory::get(&ctx.data).await;
        let mut conversations = conversations_lock.write().await;
        conversations.insert(conversation_key, messages);
    }

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("query")
        .description("Start a conversation with the Marvel Champions Agent")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "prompt", "Prompt for the Agent")
                .required(true),
        )
}

pub(crate) async fn agents_call(
    client: &Client,
    tools: Vec<AgentTool>,
    inference_model_id: &str,
    conversation: &mut Vec<HerokuMiaMessage>,
) -> Result<Vec<String>, DiscordError> {
    let request = AgentRequest::builder(inference_model_id, conversation.clone())
        .tools(tools)
        .build();
    let mut messages = Vec::new();

    for message in client.agents_call(&request).await? {
        if let Some(choice) = message.choices.get(0) {
            conversation.push(choice.message.clone());
            if let HerokuMiaMessage::Assistant { content, .. } = &choice.message {
                messages.push(content.clone());
            }
        }
    }

    Ok(messages)
}

pub(crate) async fn get_original_message_id(
    ctx: &Context,
    msg: &SerenityMessage,
) -> Result<MessageId, serenity::Error> {
    let mut current_msg = msg.clone();
    while let Some(reference_message) = &current_msg.referenced_message {
        current_msg = msg
            .channel_id
            .message(&ctx.http, reference_message.id)
            .await?;
    }

    Ok(current_msg.id)
}

pub(crate) fn bootstrap_messages() -> Vec<HerokuMiaMessage> {
    vec![HerokuMiaMessage::System { content: serde_json::Value::String("You are a helpful expert on the Marvel Champions card game with access to all the card, pack, and set data. When querying for data stick to only official cards. Hero sets or signature sets are identified by their SetId.".to_string()) }]
}
