//! macOS notification center integration

use crate::error::GearClawError;
use std::process::Command;

pub struct NotificationSender;

impl NotificationSender {
    pub fn new() -> Self {
        Self
    }

    /// Send a macOS notification
    pub async fn send(&self, title: &str, message: &str, sound: bool) -> Result<String, GearClawError> {
        // Use osascript to send notification
        let sound_option = if sound { "sound name \"Glass\"" } else { "" };

        let script = format!(
            "display notification \"{}\" with title \"{}\" {}",
            message.replace('"', "\\\""),
            title.replace('"', "\\\""),
            sound_option
        );

        let output = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| GearClawError::ToolExecutionError(
                format!("发送通知失败: {}", e)
            ))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GearClawError::ToolExecutionError(
                format!("发送通知失败: {}", stderr)
            ));
        }

        Ok(format!("✓ 已发送通知: {}", message))
    }
}
