// Channel Adapter Interface
//
// Defines unified interface for all messaging platforms.

use async_trait::async_trait;
use serde_json::Value as JsonValue;
use std::pin::Pin;

/// Message target (where to send)
#[derive(Debug, Clone)]
pub enum MessageTarget {
    Channel(String),
    DirectMessage(String),
    Group(String),
}

/// Message content
#[derive(Debug, Clone)]
pub struct MessageContent {
    pub text: Option<String>,
    pub embeds: Vec<Embed>,
}

/// Rich embed (image, file, etc.)
#[derive(Debug, Clone)]
pub struct Embed {
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub color: Option<u32>,
}

/// Incoming message from platform
#[derive(Debug, Clone)]
pub struct IncomingMessage {
    pub platform: String,
    pub source: MessageSource,
    pub content: String,
    pub metadata: JsonValue,
}

/// Message source
#[derive(Debug, Clone)]
pub enum MessageSource {
    User { id: String, name: String },
    Channel { id: String, name: String },
    Group { id: String, name: String },
}

/// Outgoing message to platform
#[derive(Debug, Clone)]
pub struct OutgoingMessage {
    pub target: MessageTarget,
    pub content: MessageContent,
}

/// Unified channel adapter trait
#[async_trait]
pub trait ChannelAdapter: Send + Sync {
    /// Platform name (e.g., "discord", "telegram", "whatsapp")
    fn platform_name(&self) -> &str;

    /// Start of adapter (connect to platform)
    async fn start(&mut self) -> Result<(), ChannelError>;

    /// Send message to target
    async fn send_message(&self, target: MessageTarget, content: MessageContent)
        -> Result<(), ChannelError>;

    /// Subscribe to incoming messages
    fn on_message(&self) -> Pin<Box<dyn futures_util::stream::Stream<Item = IncomingMessage> + Send>>;

    /// Resolve identifier to target
    async fn resolve_target(&self, identifier: &str) -> Result<MessageTarget, ChannelError>;

    /// Check if adapter is healthy
    async fn health_check(&self) -> Result<bool, ChannelError>;
}

/// Channel error type
#[derive(Debug)]
pub enum ChannelError {
    ConnectionFailed { platform: String, source: String },

    SendFailed { target: MessageTarget, source: String },

    ResolveFailed { identifier: String, source: String },

    HealthCheckFailed { platform: String, source: String },

    AuthenticationFailed { platform: String, source: String },
}

impl std::fmt::Display for ChannelError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ChannelError::ConnectionFailed { platform, source } => {
                write!(f, "Connection failed to {}: {}", platform, source)
            }
            ChannelError::SendFailed { target, source } => {
                write!(f, "Send failed to {:?}: {}", target, source)
            }
            ChannelError::ResolveFailed { identifier, source } => {
                write!(f, "Resolve failed for {}: {}", identifier, source)
            }
            ChannelError::HealthCheckFailed { platform, source } => {
                write!(f, "Health check failed for {}: {}", platform, source)
            }
            ChannelError::AuthenticationFailed { platform, source } => {
                write!(f, "Authentication failed for {}: {}", platform, source)
            }
        }
    }
}

impl std::error::Error for ChannelError {}

/// Channel manager
pub struct ChannelManager {
    adapters: Vec<Box<dyn ChannelAdapter + Send + Sync>>,
}

impl ChannelManager {
    pub fn new() -> Self {
        Self {
            adapters: Vec::new(),
        }
    }

    /// Register a channel adapter
    pub fn register(&mut self, adapter: Box<dyn ChannelAdapter + Send + Sync>) {
        self.adapters.push(adapter);
    }

    /// Get adapter by platform name
    pub fn get(&self, platform: &str) -> Option<&(dyn ChannelAdapter + Send + Sync)> {
        self.adapters
            .iter()
            .find(|a| a.platform_name() == platform)
            .map(|a| a.as_ref())
    }

    /// List all platforms
    pub fn platforms(&self) -> Vec<&str> {
        self.adapters.iter().map(|a| a.platform_name()).collect()
    }
}

impl Default for ChannelManager {
    fn default() -> Self {
        Self::new()
    }
}
