use crate::auth::TokenAuth;
use crate::handlers::MethodHandlers;
use crate::protocol::*;
use anyhow::{Context, Result};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::tungstenite::Message;

/// Gateway configuration
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub host: String,
    pub port: u16,
    pub ws_path: String,
    pub allow_unauthenticated_requests: bool,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 18789,
            ws_path: "/ws".to_string(),
            allow_unauthenticated_requests: false,
        }
    }
}

const MAX_SESSION_ID_LENGTH: usize = 128;

fn validate_agent_session_id(session_id: &str) -> Result<(), ProtocolError> {
    if session_id.trim().is_empty() {
        return Err(ProtocolError::new(
            ProtocolError::INVALID_REQUEST,
            "Invalid session id: cannot be empty",
        ));
    }
    if session_id.len() > MAX_SESSION_ID_LENGTH {
        return Err(ProtocolError::new(
            ProtocolError::INVALID_REQUEST,
            format!(
                "Invalid session id: too long (max {})",
                MAX_SESSION_ID_LENGTH
            ),
        ));
    }
    if session_id == "." || session_id == ".." || session_id.contains("..") {
        return Err(ProtocolError::new(
            ProtocolError::INVALID_REQUEST,
            "Invalid session id: path traversal sequence is not allowed",
        ));
    }
    if session_id.contains('/') || session_id.contains('\\') {
        return Err(ProtocolError::new(
            ProtocolError::INVALID_REQUEST,
            "Invalid session id: path separators are not allowed",
        ));
    }
    if !session_id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':'))
    {
        return Err(ProtocolError::new(
            ProtocolError::INVALID_REQUEST,
            "Invalid session id: contains unsupported characters",
        ));
    }
    Ok(())
}

fn validate_request(request: &GatewayRequest) -> Result<(), ProtocolError> {
    if request.id.trim().is_empty() {
        return Err(ProtocolError::new(
            ProtocolError::INVALID_REQUEST,
            "Request id cannot be empty",
        ));
    }

    if request.method == "agent" {
        let session_id = request.device_id.as_deref().unwrap_or(request.id.as_str());
        validate_agent_session_id(session_id)?;
    }

    Ok(())
}

async fn authorize_request(
    auth: &TokenAuth,
    request: &GatewayRequest,
    allow_unauthenticated_requests: bool,
) -> Result<(), ProtocolError> {
    if allow_unauthenticated_requests {
        return Ok(());
    }

    let token = request.signature.as_deref().ok_or_else(|| {
        ProtocolError::new(
            ProtocolError::UNAUTHORIZED,
            "Authentication required: missing token signature",
        )
    })?;

    if !auth.validate(token).await {
        return Err(ProtocolError::new(
            ProtocolError::UNAUTHORIZED,
            "Invalid or expired token",
        ));
    }

    auth.update_last_used(token).await;
    Ok(())
}

/// Active connection information
#[derive(Debug, Clone)]
pub struct ActiveConnection {
    pub id: String,
    pub addr: String,
    pub connected_at: std::time::SystemTime,
}

/// Gateway server
pub struct GatewayServer {
    config: GatewayConfig,
    handlers: Arc<MethodHandlers>,
    auth: Arc<TokenAuth>,
    connections: Arc<RwLock<Vec<ActiveConnection>>>,
    /// Event broadcast channel - sends events to all connected clients
    event_tx: broadcast::Sender<GatewayEvent>,
}

impl GatewayServer {
    pub fn new(config: GatewayConfig) -> Self {
        // Create event broadcast channel with capacity for 100 events
        let (event_tx, _) = broadcast::channel(100);

        Self {
            config,
            handlers: Arc::new(MethodHandlers::new()),
            auth: Arc::new(TokenAuth::new()),
            connections: Arc::new(RwLock::new(Vec::new())),
            event_tx,
        }
    }

    pub fn with_handlers(mut self, handlers: Arc<MethodHandlers>) -> Self {
        self.handlers = handlers;
        self
    }

    pub fn with_auth(mut self, auth: Arc<TokenAuth>) -> Self {
        self.auth = auth;
        self
    }

