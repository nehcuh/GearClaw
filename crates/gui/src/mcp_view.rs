use gpui::prelude::FluentBuilder;
use gpui::*;

use crate::app::DesktopApp;
use crate::theme;

impl DesktopApp {
    pub fn render_mcp(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let text_muted = theme::text_muted(cx);
        let border = theme::border(cx);
        let card_bg = theme::sidebar_bg(cx);
        let accent_hover = theme::accent_hover(cx);

        let enabled_count = self.mcp_configured.iter().filter(|(_, e)| *e).count();

        div()
            .id("mcp-scroll")
            .flex_grow()
            .overflow_y_scroll()
            .p(px(24.))
            .flex()
            .flex_col()
            .gap(px(16.))
            // Title
            .child(div().text_xl().child("🔌 MCP Servers"))
            .child(
                div()
                    .text_sm()
                    .text_color(text_muted)
                    .child(format!(
                        "{} configured, {} enabled",
                        self.mcp_configured.len(),
                        enabled_count
                    )),
            )
            // Configured servers section
            .child(div().text_sm().font_weight(FontWeight::SEMIBOLD).child("Configured Servers"))
            .when(self.mcp_configured.is_empty(), |el| {
                el.child(
                    div()
                        .text_sm()
                        .text_color(text_muted)
                        .child("No servers configured. Use the search below or edit ~/.gearclaw/config.toml."),
                )
            })
            .children(self.mcp_configured.iter().map(|(name, enabled)| {
                let label = if *enabled { "✅ Enabled" } else { "⏸ Disabled" };
                div()
                    .border_1()
                    .border_color(border)
                    .rounded_md()
                    .bg(card_bg)
                    .px(px(12.))
                    .py(px(8.))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .child(div().text_sm().child(name.clone())),
                    )
                    .child(div().text_xs().text_color(text_muted).child(label))
            }))
            // Separator
            .child(div().h(px(1.)).bg(border).my(px(4.)))
            // Registry search section
            .child(div().text_sm().font_weight(FontWeight::SEMIBOLD).child("Registry Search"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(8.))
                    .items_center()
                    .child(div().flex_grow().child(self.mcp_search_input.clone()))
                    .child(
                        div()
                            .id("mcp-search-btn")
                            .px(px(14.))
                            .py(px(6.))
                            .bg(theme::accent(cx))
                            .rounded_md()
                            .cursor_pointer()
                            .text_sm()
                            .text_color(gpui::white())
                            .child("Search")
                            .hover(move |s| s.bg(accent_hover))
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.on_mcp_search(cx);
                            })),
                    ),
            )
            // Search results
            .when(!self.mcp_registry_results.is_empty(), |el| {
                el.child(
                    div()
                        .text_xs()
                        .text_color(text_muted)
                        .child(format!("{} results", self.mcp_registry_results.len())),
                )
            })
            .children(self.mcp_registry_results.iter().map(|entry| {
                let install_label = entry.install_method.label().to_string();
                div()
                    .border_1()
                    .border_color(border)
                    .rounded_md()
                    .bg(card_bg)
                    .px(px(12.))
                    .py(px(10.))
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_between()
                            .child(div().text_sm().font_weight(FontWeight::SEMIBOLD).child(entry.name))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(text_muted)
                                    .child(format!("[{}]", install_label)),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(text_muted)
                            .child(entry.description),
                    )
                    .when(entry.required_env.len() > 0, |el| {
                        el.child(
                            div()
                                .text_xs()
                                .text_color(text_muted)
                                .child(format!("Requires env: {}", entry.required_env.join(", "))),
                        )
                    })
                    .when_some(entry.notes, |el, notes| {
                        el.child(div().text_xs().text_color(text_muted).child(format!("Note: {}", notes)))
                    })
                    .child(
                        div()
                            .text_xs()
                            .text_color(text_muted)
                            .child(format!("Package: {}", entry.package)),
                    )
            }))
    }
}
