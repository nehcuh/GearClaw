use crate::config::{default_endpoint, Config};
use crate::error::GearClawError;
use crate::llm::{FunctionCall, LLMClient, Message, ToolCall};
use crate::mcp::McpManager;
use crate::memory::MemoryManager;
use crate::session::{Session, SessionManager};
use crate::skills::SkillManager;
use crate::tools::{ToolExecutor, ToolResult};
use futures::StreamExt;
use rustyline::history::DefaultHistory;
use rustyline::Editor;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use tracing::{error, info};

pub struct Agent {
    config: Config,
    llm_client: Arc<LLMClient>,
    tool_executor: ToolExecutor,
    pub session_manager: SessionManager,
    pub skill_manager: SkillManager,
    pub memory_manager: MemoryManager,
    pub mcp_manager: Arc<McpManager>,
}
/// Tool routing abstraction for Agent tool-call dispatch.
pub struct ToolRouter<'a> {
    agent: &'a Agent,
}

impl<'a> ToolRouter<'a> {
    pub fn new(agent: &'a Agent) -> Self {
        Self { agent }
    }

    pub async fn route(
        &self,
        session: &mut Session,
        tool_name: &str,
        arguments: &str,
    ) -> Result<ToolResult, GearClawError> {
        self.agent
            .execute_tool_call(session, tool_name, arguments)
            .await
    }
}

/// LLM loop abstraction for Agent multi-turn tool-calling orchestration.
pub struct LLMLoop<'a> {
    agent: &'a Agent,
}

impl<'a> LLMLoop<'a> {
    pub fn new(agent: &'a Agent) -> Self {
        Self { agent }
    }

    pub async fn run(
        &self,
        session: &mut Session,
        user_message: &str,
    ) -> Result<String, GearClawError> {
        self.agent
            .process_message_inner(session, user_message)
            .await
    }
}

impl Agent {
    pub async fn new(config: Config) -> Result<Self, GearClawError> {
        info!("初始化 Agent: {}", config.agent.name);

        // 优先使用配置文件中的 API key，如果配置文件中没有，则尝试环境变量
        let (api_key, api_key_source) = if let Some(key) = config.llm.api_key.clone() {
            (key, "config file")
        } else if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            (key, "environment variable OPENAI_API_KEY")
        } else {
            return Err(GearClawError::ConfigNotFound(
                "未设置 LLM API key。请在配置中设置或设置环境变量 OPENAI_API_KEY".to_string(),
            ));
        };

        // 优先使用配置文件中的 endpoint，如果配置文件中是默认值，则尝试环境变量
        let (endpoint, endpoint_source) = if config.llm.endpoint != default_endpoint() {
            (config.llm.endpoint.clone(), "config file")
        } else if let Ok(url) = std::env::var("OPENAI_BASE_URL") {
            (url, "environment variable OPENAI_BASE_URL")
        } else {
            (config.llm.endpoint.clone(), "default")
        };

        info!("Using API key from: {}", api_key_source);
        info!("Using endpoint from: {} - {}", endpoint_source, endpoint);
        info!("Using model: {}", config.llm.primary);

        let llm_client = Arc::new(LLMClient::new(
            api_key,
            endpoint,
            config.llm.primary.clone(),
            config.llm.embedding_model.clone(),
            config.llm.temperature,
        ));

        let tool_executor = ToolExecutor::new(&config.tools.security);

        let mut skill_manager = SkillManager::new();
        if let Err(e) = skill_manager.load_from_dir(&config.agent.skills_path) {
            tracing::warn!("Failed to load skills: {}", e);
        }

        let mcp_manager = Arc::new(McpManager::new(config.mcp.clone()));
        if let Err(e) = mcp_manager.init_clients().await {
            tracing::error!("Failed to initialize MCP clients: {}", e);
        }

        let session_manager = SessionManager::new(config.session.clone())?;

