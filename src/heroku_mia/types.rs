use serde::{
    ser::Serializer,
    {Deserialize, Serialize},
};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

#[derive(Serialize, Deserialize, Debug)]
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
        content: serde_json::Value, // Can be string or array
    },
    Tool {
        content: serde_json::Value,
        tool_call_id: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ToolCall {
    id: String,
    r#type: String, // always "function"
    function: FunctionCall,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FunctionCall {
    name: String,
    arguments: serde_json::Value,
}

#[derive(Serialize, Debug)]
pub struct ExtendedThinking {
    #[serde(skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    budget_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_reasoning: Option<bool>,
}

#[derive(Debug)]
pub enum ToolChoice {
    None,
    Auto,
    Required,
    Tool(ChatCompletionTool),
}

impl Serialize for ToolChoice {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ToolChoice::None => serializer.serialize_str("none"),
            ToolChoice::Auto => serializer.serialize_str("auto"),
            ToolChoice::Required => serializer.serialize_str("required"),
            ToolChoice::Tool(tool) => tool.serialize(serializer),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct ChatCompletionTool {
    r#type: String, // always "function"
    function: FunctionDefinition,
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
pub struct FunctionDefinition {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<FunctionParameters>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FunctionParameters {
    pub r#type: String,
    pub properties: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}
