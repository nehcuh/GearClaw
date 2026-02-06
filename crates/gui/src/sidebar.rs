use gpui::*;
use gpui::prelude::FluentBuilder;
use crate::app::{DesktopApp, ViewMode};
use crate::theme;

impl DesktopApp {
    pub fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let accent_hover = theme::accent_hover(cx);
        let sidebar_hover = theme::sidebar_hover(cx);
        let sidebar_active_bg = theme::sidebar_active(cx);
        let text_c = theme::text(cx);
        let text_muted_c = theme::text_muted(cx);

        div()
            .w(px(240.))
            .bg(theme::sidebar_bg(cx))
            .border_r_1()
            .border_color(theme::border(cx))
            .flex()
            .flex_col()
            .child(
                // Header + New Chat button
                div()
                    .p(px(12.))
                    .child(
                        div()
                            .id("new-chat")
                            .px(px(12.))
                            .py(px(8.))
                            .bg(theme::accent(cx))
                            .rounded_md()
                            .cursor_pointer()
                            .text_sm()
                            .text_color(gpui::white())
                            .flex()
                            .items_center()
                            .justify_center()
                            .child("+ New Chat")
                            .hover(move |s| s.bg(accent_hover))
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.new_session(cx);
                            })),
                    ),
            )
            .child(
                // Session list
                div()
                    .flex_grow()
                    .overflow_hidden()
                    .px(px(8.))
                    .py(px(4.))
                    .children(
                        self.sessions.iter().enumerate().map(|(i, session)| {
                            let is_active = i == self.active_session;
                            let session_label = if session.is_empty() {
                                format!("Chat {}", i + 1)
                            } else {
                                session.clone()
                            };
                            div()
                                .id(ElementId::Name(format!("session-{}", i).into()))
                                .px(px(12.))
                                .py(px(6.))
                                .my(px(1.))
                                .rounded_md()
                                .cursor_pointer()
                                .text_sm()
                                .text_color(if is_active {
                                    text_c
                                } else {
                                    text_muted_c
                                })
                                .when(is_active, move |el: Stateful<Div>| el.bg(sidebar_active_bg))
                                .hover(move |s: StyleRefinement| s.bg(sidebar_hover))
                                .child(session_label)
                                .on_click(cx.listener(move |this, _event, _window, cx| {
                                    this.switch_session(i, cx);
                                }))
                        }),
                    ),
            )
            .child(
                // Settings button
                div()
                    .px(px(12.))
                    .py(px(4.))
                    .child(
                        div()
                            .id("settings-btn")
                            .px(px(12.))
                            .py(px(6.))
                            .rounded_md()
                            .cursor_pointer()
                            .text_sm()
                            .text_color(text_muted_c)
                            .hover(move |s: StyleRefinement| s.bg(sidebar_hover))
                            .when(self.view_mode == ViewMode::Settings, move |el: Stateful<Div>| el.bg(sidebar_active_bg))
                            .child("⚙️ Settings")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.view_mode = if this.view_mode == ViewMode::Settings {
                                    ViewMode::Chat
                                } else {
                                    ViewMode::Settings
                                };
                                cx.notify();
                            })),
                    ),
            )
            .child(
                // Footer
                div()
                    .p(px(12.))
                    .border_t_1()
                    .border_color(theme::border(cx))
                    .text_sm()
                    .text_color(text_muted_c)
                    .child("GearClaw v0.1.0"),
            )
    }
}
