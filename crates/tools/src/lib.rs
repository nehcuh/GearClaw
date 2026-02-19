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
        Self::validate_exec_input(cmd, &args)?;
        if self.security_level == SecurityLevel::Deny {
            return Err(ToolError::Execution(
                "工具执行被禁止 (security=deny)".to_string(),
            ));
        }
        info!("执行命令: {} {:?} (cwd: {:?})", cmd, args, cwd);

        let output = if self.security_level == SecurityLevel::Allowlist {
            self.validate_allowlist_policy(cmd, &args)?;
            self.execute_any_command(cmd, &args, cwd).await?
        } else {
            self.execute_any_command(cmd, &args, cwd).await?
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
    fn validate_exec_input(cmd: &str, args: &[String]) -> Result<(), ToolError> {
        if cmd.trim().is_empty() {
            return Err(ToolError::Execution("命令不能为空".to_string()));
        }

        if cmd.contains('\0') {
            return Err(ToolError::Execution("命令包含非法空字符".to_string()));
        }

        if args.iter().any(|arg| arg.contains('\0')) {
            return Err(ToolError::Execution("参数包含非法空字符".to_string()));
        }

        Ok(())
    }

    fn validate_allowlist_policy(&self, cmd: &str, args: &[String]) -> Result<(), ToolError> {
        if !self.is_safe_command(cmd) {
            return Err(ToolError::Execution(format!("命令不在允许列表中: {}", cmd)));
        }

        if let Some(reason) = Self::allowlist_block_reason(cmd, args) {
            return Err(ToolError::Execution(reason));
        }

        Ok(())
    }

    fn allowlist_block_reason(cmd: &str, args: &[String]) -> Option<String> {
        const DANGEROUS_TOKENS: &[&str] = &["&&", "||", ";", "|", "`", "$("];
        if args.iter().any(|arg| {
            arg.contains('\n')
                || arg.contains('\r')
                || DANGEROUS_TOKENS.iter().any(|token| arg.contains(token))
        }) {
            return Some("参数中包含潜在命令注入 token".to_string());
        }

        match cmd {
            "python" | "python3" => {
                if args.iter().any(|arg| arg == "-c") {
                    return Some("allowlist 模式禁止 python -c 动态执行".to_string());
                }
            }
            "node" => {
                if args
                    .iter()
                    .any(|arg| matches!(arg.as_str(), "-e" | "--eval" | "-p"))
                {
                    return Some("allowlist 模式禁止 node eval 参数".to_string());
                }
            }
            "git" => {
                const ALLOWED_GIT_SUBCOMMANDS: &[&str] = &[
                    "status",
                    "diff",
                    "log",
                    "show",
                    "branch",
                    "rev-parse",
                    "ls-files",
                ];
                let subcommand = args
                    .iter()
                    .find(|arg| !arg.starts_with('-'))
                    .map(String::as_str)
                    .unwrap_or("status");
                if !ALLOWED_GIT_SUBCOMMANDS.contains(&subcommand) {
                    return Some(format!("allowlist 模式禁止 git 子命令: {}", subcommand));
                }
            }
            "docker" | "docker-compose" => {
                const ALLOWED_DOCKER_SUBCOMMANDS: &[&str] = &["ps", "images", "logs", "inspect"];
                let subcommand = args
                    .iter()
                    .find(|arg| !arg.starts_with('-'))
                    .map(String::as_str)
                    .unwrap_or("ps");
                if !ALLOWED_DOCKER_SUBCOMMANDS.contains(&subcommand) {
                    return Some(format!("allowlist 模式禁止 docker 子命令: {}", subcommand));
                }
            }
            "cargo" => {
                const ALLOWED_CARGO_SUBCOMMANDS: &[&str] =
                    &["build", "check", "test", "fmt", "clippy", "run", "metadata"];
                let subcommand = args
                    .iter()
                    .find(|arg| !arg.starts_with('-'))
                    .map(String::as_str)
                    .unwrap_or("build");
                if !ALLOWED_CARGO_SUBCOMMANDS.contains(&subcommand) {
                    return Some(format!("allowlist 模式禁止 cargo 子命令: {}", subcommand));
                }
            }
            _ => {}
        }

        None
    }

    async fn execute_any_command(
        &self,
        cmd: &str,
        args: &[String],
        cwd: Option<&std::path::Path>,
    ) -> Result<String, ToolError> {
        let mut command = Command::new(cmd);
        command.args(args);

        if let Some(dir) = cwd {
            command.current_dir(dir);
        }

        let output = command
            .output()
            .await
            .map_err(|e| ToolError::Execution(format!("执行失败: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if !output.status.success() {
            error!("命令执行失败: {} stderr: {}", cmd, stderr);
            let message = if stderr.is_empty() {
                format!("命令返回错误码: {}", output.status)
            } else {
                format!("命令返回错误码: {}, stderr: {}", output.status, stderr)
            };
            return Err(ToolError::Execution(message));
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

#[cfg(test)]
mod tests {
    use super::ToolExecutor;

    #[test]
    fn allowlist_blocks_shell_control_tokens() {
        let executor = ToolExecutor::new("allowlist");
        let result =
            executor.validate_allowlist_policy("ls", &[String::from("&&"), String::from("whoami")]);
        assert!(result.is_err());
    }

    #[test]
    fn allowlist_blocks_eval_flags() {
        let executor = ToolExecutor::new("allowlist");
        let py_result = executor.validate_allowlist_policy("python3", &[String::from("-c")]);
        let node_result = executor.validate_allowlist_policy("node", &[String::from("--eval")]);
        assert!(py_result.is_err());
        assert!(node_result.is_err());
    }

    #[test]
    fn allowlist_blocks_unsafe_git_subcommand() {
        let executor = ToolExecutor::new("allowlist");
        let result = executor.validate_allowlist_policy("git", &[String::from("push")]);
        assert!(result.is_err());
    }

    #[test]
    fn allowlist_allows_expected_readonly_commands() {
        let executor = ToolExecutor::new("allowlist");
        assert!(executor
            .validate_allowlist_policy("git", &[String::from("status")])
            .is_ok());
        assert!(executor
            .validate_allowlist_policy("docker", &[String::from("ps")])
            .is_ok());
        assert!(executor
            .validate_allowlist_policy("cargo", &[String::from("check")])
            .is_ok());
    }

    #[test]
    fn validate_exec_input_rejects_empty_and_nul() {
        assert!(ToolExecutor::validate_exec_input("", &[]).is_err());
        assert!(ToolExecutor::validate_exec_input("ls\0", &[]).is_err());
        assert!(ToolExecutor::validate_exec_input("ls", &[String::from("a\0b")]).is_err());
    }
}
