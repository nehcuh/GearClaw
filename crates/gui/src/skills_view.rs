use gpui::prelude::FluentBuilder;
use gpui::*;

use crate::app::DesktopApp;
use crate::theme;

impl DesktopApp {
    pub fn render_skills(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let text_muted = theme::text_muted(cx);
        let border = theme::border(cx);
        let card_bg = theme::sidebar_bg(cx);

        div()
            .id("skills-scroll")
            .flex_grow()
            .overflow_y_scroll()
            .p(px(24.))
            .flex()
            .flex_col()
            .gap(px(16.))
            // Title
            .child(div().text_xl().child("🛠 Skills"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(16.))
                    .child(
                        div()
                            .text_sm()
                            .text_color(text_muted)
                            .child(format!("{} skills loaded", self.loaded_skills.len())),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(text_muted)
                            .child(format!("from {}", self.skills_path_label)),
                    ),
            )
            // Skills list
            .when(self.loaded_skills.is_empty(), |el| {
                el.child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .pt(px(48.))
                        .gap(px(8.))
                        .child(div().text_xl().text_color(text_muted).child("📭"))
                        .child(
                            div()
                                .text_sm()
                                .text_color(text_muted)
                                .child("No skills found. Add SKILL.md files to your skills directory."),
                        ),
                )
            })
            .children(self.loaded_skills.iter().enumerate().map(|(i, (name, desc, path))| {
                div()
                    .id(ElementId::Name(format!("skill-{}", i).into()))
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
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(name.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(text_muted)
                            .child(desc.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(text_muted)
                            .child(path.clone()),
                    )
            }))
    }
}
