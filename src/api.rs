use futures_util::StreamExt;
use reqwest_eventsource::{Event, EventSource};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Network error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("API error: {0}")]
    ApiCallError(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

#[derive(Serialize, Debug)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_thinking: Option<ExtendedThinking>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ChatCompletionTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
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
    function: Function,
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
pub struct Function {
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Deserialize, Debug)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Deserialize, Debug)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    Empty,
}

#[derive(Deserialize, Debug)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Deserialize, Debug)]
pub struct McpServerResponse {
    pub id: String,
    pub app_id: String,
    pub process_type: String,
    pub process_command: String,
    pub created_at: String,
    pub updated_at: String,
    pub tools: Vec<ToolDetails>,
    pub server_status: String,
    pub primitives_status: String,
    pub namespace: String,
}

#[derive(Deserialize, Debug)]
pub struct ToolDetails {
    pub name: String,
    pub namespaced_name: String,
    pub description: String,
    pub input_schema: Value,
    pub annotations: Value,
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

pub struct Client {
    inference_url: String,
    inference_key: String,
    inference_model_id: String,
}

impl Client {
    pub fn new(inference_url: String, inference_key: String, inference_model_id: String) -> Self {
        Self {
            inference_url,
            inference_key,
            inference_model_id,
        }
    }

    pub async fn agents_call(
        &self,
        messages: Vec<Message>,
    ) -> Result<Vec<CompletionObject>, ApiError> {
        let client = reqwest::Client::new();
        let request_builder = client
            .get(format!("{}/v1/agents/heroku", self.inference_url))
            .header("Authorization", format!("Bearer {}", self.inference_key))
            .header("Content-Type", "application/json");
        let mut event_source = EventSource::new(request_builder).unwrap();

        let mut messages = Vec::new();
        while let Some(event) = event_source.next().await {
            match event {
                Ok(Event::Open) => (),
                Ok(Event::Message(message)) => {
                    if message.event == "message" {
                        messages.push(serde_json::from_str::<CompletionObject>(&message.data)?);
                    } else if message.event == "done" {
                        event_source.close();
                    }
                }
                Err(err) => event_source.close(),
            }
        }

        Ok(messages)
    }

    pub async fn chat_completion(
        &self,
        messages: Vec<Message>,
    ) -> Result<ChatCompletionResponse, ApiError> {
        let request_body = ChatCompletionRequest {
            model: self.inference_model_id.clone(),
            messages,
            extended_thinking: None,
            temperature: None,
            max_tokens: None,
            stop: None,
            stream: Some(false), // Assuming stream should be false by default for non-streaming chat completion
            tools: None,
            tool_choice: None,
            top_p: None,
        };

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/v1/chat/completions", self.inference_url))
            .header("Authorization", format!("Bearer {}", self.inference_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let response_body = response.json::<ChatCompletionResponse>().await?;
            Ok(response_body)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown API error".to_string());
            Err(ApiError::ApiCallError(error_text))
        }
    }

    pub async fn list_mcp_servers(&self) -> Result<Vec<McpServerResponse>, ApiError> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/v1/mcp/servers", self.inference_url))
            .header("Authorization", format!("Bearer {}", self.inference_key))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if response.status().is_success() {
            let response_body = response.json::<Vec<McpServerResponse>>().await?;
            Ok(response_body)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown API error".to_string());
            Err(ApiError::ApiCallError(error_text))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tool_choice_serialize_none() {
        let tool_choice = ToolChoice::None;
        let serialized = serde_json::to_string(&tool_choice).unwrap();
        assert_eq!(serialized, "\"none\"");
    }

    #[test]
    fn test_tool_choice_serialize_auto() {
        let tool_choice = ToolChoice::Auto;
        let serialized = serde_json::to_string(&tool_choice).unwrap();
        assert_eq!(serialized, "\"auto\"");
    }

    #[test]
    fn test_tool_choice_serialize_required() {
        let tool_choice = ToolChoice::Required;
        let serialized = serde_json::to_string(&tool_choice).unwrap();
        assert_eq!(serialized, "\"required\"");
    }

    #[test]
    fn test_tool_choice_serialize_tool() {
        let tool = ChatCompletionTool {
            r#type: "function".to_string(),
            function: Function {
                name: "my_function".to_string(),
                description: Some("A test function".to_string()),
                parameters: None,
            },
        };
        let tool_choice = ToolChoice::Tool(tool);
        let serialized = serde_json::to_string(&tool_choice).unwrap();
        let expected = json!({
            "type": "function",
            "function": {
                "name": "my_function",
                "description": "A test function"
            }
        })
        .to_string();
        let serialized_value: Value = serde_json::from_str(&serialized).unwrap();
        let expected_value: Value = serde_json::from_str(&expected).unwrap();
        assert_eq!(serialized_value, expected_value);
    }
}