    /// Register a channel adapter
    pub async fn register_channel<T: gearclaw_channels::ChannelAdapter + Send + Sync + 'static>(
        &self,
        adapter: T,
    ) -> Result<()> {
        let channel_manager = self.handlers.channel_manager();
        let mut manager = channel_manager.lock().await;
        manager.register(Box::new(adapter));
        tracing::info!("Channel adapter registered: {}", std::any::type_name::<T>());
        Ok(())
    }

    /// Get event sender for broadcasting events to all clients
    pub fn event_sender(&self) -> broadcast::Sender<GatewayEvent> {
        self.event_tx.clone()
    }

    /// Get handlers reference for configuration (e.g., setting Agent)
    pub fn handlers(&self) -> Arc<MethodHandlers> {
        self.handlers.clone()
    }

    pub async fn start(self) -> Result<()> {
        use tokio::net::TcpListener;

        // Start channel message listener
        self.start_channel_listener().await?;

        // Note: Channel adapters should be initialized and started before calling this method
        // Example:
        //   let mut discord = DiscordAdapter::from_env()?;
        //   discord.start().await?;
        //   channel_manager.register(Box::new(discord));

        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&addr).await?;
        tracing::info!("Gateway server listening on {}", addr);

        loop {
            if let Ok((stream, addr)) = listener.accept().await {
                let handlers = self.handlers.clone();
                let auth = self.auth.clone();
                let connections = self.connections.clone();
                let event_rx = self.event_tx.subscribe();
                let allow_unauthenticated_requests = self.config.allow_unauthenticated_requests;
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(
                        stream,
                        addr.to_string(),
                        handlers,
                        auth,
                        connections,
                        event_rx,
                        allow_unauthenticated_requests,
                    )
                    .await
                    {
                        tracing::error!("Connection error: {}", e);
                    }
                });
            }
        }
    }

    /// Start background task to listen for channel messages and broadcast them
    async fn start_channel_listener(&self) -> Result<()> {
        use futures_util::StreamExt;

        let channel_manager = self.handlers.channel_manager();
        let event_tx = self.event_tx.clone();
        let handlers = self.handlers.clone();

        tokio::spawn(async move {
            tracing::info!("Channel message listener started");

            // Get list of registered platforms
            let platforms = {
                let mgr = channel_manager.lock().await;
                mgr.platforms()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            };

            // Subscribe to messages from each platform
            for platform in platforms {
                let platform = platform.clone();
                let event_tx = event_tx.clone();
                let channel_manager = channel_manager.clone();
                let handlers_clone = handlers.clone();

                tokio::spawn(async move {
                    let mgr = channel_manager.lock().await;
                    if let Some(adapter) = mgr.get(&platform) {
                        let mut message_stream = adapter.on_message();

                        tracing::info!("Listening for messages from platform: {}", platform);

                        while let Some(incoming) = message_stream.next().await {
                            tracing::debug!(
                                "Received message from {}: {}",
                                platform,
                                incoming.content
                            );

                            // Convert IncomingMessage to GatewayEvent
                            let source = match &incoming.source {
                                gearclaw_channels::MessageSource::User { id, name } => {
                                    ChannelSource::User {
                                        id: id.clone(),
                                        name: name.clone(),
                                    }
                                }
                                gearclaw_channels::MessageSource::Channel { id, name } => {
                                    ChannelSource::Channel {
                                        id: id.clone(),
                                        name: name.clone(),
                                    }
                                }
                                gearclaw_channels::MessageSource::Group { id, name } => {
                                    ChannelSource::Group {
                                        id: id.clone(),
                                        name: name.clone(),
                                    }
                                }
                            };

                            let event = GatewayEvent::ChannelMessage(ChannelMessageEvent {
                                platform: incoming.platform.clone(),
                                source: source.clone(),
                                content: incoming.content.clone(),
                                metadata: Some(incoming.metadata.clone()),
                                ts: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs() as i64,
                            });

                            // Broadcast to all connected clients
                            let _ = event_tx.send(event);

                            // Check if Agent should be triggered based on config
                            let trigger_config = handlers_clone.get_trigger_config().await;
                            let should_trigger = crate::triggers::should_trigger_agent(
                                &platform,
                                &source,
                                &incoming.content,
                                &trigger_config,
                            );

                            if should_trigger {
                                // Trigger Agent processing if available
                                if let Some(agent) = handlers_clone.get_agent().await {
                                    let platform_clone = platform.clone();
                                    let source_clone = source.clone();
                                    let content_clone = incoming.content.clone();
                                    let channel_mgr = channel_manager.clone();

                                    // Process in background task
                                    tokio::spawn(async move {
                                        if let Err(e) = process_agent_response(
                                            agent,
                                            &platform_clone,
                                            &source_clone,
                                            &content_clone,
                                            channel_mgr,
                                        )
                                        .await
                                        {
                                            tracing::error!("Agent processing failed: {}", e);
                                        }
                                    });
                                }
                            }
                        }
                    }
                });
            }

            Ok::<(), anyhow::Error>(())
        });

        Ok(())
    }
}

