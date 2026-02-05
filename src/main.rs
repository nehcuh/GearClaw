mod agent;
mod config;
mod error;
mod llm;
mod session;
mod tools;
mod skills;
mod cli;
mod memory;

use clap::Parser;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter, prelude::*};

use crate::cli::{Cli, Commands};
use crate::error::GearClawError;
use crate::agent::Agent;
use crate::config::Config;

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

    // Create agent
    let agent = Agent::new(config).await?;

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
    use std::path::PathBuf;

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
