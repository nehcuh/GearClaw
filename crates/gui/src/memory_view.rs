use gpui::prelude::FluentBuilder;
use gpui::*;

use crate::app::DesktopApp;
use crate::theme;

impl DesktopApp {
    pub fn render_memory(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let text_muted = theme::text_muted(cx);
        let border = theme::border(cx);
        let card_bg = theme::sidebar_bg(cx);
        let accent_hover = theme::accent_hover(cx);
        let is_searching = self.memory_is_searching;
        let has_results = !self.memory_results.is_empty();

        div()
            .id("memory-scroll")
            .flex_grow()
            .overflow_y_scroll()
            .p(px(24.))
            .flex()
            .flex_col()
            .gap(px(16.))
            // Title
            .child(div().text_xl().child("🧠 Memory Search"))
            .child(
                div()
                    .text_sm()
                    .text_color(text_muted)
                    .child("Semantic search over indexed workspace files. Requires a configured LLM API key."),
            )
            // Search bar
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(8.))
                    .items_center()
                    .child(div().flex_grow().child(self.memory_search_input.clone()))
                    .child(
                        div()
                            .id("memory-search-btn")
                            .px(px(14.))
                            .py(px(6.))
                            .bg(theme::accent(cx))
                            .rounded_md()
                            .cursor_pointer()
                            .text_sm()
                            .text_color(gpui::white())
                            .opacity(if is_searching { 0.5 } else { 1.0 })
                            .child(if is_searching { "Searching..." } else { "Search" })
                            .hover(move |s| s.bg(accent_hover))
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.on_memory_search(cx);
                            })),
                    ),
            )
            // Loading indicator
            .when(is_searching, |el| {
                el.child(
                    div()
                        .text_sm()
                        .text_color(text_muted)
                        .child("⏳ Embedding query and searching..."),
                )
            })
            // Result count
            .when(has_results && !is_searching, |el| {
                el.child(
                    div()
                        .text_xs()
                        .text_color(text_muted)
                        .child(format!("{} results", self.memory_results.len())),
                )
            })
            // Results list
            .children(self.memory_results.iter().map(|hit| {
                let score_str = format!("{:.3}", hit.score);
                let line_str = if hit.line > 0 {
                    format!(":{}", hit.line)
                } else {
                    String::new()
                };
                let location = format!("{}{}", hit.path, line_str);
                div()
                    .border_1()
                    .border_color(border)
                    .rounded_md()
                    .bg(card_bg)
                    .px(px(12.))
                    .py(px(10.))
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_between()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(location),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(text_muted)
                                    .child(format!("score: {}", score_str)),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(text_muted)
                            .child(hit.preview.clone()),
                    )
            }))
            // Empty state
            .when(!has_results && !is_searching, |el| {
                el.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .pt(px(48.))
                        .flex_col()
                        .gap(px(8.))
                        .child(
                            div()
                                .text_xl()
                                .text_color(text_muted)
                                .child("🔍"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(text_muted)
                                .child("Enter a query above to search indexed memories"),
                        ),
                )
            })
    }
}
