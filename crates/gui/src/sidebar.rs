use gpui::*;
use gpui::prelude::FluentBuilder;
use crate::app::{DesktopApp, ViewMode};
use crate::theme;

impl DesktopApp {
    pub fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let sidebar_hover = theme::sidebar_hover(cx);
        let sidebar_active_bg = theme::sidebar_active(cx);
        let text_c = theme::text(cx);
        let text_muted_c = theme::text_muted(cx);
        let border_c = theme::border(cx);
        let accent_hover = theme::accent_hover(cx);
        let current_mode = self.view_mode;

        div()
            .w(px(220.))
            .bg(theme::sidebar_bg(cx))
            .border_r_1()
            .border_color(border_c)
            .flex()
            .flex_col()
            // Nav buttons (top)
            .child(
                div()
                    .p(px(8.))
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .child(self.nav_btn("💬", "Chat", ViewMode::Chat, current_mode, sidebar_active_bg, sidebar_hover, text_c, text_muted_c, cx))
                    .child(self.nav_btn("🧠", "Memory", ViewMode::Memory, current_mode, sidebar_active_bg, sidebar_hover, text_c, text_muted_c, cx))
                    .child(self.nav_btn("🔌", "MCP", ViewMode::Mcp, current_mode, sidebar_active_bg, sidebar_hover, text_c, text_muted_c, cx))
                    .child(self.nav_btn("🛠", "Skills", ViewMode::Skills, current_mode, sidebar_active_bg, sidebar_hover, text_c, text_muted_c, cx))
                    .child(self.nav_btn("🩺", "Monitor", ViewMode::Monitor, current_mode, sidebar_active_bg, sidebar_hover, text_c, text_muted_c, cx))
                    .child(self.nav_btn("⚙️", "Settings", ViewMode::Settings, current_mode, sidebar_active_bg, sidebar_hover, text_c, text_muted_c, cx)),
            )
            // Divider
            .child(div().h(px(1.)).mx(px(8.)).bg(border_c))
            // Context sub-panel
            .child(self.render_context_panel(current_mode, sidebar_active_bg, sidebar_hover, accent_hover, text_c, text_muted_c, cx))
            // Footer
            .child(
                div()
                    .p(px(12.))
                    .border_t_1()
                    .border_color(border_c)
                    .text_xs()
                    .text_color(text_muted_c)
                    .child("GearClaw v0.1.0"),
            )
    }

    fn nav_btn(
        &self,
        icon: &'static str,
        label: &'static str,
        target: ViewMode,
        current: ViewMode,
        active_bg: gpui::Rgba,
        hover_bg: gpui::Rgba,
        text_c: gpui::Rgba,
        text_muted_c: gpui::Rgba,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_active = current == target;
        div()
            .id(ElementId::Name(format!("nav-{}", label).into()))
            .px(px(10.))
            .py(px(6.))
            .rounded_md()
            .cursor_pointer()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.))
            .text_sm()
            .text_color(if is_active { text_c } else { text_muted_c })
            .when(is_active, move |el: Stateful<Div>| el.bg(active_bg))
            .hover(move |s: StyleRefinement| s.bg(hover_bg))
            .child(icon)
            .child(label)
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.view_mode = target;
                cx.notify();
            }))
    }

    fn render_context_panel(
        &self,
        mode: ViewMode,
        sidebar_active_bg: gpui::Rgba,
        sidebar_hover: gpui::Rgba,
        accent_hover: gpui::Rgba,
        text_c: gpui::Rgba,
        text_muted_c: gpui::Rgba,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        match mode {
            ViewMode::Chat => {
                div()
                    .flex_grow()
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .px(px(12.))
                            .py(px(6.))
                            .child(
                                div()
                                    .id("new-chat")
                                    .px(px(10.))
                                    .py(px(6.))
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
                        div()
                            .flex_grow()
                            .overflow_hidden()
                            .px(px(8.))
                            .py(px(4.))
                            .children(self.sessions.iter().enumerate().map(|(i, session)| {
                                let is_active = i == self.active_session;
                                let label = if session.is_empty() {
                                    format!("Chat {}", i + 1)
                                } else {
                                    session.clone()
                                };
                                div()
                                    .id(ElementId::Name(format!("session-{}", i).into()))
                                    .px(px(10.))
                                    .py(px(5.))
                                    .my(px(1.))
                                    .rounded_md()
                                    .cursor_pointer()
                                    .text_sm()
                                    .text_color(if is_active { text_c } else { text_muted_c })
                                    .when(is_active, move |el: Stateful<Div>| el.bg(sidebar_active_bg))
                                    .hover(move |s: StyleRefinement| s.bg(sidebar_hover))
                                    .child(label)
                                    .on_click(cx.listener(move |this, _event, _window, cx| {
                                        this.switch_session(i, cx);
                                    }))
                            }))
                    )
            }
            ViewMode::Mcp => {
                div()
                    .flex_grow()
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .px(px(12.))
                            .py(px(8.))
                            .text_xs()
                            .text_color(text_muted_c)
                            .child(format!("{} configured", self.mcp_configured.len())),
                    )
                    .children(self.mcp_configured.iter().map(|(name, enabled)| {
                        let icon = if *enabled { "● " } else { "○ " };
                        div()
                            .px(px(12.))
                            .py(px(4.))
                            .text_xs()
                            .text_color(text_muted_c)
                            .child(format!("{}{}", icon, name))
                    }))
            }
            ViewMode::Skills => {
                div()
                    .flex_grow()
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .px(px(12.))
                            .py(px(8.))
                            .text_xs()
                            .text_color(text_muted_c)
                            .child(format!("{} skills loaded", self.loaded_skills.len())),
                    )
                    .children(self.loaded_skills.iter().map(|(name, _desc, _path)| {
                        div()
                            .px(px(12.))
                            .py(px(4.))
                            .text_xs()
                            .text_color(text_muted_c)
                            .child(format!("▸ {}", name))
                    }))
            }
            _ => {
                // Memory, Monitor, Settings: minimal panel
                div().flex_grow()
            }
        }
    }
}
