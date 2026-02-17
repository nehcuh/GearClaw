//! Compatibility wrapper for tools subsystem.
//! Delegates base executor behavior to `gearclaw_tools` and augments
//! macOS-specific tool exposure in `gearclaw_core`.
use crate::error::GearClawError;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[cfg(target_os = "macos")]
use crate::macos::MacosController;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub requires_args: bool,
    #[serde(default)]
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

pub struct ToolExecutor {
    inner: gearclaw_tools::ToolExecutor,
    #[cfg(target_os = "macos")]
    pub(crate) macos: MacosController,
}

impl ToolExecutor {
    pub fn new(security: &str) -> Self {
        #[cfg(target_os = "macos")]
        let macos = MacosController::new().expect("Failed to initialize macOS controller");
        Self {
            inner: gearclaw_tools::ToolExecutor::new(security),
            #[cfg(target_os = "macos")]
            macos,
        }
    }

    pub async fn exec_command(
        &self,
        cmd: &str,
        args: Vec<String>,
        cwd: Option<&std::path::Path>,
    ) -> Result<ToolResult, GearClawError> {
        self.inner
            .exec_command(cmd, args, cwd)
            .await
            .map(|r| ToolResult {
                success: r.success,
                output: r.output,
                error: r.error,
            })
            .map_err(|e| GearClawError::tool_execution_error(e.to_string()))
    }

    pub fn available_tools(&self) -> Vec<ToolSpec> {
        let mut tools = self
            .inner
            .available_tools()
            .into_iter()
            .map(|t| ToolSpec {
                name: t.name,
                description: t.description,
                requires_args: t.requires_args,
                parameters: t.parameters,
            })
            .collect::<Vec<_>>();

        #[cfg(target_os = "macos")]
        {
            let macos_tools = vec![
                ToolSpec {
                    name: "macos_launch_app".to_string(),
                    description: "启动 macOS 应用程序".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "app_name": { "type": "string", "description": "应用名称 (如 Safari, Chrome, Terminal)" }
                        },
                        "required": ["app_name"]
                    })),
                },
                ToolSpec {
                    name: "macos_quit_app".to_string(),
                    description: "退出 macOS 应用程序".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": { "app_name": { "type": "string", "description": "应用名称" } },
                        "required": ["app_name"]
                    })),
                },
                ToolSpec {
                    name: "macos_bring_to_front".to_string(),
                    description: "将应用程序切换到前台".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": { "app_name": { "type": "string", "description": "应用名称" } },
                        "required": ["app_name"]
                    })),
                },
                ToolSpec {
                    name: "macos_is_running".to_string(),
                    description: "检查应用是否正在运行".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": { "app_name": { "type": "string", "description": "应用名称" } },
                        "required": ["app_name"]
                    })),
                },
                ToolSpec {
                    name: "macos_applescript".to_string(),
                    description: "执行 AppleScript 代码".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": { "script": { "type": "string", "description": "AppleScript 代码" } },
                        "required": ["script"]
                    })),
                },
                ToolSpec {
                    name: "macos_jxa".to_string(),
                    description: "执行 JavaScript for Automation (JXA) 代码".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": { "script": { "type": "string", "description": "JXA JavaScript 代码" } },
                        "required": ["script"]
                    })),
                },
                ToolSpec {
                    name: "macos_type_text".to_string(),
                    description: "模拟键盘输入文本".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": { "text": { "type": "string", "description": "要输入的文本" } },
                        "required": ["text"]
                    })),
                },
                ToolSpec {
                    name: "macos_key_combo".to_string(),
                    description: "模拟组合键 (如 cmd+c, cmd+v)".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "keys": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "按键数组，如 [\"cmd\", \"c\"] 或 [\"cmd\", \"shift\", \"3\"]"
                            }
                        },
                        "required": ["keys"]
                    })),
                },
                ToolSpec {
                    name: "macos_clipboard_read".to_string(),
                    description: "读取剪贴板内容".to_string(),
                    requires_args: false,
                    parameters: Some(json!({ "type": "object", "properties": {}, "required": [] })),
                },
                ToolSpec {
                    name: "macos_clipboard_write".to_string(),
                    description: "写入剪贴板内容".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": { "text": { "type": "string", "description": "要复制到剪贴板的文本" } },
                        "required": ["text"]
                    })),
                },
                ToolSpec {
                    name: "macos_notify".to_string(),
                    description: "发送系统通知".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "title": { "type": "string", "description": "通知标题 (默认: GearClaw)" },
                            "message": { "type": "string", "description": "通知内容" },
                            "sound": { "type": "boolean", "description": "是否播放提示音 (默认: false)" }
                        },
                        "required": ["message"]
                    })),
                },
                ToolSpec {
                    name: "macos_open_url".to_string(),
                    description: "在默认浏览器中打开 URL".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": { "url": { "type": "string", "description": "要打开的 URL" } },
                        "required": ["url"]
                    })),
                },
                ToolSpec {
                    name: "macos_search_in_browser".to_string(),
                    description: "在浏览器中执行搜索，打开浏览器窗口让用户可以看到和浏览搜索结果。当用户想\"在浏览器中查看\"或\"打开浏览器搜索\"时使用此工具。".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": { "query": { "type": "string", "description": "搜索关键词" } },
                        "required": ["query"]
                    })),
                },
                ToolSpec {
                    name: "macos_say".to_string(),
                    description: "文字转语音 (TTS)".to_string(),
                    requires_args: true,
                    parameters: Some(json!({
                        "type": "object",
                        "properties": {
                            "text": { "type": "string", "description": "要朗读的文本" },
                            "voice": { "type": "string", "description": "语音名称 (可选，如 'Ting-Ting')" },
                            "rate": { "type": "integer", "description": "语速 (默认: 175)" }
                        },
                        "required": ["text"]
                    })),
                },
            ];
            tools.extend(macos_tools);
        }

        tools
    }
}
