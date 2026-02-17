use eventsource_stream::Eventsource;
use futures::Stream;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("request error: {0}")]
    Request(String),
    #[error("response error: {0}")]
    Response(String),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolSpec>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub data: Vec<EmbeddingData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingData {
    pub embedding: Vec<f32>,
}

pub struct LLMClient {
    client: Client,
    api_key: String,
    endpoint: String,
    model: String,
    embedding_model: String,
}

impl LLMClient {
    pub fn new(api_key: String, endpoint: String, model: String, embedding_model: String) -> Self {
        Self {
            client: Client::builder()
                .http1_only()
                .build()
                .unwrap_or_else(|_| Client::new()),
            api_key,
            endpoint,
            model,
            embedding_model,
        }
    }

    pub async fn get_embedding(&self, text: &str) -> Result<Vec<f32>, LlmError> {
        let request = EmbeddingRequest {
            model: self.embedding_model.clone(),
            input: text.to_string(),
        };
        let url = format!("{}/embeddings", self.endpoint.trim_end_matches('/'));

        info!("Sending embedding request to: {}", url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| LlmError::Request(format!("embedding request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::Response(format!(
                "embedding API error {}: {}",
                status, error_text
            )));
        }

        let embedding_response: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| LlmError::Request(format!("embedding json parse failed: {}", e)))?;

        embedding_response
            .data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .ok_or_else(|| LlmError::Response("no embedding returned".to_string()))
    }

    pub async fn chat_completion_stream(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<ToolSpec>>,
        max_tokens: Option<usize>,
    ) -> Result<
        Pin<Box<dyn Stream<Item = Result<ChatCompletionStreamResponse, LlmError>> + Send>>,
        LlmError,
    > {
        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages,
            max_tokens,
            temperature: Some(0.7),
            tools,
            tool_choice: None,
            stream: Some(true),
        };

        let url = format!("{}/chat/completions", self.endpoint.trim_end_matches('/'));
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| LlmError::Request(format!("request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::Response(format!(
                "API error {}: {}",
                status, error_text
            )));
        }

        let stream = response
            .bytes_stream()
            .eventsource()
            .map(|event| match event {
                Ok(event) => {
                    if event.data == "[DONE]" {
                        Err(LlmError::Response("Stream finished".to_string()))
                    } else {
                        serde_json::from_str::<ChatCompletionStreamResponse>(&event.data)
                            .map_err(LlmError::Json)
                    }
                }
                Err(e) => Err(LlmError::Request(format!("stream error: {}", e))),
            });

        Ok(Box::pin(stream))
    }
}
