use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use gpui::prelude::FluentBuilder;
use gpui::*;
use serde::{Deserialize, Serialize};

use gearclaw_core::config::Config;
use gearclaw_core::skills::SkillManager;

use crate::multiline_input::MultiLineTextInput;
use crate::session_store::SessionStore;
use crate::text_input::TextInput;
use crate::theme;

/// Determines which view to show in the main content area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Chat,
    Memory,
    Mcp,
    Skills,
    Monitor,
    Settings,
}

/// Active tab in the Settings view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Llm,
    Agent,
    Tools,
    Memory,
    Session,
}

/// A single memory search result shown in the Memory view.
#[derive(Debug, Clone)]
pub struct MemoryHit {
    pub score: f32,
    pub path: String,
    pub preview: String,
    pub line: usize,
}

/// A single chat message displayed in the UI.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "user", "assistant", "error"
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevelFilter {
    All,
    Info,
    Warn,
    Error,
}

impl LogLevelFilter {
    pub fn next(self) -> Self {
        match self {
            LogLevelFilter::All => LogLevelFilter::Info,
            LogLevelFilter::Info => LogLevelFilter::Warn,
            LogLevelFilter::Warn => LogLevelFilter::Error,
            LogLevelFilter::Error => LogLevelFilter::All,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            LogLevelFilter::All => "All",
            LogLevelFilter::Info => "Info",
            LogLevelFilter::Warn => "Warn",
            LogLevelFilter::Error => "Error",
        }
    }
}

// Application-level actions
gpui::actions!(gearclaw, [SendMessage, Quit]);

pub struct DesktopApp {
    // Chat state
    pub messages: Vec<ChatMessage>,
    pub session_messages: HashMap<usize, Vec<ChatMessage>>,
    pub sessions: Vec<String>,
    pub active_session: usize,
    pub session_store: Option<SessionStore>,

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
    pub setting_temperature: Entity<TextInput>,
    pub setting_session_max_tokens: Entity<TextInput>,
    pub setting_agent_name: Entity<TextInput>,
    pub setting_system_prompt: Entity<MultiLineTextInput>,
    pub setting_tools_security: Entity<TextInput>,
    pub setting_tools_profile: Entity<TextInput>,
    pub setting_memory_enabled: Entity<TextInput>,
    pub setting_memory_db_path: Entity<TextInput>,
    pub setting_session_dir: Entity<TextInput>,
    pub setting_session_save_interval: Entity<TextInput>,

    // Settings
    pub settings_tab: SettingsTab,

    // MCP view state
    pub mcp_search_input: Entity<TextInput>,
    pub mcp_registry_results: Vec<&'static gearclaw_mcp::RegistryEntry>,
    pub mcp_configured: Vec<(String, bool)>, // (name, enabled)

    // Memory view state
    pub memory_search_input: Entity<TextInput>,
    pub memory_results: Vec<MemoryHit>,
    pub memory_is_searching: bool,

    // Skills view state
    pub loaded_skills: Vec<(String, String, String)>, // (name, description, path)
    pub skills_path_label: String,

    // Derived config info for display
    pub config_model_name: String,
    pub config_memory_enabled: bool,

    // Log filters
    pub log_filter: Entity<TextInput>,
    pub log_level_filter: LogLevelFilter,

    // Status indicators
    pub status_gateway: String,
    pub status_channels: String,
    pub status_llm: String,
    pub status_memory: String,
    pub status_mcp: String,
    pub status_updated_at: Option<String>,
}

