// Gateway Server with Discord and Agent Integration Example
//
// This example demonstrates the complete flow:
// 1. Gateway server starts
// 2. Discord adapter connects and listens for messages
// 3. Agent processes incoming Discord messages
// 4. Agent responses are sent back to Discord

use anyhow::Result;
use gearclaw_channels::{ChannelAdapter, DiscordAdapter};
use gearclaw_core::{Agent, AgentTriggerConfig, Config, TriggerMode};
use gearclaw_gateway::GatewayConfig;
use gearclaw_gateway::GatewayServer;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    tracing::info!("🦾⚙️  GearClaw Gateway with Agent Integration");

    // Create Gateway server
    let server = GatewayServer::new(GatewayConfig::default());

    // Try to initialize Agent (if API key is configured)
    let agent = match Config::load(&None) {
        Ok(config) => match Agent::new(config).await {
            Ok(agent) => {
                tracing::info!("✅ Agent initialized successfully");
                tracing::info!("   - Messages from Discord will trigger Agent responses");
                Some(Arc::new(agent))
            }
            Err(e) => {
                tracing::warn!("⚠️  Failed to initialize Agent: {}", e);
                tracing::warn!("   - Discord messages will be broadcast but not processed");
                None
            }
        },
        Err(e) => {
            tracing::warn!("⚠️  Failed to load config: {}", e);
            tracing::warn!("   - Discord messages will be broadcast but not processed");
            None
        }
    };

    // Set Agent on handlers
    if let Some(ref agent) = agent {
        let handlers = server.handlers();
        handlers.set_agent(agent.clone()).await;
        tracing::info!("✅ Agent linked to Gateway handlers");

        // Configure agent triggers
        // Option 1: Mention mode (default) - only respond when mentioned
        let trigger_config = AgentTriggerConfig {
            mode: TriggerMode::Mention,
            mention_patterns: vec!["@agent".to_string(), "@bot".to_string()],
            keywords: vec![],
            enabled_channels: vec![],
            disabled_channels: vec![],
        };
        handlers.set_trigger_config(trigger_config).await;
        tracing::info!("✅ Trigger config set: Mention mode (@agent, @bot)");

        // Option 2: Keyword mode - respond to specific keywords
        // let trigger_config = AgentTriggerConfig {
        //     mode: TriggerMode::Keyword,
        //     mention_patterns: vec![],
        //     keywords: vec!["help".to_string(), "error".to_string()],
        //     enabled_channels: vec![],
        //     disabled_channels: vec![],
        // };

        // Option 3: Always mode - respond to everything
        // let trigger_config = AgentTriggerConfig {
        //     mode: TriggerMode::Always,
        //     ..Default::default()
        // };

        // Option 4: Channel filtering
        // let trigger_config = AgentTriggerConfig {
        //     mode: TriggerMode::Mention,
        //     enabled_channels: vec!["discord:123456789".to_string()], // Only these channels
        //     disabled_channels: vec!["discord:987654321".to_string()], // Not these channels
        //     ..Default::default()
        // };
    }

    // Try to initialize Discord adapter
    if let Ok(mut discord) = DiscordAdapter::from_env() {
        tracing::info!("🔷 Starting Discord adapter...");

        // Start the Discord Gateway connection
        match discord.start().await {
            Ok(()) => {
                tracing::info!("✅ Discord Gateway connected");

                // Register Discord adapter with Gateway
                server.register_channel(discord).await?;
                tracing::info!("✅ Discord adapter registered");
            }
            Err(e) => {
                tracing::error!("❌ Failed to start Discord: {}", e);
                tracing::error!("   Check DISCORD_BOT_TOKEN environment variable");
                return Err(e.into());
            }
        }
    } else {
        tracing::warn!("⚠️  DISCORD_BOT_TOKEN not set");
        tracing::warn!("   Discord integration disabled");
    }

    // Display information
    tracing::info!("");
    tracing::info!("🚀 Gateway Server Ready");
    tracing::info!("   WebSocket: ws://127.0.0.1:18789/ws");
    tracing::info!("");
    tracing::info!("📡 Event Flow:");
    tracing::info!("   Discord Message → Gateway → WebSocket Clients");
    if agent.is_some() {
        let handlers = server.handlers();
        let trigger_config = handlers.get_trigger_config().await;
        match trigger_config.mode {
            TriggerMode::Always => {
                tracing::info!("   Agent: ✅ ACTIVE (responds to all messages)");
            }
            TriggerMode::Mention => {
                tracing::info!(
                    "   Agent: ✅ ACTIVE (mention mode: {:?})",
                    trigger_config.mention_patterns
                );
            }
            TriggerMode::Keyword => {
                tracing::info!(
                    "   Agent: ✅ ACTIVE (keyword mode: {:?})",
                    trigger_config.keywords
                );
            }
        }
        tracing::info!("   Discord Message → Agent → Discord Response (when triggered)");
    }
    tracing::info!("");

    // Start Gateway server (blocks forever)
    server.start().await?;

    Ok(())
}
