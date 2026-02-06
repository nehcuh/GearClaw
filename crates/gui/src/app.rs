use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use gpui::*;
use gpui::prelude::FluentBuilder;
use futures::StreamExt;

use gearclaw_core::config::Config;
use gearclaw_core::llm::{LLMClient, Message};

use crate::text_input::TextInput;
use crate::theme;

/// Determines which view to show in the main content area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Chat,
    Settings,
}

/// A single chat message displayed in the UI.
#[derive(Clone)]
pub struct ChatMessage {
    pub role: String,   // "user", "assistant", "error"
    pub content: String,
}

// Application-level actions
gpui::actions!(gearclaw, [SendMessage, Quit]);

pub struct DesktopApp {
    // Chat state
    pub messages: Vec<ChatMessage>,
    pub session_messages: HashMap<usize, Vec<ChatMessage>>,
    pub sessions: Vec<String>,
    pub active_session: usize,

    // UI state
    pub input: Entity<TextInput>,
    pub focus_handle: FocusHandle,
    pub scroll_handle: ScrollHandle,
    pub is_loading: bool,
    pub cancel_flag: Arc<AtomicBool>,
    pub view_mode: ViewMode,
    pub window_title: String,
    pub show_logs: bool,

    // Settings fields (TextInput entities)
    pub setting_endpoint: Entity<TextInput>,
    pub setting_api_key: Entity<TextInput>,
    pub setting_model: Entity<TextInput>,
    pub setting_embedding: Entity<TextInput>,

    // Toggles
    pub skills_on: bool,
    pub memory_on: bool,
    pub security_full: bool,
}