// Handle a WebSocket connection
async fn handle_connection(
    stream: tokio::net::TcpStream,
    addr: String,
    handlers: Arc<MethodHandlers>,
    auth: Arc<TokenAuth>,
    _connections: Arc<RwLock<Vec<ActiveConnection>>>,
    mut event_rx: broadcast::Receiver<GatewayEvent>,
    allow_unauthenticated_requests: bool,
) -> Result<()> {
    // Upgrade to WebSocket
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .context("WebSocket handshake failed")?;

    tracing::info!("WebSocket connection established from {}", addr);

    let conn_id = uuid::Uuid::new_v4().to_string();
    let (mut ws_sender, mut ws_receiver) = ws_stream.split::<Message>();

    // Send hello-ok
    let hello_payload = serde_json::to_value(create_hello_ok()).unwrap();
    let hello_response =
        GatewayFrame::Response(GatewayResponse::ok("hello".to_string(), hello_payload));
    let hello_msg = serde_json::to_string(&hello_response)?;
    ws_sender.send(Message::Text(hello_msg.into())).await?;

    // Connection loop with both request handling and event broadcasting
    loop {
        tokio::select! {
            // Handle incoming WebSocket messages (requests)
            msg_result = ws_receiver.next() => {
                match msg_result {
                    Some(Ok(msg)) => {
                        if msg.is_text() {
                            let text = msg.to_text()?;
                            tracing::debug!("Received message: {}", text);

                            // Parse frame
                            if let Ok(frame) = serde_json::from_str::<GatewayFrame>(text) {
                                if let GatewayFrame::Request(request) = frame {
                                    // Handle request
                                    let response = handle_request(
                                        &request,
                                        &handlers,
                                        &auth,
                                        allow_unauthenticated_requests,
                                    )
                                    .await;

                                    // Send response
                                    let response_msg = serde_json::to_string(&response)?;
                                    ws_sender.send(Message::Text(response_msg.into())).await?;
                                }
                            }
                        } else if msg.is_close() {
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        tracing::error!("WebSocket error: {}", e);
                        break;
                    }
                    None => break,
                }
            }

            // Handle broadcasted events
            event_result = event_rx.recv() => {
                match event_result {
                    Ok(event) => {
                        // Send event to client
                        let event_frame = GatewayFrame::Event(event);
                        let event_msg = serde_json::to_string(&event_frame)?;
                        ws_sender.send(Message::Text(event_msg.into())).await?;
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!("Event receiver lagged, skipped {} events", skipped);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        tracing::debug!("Event channel closed");
                        break;
                    }
                }
            }
        }
    }

    // Clean up connection
    tracing::info!("Connection {} closed", conn_id);
    Ok(())
}

/// Handle incoming request
async fn handle_request(
    request: &GatewayRequest,
    handlers: &MethodHandlers,
    auth: &TokenAuth,
    allow_unauthenticated_requests: bool,
) -> GatewayResponse {
    if let Err(error) = authorize_request(auth, request, allow_unauthenticated_requests).await {
        return GatewayResponse::error(request.id.clone(), error);
    }
    if let Err(error) = validate_request(request) {
        return GatewayResponse::error(request.id.clone(), error);
    }

    // Route to handler
    let result = match request.method.as_str() {
        "health" => handlers.health(request).await,
        "status" => handlers.status(request).await,
        "send" => handlers.send(request).await,
        "agent" => handlers.agent_execute(request).await,
        _ => Err(anyhow::anyhow!("Unknown method: {}", request.method)),
    };

    match result {
        Ok(payload) => GatewayResponse::ok(request.id.clone(), payload),
        Err(e) => GatewayResponse::error(
            request.id.clone(),
            ProtocolError::new("INTERNAL_ERROR", e.to_string()),
        ),
    }
}

/// Process message through Agent and send response back to channel
async fn process_agent_response(
    agent: Arc<gearclaw_agent::Agent>,
    platform: &str,
    source: &ChannelSource,
    content: &str,
    channel_manager: Arc<tokio::sync::Mutex<gearclaw_channels::ChannelManager>>,
) -> Result<()> {
    use gearclaw_channels::MessageContent;

    tracing::info!(
        "Processing message from {}:{} with Agent",
        platform,
        match source {
            ChannelSource::User { name, .. } => name,
            ChannelSource::Channel { name, .. } => name,
            ChannelSource::Group { name, .. } => name,
        }
    );

    // Create session ID from platform and source info
    let session_id = match source {
        ChannelSource::User { id, .. } => format!("{}:user:{}", platform, id),
        ChannelSource::Channel { id, .. } => format!("{}:channel:{}", platform, id),
        ChannelSource::Group { id, .. } => format!("{}:group:{}", platform, id),
    };

    // Get or create session
    let mut session = agent
        .session_manager
        .get_or_create_session(&session_id)
        .map_err(|e| anyhow::anyhow!("Failed to get session: {}", e))?;

    // Add context about message source
    let context_msg = format!(
        "[Message from {}:{}]: {}",
        platform,
        match source {
            ChannelSource::User { name, .. } => name,
            ChannelSource::Channel { name, .. } => name,
            ChannelSource::Group { name, .. } => name,
        },
        content
    );

    // Process with agent
    let response = agent
        .process_message(&mut session, &context_msg)
        .await
        .map_err(|e| anyhow::anyhow!("Agent processing failed: {}", e))?;

    // Save session
    agent
        .session_manager
        .save_session(&session)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to save session: {}", e))?;

    // Extract the actual response (remove context prefix if present)
    let agent_response = if response.starts_with("[Message from") {
        // Try to extract just the agent's response
        if let Some(idx) = response.find("]: ") {
            let after_context = &response[idx + 2..];
            after_context.trim().to_string()
        } else {
            response
        }
    } else {
        response
    };

    tracing::info!("Agent response: {}", agent_response);

    // Send response back to the channel
    let mgr = channel_manager.lock().await;
    if let Some(adapter) = mgr.get(platform) {
        // Determine target from source
        let target_identifier = match source {
            ChannelSource::User { id, .. } => id.clone(),
            ChannelSource::Channel { id, .. } => id.clone(),
            ChannelSource::Group { id, .. } => id.clone(),
        };

        let target = adapter
            .resolve_target(&target_identifier)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to resolve target: {}", e))?;

        let message_content = MessageContent {
            text: Some(agent_response),
            embeds: vec![],
        };

        adapter
            .send_message(target, message_content)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send agent response: {}", e))?;

        tracing::info!("Agent response sent to {}:{}", platform, target_identifier);
    } else {
        tracing::warn!("No adapter found for platform: {}", platform);
    }

    Ok(())
}

/// Create hello-ok payload
fn create_hello_ok() -> HelloOkPayload {
    HelloOkPayload {
        protocol: ProtocolVersion { min: 1, max: 1 },
        presence: vec![],
        health: serde_json::json!({
            "status": "ok",
            "version": env!("CARGO_PKG_VERSION"),
        }),
        state_version: StateVersion {
            presence: 0,
            health: 0,
        },
        uptime_ms: 0,
        policy: GatewayPolicy {
            max_payload: 1024 * 1024,             // 1MB
            max_buffered_bytes: 10 * 1024 * 1024, // 10MB
            tick_interval_ms: 30000,              // 30 seconds
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn health_request(signature: Option<&str>) -> GatewayRequest {
        let mut request =
            GatewayRequest::new("req-1".to_string(), "health".to_string(), Value::Null);
        request.signature = signature.map(ToString::to_string);
        request
    }

    fn agent_request(
        request_id: &str,
        signature: Option<&str>,
        device_id: Option<&str>,
    ) -> GatewayRequest {
        let mut request = GatewayRequest::new(
            request_id.to_string(),
            "agent".to_string(),
            json!({ "prompt": "hello" }),
        );
        request.signature = signature.map(ToString::to_string);
        request.device_id = device_id.map(ToString::to_string);
        request
    }

    #[test]
    fn test_config_defaults() {
        let config = GatewayConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 18789);
        assert_eq!(config.ws_path, "/ws");
        assert!(!config.allow_unauthenticated_requests);
    }

    #[tokio::test]
    async fn test_handle_request_rejects_missing_signature() {
        let handlers = MethodHandlers::new();
        let auth = TokenAuth::new();
        let request = health_request(None);

        let response = handle_request(&request, &handlers, &auth, false).await;
        assert!(!response.ok);
        assert_eq!(
            response.error.as_ref().map(|e| e.code.as_str()),
            Some(ProtocolError::UNAUTHORIZED)
        );
    }

    #[tokio::test]
    async fn test_handle_request_rejects_invalid_signature() {
        let handlers = MethodHandlers::new();
        let auth = TokenAuth::new();
        let request = health_request(Some("Bearer invalid-token"));

        let response = handle_request(&request, &handlers, &auth, false).await;
        assert!(!response.ok);
        assert_eq!(
            response.error.as_ref().map(|e| e.code.as_str()),
            Some(ProtocolError::UNAUTHORIZED)
        );
    }

    #[tokio::test]
    async fn test_handle_request_accepts_valid_signature() {
        let handlers = MethodHandlers::new();
        let auth = TokenAuth::new();
        let token = "test-token-abcdefghijklmnopqrstuvwxyz123456";
        auth.register(
            token.to_string(),
            "device-1".to_string(),
            "gateway".to_string(),
        )
        .await
        .unwrap();

        let request = health_request(Some(&format!("Bearer {}", token)));
        let response = handle_request(&request, &handlers, &auth, false).await;
        assert!(response.ok);
    }

    #[tokio::test]
    async fn test_handle_request_bypass_when_allow_unauthenticated_enabled() {
        let handlers = MethodHandlers::new();
        let auth = TokenAuth::new();
        let request = health_request(None);

        let response = handle_request(&request, &handlers, &auth, true).await;
        assert!(response.ok);
    }

    #[tokio::test]
    async fn test_handle_request_rejects_invalid_agent_device_id() {
        let handlers = MethodHandlers::new();
        let auth = TokenAuth::new();
        let token = "test-token-abcdefghijklmnopqrstuvwxyz123456";
        auth.register(
            token.to_string(),
            "device-1".to_string(),
            "gateway".to_string(),
        )
        .await
        .unwrap();

        let request = agent_request(
            "req-1",
            Some(&format!("Bearer {}", token)),
            Some("../escape"),
        );
        let response = handle_request(&request, &handlers, &auth, false).await;
        assert!(!response.ok);
        assert_eq!(
            response.error.as_ref().map(|e| e.code.as_str()),
            Some(ProtocolError::INVALID_REQUEST)
        );
    }

    #[tokio::test]
    async fn test_handle_request_rejects_invalid_agent_request_id_when_no_device_id() {
        let handlers = MethodHandlers::new();
        let auth = TokenAuth::new();
        let token = "test-token-abcdefghijklmnopqrstuvwxyz123456";
        auth.register(
            token.to_string(),
            "device-1".to_string(),
            "gateway".to_string(),
        )
        .await
        .unwrap();

        let request = agent_request("../escape", Some(&format!("Bearer {}", token)), None);
        let response = handle_request(&request, &handlers, &auth, false).await;
        assert!(!response.ok);
        assert_eq!(
            response.error.as_ref().map(|e| e.code.as_str()),
            Some(ProtocolError::INVALID_REQUEST)
        );
    }
}
