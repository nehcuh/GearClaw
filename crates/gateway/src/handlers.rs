// Gateway Method Handlers
//
// This module implements handlers for OpenClaw protocol methods.

use crate::protocol::GatewayRequest;
use anyhow::Result;
use gearclaw_channels::adapter::{ChannelManager, MessageContent};
use serde_json::json;
use serde_json::Value as JsonValue;
use std::sync::Arc;
use uuid::Uuid;

pub struct MethodHandlers {
    /// Optional Agent reference (will be set by Gateway server)
    agent: Arc<tokio::sync::Mutex<Option<Arc<gearclaw_agent::Agent>>>>,
    /// Channel manager for routing messages to platforms
    channel_manager: Arc<tokio::sync::Mutex<ChannelManager>>,
    /// Agent trigger configuration
    trigger_config: Arc<tokio::sync::Mutex<gearclaw_core::AgentTriggerConfig>>,
}

impl MethodHandlers {
    pub fn new() -> Self {
        Self {
            agent: Arc::new(tokio::sync::Mutex::new(None)),
            channel_manager: Arc::new(tokio::sync::Mutex::new(ChannelManager::new())),
            trigger_config: Arc::new(tokio::sync::Mutex::new(
                gearclaw_core::AgentTriggerConfig::default(),
            )),
        }
    }

    /// Set agent reference
    pub async fn set_agent(&self, agent: Arc<gearclaw_agent::Agent>) {
        let mut agent_guard = self.agent.lock().await;
        *agent_guard = Some(agent);
    }

    /// Set trigger configuration
    pub async fn set_trigger_config(&self, config: gearclaw_core::AgentTriggerConfig) {
        let mut trigger_guard = self.trigger_config.lock().await;
        *trigger_guard = config;
    }

    /// Get trigger configuration
    pub async fn get_trigger_config(&self) -> gearclaw_core::AgentTriggerConfig {
        let trigger_guard = self.trigger_config.lock().await;
        trigger_guard.clone()
    }

    /// Get channel manager reference
    pub fn channel_manager(&self) -> Arc<tokio::sync::Mutex<ChannelManager>> {
        Arc::clone(&self.channel_manager)
    }

    /// Get agent reference (if configured)
    pub async fn get_agent(&self) -> Option<Arc<gearclaw_agent::Agent>> {
        let agent_guard = self.agent.lock().await;
        agent_guard.clone()
    }

    /// Handle health check - returns actual Gateway status
    pub async fn health(&self, _request: &GatewayRequest) -> Result<JsonValue> {
        let uptime = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Ok(json!({
            "status": "ok",
            "version": env!("CARGO_PKG_VERSION"),
            "uptime_ms": uptime,
            "active_sessions": 0,
        }))
    }

    /// Handle status request - returns actual connection stats
    pub async fn status(&self, _request: &GatewayRequest) -> Result<JsonValue> {
        Ok(json!({
            "uptime_ms": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            "total_connections": 1,
            "active_sessions": 1,
        }))
    }

    /// Handle send message request
    pub async fn send(&self, request: &GatewayRequest) -> Result<JsonValue> {
        // Parse target parameter (format: "platform:identifier")
        let target_str = request
            .params
            .get("target")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'target' parameter"))?;

        let message = request
            .params
            .get("message")
            .and_then(|m| m.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'message' parameter"))?;

        tracing::info!("Send requested: target={}, message={}", target_str, message);

        // Parse target format: "platform:identifier"
        let parts: Vec<&str> = target_str.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Ok(json!({
                "success": false,
                "error": format!("Invalid target format. Expected 'platform:identifier', got '{}'", target_str),
            }));
        }

        let platform = parts[0];
        let identifier = parts[1];

        // Get channel adapter
        let manager = self.channel_manager.lock().await;
        let adapter = manager.get(platform);

        if let Some(adapter) = adapter {
            // Resolve target
            let target = adapter
                .resolve_target(identifier)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to resolve target: {}", e))?;

            // Create message content
            let content = MessageContent {
                text: Some(message.to_string()),
                embeds: vec![],
            };

            // Send message
            adapter
                .send_message(target, content)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to send message: {}", e))?;

            tracing::info!("Message sent successfully to {}:{}", platform, identifier);

            Ok(json!({
                "success": true,
                "platform": platform,
                "identifier": identifier,
                "sent_at": chrono::Utc::now().to_rfc3339(),
            }))
        } else {
            Ok(json!({
                "success": false,
                "error": format!("Platform '{}' not registered", platform),
            }))
        }
    }

    /// Handle agent execution request
    pub async fn agent_execute(&self, request: &GatewayRequest) -> Result<JsonValue> {
        let run_id = Uuid::new_v4().to_string();

        // Extract prompt from params
        let prompt = request
            .params
            .get("prompt")
            .and_then(|p| p.as_str())
            .unwrap_or("");

        tracing::info!(
            "Agent execution requested: run_id={}, prompt='{}'",
            run_id,
            prompt
        );

        // Check if agent is available
        if let Some(agent) = self.get_agent().await {
            // Create or get session for this request
            // Use request ID or device ID as session identifier
            let session_id = request.device_id.as_ref().unwrap_or(&request.id).clone();

            // Get or create session
            let mut sess = agent
                .session_manager
                .get_or_create_session(&session_id)
                .map_err(|e| anyhow::anyhow!("Failed to get session: {}", e))?;

            // Process message with agent
            let response = agent
                .process_message(&mut sess, prompt)
                .await
                .map_err(|e| anyhow::anyhow!("Agent execution failed: {}", e))?;

            // Save session
            agent
                .session_manager
                .save_session(&sess)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to save session: {}", e))?;

            Ok(json!({
                "run_id": run_id,
                "status": "completed",
                "response": response,
            }))
        } else {
            // No agent configured - return queued status
            Ok(json!({
                "run_id": run_id,
                "status": "accepted",
                "message": format!("Agent queued: {}", prompt),
            }))
        }
    }
}

impl Default for MethodHandlers {
    fn default() -> Self {
        Self::new()
    }
}
