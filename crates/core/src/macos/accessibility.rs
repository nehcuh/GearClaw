//! Accessibility permission detection for macOS automation tools.

use crate::error::GearClawError;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

const OSASCRIPT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Default)]
pub struct AccessibilityChecker;

impl AccessibilityChecker {
    pub fn new() -> Self {
        Self
    }

    /// Check whether the current process has Accessibility permission.
    ///
    /// Probes by asking System Events for the name of the first process.
    /// This call requires Accessibility; if denied, osascript returns a
    /// recognisable "not authorized" error.
    ///
    /// Returns:
    /// - `"OK"` — permission is granted.
    /// - `"DENIED: ..."` — permission is not granted, with guidance.
    pub async fn check_accessibility(&self) -> Result<String, GearClawError> {
        let script = "tell application \"System Events\" to get name of first process";
        let fut = Command::new("osascript").arg("-e").arg(script).output();
        let output = timeout(OSASCRIPT_TIMEOUT, fut)
            .await
            .map_err(|_| {
                GearClawError::ToolExecutionError("ERROR:TIMEOUT: 权限检测超时".to_string())
            })?
            .map_err(|e| GearClawError::ToolExecutionError(format!("权限检测失败: {}", e)))?;

        if output.status.success() {
            return Ok("OK".to_string());
        }

        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if stderr.contains("not authorized") || stderr.contains("Not authorized") {
            Ok("DENIED: 辅助功能权限未授权。\
                 请前往 系统设置 → 隐私与安全性 → 辅助功能，\
                 为终端（Terminal）或本应用授权后重试。"
                .to_string())
        } else {
            // Other error — treat as permission denied to be safe
            Ok(format!(
                "DENIED: 无法验证权限（{}），\
                 请确认辅助功能已授权。",
                stderr.trim()
            ))
        }
    }
}
