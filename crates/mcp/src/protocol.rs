//! MCP (Model Context Protocol) JSON-RPC 2.0 message types.
//!
//! MCP uses JSON-RPC 2.0 over stdio transport.
//! Each message is a single JSON line terminated by `\n`.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// ============================================================================
// JSON-RPC 2.0 Core Types
// ============================================================================

/// Outgoing JSON-RPC request (client → server).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    pub fn new(id: u64, method: impl Into<String>, params: impl Serialize) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.into(),
            params: Some(serde_json::to_value(params).unwrap_or(Value::Null)),
        }
    }

    pub fn new_no_params(id: u64, method: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.into(),
            params: None,
        }
    }
}

/// Outgoing JSON-RPC notification (client → server, no id, no response expected).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcNotification {
    pub fn new(method: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params: None,
        }
    }
}

/// Incoming JSON-RPC response (server → client).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

// ============================================================================
// MCP Protocol Types
// ============================================================================

/// Parameters for the `initialize` request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    pub client_info: ClientInfo,
}

impl Default for InitializeParams {
    fn default() -> Self {
        Self {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: ClientInfo {
                name: "gearclaw".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }
}

/// Client capabilities sent during initialize.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClientCapabilities {
    // We don't advertise any special capabilities initially
}

/// Client identity info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// Result of the `initialize` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: String,
    #[serde(default)]
    pub capabilities: ServerCapabilities,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_info: Option<ServerInfo>,
}

/// Server capabilities from initialize response.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<Value>,
}

/// Tools capability from server.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCapability {
    #[serde(default)]
    pub list_changed: bool,
}

/// Server identity from initialize response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

// ============================================================================
// Tool Types
// ============================================================================

/// A tool exposed by an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpTool {
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// JSON Schema for the tool's input parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<Value>,
}

/// Result of `tools/list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsListResult {
    pub tools: Vec<McpTool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Parameters for `tools/call`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallParams {
    pub name: String,
    #[serde(default)]
    pub arguments: HashMap<String, Value>,
}

/// A single content item in a tool call result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ContentItem {
    Text { text: String },
    Image { data: String, mime_type: String },
    Resource { resource: Value },
}

impl ContentItem {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ContentItem::Text { text } => Some(text),
            _ => None,
        }
    }
}

/// Result of `tools/call`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallResult {
    pub content: Vec<ContentItem>,
    #[serde(default)]
    pub is_error: bool,
}

// ============================================================================
// Incoming message (either response or notification from server)
// ============================================================================

/// An incoming message from the server — either a response to a request
/// or a server-initiated notification.
#[derive(Debug, Clone, Deserialize)]
pub struct IncomingMessage {
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<u64>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub result: Option<Value>,
    #[serde(default)]
    pub error: Option<JsonRpcError>,
    #[serde(default)]
    pub params: Option<Value>,
}

impl IncomingMessage {
    pub fn is_response(&self) -> bool {
        self.id.is_some() && self.method.is_none()
    }

    pub fn as_response(self) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: self.jsonrpc,
            id: self.id,
            result: self.result,
            error: self.error,
        }
    }
}