impl DesktopApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextInput::new("Type a message...", cx));
        let runtime = Arc::new(
            tokio::runtime::Runtime::new()
                .expect("Failed to initialize shared Tokio runtime for GUI"),
        );

        let (config, config_exists) = match Config::load(&None) {
            Ok(config) => (config, true),
            Err(_) => (Config::sample(), false),
        };

        // Initialize session store
        let session_store = SessionStore::new(config.session.session_dir.join("sessions/"))
            .ok(); // Don't fail if we can't create the store

        // Load existing sessions from disk
        let (sessions, messages, session_messages, active_session) = if let Some(store) = &session_store {
            match store.load_sessions() {
                Ok(loaded_sessions) if !loaded_sessions.is_empty() => {
                    let session_names: Vec<String> = loaded_sessions.iter()
                        .map(|s| s.name.clone())
                        .collect();
                    let mut session_map = HashMap::new();
                    for session in &loaded_sessions {
                        session_map.insert(session.id, session.messages.clone());
                    }
                    let first_session_id = loaded_sessions[0].id;
                    let first_messages = session_map.get(&first_session_id)
                        .cloned()
                        .unwrap_or_default();

                    (session_names, first_messages, session_map, 0)
                }
                _ => (
                    vec!["Chat 1".to_string()],
                    Vec::new(),
                    HashMap::new(),
                    0
                )
            }
        } else {
            (
                vec!["Chat 1".to_string()],
                Vec::new(),
                HashMap::new(),
                0
            )
        };

        let endpoint = config.llm.endpoint.clone();
        let api_key = config.llm.api_key.clone().unwrap_or_default();
        let model = config.llm.primary.clone();
        let embedding = config.llm.embedding_model.clone();
        let temperature = config.llm.temperature.unwrap_or(0.7);
        let session_max_tokens = config.session.max_tokens;
        let agent_name = config.agent.name.clone();
        let system_prompt = config.agent.system_prompt.clone();
        let tools_security = config.tools.security.clone();
        let tools_profile = config.tools.profile.clone();
        let memory_enabled = config.memory.enabled;
        let memory_db_path = config.memory.db_path.to_string_lossy().to_string();
        let session_dir = config.session.session_dir.to_string_lossy().to_string();
        let session_save_interval = config.session.save_interval;

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
        let setting_temperature = cx.new(|cx| {
            let mut ti = TextInput::new("Temperature (e.g. 0.7)", cx);
            ti.set_content(&format!("{:.2}", temperature), cx);
            ti
        });
        let setting_session_max_tokens = cx.new(|cx| {
            let mut ti = TextInput::new("Max tokens (session)", cx);
            ti.set_content(&session_max_tokens.to_string(), cx);
            ti
        });
        let setting_agent_name = cx.new(|cx| {
            let mut ti = TextInput::new("Agent name", cx);
            ti.set_content(&agent_name, cx);
            ti
        });
        let setting_system_prompt = cx.new(|cx| {
            let mut ti = MultiLineTextInput::new("System prompt", cx);
            ti.set_content(&system_prompt, cx);
            ti
        });
        let setting_tools_security = cx.new(|cx| {
            let mut ti = TextInput::new("Tools security (deny/allowlist/full)", cx);
            ti.set_content(&tools_security, cx);
            ti
        });
        let setting_tools_profile = cx.new(|cx| {
            let mut ti = TextInput::new("Tools profile (minimal/coding/messaging/full)", cx);
            ti.set_content(&tools_profile, cx);
            ti
        });
        let setting_memory_enabled = cx.new(|cx| {
            let mut ti = TextInput::new("Memory enabled (true/false)", cx);
            ti.set_content(if memory_enabled { "true" } else { "false" }, cx);
            ti
        });
        let setting_memory_db_path = cx.new(|cx| {
            let mut ti = TextInput::new("Memory DB path", cx);
            ti.set_content(&memory_db_path, cx);
            ti
        });
        let setting_session_dir = cx.new(|cx| {
            let mut ti = TextInput::new("Session directory", cx);
            ti.set_content(&session_dir, cx);
            ti
        });
        let setting_session_save_interval = cx.new(|cx| {
            let mut ti = TextInput::new("Save interval (seconds)", cx);
            ti.set_content(&session_save_interval.to_string(), cx);
            ti
        });
        let log_filter = cx.new(|cx| TextInput::new("Filter logs...", cx));

        // New view search inputs
        let mcp_search_input = cx.new(|cx| TextInput::new("Search MCP registry...", cx));
        let memory_search_input = cx.new(|cx| TextInput::new("Search memories...", cx));

        // Load skills at startup
        let skills_path = config.agent.skills_path.clone();
        let skills_path_label = skills_path.to_string_lossy().to_string();
        let mut skill_manager = SkillManager::new();
        let _ = skill_manager.load_from_dir(&skills_path);
        let loaded_skills: Vec<(String, String, String)> = skill_manager
            .skills
            .into_iter()
            .map(|s| (s.name, s.description, s.path.to_string_lossy().to_string()))
            .collect();

        // Collect MCP configured servers
        let mcp_configured: Vec<(String, bool)> = config
            .mcp
            .servers
            .iter()
            .map(|(k, v)| (k.clone(), v.enabled))
            .collect();

        let config_model_name = model.clone();
        let config_memory_enabled = memory_enabled;

        DesktopApp {
            messages,
            session_messages,
            sessions,
            active_session,
            session_store,
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
            setting_temperature,
            setting_session_max_tokens,
            setting_agent_name,
            setting_system_prompt,
            setting_tools_security,
            setting_tools_profile,
            setting_memory_enabled,
            setting_memory_db_path,
            setting_session_dir,
            setting_session_save_interval,
            settings_tab: crate::app::SettingsTab::Llm,
            mcp_search_input,
            mcp_registry_results: Vec::new(),
            mcp_configured,
            memory_search_input,
            memory_results: Vec::new(),
            memory_is_searching: false,
            loaded_skills,
            skills_path_label,
            config_model_name,
            config_memory_enabled,
            log_filter,
            log_level_filter: LogLevelFilter::All,
            status_gateway: "Unknown".to_string(),
            status_channels: "Unknown".to_string(),
            status_llm: "Unknown".to_string(),
            status_memory: "Unknown".to_string(),
            status_mcp: "Unknown".to_string(),
            status_updated_at: None,
        }
    }

    pub fn new_session(&mut self, cx: &mut Context<Self>) {
        // Save current session to disk
        self.save_current_session(cx);

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
            // Save current session to disk
            self.save_current_session(cx);

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

    /// Saves the current session to disk.
    fn save_current_session(&self, cx: &mut Context<Self>) {
        if let Some(store) = &self.session_store {
            use crate::session_store::SessionData;

            let session_data = SessionData {
                id: self.active_session,
                name: self.sessions.get(self.active_session)
                    .cloned()
                    .unwrap_or_else(|| format!("Session {}", self.active_session)),
                messages: self.messages.clone(),
                created_at: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.6f%:z").to_string(),
                updated_at: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.6f%:z").to_string(),
            };

            if let Err(e) = store.save_session(&session_data) {
                tracing::error!("Failed to save session {}: {}", self.active_session, e);
            } else {
                tracing::debug!("Saved session {} to disk", self.active_session);
            }
        }
        cx.notify();
    }

    pub fn refresh_status(&mut self, cx: &mut Context<Self>) {
        // Update timestamp first
        self.status_updated_at = Some(chrono::Local::now().format("%H:%M:%S").to_string());

        // Check MCP status (immediate check)
        let mcp_enabled = self.mcp_configured.iter().filter(|(_, e)| *e).count();
        self.status_mcp = if mcp_enabled > 0 {
            format!("{} servers", mcp_enabled)
        } else {
            "None configured".to_string()
        };

        // Check Memory status (immediate check)
        self.status_memory = if self.config_memory_enabled {
            "Enabled".to_string()
        } else {
            "Disabled".to_string()
        };

        // For LLM and Gateway, we'll perform async checks
        // Set initial status to "Checking..."
        self.status_llm = "Checking...".to_string();
        self.status_gateway = "Checking...".to_string();
        cx.notify();

        // Spawn background task to check actual status
        let runtime = self.runtime.clone();
        cx.background_spawn(async move {
            // TODO: Implement actual health checks here
            // For now, just simulate the checks with delays
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            let llm_status = "Available".to_string();
            let gateway_status = "Connected".to_string();
            (llm_status, gateway_status)
        });
        cx.notify();
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

                    // Auto-save session after message is processed
                    this.save_current_session(cx);

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

    pub fn on_mcp_search(&mut self, cx: &mut Context<Self>) {
        let query = self.mcp_search_input.read(cx).content().to_string();
        self.mcp_registry_results = gearclaw_mcp::search_registry(&query);
        cx.notify();
    }

    /// Toggles the enabled state of an MCP server.
    pub fn toggle_mcp_server(&mut self, server_name: String, cx: &mut Context<Self>) {
        // Find and toggle the server
        if let Some(server) = self.mcp_configured.iter_mut().find(|(name, _)| name == &server_name) {
            server.1 = !server.1;

            // Update the config file
            if let Ok(config) = Config::load(&None) {
                tracing::info!("MCP server '{}' toggled to {}", server_name, server.1);

                // Note: Config changes would need to be saved here
                // For now, we're just updating the in-memory state
            }
        }
        cx.notify();
    }

    /// Deletes an MCP server from the configuration.
    pub fn delete_mcp_server(&mut self, server_name: String, cx: &mut Context<Self>) {
        // Remove from configured list
        self.mcp_configured.retain(|(name, _)| name != &server_name);

        tracing::info!("MCP server '{}' removed from configuration", server_name);

        // Note: Config file would need to be updated here
        cx.notify();
    }

    pub fn on_memory_search(&mut self, cx: &mut Context<Self>) {
        if self.memory_is_searching {
            return;
        }
        let query = self.memory_search_input.read(cx).content().to_string();
        if query.trim().is_empty() {
            return;
        }
        self.memory_is_searching = true;
        self.memory_results.clear();
        cx.notify();

        let runtime = self.runtime.clone();
        let task = cx.background_spawn(async move {
            let join_handle = runtime.spawn(async move {
                let config = Config::load(&None).map_err(|e| format!("Config error: {}", e))?;
                if config.llm.api_key.is_none() {
                    return Err("No API key configured — cannot run memory search".to_string());
                }
                use gearclaw_llm::LLMClient;
                use gearclaw_memory::{MemoryConfig, MemoryManager};
                use std::sync::Arc;
                let llm_client = Arc::new(LLMClient::new(
                    config.llm.api_key.clone().unwrap_or_default(),
                    config.llm.endpoint.clone(),
                    config.llm.embedding_endpoint.clone(),
                    config.llm.primary.clone(),
                    config.llm.embedding_model.clone(),
                    config.llm.temperature,
                ));
                let mem_config = MemoryConfig {
                    enabled: true,
                    db_path: config.memory.db_path.clone(),
                };
                let manager = MemoryManager::new(
                    mem_config,
                    config.agent.workspace.clone(),
                    llm_client,
                )
                .map_err(|e| format!("Memory init error: {}", e))?;
                let results = manager
                    .search(&query, 10)
                    .await
                    .map_err(|e| format!("Search error: {}", e))?;
                Ok::<Vec<gearclaw_memory::SearchResult>, String>(results)
            });
            join_handle
                .await
                .map_err(|e| format!("Task join error: {}", e))?
        });

        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.memory_is_searching = false;
                match result {
                    Ok(results) => {
                        this.memory_results = results
                            .into_iter()
                            .map(|r| MemoryHit {
                                score: r.score,
                                path: r.path,
                                preview: r.text.chars().take(120).collect(),
                                line: r.start_line.unwrap_or(0),
                            })
                            .collect();
                    }
                    Err(e) => {
                        this.memory_results = vec![MemoryHit {
                            score: 0.0,
                            path: "(error)".to_string(),
                            preview: e,
                            line: 0,
                        }];
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    async fn run_agent(
        user_message: String,
        cancel_flag: Arc<AtomicBool>,
    ) -> Result<String, String> {
        use gearclaw_agent::Agent;
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
            ViewMode::Memory => div()
                .flex()
                .flex_col()
                .flex_grow()
                .min_h(px(0.0))
                .bg(theme::bg(cx))
                .child(self.render_memory(cx)),
            ViewMode::Mcp => div()
                .flex()
                .flex_col()
                .flex_grow()
                .min_h(px(0.0))
                .bg(theme::bg(cx))
                .child(self.render_mcp(cx)),
            ViewMode::Skills => div()
                .flex()
                .flex_col()
                .flex_grow()
                .min_h(px(0.0))
                .bg(theme::bg(cx))
                .child(self.render_skills(cx)),
            ViewMode::Settings => div()
                .flex()
                .flex_col()
                .flex_grow()
                .min_h(px(0.0))
                .bg(theme::bg(cx))
                .child(self.render_settings(cx)),
            ViewMode::Monitor => div()
                .flex()
                .flex_col()
                .flex_grow()
                .min_h(px(0.0))
                .bg(theme::bg(cx))
                .child(self.render_monitor(cx)),
        };

        let show_logs = self.show_logs;

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme::bg(cx))
            .text_color(theme::text(cx))
            .font_family("Menlo") // Use specific monospace font
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
