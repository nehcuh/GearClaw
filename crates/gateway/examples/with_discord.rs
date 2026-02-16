// Gateway Server with Discord Channel Example
//
// This example demonstrates how to start the Gateway server
// with Discord channel adapter integration.

use gearclaw_gateway::GatewayServer;
use gearclaw_channels::{DiscordAdapter, ChannelAdapter};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create Gateway server
    let server = GatewayServer::new(gearclaw_gateway::GatewayConfig::default());

    // Try to initialize Discord adapter
    if let Ok(mut discord) = DiscordAdapter::from_env() {
        tracing::info!("Starting Discord adapter...");

        // Start the Discord Gateway connection
        if let Err(e) = discord.start().await {
            tracing::warn!("Failed to start Discord adapter: {}", e);
            tracing::warn!("Continuing without Discord integration...");
        } else {
            // Register Discord adapter with Gateway
            server.register_channel(discord).await?;
            tracing::info!("Discord adapter registered successfully");
        }
    } else {
        tracing::info!("DISCORD_BOT_TOKEN not set, skipping Discord integration");
    }

    // Start Gateway server
    tracing::info!("Starting Gateway server on ws://127.0.0.1:18789/ws");
    server.start().await?;

    Ok(())
}
