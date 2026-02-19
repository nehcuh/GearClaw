//! Clipboard operations

use crate::error::GearClawError;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::timeout;

const CLIPBOARD_TIMEOUT: Duration = Duration::from_secs(5);

/// Type of content currently in the system clipboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardType {
    /// Plain text (UTF-8 / ASCII)
    Text,
    /// Image data (TIFF, PNG, JPEG, etc.)
    Image,
    /// File references (e.g. Finder copy)
    File,
    /// Clipboard is empty
    Empty,
    /// Unknown or unsupported type
    Unknown(String),
}

impl std::fmt::Display for ClipboardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipboardType::Text => write!(f, "Text"),
            ClipboardType::Image => write!(f, "Image"),
            ClipboardType::File => write!(f, "File"),
            ClipboardType::Empty => write!(f, "Empty"),
            ClipboardType::Unknown(s) => write!(f, "Unknown({})", s),
        }
    }
}

#[derive(Default)]
pub struct ClipboardManager;

impl ClipboardManager {
    pub fn new() -> Self {
        Self
    }

    /// Detect the type of content currently in the clipboard.
    ///
    /// Uses `osascript -e 'clipboard info'` which returns lines like:
    /// `«class utf8», 42` or `«class PNGf», 1234` etc.
    pub async fn content_type(&self) -> Result<ClipboardType, GearClawError> {
        let fut = Command::new("osascript")
            .arg("-e")
            .arg("clipboard info")
            .output();
        let output = timeout(CLIPBOARD_TIMEOUT, fut)
            .await
            .map_err(|_| {
                GearClawError::ToolExecutionError("ERROR:TIMEOUT: 检测剪贴板类型超时".to_string())
            })?
            .map_err(|e| GearClawError::ToolExecutionError(format!("检测剪贴板类型失败: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let info = stdout.trim();

        if info.is_empty() || info == "{}" {
            return Ok(ClipboardType::Empty);
        }

        // Check for known text types first
        if info.contains("class utf8")
            || info.contains("class ktxt")
            || info.contains("class TEXT")
            || info.contains("string")
        {
            return Ok(ClipboardType::Text);
        }
        // Image types
        if info.contains("class PNGf")
            || info.contains("class TIFF")
            || info.contains("class JPEG")
            || info.contains("class GIFf")
            || info.contains("class pict")
        {
            return Ok(ClipboardType::Image);
        }
        // File reference types
        if info.contains("class furl") || info.contains("class hfs ") {
            return Ok(ClipboardType::File);
        }

        Ok(ClipboardType::Unknown(info.chars().take(80).collect()))
    }

    /// Read plain-text clipboard content via `pbpaste`.
    pub async fn read(&self) -> Result<String, GearClawError> {
        let fut = Command::new("pbpaste").output();
        let output = timeout(CLIPBOARD_TIMEOUT, fut)
            .await
            .map_err(|_| {
                GearClawError::ToolExecutionError("ERROR:TIMEOUT: 读取剪贴板超时".to_string())
            })?
            .map_err(|e| GearClawError::ToolExecutionError(format!("读取剪贴板失败: {}", e)))?;

        let content = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(content)
    }

    /// Write text to clipboard via `pbcopy`.
    ///
    /// Uses `tokio::process::Command` with stdin correctly closed to avoid deadlock.
    pub async fn write(&self, text: &str) -> Result<String, GearClawError> {
        let mut child = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| GearClawError::ToolExecutionError(format!("写入剪贴板失败: {}", e)))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes()).await.map_err(|e| {
                GearClawError::ToolExecutionError(format!("写入剪贴板 stdin 失败: {}", e))
            })?;
            // Explicitly drop/close stdin so pbcopy reads EOF and exits
        }

        let status = timeout(CLIPBOARD_TIMEOUT, child.wait())
            .await
            .map_err(|_| {
                GearClawError::ToolExecutionError("ERROR:TIMEOUT: 写入剪贴板超时".to_string())
            })?
            .map_err(|e| {
                GearClawError::ToolExecutionError(format!("写入剪贴板进程等待失败: {}", e))
            })?;

        if !status.success() {
            return Err(GearClawError::ToolExecutionError(
                "写入剪贴板失败: pbcopy 返回非零退出码".to_string(),
            ));
        }

        Ok("✓ 已复制到剪贴板".to_string())
    }

    /// Clear clipboard by writing an empty string.
    pub async fn clear(&self) -> Result<String, GearClawError> {
        self.write("").await?;
        Ok("✓ 剪贴板已清空".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- ClipboardType: Display ---

    #[test]
    fn clipboard_type_display_text() {
        assert_eq!(ClipboardType::Text.to_string(), "Text");
    }

    #[test]
    fn clipboard_type_display_image() {
        assert_eq!(ClipboardType::Image.to_string(), "Image");
    }

    #[test]
    fn clipboard_type_display_file() {
        assert_eq!(ClipboardType::File.to_string(), "File");
    }

    #[test]
    fn clipboard_type_display_empty() {
        assert_eq!(ClipboardType::Empty.to_string(), "Empty");
    }

    #[test]
    fn clipboard_type_display_unknown() {
        let t = ClipboardType::Unknown("class fooX".to_string());
        assert_eq!(t.to_string(), "Unknown(class fooX)");
    }

    // --- ClipboardType: PartialEq ---

    #[test]
    fn clipboard_type_eq_same_variant() {
        assert_eq!(ClipboardType::Text, ClipboardType::Text);
        assert_eq!(ClipboardType::Empty, ClipboardType::Empty);
    }

    #[test]
    fn clipboard_type_eq_different_variants() {
        assert_ne!(ClipboardType::Text, ClipboardType::Image);
        assert_ne!(ClipboardType::File, ClipboardType::Empty);
    }

    #[test]
    fn clipboard_type_unknown_eq() {
        assert_eq!(
            ClipboardType::Unknown("foo".to_string()),
            ClipboardType::Unknown("foo".to_string())
        );
        assert_ne!(
            ClipboardType::Unknown("foo".to_string()),
            ClipboardType::Unknown("bar".to_string())
        );
    }

    // --- ClipboardType: Clone ---

    #[test]
    fn clipboard_type_clone() {
        let original = ClipboardType::Unknown("test".to_string());
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }
}