impl DesktopApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextInput::new("Type a message...", cx));

        // Load existing config to pre-fill settings, or use defaults
        let (endpoint, api_key, model, embedding) = match Config::load(&None) {
            Ok(config) => (
                config.llm.endpoint.clone(),
                config.llm.api_key.clone().unwrap_or_default(),
                config.llm.primary.clone(),
                config.llm.embedding_model.clone(),
            ),
            Err(_) => (
                "http://localhost:1234/v1".to_string(),
                String::new(),
                "gpt-4".to_string(),
                "text-embedding-3-small".to_string(),
            ),
        };

        let config_exists = Config::load(&None).is_ok();

        let setting_endpoint = cx.new(|cx| {
            let mut ti = TextInput::new("Endpoint URL", cx);
            ti.set_content(&endpoint, cx);
            ti
        });
        let setting_api_key = cx.new(|cx| {
            let mut ti = TextInput::new("API Key", cx);
            ti.set_content(&api_key, cx);
            ti
        });
        let setting_model = cx.new(|cx| {
            let mut ti = TextInput::new("Model name", cx);
            ti.set_content(&model, cx);
            ti
        });
        let setting_embedding = cx.new(|cx| {
            let mut ti = TextInput::new("Embedding model", cx);
            ti.set_content(&embedding, cx);
            ti
        });

        DesktopApp {
            messages: Vec::new(),
            session_messages: HashMap::new(),
            sessions: vec!["Chat 1".to_string()],
            active_session: 0,
            input,
            focus_handle: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
            is_loading: false,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            view_mode: if config_exists { ViewMode::Chat } else { ViewMode::Settings },
            window_title: "GearClaw".to_string(),
            show_logs: false,
            setting_endpoint,
            setting_api_key,
            setting_model,
            setting_embedding,
            skills_on: true,
            memory_on: true,
            security_full: true,
        }
    }

    pub fn new_session(&mut self, cx: &mut Context<Self>) {
        // Save current session messages
        self.session_messages.insert(self.active_session, std::mem::take(&mut self.messages));

        let idx = self.sessions.len() + 1;
        self.sessions.push(format!("Chat {}", idx));
        self.active_session = self.sessions.len() - 1;
        self.messages.clear();
        self.input.update(cx, |input, cx| input.clear(cx));
        self.window_title = "GearClaw".to_string();
        self.view_mode = ViewMode::Chat;
        cx.notify();
    }

    pub fn switch_session(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.sessions.len() && index != self.active_session {
            // Save current session messages
            self.session_messages.insert(self.active_session, std::mem::take(&mut self.messages));
            // Restore target session messages
            self.messages = self.session_messages.remove(&index).unwrap_or_default();
            self.active_session = index;
            self.view_mode = ViewMode::Chat;
            cx.notify();
        }
    }

    pub fn on_send(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let content = self.input.read(cx).content().to_string();
        let content = content.trim().to_string();
        if content.is_empty() || self.is_loading {
            return;
        }

        // Update window title from first user message
        if self.messages.is_empty() {
            let title_preview: String = content.chars().take(30).collect();
            let suffix = if content.chars().count() > 30 { "..." } else { "" };
            self.window_title = format!("{}{} - GearClaw", title_preview, suffix);
            // Also update the sidebar session name
            if self.active_session < self.sessions.len() {
                let session_name: String = content.chars().take(20).collect();
                let s_suffix = if content.chars().count() > 20 { "..." } else { "" };
                self.sessions[self.active_session] = format!("{}{}", session_name, s_suffix);
            }
            window.set_window_title(&self.window_title);
        }

        // Add user message
        self.messages.push(ChatMessage {
            role: "user".to_string(),
            content: content.clone(),
        });

        // Clear input
        self.input.update(cx, |input, cx| input.clear(cx));

        // Set loading
        self.is_loading = true;
        self.cancel_flag.store(false, Ordering::SeqCst);
        cx.notify();

        // Spawn background thread with its own Tokio runtime for network I/O
        let cancel_flag = self.cancel_flag.clone();
        let task = cx.background_spawn({
            let cancel_flag = cancel_flag.clone();
            async move {
                // reqwest needs a Tokio runtime; GPUI uses its own executor
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| format!("Failed to create runtime: {}", e))?;
                rt.block_on(Self::run_agent(content, cancel_flag))
            }
        });

        cx.spawn_in(window, async move |this, cx| {
            let result = task.await;

            cx.update(|window, cx| {
                let _ = this.update(cx, |this, cx| {
                    match result {
                        Ok(response) => {
                            if !response.is_empty() {
                                this.messages.push(ChatMessage {
                                    role: "assistant".to_string(),
                                    content: response,
                                });
                            }
                        }
                        Err(e) => {
                            if !cancel_flag.load(Ordering::SeqCst) {
                                this.messages.push(ChatMessage {
                                    role: "error".to_string(),
                                    content: format!("Error: {}", e),
                                });
                            }
                        }
                    }
                    this.is_loading = false;
                    cx.notify();

                    // Auto-scroll to bottom
                    this.scroll_handle.scroll_to_bottom();
                    window.refresh();
                });
            }).ok();
        })
        .detach();
    }

    pub fn on_stop(&mut self, cx: &mut Context<Self>) {
        self.cancel_flag.store(true, Ordering::SeqCst);
        self.is_loading = false;
        self.messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: "[Stopped]".to_string(),
        });
        cx.notify();
    }

    fn on_send_action(&mut self, _: &SendMessage, window: &mut Window, cx: &mut Context<Self>) {
        self.on_send(window, cx);
    }

    async fn run_agent(
        user_message: String,
        cancel_flag: Arc<AtomicBool>,
    ) -> Result<String, String> {
        // Load config to get LLM settings
        let config = Config::load(&None).map_err(|e| format!("{}", e))?;

        let api_key = config.llm.api_key.clone()
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .ok_or_else(|| "未设置 API Key。请配置 OPENAI_API_KEY 环境变量".to_string())?;

        let endpoint = std::env::var("OPENAI_BASE_URL")
            .unwrap_or_else(|_| config.llm.endpoint.clone());

        let llm_client = LLMClient::new(
            api_key,
            endpoint,
            config.llm.primary.clone(),
            config.llm.embedding_model.clone(),
        );

        // Build messages
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: Some(config_system_prompt()),
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: "user".to_string(),
                content: Some(user_message),
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        // Make streaming request
        let mut stream = llm_client
            .chat_completion_stream(messages, None, Some(4096))
            .await
            .map_err(|e| format!("{}", e))?;

        let mut full_response = String::new();

        while let Some(result) = stream.next().await {
            if cancel_flag.load(Ordering::SeqCst) {
                break;
            }
            match result {
                Ok(response) => {
                    for choice in response.choices {
                        if let Some(content) = choice.delta.content {
                            full_response.push_str(&content);
                        }
                    }
                }
                Err(e) => {
                    let msg = format!("{}", e);
                    if msg.contains("Stream finished") {
                        break;
                    }
                }
            }
        }

        Ok(full_response)
    }
}

fn config_system_prompt() -> String {
    "你是 GearClaw 🦞，一个智能 AI 助手。请用友好、简洁的方式与用户交流。".to_string()
}

impl Focusable for DesktopApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DesktopApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let main_content = match self.view_mode {
            ViewMode::Chat => div()
                .flex()
                .flex_col()
                .flex_grow()
                .bg(theme::bg(cx))
                .child(self.render_chat(cx))
                .child(self.render_input_bar(cx)),
            ViewMode::Settings => div()
                .flex()
                .flex_col()
                .flex_grow()
                .bg(theme::bg(cx))
                .child(self.render_settings(cx)),
        };

        let show_logs = self.show_logs;

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme::bg(cx))
            .text_color(theme::text(cx))
            .font_family("Menlo")
            .track_focus(&self.focus_handle(cx))
            .on_action(cx.listener(Self::on_send_action))
            .child(
                // Main content area
                div()
                    .flex()
                    .flex_row()
                    .flex_grow()
                    .child(self.render_sidebar(cx))
                    .child(main_content),
            )
            .when(show_logs, |el: Div| {
                el.child(self.render_log_panel(cx))
            })
            .child(self.render_status_bar(cx))
    }
}
