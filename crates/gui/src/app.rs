use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use gpui::prelude::FluentBuilder;
use gpui::*;

use gearclaw_core::config::Config;

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
    pub role: String, // "user", "assistant", "error"
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
    pub runtime: Arc<tokio::runtime::Runtime>,
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
        let runtime = Arc::new(
            tokio::runtime::Runtime::new()
                .expect("Failed to initialize shared Tokio runtime for GUI"),
        );

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
            runtime,
            view_mode: if config_exists {
                ViewMode::Chat
            } else {
                ViewMode::Settings
            },
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
        self.session_messages
            .insert(self.active_session, std::mem::take(&mut self.messages));

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
            self.session_messages
                .insert(self.active_session, std::mem::take(&mut self.messages));
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
            let suffix = if content.chars().count() > 30 {
                "..."
            } else {
                ""
            };
            self.window_title = format!("{}{} - GearClaw", title_preview, suffix);
            // Also update the sidebar session name
            if self.active_session < self.sessions.len() {
                let session_name: String = content.chars().take(20).collect();
                let s_suffix = if content.chars().count() > 20 {
                    "..."
                } else {
                    ""
                };
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
        let runtime = self.runtime.clone();
        let task = cx.background_spawn({
            let cancel_flag = cancel_flag.clone();
            let runtime = runtime.clone();
            async move {
                let join_handle = runtime.spawn(Self::run_agent(content, cancel_flag));
                join_handle
                    .await
                    .map_err(|e| format!("Agent task join error: {}", e))?
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

                    // Auto-scroll to bottom after next frame is rendered
                    let scroll_handle = this.scroll_handle.clone();
                    window.on_next_frame(move |window, _cx| {
                        scroll_handle.scroll_to_bottom();
                        window.refresh();
                    });
                });
            })
            .ok();
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

    pub fn regenerate_message(
        &mut self,
        message_index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Find the user message that triggered this assistant response
        let user_message_index = self.messages[..message_index]
            .iter()
            .rposition(|m| m.role == "user");

        if let Some(user_idx) = user_message_index {
            let user_content = self.messages[user_idx].content.clone();

            // Remove this assistant message and all subsequent messages
            self.messages.truncate(message_index);

            // Set the input to the user message and send
            self.input.update(cx, |input, cx| {
                input.set_content(&user_content, cx);
            });
            self.on_send(window, cx);
        }
    }

    fn on_send_action(&mut self, _: &SendMessage, window: &mut Window, cx: &mut Context<Self>) {
        self.on_send(window, cx);
    }

    async fn run_agent(
        user_message: String,
        cancel_flag: Arc<AtomicBool>,
    ) -> Result<String, String> {
        use gearclaw_core::agent::Agent;
        use gearclaw_core::session::Session;

        // Load config and create Agent
        let config = Config::load(&None).map_err(|e| format!("{}", e))?;

        let agent = Agent::new(config)
            .await
            .map_err(|e| format!("Failed to create agent: {}", e))?;

        // Create a new session
        let mut session = Session::new("gui_session".to_string());

        // Process message with full agent capabilities (tools, MCP, etc.)
        let result = agent.process_message(&mut session, &user_message).await;

        // Check if cancelled during processing
        if cancel_flag.load(Ordering::SeqCst) {
            return Ok("[Stopped]".to_string());
        }

        match result {
            Ok(response) => Ok(response),
            Err(e) => Err(format!("Agent error: {}", e)),
        }
    }
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
                .min_h(px(0.0))
                .bg(theme::bg(cx))
                .child(self.render_chat(cx))
                .child(self.render_input_bar(cx)),
            ViewMode::Settings => div()
                .flex()
                .flex_col()
                .flex_grow()
                .min_h(px(0.0))
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
            .when(show_logs, |el: Div| el.child(self.render_log_panel(cx)))
            .child(self.render_status_bar(cx))
    }
}
