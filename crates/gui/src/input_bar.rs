use gpui::*;
use crate::app::DesktopApp;
use crate::theme;

impl DesktopApp {
    pub fn render_input_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let accent_hover = theme::accent_hover(cx);
        let text_muted = theme::text_muted(cx);
        let badge_bg = theme::sidebar_bg(cx);
        let border = theme::border(cx);
        let skills_count = self.loaded_skills.len();
        let mcp_count = self.mcp_configured.iter().filter(|(_, e)| *e).count();
        let memory_label = if self.config_memory_enabled { "ON" } else { "OFF" };

        div()
            .border_t_1()
            .border_color(border)
            .px(px(12.))
            .pt(px(6.))
            .pb(px(12.))
            .flex()
            .flex_col()
            .gap(px(6.))
            // Capability badges row
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(6.))
                    .items_center()
                    .child(
                        div()
                            .px(px(8.))
                            .py(px(2.))
                            .bg(badge_bg)
                            .border_1()
                            .border_color(border)
                            .rounded_md()
                            .text_xs()
                            .text_color(text_muted)
                            .child(format!("🛠 Skills:{}", skills_count)),
                    )
                    .child(
                        div()
                            .px(px(8.))
                            .py(px(2.))
                            .bg(badge_bg)
                            .border_1()
                            .border_color(border)
                            .rounded_md()
                            .text_xs()
                            .text_color(text_muted)
                            .child(format!("🧠 Memory:{}", memory_label)),
                    )
                    .child(
                        div()
                            .px(px(8.))
                            .py(px(2.))
                            .bg(badge_bg)
                            .border_1()
                            .border_color(border)
                            .rounded_md()
                            .text_xs()
                            .text_color(text_muted)
                            .child(format!("🔌 MCP:{}", mcp_count)),
                    ),
            )
            // Input row
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(8.))
                    .items_end()
                    .child(
                        // Text input area - takes most of the space
                        div()
                            .flex_grow()
                            .child(self.input.clone()),
                    )
                    .child(
                        // Send or Stop button
                        if self.is_loading {
                            div()
                                .id("stop-btn")
                                .px(px(16.))
                                .py(px(6.))
                                .bg(theme::stop_button(cx))
                                .rounded_md()
                                .cursor_pointer()
                                .text_sm()
                                .text_color(gpui::white())
                                .flex()
                                .items_center()
                                .child("Stop")
                                .hover(|s| s.opacity(0.8))
                                .on_click(cx.listener(|this, _event, _window, cx| {
                                    this.on_stop(cx);
                                }))
                        } else {
                            div()
                                .id("send-btn")
                                .px(px(16.))
                                .py(px(6.))
                                .bg(theme::accent(cx))
                                .rounded_md()
                                .cursor_pointer()
                                .text_sm()
                                .text_color(gpui::white())
                                .flex()
                                .items_center()
                                .child("Send")
                                .hover(move |s| s.bg(accent_hover))
                                .on_click(cx.listener(|this, _event, window, cx| {
                                    this.on_send(window, cx);
                                }))
                        },
                    ),
            )
    }
}
