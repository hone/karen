use serde::{Deserialize, Serialize, ser::Serializer};

use super::types::{Choice, ExtendedThinking, Message, Usage};

#[derive(Serialize, Debug)]
pub struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extended_thinking: Option<ExtendedThinking>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ChatCompletionTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
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

#[derive(Serialize, Debug)]
pub struct FunctionDefinition {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<FunctionParameters>,
}

#[derive(Serialize, Debug)]
pub struct FunctionParameters {
    pub r#type: String,
    pub properties: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

pub struct ChatCompletionRequestBuilder {
    model: String,
    messages: Vec<Message>,
    extended_thinking: Option<ExtendedThinking>,
    max_tokens: Option<u32>,
    stop: Option<Vec<String>>,
    stream: Option<bool>,
    temperature: Option<f32>,
    tool_choice: Option<ToolChoice>,
    tools: Option<Vec<ChatCompletionTool>>,
    top_p: Option<f32>,
}

impl ChatCompletionRequestBuilder {
    pub fn new(model: impl Into<String>, messages: Vec<Message>) -> Self {
        ChatCompletionRequestBuilder {
            model: model.into(),
            messages,
            extended_thinking: None,
            max_tokens: None,
            stop: None,
            stream: None,
            temperature: None,
            tool_choice: None,
            tools: None,
            top_p: None,
        }
    }

    pub fn extended_thinking(mut self, extended_thinking: ExtendedThinking) -> Self {
        self.extended_thinking = Some(extended_thinking);
        self
    }

    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn stop(mut self, stop: Vec<String>) -> Self {
        self.stop = Some(stop);
        self
    }

    pub fn stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    pub fn tools(mut self, tools: Vec<ChatCompletionTool>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    pub fn build(self) -> ChatCompletionRequest {
        ChatCompletionRequest {
            model: self.model,
            messages: self.messages,
            extended_thinking: self.extended_thinking,
            max_tokens: self.max_tokens,
            stop: self.stop,
            stream: self.stream,
            temperature: self.temperature,
            tool_choice: self.tool_choice,
            tools: self.tools,
            top_p: self.top_p,
        }
    }
}

impl ChatCompletionRequest {
    pub fn builder(
        model: impl Into<String>,
        messages: Vec<Message>,
    ) -> ChatCompletionRequestBuilder {
        ChatCompletionRequestBuilder::new(model.into(), messages)
    }
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub system_fingerprint: Option<String>,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tool_choice_none_serialization() {
        let tool_choice_none = ToolChoice::None;
        let serialized_none = serde_json::to_value(&tool_choice_none).unwrap();
        assert_eq!(serialized_none, json!("none"));
    }

    #[test]
    fn test_tool_choice_auto_serialization() {
        let tool_choice_auto = ToolChoice::Auto;
        let serialized_auto = serde_json::to_value(&tool_choice_auto).unwrap();
        assert_eq!(serialized_auto, json!("auto"));
    }

    #[test]
    fn test_tool_choice_required_serialization() {
        let tool_choice_required = ToolChoice::Required;
        let serialized_required = serde_json::to_value(&tool_choice_required).unwrap();
        assert_eq!(serialized_required, json!("required"));
    }

    #[test]
    fn test_tool_choice_tool_serialization() {
        let tool = ChatCompletionTool {
            r#type: "function".to_string(),
            function: FunctionDefinition {
                name: "get_weather".to_string(),
                description: Some("Get the current weather in a given location".to_string()),
                parameters: Some(FunctionParameters {
                    r#type: "object".to_string(),
                    properties: json!({
                        "location": {
                            "type": "string",
                            "description": "The city and state, e.g. San Francisco, CA",
                        },
                        "unit": {
                            "type": "string",
                            "enum": ["celsius", "fahrenheit"],
                        },
                    }),
                    required: Some(vec!["location".to_string()]),
                }),
            },
        };
        let tool_choice_tool = ToolChoice::Tool(tool);
        let serialized_tool = serde_json::to_value(&tool_choice_tool).unwrap();
        assert_eq!(
            serialized_tool,
            json!({
                "type": "function",
                "function": {
                    "name": "get_weather",
                    "description": "Get the current weather in a given location",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "location": {
                                "type": "string",
                                "description": "The city and state, e.g. San Francisco, CA"
                            },
                            "unit": {
                                "type": "string",
                                "enum": ["celsius", "fahrenheit"]
                            }
                        },
                        "required": ["location"]
                    }
                }
            })
        );
    }

    #[test]
    fn test_chat_completion_response_deserialization() {
        let json_data = json!({
          "id": "chatcmpl-1839afa8133ceda215788",
          "object": "chat.completion",
          "created": 1745619466,
          "model": "claude-3-7-sonnet",
          "system_fingerprint": "heroku-inf-1y38gdr",
          "choices": [
            {
              "index": 0,
              "message": {
                "role": "assistant",
                "content": "Hi! How can I help you today?",
                "refusal": null
              },
              "finish_reason": "stop"
            }
          ],
          "usage": {
            "prompt_tokens": 8,
            "completion_tokens": 12,
            "total_tokens": 20
          }
        });

        let response: ChatCompletionResponse = serde_json::from_value(json_data).unwrap();

        assert_eq!(response.id, "chatcmpl-1839afa8133ceda215788");
        assert_eq!(response.object, "chat.completion");
        assert_eq!(response.created, 1745619466);
        assert_eq!(response.model, "claude-3-7-sonnet");
        assert_eq!(
            response.system_fingerprint,
            Some("heroku-inf-1y38gdr".to_string())
        );
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.usage.prompt_tokens, 8);
        assert_eq!(response.usage.completion_tokens, 12);
        assert_eq!(response.usage.total_tokens, 20);

        let choice = &response.choices[0];
        assert_eq!(choice.index, 0);
        assert_eq!(choice.finish_reason, "stop");

        match &choice.message {
            Message::Assistant {
                content,
                refusal,
                tool_calls,
            } => {
                assert_eq!(content, "Hi! How can I help you today?");
                assert_eq!(refusal, &None);
                assert_eq!(tool_calls, &None);
            }
            _ => panic!("Unexpected message type"),
        }

        assert_eq!(response.usage.prompt_tokens, 8);
        assert_eq!(response.usage.completion_tokens, 12);
        assert_eq!(response.usage.total_tokens, 20);
    }
}
