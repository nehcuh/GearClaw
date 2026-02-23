//! macOS notification center integration

use crate::error::GearClawError;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

const NOTIFY_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Default)]
pub struct NotificationSender;

impl NotificationSender {
    pub fn new() -> Self {
        Self
    }

    /// Send a macOS notification
    pub async fn send(
        &self,
        title: &str,
        message: &str,
        sound: bool,
    ) -> Result<String, GearClawError> {
        let sound_option = if sound { "sound name \"Glass\"" } else { "" };

        let script = format!(
            "display notification \"{}\" with title \"{}\" {}",
            message.replace('"', "\\\""),
            title.replace('"', "\\\""),
            sound_option
        );

        let fut = Command::new("osascript").arg("-e").arg(&script).output();
        let output = timeout(NOTIFY_TIMEOUT, fut)
            .await
            .map_err(|_| {
                GearClawError::ToolExecutionError("ERROR:TIMEOUT: 发送通知超时".to_string())
            })?
            .map_err(|e| GearClawError::ToolExecutionError(format!("发送通知失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GearClawError::ToolExecutionError(format!(
                "发送通知失败: {}",
                stderr
            )));
        }

        Ok(format!("✓ 已发送通知: {}", message))
    }
}
