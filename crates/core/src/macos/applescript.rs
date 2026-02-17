//! AppleScript and JavaScript for Automation (JXA) execution

use crate::error::GearClawError;
use std::process::Command;

pub struct AppleScriptExecutor;

impl AppleScriptExecutor {
    pub fn new() -> Self {
        Self
    }

    /// Execute AppleScript code
    pub async fn execute(&self, script: &str) -> Result<String, GearClawError> {
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| {
                GearClawError::ToolExecutionError(format!("执行 AppleScript 失败: {}", e))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(GearClawError::ToolExecutionError(format!(
                "AppleScript 执行失败: {}",
                stderr
            )));
        }

        Ok(stdout)
    }

    /// Execute JavaScript for Automation (JXA) code
    pub async fn execute_jxa(&self, script: &str) -> Result<String, GearClawError> {
        let output = Command::new("osascript")
            .arg("-l")
            .arg("JavaScript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| GearClawError::ToolExecutionError(format!("执行 JXA 失败: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(GearClawError::ToolExecutionError(format!(
                "JXA 执行失败: {}",
                stderr
            )));
        }

        Ok(stdout)
    }

    /// Execute AppleScript from file
    pub async fn execute_file(&self, path: &str) -> Result<String, GearClawError> {
        let output = Command::new("osascript").arg(path).output().map_err(|e| {
            GearClawError::ToolExecutionError(format!("执行 AppleScript 文件失败: {}", e))
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(GearClawError::ToolExecutionError(format!(
                "AppleScript 文件执行失败: {}",
                stderr
            )));
        }

        Ok(stdout)
    }
}
