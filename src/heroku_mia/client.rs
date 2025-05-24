use futures::{
    Stream, StreamExt,
    stream::{self, BoxStream},
};
use reqwest::Client as ReqwestClient;
use reqwest_eventsource::{Event, EventSource};
use std::pin::Pin;
use thiserror::Error;

use super::{
    agents::{AgentRequest, CompletionObject},
    chat_completion::{ChatCompletionRequest, ChatCompletionResponse},
    mcp_servers::McpServerResponse,
};

#[derive(Error, Debug)]
pub enum HerokuMiaError {
    #[error("Network error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Server Side Event error: {0}")]
    EventSourceError(#[from] reqwest_eventsource::Error),
    #[error("RequestBuilder can not be cloned: {0}")]
    CannotCloneRequestError(#[from] reqwest_eventsource::CannotCloneRequestError),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("API error: {0}")]
    ApiCallError(String),
}

#[derive(Debug, Clone)]
pub struct Client {
    inference_url: String,
    inference_key: String,
    reqwest_client: ReqwestClient,
}

impl Client {
    pub fn new(inference_url: String, inference_key: String) -> Self {
        Self {
            inference_url,
            inference_key,
            reqwest_client: ReqwestClient::new(),
        }
    }

    pub async fn agents_call(
        &self,
        request_body: &AgentRequest,
    ) -> Pin<Box<dyn Stream<Item = Result<CompletionObject, HerokuMiaError>> + Send>> {
        let request_builder = self
            .reqwest_client
            .post(format!("{}/v1/agents/heroku", self.inference_url))
            .header("Authorization", format!("Bearer {}", self.inference_key))
            .header("Content-Type", "application/json")
            .json(request_body);

        let event_source = match EventSource::new(request_builder) {
            Ok(es) => es,
            Err(e) => {
                return Box::pin(stream::once(async move {
                    Err(HerokuMiaError::CannotCloneRequestError(e))
                }));
            }
        };

        Box::pin(event_source.filter_map(|event| async move {
            match event {
                Ok(Event::Open) => {
                    tracing::debug!("Agent Call: Open Event!");
                    None
                }
                Ok(Event::Message(message)) => {
                    if message.event == "message" {
                        tracing::debug!("Agent Call: Received Message");
                        match serde_json::from_str::<CompletionObject>(&message.data) {
                            Ok(obj) => Some(Ok(obj)),
                            Err(e) => Some(Err(HerokuMiaError::JsonError(e))),
                        }
                    } else if message.event == "heartbeat" {
                        tracing::debug!("Agent Call: Heartbeat");
                        None
                    } else if message.event == "done" {
                        tracing::debug!("Agent Call: Close");
                        None
                    } else {
                        tracing::error!("Agent Call: Unknown Message {:?}", message);
                        None
                    }
                }
                Err(err) => {
                    tracing::debug!("Agent Call: Error: {}", err);
                    match err {
                        reqwest_eventsource::Error::StreamEnded => {
                            tracing::debug!("Agent Call: StreamEnded {}", err);
                        }
                        _ => {
                            tracing::error!("Agent Call: Error {}", err);
                        }
                    }
                    Some(Err(HerokuMiaError::EventSourceError(err)))
                }
            }
        }))
    }

    pub async fn chat_completion(
        &self,
        request_body: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, HerokuMiaError> {
        let response = self
            .reqwest_client
            .post(format!("{}/v1/chat/completions", self.inference_url))
            .header("Authorization", format!("Bearer {}", self.inference_key))
            .header("Content-Type", "application/json")
            .json(request_body)
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
