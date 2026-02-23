//! MCP subsystem wrapper for `gearclaw_core`.
//!
//! Wraps `gearclaw_mcp::McpManager` behind a `tokio::sync::Mutex` to support
//! dynamic reload from the Agent's self-management tools.

use crate::config::{McpConfig as CoreMcpConfig, McpServerConfig as CoreMcpServerConfig};
use crate::error::GearClawError;
use crate::tools::{ToolResult as CoreToolResult, ToolSpec as CoreToolSpec};
pub use gearclaw_mcp::{McpCapability, ServerStatusEntry};
use std::collections::HashMap;
use tokio::sync::Mutex;

/// Thread-safe wrapper around `gearclaw_mcp::McpManager`.
/// Use via `Arc<McpManager>` — the internal Mutex serialises all async operations.
pub struct McpManager {
    inner: Mutex<gearclaw_mcp::McpManager>,
}

impl McpManager {
    pub fn new(config: CoreMcpConfig) -> Self {
        Self {
            inner: Mutex::new(gearclaw_mcp::McpManager::new(to_mcp_config(config))),
        }
    }

    pub fn capability(&self) -> McpCapability {
        gearclaw_mcp::BUILD_MCP_CAPABILITY
    }

    pub fn is_enabled(&self) -> bool {
        matches!(self.capability(), McpCapability::Enabled)
    }

    /// Connect to all enabled MCP servers.
    pub async fn init_clients(&self) -> Result<(), GearClawError> {
        self.inner.lock().await.init_clients().await.map_err(|e| {
            GearClawError::from(crate::error::DomainError::Mcp {
                server: "manager".to_string(),
                reason: e.to_string(),
            })
        })
    }

    /// Disconnect all clients and reconnect from the current config.
    pub async fn reload(&self) -> Result<(), GearClawError> {
        self.inner.lock().await.reload().await.map_err(|e| {
            GearClawError::from(crate::error::DomainError::Mcp {
                server: "manager".to_string(),
                reason: e.to_string(),
            })
        })
    }

    /// List all tools from connected servers.
    pub async fn list_tools(&self) -> Vec<CoreToolSpec> {
        self.inner
            .lock()
            .await
            .list_tools()
            .await
            .into_iter()
            .map(|t| CoreToolSpec {
                name: t.name,
                description: t.description,
                requires_args: t.requires_args,
                parameters: t.parameters,
            })
            .collect()
    }

    /// Call an MCP tool by qualified name `{server}__{tool}`.
    pub async fn call_tool(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<CoreToolResult, GearClawError> {
        self.inner
            .lock()
            .await
            .call_tool(name, args)
            .await
            .map(|r| CoreToolResult {
                success: r.success,
                output: r.output,
                error: r.error,
            })
            .map_err(|e| {
                GearClawError::from(crate::error::DomainError::Mcp {
                    server: name.split("__").next().unwrap_or("unknown").to_string(),
                    reason: e.to_string(),
                })
            })
    }

    /// Returns the status summary for all configured servers.
    pub async fn status_summary(&self) -> Vec<ServerStatusEntry> {
        self.inner.lock().await.status_summary()
    }

    /// Add a new server config entry and reconnect all servers.
    pub async fn add_server(
        &self,
        name: String,
        config: CoreMcpServerConfig,
    ) -> Result<(), GearClawError> {
        let mut inner = self.inner.lock().await;
        inner.config.servers.insert(name, to_mcp_server_config(config));
        inner.reload().await.map_err(|e| {
            GearClawError::from(crate::error::DomainError::Mcp {
                server: "manager".to_string(),
                reason: e.to_string(),
            })
        })
    }

    /// Enable or disable a server by name, then reload.
    pub async fn set_server_enabled(
        &self,
        name: &str,
        enabled: bool,
    ) -> Result<(), GearClawError> {
        let mut inner = self.inner.lock().await;
        if let Some(cfg) = inner.config.servers.get_mut(name) {
            cfg.enabled = enabled;
            inner.reload().await.map_err(|e| {
                GearClawError::from(crate::error::DomainError::Mcp {
                    server: name.to_string(),
                    reason: e.to_string(),
                })
            })
        } else {
            Err(GearClawError::from(crate::error::DomainError::Mcp {
                server: name.to_string(),
                reason: format!("Server '{}' not found in config", name),
            }))
        }
    }

    /// Retrieve a snapshot of the current MCP config.
    pub async fn current_config(&self) -> gearclaw_mcp::McpConfig {
        self.inner.lock().await.config.clone()
    }
}

fn to_mcp_config(config: CoreMcpConfig) -> gearclaw_mcp::McpConfig {
    gearclaw_mcp::McpConfig {
        servers: config
            .servers
            .into_iter()
            .map(|(k, v)| (k, to_mcp_server_config(v)))
            .collect::<HashMap<_, _>>(),
    }
}

fn to_mcp_server_config(config: CoreMcpServerConfig) -> gearclaw_mcp::McpServerConfig {
    gearclaw_mcp::McpServerConfig {
        command: config.command,
        args: config.args,
        env: config.env,
        enabled: config.enabled,
    }
}
