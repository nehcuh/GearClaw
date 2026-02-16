//! System-level operations

use crate::error::GearClawError;
use std::process::Command;

pub struct SystemControl;

impl SystemControl {
    pub fn new() -> Self {
        Self
    }

    /// Open URL in default browser
    pub async fn open_url(&self, url: &str) -> Result<String, GearClawError> {
        let output = Command::new("open")
            .arg(url)
            .output()
            .map_err(|e| GearClawError::ToolExecutionError(
                format!("打开 URL 失败: {}", e)
            ))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GearClawError::ToolExecutionError(
                format!("打开 URL {} 失败: {}", url, stderr)
            ));
        }

        Ok(format!("✓ 已在浏览器中打开: {}", url))
    }

    /// Search in browser
    pub async fn search_in_browser(&self, query: &str) -> Result<String, GearClawError> {
        // Encode the query and create a search URL
        let encoded_query = urlencoding::encode(query);
        let search_url = format!("https://www.google.com/search?q={}", encoded_query);

        let output = Command::new("open")
            .arg(&search_url)
            .output()
            .map_err(|e| GearClawError::ToolExecutionError(
                format!("浏览器搜索失败: {}", e)
            ))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GearClawError::ToolExecutionError(
                format!("浏览器搜索失败: {}", stderr)
            ));
        }

        Ok(format!("✓ 已在浏览器中搜索: {}", query))
    }

    /// Text-to-speech using macOS say command
    pub async fn say(&self, text: &str, voice: Option<&str>, rate: u64) -> Result<String, GearClawError> {
        let mut cmd = Command::new("say");

        if let Some(v) = voice {
            cmd.arg("-v").arg(v);
        }

        cmd.arg("-r").arg(rate.to_string());
        cmd.arg(text);

        let output = cmd.output()
            .map_err(|e| GearClawError::ToolExecutionError(
                format!("语音合成失败: {}", e)
            ))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GearClawError::ToolExecutionError(
                format!("语音合成失败: {}", stderr)
            ));
        }

        Ok(format!("✓ 已朗读: {}", text))
    }

    /// Get system volume level
    pub async fn get_volume(&self) -> Result<String, GearClawError> {
        let script = "output volume of (get volume settings)";

        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| GearClawError::ToolExecutionError(
                format!("获取音量失败: {}", e)
            ))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GearClawError::ToolExecutionError(
                format!("获取音量失败: {}", stderr)
            ));
        }

        let volume = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(format!("当前音量: {}%", volume))
    }

    /// Set system volume level (0-100)
    pub async fn set_volume(&self, level: u8) -> Result<String, GearClawError> {
        if level > 100 {
            return Err(GearClawError::ToolExecutionError(
                "音量必须在 0-100 之间".to_string()
            ));
        }

        let script = format!("set volume output volume {}", level);

        let output = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| GearClawError::ToolExecutionError(
                format!("设置音量失败: {}", e)
            ))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GearClawError::ToolExecutionError(
                format!("设置音量失败: {}", stderr)
            ));
        }

        Ok(format!("✓ 已设置音量: {}%", level))
    }
}
