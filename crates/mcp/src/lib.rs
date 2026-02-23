//! GearClaw MCP (Model Context Protocol) subsystem.
//!
//! Provides real stdio-based MCP client connections, tool discovery,
//! and tool invocation. Also includes a built-in server registry for
//! autonomous search and installation.

pub mod client;
pub mod error;
pub mod protocol;
pub mod registry;
pub mod transport;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

pub use client::{ConnectionStatus, McpClient};
pub use error::McpError;
pub use registry::{builtin_registry, find_by_id, search as search_registry, InstallMethod, RegistryEntry};

// ============================================================================
// Capability flag — now Enabled
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpCapability {
    Enabled,
    Disabled,
}

/// MCP is now fully enabled.
pub const BUILD_MCP_CAPABILITY: McpCapability = McpCapability::Enabled;

// ============================================================================
// Config types (kept here so gearclaw_core can depend on them)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpConfig {
    #[serde(default)]
    pub servers: HashMap<String, McpServerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Whether this server is enabled (default: true).
    #[serde(default = "McpServerConfig::default_enabled")]
    pub enabled: bool,
}

impl McpServerConfig {
    fn default_enabled() -> bool {
        true
    }
}

// ============================================================================
// Shared tool types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub requires_args: bool,
    #[serde(default)]
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

// ============================================================================
// McpManager — manages multiple server clients
// ============================================================================

/// Manages connections to multiple MCP servers.
/// Use `Arc<tokio::sync::Mutex<McpManager>>` when sharing across async tasks.
pub struct McpManager {
    pub config: McpConfig,
    /// Active client connections keyed by server name.
    pub clients: HashMap<String, McpClient>,
}

impl McpManager {
    /// Create a new manager from config. Does not connect yet.
    pub fn new(config: McpConfig) -> Self {
        Self {
            config,
            clients: HashMap::new(),
        }
    }

    pub fn capability(&self) -> McpCapability {
        BUILD_MCP_CAPABILITY
    }

    pub fn is_enabled(&self) -> bool {
        matches!(self.capability(), McpCapability::Enabled)
    }

    /// Connect to all enabled servers defined in the config.
    pub async fn init_clients(&mut self) -> Result<(), McpError> {
        if self.config.servers.is_empty() {
            return Ok(());
        }

        let server_names: Vec<String> = self
            .config
            .servers
            .iter()
            .filter(|(_, cfg)| cfg.enabled)
            .map(|(name, _)| name.clone())
            .collect();

        info!("[MCP] Initializing {} server(s)...", server_names.len());

        for name in server_names {
            if let Some(cfg) = self.config.servers.get(&name) {
                match McpClient::connect(&name, cfg).await {
                    Ok(client) => {
                        info!("[MCP] '{}' connected ({} tools)", name, client.tools.len());
                        self.clients.insert(name, client);
                    }
                    Err(e) => {
                        warn!("[MCP] Failed to connect '{}': {}", name, e);
                        // Non-fatal: continue with other servers
                    }
                }
            }
        }

        Ok(())
    }

    /// Disconnect all clients and reconnect from current config.
    pub async fn reload(&mut self) -> Result<(), McpError> {
        self.clients.clear();
        self.init_clients().await
    }

    /// Returns all tools from all connected servers.
    /// Tool names are prefixed as `{server_name}__{tool_name}`.
    pub async fn list_tools(&self) -> Vec<ToolSpec> {
        let mut specs = Vec::new();
        for (server_name, client) in &self.clients {
            if client.status != ConnectionStatus::Connected {
                continue;
            }
            for tool in &client.tools {
                specs.push(ToolSpec {
                    name: format!("{}__{}" , server_name, tool.name),
                    description: format!("[{}] {}", server_name, tool.description),
                    requires_args: tool.input_schema.is_some(),
                    parameters: tool.input_schema.clone(),
                });
            }
        }
        specs
    }

    /// Call an MCP tool by its qualified name `{server_name}__{tool_name}`.
    pub async fn call_tool(
        &self,
        qualified_name: &str,
        args: serde_json::Value,
    ) -> Result<ToolResult, McpError> {
        let (server_name, tool_name) = qualified_name.split_once("__").ok_or_else(|| {
            McpError::ToolNotFound(format!(
                "Invalid MCP tool name '{}' (expected 'server__tool' format)",
                qualified_name
            ))
        })?;

        let client = self.clients.get(server_name).ok_or_else(|| {
            McpError::ToolNotFound(format!(
                "MCP server '{}' is not connected",
                server_name
            ))
        })?;

        let output = client.call_tool(tool_name, args).await?;

        Ok(ToolResult {
            success: true,
            output,
            error: None,
        })
    }

    /// Returns the connection status summary for all configured servers.
    pub fn status_summary(&self) -> Vec<ServerStatusEntry> {
        let mut entries = Vec::new();

        for (name, cfg) in &self.config.servers {
            let status = if !cfg.enabled {
                "disabled".to_string()
            } else if let Some(client) = self.clients.get(name) {
                client.status.to_string()
            } else {
                "not connected".to_string()
            };

            let tool_count = self
                .clients
                .get(name)
                .map(|c| c.tools.len())
                .unwrap_or(0);

            entries.push(ServerStatusEntry {
                name: name.clone(),
                command: cfg.command.clone(),
                enabled: cfg.enabled,
                status,
                tool_count,
                server_info: self.clients.get(name).and_then(|c| c.server_info.clone()),
            });
        }

        entries.sort_by(|a, b| a.name.cmp(&b.name));
        entries
    }
}

/// Summary of a single server's connection status.
#[derive(Debug, Clone)]
pub struct ServerStatusEntry {
    pub name: String,
    pub command: String,
    pub enabled: bool,
    pub status: String,
    pub tool_count: usize,
    pub server_info: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{McpCapability, McpConfig, McpManager};

    #[test]
    fn build_capability_is_enabled() {
        let manager = McpManager::new(McpConfig::default());
        assert_eq!(manager.capability(), McpCapability::Enabled);
        assert!(manager.is_enabled());
    }

    #[test]
    fn empty_config_has_no_clients() {
        let manager = McpManager::new(McpConfig::default());
        assert!(manager.clients.is_empty());
    }
}
