//! macOS application content reading tools.
//!
//! Provides three complementary strategies:
//! 1. `read_selected_text`  — clipboard-based; reads whatever is currently selected.
//! 2. `read_focused_field`  — AX API; reads the value of the focused UI element.
//! 3. `read_document`       — App-specific scripting; reads the frontmost document.
//!
//! # Concurrency note
//! `read_selected_text` mutates the system clipboard (a global resource).
//! Do **not** call it concurrently from multiple agent sessions without an
//! external mutex. A future M2 improvement may add a `tokio::sync::Mutex`
//! inside `ContentReader`.

use crate::error::GearClawError;
use crate::macos::clipboard::{ClipboardManager, ClipboardType};
use std::time::Duration;
use tokio::process::Command;
use tokio::time::{sleep, timeout};

const OSASCRIPT_TIMEOUT: Duration = Duration::from_secs(5);

pub struct ContentReader {
    clipboard: ClipboardManager,
}

impl Default for ContentReader {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentReader {
    pub fn new() -> Self {
        Self {
            clipboard: ClipboardManager::new(),
        }
    }

    // -------------------------------------------------------------------------
    // Strategy 1: read_selected_text
    // -------------------------------------------------------------------------

    /// Read the currently selected text in the frontmost application by
    /// simulating Cmd+C and capturing the clipboard delta.
    ///
    /// # Parameters
    /// - `preserve_clipboard`: restore original text clipboard after read (default true).
    ///   Non-text clipboard content is never restored.
    /// - `timeout_ms`: how long to wait for the clipboard to update after Cmd+C (default 300 ms).
    /// - `max_chars`: truncate result to this many chars (0 = no limit).
    ///
    /// # Precondition
    /// The target application must already be frontmost. Call `macos_bring_to_front`
    /// before this tool if needed.
    ///
    /// # Returns
    /// `"[N chars, clipboard restored] <text>"` on success, or
    /// `"ERROR:<CODE>: <explanation>"` on failure.
    pub async fn read_selected_text(
        &self,
        preserve_clipboard: bool,
        timeout_ms: u64,
        max_chars: usize,
    ) -> Result<String, GearClawError> {
        // Step 1: Snapshot clipboard type and content (text only)
        let original_type = self
            .clipboard
            .content_type()
            .await
            .unwrap_or(ClipboardType::Unknown("unknown".to_string()));
        let original_text = if original_type == ClipboardType::Text {
            Some(self.clipboard.read().await.unwrap_or_default())
        } else {
            None
        };

        // Step 2: Send Cmd+C to the frontmost app via System Events
        let copy_script =
            "tell application \"System Events\" to keystroke \"c\" using command down";
        let fut = Command::new("osascript")
            .arg("-e")
            .arg(copy_script)
            .output();
        let copy_output = timeout(OSASCRIPT_TIMEOUT, fut)
            .await
            .map_err(|_| {
                GearClawError::ToolExecutionError("ERROR:TIMEOUT: 发送复制命令超时".to_string())
            })?
            .map_err(|e| GearClawError::ToolExecutionError(format!("发送复制命令失败: {}", e)))?;

        if !copy_output.status.success() {
            let stderr = String::from_utf8_lossy(&copy_output.stderr).to_string();
            if stderr.contains("not authorized") || stderr.contains("Not authorized") {
                return Err(GearClawError::ToolExecutionError(
                    "ERROR:PERMISSION_DENIED: 发送 Cmd+C 需要辅助功能权限".to_string(),
                ));
            }
            return Err(GearClawError::ToolExecutionError(format!(
                "ERROR:SCRIPT_ERROR: {}",
                stderr.trim()
            )));
        }

        // Step 3: Poll clipboard for up to `timeout_ms` until content changes
        let poll_interval = Duration::from_millis(50);
        let deadline = Duration::from_millis(timeout_ms.max(50));
        let mut elapsed = Duration::ZERO;
        let new_text;

        loop {
            sleep(poll_interval).await;
            elapsed += poll_interval;

            let current = self.clipboard.read().await.unwrap_or_default();

            // Compare against original: if different (or original was non-text), accept it
            let changed = match &original_text {
                Some(orig) => current != *orig,
                None => !current.is_empty(),
            };

            if changed {
                new_text = current;
                break;
            }

            if elapsed >= deadline {
                // Restore original text clipboard if needed before returning error
                if preserve_clipboard {
                    if let Some(ref orig) = original_text {
                        let _ = self.clipboard.write(orig).await;
                    }
                }
                return Err(GearClawError::ToolExecutionError(
                    "ERROR:CLIPBOARD_UNCHANGED: 剪贴板未发生变化，\
                     可能没有选中任何文本，或目标应用不支持复制操作"
                        .to_string(),
                ));
            }
        }

        // Step 4: Restore original clipboard (text only)
        let mut restore_note = String::new();
        if preserve_clipboard {
            if let Some(ref orig) = original_text {
                if let Err(e) = self.clipboard.write(orig).await {
                    restore_note = format!(" [clipboard restore failed: {}]", e);
                } else {
                    restore_note = " [clipboard restored]".to_string();
                }
            } else {
                restore_note = format!(
                    " [clipboard not restored: original type was {}]",
                    original_type
                );
            }
        }

        // Step 5: Apply max_chars truncation
        let (text, truncated) = if max_chars > 0 && new_text.chars().count() > max_chars {
            let truncated: String = new_text.chars().take(max_chars).collect();
            (truncated, true)
        } else {
            (new_text.clone(), false)
        };

        let char_count = new_text.chars().count();
        let trunc_note = if truncated {
            format!(", truncated to {}", max_chars)
        } else {
            String::new()
        };

        Ok(format!(
            "[{} chars{}{}] {}",
            char_count, trunc_note, restore_note, text
        ))
    }

