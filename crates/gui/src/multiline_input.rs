use std::ops::Range;

use gpui::{
    App, Bounds, ClipboardItem, Context, CursorStyle, ElementId, ElementInputHandler, Entity,
    EntityInputHandler, FocusHandle, Focusable, GlobalElementId, LayoutId, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels, Point, SharedString, Style,
    TextRun, UTF16Selection, UnderlineStyle, Window, WrappedLine, div, fill, hsla, point,
    prelude::*, px, relative, size,
};
use unicode_segmentation::*;

use crate::theme;
use crate::text_input::{
    Backspace, CopyText, CutText, Delete, End, Home, Left, PasteText, Right, SelectAll, SelectLeft,
    SelectRight, ShowCharacterPalette,
};

// Actions for multiline input
gpui::actions!(multiline_input, [InsertNewline]);

pub struct MultiLineTextInput {
    focus_handle: FocusHandle,
    content: SharedString,
    placeholder: SharedString,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    is_selecting: bool,
    last_lines: Option<Vec<WrappedLine>>,
    last_bounds: Option<Bounds<Pixels>>,
    last_line_height: Option<Pixels>,
}

impl MultiLineTextInput {
    pub fn new(placeholder: &str, cx: &mut Context<Self>) -> Self {
        MultiLineTextInput {
            focus_handle: cx.focus_handle(),
            content: "".into(),
            placeholder: placeholder.to_string().into(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            is_selecting: false,
            last_lines: None,
            last_bounds: None,
            last_line_height: None,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn set_content(&mut self, text: &str, cx: &mut Context<Self>) {
        self.content = text.to_string().into();
        self.selected_range = text.len()..text.len();
        self.marked_range = None;
        cx.notify();
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.content = "".into();
        self.selected_range = 0..0;
        self.selection_reversed = false;
        self.marked_range = None;
        self.is_selecting = false;
        cx.notify();
    }

    fn insert_newline(&mut self, _: &InsertNewline, window: &mut Window, cx: &mut Context<Self>) {
        self.replace_text_in_range(None, "\n", window, cx);
    }

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.previous_boundary(self.cursor_offset()), cx);
        } else {
            self.move_to(self.selected_range.start, cx)
        }
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.next_boundary(self.selected_range.end), cx);
        } else {
            self.move_to(self.selected_range.end, cx)
        }
    }

    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.previous_boundary(self.cursor_offset()), cx);
    }

    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.next_boundary(self.cursor_offset()), cx);
    }

    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
        self.select_to(self.content.len(), cx)
    }

    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
    }

    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), cx);
    }

    fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.previous_boundary(self.cursor_offset()), cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }

    fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.next_boundary(self.cursor_offset()), cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }

    fn on_mouse_down(&mut self, event: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.is_selecting = true;
        if event.modifiers.shift {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        } else {
            self.move_to(self.index_for_mouse_position(event.position), cx)
        }
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.is_selecting {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        }
    }

    fn show_character_palette(
        &mut self,
        _: &ShowCharacterPalette,
        window: &mut Window,
        _: &mut Context<Self>,
    ) {
        window.show_character_palette();
    }

    fn paste(&mut self, _: &PasteText, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.replace_text_in_range(None, &text, window, cx);
        }
    }

    fn copy_text(&mut self, _: &CopyText, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
        }
    }

    fn cut_text(&mut self, _: &CutText, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
            self.replace_text_in_range(None, "", window, cx)
        }
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        cx.notify()
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.content.is_empty() && self.placeholder.is_empty() {
            return 0;
        }
        let (Some(bounds), Some(lines), Some(line_height)) = (
            self.last_bounds.as_ref(),
            self.last_lines.as_ref(),
            self.last_line_height,
        ) else {
            return 0;
        };
        if position.y < bounds.top() {
            return 0;
        }

        let mut line_origin = bounds.origin;
        let mut line_start_ix = 0;
        for line in lines {
            let line_height_px = line.size(line_height).height;
            let line_bottom = line_origin.y + line_height_px;
            if position.y > line_bottom {
                line_origin.y = line_bottom;
                line_start_ix += line.len() + 1;
                continue;
            }

            let position_within_line = position - line_origin;
            match line
                .layout
                .closest_index_for_position(position_within_line, line_height)
            {
                Ok(index_within_line) | Err(index_within_line) => {
                    return line_start_ix + index_within_line
                }
            }
        }

        line_start_ix.saturating_sub(1)
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        if self.selection_reversed {
            self.selected_range.start = offset
        } else {
            self.selected_range.end = offset
        };
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        cx.notify()
    }

    fn utf16_offset_to_utf8(text: &str, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        for ch in text.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        utf8_offset
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        Self::utf16_offset_to_utf8(&self.content, offset)
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;
        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }
        utf16_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content.len())
    }

    fn position_for_index(
        lines: &[WrappedLine],
        line_height: Pixels,
        index: usize,
    ) -> Option<Point<Pixels>> {
        let mut line_origin = point(px(0.), px(0.));
        let mut line_start_ix = 0;
        for line in lines {
            let line_end_ix = line_start_ix + line.len();
            if index < line_start_ix {
                break;
            } else if index > line_end_ix {
                line_origin.y += line.size(line_height).height;
                line_start_ix = line_end_ix + 1;
                continue;
            } else {
                let ix_within_line = index - line_start_ix;
                return line
                    .layout
                    .position_for_index(ix_within_line, line_height)
                    .map(|p| line_origin + p);
            }
        }
        None
    }
}

