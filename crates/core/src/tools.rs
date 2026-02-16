use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::process::Command;
use tracing::{info, error, debug};
use crate::error::GearClawError;
use crate::macos::MacosController;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub requires_args: bool,
    #[serde(default)]
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

/// Tool executor for running shell commands and other operations
pub struct ToolExecutor {
    security_level: SecurityLevel,
    #[cfg(target_os = "macos")]
    pub(crate) macos: MacosController,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SecurityLevel {
    Deny,
    Allowlist,
    Full,
}

impl ToolExecutor {
    pub fn new(security: &str) -> Self {
        let level = match security.to_lowercase().as_str() {
            "deny" => SecurityLevel::Deny,
            "allowlist" => SecurityLevel::Allowlist,
            "full" | _ => SecurityLevel::Full,
        };

        #[cfg(target_os = "macos")]
        let macos = MacosController::new().expect("Failed to initialize macOS controller");

        ToolExecutor {
            security_level: level,
            #[cfg(target_os = "macos")]
            macos,
        }
    }
    
    /// Execute a shell command
    pub async fn exec_command(&self, cmd: &str, args: Vec<String>, cwd: Option<&std::path::Path>) -> Result<ToolResult, GearClawError> {
        if self.security_level == SecurityLevel::Deny {
            return Err(GearClawError::ToolExecutionError(
                "工具执行被禁止 (security=deny)".to_string()
            ));
        }
        
        info!("执行命令: {} {:?} (cwd: {:?})", cmd, args, cwd);
        
        let output = if self.security_level == SecurityLevel::Allowlist {
            // In allowlist mode, only allow safe commands
            if !self.is_safe_command(cmd) {
                return Err(GearClawError::ToolExecutionError(
                    format!("命令不在允许列表中: {}", cmd)
                ));
            }
            self.execute_safe_command(cmd, args, cwd).await?
        } else {
            // Full mode - execute any command
            self.execute_any_command(cmd, args, cwd).await?
        };
        
        Ok(ToolResult {
            success: true,
            output,
            error: None,
        })
    }
    
    fn is_safe_command(&self, cmd: &str) -> bool {
        const SAFE_COMMANDS: &[&str] = &[
            "ls", "cat", "grep", "head", "tail", "find",
            "git", "npm", "node", "cargo", "rustc",
            "go", "java", "python", "python3", "pip",
            "docker", "docker-compose", "kubectl",
            "curl", "wget", "mkdir", "cp", "mv", "echo",
        ];
        
        SAFE_COMMANDS.contains(&cmd)
    }
    
    async fn execute_safe_command(&self, cmd: &str, args: Vec<String>, cwd: Option<&std::path::Path>) -> Result<String, GearClawError> {
        self.execute_any_command(cmd, args, cwd).await
    }
    
    async fn execute_any_command(&self, cmd: &str, args: Vec<String>, cwd: Option<&std::path::Path>) -> Result<String, GearClawError> {
        let mut command;
        
        // Use shell execution to support complex commands and simple strings
        if cfg!(target_os = "windows") {
            command = Command::new("cmd");
            command.arg("/C");
            
            // Reconstruct command line
            let mut full_cmd = cmd.to_string();
            for arg in args {
                full_cmd.push_str(" ");
                full_cmd.push_str(&arg);
            }
            command.arg(full_cmd);
        } else {
            command = Command::new("sh");
            command.arg("-c");
            
            // Reconstruct command line
            let mut full_cmd = cmd.to_string();
            for arg in args {
                full_cmd.push_str(" ");
                // Basic escaping for safety might be good, but for now raw string
                full_cmd.push_str(&arg);
            }
            command.arg(full_cmd);
        }

        if let Some(dir) = cwd {
            command.current_dir(dir);
        }
        
        let output = command
            .output()
            .await
            .map_err(|e| GearClawError::ToolExecutionError(format!("执行失败: {}", e)))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        if !output.status.success() {
            error!("命令执行失败: {} stderr: {}", cmd, stderr);
            return Err(GearClawError::ToolExecutionError(
                format!("命令返回错误码: {}", output.status)
            ));
        }
        
        debug!("命令输出: {}", stdout);
        Ok(stdout)
    }
    
