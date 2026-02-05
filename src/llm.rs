use serde::{Deserialize, Serialize};
use reqwest::Client;
use tracing::info;
use crate::error::GearClawError;
use futures::Stream;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use std::pin::Pin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub tools: Option<Vec<ToolSpec>>,
    pub tool_choice: Option<String>,
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionStreamResponse {
    pub id: String,
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChoice {
    pub delta: StreamDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamDelta {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<StreamToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamToolCall {
    pub index: usize,
    pub id: Option<String>,
    pub r#type: Option<String>,
    pub function: Option<StreamFunctionCall>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamFunctionCall {
    pub name: Option<String>,
    pub arguments: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub r#type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub r#type: String,
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// LLM client for making API calls
pub struct LLMClient {
    client: Client,
    api_key: String,
    endpoint: String,
    model: String,
}

impl LLMClient {
    pub fn new(api_key: String, endpoint: String, model: String) -> Self {
        LLMClient {
            client: Client::builder()
                .http1_only()
                .build()
                .unwrap_or_else(|_| Client::new()),
            api_key,
            endpoint,
            model,
        }
    }
    

    /// Send streaming chat completion request
    pub async fn chat_completion_stream(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolSpec>>,
        max_tokens: Option<usize>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatCompletionStreamResponse, GearClawError>> + Send>>, GearClawError> {
        info!("发送 LLM 流式请求: {} 条消息", messages.len());
        
        let tool_choice = tools.as_ref().map(|_| "auto".to_string());
        
        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages,
            max_tokens,
            temperature: Some(0.7),
            tools,
            tool_choice,
            stream: Some(true),
        };
        
        let url = format!("{}/chat/completions", self.endpoint.trim_end_matches('/'));
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| GearClawError::LLMError(format!("请求失败: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(GearClawError::LLMResponseError(
                format!("API 错误 {}: {}", status, error_text)
            ));
        }

        let stream = response
            .bytes_stream()
            .eventsource()
            .map(|event| {
                match event {
                    Ok(event) => {
                        if event.data == "[DONE]" {
                            // End of stream, we can probably filter this out or handle it
                            // For now let's just ignore it by returning an error that will be filtered?
                            // Or better, we make the stream item Option?
                            // Let's rely on JSON parse error to filter it out or explicit check
                            Err(GearClawError::LLMResponseError("Stream finished".to_string()))
                        } else {
                            serde_json::from_str::<ChatCompletionStreamResponse>(&event.data)
                                .map_err(|e| GearClawError::SerdeError(e))
                        }
                    }
                    Err(e) => Err(GearClawError::LLMError(format!("Stream error: {}", e))),
                }
            });

        Ok(Box::pin(stream))
    }
}