impl EntityInputHandler for MultiLineTextInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.content =
            (self.content[0..range.start].to_owned() + new_text + &self.content[range.end..])
                .into();
        self.selected_range = range.start + new_text.len()..range.start + new_text.len();
        self.marked_range.take();
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.content =
            (self.content[0..range.start].to_owned() + new_text + &self.content[range.end..])
                .into();
        if !new_text.is_empty() {
            self.marked_range = Some(range.start..range.start + new_text.len());
        } else {
            self.marked_range = None;
        }
        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|range_utf16| {
                let start = Self::utf16_offset_to_utf8(new_text, range_utf16.start);
                let end = Self::utf16_offset_to_utf8(new_text, range_utf16.end);
                range.start + start..range.start + end
            })
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());

        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let line_height = self.last_line_height?;
        let lines = self.last_lines.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        let start = Self::position_for_index(lines, line_height, range.start)?;
        let end = Self::position_for_index(lines, line_height, range.end)?;
        Some(Bounds::from_corners(
            point(bounds.left() + start.x, bounds.top() + start.y),
            point(bounds.left() + end.x, bounds.top() + end.y + line_height),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        Some(self.offset_to_utf16(self.index_for_mouse_position(point)))
    }
}

// Custom element for rendering the multiline text with cursor and selection
pub struct MultiLineTextElement {
    input: Entity<MultiLineTextInput>,
}

pub struct PrepaintState {
    lines: Vec<WrappedLine>,
    cursor: Option<PaintQuad>,
    selections: Vec<PaintQuad>,
    line_height: Pixels,
    bounds: Bounds<Pixels>,
}

