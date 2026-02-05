use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;
use tracing::{info, error, warn};
use rmcp::{
    model::{CallToolRequestParams, PaginatedRequestParams, RawContent, ResourceContents},
    service::{RunningService, RoleClient, serve_client},
    transport::TokioChildProcess,
};

use crate::config::McpConfig;
use crate::error::GearClawError;
use crate::tools::{ToolSpec, ToolResult};

pub struct McpManager {
    clients: Arc<Mutex<HashMap<String, Box<dyn McpClient + Send + Sync>>>>,
    config: McpConfig,
}

// Trait to abstract the client for easier storage
#[async_trait::async_trait]
trait McpClient {
    async fn list_tools(&self) -> Result<Vec<rmcp::model::Tool>, String>;
    async fn call_tool(&self, name: &str, args: serde_json::Value) -> Result<rmcp::model::CallToolResult, String>;
}

type McpServiceClient = RunningService<RoleClient, ()>;

struct RmcpClientWrapper {
    inner: McpServiceClient,
}

#[async_trait::async_trait]
impl McpClient for RmcpClientWrapper {
    async fn list_tools(&self) -> Result<Vec<rmcp::model::Tool>, String> {
        let response: rmcp::model::ListToolsResult = self.inner.list_tools(Some(PaginatedRequestParams::default()))
            .await
            .map_err(|e: rmcp::service::ServiceError| e.to_string())?;
        Ok(response.tools)
    }

    async fn call_tool(&self, name: &str, args: serde_json::Value) -> Result<rmcp::model::CallToolResult, String> {
        let args_obj = args.as_object().cloned();
        
        let result: rmcp::model::CallToolResult = self.inner.call_tool(CallToolRequestParams {
            name: name.to_string().into(),
            arguments: args_obj,
            meta: None,
            task: None,
        })
        .await
        .map_err(|e: rmcp::service::ServiceError| e.to_string())?;
        
        Ok(result)
    }
}

impl McpManager {
    pub fn new(config: McpConfig) -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    pub async fn init_clients(&self) -> Result<(), GearClawError> {
        let mut clients_guard = self.clients.lock().await;

        for (name, server_config) in &self.config.servers {
            info!("Initializing MCP server: {}", name);

            let mut cmd = Command::new(&server_config.command);
            cmd.args(&server_config.args);
            cmd.envs(&server_config.env);
            
            // Need to configure the command for stdio
            let transport_result = TokioChildProcess::new(cmd);
            
            match transport_result {
                Ok(transport) => {
                    // rmcp serve returns a Service. 
                    match serve_client((), transport).await {
                        Ok(service) => {
                            let wrapper = RmcpClientWrapper { inner: service };
                            clients_guard.insert(name.clone(), Box::new(wrapper));
                            info!("MCP server connected: {}", name);
                        },
                        Err(e) => {
                            error!("Failed to connect to MCP server {}: {}", name, e);
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to create transport for MCP server {}: {}", name, e);
                }
            }
        }

        Ok(())
    }

    pub async fn list_tools(&self) -> Vec<ToolSpec> {
        let clients_guard = self.clients.lock().await;
        let mut all_tools = Vec::new();

        for (server_name, client) in clients_guard.iter() {
            match client.list_tools().await {
                Ok(tools) => {
                    for tool in tools {
                        // Prefix tool name with server name to avoid collisions
                        // Format: server_name__tool_name
                        let unique_name = format!("{}__{}", server_name, tool.name);
                        
                        all_tools.push(ToolSpec {
                            name: unique_name,
                            description: tool.description.clone().map(|d| d.into_owned()).unwrap_or_default(),
                            // MCP tools always take arguments (object)
                            requires_args: true,
                            parameters: serde_json::to_value(tool.input_schema).ok(),
                        });
                    }
                },
                Err(e) => {
                    warn!("Failed to list tools for server {}: {}", server_name, e);
                }
            }
        }

        all_tools
    }

    pub async fn call_tool(&self, name: &str, args: serde_json::Value) -> Result<ToolResult, GearClawError> {
        // Parse server name and tool name from unique name
        let parts: Vec<&str> = name.splitn(2, "__").collect();
        if parts.len() != 2 {
            return Err(GearClawError::ToolNotFound(format!("Invalid MCP tool name: {}", name)));
        }

        let server_name = parts[0];
        let tool_name = parts[1];

        let clients_guard = self.clients.lock().await;
        if let Some(client) = clients_guard.get(server_name) {
            match client.call_tool(tool_name, args).await {
                Ok(result) => {
                    // Convert MCP result to ToolResult
                    // Combine all content items into a single string for now
                    let mut output = String::new();
                    let mut error = None;

                    if let Some(is_error) = result.is_error {
                        if is_error {
                            error = Some("Tool reported an error".to_string());
                        }
                    }

                    for content in result.content {
                        match content.raw {
                            RawContent::Text(text) => {
                                output.push_str(&text.text);
                                output.push('\n');
                            },
                            RawContent::Image(img) => {
                                output.push_str(&format!("[Image: {}]\n", img.mime_type));
                            },
                            RawContent::Resource(res) => {
                                let uri = match &res.resource {
                                    ResourceContents::TextResourceContents { uri, .. } => uri,
                                    ResourceContents::BlobResourceContents { uri, .. } => uri,
                                };
                                output.push_str(&format!("[Resource: {}]\n", uri));
                            },
                            RawContent::Audio(audio) => {
                                output.push_str(&format!("[Audio: {}]\n", audio.mime_type));
                            },
                            RawContent::ResourceLink(link) => {
                                output.push_str(&format!("[ResourceLink: {}]\n", link.uri));
                            }
                        }
                    }

                    Ok(ToolResult {
                        success: !result.is_error.unwrap_or(false),
                        output: output.trim().to_string(),
                        error,
                    })
                },
                Err(e) => {
                     Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!("MCP call failed: {}", e)),
                    })
                }
            }
        } else {
            Err(GearClawError::ToolNotFound(format!("MCP server not found: {}", server_name)))
        }
    }
}
