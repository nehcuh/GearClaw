//! Query the currently frontmost (active) macOS application.

use crate::error::GearClawError;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

const OSASCRIPT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Default)]
pub struct FrontmostAppReader;

impl FrontmostAppReader {
    pub fn new() -> Self {
        Self
    }

    /// Return the name of the application that currently has focus.
    ///
    /// Uses `System Events` which requires Accessibility permission.
    /// Returns: `"Frontmost app: TextEdit"` or `"ERROR:<CODE>: ..."`.
    pub async fn get_frontmost_app(&self) -> Result<String, GearClawError> {
        let script =
            "tell application \"System Events\" to get name of first process whose frontmost is true";
        let fut = Command::new("osascript").arg("-e").arg(script).output();
        let output = timeout(OSASCRIPT_TIMEOUT, fut)
            .await
            .map_err(|_| {
                GearClawError::ToolExecutionError("ERROR:TIMEOUT: 查询前台应用超时".to_string())
            })?
            .map_err(|e| GearClawError::ToolExecutionError(format!("查询前台应用失败: {}", e)))?;

        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            if stderr.contains("not authorized") || stderr.contains("Not authorized") {
                tracing::warn!(
                    tool = "macos_get_frontmost_app",
                    error = "PERMISSION_DENIED",
                    "辅助功能权限未授权"
                );
                return Err(GearClawError::ToolExecutionError(
                    "ERROR:PERMISSION_DENIED: 查询前台应用需要辅助功能权限，\
                     请前往 系统设置 → 隐私与安全性 → 辅助功能 授权"
                        .to_string(),
                ));
            }
            return Err(GearClawError::ToolExecutionError(format!(
                "ERROR:SCRIPT_ERROR: {}",
                stderr.trim()
            )));
        }

        let app_name = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if app_name.is_empty() {
            return Err(GearClawError::ToolExecutionError(
                "ERROR:NOT_FOCUSED: 无法确定当前前台应用".to_string(),
            ));
        }

        tracing::debug!(tool = "macos_get_frontmost_app", app = %app_name, "前台应用查询成功");
        Ok(format!("Frontmost app: {}", app_name))
    }
}
