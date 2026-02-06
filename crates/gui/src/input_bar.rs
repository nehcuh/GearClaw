use gpui::*;
use crate::app::DesktopApp;
use crate::theme;

impl DesktopApp {
    pub fn render_input_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let accent_hover = theme::accent_hover(cx);
        div()
            .border_t_1()
            .border_color(theme::border(cx))
            .p(px(12.))
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
            )
    }
}
