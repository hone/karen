use serde::Deserialize;

use super::types::Message;

#[derive(Deserialize, Debug)]
pub struct CompletionObject {
    pub id: String,
    pub object: Object,
    pub created: u32,
    pub model: String,
    pub system_fingerprint: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Deserialize, Debug)]
pub enum Object {
    #[serde(rename = "chat.completion")]
    ChatCompletion,
    #[serde(rename = "tool.completion")]
    ToolCompletion,
}

#[derive(Deserialize, Debug)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: String,
}

#[derive(Deserialize, Debug)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
