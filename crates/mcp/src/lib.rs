use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::warn;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpCapability {
    Enabled,
    Disabled,
}

pub const BUILD_MCP_CAPABILITY: McpCapability = McpCapability::Disabled;

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
}

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

#[derive(Debug, Error)]
pub enum McpError {
    #[error("tool not found: {0}")]
    ToolNotFound(String),
    #[error("{0}")]
    Other(String),
}

pub struct McpManager {
    #[allow(dead_code)]
    config: McpConfig,
}

impl McpManager {
    pub fn new(config: McpConfig) -> Self {
        Self { config }
    }
    pub fn capability(&self) -> McpCapability {
        BUILD_MCP_CAPABILITY
    }

    pub fn is_enabled(&self) -> bool {
        matches!(self.capability(), McpCapability::Enabled)
    }

    pub async fn init_clients(&self) -> Result<(), McpError> {
        if !self.config.servers.is_empty() {
            warn!(
                "MCP servers configured but MCP support is disabled. {} server(s) will not be initialized.",
                self.config.servers.len()
            );
        }
        Ok(())
    }

    pub async fn list_tools(&self) -> Vec<ToolSpec> {
        vec![]
    }

    pub async fn call_tool(
        &self,
        name: &str,
        _args: serde_json::Value,
    ) -> Result<ToolResult, McpError> {
        Err(McpError::ToolNotFound(format!(
            "MCP tool '{}' not available (MCP support disabled)",
            name
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::{McpCapability, McpConfig, McpManager};

    #[test]
    fn build_capability_is_explicitly_disabled() {
        let manager = McpManager::new(McpConfig::default());
        assert_eq!(manager.capability(), McpCapability::Disabled);
        assert!(!manager.is_enabled());
    }
}
