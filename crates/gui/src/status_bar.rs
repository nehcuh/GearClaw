use gpui::*;
use crate::app::DesktopApp;
use crate::theme;
use crate::theme::Theme;

impl DesktopApp {
    pub fn render_status_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let current_mode = theme::mode(cx);
        let mcp_count = self.mcp_configured.iter().filter(|(_, e)| *e).count();
        let memory_label = if self.config_memory_enabled { "ON" } else { "OFF" };

        div()
            .h(px(28.))
            .bg(theme::accent(cx))
            .flex()
            .items_center()
            .px(px(12.))
            .gap(px(16.))
            .text_xs()
            .text_color(gpui::white())
            // Status
            .child(
                if self.is_loading {
                    "⏳ Processing..."
                } else {
                    "🟢 Ready"
                },
            )
            // Model name
            .child(
                div()
                    .id("status-model")
                    .opacity(0.85)
                    .child(self.config_model_name.clone()),
            )
            // MCP count
            .child(
                div()
                    .id("status-mcp")
                    .opacity(0.85)
                    .child(format!("MCP:{}", mcp_count)),
            )
            // Memory status
            .child(
                div()
                    .id("status-memory")
                    .opacity(0.85)
                    .child(format!("Memory:{}", memory_label)),
            )
            // Spacer
            .child(div().flex_grow())
            // Theme toggle
            .child(
                div()
                    .id("toggle-theme")
                    .cursor_pointer()
                    .hover(|s| s.opacity(0.8))
                    .child(format!("Theme: {}", current_mode.label()))
                    .on_click(cx.listener(|_this, _event, window, cx| {
                        let current = theme::mode(cx);
                        let new_mode = current.next();
                        let appearance = window.appearance();
                        cx.set_global(Theme::for_appearance(appearance, new_mode));
                        cx.notify();
                    })),
            )
            // Logs toggle
            .child(
                div()
                    .id("toggle-logs")
                    .cursor_pointer()
                    .hover(|s| s.opacity(0.8))
                    .child(if self.show_logs { "📜 Logs ▼" } else { "📜 Logs ▲" })
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.show_logs = !this.show_logs;
                        cx.notify();
                    })),
            )
    }
}