        let memory_manager = MemoryManager::new(
            config.memory.clone(),
            config.agent.workspace.clone(),
            llm_client.clone(),
        )?;

        let agent = Agent {
            config,
            llm_client,
            tool_executor,
            session_manager,
            skill_manager,
            memory_manager,
            mcp_manager,
        };

        // Auto-sync memory if enabled
        if agent.config.memory.enabled {
            info!("Memory is enabled, starting initial sync...");
            let memory_manager_for_sync = agent.memory_manager.clone();
            tokio::spawn(async move {
                if let Err(e) = memory_manager_for_sync.sync().await {
                    tracing::warn!("Initial memory sync failed: {}", e);
                }
            });
        }

        Ok(agent)
    }

    pub async fn start_interactive(&self) -> Result<(), GearClawError> {
        let mut session = self.session_manager.get_or_create_session("interactive")?;
        let mut rl = Editor::<(), DefaultHistory>::new()
            .map_err(|e| GearClawError::IoError(std::io::Error::other(e)))?;

        println!("⚙️ GearClaw 交互模式已启动");
        println!("输入 'exit' 或 'quit' 退出");
        println!("输入 'clear' 清除对话历史");
        println!("输入 'help' 查看可用命令");
        println!("提示: 使用 ↑/↓ 浏览历史，左/右移动光标，Backspace/Delete 删除字符");
        println!();

        loop {
            let readline = rl.readline("> ");

            match readline {
                Ok(line) => {
                    let input = line.trim();

                    if input.is_empty() {
                        continue;
                    }

                    // 添加到历史记录（排除特殊命令）
                    if !matches!(input, "exit" | "quit" | "clear" | "help") {
                        let _ = rl.add_history_entry(input);
                    }

                    match input {
                        "exit" | "quit" => {
                            info!("退出交互模式");
                            break;
                        }
                        "clear" => {
                            session.clear_history();
                            println!("✓ 对话历史已清除");
                            let _ = rl.clear_history();
                            continue;
                        }
                        "help" => {
                            self.print_help();
                            continue;
                        }
                        _ => {
                            println!("🤖 GearClaw: ");
                            std::io::stdout().flush().ok();

                            let _ = self.process_message(&mut session, input).await?;
                            println!();
                        }
                    }
                }
                Err(_) => {
                    // Ctrl+D 或 Ctrl+C
                    println!("\n👋 再见！");
                    break;
                }
            }
        }

        self.session_manager.save_session(&session).await?;
        Ok(())
    }

    pub async fn process_message(
        &self,
        session: &mut Session,
        user_message: &str,
    ) -> Result<String, GearClawError> {
        LLMLoop::new(self).run(session, user_message).await
    }

    async fn process_message_inner(
        &self,
        session: &mut Session,
        user_message: &str,
    ) -> Result<String, GearClawError> {
        if !user_message.is_empty() {
            session.add_message(Message {
                role: "user".to_string(),
                content: Some(user_message.to_string()),
                tool_calls: None,
                tool_call_id: None,
            });
        }

        let mut final_response_content = String::new();
        let mut loop_count = 0;

        // Loop for tool calls (max 15 turns to prevent infinite loops)
        while loop_count < 15 {
            loop_count += 1;

            let mut tool_specs = self.tool_executor.available_tools();
            if self.mcp_manager.is_enabled() {
                let mcp_tools = self.mcp_manager.list_tools().await;
                tool_specs.extend(mcp_tools);
            }

            let llm_tools = self.convert_to_llm_tools(tool_specs);

            // Construct messages with system prompt and skills context
            let mut messages = Vec::new();

            // Build system prompt with memory context if enabled
            let mut system_prompt = self.config.agent.system_prompt.clone();
            system_prompt.push_str(&self.skill_manager.get_prompt_context());

            // Search memory if enabled and add to system prompt
            if self.config.agent.memory_enabled && !user_message.is_empty() {
                match self.memory_manager.search(user_message, 3).await {
                    Ok(memories) if !memories.is_empty() => {
                        tracing::debug!("Found {} relevant memories", memories.len());
                        let memory_context = memories
                            .iter()
                            .map(|m| format!("- [{}] {} (score: {:.2})", m.path, m.text, m.score))
                            .collect::<Vec<_>>()
                            .join("\n");

                        system_prompt.push_str("\n\n=== Relevant Context ===\n");
                        system_prompt.push_str("The following information from your memory may be relevant to this conversation:\n\n");
                        system_prompt.push_str(&memory_context);
                        system_prompt.push_str("\n========================\n");
                    }
                    Ok(_) => {
                        tracing::debug!("No relevant memories found");
                    }
                    Err(e) => {
                        tracing::warn!("Memory search failed: {}", e);
                        // Continue without memory context rather than failing
                    }
                }
            }

            messages.push(Message {
                role: "system".to_string(),
                content: Some(system_prompt),
                tool_calls: None,
                tool_call_id: None,
            });

            messages.extend(session.get_messages());

            let mut stream = self
                .llm_client
                .chat_completion_stream(
                    messages,
                    Some(llm_tools.clone()),
                    Some(self.config.session.max_tokens),
                )
                .await?;

            let mut current_content = String::new();
            let mut tool_call_chunks: HashMap<usize, (String, String, String)> = HashMap::new();

            while let Some(result) = stream.next().await {
                match result {
                    Ok(response) => {
                        for choice in response.choices {
                            if let Some(content) = choice.delta.content {
                                print!("{}", content);
                                std::io::stdout().flush().ok();
                                current_content.push_str(&content);
                            }

                            if let Some(tool_calls) = choice.delta.tool_calls {
                                for tc in tool_calls {
                                    let entry = tool_call_chunks.entry(tc.index).or_insert((
                                        String::new(),
                                        String::new(),
                                        String::new(),
                                    ));
                                    if let Some(id) = tc.id {
                                        entry.0 = id;
                                    }
                                    if let Some(func) = tc.function {
                                        if let Some(name) = func.name {
                                            entry.1.push_str(&name);
                                        }
                                        if let Some(args) = func.arguments {
                                            entry.2.push_str(&args);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if e.to_string().contains("Stream finished") {
                            // Normal stream termination
                        } else {
                            error!("Stream chunk error: {}", e);
                        }
                    }
                }
            }

            // Reconstruct tool calls
            let mut tool_calls_vec = Vec::new();
            let mut sorted_calls: Vec<_> = tool_call_chunks.into_iter().collect();
            sorted_calls.sort_by_key(|(idx, _)| *idx);

            for (_, (id, name, args)) in sorted_calls {
                tool_calls_vec.push(ToolCall {
                    id,
                    r#type: "function".to_string(),
                    function: FunctionCall {
                        name,
                        arguments: args,
                    },
                });
            }

            if !current_content.is_empty() {
                final_response_content = current_content.clone();
            }

            // Add Assistant Message
            session.add_message(Message {
                role: "assistant".to_string(),
                content: if current_content.is_empty() {
                    None
                } else {
                    Some(current_content.clone())
                },
                tool_calls: if tool_calls_vec.is_empty() {
                    None
                } else {
                    Some(tool_calls_vec.clone())
                },
                tool_call_id: None,
            });

            if tool_calls_vec.is_empty() {
                break; // Done
            }

            // Execute tools
            println!();
            let tool_router = ToolRouter::new(self);
            for tc in &tool_calls_vec {
                info!("工具调用: {} - {}", tc.function.name, tc.function.arguments);
                let result = tool_router
                    .route(session, &tc.function.name, &tc.function.arguments)
                    .await;

                let output = match result {
                    Ok(res) => res.output,
                    Err(e) => format!("Error: {}", e),
                };

                // Add Tool Message
                session.add_message(Message {
                    role: "tool".to_string(),
                    content: Some(output),
                    tool_calls: None,
                    tool_call_id: Some(tc.id.clone()),
                });
            }
        }

        Ok(final_response_content)
    }

    pub async fn execute_tool_call(
        &self,
        session: &mut Session,
        tool_name: &str,
        arguments: &str,
    ) -> Result<ToolResult, GearClawError> {
        let args: Value = serde_json::from_str(arguments).unwrap_or(json!({}));

        // Check if it's an MCP tool
        if tool_name.contains("__") {
            if !self.mcp_manager.is_enabled() {
                return Err(GearClawError::from(crate::error::DomainError::Mcp {
                    server: tool_name
                        .split("__")
                        .next()
                        .unwrap_or("unknown")
                        .to_string(),
                    reason: "MCP support is disabled in this build".to_string(),
                }));
            }
            return self.mcp_manager.call_tool(tool_name, args).await;
        }

        match tool_name {
            "exec" => {
                if let Some(cmd) = args.get("command").and_then(|v| v.as_str()) {
                    let cmd_args: Vec<String> = args
                        .get("args")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str())
                                .map(|s| s.to_string())
                                .collect()
                        })
                        .unwrap_or_default();

                    if cmd == "cd" {
                        let new_dir = if let Some(path) = cmd_args.first() {
                            std::path::PathBuf::from(path)
                        } else {
                            dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/"))
                        };

                        let new_cwd = if new_dir.is_absolute() {
                            new_dir
                        } else {
                            session.cwd.join(new_dir)
                        };

                        let resolved_cwd = match std::fs::canonicalize(&new_cwd) {
                            Ok(p) => p,
                            Err(_) => new_cwd,
                        };

                        if resolved_cwd.exists() && resolved_cwd.is_dir() {
                            session.cwd = resolved_cwd.clone();
                            Ok(ToolResult {
                                success: true,
                                output: format!("Changed directory to {}", resolved_cwd.display()),
                                error: None,
                            })
                        } else {
                            Ok(ToolResult {
                                success: false,
                                output: "".to_string(),
                                error: Some(format!(
                                    "Directory not found: {}",
                                    resolved_cwd.display()
                                )),
                            })
                        }
                    } else {
                        self.tool_executor
                            .exec_command(cmd, cmd_args, Some(&session.cwd))
                            .await
                    }
                } else {
                    Err(GearClawError::ToolExecutionError(
                        "exec 工具需要 'command' 参数".to_string(),
                    ))
                }
            }
            "read_file" => {
                let path_str = args.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
                    GearClawError::ToolExecutionError("read_file 需要路径参数".to_string())
                })?;
                let start_line = args
                    .get("start_line")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize);
                let end_line = args
                    .get("end_line")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize);

                let path = std::path::Path::new(path_str);

                let full_path = if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    session.cwd.join(path)
                };

                let content =
                    std::fs::read_to_string(&full_path).map_err(GearClawError::IoError)?;

                let output = if start_line.is_some() || end_line.is_some() {
                    let lines: Vec<&str> = content.lines().collect();
                    let start = start_line.unwrap_or(1).saturating_sub(1);
                    let end = end_line.unwrap_or(lines.len());

                    if start >= lines.len() {
                        String::new()
                    } else {
                        let end = std::cmp::min(end, lines.len());
                        lines[start..end]
                            .iter()
                            .enumerate()
                            .map(|(i, line)| format!("{}|{}", start + i + 1, line))
                            .collect::<Vec<_>>()
                            .join("\n")
                    }
                } else {
                    content
                };

                Ok(ToolResult {
                    success: true,
                    output,
                    error: None,
                })
            }
            "write_file" => {
                let path_str = args.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
                    GearClawError::ToolExecutionError("write_file 需要路径参数".to_string())
                })?;
                let content = args
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        GearClawError::ToolExecutionError("write_file 需要内容参数".to_string())
                    })?;

                let path = std::path::Path::new(path_str);
                let full_path = if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    session.cwd.join(path)
                };

                if let Some(parent) = full_path.parent() {
                    std::fs::create_dir_all(parent).map_err(GearClawError::IoError)?;
                }

                std::fs::write(&full_path, content).map_err(GearClawError::IoError)?;

                Ok(ToolResult {
                    success: true,
                    output: format!("文件已写入: {}", full_path.display()),
                    error: None,
                })
            }
            "list_files" => {
                let path_str = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
                let recursive = args
                    .get("recursive")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let max_depth = args
                    .get("max_depth")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize)
                    .unwrap_or(1);

                let path = std::path::Path::new(path_str);
                let full_path = if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    session.cwd.join(path)
                };

                if !full_path.exists() {
                    return Ok(ToolResult {
                        success: false,
                        output: "".to_string(),
                        error: Some(format!("Directory not found: {}", full_path.display())),
                    });
                }

                let mut output = String::new();
                let mut stack = vec![(full_path.clone(), 0)];

                while let Some((dir, depth)) = stack.pop() {
                    if depth > max_depth {
                        continue;
                    }

                    if let Ok(entries) = std::fs::read_dir(&dir) {
                        let mut entries_vec: Vec<_> = entries.filter_map(Result::ok).collect();
                        entries_vec.sort_by_key(|e| e.file_name());

                        for entry in entries_vec {
                            let file_type = entry.file_type().map_err(GearClawError::IoError)?;
                            let name = entry.file_name().to_string_lossy().to_string();
                            let prefix = "  ".repeat(depth);

                            if file_type.is_dir() {
                                output.push_str(&format!("{}📂 {}/\n", prefix, name));
                                if recursive && depth < max_depth {
                                    stack.push((entry.path(), depth + 1));
                                }
                            } else {
                                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                                output.push_str(&format!("{}📄 {} ({})\n", prefix, name, size));
                            }
                        }
                    }
                }

                Ok(ToolResult {
                    success: true,
                    output,
                    error: None,
                })
            }
            "file_info" => {
                let path_str = args.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
                    GearClawError::ToolExecutionError("file_info 需要路径参数".to_string())
                })?;
                let path = std::path::Path::new(path_str);
                let full_path = if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    session.cwd.join(path)
                };

                if !full_path.exists() {
                    return Ok(ToolResult {
                        success: false,
                        output: "".to_string(),
                        error: Some(format!("Path not found: {}", full_path.display())),
                    });
                }

                let metadata = std::fs::metadata(&full_path).map_err(GearClawError::IoError)?;
                let file_type = if metadata.is_dir() {
                    "Directory"
                } else {
                    "File"
                };
                let size = metadata.len();
                let modified = metadata
                    .modified()
                    .ok()
                    .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
                    .unwrap_or_else(|| "Unknown".to_string());

                let info = format!(
                    "Type: {}\nSize: {} bytes\nModified: {}\nPath: {}",
                    file_type,
                    size,
                    modified,
                    full_path.display()
                );

                Ok(ToolResult {
                    success: true,
                    output: info,
                    error: None,
                })
            }
            "web_search" => {
                let query = args.get("query").and_then(|v| v.as_str()).ok_or_else(|| {
                    GearClawError::ToolExecutionError("web_search 需要查询参数".to_string())
                })?;

                self.tool_executor
                    .exec_command(
                        "curl",
                        vec![
                            "-s".to_string(),
                            "-L".to_string(),
                            format!(
                                "https://api.duckduckgo.com/?q={}",
                                urlencoding::encode(query)
                            ),
                        ],
                        None,
                    )
                    .await
            }
            "git_status" => {
                self.tool_executor
                    .exec_command("git", vec!["status".to_string()], Some(&session.cwd))
                    .await
            }
            "docker_ps" => {
                self.tool_executor
                    .exec_command("docker", vec!["ps".to_string()], Some(&session.cwd))
                    .await
            }
            _ => {
                // Check if it's a macOS-specific tool
                #[cfg(target_os = "macos")]
                if tool_name.starts_with("macos_") {
                    let output = self
                        .tool_executor
                        .macos
                        .execute_tool(tool_name, &args)
                        .await?;
                    return Ok(ToolResult {
                        success: true,
                        output,
                        error: None,
                    });
                }

                Err(GearClawError::ToolNotFound(tool_name.to_string()))
            }
        }
    }

    fn convert_to_llm_tools(
        &self,
        tools: Vec<crate::tools::ToolSpec>,
    ) -> Vec<crate::llm::ToolSpec> {
        tools
            .into_iter()
            .map(|tool| {
                let parameters = tool.parameters.unwrap_or_else(|| {
                    json!({
                        "type": "object",
                        "properties": {},
                        "required": []
                    })
                });

                crate::llm::ToolSpec {
                    r#type: "function".to_string(),
                    function: crate::llm::ToolFunction {
                        name: tool.name.clone(),
                        description: tool.description.clone(),
                        parameters,
                    },
                }
            })
            .collect()
    }

    fn print_help(&self) {
        println!("📖 可用工具:");
        println!();

        for tool in self.tool_executor.available_tools() {
            println!("  • {} - {}", tool.name, tool.description);
        }
    }

    /// Process a message from a channel (Discord, Telegram, etc.)
    ///
    /// Parameters:
    /// - platform: Platform name (e.g., "discord", "telegram")
    /// - source_id: User or channel ID from the platform
    /// - content: Message content
    pub async fn process_channel_message(
        &self,
        platform: &str,
        source_id: &str,
        content: &str,
    ) -> Result<String, GearClawError> {
        // Create session ID from platform and source
        let session_id = format!("{}:{}", platform, source_id);

        // Get or create session
        let mut session = self.session_manager.get_or_create_session(&session_id)?;

        // Add user message to session
        session.add_message(Message {
            role: "user".to_string(),
            content: Some(content.to_string()),
            tool_calls: None,
            tool_call_id: None,
        });

        // Check if agent should respond
        let should_respond = self.should_respond_to_message(platform, source_id, content)?;

        if !should_respond {
            tracing::debug!("Agent not triggered for message: {}", content);
            return Ok(String::new());
        }

        // Process message and get response
        let response = self.process_message(&mut session, content).await?;

        // Save session
        self.session_manager.save_session(&session).await?;

        Ok(response)
    }

    /// Check if agent should respond to a message based on trigger mode
    fn should_respond_to_message(
        &self,
        platform: &str,
        source_id: &str,
        content: &str,
    ) -> Result<bool, GearClawError> {
        let trigger_config = &self.config.agent.triggers;

        // Check channel whitelist/blacklist
        let channel_key = format!("{}:{}", platform, source_id);

        // Check disabled channels (blacklist)
        if trigger_config.disabled_channels.contains(&channel_key) {
            return Ok(false);
        }

        // Check enabled channels (whitelist)
        if !trigger_config.enabled_channels.is_empty()
            && !trigger_config.enabled_channels.contains(&channel_key)
        {
            return Ok(false);
        }

        // Check trigger mode
        match trigger_config.mode {
            crate::config::TriggerMode::Always => Ok(true),
            crate::config::TriggerMode::Mention => {
                // Check if message starts with mention pattern
                for pattern in &trigger_config.mention_patterns {
                    if content.starts_with(pattern) || content.contains(pattern) {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            crate::config::TriggerMode::Keyword => {
                // Check if message contains any keyword
                for keyword in &trigger_config.keywords {
                    if content.to_lowercase().contains(&keyword.to_lowercase()) {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }
}

#[allow(dead_code)]
pub struct AgentConfig {
    pub interactive_timeout: Option<u64>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        AgentConfig {
            interactive_timeout: Some(300), // 5 minutes
        }
    }
}
