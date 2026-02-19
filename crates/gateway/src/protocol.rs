// OpenClaw WebSocket Protocol Implementation
//
// This module implements the OpenClaw protocol frame types for WebSocket communication.
//
// Protocol flow:
// 1. Client sends "connect" request with device identity
// 2. Server responds with "hello-ok" containing snapshot
// 3. Bi-directional request/response communication
// 4. Server pushes events (agent, presence, tick, shutdown)

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Gateway frame - top-level message type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum GatewayFrame {
    #[serde(rename = "req")]
    Request(GatewayRequest),

    #[serde(rename = "res")]
    Response(GatewayResponse),

    #[serde(rename = "event")]
    Event(GatewayEvent),
}

/// Request frame from client to gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayRequest {
    /// Unique request ID
    pub id: String,

    /// Method name (e.g., "health", "status", "send", "agent")
    pub method: String,

    /// Method parameters
    pub params: JsonValue,

    /// Monotonically increasing sequence number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<u64>,

    /// Device identifier (from device identity)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,

    /// Request signature (for authentication)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl GatewayRequest {
    pub fn new(id: String, method: String, params: JsonValue) -> Self {
        Self {
            id,
            method,
            params,
            sequence: None,
            device_id: None,
            signature: None,
        }
    }

    pub fn with_sequence(mut self, seq: u64) -> Self {
        self.sequence = Some(seq);
        self
    }

    pub fn with_device_id(mut self, device_id: String) -> Self {
        self.device_id = Some(device_id);
        self
    }

    pub fn with_signature(mut self, signature: String) -> Self {
        self.signature = Some(signature);
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn params(&self) -> &JsonValue {
        &self.params
    }
}

/// Response frame from gateway to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayResponse {
    /// Request ID being responded to
    pub id: String,

    /// Success flag
    pub ok: bool,

    /// Response payload (if ok)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<JsonValue>,

    /// Error details (if not ok)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ProtocolError>,
}

impl GatewayResponse {
    pub fn ok(id: String, payload: JsonValue) -> Self {
        Self {
            id,
            ok: true,
            payload: Some(payload),
            error: None,
        }
    }

    pub fn error(id: String, error: ProtocolError) -> Self {
        Self {
            id,
            ok: false,
            payload: None,
            error: Some(error),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

/// Protocol error type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolError {
    /// Error code
    pub code: String,

    /// Human-readable error message
    pub message: String,

    /// Additional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<JsonValue>,

    /// Whether the request is retryable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retryable: Option<bool>,

    /// Retry after milliseconds (if retryable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_ms: Option<u64>,
}

impl ProtocolError {
    // Standard error codes
    pub const INVALID_REQUEST: &'static str = "INVALID_REQUEST";
    pub const NOT_LINKED: &'static str = "NOT_LINKED";
    pub const AGENT_TIMEOUT: &'static str = "AGENT_TIMEOUT";
    pub const UNAVAILABLE: &'static str = "UNAVAILABLE";
    pub const UNAUTHORIZED: &'static str = "UNAUTHORIZED";
    pub const NOT_FOUND: &'static str = "NOT_FOUND";
    pub const INTERNAL_ERROR: &'static str = "INTERNAL_ERROR";

    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
            retryable: None,
            retry_after_ms: None,
        }
    }

    pub fn with_details(mut self, details: JsonValue) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_retryable(mut self, retry_after_ms: u64) -> Self {
        self.retryable = Some(true);
        self.retry_after_ms = Some(retry_after_ms);
        self
    }
}

/// Event frame from gateway to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload")]
pub enum GatewayEvent {
    /// Agent execution events (streamed output)
    #[serde(rename = "agent")]
    Agent(AgentEvent),

    /// Presence updates (device online/offline)
    #[serde(rename = "presence")]
    Presence(PresenceEvent),

    /// Keepalive tick
    #[serde(rename = "tick")]
    Tick {},

    /// Gateway shutting down
    #[serde(rename = "shutdown")]
    Shutdown(ShutdownEvent),

    /// Channel message received (from Discord, Telegram, WhatsApp, etc.)
    #[serde(rename = "channel.message")]
    ChannelMessage(ChannelMessageEvent),
}

