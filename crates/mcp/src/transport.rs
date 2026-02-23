//! MCP stdio transport layer.
//!
//! Spawns an MCP server subprocess and communicates via JSON-RPC 2.0
//! over stdin/stdout. Each message is a newline-terminated JSON string.

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::{oneshot, Mutex};
use tokio::time::timeout;
use tracing::{debug, warn};

use crate::error::McpError;
use crate::protocol::{
    IncomingMessage, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse,
};

/// Timeout for individual request/response pairs.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Pending request waiting for a response.
type PendingMap = Arc<Mutex<HashMap<u64, oneshot::Sender<JsonRpcResponse>>>>;

/// Manages the stdio transport connection to a single MCP server process.
pub struct StdioTransport {
    stdin: Arc<Mutex<ChildStdin>>,
    _child: Arc<Mutex<Child>>,
    next_id: Arc<AtomicU64>,
    pending: PendingMap,
    server_name: String,
}

impl StdioTransport {
    /// Spawn a new MCP server process and start the read loop.
    pub async fn spawn(
        server_name: impl Into<String>,
        command: &str,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<Self, McpError> {
        let server_name = server_name.into();

        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null()); // suppress server stderr noise

        for (k, v) in env {
            cmd.env(k, v);
        }

        let mut child = cmd.spawn().map_err(|e| McpError::Spawn {
            server: server_name.clone(),
            reason: format!("Failed to spawn '{}': {}", command, e),
        })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| McpError::Other("Failed to capture stdin".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| McpError::Other("Failed to capture stdout".to_string()))?;

        let pending: PendingMap = Arc::new(Mutex::new(HashMap::new()));
        let pending_clone = pending.clone();
        let name_clone = server_name.clone();

        // Spawn background task to read responses from stdout
        tokio::spawn(Self::read_loop(stdout, pending_clone, name_clone));

        Ok(Self {
            stdin: Arc::new(Mutex::new(stdin)),
            _child: Arc::new(Mutex::new(child)),
            next_id: Arc::new(AtomicU64::new(1)),
            pending,
            server_name,
        })
    }

    /// Send a JSON-RPC request and wait for the matching response.
    pub async fn send_request(&self, method: &str, params: Option<Value>) -> Result<JsonRpcResponse, McpError> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };

        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending.lock().await;
            pending.insert(id, tx);
        }

        // Write request to stdin
        let mut line = serde_json::to_string(&request).map_err(|e| McpError::Protocol {
            server: self.server_name.clone(),
            reason: format!("Serialize failed: {}", e),
        })?;
        line.push('\n');

        debug!("[{}] → {}", self.server_name, line.trim());

        {
            let mut stdin = self.stdin.lock().await;
            stdin.write_all(line.as_bytes()).await.map_err(|e| McpError::Io {
                server: self.server_name.clone(),
                reason: format!("Write failed: {}", e),
            })?;
            stdin.flush().await.map_err(|e| McpError::Io {
                server: self.server_name.clone(),
                reason: format!("Flush failed: {}", e),
            })?;
        }

        // Wait for response with timeout
        let response = timeout(REQUEST_TIMEOUT, rx).await
            .map_err(|_| McpError::Timeout { server: self.server_name.clone() })?
            .map_err(|_| McpError::Other(format!("[{}] Response channel closed", self.server_name)))?;

        Ok(response)
    }

    /// Send a JSON-RPC notification (no response expected).
    pub async fn send_notification(&self, method: &str) -> Result<(), McpError> {
        let notification = JsonRpcNotification::new(method);
        let mut line = serde_json::to_string(&notification).map_err(|e| McpError::Protocol {
            server: self.server_name.clone(),
            reason: format!("Serialize notification failed: {}", e),
        })?;
        line.push('\n');

        debug!("[{}] → (notification) {}", self.server_name, line.trim());

        let mut stdin = self.stdin.lock().await;
        stdin.write_all(line.as_bytes()).await.map_err(|e| McpError::Io {
            server: self.server_name.clone(),
            reason: format!("Write notification failed: {}", e),
        })?;
        stdin.flush().await.map_err(|e| McpError::Io {
            server: self.server_name.clone(),
            reason: format!("Flush notification failed: {}", e),
        })?;

        Ok(())
    }

    /// Background task: reads stdout line-by-line and dispatches responses.
    async fn read_loop(stdout: ChildStdout, pending: PendingMap, server_name: String) {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    // EOF — server exited
                    debug!("[{}] stdout closed (server exited)", server_name);
                    break;
                }
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    debug!("[{}] ← {}", server_name, trimmed);

                    match serde_json::from_str::<IncomingMessage>(trimmed) {
                        Ok(msg) if msg.is_response() => {
                            if let Some(id) = msg.id {
                                let mut pending = pending.lock().await;
                                if let Some(tx) = pending.remove(&id) {
                                    let _ = tx.send(msg.as_response());
                                }
                            }
                        }
                        Ok(_) => {
                            // Server-initiated notification — ignore for now
                        }
                        Err(e) => {
                            warn!("[{}] Failed to parse server message: {} — {}", server_name, e, trimmed);
                        }
                    }
                }
                Err(e) => {
                    warn!("[{}] Read error: {}", server_name, e);
                    break;
                }
            }
        }

        // Clean up pending requests on disconnect
        let mut pending = pending.lock().await;
        pending.clear();
    }
}
