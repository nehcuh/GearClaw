use gpui::*;
use gpui::prelude::FluentBuilder;
use crate::app::DesktopApp;
use crate::theme;

impl DesktopApp {
    pub fn render_chat(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let user_bubble = theme::user_bubble(cx);
        let assistant_bubble = theme::assistant_bubble(cx);
        let text_color = theme::text(cx);
        let text_muted = theme::text_muted(cx);
        let error_color = theme::error_color(cx);

        div()
            .id("chat-scroll")
            .flex_grow()
            .overflow_y_scroll()
            .track_scroll(&self.scroll_handle)
            .p(px(16.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .children(
                self.messages.iter().enumerate().map(|(i, msg)| {
                    let is_user = msg.role == "user";
                    let is_error = msg.role == "error";
                    let content_for_copy = msg.content.clone();

                    div()
                        .id(ElementId::Name(format!("msg-{}", i).into()))
                        .flex()
                        .flex_col()
                        .when(is_user, |el| el.items_end())
                        .child(
                            div()
                                .group("msg-group")
                                .relative()
                                .max_w(px(600.))
                                .px(px(14.))
                                .py(px(10.))
                                .rounded_lg()
                                .text_sm()
                                .when(is_user, |el| {
                                    el.bg(user_bubble)
                                        .text_color(gpui::white())
                                })
                                .when(!is_user && !is_error, |el| {
                                    el.bg(assistant_bubble)
                                        .text_color(text_color)
                                })
                                .when(is_error, |el| {
                                    el.bg(error_color)
                                        .text_color(gpui::white())
                                })
                                .child(msg.content.clone())
                                .child(
                                    // Copy button - shown on hover
                                    div()
                                        .id(ElementId::Name(format!("copy-{}", i).into()))
                                        .absolute()
                                        .top(px(-8.))
                                        .right(px(-8.))
                                        .px(px(6.))
                                        .py(px(2.))
                                        .rounded_md()
                                        .bg(assistant_bubble)
                                        .border_1()
                                        .border_color(text_muted)
                                        .text_xs()
                                        .cursor_pointer()
                                        .opacity(0.0)
                                        .group_hover("msg-group", |s| s.opacity(1.0))
                                        .child("📋")
                                        .on_click(cx.listener(move |_this, _event, _window, cx| {
                                            cx.write_to_clipboard(ClipboardItem::new_string(
                                                content_for_copy.clone(),
                                            ));
                                        })),
                                ),
                        )
                }),
            )
            .when(self.is_loading, |el| {
                el.child(
                    div()
                        .flex()
                        .child(
                            div()
                                .px(px(14.))
                                .py(px(10.))
                                .rounded_lg()
                                .bg(assistant_bubble)
                                .text_sm()
                                .text_color(text_muted)
                                .child("Thinking..."),
                        ),
                )
            })
            .when(self.messages.is_empty() && !self.is_loading, |el| {
                el.child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .flex_grow()
                        .gap(px(8.))
                        .child(
                            div()
                                .text_xl()
                                .text_color(text_muted)
                                .child("🦞 GearClaw"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(text_muted)
                                .child("Type a message to start chatting"),
                        ),
                )
            })
    }
}
