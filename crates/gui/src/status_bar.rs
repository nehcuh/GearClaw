use gpui::*;
use crate::app::DesktopApp;
use crate::theme;
use crate::theme::Theme;

impl DesktopApp {
    pub fn render_status_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let current_mode = theme::mode(cx);
        div()
            .h(px(28.))
            .bg(theme::accent(cx))
            .flex()
            .items_center()
            .px(px(12.))
            .gap(px(16.))
            .text_xs()
            .text_color(gpui::white())
            .child(
                if self.is_loading {
                    "⏳ Processing..."
                } else {
                    "🟢 Ready"
                },
            )
            .child(
                div()
                    .id("toggle-skills")
                    .cursor_pointer()
                    .hover(|s| s.opacity(0.8))
                    .child(format!(
                        "🔧 Skills: {}",
                        if self.skills_on { "ON" } else { "OFF" }
                    ))
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.skills_on = !this.skills_on;
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .id("toggle-memory")
                    .cursor_pointer()
                    .hover(|s| s.opacity(0.8))
                    .child(format!(
                        "🧠 Memory: {}",
                        if self.memory_on { "ON" } else { "OFF" }
                    ))
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.memory_on = !this.memory_on;
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .id("toggle-security")
                    .cursor_pointer()
                    .hover(|s| s.opacity(0.8))
                    .child(format!(
                        "🛡️ Mode: {}",
                        if self.security_full { "Full" } else { "Safe" }
                    ))
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.security_full = !this.security_full;
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .id("toggle-theme")
                    .cursor_pointer()
                    .hover(|s| s.opacity(0.8))
                    .child(format!("🎨 Theme: {}", current_mode.label()))
                    .on_click(cx.listener(|_this, _event, window, cx| {
                        let current = theme::mode(cx);
                        let new_mode = current.next();
                        let appearance = window.appearance();
                        cx.set_global(Theme::for_appearance(appearance, new_mode));
                        cx.notify();
                    })),
            )
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
