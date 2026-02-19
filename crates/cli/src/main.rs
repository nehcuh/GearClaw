mod cli;

use clap::Parser;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::cli::{Cli, Commands};
use gearclaw_agent::Agent;
use gearclaw_core::config::{Config, SkillSourceKind, SkillTrustPolicy};
use gearclaw_core::error::GearClawError;

#[tokio::main]
async fn main() -> Result<(), GearClawError> {
    // Initialize tracing
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("gearclaw=info,warn"));

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
        let path = output
            .clone()
            .unwrap_or_else(|| std::path::PathBuf::from("./gearclaw.sample.toml"));
        sample_config.save(&path)?;
        println!("✅ 示例配置已生成: {:?}", path);
        return Ok(());
    }

    // Load configuration
    let config = Config::load(&cli.config_path)?;
    // Handle commands that do not require LLM/Agent initialization
    match &cli.command {
        Some(Commands::ListSources) => {
            handle_list_sources(&config);
            return Ok(());
        }
        Some(Commands::TrustPolicy) => {
            handle_trust_policy(&config);
            return Ok(());
        }
        Some(Commands::ListAudit {
            limit,
            source,
            skill,
            status,
            since,
            until,
            json,
            output,
        }) => {
            handle_list_audit(
                &config,
                *limit,
                source.as_deref(),
                skill.as_deref(),
                status.as_deref(),
                *since,
                *until,
                *json,
                output.as_str(),
            )?;
            return Ok(());
        }
        Some(Commands::SearchSkill {
            query,
            source,
            update,
        }) => {
            handle_search_skill(&config, query, source.as_deref(), *update)?;
            return Ok(());
        }
        Some(Commands::InstallSkill {
            name,
            source,
            force,
            dry_run,
            update,
        }) => {
            handle_install_skill(&config, name, source.as_deref(), *force, *dry_run, *update)?;
            return Ok(());
        }
        _ => {}
    }

    // Create agent (clone config for agent use)
    let agent = Agent::new(config.clone()).await?;

    // Handle different commands
    match cli.command {
        Some(Commands::Chat) => {
            // Start interactive chat
            agent.start_interactive().await?;
        }
        Some(Commands::ConfigSample { .. })
        | Some(Commands::Init)
        | Some(Commands::ListSources)
        | Some(Commands::ListAudit { .. })
        | Some(Commands::TrustPolicy)
        | Some(Commands::SearchSkill { .. })
        | Some(Commands::InstallSkill { .. }) => {
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
            let mut sess = agent
                .session_manager
                .get_or_create_session(session.as_deref().unwrap_or("default"))?;
            let _ = agent.process_message(&mut sess, &prompt).await?;
            println!(); // Ensure newline
            agent.session_manager.save_session(&sess).await?;
        }
        Some(Commands::Memory { command }) => match command {
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
                        println!(
                            "{}. [{:.2}] {} (Line {})",
                            i + 1,
                            res.score,
                            res.path,
                            res.start_line.unwrap_or(0)
                        );
                        let preview: String = res
                            .text
                            .lines()
                            .take(1)
                            .collect::<String>()
                            .chars()
                            .take(80)
                            .collect();
                        println!("   {}...", preview);
                    }
                }
            }
        },
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
            if !agent.mcp_manager.is_enabled() {
                println!("ℹ️  MCP capability is disabled in this build.");
                println!("⚠️  Skipping MCP connectivity and Agent MCP execution checks.");
            } else {
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
                let mut session = agent
                    .session_manager
                    .get_or_create_session("test_session")?;

                // Define a test case
                let target_tool = "filesystem__list_directory";
                if tools.iter().any(|t| t.name == target_tool) {
                    let args_str = r#"{"path": "/private/tmp"}"#;
                    println!(
                        "Simulating Agent calling '{}' with args: {}",
                        target_tool, args_str
                    );

                    match agent
                        .execute_tool_call(&mut session, target_tool, args_str)
                        .await
                    {
                        Ok(result) => {
                            if result.success {
                                println!("✅ Agent successfully executed MCP tool!");
                                println!(
                                    "Output snippet: {}",
                                    result.output.lines().take(3).collect::<Vec<_>>().join("\n")
                                );
                            } else {
                                println!("❌ Agent executed tool but it returned failure.");
                                println!("Error: {:?}", result.error);
                            }
                        }
                        Err(e) => {
                            println!("❌ Agent failed to execute tool: {}", e);
                        }
                    }
                } else {
                    println!(
                        "⚠️  Skipping Agent Mock test: '{}' tool not found.",
                        target_tool
                    );
                }
            }

            println!("\n✨ Verification Complete.");
        }
        Some(Commands::Gateway {
            host,
            port,
            dev,
            allow_unauthenticated,
        }) => {
            // Start Gateway server
            handle_gateway(&config, host, port, dev, allow_unauthenticated).await?;
        }
        None => {
            // Default to interactive mode
            agent.start_interactive().await?;
        }
    }

    Ok(())
}

