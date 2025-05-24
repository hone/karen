use futures::{Stream, StreamExt};
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateInteractionResponseMessage, Message as SerenityMessage, MessageId,
};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    discord::{DiscordError, type_map_keys},
    heroku_mia::{
        Client,
        agents::{AgentRequest, AgentTool},
        types::Message as HerokuMiaMessage,
    },
};

const MAX_DISCORD_MESSAGE_LENGTH: usize = 2000;
const MAX_CONVERSATION_MESSAGES: usize = 10; // Keep last 10 messages (5 turns)
const MAX_TOOL_OUTPUT_CHARS: usize = 1000; // Max characters for tool output summary

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

    let messages = bootstrap_messages();
    let mut initial_messages = messages;
    initial_messages.push(HerokuMiaMessage::User {
        content: prompt.to_string(),
    });

    let conversation_arc = Arc::new(Mutex::new(initial_messages));

    let mut stream = agents_call(
        &type_map_keys::HerokuMiaClient::get(&ctx.data).await,
        type_map_keys::AgentTools::get(&ctx.data).await,
        &type_map_keys::InferenceModelId::get(&ctx.data).await,
        Arc::clone(&conversation_arc),
    )
    .await;

    while let Some(message_result) = stream.next().await {
        match message_result {
            Ok(message) => {
                tracing::info!("Query {conversation_key}: Received streamed message");
                if !message.is_empty() {
                    let chunks = split_message_into_chunks(&message);
                    for chunk in chunks {
                        match last_message.reply(&ctx.http, chunk).await {
                            Ok(message) => last_message = message,
                            Err(e) => tracing::error!("Error sending error message: {:?}", e),
                        }
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
                break;
            }
        }
    }

    {
        let conversations_lock = type_map_keys::ConversationHistory::get(&ctx.data).await;
        let mut conversations = conversations_lock.write().await;
        let final_conversation = conversation_arc.lock().await;
        conversations.insert(conversation_key, final_conversation.clone());
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
    conversation: Arc<Mutex<Vec<HerokuMiaMessage>>>,
) -> Pin<Box<dyn Stream<Item = Result<String, DiscordError>> + Send>> {
    let initial_conversation_for_request: Vec<HerokuMiaMessage>;
    {
        let mut conv_guard = conversation.lock().await;
        prune_conversation_history(
            &mut conv_guard,
            MAX_CONVERSATION_MESSAGES,
            MAX_TOOL_OUTPUT_CHARS,
        );
        initial_conversation_for_request = conv_guard.clone();
    }

    let request = AgentRequest::builder(inference_model_id, initial_conversation_for_request)
        .max_tokens_per_inference_request(8192)
        .tools(tools)
        .build();

    let client_stream = client.agents_call(&request).await;

    Box::pin(client_stream.filter_map(move |message_result| {
        let conversation_clone_for_move = Arc::clone(&conversation);
        async move {
            match message_result {
                Ok(message) => {
                    if let Some(choice) = message.choices.get(0) {
                        let mut conv_guard = conversation_clone_for_move.lock().await;
                        conv_guard.push(choice.message.clone());
                        if let HerokuMiaMessage::Assistant { content, .. } = &choice.message {
                            Some(Ok(content.clone()))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                Err(e) => Some(Err(DiscordError::HerokuMiaError(e))),
            }
        }
    }))
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
    vec![HerokuMiaMessage::System {
        content: serde_json::Value::String("You are a helpful expert on the Marvel Champions card game with access to all the card, pack, and set data. When querying for data stick to only official cards. Hero sets or signature sets are identified by their SetId.".to_string()),
    }]
}

fn prune_conversation_history(
    messages: &mut Vec<HerokuMiaMessage>,
    max_messages: usize,
    max_tool_output_chars: usize,
) {
    // message pruning
    if messages.len() > max_messages {
        let mut start_index = 0;
        if let Some(first_msg) = messages.first() {
            if let HerokuMiaMessage::System { .. } = first_msg {
                start_index = 1; // Keep the system message
            }
        }
        let messages_to_remove = messages.len() - max_messages;
        if messages_to_remove > 0 {
            let actual_remove_count =
                (messages_to_remove as isize - start_index as isize).max(0) as usize;
            if actual_remove_count > 0 {
                messages.drain(start_index..start_index + actual_remove_count);
            }
        }
    }

    // handle large tool output content pruning
    for message in messages.iter_mut() {
        if let HerokuMiaMessage::Tool { content, .. } = message {
            let content_string = content.to_string();
            if content_string.len() > max_tool_output_chars {
                let item_count = content.as_array().map_or(0, |arr| arr.len());
                let summary_string = format!(
                    "{{Tool output: {} items. Full details omitted to save context tokens. Re-run the tool if more details are needed.}}",
                    item_count
                );
                *content = serde_json::Value::String(summary_string);
            }
        }
    }
}

fn split_message_into_chunks(message: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    let mut in_code_block = false;

    for line in message.lines() {
        if line.trim().starts_with("```") {
            in_code_block = !in_code_block;
        }

        // +1 for newline
        if current_chunk.len() + line.len() + 1 > MAX_DISCORD_MESSAGE_LENGTH {
            if !current_chunk.is_empty() {
                chunks.push(current_chunk.trim().to_string());
                current_chunk = String::new();
            }

            if line.len() > MAX_DISCORD_MESSAGE_LENGTH {
                let mut remaining_line = line;
                while remaining_line.len() > MAX_DISCORD_MESSAGE_LENGTH {
                    let (part, rest) = remaining_line.split_at(MAX_DISCORD_MESSAGE_LENGTH);
                    chunks.push(part.to_string());
                    remaining_line = rest;
                }
                current_chunk.push_str(remaining_line);
            } else {
                current_chunk.push_str(line);
            }
        } else {
            current_chunk.push_str(line);
        }
        current_chunk.push('\n');
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk.trim().to_string());
    }

    chunks
}
