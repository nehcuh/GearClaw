//! MCP (Model Context Protocol) Integration
//!
//! This module provides MCP client functionality for integrating external tools.
//! Note: Full MCP support requires the `rmcp` crate which needs nightly Rust.
//! This is a stub implementation that can be extended when rmcp becomes stable.

use crate::config::McpConfig;
use crate::error::GearClawError;
use crate::tools::{ToolResult, ToolSpec};
use tracing::warn;

/// MCP Manager for handling Model Context Protocol servers
pub struct McpManager {
    #[allow(dead_code)]
    config: McpConfig,
}

impl McpManager {
    /// Create a new MCP manager
    pub fn new(config: McpConfig) -> Self {
        Self { config }
    }

    /// Initialize MCP clients
    ///
    /// Note: This is a stub implementation. Full MCP support requires
    /// the rmcp crate which currently needs nightly Rust (edition 2024).
    pub async fn init_clients(&self) -> Result<(), GearClawError> {
        if !self.config.servers.is_empty() {
            warn!(
                "MCP servers configured but MCP support is disabled. \
                 {} server(s) will not be initialized. \
                 To enable MCP, compile with nightly Rust and the 'mcp' feature.",
                self.config.servers.len()
            );
        }
        Ok(())
    }

    /// List available MCP tools
    pub async fn list_tools(&self) -> Vec<ToolSpec> {
        // No tools available in stub implementation
        vec![]
    }

    /// Call an MCP tool
    pub async fn call_tool(
        &self,
        name: &str,
        _args: serde_json::Value,
    ) -> Result<ToolResult, GearClawError> {
        Err(GearClawError::tool_not_found(format!(
            "MCP tool '{}' not available (MCP support disabled)",
            name
        )))
    }
}