fn print_banner() {
    println!(
        r#"
   ______                  ________            
  / ____/___  ____ ______ / ____/ /___ __      __
 / / __/ __ \/ __ `/ ___// /   / / __ `/ | /| / /
/ /_/ /  ___/ /_/ / /   / /___/ / /_/ /| |/ |/ / 
\____/\____/\__,_/_/    \____/_/\__,_/ |__/|__/  
    "#
    );
}

fn handle_init() -> Result<(), GearClawError> {
    use std::io::{self, Write};

    println!("🦾⚙️ GearClaw 初始化");
    println!("================");

    let home = dirs::home_dir()
        .ok_or_else(|| GearClawError::ConfigNotFound("无法找到用户主目录".to_string()))?;
    let gearclaw_dir = home.join(".gearclaw");
    let config_path = gearclaw_dir.join("config.toml");
    let openclaw_dir = home.join(".openclaw");

    if config_path.exists() {
        print!("⚠️  配置文件已存在于 {:?}。是否覆盖? [y/N] ", config_path);
        io::stdout().flush().map_err(GearClawError::IoError)?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(GearClawError::IoError)?;
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
    io::stdin()
        .read_line(&mut input)
        .map_err(GearClawError::IoError)?;
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
        std::fs::write(sample_skill_dir.join("SKILL.md"), skill_md)
            .map_err(GearClawError::IoError)?;
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

#[derive(Debug, Clone)]
struct EffectiveSkillSource {
    name: String,
    kind: SkillSourceKind,
    location: PathBuf,
    revision: Option<String>,
    enabled: bool,
    trusted: bool,
    verify_head_commit_signature: bool,
}

#[derive(Debug, Clone)]
struct SkillCatalogEntry {
    name: String,
    description: String,
    source_name: String,
    source_kind: SkillSourceKind,
    source_location: PathBuf,
    source_revision: Option<String>,
    source_head_commit: Option<String>,
    source_signature_verified: bool,
    source_trusted: bool,
    skill_dir: PathBuf,
}

#[derive(Debug, Clone)]
struct GitSyncResult {
    cache_dir: PathBuf,
    head_commit: Option<String>,
    signature_verified: bool,
}

#[derive(Debug, Clone, Copy, Default)]
struct DiscoveryOptions {
    force_update: bool,
}

#[derive(Debug, Clone)]
struct AuditRecord {
    timestamp: u64,
    fields: BTreeMap<String, String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuditOutputFormat {
    Text,
    Json,
    Jsonl,
}

fn resolve_audit_output_format(
    output: &str,
    json_flag: bool,
) -> Result<AuditOutputFormat, GearClawError> {
    if json_flag {
        return Ok(AuditOutputFormat::Json);
    }
    match output {
        "text" => Ok(AuditOutputFormat::Text),
        "json" => Ok(AuditOutputFormat::Json),
        "jsonl" => Ok(AuditOutputFormat::Jsonl),
        other => Err(GearClawError::Other(format!(
            "Unsupported audit output format: {}",
            other
        ))),
    }
}

impl AuditRecord {
    fn field(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(String::as_str)
    }
}

fn parse_audit_record_line(line: &str) -> Option<AuditRecord> {
    let parts = split_escaped_audit_fields(line);
    if parts.is_empty() {
        return None;
    }

    let timestamp = parts.first()?.parse::<u64>().ok()?;
    let mut fields = BTreeMap::new();

    for part in parts.iter().skip(1) {
        if let Some((k, v)) = part.split_once('=') {
            fields.insert(k.trim().to_string(), v.trim().to_string());
        }
    }

    Some(AuditRecord { timestamp, fields })
}

fn split_escaped_audit_fields(line: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut escape = false;

    for ch in line.chars() {
        if escape {
            match ch {
                'n' => current.push('\n'),
                '|' => current.push('|'),
                '\\' => current.push('\\'),
                other => {
                    current.push('\\');
                    current.push(other);
                }
            }
            escape = false;
            continue;
        }

        if ch == '\\' {
            escape = true;
            continue;
        }

        if ch == '|' {
            parts.push(current);
            current = String::new();
        } else {
            current.push(ch);
        }
    }

    if escape {
        current.push('\\');
    }
    parts.push(current);
    parts
}

fn effective_skill_sources(config: &Config) -> Vec<EffectiveSkillSource> {
    if config.agent.skill_sources.is_empty() {
        return vec![EffectiveSkillSource {
            name: "local-default".to_string(),
            kind: SkillSourceKind::LocalDir,
            location: config.agent.skills_path.clone(),
            revision: None,
            enabled: true,
            trusted: true,
            verify_head_commit_signature: false,
        }];
    }

    config
        .agent
        .skill_sources
        .iter()
        .map(|s| EffectiveSkillSource {
            name: s.name.clone(),
            kind: s.kind.clone(),
            location: PathBuf::from(&s.location),
            revision: s.revision.clone(),
            enabled: s.enabled,
            trusted: s.trusted,
            verify_head_commit_signature: s.verify_head_commit_signature,
        })
        .collect()
}

fn trust_policy_label(policy: &SkillTrustPolicy) -> &'static str {
    match policy {
        SkillTrustPolicy::LocalOnly => "local_only",
        SkillTrustPolicy::TrustedOnly => "trusted_only",
        SkillTrustPolicy::AllowUntrusted => "allow_untrusted",
    }
}

fn git_ref_exists(repo: &Path, revision: &str) -> Result<bool, GearClawError> {
    let candidate = format!("{}^{{commit}}", revision);
    let output = run_git_command(
        Some(repo),
        ["rev-parse", "--verify", "--quiet", candidate.as_str()].as_slice(),
    )?;
    Ok(output.status == 0)
}

fn handle_list_audit(
    config: &Config,
    limit: usize,
    source_filter: Option<&str>,
    skill_filter: Option<&str>,
    status_filter: Option<&str>,
    since: Option<u64>,
    until: Option<u64>,
    json_output: bool,
    output: &str,
) -> Result<(), GearClawError> {
    let output_format = resolve_audit_output_format(output, json_output)?;
    let log_path = skill_install_audit_log_path(config);
    if !log_path.exists() {
        match output_format {
            AuditOutputFormat::Json => println!("[]"),
            AuditOutputFormat::Jsonl => {}
            AuditOutputFormat::Text => println!("暂无安装审计日志: {}", log_path.display()),
        }
        return Ok(());
    }

    let content = std::fs::read_to_string(&log_path).map_err(GearClawError::IoError)?;
    let mut records: Vec<AuditRecord> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(parse_audit_record_line)
        .collect();

    if let Some(source) = source_filter {
        records.retain(|r| r.field("source") == Some(source));
    }
    if let Some(skill) = skill_filter {
        records.retain(|r| r.field("skill") == Some(skill));
    }
    if let Some(status) = status_filter {
        records.retain(|r| r.field("status") == Some(status));
    }
    if let Some(min_ts) = since {
        records.retain(|r| r.timestamp >= min_ts);
    }
    if let Some(max_ts) = until {
        records.retain(|r| r.timestamp <= max_ts);
    }

    if records.is_empty() {
        match output_format {
            AuditOutputFormat::Json => println!("[]"),
            AuditOutputFormat::Jsonl => {}
            AuditOutputFormat::Text => println!("安装审计日志为空: {}", log_path.display()),
        }
        return Ok(());
    }

    let max_lines = limit.max(1);
    let start = records.len().saturating_sub(max_lines);
    let selected = &records[start..];
    if output_format == AuditOutputFormat::Text {
        println!(
            "🧾 Skill install audit (latest {} / filtered total {})",
            selected.len(),
            records.len()
        );
    }
    let mut json_records = Vec::with_capacity(selected.len());
    for record in selected {
        let mut object = serde_json::Map::new();
        object.insert(
            "timestamp".to_string(),
            serde_json::Value::from(record.timestamp),
        );
        for (k, v) in &record.fields {
            object.insert(k.clone(), serde_json::Value::from(v.clone()));
        }
        json_records.push(serde_json::Value::Object(object));
    }
    match output_format {
        AuditOutputFormat::Json => {
            let out = serde_json::to_string_pretty(&json_records)
                .map_err(|e| GearClawError::Other(format!("serialize audit json failed: {}", e)))?;
            println!("{}", out);
        }
        AuditOutputFormat::Jsonl => {
            for record in json_records {
                let line = serde_json::to_string(&record).map_err(|e| {
                    GearClawError::Other(format!("serialize audit jsonl failed: {}", e))
                })?;
                println!("{}", line);
            }
        }
        AuditOutputFormat::Text => {
            for record in selected {
                println!(
                    "  {} | skill={} | source={} | status={} | commit={} | target={}",
                    record.timestamp,
                    record.field("skill").unwrap_or("unknown"),
                    record.field("source").unwrap_or("unknown"),
                    record.field("status").unwrap_or("unknown"),
                    record.field("commit").unwrap_or("unknown"),
                    record.field("target").unwrap_or("unknown"),
                );
            }
        }
    }

    Ok(())
}

fn handle_list_sources(config: &Config) {
    let sources = effective_skill_sources(config);
    println!("📦 Skill Sources:");
    if sources.is_empty() {
        println!("  (none)");
        return;
    }
    for source in sources {
        let revision = source.revision.as_deref().unwrap_or("default");
        println!(
            "  • {} | kind={} | enabled={} | trusted={} | verify_sig={} | revision={} | location={}",
            source.name,
            source_kind_label(&source.kind),
            source.enabled,
            source.trusted,
            source.verify_head_commit_signature,
            revision,
            source.location.display()
        );
    }
}

fn handle_trust_policy(config: &Config) {
    println!(
        "🔐 Skill trust policy: {}",
        trust_policy_label(&config.agent.skill_trust_policy)
    );
    match config.agent.skill_trust_policy {
        SkillTrustPolicy::LocalOnly => {
            println!("   - only local_dir sources can be installed");
        }
        SkillTrustPolicy::TrustedOnly => {
            println!("   - only sources marked trusted=true can be installed");
        }
        SkillTrustPolicy::AllowUntrusted => {
            println!("   - all configured sources can be installed (unsafe)");
        }
    }
}

fn handle_search_skill(
    config: &Config,
    query: &str,
    source_filter: Option<&str>,
    force_update: bool,
) -> Result<(), GearClawError> {
    let all_entries = discover_skills(config, source_filter, DiscoveryOptions { force_update })?;
    let q = query.to_lowercase();
    let matched: Vec<_> = all_entries
        .into_iter()
        .filter(|entry| {
            entry.name.to_lowercase().contains(&q) || entry.description.to_lowercase().contains(&q)
        })
        .collect();

    if matched.is_empty() {
        println!("没有匹配的 skill: {}", query);
        return Ok(());
    }

    println!("🔎 Skills matched '{}':", query);
    for skill in matched {
        let revision = skill.source_revision.as_deref().unwrap_or("default");
        let commit = skill.source_head_commit.as_deref().unwrap_or("unknown");
        println!(
            "  • {}  [{}] trusted={} rev={} commit={} sig_verified={}  {}",
            skill.name,
            skill.source_name,
            skill.source_trusted,
            revision,
            commit,
            skill.source_signature_verified,
            skill.description
        );
    }
    Ok(())
}

fn handle_install_skill(
    config: &Config,
    name: &str,
    source_filter: Option<&str>,
    force: bool,
    dry_run: bool,
    force_update: bool,
) -> Result<(), GearClawError> {
    let entries = discover_skills(config, source_filter, DiscoveryOptions { force_update })?;
    let mut exact_matches: Vec<_> = entries
        .into_iter()
        .filter(|entry| entry.name.eq_ignore_ascii_case(name))
        .collect();

    if exact_matches.is_empty() {
        println!("❌ 未找到 skill: {}", name);
        println!("提示: 先运行 `gearclaw search-skill \"{}\"`", name);
        return Ok(());
    }

    if exact_matches.len() > 1 {
        println!("⚠️ 找到多个同名 skill，请使用 --source 指定来源:");
        for skill in exact_matches {
            println!(
                "  • {} [{}] trusted={} revision={} commit={} sig_verified={}",
                skill.name,
                skill.source_name,
                skill.source_trusted,
                skill.source_revision.as_deref().unwrap_or("default"),
                skill.source_head_commit.as_deref().unwrap_or("unknown"),
                skill.source_signature_verified
            );
        }
        return Ok(());
    }

    let selected = exact_matches.remove(0);
    if !is_install_allowed_by_policy(&selected, &config.agent.skill_trust_policy) {
        println!(
            "❌ 当前 trust policy={}，不允许安装来自 source='{}' 的 skill（trusted={}）",
            trust_policy_label(&config.agent.skill_trust_policy),
            selected.source_name,
            selected.source_trusted
        );
        return Ok(());
    }

    let target_root = &config.agent.skills_path;
    std::fs::create_dir_all(target_root).map_err(GearClawError::IoError)?;

    let install_dir_name = sanitize_skill_dir_name(&selected.name).ok_or_else(|| {
        GearClawError::Other(format!(
            "Skill name '{}' cannot be converted to a valid directory name",
            selected.name
        ))
    })?;
    let target_dir = target_root.join(install_dir_name);

    if target_dir.exists() {
        if !force {
            println!(
                "⚠️ 目标已存在: {}。如需覆盖请加 --force",
                target_dir.display()
            );
            return Ok(());
        }
        if !dry_run {
            std::fs::remove_dir_all(&target_dir).map_err(GearClawError::IoError)?;
        }
    }

    if dry_run {
        println!("🧪 Dry-run: install plan");
        println!("  skill: {}", selected.name);
        println!("  source: {}", selected.source_name);
        println!(
            "  revision: {}",
            selected.source_revision.as_deref().unwrap_or("default")
        );
        println!(
            "  commit: {}",
            selected.source_head_commit.as_deref().unwrap_or("unknown")
        );
        println!("  target: {}", target_dir.display());
        println!("  overwrite: {}", force);
        println!("  update: {}", force_update);
        println!("  no files were changed.");
        return Ok(());
    }

    copy_dir_recursive(&selected.skill_dir, &target_dir)?;
    append_install_audit_log(
        config,
        &selected,
        &target_dir,
        trust_policy_label(&config.agent.skill_trust_policy),
        "installed",
    )?;
    println!(
        "✅ 已安装 skill '{}' 到 {}",
        selected.name,
        target_dir.display()
    );
    if let Some(commit) = &selected.source_head_commit {
        println!("   来源提交: {}", commit);
    }
    println!("提示: 新技能在新建 Agent 会话后可用。");

    Ok(())
}

fn discover_skills(
    config: &Config,
    source_filter: Option<&str>,
    options: DiscoveryOptions,
) -> Result<Vec<SkillCatalogEntry>, GearClawError> {
    let sources = effective_skill_sources(config);
    let source_filter = source_filter.map(|s| s.to_lowercase());
    let mut entries = Vec::new();

    for source in sources {
        if !source.enabled {
            continue;
        }

        if let Some(filter) = &source_filter {
            if source.name.to_lowercase() != *filter {
                continue;
            }
        }

        let source_kind = source.kind.clone();
        let (source_root, source_head_commit, source_signature_verified) = match source_kind {
            SkillSourceKind::LocalDir => (source.location.clone(), None, false),
            SkillSourceKind::GitRepo => {
                let synced = sync_git_source(config, &source, options)?;
                (
                    synced.cache_dir,
                    synced.head_commit,
                    synced.signature_verified,
                )
            }
        };

        for skill_file in collect_skill_files(&source_root)? {
            match parse_skill_metadata(&skill_file) {
                Ok((name, description)) => {
                    if let Some(skill_dir) = skill_file.parent() {
                        entries.push(SkillCatalogEntry {
                            name,
                            description,
                            source_name: source.name.clone(),
                            source_kind: source_kind.clone(),
                            source_location: source.location.clone(),
                            source_revision: source.revision.clone(),
                            source_head_commit: source_head_commit.clone(),
                            source_signature_verified,
                            source_trusted: source.trusted,
                            skill_dir: skill_dir.to_path_buf(),
                        });
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Skipping invalid skill file {}: {}",
                        skill_file.display(),
                        e
                    );
                }
            }
        }
    }

    entries.sort_by(|a, b| {
        a.name
            .cmp(&b.name)
            .then_with(|| a.source_name.cmp(&b.source_name))
    });
    Ok(entries)
}

fn sync_git_source(
    config: &Config,
    source: &EffectiveSkillSource,
    options: DiscoveryOptions,
) -> Result<GitSyncResult, GearClawError> {
    let location = source.location.to_string_lossy().trim().to_string();
    if location.is_empty() {
        return Err(GearClawError::Other(format!(
            "Source '{}' has empty git repository location",
            source.name
        )));
    }

    let cache_root = skill_source_cache_root(config);
    std::fs::create_dir_all(&cache_root).map_err(GearClawError::IoError)?;

    let source_slug = sanitize_skill_dir_name(&source.name).unwrap_or_else(|| "source".to_string());
    let cache_dir = cache_root.join(format!("{}-{:016x}", source_slug, stable_hash(&location)));
    let git_dir = cache_dir.join(".git");
    let mut needs_fresh_clone = !cache_dir.exists() || !git_dir.exists();
    let mut did_network_update = false;

    if !needs_fresh_clone {
        let remote_check =
            run_git_command(Some(&cache_dir), ["remote", "get-url", "origin"].as_slice())?;
        if remote_check.status != 0 || remote_check.stdout.trim() != location {
            needs_fresh_clone = true;
        }
    }

    if needs_fresh_clone {
        if cache_dir.exists() {
            std::fs::remove_dir_all(&cache_dir).map_err(GearClawError::IoError)?;
        }

        let clone_result = run_git_command(
            None,
            [
                "clone",
                "--depth",
                "1",
                "--quiet",
                location.as_str(),
                cache_dir.to_string_lossy().as_ref(),
            ]
            .as_slice(),
        )?;
        if clone_result.status != 0 {
            return Err(GearClawError::Other(format!(
                "Failed to clone source '{}' from '{}': {}",
                source.name, location, clone_result.stderr
            )));
        }
        did_network_update = true;
    } else if should_refresh_source_cache(config, &cache_dir, options.force_update) {
        let fetch_result = run_git_command(
            Some(&cache_dir),
            ["fetch", "--depth", "1", "--prune", "origin"].as_slice(),
        )?;
        if fetch_result.status != 0 {
            return Err(GearClawError::Other(format!(
                "Failed to fetch source '{}' from '{}': {}",
                source.name, location, fetch_result.stderr
            )));
        }
        did_network_update = true;
    }
    let mut checkout_ref = "HEAD".to_string();

    if let Some(revision) = source.revision.as_deref() {
        if options.force_update || needs_fresh_clone || !git_ref_exists(&cache_dir, revision)? {
            let fetch_revision = run_git_command(
                Some(&cache_dir),
                ["fetch", "--depth", "1", "origin", revision].as_slice(),
            )?;
            if fetch_revision.status != 0 {
                return Err(GearClawError::Other(format!(
                    "Failed to fetch revision '{}' for source '{}': {}",
                    revision, source.name, fetch_revision.stderr
                )));
            }
            did_network_update = true;
            checkout_ref = "FETCH_HEAD".to_string();
        } else {
            checkout_ref = revision.to_string();
        }
    } else if options.force_update || needs_fresh_clone {
        let fetch_default = run_git_command(
            Some(&cache_dir),
            ["fetch", "--depth", "1", "origin"].as_slice(),
        )?;
        if fetch_default.status != 0 {
            return Err(GearClawError::Other(format!(
                "Failed to update default revision for source '{}': {}",
                source.name, fetch_default.stderr
            )));
        }
        did_network_update = true;
        checkout_ref = "FETCH_HEAD".to_string();
    }

    let checkout_result = run_git_command(
        Some(&cache_dir),
        ["checkout", "--force", "--detach", checkout_ref.as_str()].as_slice(),
    )?;
    if checkout_result.status != 0 {
        return Err(GearClawError::Other(format!(
            "Failed to checkout source '{}' revision '{}': {}",
            source.name, checkout_ref, checkout_result.stderr
        )));
    }

    let mut signature_verified = false;
    if source.verify_head_commit_signature {
        let verify_result =
            run_git_command(Some(&cache_dir), ["verify-commit", "HEAD"].as_slice())?;
        if verify_result.status != 0 {
            return Err(GearClawError::Other(format!(
                "Signature verification failed for source '{}': {}",
                source.name, verify_result.stderr
            )));
        }
        signature_verified = true;
    }

    if did_network_update {
        write_source_last_sync_epoch(&cache_dir, now_epoch_secs())?;
    } else if source_last_sync_epoch(&cache_dir).is_none() {
        write_source_last_sync_epoch(&cache_dir, now_epoch_secs())?;
    }
    let head_commit = git_head_commit(&cache_dir)?;
    Ok(GitSyncResult {
        cache_dir,
        head_commit,
        signature_verified,
    })
}

fn skill_source_cache_root(config: &Config) -> PathBuf {
    config
        .agent
        .skills_path
        .parent()
        .map(|p| p.join("skill_sources_cache"))
        .unwrap_or_else(|| config.agent.skills_path.join(".skill_sources_cache"))
}

fn stable_hash(input: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

fn should_refresh_source_cache(config: &Config, cache_dir: &Path, force_update: bool) -> bool {
    if force_update {
        return true;
    }

    let ttl = config.agent.skill_source_cache_ttl_seconds;
    if ttl == 0 {
        return true;
    }

    match source_last_sync_epoch(cache_dir) {
        Some(last_sync) => now_epoch_secs().saturating_sub(last_sync) >= ttl,
        None => true,
    }
}

fn source_last_sync_epoch(cache_dir: &Path) -> Option<u64> {
    let path = source_sync_meta_path(cache_dir);
    let raw = std::fs::read_to_string(path).ok()?;
    raw.trim().parse::<u64>().ok()
}

fn write_source_last_sync_epoch(cache_dir: &Path, epoch_secs: u64) -> Result<(), GearClawError> {
    let path = source_sync_meta_path(cache_dir);
    std::fs::write(path, epoch_secs.to_string()).map_err(GearClawError::IoError)
}

fn source_sync_meta_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join(".gearclaw_source_last_sync")
}

fn now_epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

struct GitCommandOutput {
    status: i32,
    stdout: String,
    stderr: String,
}

fn run_git_command(cwd: Option<&Path>, args: &[&str]) -> Result<GitCommandOutput, GearClawError> {
    let mut cmd = Command::new("git");
    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }
    cmd.args(args);

    let output = cmd.output().map_err(GearClawError::IoError)?;
    Ok(GitCommandOutput {
        status: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
    })
}

fn git_head_commit(repo: &Path) -> Result<Option<String>, GearClawError> {
    let output = run_git_command(Some(repo), ["rev-parse", "HEAD"].as_slice())?;
    if output.status != 0 {
        return Ok(None);
    }

    let commit = output.stdout.trim();
    if commit.is_empty() {
        Ok(None)
    } else {
        Ok(Some(commit.to_string()))
    }
}

fn collect_skill_files(root: &Path) -> Result<Vec<PathBuf>, GearClawError> {
    let mut files = Vec::new();
    if !root.exists() || !root.is_dir() {
        return Ok(files);
    }
    collect_skill_files_recursive(root, &mut files)?;
    Ok(files)
}

fn collect_skill_files_recursive(root: &Path, out: &mut Vec<PathBuf>) -> Result<(), GearClawError> {
    for entry in std::fs::read_dir(root).map_err(GearClawError::IoError)? {
        let entry = entry.map_err(GearClawError::IoError)?;
        let path = entry.path();

        if path.is_dir() {
            let skill_md = path.join("SKILL.md");
            if skill_md.is_file() {
                out.push(skill_md);
            } else {
                collect_skill_files_recursive(&path, out)?;
            }
        }
    }
    Ok(())
}

fn parse_skill_metadata(path: &Path) -> Result<(String, String), GearClawError> {
    let content = std::fs::read_to_string(path).map_err(GearClawError::IoError)?;
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 || !parts[0].trim().is_empty() {
        return Err(GearClawError::Other(format!(
            "invalid frontmatter format in {}",
            path.display()
        )));
    }

    let frontmatter = parts[1];
    let mut name: Option<String> = None;
    let mut description: Option<String> = None;

    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(raw) = line.strip_prefix("name:") {
            name = Some(parse_frontmatter_scalar(raw));
        } else if let Some(raw) = line.strip_prefix("description:") {
            description = Some(parse_frontmatter_scalar(raw));
        }
    }

    let name =
        name.ok_or_else(|| GearClawError::Other(format!("missing `name` in {}", path.display())))?;
    let description = description.unwrap_or_else(|| "No description".to_string());
    Ok((name, description))
}

