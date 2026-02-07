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
            // Endpoint
            .child(self.render_field("Endpoint", &self.setting_endpoint, cx))
            // API Key
            .child(self.render_field("API Key", &self.setting_api_key, cx))
            // Model
            .child(self.render_field("Model", &self.setting_model, cx))
            // Embedding Model
            .child(self.render_field("Embedding Model", &self.setting_embedding, cx))
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

    fn save_settings(&mut self, cx: &mut Context<Self>) {
        let endpoint = self.setting_endpoint.read(cx).content().to_string();
        let api_key = self.setting_api_key.read(cx).content().to_string();
        let model = self.setting_model.read(cx).content().to_string();
        let embedding = self.setting_embedding.read(cx).content().to_string();

        // Load existing config or create sample
        let mut config = Config::load(&None).unwrap_or_else(|_| Config::sample());

        config.llm.endpoint = endpoint;
        config.llm.api_key = if api_key.is_empty() {
            None
        } else {
            Some(api_key)
        };
        config.llm.primary = model;
        config.llm.embedding_model = embedding;

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
