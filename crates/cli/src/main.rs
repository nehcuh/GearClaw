mod cli;

use clap::Parser;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter, prelude::*};

use crate::cli::{Cli, Commands};
use gearclaw_core::error::GearClawError;
use gearclaw_core::agent::Agent;
use gearclaw_core::config::Config;

#[tokio::main]
async fn main() -> Result<(), GearClawError> {
    // Initialize tracing
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("gearclaw=info,warn"));
    
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .init();

    print_banner();
    info!("🦾⚙️ GearClaw - OpenClaw Rust 原型");
    info!("版本: 0.1.0");
    info!("");

    // Parse CLI arguments
    let cli = Cli::parse();

    // Handle Init command immediately
    if let Some(Commands::Init) = &cli.command {
        handle_init()?;
        return Ok(());
    }

    // Handle ConfigSample immediately without loading config
    if let Some(Commands::ConfigSample { output }) = &cli.command {
        let sample_config = Config::sample();
        let path = output.clone().unwrap_or_else(|| std::path::PathBuf::from("./gearclaw.sample.toml"));
        sample_config.save(&path)?;
        println!("✅ 示例配置已生成: {:?}", path);
        return Ok(());
    }

    // Load configuration
    let config = Config::load(&cli.config_path)?;

    // Create agent (clone config for agent use)
    let agent = Agent::new(config.clone()).await?;

    // Handle different commands
    match cli.command {
        Some(Commands::Chat) => {
            // Start interactive chat
            agent.start_interactive().await?;
        }
        Some(Commands::ConfigSample { .. }) | Some(Commands::Init) => {
            // Already handled
        }
        Some(Commands::ListSessions) => {
            // List sessions
            let sessions = agent.session_manager.list_sessions()?;
            if sessions.is_empty() {
                println!("没有会话记录");
            } else {
                println!("📝 会话列表:");
                for session in sessions {
                    println!("  • {}", session);
                }
            }
        }
        Some(Commands::DeleteSession { session_id }) => {
            // Delete session
            agent.session_manager.delete_session(&session_id)?;
            println!("✅ 会话已删除: {}", session_id);
        }
        Some(Commands::Run { prompt, session }) => {
            // Run single command
            let mut sess = agent.session_manager.get_or_create_session(
                session.as_deref().unwrap_or("default")
            )?;
            let _ = agent.process_message(&mut sess, &prompt).await?;
            println!(); // Ensure newline
            agent.session_manager.save_session(&sess).await?;
        }
        Some(Commands::Memory { command }) => {
            match command {
                crate::cli::MemoryCommands::Sync => {
                    agent.memory_manager.sync().await?;
                    println!("✅ 记忆同步完成");
                }
                crate::cli::MemoryCommands::Search { query } => {
                    let results = agent.memory_manager.search(&query, 5).await?;
                    if results.is_empty() {
                         println!("没有找到相关记忆");
                    } else {
                         println!("🔍 搜索结果:");
                         for (i, res) in results.iter().enumerate() {
                             println!("{}. [{:.2}] {} (Line {})", i+1, res.score, res.path, res.start_line.unwrap_or(0));
                             let preview: String = res.text.lines().take(1).collect::<String>().chars().take(80).collect();
                             println!("   {}...", preview);
                         }
                    }
                }
            }
        }
        Some(Commands::TestMcp) => {
            println!("🧪 Testing System Capabilities...");
            println!("================================");

            // 1. Verify Skills
            println!("\n📘 [1/3] Verifying Skills...");
            let skills = &agent.skill_manager.skills;
            if skills.is_empty() {
                println!("⚠️  No skills loaded.");
            } else {
                println!("✅ Loaded {} skills:", skills.len());
                for skill in skills {
                    println!("  • {} ({})", skill.name, skill.path.display());
                }
            }

            // 2. Verify MCP
            println!("\n🔌 [2/3] Verifying MCP Connection...");
            let tools = agent.mcp_manager.list_tools().await;
            if tools.is_empty() {
                println!("⚠️  No MCP tools found. Is the server running?");
            } else {
                println!("✅ Found {} MCP tools.", tools.len());
                // List first 3 tools
                for tool in tools.iter().take(3) {
                    println!("  • {}", tool.name);
                }
                if tools.len() > 3 {
                    println!("  ... and {} more", tools.len() - 3);
                }
            }

            // 3. Verify Agent Tool Execution (Mock)
            println!("\n🤖 [3/3] Verifying Agent Tool Execution (Mock Integration)...");
            // Create a dummy session
            let mut session = agent.session_manager.get_or_create_session("test_session")?;
            
            // Define a test case
            let target_tool = "filesystem__list_directory";
            if tools.iter().any(|t| t.name == target_tool) {
                let args_str = r#"{"path": "/private/tmp"}"#;
                println!("Simulating Agent calling '{}' with args: {}", target_tool, args_str);
                
                match agent.execute_tool_call(&mut session, target_tool, args_str).await {
                    Ok(result) => {
                        if result.success {
                            println!("✅ Agent successfully executed MCP tool!");
                            println!("Output snippet: {}", result.output.lines().take(3).collect::<Vec<_>>().join("\n"));
                        } else {
                            println!("❌ Agent executed tool but it returned failure.");
                            println!("Error: {:?}", result.error);
                        }
                    },
                    Err(e) => {
                        println!("❌ Agent failed to execute tool: {}", e);
                    }
                }
            } else {
                println!("⚠️  Skipping Agent Mock test: '{}' tool not found.", target_tool);
            }
            
            println!("\n✨ Verification Complete.");
        }
        Some(Commands::Gateway { host, port, dev }) => {
            // Start Gateway server
            handle_gateway(&config, host, port, dev).await?;
        }
        None => {
            // Default to interactive mode
            agent.start_interactive().await?;
        }
    }

    Ok(())
}

