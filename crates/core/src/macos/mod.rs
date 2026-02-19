//! macOS-specific application control and automation
//!
//! This module provides tools for controlling macOS applications,
//! executing AppleScript, simulating input, and more.

mod app;
mod applescript;
mod clipboard;
mod input;
mod notification;
mod system;

pub use app::AppManager;
pub use applescript::AppleScriptExecutor;
pub use clipboard::ClipboardManager;
pub use input::InputSimulator;
pub use notification::NotificationSender;
pub use system::SystemControl;

use crate::error::GearClawError;
use serde_json::{json, Value};

/// Unified macOS automation controller
pub struct MacosController {
    pub app: AppManager,
    pub script: AppleScriptExecutor,
    pub clipboard: ClipboardManager,
    pub input: InputSimulator,
    pub notification: NotificationSender,
    pub system: SystemControl,
}

impl MacosController {
    pub fn new() -> Result<Self, GearClawError> {
        Ok(Self {
            app: AppManager::new(),
            script: AppleScriptExecutor::new(),
            clipboard: ClipboardManager::new(),
            input: InputSimulator::new(),
            notification: NotificationSender::new(),
            system: SystemControl::new(),
        })
    }

    /// Execute a macOS-specific tool by name
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        args: &Value,
    ) -> Result<String, GearClawError> {
        match tool_name {
            "macos_launch_app" => {
                let app_name = args["app_name"].as_str().ok_or_else(|| {
                    GearClawError::ToolExecutionError("缺少 app_name 参数".to_string())
                })?;
                self.app.launch(app_name).await
            }

            "macos_quit_app" => {
                let app_name = args["app_name"].as_str().ok_or_else(|| {
                    GearClawError::ToolExecutionError("缺少 app_name 参数".to_string())
                })?;
                self.app.quit(app_name).await
            }

            "macos_bring_to_front" => {
                let app_name = args["app_name"].as_str().ok_or_else(|| {
                    GearClawError::ToolExecutionError("缺少 app_name 参数".to_string())
                })?;
                self.app.bring_to_front(app_name).await
            }

            "macos_is_running" => {
                let app_name = args["app_name"].as_str().ok_or_else(|| {
                    GearClawError::ToolExecutionError("缺少 app_name 参数".to_string())
                })?;
                self.app.is_running(app_name).await
            }

            "macos_applescript" => {
                let script = args["script"].as_str().ok_or_else(|| {
                    GearClawError::ToolExecutionError("缺少 script 参数".to_string())
                })?;
                self.script.execute(script).await
            }

            "macos_jxa" => {
                let script = args["script"].as_str().ok_or_else(|| {
                    GearClawError::ToolExecutionError("缺少 script 参数".to_string())
                })?;
                self.script.execute_jxa(script).await
            }

            "macos_type_text" => {
                let text = args["text"].as_str().ok_or_else(|| {
                    GearClawError::ToolExecutionError("缺少 text 参数".to_string())
                })?;
                self.input.type_text(text).await
            }

            "macos_key_combo" => {
                let keys = args["keys"].as_array().ok_or_else(|| {
                    GearClawError::ToolExecutionError("keys 必须是数组".to_string())
                })?;
                let key_strs: Vec<&str> = keys.iter().filter_map(|k| k.as_str()).collect();
                self.input.key_combo(&key_strs).await
            }

            "macos_clipboard_read" => self.clipboard.read().await,

            "macos_clipboard_write" => {
                let text = args["text"].as_str().ok_or_else(|| {
                    GearClawError::ToolExecutionError("缺少 text 参数".to_string())
                })?;
                self.clipboard.write(text).await
            }

            "macos_notify" => {
                let title = args["title"].as_str().unwrap_or("GearClaw");
                let message = args["message"].as_str().ok_or_else(|| {
                    GearClawError::ToolExecutionError("缺少 message 参数".to_string())
                })?;
                let sound = args["sound"].as_bool().unwrap_or(false);
                self.notification.send(title, message, sound).await
            }

            "macos_open_url" => {
                let url = args["url"].as_str().ok_or_else(|| {
                    GearClawError::ToolExecutionError("缺少 url 参数".to_string())
                })?;
                self.system.open_url(url).await
            }

            "macos_search_in_browser" => {
                let query = args["query"].as_str().ok_or_else(|| {
                    GearClawError::ToolExecutionError("缺少 query 参数".to_string())
                })?;
                self.system.search_in_browser(query).await
            }

            "macos_say" => {
                let text = args["text"].as_str().ok_or_else(|| {
                    GearClawError::ToolExecutionError("缺少 text 参数".to_string())
                })?;
                let voice = args["voice"].as_str();
                let rate = args["rate"].as_u64().unwrap_or(175);
                self.system.say(text, voice, rate).await
            }

            _ => Err(GearClawError::ToolExecutionError(format!(
                "未知的 macOS 工具: {}",
                tool_name
            ))),
        }
    }

    /// Get list of available macOS tools
    pub fn available_tools(&self) -> Vec<serde_json::Value> {
        vec![
            // Application management
            json!({
                "name": "macos_launch_app",
                "description": "启动 macOS 应用程序",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "app_name": { "type": "string", "description": "应用名称 (如 Safari, Chrome, Terminal)" }
                    },
                    "required": ["app_name"]
                }
            }),
            json!({
                "name": "macos_quit_app",
                "description": "退出 macOS 应用程序",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "app_name": { "type": "string", "description": "应用名称" }
                    },
                    "required": ["app_name"]
                }
            }),
            json!({
                "name": "macos_bring_to_front",
                "description": "将应用程序切换到前台",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "app_name": { "type": "string", "description": "应用名称" }
                    },
                    "required": ["app_name"]
                }
            }),
            json!({
                "name": "macos_is_running",
                "description": "检查应用是否正在运行",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "app_name": { "type": "string", "description": "应用名称" }
                    },
                    "required": ["app_name"]
                }
            }),
            // Script execution
            json!({
                "name": "macos_applescript",
                "description": "执行 AppleScript 代码",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "script": { "type": "string", "description": "AppleScript 代码" }
                    },
                    "required": ["script"]
                }
            }),
            json!({
                "name": "macos_jxa",
                "description": "执行 JavaScript for Automation (JXA) 代码",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "script": { "type": "string", "description": "JXA JavaScript 代码" }
                    },
                    "required": ["script"]
                }
            }),
            // Input simulation
            json!({
                "name": "macos_type_text",
                "description": "模拟键盘输入文本",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "text": { "type": "string", "description": "要输入的文本" }
                    },
                    "required": ["text"]
                }
            }),
            json!({
                "name": "macos_key_combo",
                "description": "模拟组合键 (如 cmd+c, cmd+v)",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "keys": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "按键数组，如 [\"cmd\", \"c\"] 或 [\"cmd\", \"shift\", \"3\"]"
                        }
                    },
                    "required": ["keys"]
                }
            }),
            // Clipboard
            json!({
                "name": "macos_clipboard_read",
                "description": "读取剪贴板内容",
                "parameters": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            }),
            json!({
                "name": "macos_clipboard_write",
                "description": "写入剪贴板内容",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "text": { "type": "string", "description": "要复制到剪贴板的文本" }
                    },
                    "required": ["text"]
                }
            }),
            // Notifications
            json!({
                "name": "macos_notify",
                "description": "发送系统通知",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "title": { "type": "string", "description": "通知标题 (默认: GearClaw)" },
                        "message": { "type": "string", "description": "通知内容" },
                        "sound": { "type": "boolean", "description": "是否播放提示音 (默认: false)" }
                    },
                    "required": ["message"]
                }
            }),
            // System
            json!({
                "name": "macos_open_url",
                "description": "在默认浏览器中打开 URL",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "url": { "type": "string", "description": "要打开的 URL" }
                    },
                    "required": ["url"]
                }
            }),
            json!({
                "name": "macos_search_in_browser",
                "description": "在浏览器中执行搜索（打开 Google 搜索页面）",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "搜索关键词" }
                    },
                    "required": ["query"]
                }
            }),
            json!({
                "name": "macos_say",
                "description": "文字转语音 (TTS)",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "text": { "type": "string", "description": "要朗读的文本" },
                        "voice": { "type": "string", "description": "语音名称 (可选，如 'Ting-Ting')" },
                        "rate": { "type": "integer", "description": "语速 (默认: 175)" }
                    },
                    "required": ["text"]
                }
            }),
        ]
    }
}

impl Default for MacosController {
    fn default() -> Self {
        Self::new().expect("Failed to create MacosController")
    }
}
