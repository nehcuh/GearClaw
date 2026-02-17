//! Clipboard operations

use crate::error::GearClawError;
use std::process::{Command, Stdio};

pub struct ClipboardManager;

impl ClipboardManager {
    pub fn new() -> Self {
        Self
    }

    /// Read clipboard content
    pub async fn read(&self) -> Result<String, GearClawError> {
        let output = Command::new("pbpaste")
            .output()
            .map_err(|e| GearClawError::ToolExecutionError(format!("读取剪贴板失败: {}", e)))?;

        let content = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(content)
    }

    /// Write text to clipboard
    pub async fn write(&self, text: &str) -> Result<String, GearClawError> {
        Command::new("pbcopy")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    stdin.write_all(text.as_bytes())?;
                }
                child.wait_with_output()
            })
            .map_err(|e| GearClawError::ToolExecutionError(format!("写入剪贴板失败: {}", e)))?;

        Ok("✓ 已复制到剪贴板".to_string())
    }

    /// Clear clipboard
    pub async fn clear(&self) -> Result<String, GearClawError> {
        Command::new("pbcopy")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    stdin.write_all(b"")?;
                }
                child.wait_with_output()
            })
            .map_err(|e| GearClawError::ToolExecutionError(format!("清空剪贴板失败: {}", e)))?;

        Ok("✓ 剪贴板已清空".to_string())
    }
}
