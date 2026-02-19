use gpui::*;
use crate::app::DesktopApp;
use crate::log_store;
use crate::theme;

impl DesktopApp {
    pub fn render_log_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let entries = log_store::log_entries(cx);
        let filter_text = self.log_filter.read(cx).content().to_lowercase();
        let level_filter = self.log_level_filter;
        let filtered_entries: Vec<String> = entries
            .into_iter()
            .filter(|entry| {
                if !filter_text.is_empty() && !entry.to_lowercase().contains(&filter_text) {
                    return false;
                }
                match level_filter {
                    crate::app::LogLevelFilter::All => true,
                    crate::app::LogLevelFilter::Info => entry.contains(" INFO "),
                    crate::app::LogLevelFilter::Warn => entry.contains(" WARN "),
                    crate::app::LogLevelFilter::Error => entry.contains(" ERROR "),
                }
            })
            .collect();
        let border = theme::border(cx);
        let text_muted = theme::text_muted(cx);
        let bg = theme::sidebar_bg(cx);

        div()
            .id("log-panel")
            .h(px(200.))
            .border_t_1()
            .border_color(border)
            .bg(bg)
            .flex()
            .flex_col()
            .child(
                // Header
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px(px(12.))
                    .py(px(4.))
                    .border_b_1()
                    .border_color(border)
                    .child(
                        div()
                            .text_xs()
                            .text_color(text_muted)
                            .child(format!("📜 Logs ({})", filtered_entries.len())),
                    )
                    .child(
                        div()
                            .id("close-logs")
                            .text_xs()
                            .cursor_pointer()
                            .text_color(text_muted)
                            .hover(|s| s.opacity(0.7))
                            .child("✕")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.show_logs = false;
                                cx.notify();
                            })),
                    ),
            )
            .child(
                // Filters
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(8.))
                    .px(px(12.))
                    .py(px(6.))
                    .border_b_1()
                    .border_color(border)
                    .child(div().w(px(240.)).child(self.log_filter.clone()))
                    .child(
                        div()
                            .id("log-level-filter")
                            .text_xs()
                            .cursor_pointer()
                            .text_color(text_muted)
                            .hover(|s| s.opacity(0.7))
                            .child(format!("Level: {}", level_filter.label()))
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.log_level_filter = this.log_level_filter.next();
                                cx.notify();
                            })),
                    ),
            )
            .child(
                // Log entries
                div()
                    .id("log-scroll")
                    .flex_grow()
                    .overflow_y_scroll()
                    .px(px(8.))
                    .py(px(4.))
                    .text_xs()
                    .font_family("Menlo")
                    .text_color(text_muted)
                    .children(
                        filtered_entries.iter().enumerate().map(|(i, entry)| {
                            div()
                                .id(ElementId::Name(format!("log-{}", i).into()))
                                .py(px(1.))
                                .child(entry.clone())
                        }),
                    ),
            )
    }
}
