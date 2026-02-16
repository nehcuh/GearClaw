// Discord Platform Adapter
//
// Implements Discord bot integration using twilight-rs library.

use crate::adapter::{ChannelAdapter, ChannelError, MessageTarget, MessageContent, IncomingMessage, MessageSource};
use async_trait::async_trait;
use serde_json::json;
use std::pin::Pin;
use std::sync::Arc;
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt as TwilightStreamExt};
use twilight_http::Client as HttpClient;
use twilight_model::id::{marker::ChannelMarker, Id};
use tokio::sync::broadcast;

/// Discord adapter configuration
#[derive(Debug, Clone)]
pub struct DiscordConfig {
    /// Bot token
    pub bot_token: String,

    /// Message character limit
    pub message_limit: usize,
}

impl Default for DiscordConfig {
    fn default() -> Self {
        Self {
            bot_token: std::env::var("DISCORD_BOT_TOKEN")
                .unwrap_or_else(|_| String::new()),
            message_limit: 2000,
        }
    }
}

/// Discord adapter
pub struct DiscordAdapter {
    config: DiscordConfig,
    http: Arc<HttpClient>,
    message_tx: Arc<tokio::sync::Mutex<Option<broadcast::Sender<IncomingMessage>>>>,
}

impl DiscordAdapter {
    /// Create new Discord adapter
    pub fn new(config: DiscordConfig) -> Self {
        let http = HttpClient::new(config.bot_token.clone());
        let (tx, _) = broadcast::channel(100);
        Self {
            config,
            http: Arc::new(http),
            message_tx: Arc::new(tokio::sync::Mutex::new(Some(tx))),
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self, ChannelError> {
        let bot_token = std::env::var("DISCORD_BOT_TOKEN")
            .map_err(|_| ChannelError::AuthenticationFailed {
                platform: "discord".to_string(),
                source: "DISCORD_BOT_TOKEN not set".to_string(),
            })?;

        Ok(Self::new(DiscordConfig {
            bot_token,
            ..Default::default()
        }))
    }

    /// Split message into chunks if too long
    pub fn chunk_message(message: &str, limit: usize) -> Vec<String> {
        if message.len() <= limit {
            return vec![message.to_string()];
        }

        let mut chunks = Vec::new();
        let mut current = String::new();

        for line in message.lines() {
            if current.len() + line.len() + 1 > limit {
                if !current.is_empty() {
                    chunks.push(std::mem::take(&mut current));
                }
                current = line.to_string();
            } else {
                if !current.is_empty() {
                    current.push('\n');
                }
                current.push_str(line);
            }
        }

        if !current.is_empty() {
            chunks.push(current);
        }

        chunks
    }

    /// Parse channel ID from various formats
    pub fn parse_channel_id(identifier: &str) -> Result<u64, ChannelError> {
        if identifier.starts_with('<') && identifier.ends_with('>') {
            // Channel mention <#123456789>
            let id_str = identifier.trim_start_matches("<#").trim_end_matches(">");
            id_str.parse::<u64>().map_err(|_| ChannelError::ResolveFailed {
                identifier: identifier.to_string(),
                source: "Invalid snowflake ID".to_string(),
            })
        } else if identifier.chars().all(|c| c.is_numeric()) && identifier.len() <= 20 {
            // Raw snowflake ID
            identifier.parse::<u64>().map_err(|_| ChannelError::ResolveFailed {
                identifier: identifier.to_string(),
                source: "Invalid snowflake ID".to_string(),
            })
        } else {
            Err(ChannelError::ResolveFailed {
                identifier: identifier.to_string(),
                source: "Channel name resolution not implemented".to_string(),
            })
        }
    }
}

#[async_trait]
impl ChannelAdapter for DiscordAdapter {
    fn platform_name(&self) -> &str {
        "discord"
    }

    async fn start(&mut self) -> Result<(), ChannelError> {
        tracing::info!("Discord adapter starting...");

        // Clone Arcs for the spawned task
        let message_tx = self.message_tx.clone();
        let token = self.config.bot_token.clone();

        // Spawn gateway task in background
        tokio::spawn(async move {
            tracing::info!("Discord Gateway task starting");

            // Create shard with message intents
            let intents = Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT;

            tracing::info!("Creating Discord Gateway shard with intents: {:?}", intents);

            let mut shard = Shard::new(ShardId::ONE, token, intents);

            tracing::info!("✅ Discord Gateway shard created successfully");
            tracing::info!("Discord Gateway shard created, connecting...");

            // Listen for all events to keep connection alive
            tracing::info!("Waiting for Discord events...");

            // Use all event types to keep connection alive
            let event_types = EventTypeFlags::all();

            let mut event_count = 0;

            while let Some(item) = shard.next_event(event_types).await {
                event_count += 1;

                let event = match item {
                    Ok(event) => {
                        // Identify event type for debugging
                        let event_name = match &event {
                            Event::Ready(_) => "Ready",
                            Event::MessageCreate(_) => "MessageCreate",
                            Event::GuildCreate(_) => "GuildCreate",
                            _ => "Other",
                        };
                        tracing::info!("Received Discord event #{}: {} ({:?})", event_count, event_name, std::mem::discriminant(&event));
                        event
                    }
                    Err(e) => {
                        tracing::error!("Error receiving Discord event: {}", e);

                        // Check for common errors
                        let error_str = e.to_string().to_lowercase();
                        if error_str.contains("401") || error_str.contains("unauthorized") {
                            tracing::error!("❌ Discord Bot Token is invalid or expired!");
                            tracing::error!("Please check your DISCORD_BOT_TOKEN environment variable");
                        } else if error_str.contains("disallowed intent") {
                            tracing::error!("❌ MESSAGE CONTENT INTENT is not enabled!");
                            tracing::error!("Go to: https://discord.com/developers/applications");
                            tracing::error!("Your App → Bot → Privileged Gateway Intents");
                            tracing::error!("✅ Enable 'MESSAGE CONTENT INTENT' → Save Changes");
                        } else if error_str.contains("403") {
                            tracing::error!("❌ Bot lacks permissions. Please check bot permissions in Discord server settings");
                        }

                        continue;
                    }
                };

                // Process only message create events
                if let Event::MessageCreate(msg) = event {
                    // Skip messages from bots (including self)
                    if msg.author.bot {
                        tracing::debug!("⏭️  Skipping bot message from {}", msg.author.name);
                        continue;
                    }

                    tracing::info!("📨 Processing MessageCreate: author={}, content={}",
                        msg.author.name,
                        msg.content.chars().take(50).collect::<String>()
                    );

                    // Convert Discord message to IncomingMessage
                    let incoming = IncomingMessage {
                        platform: "discord".to_string(),
                        source: MessageSource::User {
                            id: msg.author.id.get().to_string(),
                            name: msg.author.name.clone(),
                        },
                        content: msg.content.clone(),
                        metadata: json!({
                            "channel_id": msg.channel_id.get().to_string(),
                            "guild_id": msg.guild_id.map(|id| id.get().to_string()),
                            "message_id": msg.id.get().to_string(),
                        }),
                    };

                    // Send to broadcast channel (ignore if no receivers)
                    let tx = message_tx.lock().await;
                    if let Some(ref tx) = *tx {
                        match tx.send(incoming) {
                            Ok(_) => {
                                tracing::info!("✅ Message sent to broadcast channel");
                            }
                            Err(e) => {
                                tracing::error!("❌ Failed to send to broadcast channel: {}", e);
                            }
                        }
                    } else {
                        tracing::warn!("⚠️  No broadcast channel receivers available");
                    }
                }
            }

            tracing::info!("Discord Gateway task ended");
        });

        Ok(())
    }

    async fn send_message(&self, target: MessageTarget, content: MessageContent)
        -> Result<(), ChannelError>
    {
        let channel_id = match &target {
            MessageTarget::Channel(id) => id,
            MessageTarget::DirectMessage(_) => {
                return Err(ChannelError::SendFailed {
                    target,
                    source: "Direct messages not yet implemented".to_string(),
                });
            }
            MessageTarget::Group(_) => {
                return Err(ChannelError::SendFailed {
                    target,
                    source: "Group messages not supported on Discord".to_string(),
                });
            }
        };

        // Parse channel ID
        let parsed_id = Self::parse_channel_id(channel_id)?;

        // Get message text
        let text = content.text.as_ref()
            .ok_or_else(|| ChannelError::SendFailed {
                target: target.clone(),
                source: "Message content is empty".to_string(),
            })?;

        // Chunk message if needed
        let chunks = Self::chunk_message(text, self.config.message_limit);
        let chunk_count = chunks.len();

        // Send each chunk
        for chunk in &chunks {
            let id = Id::<ChannelMarker>::new(parsed_id);

            self.http
                .create_message(id)
                .content(chunk)
                .await
                .map_err(|e| ChannelError::SendFailed {
                    target: target.clone(),
                    source: format!("HTTP error: {}", e),
                })?;

            tracing::debug!("Sent message chunk to Discord channel {}", parsed_id);
        }

        tracing::info!(
            "Discord send_message: target={:?}, chunks={}",
            target,
            chunk_count
        );

        Ok(())
    }

    fn on_message(&self) -> Pin<Box<dyn futures_util::stream::Stream<Item = IncomingMessage> + Send>> {
        tracing::info!("📞 on_message() called, creating receiver");

        // Create a new receiver from the broadcast channel
        let tx_guard = self.message_tx.try_lock();
        let mut rx = if let Ok(guard) = tx_guard {
            if let Some(tx) = guard.as_ref() {
                tracing::info!("✅ Broadcast receiver created successfully");
                tx.subscribe()
            } else {
                // Return empty stream if sender not available
                tracing::error!("❌ No broadcast sender available");
                return Box::pin(futures_util::stream::empty());
            }
        } else {
            // Return empty stream if lock failed
            tracing::error!("❌ Failed to lock message_tx mutex");
            return Box::pin(futures_util::stream::empty());
        };

        // Wrap the broadcast receiver in a stream
        let stream = async_stream::stream! {
            tracing::info!("🎯 Message stream waiting for messages...");
            while let Ok(msg) = rx.recv().await {
                tracing::info!("📬 Stream received message from broadcast: {}", msg.content.chars().take(30).collect::<String>());
                yield msg;
            }
            tracing::warn!("⚠️  Message stream ended");
        };

        Box::pin(stream)
    }

    async fn resolve_target(&self, identifier: &str) -> Result<MessageTarget, ChannelError> {
        let _id = Self::parse_channel_id(identifier)?;
        Ok(MessageTarget::Channel(identifier.to_string()))
    }

    async fn health_check(&self) -> Result<bool, ChannelError> {
        // Simple health check: verify bot token is not empty
        Ok(!self.config.bot_token.is_empty())
    }
}
