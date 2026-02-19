use gpui::*;

use crate::app::DesktopApp;
use crate::theme;

impl DesktopApp {
    pub fn render_monitor(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let text_muted = theme::text_muted(cx);
        let border = theme::border(cx);
        let card_bg = theme::sidebar_bg(cx);
        let updated = self
            .status_updated_at
            .clone()
            .unwrap_or_else(|| "Never".to_string());

        div()
            .id("monitor-scroll")
            .flex_grow()
            .overflow_y_scroll()
            .p(px(24.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(div().text_xl().child("🩺 System Monitor"))
            .child(
                div()
                    .text_sm()
                    .text_color(text_muted)
                    .child("Status data is placeholder until gateway reporting is wired."),
            )
            .child(self.render_status_card("Gateway", &self.status_gateway, card_bg, border, cx))
            .child(self.render_status_card(
                "Channels",
                &self.status_channels,
                card_bg,
                border,
                cx,
            ))
            .child(self.render_status_card("LLM", &self.status_llm, card_bg, border, cx))
            .child(self.render_status_card("Memory", &self.status_memory, card_bg, border, cx))
            .child(self.render_status_card("MCP", &self.status_mcp, card_bg, border, cx))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        div()
                            .id("monitor-refresh")
                            .px(px(16.))
                            .py(px(6.))
                            .bg(theme::accent(cx))
                            .rounded_md()
                            .cursor_pointer()
                            .text_sm()
                            .text_color(gpui::white())
                            .child("Refresh")
                            .hover(|s| s.opacity(0.85))
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.refresh_status(cx);
                            })),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(text_muted)
                            .child(format!("Last update: {}", updated)),
                    ),
            )
    }

    fn render_status_card(
        &self,
        title: &str,
        value: &str,
        card_bg: gpui::Hsla,
        border: gpui::Hsla,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .border_1()
            .border_color(border)
            .rounded_md()
            .bg(card_bg)
            .px(px(12.))
            .py(px(10.))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(div().text_sm().child(title.to_string()))
            .child(div().text_sm().child(value.to_string()))
    }
}
