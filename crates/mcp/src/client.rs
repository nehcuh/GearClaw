//! MCP single-server client.
//!
//! Wraps `StdioTransport` and handles the MCP protocol handshake,
//! tool enumeration, and tool invocation for one server.

use std::collections::HashMap;

use serde_json::Value;
use tracing::{info, warn};

use crate::error::McpError;
use crate::protocol::{
    ContentItem, InitializeParams, InitializeResult, ToolCallParams, ToolCallResult, ToolsListResult,
    McpTool,
};
use crate::transport::StdioTransport;
use crate::McpServerConfig;

/// Status of an MCP server connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Failed(String),
}

impl std::fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionStatus::Connected => write!(f, "Connected"),
            ConnectionStatus::Disconnected => write!(f, "Disconnected"),
            ConnectionStatus::Failed(reason) => write!(f, "Failed: {}", reason),
        }
    }
}

/// A connected client for a single MCP server.
pub struct McpClient {
    pub name: String,
    transport: StdioTransport,
    pub tools: Vec<McpTool>,
    pub status: ConnectionStatus,
    pub server_info: Option<String>,
}

impl McpClient {
    /// Spawn the MCP server process, complete the initialize handshake,
    /// and enumerate the available tools.
    pub async fn connect(name: impl Into<String>, config: &McpServerConfig) -> Result<Self, McpError> {
        let name = name.into();

        info!("[MCP] Connecting to server '{}'...", name);

        let transport = StdioTransport::spawn(
            &name,
            &config.command,
            &config.args,
            &config.env,
        )
        .await?;

        let mut client = Self {
            name: name.clone(),
            transport,
            tools: vec![],
            status: ConnectionStatus::Disconnected,
            server_info: None,
        };

        // Step 1: send initialize
        let init_params = InitializeParams::default();
        let init_value = serde_json::to_value(init_params)
            .map_err(|e| McpError::Protocol { server: name.clone(), reason: e.to_string() })?;

        let response = client.transport.send_request("initialize", Some(init_value)).await?;

        if let Some(err) = &response.error {
            return Err(McpError::Protocol {
                server: name.clone(),
                reason: format!("initialize error: {}", err.message),
            });
        }

        if let Some(result_value) = response.result {
            match serde_json::from_value::<InitializeResult>(result_value) {
                Ok(init_result) => {
                    client.server_info = init_result.server_info.map(|si| {
                        format!("{} v{}", si.name, si.version)
                    });
                    info!("[MCP] '{}' initialized (protocol: {}, server: {:?})",
                        name, init_result.protocol_version, client.server_info);
                }
                Err(e) => {
                    warn!("[MCP] '{}' initialize result parse warning: {}", name, e);
                }
            }
        }

        // Step 2: send notifications/initialized
        client.transport.send_notification("notifications/initialized").await?;

        // Step 3: enumerate tools
        client.tools = client.fetch_tools().await?;
        client.status = ConnectionStatus::Connected;

        info!("[MCP] '{}' ready — {} tools available", name, client.tools.len());

        Ok(client)
    }

    /// Re-fetch the tools list from the server (useful after server updates).
    pub async fn refresh_tools(&mut self) -> Result<(), McpError> {
        self.tools = self.fetch_tools().await?;
        Ok(())
    }

    /// Fetch the list of tools from the server.
    async fn fetch_tools(&self) -> Result<Vec<McpTool>, McpError> {
        let response = self.transport.send_request("tools/list", None).await?;

        if let Some(err) = &response.error {
            return Err(McpError::Protocol {
                server: self.name.clone(),
                reason: format!("tools/list error: {}", err.message),
            });
        }

        let result_value = response.result.ok_or_else(|| McpError::Protocol {
            server: self.name.clone(),
            reason: "tools/list returned empty result".to_string(),
        })?;

        let list_result = serde_json::from_value::<ToolsListResult>(result_value)
            .map_err(|e| McpError::Protocol {
                server: self.name.clone(),
                reason: format!("tools/list parse failed: {}", e),
            })?;

        Ok(list_result.tools)
    }

    /// Call a tool by its bare name (without the `server__` prefix).
    /// Returns the combined text output from all content items.
    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: Value,
    ) -> Result<String, McpError> {
        let args_map: HashMap<String, Value> = match arguments {
            Value::Object(map) => map.into_iter().collect(),
            Value::Null => HashMap::new(),
            other => {
                // Try to wrap scalar in a "value" key as a best-effort
                let mut m = HashMap::new();
                m.insert("value".to_string(), other);
                m
            }
        };

        let params = ToolCallParams {
            name: tool_name.to_string(),
            arguments: args_map,
        };
        let params_value = serde_json::to_value(params)
            .map_err(|e| McpError::Protocol { server: self.name.clone(), reason: e.to_string() })?;

        let response = self.transport.send_request("tools/call", Some(params_value)).await?;

        if let Some(err) = &response.error {
            return Err(McpError::ToolCall {
                server: self.name.clone(),
                tool: tool_name.to_string(),
                reason: err.message.clone(),
            });
        }

        let result_value = response.result.ok_or_else(|| McpError::Protocol {
            server: self.name.clone(),
            reason: "tools/call returned empty result".to_string(),
        })?;

        let call_result = serde_json::from_value::<ToolCallResult>(result_value)
            .map_err(|e| McpError::Protocol {
                server: self.name.clone(),
                reason: format!("tools/call parse failed: {}", e),
            })?;

        if call_result.is_error {
            let msg = call_result
                .content
                .iter()
                .filter_map(|c| c.as_text())
                .collect::<Vec<_>>()
                .join("\n");
            return Err(McpError::ToolCall {
                server: self.name.clone(),
                tool: tool_name.to_string(),
                reason: msg,
            });
        }

        let output = call_result
            .content
            .iter()
            .filter_map(|item| match item {
                ContentItem::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(output)
    }

    /// Check whether this client has a tool with the given bare name.
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.iter().any(|t| t.name == name)
    }
}