    /// Get list of available tools
    pub fn available_tools(&self) -> Vec<ToolSpec> {
        let mut tools = vec![
            ToolSpec {
                name: "exec".to_string(),
                description: "执行 shell 命令".to_string(),
                requires_args: true,
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "command": { "type": "string", "description": "要执行的命令" },
                        "args": { "type": "array", "items": { "type": "string" }, "description": "命令参数" }
                    },
                    "required": ["command"]
                })),
            },
            ToolSpec {
                name: "read_file".to_string(),
                description: "读取文件内容 (支持行号范围)".to_string(),
                requires_args: true,
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "文件路径" },
                        "start_line": { "type": "integer", "description": "起始行号 (1-based, 可选)" },
                        "end_line": { "type": "integer", "description": "结束行号 (1-based, 可选)" }
                    },
                    "required": ["path"]
                })),
            },
            ToolSpec {
                name: "write_file".to_string(),
                description: "写入文件内容".to_string(),
                requires_args: true,
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "文件路径" },
                        "content": { "type": "string", "description": "文件内容" }
                    },
                    "required": ["path", "content"]
                })),
            },
            ToolSpec {
                name: "list_files".to_string(),
                description: "列出目录下的文件和子目录".to_string(),
                requires_args: true,
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "目录路径 (默认当前目录)" },
                        "recursive": { "type": "boolean", "description": "是否递归列出子目录" },
                        "max_depth": { "type": "integer", "description": "最大递归深度" }
                    },
                    "required": []
                })),
            },
            ToolSpec {
                name: "file_info".to_string(),
                description: "获取文件或目录的元数据(大小、修改时间等)".to_string(),
                requires_args: true,
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "文件或目录路径" }
                    },
                    "required": ["path"]
                })),
            },
            ToolSpec {
                name: "web_search".to_string(),
                description: "使用命令行搜索网页内容，返回文本结果（不打开浏览器）。适合快速获取信息，但用户看不到浏览器界面。".to_string(),
                requires_args: true,
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "搜索查询" }
                    },
                    "required": ["query"]
                })),
            },
            ToolSpec {
                name: "git_status".to_string(),
                description: "查看 Git 状态".to_string(),
                requires_args: false,
                parameters: None,
            },
            ToolSpec {
                name: "docker_ps".to_string(),
                description: "列出运行中的容器".to_string(),
                requires_args: false,
                parameters: None,
            },
        ];

        // Add macOS-specific tools if on macOS
        #[cfg(target_os = "macos")]
        {
            let macos_tools = vec![
                ToolSpec {
                    name: "macos_launch_app".to_string(),
                    description: "启动 macOS 应用程序".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "app_name": { "type": "string", "description": "应用名称 (如 Safari, Chrome, Terminal)" }
                        },
                        "required": ["app_name"]
                    })),
                },
                ToolSpec {
                    name: "macos_quit_app".to_string(),
                    description: "退出 macOS 应用程序".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "app_name": { "type": "string", "description": "应用名称" }
                        },
                        "required": ["app_name"]
                    })),
                },
                ToolSpec {
                    name: "macos_bring_to_front".to_string(),
                    description: "将应用程序切换到前台".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "app_name": { "type": "string", "description": "应用名称" }
                        },
                        "required": ["app_name"]
                    })),
                },
                ToolSpec {
                    name: "macos_is_running".to_string(),
                    description: "检查应用是否正在运行".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "app_name": { "type": "string", "description": "应用名称" }
                        },
                        "required": ["app_name"]
                    })),
                },
                ToolSpec {
                    name: "macos_applescript".to_string(),
                    description: "执行 AppleScript 代码".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "script": { "type": "string", "description": "AppleScript 代码" }
                        },
                        "required": ["script"]
                    })),
                },
                ToolSpec {
                    name: "macos_jxa".to_string(),
                    description: "执行 JavaScript for Automation (JXA) 代码".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "script": { "type": "string", "description": "JXA JavaScript 代码" }
                        },
                        "required": ["script"]
                    })),
                },
                ToolSpec {
                    name: "macos_type_text".to_string(),
                    description: "模拟键盘输入文本".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "text": { "type": "string", "description": "要输入的文本" }
                        },
                        "required": ["text"]
                    })),
                },
                ToolSpec {
                    name: "macos_key_combo".to_string(),
                    description: "模拟组合键 (如 cmd+c, cmd+v)".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "keys": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "按键数组，如 [\"cmd\", \"c\"] 或 [\"cmd\", \"shift\", \"3\"]"
                            }
                        },
                        "required": ["keys"]
                    })),
                },
                ToolSpec {
                    name: "macos_clipboard_read".to_string(),
                    description: "读取剪贴板内容".to_string(),
                    requires_args: false,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {},
                        "required": []
                    })),
                },
                ToolSpec {
                    name: "macos_clipboard_write".to_string(),
                    description: "写入剪贴板内容".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "text": { "type": "string", "description": "要复制到剪贴板的文本" }
                        },
                        "required": ["text"]
                    })),
                },
                ToolSpec {
                    name: "macos_notify".to_string(),
                    description: "发送系统通知".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "title": { "type": "string", "description": "通知标题 (默认: GearClaw)" },
                            "message": { "type": "string", "description": "通知内容" },
                            "sound": { "type": "boolean", "description": "是否播放提示音 (默认: false)" }
                        },
                        "required": ["message"]
                    })),
                },
                ToolSpec {
                    name: "macos_open_url".to_string(),
                    description: "在默认浏览器中打开 URL".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "url": { "type": "string", "description": "要打开的 URL" }
                        },
                        "required": ["url"]
                    })),
                },
                ToolSpec {
                    name: "macos_search_in_browser".to_string(),
                    description: "在浏览器中执行搜索，打开浏览器窗口让用户可以看到和浏览搜索结果。当用户想\"在浏览器中查看\"或\"打开浏览器搜索\"时使用此工具。".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "query": { "type": "string", "description": "搜索关键词" }
                        },
                        "required": ["query"]
                    })),
                },
                ToolSpec {
                    name: "macos_say".to_string(),
                    description: "文字转语音 (TTS)".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "text": { "type": "string", "description": "要朗读的文本" },
                            "voice": { "type": "string", "description": "语音名称 (可选，如 'Ting-Ting')" },
                            "rate": { "type": "integer", "description": "语速 (默认: 175)" }
                        },
                        "required": ["text"]
                    })),
                },
            ];

            tools.extend(macos_tools);
        }

        tools
    }
}
