//! MCP error types.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("MCP server '{server}' spawn failed: {reason}")]
    Spawn { server: String, reason: String },

    #[error("MCP server '{server}' I/O error: {reason}")]
    Io { server: String, reason: String },

    #[error("MCP server '{server}' protocol error: {reason}")]
    Protocol { server: String, reason: String },

    #[error("MCP server '{server}' request timed out")]
    Timeout { server: String },

    #[error("MCP tool '{tool}' on server '{server}' failed: {reason}")]
    ToolCall {
        server: String,
        tool: String,
        reason: String,
    },

    #[error("MCP tool not found: {0}")]
    ToolNotFound(String),

    #[error("{0}")]
    Other(String),
}
