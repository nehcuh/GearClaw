//! System-level operations

use crate::error::GearClawError;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

const SYSTEM_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Default)]
pub struct SystemControl;

impl SystemControl {
    pub fn new() -> Self {
        Self
    }

    /// Open URL in default browser
    pub async fn open_url(&self, url: &str) -> Result<String, GearClawError> {
        let fut = Command::new("open").arg(url).output();
        let output = timeout(SYSTEM_TIMEOUT, fut)
            .await
            .map_err(|_| {
                GearClawError::ToolExecutionError("ERROR:TIMEOUT: 打开 URL 超时".to_string())
            })?
            .map_err(|e| GearClawError::ToolExecutionError(format!("打开 URL 失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GearClawError::ToolExecutionError(format!(
                "打开 URL {} 失败: {}",
                url, stderr
            )));
        }

        Ok(format!("✓ 已在浏览器中打开: {}", url))
    }

    /// Search in browser
    pub async fn search_in_browser(&self, query: &str) -> Result<String, GearClawError> {
        let encoded_query = urlencoding::encode(query);
        let search_url = format!("https://www.google.com/search?q={}", encoded_query);

        let fut = Command::new("open").arg(&search_url).output();
        let output = timeout(SYSTEM_TIMEOUT, fut)
            .await
            .map_err(|_| {
                GearClawError::ToolExecutionError("ERROR:TIMEOUT: 浏览器搜索超时".to_string())
            })?
            .map_err(|e| GearClawError::ToolExecutionError(format!("浏览器搜索失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GearClawError::ToolExecutionError(format!(
                "浏览器搜索失败: {}",
                stderr
            )));
        }

        Ok(format!("✓ 已在浏览器中搜索: {}", query))
    }

    /// Text-to-speech using macOS say command.
    ///
    /// Timeout is estimated dynamically:
    /// `min(300, max(60, char_count * 60 / rate + 10))` seconds,
    /// where rate is words-per-minute (default 175).
    /// This allows up to 5 minutes for very long texts while keeping a
    /// reasonable floor of 60 seconds.
    pub async fn say(
        &self,
        text: &str,
        voice: Option<&str>,
        rate: u64,
    ) -> Result<String, GearClawError> {
        let effective_rate = rate.max(1);
        let char_count = text.chars().count() as u64;
        // Approximate: chars ≈ words * 5; time_secs = chars / (rate * 5 / 60)
        let estimated_secs = char_count * 60 / (effective_rate * 5).max(1);
        let say_timeout = Duration::from_secs(estimated_secs.clamp(60, 300) + 10);

        let mut cmd = Command::new("say");
        if let Some(v) = voice {
            cmd.arg("-v").arg(v);
        }
        cmd.arg("-r").arg(effective_rate.to_string());
        cmd.arg(text);

        let fut = cmd.output();
        let output = timeout(say_timeout, fut)
            .await
            .map_err(|_| {
                GearClawError::ToolExecutionError("ERROR:TIMEOUT: 语音合成超时".to_string())
            })?
            .map_err(|e| GearClawError::ToolExecutionError(format!("语音合成失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GearClawError::ToolExecutionError(format!(
                "语音合成失败: {}",
                stderr
            )));
        }

        Ok(format!("✓ 已朗读: {}", text))
    }

    /// Get system volume level
    pub async fn get_volume(&self) -> Result<String, GearClawError> {
        let script = "output volume of (get volume settings)";
        let fut = Command::new("osascript").arg("-e").arg(script).output();
        let output = timeout(SYSTEM_TIMEOUT, fut)
            .await
            .map_err(|_| {
                GearClawError::ToolExecutionError("ERROR:TIMEOUT: 获取音量超时".to_string())
            })?
            .map_err(|e| GearClawError::ToolExecutionError(format!("获取音量失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GearClawError::ToolExecutionError(format!(
                "获取音量失败: {}",
                stderr
            )));
        }

        let volume = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(format!("当前音量: {}%", volume))
    }

    /// Set system volume level (0-100)
    pub async fn set_volume(&self, level: u8) -> Result<String, GearClawError> {
        if level > 100 {
            return Err(GearClawError::ToolExecutionError(
                "音量必须在 0-100 之间".to_string(),
            ));
        }

        let script = format!("set volume output volume {}", level);
        let fut = Command::new("osascript").arg("-e").arg(&script).output();
        let output = timeout(SYSTEM_TIMEOUT, fut)
            .await
            .map_err(|_| {
                GearClawError::ToolExecutionError("ERROR:TIMEOUT: 设置音量超时".to_string())
            })?
            .map_err(|e| GearClawError::ToolExecutionError(format!("设置音量失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GearClawError::ToolExecutionError(format!(
                "设置音量失败: {}",
                stderr
            )));
        }

        Ok(format!("✓ 已设置音量: {}%", level))
    }
}
