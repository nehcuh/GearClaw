//! AppleScript and JavaScript for Automation (JXA) execution

use crate::error::GearClawError;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

const OSASCRIPT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Default)]
pub struct AppleScriptExecutor;

impl AppleScriptExecutor {
    pub fn new() -> Self {
        Self
    }

    /// Execute AppleScript code
    pub async fn execute(&self, script: &str) -> Result<String, GearClawError> {
        let fut = Command::new("osascript").arg("-e").arg(script).output();
        let output = timeout(OSASCRIPT_TIMEOUT, fut)
            .await
            .map_err(|_| {
                GearClawError::ToolExecutionError("ERROR:TIMEOUT: AppleScript 执行超时".to_string())
            })?
            .map_err(|e| {
                GearClawError::ToolExecutionError(format!("执行 AppleScript 失败: {}", e))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            if stderr.contains("not authorized") || stderr.contains("Not authorized") {
                return Err(GearClawError::ToolExecutionError(
                    "ERROR:PERMISSION_DENIED: AppleScript 未获授权，请检查辅助功能权限".to_string(),
                ));
            }
            return Err(GearClawError::ToolExecutionError(format!(
                "ERROR:SCRIPT_ERROR: {}",
                stderr.trim()
            )));
        }

        Ok(stdout)
    }

    /// Execute JavaScript for Automation (JXA) code
    pub async fn execute_jxa(&self, script: &str) -> Result<String, GearClawError> {
        let fut = Command::new("osascript")
            .arg("-l")
            .arg("JavaScript")
            .arg("-e")
            .arg(script)
            .output();
        let output = timeout(OSASCRIPT_TIMEOUT, fut)
            .await
            .map_err(|_| {
                GearClawError::ToolExecutionError("ERROR:TIMEOUT: JXA 执行超时".to_string())
            })?
            .map_err(|e| GearClawError::ToolExecutionError(format!("执行 JXA 失败: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            if stderr.contains("not authorized") || stderr.contains("Not authorized") {
                return Err(GearClawError::ToolExecutionError(
                    "ERROR:PERMISSION_DENIED: JXA 未获授权，请检查辅助功能权限".to_string(),
                ));
            }
            return Err(GearClawError::ToolExecutionError(format!(
                "ERROR:SCRIPT_ERROR: {}",
                stderr.trim()
            )));
        }

        Ok(stdout)
    }

    /// Execute AppleScript from file
    pub async fn execute_file(&self, path: &str) -> Result<String, GearClawError> {
        let fut = Command::new("osascript").arg(path).output();
        let output = timeout(OSASCRIPT_TIMEOUT, fut)
            .await
            .map_err(|_| {
                GearClawError::ToolExecutionError(
                    "ERROR:TIMEOUT: AppleScript 文件执行超时".to_string(),
                )
            })?
            .map_err(|e| {
                GearClawError::ToolExecutionError(format!("执行 AppleScript 文件失败: {}", e))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(GearClawError::ToolExecutionError(format!(
                "ERROR:SCRIPT_ERROR: {}",
                stderr.trim()
            )));
        }

        Ok(stdout)
    }
}
