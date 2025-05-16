use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Network error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("API error: {0}")]
    ApiCallError(String), // To capture error messages from the API
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

#[derive(Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: String,
}

#[derive(Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
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

    pub async fn chat_completion(
        &self,
        messages: Vec<Message>,
    ) -> Result<ChatCompletionResponse, ApiError> {
        let request_body = ChatCompletionRequest {
            model: self.inference_model_id.clone(),
            messages,
            temperature: None,
            max_tokens: None,
        };

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/v1/chat/completions", self.inference_url))
            .header("Authorization", format!("Bearer {}", self.inference_key))
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
}
