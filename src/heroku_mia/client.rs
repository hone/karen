use futures_util::StreamExt;
use reqwest::Client as ReqwestClient;
use thiserror::Error;

use super::{
    agents::CompletionObject,
    chat_completion::{ChatCompletionRequest, ChatCompletionResponse},
    mcp_servers::McpServerResponse,
    types::Message,
};

#[derive(Error, Debug)]
pub enum HerokuMiaError {
    #[error("Network error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("API error: {0}")]
    ApiCallError(String),
}

pub struct Client {
    inference_url: String,
    inference_key: String,
    inference_model_id: String,
    reqwest_client: ReqwestClient,
}

impl Client {
    pub fn new(inference_url: String, inference_key: String, inference_model_id: String) -> Self {
        Self {
            inference_url,
            inference_key,
            inference_model_id,
            reqwest_client: ReqwestClient::new(),
        }
    }

    pub async fn agents_call(
        &self,
        messages: Vec<Message>,
    ) -> Result<Vec<CompletionObject>, HerokuMiaError> {
        let request_builder = self
            .reqwest_client
            .get(format!("{}/v1/agents/heroku", self.inference_url))
            .header("Authorization", format!("Bearer {}", self.inference_key))
            .header("Content-Type", "application/json");
        let mut event_source = reqwest_eventsource::EventSource::new(request_builder).unwrap();

        let mut messages = Vec::new();
        while let Some(event) = event_source.next().await {
            match event {
                Ok(reqwest_eventsource::Event::Open) => (),
                Ok(reqwest_eventsource::Event::Message(message)) => {
                    if message.event == "message" {
                        messages.push(serde_json::from_str::<CompletionObject>(&message.data)?);
                    } else if message.event == "done" {
                        event_source.close();
                    }
                }
                Err(err) => {
                    event_source.close();
                }
            }
        }

        Ok(messages)
    }

    pub async fn chat_completion(
        &self,
        messages: Vec<Message>,
    ) -> Result<ChatCompletionResponse, HerokuMiaError> {
        let request_body = ChatCompletionRequest {
            model: self.inference_model_id.clone(),
            messages,
            extended_thinking: None,
            temperature: None,
            max_tokens: None,
            stop: None,
            stream: Some(false),
            tools: None,
            tool_choice: None,
            top_p: None,
        };
        let response = self
            .reqwest_client
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
            Err(HerokuMiaError::ApiCallError(error_text))
        }
    }

    pub async fn list_mcp_servers(&self) -> Result<Vec<McpServerResponse>, HerokuMiaError> {
        let response = self
            .reqwest_client
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
            Err(HerokuMiaError::ApiCallError(error_text))
        }
    }
}