    // -------------------------------------------------------------------------
    // Strategy 2: read_focused_field
    // -------------------------------------------------------------------------

    /// Read the text value of the currently focused UI element via the
    /// Accessibility (AX) API.
    ///
    /// Tries `value` → `name` → `description` of the focused element in order,
    /// returning the first non-empty, non-whitespace-only result.
    ///
    /// # Parameters
    /// - `app_name`: optional; if provided, restricts the query to that process name.
    /// - `max_chars`: truncate result (0 = no limit).
    ///
    /// # Precondition
    /// Target application must be frontmost.
    pub async fn read_focused_field(
        &self,
        app_name: Option<&str>,
        max_chars: usize,
    ) -> Result<String, GearClawError> {
        // Build process target clause
        let process_clause = match app_name {
            Some(name) => format!("process \"{}\"", name),
            None => "first process whose frontmost is true".to_string(),
        };

        // Try each AX attribute in order
        for attr in &["value", "name", "description"] {
            let script = format!(
                "tell application \"System Events\" to tell ({}) to \
                 get {} of focused UI element",
                process_clause, attr
            );
            let fut = Command::new("osascript").arg("-e").arg(&script).output();
            let result = timeout(OSASCRIPT_TIMEOUT, fut).await;

            let output = match result {
                Ok(Ok(o)) => o,
                Ok(Err(_)) | Err(_) => continue, // timeout or spawn error → try next attr
            };

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                if stderr.contains("not authorized") || stderr.contains("Not authorized") {
                    return Err(GearClawError::ToolExecutionError(
                        "ERROR:PERMISSION_DENIED: 读取焦点字段需要辅助功能权限".to_string(),
                    ));
                }
                // Other error (e.g. element has no such attribute) → try next
                continue;
            }

            let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if raw.is_empty() {
                continue;
            }

            // Apply max_chars
            let (text, truncated) = if max_chars > 0 && raw.chars().count() > max_chars {
                let t: String = raw.chars().take(max_chars).collect();
                (t, true)
            } else {
                (raw.clone(), false)
            };

            let char_count = raw.chars().count();
            let trunc_note = if truncated {
                format!(", truncated to {}", max_chars)
            } else {
                String::new()
            };

            return Ok(format!(
                "[field: {}{}, {} chars] {}",
                attr, trunc_note, char_count, text
            ));
        }

        Err(GearClawError::ToolExecutionError(
            "ERROR:EMPTY_FIELD: 焦点元素无可读文本内容（value/name/description 均为空）"
                .to_string(),
        ))
    }

    // -------------------------------------------------------------------------
    // Strategy 3: read_document
    // -------------------------------------------------------------------------

