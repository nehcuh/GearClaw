use crate::app::{DesktopApp, ViewMode};
use crate::theme;
use gearclaw_core::config::Config;
use gpui::*;

impl DesktopApp {
    pub fn render_settings(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let accent_hover = theme::accent_hover(cx);
        let border = theme::border(cx);
        let text_muted = theme::text_muted(cx);

        div()
            .id("settings-scroll")
            .flex_grow()
            .overflow_y_scroll()
            .p(px(32.))
            .flex()
            .flex_col()
            .gap(px(20.))
            .child(div().text_xl().child("⚙️ Settings"))
            .child(
                div().text_sm().text_color(text_muted).child(
                    "Configure LLM connection. Settings are saved to ~/.gearclaw/config.toml",
                ),
            )
            .child(div().text_sm().text_color(text_muted).child("LLM"))
            // Endpoint
            .child(self.render_field("Endpoint", &self.setting_endpoint, cx))
            // API Key
            .child(self.render_field("API Key", &self.setting_api_key, cx))
            // Model
            .child(self.render_field("Model", &self.setting_model, cx))
            // Embedding Model
            .child(self.render_field("Embedding Model", &self.setting_embedding, cx))
            // Temperature
            .child(self.render_field("Temperature", &self.setting_temperature, cx))
            .child(div().text_sm().text_color(text_muted).child("Agent"))
            // Agent Name
            .child(self.render_field("Agent Name", &self.setting_agent_name, cx))
            // System Prompt
            .child(self.render_multiline_field(
                "System Prompt",
                &self.setting_system_prompt,
                160.0,
                cx,
            ))
            .child(div().text_sm().text_color(text_muted).child("Tools"))
            // Tools Security
            .child(
                self.render_field("Security Level", &self.setting_tools_security, cx),
            )
            // Tools Profile
            .child(self.render_field("Profile", &self.setting_tools_profile, cx))
            .child(div().text_sm().text_color(text_muted).child("Memory"))
            // Memory Enabled
            .child(self.render_field("Enabled", &self.setting_memory_enabled, cx))
            // Memory DB Path
            .child(self.render_field("DB Path", &self.setting_memory_db_path, cx))
            .child(div().text_sm().text_color(text_muted).child("Session"))
            // Session Directory
            .child(self.render_field("Session Dir", &self.setting_session_dir, cx))
            // Save Interval
            .child(
                self.render_field("Save Interval (s)", &self.setting_session_save_interval, cx),
            )
            // Max Tokens
            .child(self.render_field("Max Tokens", &self.setting_session_max_tokens, cx))
            // Save button
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(12.))
                    .child(
                        div()
                            .id("save-settings")
                            .px(px(20.))
                            .py(px(8.))
                            .bg(theme::accent(cx))
                            .rounded_md()
                            .cursor_pointer()
                            .text_sm()
                            .text_color(gpui::white())
                            .child("Save Settings")
                            .hover(move |s| s.bg(accent_hover))
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.save_settings(cx);
                            })),
                    )
                    .child(
                        div()
                            .id("back-to-chat")
                            .px(px(20.))
                            .py(px(8.))
                            .border_1()
                            .border_color(border)
                            .rounded_md()
                            .cursor_pointer()
                            .text_sm()
                            .child("Back to Chat")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.view_mode = ViewMode::Chat;
                                cx.notify();
                            })),
                    ),
            )
    }

    fn render_field(
        &self,
        label: &str,
        input: &Entity<crate::text_input::TextInput>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let text_muted = theme::text_muted(cx);
        div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(
                div()
                    .text_sm()
                    .text_color(text_muted)
                    .child(label.to_string()),
            )
            .child(input.clone())
    }

    fn render_multiline_field(
        &self,
        label: &str,
        input: &Entity<crate::multiline_input::MultiLineTextInput>,
        height: f32,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let text_muted = theme::text_muted(cx);
        div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(
                div()
                    .text_sm()
                    .text_color(text_muted)
                    .child(label.to_string()),
            )
            .child(div().h(px(height)).child(input.clone()))
    }

    fn save_settings(&mut self, cx: &mut Context<Self>) {
        let endpoint = self.setting_endpoint.read(cx).content().to_string();
        let api_key = self.setting_api_key.read(cx).content().to_string();
        let model = self.setting_model.read(cx).content().to_string();
        let embedding = self.setting_embedding.read(cx).content().to_string();
        let temperature = self.setting_temperature.read(cx).content().to_string();
        let session_max_tokens = self
            .setting_session_max_tokens
            .read(cx)
            .content()
            .to_string();
        let agent_name = self.setting_agent_name.read(cx).content().to_string();
        let system_prompt = self.setting_system_prompt.read(cx).content().to_string();
        let tools_security = self.setting_tools_security.read(cx).content().to_string();
        let tools_profile = self.setting_tools_profile.read(cx).content().to_string();
        let memory_enabled = self.setting_memory_enabled.read(cx).content().to_string();
        let memory_db_path = self.setting_memory_db_path.read(cx).content().to_string();
        let session_dir = self.setting_session_dir.read(cx).content().to_string();
        let session_save_interval = self
            .setting_session_save_interval
            .read(cx)
            .content()
            .to_string();

        // Load existing config or create sample
        let mut config = Config::load(&None).unwrap_or_else(|_| Config::sample());
        let mut errors = Vec::new();
        let parse_bool = |value: &str| match value.trim().to_lowercase().as_str() {
            "true" | "1" | "yes" | "y" => Some(true),
            "false" | "0" | "no" | "n" => Some(false),
            _ => None,
        };

        config.llm.endpoint = endpoint;
        config.llm.api_key = if api_key.is_empty() {
            None
        } else {
            Some(api_key)
        };
        config.llm.primary = model;
        config.llm.embedding_model = embedding;
        if !temperature.trim().is_empty() {
            match temperature.trim().parse::<f32>() {
                Ok(value) if (0.0..=2.0).contains(&value) => {
                    config.llm.temperature = Some(value)
                }
                Ok(_) => errors.push("Temperature must be between 0.0 and 2.0".to_string()),
                Err(_) => errors.push("Temperature must be a number".to_string()),
            }
        }

        if !session_max_tokens.trim().is_empty() {
            match session_max_tokens.trim().parse::<usize>() {
                Ok(value) => config.session.max_tokens = value,
                Err(_) => errors.push("Max tokens must be an integer".to_string()),
            }
        }

        if !agent_name.trim().is_empty() {
            config.agent.name = agent_name;
        }
        if !system_prompt.trim().is_empty() {
            config.agent.system_prompt = system_prompt;
        }

        let security = tools_security.trim().to_lowercase();
        if !security.is_empty() {
            match security.as_str() {
                "deny" | "allowlist" | "full" => config.tools.security = security,
                _ => errors.push("Security level must be deny, allowlist, or full".to_string()),
            }
        }

        let profile = tools_profile.trim().to_lowercase();
        if !profile.is_empty() {
            match profile.as_str() {
                "minimal" | "coding" | "messaging" | "full" => config.tools.profile = profile,
                _ => errors.push("Profile must be minimal, coding, messaging, or full".to_string()),
            }
        }

        if !memory_enabled.trim().is_empty() {
            match parse_bool(&memory_enabled) {
                Some(value) => config.memory.enabled = value,
                None => errors.push("Memory enabled must be true or false".to_string()),
            }
        }

        if !memory_db_path.trim().is_empty() {
            config.memory.db_path = memory_db_path.into();
        }

        if !session_dir.trim().is_empty() {
            config.session.session_dir = session_dir.into();
        }

        if !session_save_interval.trim().is_empty() {
            match session_save_interval.trim().parse::<u64>() {
                Ok(value) => config.session.save_interval = value,
                Err(_) => errors.push("Save interval must be an integer".to_string()),
            }
        }

        if !errors.is_empty() {
            self.messages.push(crate::app::ChatMessage {
                role: "error".to_string(),
                content: format!("Failed to save settings: {}", errors.join("; ")),
            });
            cx.notify();
            return;
        }

        // Save to default path
        let config_path = dirs::home_dir().unwrap().join(".gearclaw/config.toml");

        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        match config.save(&config_path) {
            Ok(_) => {
                self.messages.push(crate::app::ChatMessage {
                    role: "assistant".to_string(),
                    content: "✅ Settings saved successfully!".to_string(),
                });
                self.view_mode = ViewMode::Chat;
            }
            Err(e) => {
                self.messages.push(crate::app::ChatMessage {
                    role: "error".to_string(),
                    content: format!("Failed to save settings: {}", e),
                });
            }
        }
        cx.notify();
    }
}