fn print_banner() {
    println!(r#"
   ______                  ________            
  / ____/___  ____ ______ / ____/ /___ __      __
 / / __/ __ \/ __ `/ ___// /   / / __ `/ | /| / /
/ /_/ /  ___/ /_/ / /   / /___/ / /_/ /| |/ |/ / 
\____/\____/\__,_/_/    \____/_/\__,_/ |__/|__/  
    "#);
}

fn handle_init() -> Result<(), GearClawError> {
    use std::io::{self, Write};

    println!("🦾⚙️ GearClaw 初始化");
    println!("================");

    let home = dirs::home_dir().ok_or_else(|| GearClawError::ConfigNotFound("无法找到用户主目录".to_string()))?;
    let gearclaw_dir = home.join(".gearclaw");
    let config_path = gearclaw_dir.join("config.toml");
    let openclaw_dir = home.join(".openclaw");

    if config_path.exists() {
        print!("⚠️  配置文件已存在于 {:?}。是否覆盖? [y/N] ", config_path);
        io::stdout().flush().map_err(GearClawError::IoError)?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(GearClawError::IoError)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("操作已取消");
            return Ok(());
        }
    }

    println!("\n请选择配置模式:");
    println!("1) [推荐] 独立模式 (Stand-alone)");
    println!("   - 创建全新的 ~/.gearclaw 配置目录");
    println!("   - 使用独立的 Skills 和 Sessions");
    println!();
    println!("2) 兼容模式 (Reuse OpenClaw)");
    println!("   - 复用 ~/.openclaw/skills 中的技能");
    println!("   - 仍然创建 ~/.gearclaw 用于保存配置");
    println!();

    print!("请选择 [1/2] (默认 1): ");
    io::stdout().flush().map_err(GearClawError::IoError)?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(GearClawError::IoError)?;
    let choice = input.trim();

    let mut config = Config::sample();

    if choice == "2" {
        if openclaw_dir.exists() {
            println!("✅ 检测到 OpenClaw 目录: {:?}", openclaw_dir);
            config.agent.skills_path = openclaw_dir.join("skills");
        } else {
            println!("⚠️  未检测到 ~/.openclaw 目录，将回退到独立模式");
        }
    } else {
        // Default is stand-alone, nothing to change in sample config
    }

    // Create directories
    if !gearclaw_dir.exists() {
        std::fs::create_dir_all(&gearclaw_dir).map_err(GearClawError::IoError)?;
        println!("✅ 创建配置目录: {:?}", gearclaw_dir);
    }

    let skills_dir = gearclaw_dir.join("skills");
    if !skills_dir.exists() && choice != "2" {
        std::fs::create_dir_all(&skills_dir).map_err(GearClawError::IoError)?;
        println!("✅ 创建技能目录: {:?}", skills_dir);
        
        // Create a sample skill
        let sample_skill_dir = skills_dir.join("hello");
        std::fs::create_dir_all(&sample_skill_dir).map_err(GearClawError::IoError)?;
        let skill_md = r#"---
name: hello_world
description: A simple hello world skill
metadata: {}
---

# Hello World Skill

This skill allows you to say hello.

```bash
echo "Hello from GearClaw Skill!"
```
"#;
        std::fs::write(sample_skill_dir.join("SKILL.md"), skill_md).map_err(GearClawError::IoError)?;
        println!("✅ 创建示例技能: hello_world");
    }

    let sessions_dir = gearclaw_dir.join("sessions");
    if !sessions_dir.exists() {
        std::fs::create_dir_all(&sessions_dir).map_err(GearClawError::IoError)?;
        println!("✅ 创建会话目录: {:?}", sessions_dir);
    }

    // Save config
    config.save(&config_path)?;
    println!("✅ 配置文件已保存: {:?}", config_path);
    
    println!("\n🎉 初始化完成! 你现在可以运行 `gearclaw` 开始使用了。");

    Ok(())
}

