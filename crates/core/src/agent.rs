use crate::config::Config;
use crate::llm::{LLMClient, Message, ToolCall, FunctionCall};
use crate::tools::{ToolExecutor, ToolResult};
use crate::session::{Session, SessionManager};
use crate::skills::SkillManager;
use crate::memory::MemoryManager;
use crate::mcp::McpManager;
use crate::error::GearClawError;
use tracing::{info, error};
use serde_json::{json, Value};
use futures::StreamExt;
use std::io::Write;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Agent {
    config: Config,
    llm_client: Arc<LLMClient>,
    tool_executor: ToolExecutor,
    pub session_manager: SessionManager,
    pub skill_manager: SkillManager,
    pub memory_manager: MemoryManager,
    pub mcp_manager: Arc<McpManager>,
}

impl Agent {
    pub async fn new(config: Config) -> Result<Self, GearClawError> {
        info!("初始化 Agent: {}", config.agent.name);
        
        let api_key = config.llm.api_key.clone()
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .ok_or_else(|| GearClawError::ConfigNotFound(
                "未设置 LLM API key。请在配置中设置或设置环境变量 OPENAI_API_KEY".to_string()
            ))?;
        
        let endpoint = std::env::var("OPENAI_BASE_URL")
            .unwrap_or_else(|_| config.llm.endpoint.clone());

        let llm_client = Arc::new(LLMClient::new(
            api_key,
            endpoint,
            config.llm.primary.clone(),
            config.llm.embedding_model.clone(),
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
        
        Ok(Agent {
            config,
            llm_client,
            tool_executor,
            session_manager,
            skill_manager,
            memory_manager,
            mcp_manager,
        })
    }
    
    pub async fn start_interactive(&self) -> Result<(), GearClawError> {
        let mut session = self.session_manager.get_or_create_session("interactive")?;
        
        println!("⚙️ GearClaw 交互模式已启动");
        println!("输入 'exit' 或 'quit' 退出");
        println!("输入 'clear' 清除对话历史");
        println!("输入 'help' 查看可用命令");
        println!();
        
        loop {
            print!("> ");
            std::io::stdout().flush().ok();
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)
                .map_err(|e| GearClawError::IoError(e))?;
            
            let input = input.trim();
            
            if input.is_empty() {
                continue;
            }
            
            match input {
                "exit" | "quit" => {
                    info!("退出交互模式");
                    break;
                }
                "clear" => {
                    session.clear_history();
                    println!("✓ 对话历史已清除");
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
        
        self.session_manager.save_session(&session).await?;
        Ok(())
    }
    
    pub async fn process_message(&self, session: &mut Session, user_message: &str) -> Result<String, GearClawError> {
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
            let mcp_tools = self.mcp_manager.list_tools().await;
            tool_specs.extend(mcp_tools);
            
            let llm_tools = self.convert_to_llm_tools(tool_specs);
            
            // Construct messages with system prompt and skills context
            let mut messages = Vec::new();
            
            let system_prompt = format!("{}{}", 
                self.config.agent.system_prompt,
                self.skill_manager.get_prompt_context()
            );
            
            messages.push(Message {
                role: "system".to_string(),
                content: Some(system_prompt),
                tool_calls: None,
                tool_call_id: None,
            });
            
            messages.extend(session.get_messages());
            
            let mut stream = self.llm_client.chat_completion_stream(
                messages,
                Some(llm_tools.clone()),
                Some(self.config.session.max_tokens),
            ).await?;
            
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
                                    let entry = tool_call_chunks.entry(tc.index).or_insert((String::new(), String::new(), String::new()));
                                    if let Some(id) = tc.id { entry.0 = id; }
                                    if let Some(func) = tc.function {
                                        if let Some(name) = func.name { entry.1.push_str(&name); }
                                        if let Some(args) = func.arguments { entry.2.push_str(&args); }
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
                         arguments: args
                     }
                 });
            }

            if !current_content.is_empty() {
                final_response_content = current_content.clone();
            }

            // Add Assistant Message
            session.add_message(Message {
                role: "assistant".to_string(),
                content: if current_content.is_empty() { None } else { Some(current_content.clone()) },
                tool_calls: if tool_calls_vec.is_empty() { None } else { Some(tool_calls_vec.clone()) },
                tool_call_id: None,
            });
            
            if tool_calls_vec.is_empty() {
                break; // Done
            }
            
            // Execute tools
            println!(); 
            for tc in &tool_calls_vec {
                 info!("工具调用: {} - {}", tc.function.name, tc.function.arguments);
                 
                 let result = self.execute_tool_call(session, &tc.function.name, &tc.function.arguments).await;
                 
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
    
    pub async fn execute_tool_call(&self, session: &mut Session, tool_name: &str, arguments: &str) -> Result<ToolResult, GearClawError> {
        let args: Value = serde_json::from_str(arguments).unwrap_or(json!({}));
        
        // Check if it's an MCP tool
        if tool_name.contains("__") {
            return self.mcp_manager.call_tool(tool_name, args).await;
        }

        match tool_name {
            "exec" => {
                if let Some(cmd) = args.get("command").and_then(|v| v.as_str()) {
                    let cmd_args: Vec<String> = args.get("args")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
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
                                error: Some(format!("Directory not found: {}", resolved_cwd.display())),
                            })
                        }
                    } else {
                        self.tool_executor.exec_command(cmd, cmd_args, Some(&session.cwd)).await
                    }
                } else {
                    Err(GearClawError::ToolExecutionError("exec 工具需要 'command' 参数".to_string()))
                }
            }
            "read_file" => {
                let path_str = args.get("path").and_then(|v| v.as_str()).ok_or_else(|| GearClawError::ToolExecutionError("read_file 需要路径参数".to_string()))?;
                let start_line = args.get("start_line").and_then(|v| v.as_u64()).map(|v| v as usize);
                let end_line = args.get("end_line").and_then(|v| v.as_u64()).map(|v| v as usize);
                
                let path = std::path::Path::new(path_str);
                
                let full_path = if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    session.cwd.join(path)
                };
                
                let content = std::fs::read_to_string(&full_path).map_err(GearClawError::IoError)?;
                
                let output = if start_line.is_some() || end_line.is_some() {
                    let lines: Vec<&str> = content.lines().collect();
                    let start = start_line.unwrap_or(1).saturating_sub(1);
                    let end = end_line.unwrap_or(lines.len());
                    
                    if start >= lines.len() {
                        String::new()
                    } else {
                        let end = std::cmp::min(end, lines.len());
                        lines[start..end].iter().enumerate()
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
                let path_str = args.get("path").and_then(|v| v.as_str()).ok_or_else(|| GearClawError::ToolExecutionError("write_file 需要路径参数".to_string()))?;
                let content = args.get("content").and_then(|v| v.as_str()).ok_or_else(|| GearClawError::ToolExecutionError("write_file 需要内容参数".to_string()))?;
                
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
                let recursive = args.get("recursive").and_then(|v| v.as_bool()).unwrap_or(false);
                let max_depth = args.get("max_depth").and_then(|v| v.as_u64()).map(|v| v as usize).unwrap_or(1);
                
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
                    if depth > max_depth { continue; }
                    
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
                let path_str = args.get("path").and_then(|v| v.as_str()).ok_or_else(|| GearClawError::ToolExecutionError("file_info 需要路径参数".to_string()))?;
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
                let file_type = if metadata.is_dir() { "Directory" } else { "File" };
                let size = metadata.len();
                let modified = metadata.modified().ok()
                    .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
                    .unwrap_or_else(|| "Unknown".to_string());
                    
                let info = format!(
                    "Type: {}\nSize: {} bytes\nModified: {}\nPath: {}", 
                    file_type, size, modified, full_path.display()
                );
                
                Ok(ToolResult {
                    success: true,
                    output: info,
                    error: None,
                })
            }
            "web_search" => {
                let query = args.get("query").and_then(|v| v.as_str()).ok_or_else(|| GearClawError::ToolExecutionError("web_search 需要查询参数".to_string()))?;
                
                self.tool_executor.exec_command("curl", vec![
                    "-s".to_string(), 
                    "-L".to_string(), 
                    format!("https://api.duckduckgo.com/?q={}", urlencoding::encode(query))
                ], None).await
            }
            "git_status" => {
                self.tool_executor.exec_command("git", vec!["status".to_string()], Some(&session.cwd)).await
            }
            "docker_ps" => {
                self.tool_executor.exec_command("docker", vec!["ps".to_string()], Some(&session.cwd)).await
            }
            _ => {
                Err(GearClawError::ToolNotFound(tool_name.to_string()))
            }
        }
    }
    
    fn convert_to_llm_tools(&self, tools: Vec<crate::tools::ToolSpec>) -> Vec<crate::llm::ToolSpec> {
        tools.into_iter().map(|tool| {
            let parameters = tool.parameters.unwrap_or_else(|| json!({
                "type": "object",
                "properties": {},
                "required": []
            }));

            crate::llm::ToolSpec {
                r#type: "function".to_string(),
                function: crate::llm::ToolFunction {
                    name: tool.name.clone(),
                    description: tool.description.clone(),
                    parameters,
                },
            }
        }).collect()
    }
    
    fn print_help(&self) {
        println!("📖 可用工具:");
        println!();
        
        for tool in self.tool_executor.available_tools() {
            println!("  • {} - {}", tool.name, tool.description);
        }
        
        println!();
        println!("💡 使用方法:");
        println!("  直接输入问题，GearClaw 会自动调用适当的工具");
        println!("  例如: '列出当前目录的文件'");
        println!("  例如: '查看 git 状态'");
        println!("  例如: '帮我写一个 Rust Hello World 程序'");
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