impl IntoElement for MultiLineTextElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for MultiLineTextElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let content = input.content.clone();
        let selected_range = input.selected_range.clone();
        let cursor = input.cursor_offset();
        let style = window.text_style();

        let (display_text, text_color) = if content.is_empty() {
            (input.placeholder.clone(), hsla(0., 0., 0.5, 0.5))
        } else {
            (content, style.color)
        };

        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = if let Some(marked_range) = input.marked_range.as_ref() {
            vec![
                TextRun {
                    len: marked_range.start,
                    ..run.clone()
                },
                TextRun {
                    len: marked_range.end - marked_range.start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.0),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: display_text.len() - marked_range.end,
                    ..run
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect()
        } else {
            vec![run]
        };

        let font_size = style.font_size.to_pixels(window.rem_size());
        let line_height = style
            .line_height
            .to_pixels(font_size.into(), window.rem_size());

        let lines = window
            .text_system()
            .shape_text(display_text, font_size, &runs, Some(bounds.size.width), None)
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<_>>();

        let selections = if selected_range.is_empty() {
            Vec::new()
        } else {
            let mut quads = Vec::new();
            let mut line_origin = bounds.origin;
            let mut line_start_ix = 0;
            for line in &lines {
                let line_end_ix = line_start_ix + line.len();
                if selected_range.end <= line_start_ix {
                    break;
                }
                if selected_range.start > line_end_ix {
                    line_origin.y += line.size(line_height).height;
                    line_start_ix = line_end_ix + 1;
                    continue;
                }

                let line_sel_start = selected_range.start.saturating_sub(line_start_ix);
                let line_sel_end = selected_range.end.min(line_end_ix) - line_start_ix;

                if line_sel_end > line_sel_start {
                    let boundary_indices = line
                        .layout
                        .wrap_boundaries
                        .iter()
                        .map(|b| {
                            line.layout.unwrapped_layout.runs[b.run_ix].glyphs[b.glyph_ix].index
                        })
                        .collect::<Vec<_>>();
                    let mut seg_start = 0usize;
                    for (seg_idx, seg_end) in boundary_indices
                        .iter()
                        .copied()
                        .chain(std::iter::once(line.len()))
                        .enumerate()
                    {
                        let sel_start = line_sel_start.max(seg_start);
                        let sel_end = line_sel_end.min(seg_end);
                        if sel_end > sel_start {
                            let start_x = line
                                .layout
                                .position_for_index(sel_start, line_height)
                                .map(|p| p.x)
                                .unwrap_or(px(0.));
                            let end_x = line
                                .layout
                                .position_for_index(sel_end, line_height)
                                .map(|p| p.x)
                                .unwrap_or(line.layout.width());
                            let y = line_origin.y + line_height * seg_idx;
                            quads.push(fill(
                                Bounds::from_corners(
                                    point(bounds.left() + start_x, y),
                                    point(bounds.left() + end_x, y + line_height),
                                ),
                                gpui::rgba(0x3311ff30),
                            ));
                        }
                        seg_start = seg_end;
                    }
                }

                line_origin.y += line.size(line_height).height;
                line_start_ix = line_end_ix + 1;
            }
            quads
        };

        let cursor = if selected_range.is_empty() && !lines.is_empty() {
            if let Some(pos) = MultiLineTextInput::position_for_index(&lines, line_height, cursor) {
                Some(fill(
                    Bounds::new(
                        point(bounds.left() + pos.x, bounds.top() + pos.y),
                        size(px(2.), line_height),
                    ),
                    gpui::blue(),
                ))
            } else {
                None
            }
        } else {
            None
        };

        PrepaintState {
            lines,
            cursor,
            selections,
            line_height,
            bounds,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(prepaint.bounds, self.input.clone()),
            cx,
        );

        for selection in prepaint.selections.drain(..) {
            window.paint_quad(selection);
        }

        let text_style = window.text_style();
        let mut line_origin = prepaint.bounds.origin;
        for line in &prepaint.lines {
            line.paint_background(
                line_origin,
                prepaint.line_height,
                text_style.text_align,
                Some(prepaint.bounds),
                window,
                cx,
            )
            .log_err();
            line.paint(
                line_origin,
                prepaint.line_height,
                text_style.text_align,
                Some(prepaint.bounds),
                window,
                cx,
            )
            .log_err();
            line_origin.y += line.size(prepaint.line_height).height;
        }

        if focus_handle.is_focused(window) {
            if let Some(cursor) = prepaint.cursor.take() {
                window.paint_quad(cursor);
            }
        }

        self.input.update(cx, |input, _cx| {
            input.last_lines = Some(prepaint.lines.clone());
            input.last_bounds = Some(prepaint.bounds);
            input.last_line_height = Some(prepaint.line_height);
        });
    }
}

impl Render for MultiLineTextInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let line_count = self.content.chars().filter(|c| *c == '\n').count().max(0) + 1;
        let content_height = (line_count as f32 * 20.0 + 12.0).max(140.0);

        div()
            .flex()
            .key_context("MultiLineInput")
            .track_focus(&self.focus_handle(cx))
            .cursor(CursorStyle::IBeam)
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::show_character_palette))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut_text))
            .on_action(cx.listener(Self::copy_text))
            .on_action(cx.listener(Self::insert_newline))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .line_height(px(20.))
            .text_size(px(14.))
            .text_color(theme::text(cx))
            .child(
                div()
                    .h(px(160.))
                    .w_full()
                    .px(px(8.))
                    .py(px(6.))
                    .bg(theme::input_bg(cx))
                    .rounded_md()
                    .overflow_y_scroll()
                    .child(
                        div()
                            .h(px(content_height))
                            .child(MultiLineTextElement { input: cx.entity() }),
                    ),
            )
    }
}

impl Focusable for MultiLineTextInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