async fn handle_gateway(
    config: &Config,
    host: Option<String>,
    port: Option<u16>,
    dev: bool,
) -> Result<(), GearClawError> {
    use gearclaw_channels::{ChannelAdapter, DiscordAdapter};
    use gearclaw_channels::platforms::discord::DiscordConfig;
    use gearclaw_gateway::{GatewayServer, MethodHandlers};
    use std::sync::Arc;

    // Use CLI args or config file values
    let gw_host = host.unwrap_or_else(|| config.gateway.host.clone());
    let gw_port = port.unwrap_or(config.gateway.port);

    // Configure logging
    if dev {
        let env_filter = EnvFilter::new("gearclaw=debug,gearclaw_gateway=debug,gearclaw_channels=debug");
        tracing_subscriber::registry()
            .with(env_filter)
            .try_init()
            .ok();
    }

    println!("🦾 GearClaw Gateway 启动中...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  主机: {}", gw_host);
    println!("  端口: {}", gw_port);
    println!("  模式: {}", if dev { "开发" } else { "生产" });
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Check for Discord token
    let discord_token = std::env::var("DISCORD_BOT_TOKEN");
    let agent_for_discord = if discord_token.is_ok() {
        Some(Arc::new(Agent::new(config.clone()).await?))
    } else {
        None
    };

    if let Some(token) = discord_token.ok() {
        println!("📱 Discord Bot Token 已设置");
        println!("   正在初始化 Discord 适配器...");
        println!();

        // Create and start Discord adapter
        let discord_config = DiscordConfig {
            bot_token: token.clone(),
            message_limit: 2000,
        };

        let mut discord = DiscordAdapter::new(discord_config);

        // Clone agent for Discord message handling
        let agent_clone = agent_for_discord.clone().unwrap();

        // Start Discord adapter in background
        let _discord_handle = tokio::spawn(async move {
            if let Err(e) = discord.start().await {
                tracing::error!("Discord adapter failed to start: {}", e);
                return Err(e);
            }

            // Listen for Discord messages
            use futures_util::StreamExt;
            let mut message_stream = discord.on_message();

            tracing::info!("Discord message listener started");

            while let Some(incoming_msg) = message_stream.next().await {
                // Get source name and ID from MessageSource
                let (source_name, source_id) = match &incoming_msg.source {
                    gearclaw_channels::MessageSource::User { id, name } => {
                        (name.clone(), id.clone())
                    }
                    gearclaw_channels::MessageSource::Channel { id, name } => {
                        (name.clone(), id.clone())
                    }
                    gearclaw_channels::MessageSource::Group { id, name } => {
                        (name.clone(), id.clone())
                    }
                };

                tracing::info!(
                    "Received Discord message from {}: {}",
                    source_name,
                    incoming_msg.content
                );

                // Process message with agent
                tracing::info!("🤖 Calling Agent.process_channel_message()...");

                match agent_clone.process_channel_message(
                    &incoming_msg.platform,
                    &source_id,
                    &incoming_msg.content,
                ).await {
                    Ok(response) => {
                        tracing::info!("✅ Agent.process_channel_message() returned, response length: {}", response.len());

                        if response.is_empty() {
                            tracing::debug!("Agent chose not to respond (trigger not met)");
                        } else {
                            tracing::info!("Agent response: {}", response);

                            // Send response back to Discord
                            use gearclaw_channels::{MessageTarget, MessageContent};

                            let channel_id = match incoming_msg.metadata.get("channel_id")
                                .and_then(|v| v.as_str()) {
                                    Some(id) => id,
                                    None => {
                                        tracing::error!("Missing channel_id in message metadata");
                                        continue;
                                    }
                                };

                            let target = MessageTarget::Channel(channel_id.to_string());
                            let content = MessageContent {
                                text: Some(response.clone()),
                                embeds: Vec::new(),
                            };

                            if let Err(e) = discord.send_message(target, content).await {
                                tracing::error!("Failed to send response to Discord: {}", e);
                            } else {
                                tracing::info!("✅ Successfully sent response to Discord channel {}", channel_id);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("❌ Failed to process Discord message: {}", e);
                        tracing::error!("Error type: {:?}", std::error::Error::source(&e));
                    }
                }
            }

            Ok::<(), gearclaw_channels::ChannelError>(())
        });

        println!("✅ Discord 适配器已启动");
        println!("   消息监听器已启动");
        println!();
    } else {
        println!("⚠️  DISCORD_BOT_TOKEN 未设置");
        println!("   Discord 功能将被禁用");
        println!("   设置: export DISCORD_BOT_TOKEN='your_token'");
        println!();
    }

    // Create agent for WebSocket gateway
    let agent = if let Some(discord_agent) = agent_for_discord {
        discord_agent
    } else {
        Arc::new(Agent::new(config.clone()).await?)
    };

    // Create gateway config
    let gw_config = gearclaw_gateway::GatewayConfig {
        host: gw_host,
        port: gw_port,
        ws_path: config.gateway.ws_path.clone(),
    };

    // Create server with agent integration
    let handlers = MethodHandlers::new();
    handlers.set_agent(agent.clone()).await;

    let server = GatewayServer::new(gw_config)
        .with_handlers(Arc::new(handlers));

    println!("🌐 Gateway 服务器启动中...");
    println!();

    server.start().await
        .map_err(|e| GearClawError::Other(format!("Gateway error: {}", e)))?;

    Ok(())
}
