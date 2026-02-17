//! Compatibility wrapper for MCP subsystem.
//! Delegates to `gearclaw_mcp` while preserving `gearclaw_core` API.
use crate::config::{McpConfig as CoreMcpConfig, McpServerConfig as CoreMcpServerConfig};
use crate::error::GearClawError;
use crate::tools::{ToolResult as CoreToolResult, ToolSpec as CoreToolSpec};
use std::collections::HashMap;

pub struct McpManager {
    inner: gearclaw_mcp::McpManager,
}

impl McpManager {
    pub fn new(config: CoreMcpConfig) -> Self {
        Self {
            inner: gearclaw_mcp::McpManager::new(to_mcp_config(config)),
        }
    }

    pub async fn init_clients(&self) -> Result<(), GearClawError> {
        self.inner
            .init_clients()
            .await
            .map_err(|e| GearClawError::Other(e.to_string()))
    }

    pub async fn list_tools(&self) -> Vec<CoreToolSpec> {
        self.inner
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

    pub async fn call_tool(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<CoreToolResult, GearClawError> {
        self.inner
            .call_tool(name, args)
            .await
            .map(|r| CoreToolResult {
                success: r.success,
                output: r.output,
                error: r.error,
            })
            .map_err(|e| GearClawError::Other(e.to_string()))
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
    }
}
