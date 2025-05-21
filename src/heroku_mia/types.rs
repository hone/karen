use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(tag = "role")]
#[serde(rename_all = "snake_case")]
pub enum Message {
    User {
        content: String,
    },
    Assistant {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        refusal: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<ToolCall>>,
    },
    System {
        content: Value, // Can be string or array
    },
    Tool {
        content: Value,
        tool_call_id: String,
    },
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ToolCall {
    id: String,
    r#type: String, // always "function"
    function: FunctionCall,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct FunctionCall {
    name: String,
    arguments: serde_json::Value,
}

#[derive(Serialize, PartialEq, Debug)]
pub struct ExtendedThinking {
    #[serde(skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    budget_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_reasoning: Option<bool>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: String,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    Empty,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Usage {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}
