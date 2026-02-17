use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use tokio::process::Command;
use tracing::{debug, error, info};

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("execution error: {0}")]
    Execution(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub requires_args: bool,
    #[serde(default)]
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SecurityLevel {
    Deny,
    Allowlist,
    Full,
}

pub struct ToolExecutor {
    security_level: SecurityLevel,
}

impl ToolExecutor {
    pub fn new(security: &str) -> Self {
        let security_level = match security.to_lowercase().as_str() {
            "deny" => SecurityLevel::Deny,
            "allowlist" => SecurityLevel::Allowlist,
            _ => SecurityLevel::Full,
        };
        Self { security_level }
    }

    pub async fn exec_command(
        &self,
        cmd: &str,
        args: Vec<String>,
        cwd: Option<&std::path::Path>,
    ) -> Result<ToolResult, ToolError> {
        if self.security_level == SecurityLevel::Deny {
            return Err(ToolError::Execution(
                "工具执行被禁止 (security=deny)".to_string(),
            ));
        }
        info!("执行命令: {} {:?} (cwd: {:?})", cmd, args, cwd);

        let output = if self.security_level == SecurityLevel::Allowlist {
            if !self.is_safe_command(cmd) {
                return Err(ToolError::Execution(format!("命令不在允许列表中: {}", cmd)));
            }
            self.execute_any_command(cmd, args, cwd).await?
        } else {
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
            "ls",
            "cat",
            "grep",
            "head",
            "tail",
            "find",
            "git",
            "npm",
            "node",
            "cargo",
            "rustc",
            "go",
            "java",
            "python",
            "python3",
            "pip",
            "docker",
            "docker-compose",
            "kubectl",
            "curl",
            "wget",
            "mkdir",
            "cp",
            "mv",
            "echo",
        ];
        SAFE_COMMANDS.contains(&cmd)
    }

    async fn execute_any_command(
        &self,
        cmd: &str,
        args: Vec<String>,
        cwd: Option<&std::path::Path>,
    ) -> Result<String, ToolError> {
        let mut command;
        if cfg!(target_os = "windows") {
            command = Command::new("cmd");
            command.arg("/C");
            let mut full_cmd = cmd.to_string();
            for arg in args {
                full_cmd.push(' ');
                full_cmd.push_str(&arg);
            }
            command.arg(full_cmd);
        } else {
            command = Command::new("sh");
            command.arg("-c");
            let mut full_cmd = cmd.to_string();
            for arg in args {
                full_cmd.push(' ');
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
            .map_err(|e| ToolError::Execution(format!("执行失败: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if !output.status.success() {
            error!("命令执行失败: {} stderr: {}", cmd, stderr);
            return Err(ToolError::Execution(format!(
                "命令返回错误码: {}",
                output.status
            )));
        }
        debug!("命令输出: {}", stdout);
        Ok(stdout)
    }

    pub fn available_tools(&self) -> Vec<ToolSpec> {
        vec![
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
                    "properties": { "query": { "type": "string", "description": "搜索查询" } },
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
        ]
    }
}

pub trait ToolRegistry {
    fn list_tools(&self) -> Vec<ToolSpec>;
}

impl ToolRegistry for ToolExecutor {
    fn list_tools(&self) -> Vec<ToolSpec> {
        self.available_tools()
    }
}
