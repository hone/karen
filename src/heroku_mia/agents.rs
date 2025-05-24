use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::types::{Choice, Message, Usage};

#[derive(Serialize, Debug)]
pub struct AgentRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens_per_inference_request: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<AgentTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
}

pub struct AgentRequestBuilder {
    model: String,
    messages: Vec<Message>,
    max_tokens_per_inference_request: Option<u32>,
    stop: Option<Vec<String>>,
    temperature: Option<f32>,
    tools: Option<Vec<AgentTool>>,
    top_p: Option<f32>,
}

impl AgentRequestBuilder {
    pub fn new(model: impl Into<String>, messages: Vec<Message>) -> Self {
        AgentRequestBuilder {
            model: model.into(),
            messages,
            max_tokens_per_inference_request: None,
            stop: None,
            temperature: None,
            tools: None,
            top_p: None,
        }
    }

    pub fn max_tokens_per_inference_request(
        mut self,
        max_tokens_per_inference_request: u32,
    ) -> Self {
        self.max_tokens_per_inference_request = Some(max_tokens_per_inference_request);
        self
    }

    pub fn stop(mut self, stop: Vec<String>) -> Self {
        self.stop = Some(stop);
        self
    }

    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn tools(mut self, tools: Vec<AgentTool>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    pub fn build(self) -> AgentRequest {
        AgentRequest {
            model: self.model,
            messages: self.messages,
            max_tokens_per_inference_request: self.max_tokens_per_inference_request,
            stop: self.stop,
            temperature: self.temperature,
            tools: self.tools,
            top_p: self.top_p,
        }
    }
}

impl AgentRequest {
    pub fn builder(model: impl Into<String>, messages: Vec<Message>) -> AgentRequestBuilder {
        AgentRequestBuilder::new(model.into(), messages)
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct AgentTool {
    r#type: AgentToolType,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    runtime_params: Option<HerokuToolRuntimeParams>,
}

impl AgentTool {
    pub fn builder(r#type: AgentToolType, name: impl Into<String>) -> AgentToolBuilder {
        AgentToolBuilder::new(r#type, name.into())
    }
}

pub struct AgentToolBuilder {
    r#type: AgentToolType,
    name: String,
    description: Option<String>,
    runtime_params: Option<HerokuToolRuntimeParams>,
}

impl AgentToolBuilder {
    pub fn new(r#type: AgentToolType, name: String) -> Self {
        AgentToolBuilder {
            r#type,
            name,
            description: None,
            runtime_params: None,
        }
    }

    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn runtime_params(mut self, runtime_params: HerokuToolRuntimeParams) -> Self {
        self.runtime_params = Some(runtime_params);
        self
    }

    pub fn build(self) -> AgentTool {
        AgentTool {
            r#type: self.r#type,
            name: self.name,
            description: self.description,
            runtime_params: self.runtime_params,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum AgentToolType {
    HerokuTool,
    Mcp,
}

#[derive(Serialize, Debug, Clone)]
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

#[derive(Deserialize, PartialEq, Debug)]
pub struct CompletionObject {
    pub id: String,
    pub object: Object,
    pub created: u32,
    pub model: Option<String>,
    pub system_fingerprint: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Deserialize, PartialEq, Debug)]
pub enum Object {
    #[serde(rename = "chat.completion")]
    ChatCompletion,
    #[serde(rename = "tool.completion")]
    ToolCompletion,
}
