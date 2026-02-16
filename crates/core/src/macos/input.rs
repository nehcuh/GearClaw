//! Input simulation (keyboard and mouse)

use crate::error::GearClawError;
use std::process::Command;

pub struct InputSimulator;

impl InputSimulator {
    pub fn new() -> Self {
        Self
    }

    /// Type text using keyboard simulation
    pub async fn type_text(&self, text: &str) -> Result<String, GearClawError> {
        // Use AppleScript for text typing
        let script = format!(
            "tell application \"System Events\" to keystroke \"{}\"",
            text.replace('"', "\\\"")
        );

        let output = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| GearClawError::ToolExecutionError(
                format!("键盘输入失败: {}", e)
            ))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GearClawError::ToolExecutionError(
                format!("键盘输入失败: {}", stderr)
            ));
        }

        Ok(format!("✓ 已输入: {}", text))
    }

    /// Simulate key combination (e.g., cmd+c, cmd+v)
    pub async fn key_combo(&self, keys: &[&str]) -> Result<String, GearClawError> {
        let key_code = self.get_key_code(keys)?;

        // Collect modifiers
        let has_cmd = keys.iter().any(|k| *k == "cmd" || *k == "command" || *k == "⌘");
        let has_shift = keys.iter().any(|k| *k == "shift");
        let has_option = keys.iter().any(|k| *k == "option" || *k == "alt");
        let has_control = keys.iter().any(|k| *k == "control" || *k == "ctrl");

        let mut modifiers = Vec::new();
        if has_cmd { modifiers.push("command down"); }
        if has_shift { modifiers.push("shift down"); }
        if has_option { modifiers.push("option down"); }
        if has_control { modifiers.push("control down"); }

        let using_clause = if modifiers.is_empty() {
            String::from("")
        } else {
            format!(" using {}", modifiers.join(" & "))
        };

        let script = format!(
            "tell application \"System Events\" to key code {}{}",
            key_code, using_clause
        );

        let output = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| GearClawError::ToolExecutionError(
                format!("按键组合失败: {}", e)
            ))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GearClawError::ToolExecutionError(
                format!("按键组合失败: {}", stderr)
            ));
        }

        Ok(format!("✓ 已按键组合: {}", keys.join("+")))
    }

    /// Get key code for a key name
    fn get_key_code(&self, keys: &[&str]) -> Result<String, GearClawError> {
        // Filter out modifiers
        let key = keys.iter()
            .find(|k| {
                let k_lower = k.to_lowercase();
                !["cmd", "command", "⌘", "shift", "option", "alt", "control", "ctrl"].contains(&k_lower.as_str())
            })
            .ok_or_else(|| GearClawError::ToolExecutionError(
                "未指定实际按键".to_string()
            ))?;

        // Map common keys to key codes
        let key_code = match key.to_lowercase().as_str() {
            "c" => "8",      // C
            "v" => "9",      // V
            "x" => "7",      // X
            "a" => "0",      // A
            "z" => "6",      // Z
            "s" => "1",      // S
            "w" => "13",     // W
            "q" => "12",     // Q
            "tab" => "48",   // Tab
            "return" | "enter" => "36",  // Return
            "space" => "49", // Space
            "escape" | "esc" => "53",    // Escape
            "delete" | "backspace" => "51",  // Delete
            "up" => "126",   // Up arrow
            "down" => "125", // Down arrow
            "left" => "123", // Left arrow
            "right" => "124", // Right arrow
            _ => {
                return Err(GearClawError::ToolExecutionError(
                    format!("不支持的按键: {}", key)
                ))
            }
        };

        Ok(key_code.to_string())
    }

    /// Click at current mouse position
    pub async fn click(&self) -> Result<String, GearClawError> {
        let script = "tell application \"System Events\" to click at {100, 100}";

        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| GearClawError::ToolExecutionError(
                format!("鼠标点击失败: {}", e)
            ))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GearClawError::ToolExecutionError(
                format!("鼠标点击失败: {}", stderr)
            ));
        }

        Ok("✓ 已点击".to_string())
    }
}