fn parse_frontmatter_scalar(raw: &str) -> String {
    let raw = raw.trim();
    if raw.len() >= 2 {
        if (raw.starts_with('"') && raw.ends_with('"'))
            || (raw.starts_with('\'') && raw.ends_with('\''))
        {
            return raw[1..raw.len() - 1].trim().to_string();
        }
    }
    raw.to_string()
}

fn sanitize_skill_dir_name(name: &str) -> Option<String> {
    let mut out: String = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '-'
            }
        })
        .collect();

    while out.contains("--") {
        out = out.replace("--", "-");
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn is_install_allowed_by_policy(skill: &SkillCatalogEntry, policy: &SkillTrustPolicy) -> bool {
    match policy {
        SkillTrustPolicy::LocalOnly => matches!(skill.source_kind, SkillSourceKind::LocalDir),
        SkillTrustPolicy::TrustedOnly => skill.source_trusted,
        SkillTrustPolicy::AllowUntrusted => true,
    }
}

fn copy_dir_recursive(from: &Path, to: &Path) -> Result<(), GearClawError> {
    std::fs::create_dir_all(to).map_err(GearClawError::IoError)?;

    for entry in std::fs::read_dir(from).map_err(GearClawError::IoError)? {
        let entry = entry.map_err(GearClawError::IoError)?;
        let src_path = entry.path();
        let dst_path = to.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else if src_path.is_file() {
            std::fs::copy(&src_path, &dst_path).map_err(GearClawError::IoError)?;
        }
    }

    Ok(())
}

fn append_install_audit_log(
    config: &Config,
    selected: &SkillCatalogEntry,
    target_dir: &Path,
    policy_label: &str,
    status: &str,
) -> Result<(), GearClawError> {
    use std::io::Write;

    let log_path = skill_install_audit_log_path(config);
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent).map_err(GearClawError::IoError)?;
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(GearClawError::IoError)?;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let line = format!(
        "{}|skill={}|source={}|kind={}|location={}|revision={}|commit={}|sig_verified={}|trusted={}|policy={}|target={}|status={}\n",
        ts,
        sanitize_log_field(&selected.name),
        sanitize_log_field(&selected.source_name),
        source_kind_label(&selected.source_kind),
        sanitize_log_field(&selected.source_location.to_string_lossy()),
        sanitize_log_field(selected.source_revision.as_deref().unwrap_or("default")),
        sanitize_log_field(selected.source_head_commit.as_deref().unwrap_or("unknown")),
        selected.source_signature_verified,
        selected.source_trusted,
        sanitize_log_field(policy_label),
        sanitize_log_field(&target_dir.to_string_lossy()),
        sanitize_log_field(status)
    );
    file.write_all(line.as_bytes())
        .map_err(GearClawError::IoError)?;
    Ok(())
}