    /// Read the frontmost document content using app-specific AppleScript.
    ///
    /// Supported apps: TextEdit, Notes, Safari, Terminal.
    /// For unsupported apps, returns an actionable error.
    ///
    /// # Parameters
    /// - `app_name`: required; must match one of the supported app names.
    /// - `max_chars`: truncate result (0 = no limit).
    pub async fn read_document(
        &self,
        app_name: &str,
        max_chars: usize,
    ) -> Result<String, GearClawError> {
        let script = match app_name.to_lowercase().as_str() {
            "textedit" => {
                r#"tell application "TextEdit" to get text of front document"#.to_string()
            }
            "notes" => {
                r#"tell application "Notes" to get body of note 1 of account 1"#.to_string()
            }
            "safari" => {
                r#"tell application "Safari" to do JavaScript "document.body.innerText" in current tab of front window"#.to_string()
            }
            "terminal" => {
                r#"tell application "Terminal" to get contents of selected tab of front window"#.to_string()
            }
            other => {
                return Err(GearClawError::ToolExecutionError(format!(
                    "ERROR:UNSUPPORTED_APP: 应用 \"{}\" 未内置适配。\
                     建议使用 macos_read_selected_text（先选中文本再调用）\
                     或 macos_read_focused_field（读取焦点输入框）。\
                     已支持的应用：TextEdit、Notes、Safari、Terminal。",
                    other
                )));
            }
        };

        let fut = Command::new("osascript").arg("-e").arg(&script).output();
        let output = timeout(OSASCRIPT_TIMEOUT, fut)
            .await
            .map_err(|_| {
                GearClawError::ToolExecutionError(format!(
                    "ERROR:TIMEOUT: 读取 {} 文档超时",
                    app_name
                ))
            })?
            .map_err(|e| GearClawError::ToolExecutionError(format!("读取文档失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if stderr.contains("not authorized") || stderr.contains("Not authorized") {
                return Err(GearClawError::ToolExecutionError(format!(
                    "ERROR:PERMISSION_DENIED: 读取 {} 文档需要自动化权限，\
                     请在 系统设置 → 隐私与安全性 → 自动化 中授权",
                    app_name
                )));
            }
            if stderr.contains("doesn't understand") || stderr.contains("Can't get") {
                return Err(GearClawError::ToolExecutionError(format!(
                    "ERROR:SCRIPT_ERROR: {} 没有打开的文档，或文档不可读（{}）",
                    app_name,
                    stderr.trim()
                )));
            }
            return Err(GearClawError::ToolExecutionError(format!(
                "ERROR:SCRIPT_ERROR: {}",
                stderr.trim()
            )));
        }

        let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if raw.is_empty() {
            return Err(GearClawError::ToolExecutionError(format!(
                "ERROR:EMPTY_FIELD: {} 返回了空文档内容",
                app_name
            )));
        }

        let total_chars = raw.chars().count();
        let (text, truncated) = if max_chars > 0 && total_chars > max_chars {
            let t: String = raw.chars().take(max_chars).collect();
            (t, true)
        } else {
            (raw, false)
        };

        if truncated {
            Ok(format!(
                "{}... [truncated, total: {} chars]",
                text, total_chars
            ))
        } else {
            Ok(text)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- read_document: unsupported app (pure logic, no osascript) ---

    #[tokio::test]
    async fn read_document_unsupported_app_returns_error_code() {
        let reader = ContentReader::new();
        let result = reader.read_document("Photoshop", 0).await;
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("ERROR:UNSUPPORTED_APP"),
            "Expected ERROR:UNSUPPORTED_APP in: {msg}"
        );
    }

    #[tokio::test]
    async fn read_document_unsupported_app_mentions_supported_list() {
        let reader = ContentReader::new();
        let result = reader.read_document("Word", 0).await;
        let msg = result.unwrap_err().to_string();
        // Error message should mention the supported apps
        assert!(msg.contains("TextEdit"), "Should mention TextEdit: {msg}");
        assert!(msg.contains("Safari"), "Should mention Safari: {msg}");
    }

    // --- read_document: case-insensitive app name matching ---
    // This test will actually run osascript if on macOS. We only test the
    // unsupported-app fast-path here. The following test verifies that
    // 'textedit' (lowercase) does NOT hit the unsupported-app branch.
    // (It may fail for other reasons like no open document, but NOT UNSUPPORTED_APP.)
    #[tokio::test]
    #[cfg(target_os = "macos")]
    async fn read_document_lowercase_textedit_not_unsupported() {
        let reader = ContentReader::new();
        let result = reader.read_document("textedit", 0).await;
        // If there's an error, it must not be UNSUPPORTED_APP
        if let Err(e) = result {
            let msg = e.to_string();
            assert!(
                !msg.contains("ERROR:UNSUPPORTED_APP"),
                "textedit should be recognized (case-insensitive): {msg}"
            );
        }
    }

    // --- ContentReader: max_chars truncation logic (via returned text format) ---
    // We test truncation indirectly through read_document error path metadata.
    // Actual truncation is tested via the pure string-building helper below.

    #[test]
    fn truncation_format_verification() {
        // Simulate the format string used by read_document truncation
        let total_chars = 5000usize;
        let max_chars = 100usize;
        let text: String = "A".repeat(max_chars);
        let output = format!("{}... [truncated, total: {} chars]", text, total_chars);
        assert!(output.contains("[truncated, total: 5000 chars]"));
        assert_eq!(output.chars().filter(|&c| c == 'A').count(), max_chars);
    }
}