/// Agent execution event (streamed during agent.run)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEvent {
    /// Event sequence number
    pub seq: u64,

    /// Event type
    #[serde(flatten)]
    pub content: AgentEventContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentEventContent {
    /// Tool execution started
    #[serde(rename = "tool.start")]
    ToolStart { tool: String, params: JsonValue },

    /// Tool execution progress update
    #[serde(rename = "tool.progress")]
    ToolProgress { tool: String, progress: JsonValue },

    /// Tool execution completed
    #[serde(rename = "tool.end")]
    ToolEnd { tool: String, result: JsonValue },

    /// Agent output (streamed response chunks)
    #[serde(rename = "output")]
    Output {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<JsonValue>,
    },
}

/// Presence event - device online/offline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceEvent {
    /// Presence updates (deltas)
    pub updates: Vec<PresenceEntry>,

    /// State version for delta tracking
    pub state_version: u64,
}

/// Presence entry - device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceEntry {
    /// Hostname or device identifier
    pub host: String,

    /// IP address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,

    /// Gateway version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Platform (macos, linux, windows)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,

    /// Device family
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_family: Option<String>,

    /// Device model identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_identifier: Option<String>,

    /// Connection mode (gateway, node, etc.)
    pub mode: String,

    /// Last input time (seconds ago)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_input_seconds: Option<u64>,

    /// Timestamp
    pub ts: i64,

    /// Reason for presence update
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Tags for this device
    #[serde(default)]
    pub tags: Vec<String>,

    /// Instance ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
}

/// Shutdown event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownEvent {
    /// Shutdown reason
    pub reason: String,

    /// Expected restart time (milliseconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart_expected_ms: Option<u64>,
}

/// Channel message event - message received from a messaging platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessageEvent {
    /// Platform name (discord, telegram, whatsapp)
    pub platform: String,

    /// Source information
    pub source: ChannelSource,

    /// Message content
    pub content: String,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<JsonValue>,

    /// Timestamp
    pub ts: i64,
}

/// Channel message source
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ChannelSource {
    /// Message from a user
    #[serde(rename = "user")]
    User {
        /// User ID
        id: String,
        /// Username or display name
        name: String,
    },

    /// Message from a channel
    #[serde(rename = "channel")]
    Channel {
        /// Channel ID
        id: String,
        /// Channel name
        name: String,
    },

    /// Message from a group
    #[serde(rename = "group")]
    Group {
        /// Group ID
        id: String,
        /// Group name
        name: String,
    },
}

/// Hello-ok payload (response to connect request)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloOkPayload {
    /// Protocol version info
    pub protocol: ProtocolVersion,

    /// Current presence snapshot
    pub presence: Vec<PresenceEntry>,

    /// Health snapshot
    pub health: JsonValue,

    /// State version
    pub state_version: StateVersion,

    /// Gateway uptime (milliseconds)
    pub uptime_ms: u64,

    /// Gateway policy
    pub policy: GatewayPolicy,
}

/// Protocol version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolVersion {
    /// Minimum supported version
    pub min: u32,

    /// Maximum supported version
    pub max: u32,
}

/// State version for delta tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateVersion {
    /// Presence version
    pub presence: u64,

    /// Health version
    pub health: u64,
}

/// Gateway policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayPolicy {
    /// Maximum payload size
    pub max_payload: usize,

    /// Maximum buffered bytes
    pub max_buffered_bytes: usize,

    /// Tick interval (milliseconds)
    pub tick_interval_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_request_serialization() {
        let req = GatewayRequest::new("test-1".to_string(), "health".to_string(), JsonValue::Null)
            .with_sequence(1);

        // Test GatewayRequest serialization (without frame wrapper)
        let req_json = serde_json::to_string(&req).unwrap();
        assert!(req_json.contains("\"method\":\"health\""));
        assert!(req_json.contains("\"id\":\"test-1\""));

        // Test GatewayFrame::Request serialization (with frame wrapper)
        let frame = GatewayFrame::Request(req);
        let frame_json = serde_json::to_string(&frame).unwrap();
        assert!(frame_json.contains("\"type\":\"req\""));
        assert!(frame_json.contains("\"method\":\"health\""));
    }

    #[test]
    fn test_response_ok() {
        let res = GatewayResponse::ok("test-1".to_string(), json!({"status": "ok"}));

        let json = serde_json::to_string(&res).unwrap();
        assert!(json.contains("\"ok\":true"));
        assert!(json.contains("\"status\":\"ok\""));
    }

    #[test]
    fn test_response_error() {
        let err = ProtocolError::new("INVALID_REQUEST", "Invalid params");
        let res = GatewayResponse::error("test-1".to_string(), err);

        let json = serde_json::to_string(&res).unwrap();
        assert!(json.contains("\"ok\":false"));
        assert!(json.contains("\"code\":\"INVALID_REQUEST\""));
    }
}