fn skill_install_audit_log_path(config: &Config) -> PathBuf {
    config
        .agent
        .skills_path
        .parent()
        .map(|p| p.join("skill_install_audit.log"))
        .unwrap_or_else(|| config.agent.skills_path.join("skill_install_audit.log"))
}

fn source_kind_label(kind: &SkillSourceKind) -> &'static str {
    match kind {
        SkillSourceKind::LocalDir => "local_dir",
        SkillSourceKind::GitRepo => "git_repo",
    }
}

fn sanitize_log_field(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('|', "\\|")
}

async fn handle_gateway(
    config: &Config,
    host: Option<String>,
    port: Option<u16>,
    dev: bool,
    allow_unauthenticated: bool,
) -> Result<(), GearClawError> {
    use gearclaw_channels::platforms::discord::DiscordConfig;
    use gearclaw_channels::{ChannelAdapter, DiscordAdapter};
    use gearclaw_gateway::{GatewayServer, MethodHandlers};
    use std::sync::Arc;

    // Use CLI args or config file values
    let gw_host = host.unwrap_or_else(|| config.gateway.host.clone());
    let gw_port = port.unwrap_or(config.gateway.port);
    let allow_unauthenticated_requests =
        allow_unauthenticated || config.gateway.allow_unauthenticated_requests;

    // Configure logging
    if dev {
        let env_filter =
            EnvFilter::new("gearclaw=debug,gearclaw_gateway=debug,gearclaw_channels=debug");
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
    println!(
        "  鉴权: {}",
        if allow_unauthenticated_requests {
            "关闭（危险）"
        } else {
            "开启（默认）"
        }
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    if allow_unauthenticated_requests {
        tracing::warn!("Gateway unauthenticated mode is enabled. This is unsafe for production.");
    }

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
                // Get source name and sender ID from MessageSource
                let (source_name, sender_id) = match &incoming_msg.source {
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

                // Trigger checks should prefer channel_id (for enabled_channels/disabled_channels),
                // fallback to sender_id when channel metadata is missing.
                let trigger_source_id = incoming_msg
                    .metadata
                    .get("channel_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&sender_id)
                    .to_string();

                // Process message with agent
                tracing::info!("🤖 Calling Agent.process_channel_message()...");

                match agent_clone
                    .process_channel_message(
                        &incoming_msg.platform,
                        &trigger_source_id,
                        &incoming_msg.content,
                    )
                    .await
                {
                    Ok(response) => {
                        tracing::info!(
                            "✅ Agent.process_channel_message() returned, response length: {}",
                            response.len()
                        );

                        if response.is_empty() {
                            tracing::debug!("Agent chose not to respond (trigger not met)");
                        } else {
                            tracing::info!("Agent response: {}", response);

                            // Send response back to Discord
                            use gearclaw_channels::{MessageContent, MessageTarget};

                            let channel_id = match incoming_msg
                                .metadata
                                .get("channel_id")
                                .and_then(|v| v.as_str())
                            {
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
                                tracing::info!(
                                    "✅ Successfully sent response to Discord channel {}",
                                    channel_id
                                );
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
        allow_unauthenticated_requests,
    };

    // Create server with agent integration
    let handlers = MethodHandlers::new();
    handlers.set_agent(agent.clone()).await;
    handlers
        .set_trigger_config(config.agent.triggers.clone())
        .await;

    let server = GatewayServer::new(gw_config).with_handlers(Arc::new(handlers));

    println!("🌐 Gateway 服务器启动中...");
    println!();

    server
        .start()
        .await
        .map_err(|e| GearClawError::Other(format!("Gateway error: {}", e)))?;

    Ok(())
}
