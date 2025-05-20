use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::types::{Choice, Message, Usage};

#[derive(Serialize, Debug)]
pub struct AgentRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens_per_inference_request: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<AgentTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AgentTool {
    r#type: AgentToolType,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    runtime_params: Option<HerokuToolRuntimeParams>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AgentToolType {
    HerokuTool,
    Mcp,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HerokuToolRuntimeParams {
    pub target_app_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dyno_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_seconds: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_calls: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_params: Option<Value>,
}

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
