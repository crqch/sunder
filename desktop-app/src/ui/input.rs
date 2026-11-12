use crate::app::{SearchComplete, SearchNext, SearchOpenSuggestions, SearchPrevious};
use crate::assets::{Icon, icon};
use crate::theme::{Theme, text as type_scale, theme};
use gpui::{
    App, Bounds, ClipboardItem, Context, CursorStyle, ElementId, ElementInputHandler, Entity,
    EntityInputHandler, EventEmitter, FocusHandle, Focusable, GlobalElementId, InspectorElementId,
    IntoElement, KeyBinding, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    PaintQuad, Pixels, Point, Render, ShapedLine, SharedString, Style, TextRun, UTF16Selection,
    UnderlineStyle, Window, actions, div, fill, font, point, prelude::*, px, relative, size,
};
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

actions!(
    text_input,
    [
        Backspace,
        BackspaceWord,
        Delete,
        DeleteWord,
        Left,
        Right,
        Up,
        Down,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        Paste,
        Cut,
        Copy,
        Submit,
        ShowCharacterPalette,
    ]
);

#[derive(Clone, Debug)]
pub enum TextInputEvent {
    Change,
    Submit,
    /// Backspace pressed while the field is empty. Used by token inputs to
    /// remove the previous chip.
    BackspaceEmpty,
}

impl EventEmitter<TextInputEvent> for TextInput {}

const BULLET: char = '•';

/// Characters that make up a word for ctrl/alt word deletion.
fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '-'
}

/// Start of the word before `cursor`, skipping whitespace and punctuation.
fn previous_word_boundary(text: &str, cursor: usize) -> usize {
    let mut boundary = cursor.min(text.len());
    let mut state = 0u8; // 0 = whitespace, 1 = word, 2 = punctuation
    for (index, ch) in text[..boundary].char_indices().rev() {
        match state {
            0 if ch.is_whitespace() => boundary = index,
            0 if is_word_char(ch) => {
                state = 1;
                boundary = index;
            }
            0 => {
                state = 2;
                boundary = index;
            }
            1 if is_word_char(ch) => boundary = index,
            1 => break,
            _ if !ch.is_whitespace() && !is_word_char(ch) => boundary = index,
            _ => break,
        }
    }
    boundary
}

/// End of the word after `cursor`, skipping whitespace and punctuation.
fn next_word_boundary(text: &str, cursor: usize) -> usize {
    let start = cursor.min(text.len());
    let mut boundary = start;
    let mut state = 0u8; // 0 = whitespace, 1 = word, 2 = punctuation
    for (offset, ch) in text[start..].char_indices() {
        let index = start + offset;
        let next = index + ch.len_utf8();
        match state {
            0 if ch.is_whitespace() => boundary = next,
            0 if is_word_char(ch) => {
                state = 1;
                boundary = next;
            }
            0 => {
                state = 2;
                boundary = next;
            }
            1 if is_word_char(ch) => boundary = next,
            1 => break,
            _ if !ch.is_whitespace() && !is_word_char(ch) => boundary = next,
            _ => break,
        }
    }
    boundary
}

#[derive(Clone, Copy)]
pub struct InputStyle {
    pub height: f32,
    pub font_size: f32,
    pub padding_x: f32,
    pub filled: bool,
    pub multiline: bool,
    /// Transparent, borderless field used inside a composed bar.
    pub bare: bool,
    /// Optional extra key context, e.g. `SearchInput` for the power search box.
    pub key_context: &'static str,
}

impl Default for InputStyle {
    fn default() -> Self {
        Self {
            height: 40.0,
            font_size: type_scale::BODY,
            padding_x: 12.0,
            filled: true,
            multiline: false,
            bare: false,
            key_context: "TextInput",
        }
    }
}

impl InputStyle {
    pub fn compact() -> Self {
        Self {
            height: 34.0,
            font_size: type_scale::SMALL,
            padding_x: 10.0,
            ..Default::default()
        }
    }

    pub fn large() -> Self {
        Self {
            height: 44.0,
            font_size: 14.0,
            padding_x: 13.0,
            ..Default::default()
        }
    }

    pub fn multiline() -> Self {
        Self {
            height: 96.0,
            font_size: type_scale::SMALL,
            padding_x: 10.0,
            multiline: true,
            ..Default::default()
        }
    }

    pub fn bare() -> Self {
        Self {
            height: 30.0,
            font_size: type_scale::SMALL,
            padding_x: 4.0,
            bare: true,
            ..Default::default()
        }
    }
}

pub struct TextInput {
    focus_handle: FocusHandle,
    content: SharedString,
    placeholder: SharedString,
    masked: bool,
    style: InputStyle,
    prefix: Option<Icon>,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_multiline: Option<MultilineLayout>,
    multiline_scroll: f32,
    last_bounds: Option<Bounds<Pixels>>,
    is_selecting: bool,
}

impl TextInput {
    pub fn new(
        placeholder: impl Into<SharedString>,
        initial: impl Into<SharedString>,
        style: InputStyle,
        cx: &mut Context<Self>,
    ) -> Self {
        let content = initial.into();
        let cursor = content.len();
        Self {
            focus_handle: cx.focus_handle(),
            content,
            placeholder: placeholder.into(),
            masked: false,
            style,
            prefix: None,
            selected_range: cursor..cursor,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_multiline: None,
            multiline_scroll: 0.0,
            last_bounds: None,
            is_selecting: false,
        }
    }

    pub fn masked(mut self, masked: bool) -> Self {
        self.masked = masked;
        self
    }

    pub fn is_masked(&self) -> bool {
        self.masked
    }

    pub fn with_prefix(mut self, prefix: Icon) -> Self {
        self.prefix = Some(prefix);
        self
    }

    pub fn text(&self) -> &str {
        &self.content
    }

    pub fn set_text(&mut self, value: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.content = value.into();
        self.selected_range = self.content.len()..self.content.len();
        self.selection_reversed = false;
        self.marked_range = None;
        cx.emit(TextInputEvent::Change);
        cx.notify();
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        if !self.content.is_empty() {
            self.set_text("", cx);
        }
    }

    pub fn set_masked(&mut self, masked: bool, cx: &mut Context<Self>) {
        if self.masked != masked {
            self.masked = masked;
            cx.notify();
        }
    }

    pub fn focus(&self) -> FocusHandle {
        self.focus_handle.clone()
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
        if self.style.multiline {
            let line = self.line_index_at(self.cursor_offset());
            let (start, _) = self.line_bounds(line);
            self.move_to(start, cx);
        } else {
            self.move_to(0, cx);
        }
    }

    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        if self.style.multiline {
            let line = self.line_index_at(self.cursor_offset());
            let (_, end) = self.line_bounds(line);
            self.move_to(end, cx);
        } else {
            self.move_to(self.content.len(), cx);
        }
    }

    fn up(&mut self, _: &Up, _: &mut Window, cx: &mut Context<Self>) {
        if !self.style.multiline {
            return;
        }
        let cursor = self.cursor_offset();
        let line = self.line_index_at(cursor);
        if line == 0 {
            self.move_to(0, cx);
            return;
        }
        let (current_start, _) = self.line_bounds(line);
        let column = cursor.saturating_sub(current_start);
        let (start, end) = self.line_bounds(line - 1);
        self.move_to((start + column).min(end), cx);
    }

    fn down(&mut self, _: &Down, _: &mut Window, cx: &mut Context<Self>) {
        if !self.style.multiline {
            return;
        }
        let cursor = self.cursor_offset();
        let line = self.line_index_at(cursor);
        let lines = self.line_starts().len();
        if line + 1 >= lines {
            self.move_to(self.content.len(), cx);
            return;
        }
        let (current_start, _) = self.line_bounds(line);
        let column = cursor.saturating_sub(current_start);
        let (start, end) = self.line_bounds(line + 1);
        self.move_to((start + column).min(end), cx);
    }

    fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.content.is_empty() && self.selected_range.is_empty() {
            cx.emit(TextInputEvent::BackspaceEmpty);
            return;
        }
        if self.selected_range.is_empty() {
            self.select_to(self.previous_boundary(self.cursor_offset()), cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }

    fn backspace_word(&mut self, _: &BackspaceWord, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let start = previous_word_boundary(&self.content, self.cursor_offset());
            self.select_to(start, cx);
        }
        self.replace_text_in_range(None, "", window, cx)
    }

    fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.next_boundary(self.cursor_offset()), cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }

    fn delete_word(&mut self, _: &DeleteWord, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let end = next_word_boundary(&self.content, self.selected_range.end);
            self.select_to(end, cx);
        }
        self.replace_text_in_range(None, "", window, cx)
    }

    fn submit(&mut self, _: &Submit, window: &mut Window, cx: &mut Context<Self>) {
        if self.style.multiline {
            self.replace_text_in_range(None, "\n", window, cx);
        } else {
            cx.emit(TextInputEvent::Submit);
        }
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus_handle);
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

    fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            if self.style.multiline {
                self.replace_text_in_range(None, &text.replace("\r\n", "\n"), window, cx);
            } else {
                self.replace_text_in_range(None, &text.replace(['\n', '\r'], " "), window, cx);
            }
        }
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
        }
    }

    fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
            self.replace_text_in_range(None, "", window, cx)
        }
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        cx.notify()
    }

    pub fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.content.is_empty() {
            return 0;
        }

        if self.style.multiline {
            if let Some(layout) = self.last_multiline.as_ref() {
                if position.y < layout.bounds.top() {
                    return 0;
                }
                if position.y > layout.bounds.bottom() {
                    return self.content.len();
                }
                let relative_y = f32::from(position.y - layout.bounds.top());
                let (line, local_y) = layout.line_at_y(relative_y);
                let line = line.min(layout.lines.len().saturating_sub(1));
                let x = position.x - layout.bounds.left();
                let display_index = layout.lines[line]
                    .closest_index_for_position(point(x, px(local_y)), px(layout.line_height))
                    .unwrap_or_else(|index| index);
                let offset =
                    layout.line_starts[line] + display_index.min(layout.lines[line].text.len());
                return offset.min(self.content.len());
            }
            return 0;
        }

        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };
        if position.y < bounds.top() {
            return 0;
        }
        if position.y > bounds.bottom() {
            return self.content.len();
        }

        let display_index = line.closest_index_for_x(position.x - bounds.left());
        if self.masked {
            self.content_offset_for_display_index(display_index)
        } else {
            display_index
        }
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

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;

        for ch in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }

        utf8_offset
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

    fn display_text(&self) -> SharedString {
        if self.masked {
            SharedString::from(BULLET.to_string().repeat(self.content.chars().count()))
        } else {
            self.content.clone()
        }
    }

    fn display_index(&self, content_offset: usize) -> usize {
        if !self.masked {
            return content_offset;
        }
        let chars = self.content[..content_offset.min(self.content.len())]
            .chars()
            .count();
        chars * BULLET.len_utf8()
    }

    fn content_offset_for_display_index(&self, display_offset: usize) -> usize {
        if !self.masked {
            return display_offset;
        }
        let chars = self.display_text()[..display_offset.min(self.display_text().len())]
            .chars()
            .count();
        self.content
            .char_indices()
            .nth(chars)
            .map(|(index, _)| index)
            .unwrap_or(self.content.len())
    }

    fn line_starts(&self) -> Vec<usize> {
        let mut starts = vec![0];
        for (index, ch) in self.content.char_indices() {
            if ch == '\n' {
                starts.push(index + 1);
            }
        }
        starts
    }

    fn line_index_at(&self, offset: usize) -> usize {
        self.content[..offset.min(self.content.len())]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
    }

    fn line_bounds(&self, line: usize) -> (usize, usize) {
        let starts = self.line_starts();
        if starts.is_empty() {
            return (0, 0);
        }
        let line = line.min(starts.len() - 1);
        let start = starts[line];
        let end = if line + 1 < starts.len() {
            starts[line + 1] - 1
        } else {
            self.content.len()
        };
        (start, end)
    }
}

impl EntityInputHandler for TextInput {
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
        self.selected_range = range.start + new_text.len()..range.start + new_text.len();
        self.selection_reversed = false;
        self.marked_range.take();
        cx.emit(TextInputEvent::Change);
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
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .map(|new_range| new_range.start + range.start..new_range.end + range.end)
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());

        cx.emit(TextInputEvent::Change);
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let range = self.range_from_utf16(&range_utf16);
        if self.style.multiline {
            let layout = self.last_multiline.as_ref()?;
            let start = layout.position_for_offset(range.start)?;
            let end = layout.position_for_offset(range.end)?;
            let _ = bounds;
            return Some(Bounds::from_corners(
                point(start.x, start.y),
                point(end.x, end.y + px(layout.line_height)),
            ));
        }
        let last_layout = self.last_layout.as_ref()?;
        let display_range = self.display_index(range.start)..self.display_index(range.end);
        Some(Bounds::from_corners(
            point(
                bounds.left() + last_layout.x_for_index(display_range.start),
                bounds.top(),
            ),
            point(
                bounds.left() + last_layout.x_for_index(display_range.end),
                bounds.bottom(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        if self.style.multiline {
            let layout = self.last_multiline.as_ref()?;
            let local = layout.bounds.localize(&point)?;
            let (line, local_y) = layout.line_at_y(f32::from(local.y));
            let line = line.min(layout.lines.len().saturating_sub(1));
            let index_in_line = layout.lines[line]
                .closest_index_for_position(
                    gpui::point(local.x, px(local_y)),
                    px(layout.line_height),
                )
                .unwrap_or_else(|index| index);
            let content_index = (layout.line_starts[line]
                + index_in_line.min(layout.lines[line].text.len()))
            .min(self.content.len());
            return Some(self.offset_to_utf16(content_index));
        }
        let line_point = self.last_bounds?.localize(&point)?;
        let last_layout = self.last_layout.as_ref()?;
        let utf8_index = last_layout.index_for_x(point.x - line_point.x)?;
        let content_index = self.content_offset_for_display_index(utf8_index);
        Some(self.offset_to_utf16(content_index))
    }
}

struct TextElement {
    input: Entity<TextInput>,
    font_size: f32,
    height: f32,
    font_family: SharedString,
    text_color: gpui::Hsla,
    placeholder_color: gpui::Hsla,
    caret_color: gpui::Hsla,
    selection_color: gpui::Hsla,
}

struct TextPrepaint {
    line: Option<ShapedLine>,
    multiline: Option<MultilineLayout>,
    cursor: Option<PaintQuad>,
    selection: Vec<PaintQuad>,
}

struct MultilineLayout {
    lines: Vec<gpui::WrappedLine>,
    line_starts: Vec<usize>,
    line_offsets: Vec<f32>,
    line_height: f32,
    scroll_offset: f32,
    bounds: Bounds<Pixels>,
}

impl MultilineLayout {
    fn position_for_offset(&self, offset: usize) -> Option<Point<Pixels>> {
        let mut line = 0;
        for (index, start) in self.line_starts.iter().enumerate() {
            if *start <= offset {
                line = index;
            } else {
                break;
            }
        }
        let line_layout = self.lines.get(line)?;
        let local = offset
            .saturating_sub(*self.line_starts.get(line)?)
            .min(line_layout.text.len());
        let position = line_layout.position_for_index(local, px(self.line_height))?;
        Some(point(
            self.bounds.left() + position.x,
            self.bounds.top() + px(*self.line_offsets.get(line)?) + position.y
                - px(self.scroll_offset),
        ))
    }

    fn line_at_y(&self, y: f32) -> (usize, f32) {
        let y = y + self.scroll_offset;
        let mut line = 0;
        let mut top = 0.0;
        for (index, offset) in self.line_offsets.iter().enumerate() {
            if y >= *offset {
                line = index;
                top = *offset;
            } else {
                break;
            }
        }
        (line, (y - top).max(0.0))
    }
}

impl IntoElement for TextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl gpui::Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = TextPrepaint;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = px(self.height).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let content = input.content.clone();
        let selected_range = input.selected_range.clone();
        let cursor = input.cursor_offset();
        let marked_range = input.marked_range.clone();
        let masked = input.masked;
        let multiline = input.style.multiline;

        let empty = content.is_empty();
        let display_text = input.display_text();

        let (text, text_color) = if empty {
            (input.placeholder.clone(), self.placeholder_color)
        } else {
            (display_text, self.text_color)
        };

        let mut run_font = font(self.font_family.clone());
        run_font.weight = gpui::FontWeight::NORMAL;
        let base_run = TextRun {
            len: text.len(),
            font: run_font,
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };

        let runs = if !empty && !masked {
            if let Some(marked_range) = marked_range.as_ref() {
                vec![
                    TextRun {
                        len: marked_range.start.min(text.len()),
                        ..base_run.clone()
                    },
                    TextRun {
                        len: (marked_range.end.min(text.len())
                            - marked_range.start.min(text.len())),
                        underline: Some(UnderlineStyle {
                            color: Some(base_run.color),
                            thickness: px(1.0),
                            wavy: false,
                        }),
                        ..base_run.clone()
                    },
                    TextRun {
                        len: text.len().saturating_sub(marked_range.end.min(text.len())),
                        ..base_run.clone()
                    },
                ]
                .into_iter()
                .filter(|run| run.len > 0)
                .collect()
            } else {
                vec![base_run.clone()]
            }
        } else {
            vec![base_run.clone()]
        };

        if multiline && !masked {
            let line_height = self.font_size * 1.45;
            let shaped = window
                .text_system()
                .shape_text(
                    text.clone(),
                    px(self.font_size),
                    &runs,
                    Some(bounds.size.width),
                    None,
                )
                .unwrap_or_default();
            let mut lines: Vec<gpui::WrappedLine> = shaped.into_iter().collect();
            if lines.is_empty() {
                lines.push(gpui::WrappedLine::default());
            }
            let mut line_starts = if empty { vec![0] } else { input.line_starts() };
            while line_starts.len() < lines.len() {
                line_starts.push(content.len());
            }

            let mut line_offsets = Vec::with_capacity(lines.len());
            let mut next_y = 0.0;
            for line in &lines {
                line_offsets.push(next_y);
                next_y += f32::from(line.size(px(line_height)).height);
            }

            let total_height = next_y;
            let field_height = f32::from(bounds.size.height);
            let cursor_line = input
                .line_index_at(cursor)
                .min(lines.len().saturating_sub(1));
            let cursor_line_start = *line_starts.get(cursor_line).unwrap_or(&0);
            let cursor_local = cursor
                .saturating_sub(cursor_line_start)
                .min(lines[cursor_line].text.len());
            let cursor_position = lines[cursor_line]
                .position_for_index(cursor_local, px(line_height))
                .unwrap_or_else(|| point(px(0.0), px(0.0)));
            let cursor_content_y = line_offsets[cursor_line] + f32::from(cursor_position.y);
            let mut scroll_offset = input.multiline_scroll.max(0.0);
            if cursor_content_y + line_height > scroll_offset + field_height {
                scroll_offset = cursor_content_y + line_height - field_height;
            }
            if cursor_content_y < scroll_offset {
                scroll_offset = cursor_content_y;
            }
            let max_scroll = (total_height - field_height).max(0.0);
            let scroll_offset = scroll_offset.clamp(0.0, max_scroll);

            let mut cursor_quad = None;
            if selected_range.is_empty() {
                cursor_quad = Some(fill(
                    Bounds::new(
                        point(
                            bounds.left() + cursor_position.x,
                            bounds.top() + px(cursor_content_y) - px(scroll_offset) + px(2.0),
                        ),
                        size(px(1.5), px(line_height) - px(4.0)),
                    ),
                    self.caret_color,
                ));
            }

            let mut selection = Vec::new();
            if !selected_range.is_empty() {
                for (index, line) in lines.iter().enumerate() {
                    let line_start = *line_starts.get(index).unwrap_or(&0);
                    let line_len = line.text.len();
                    let line_end = line_start + line_len;
                    if selected_range.end < line_start || selected_range.start > line_end {
                        continue;
                    }
                    let start_column = if selected_range.start <= line_start {
                        0
                    } else {
                        (selected_range.start - line_start).min(line_len)
                    };
                    let end_column = if selected_range.end >= line_end {
                        line_len
                    } else {
                        (selected_range.end - line_start).min(line_len)
                    };
                    let start_position = line
                        .position_for_index(start_column, px(line_height))
                        .unwrap_or_else(|| point(px(0.0), px(0.0)));
                    let mut end_position = line
                        .position_for_index(end_column, px(line_height))
                        .unwrap_or_else(|| point(px(0.0), px(0.0)));
                    if selected_range.end > line_end {
                        end_position.x = bounds.size.width;
                        end_position.y = px(line_height * line.wrap_boundaries().len() as f32);
                    }
                    let top = bounds.top() + px(line_offsets[index]) - px(scroll_offset);
                    if start_position.y == end_position.y {
                        let x0 = bounds.left() + start_position.x;
                        let x1 = bounds.left() + end_position.x;
                        if x1 > x0 {
                            selection.push(fill(
                                Bounds::new(
                                    point(x0, top + start_position.y),
                                    size(x1 - x0, px(line_height)),
                                ),
                                self.selection_color,
                            ));
                        }
                    } else {
                        let x0 = bounds.left() + start_position.x;
                        let x1 = bounds.right();
                        if x1 > x0 {
                            selection.push(fill(
                                Bounds::new(
                                    point(x0, top + start_position.y),
                                    size(x1 - x0, px(line_height)),
                                ),
                                self.selection_color,
                            ));
                        }
                        let mut y = start_position.y + px(line_height);
                        while y < end_position.y {
                            selection.push(fill(
                                Bounds::new(
                                    point(bounds.left(), top + y),
                                    size(bounds.size.width, px(line_height)),
                                ),
                                self.selection_color,
                            ));
                            y += px(line_height);
                        }
                        let x1 = bounds.left() + end_position.x;
                        selection.push(fill(
                            Bounds::new(
                                point(bounds.left(), top + end_position.y),
                                size(x1 - bounds.left(), px(line_height)),
                            ),
                            self.selection_color,
                        ));
                    }
                }
            }

            return TextPrepaint {
                line: None,
                multiline: Some(MultilineLayout {
                    lines,
                    line_starts,
                    line_offsets,
                    line_height,
                    scroll_offset,
                    bounds,
                }),
                cursor: cursor_quad,
                selection,
            };
        }

        let line = window
            .text_system()
            .shape_line(text, px(self.font_size), &runs, None);

        let display_cursor = input.display_index(cursor);
        let cursor_x = line.x_for_index(display_cursor);
        let (selection, cursor) = if selected_range.is_empty() {
            (
                Vec::new(),
                Some(fill(
                    Bounds::new(
                        point(bounds.left() + cursor_x, bounds.top() + px(2.0)),
                        size(px(1.5), bounds.size.height - px(4.0)),
                    ),
                    self.caret_color,
                )),
            )
        } else {
            let start = input.display_index(selected_range.start);
            let end = input.display_index(selected_range.end);
            (
                vec![fill(
                    Bounds::from_corners(
                        point(bounds.left() + line.x_for_index(start), bounds.top()),
                        point(bounds.left() + line.x_for_index(end), bounds.bottom()),
                    ),
                    self.selection_color,
                )],
                None,
            )
        };

        TextPrepaint {
            line: Some(line),
            multiline: None,
            cursor,
            selection,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );

        for quad in prepaint.selection.drain(..) {
            window.paint_quad(quad);
        }

        if let Some(multiline) = prepaint.multiline.take() {
            for (index, line) in multiline.lines.iter().enumerate() {
                line.paint(
                    point(
                        bounds.left(),
                        bounds.top() + px(multiline.line_offsets[index])
                            - px(multiline.scroll_offset),
                    ),
                    px(multiline.line_height),
                    gpui::TextAlign::Left,
                    Some(bounds),
                    window,
                    cx,
                )
                .ok();
            }

            if focus_handle.is_focused(window)
                && let Some(cursor) = prepaint.cursor.take()
            {
                window.paint_quad(cursor);
            }

            self.input.update(cx, |input, _cx| {
                input.last_layout = None;
                input.multiline_scroll = multiline.scroll_offset;
                input.last_multiline = Some(multiline);
                input.last_bounds = Some(bounds);
            });
            return;
        }

        let line = prepaint.line.take().expect("line was shaped");
        line.paint(bounds.origin, px(self.height), window, cx).ok();

        if focus_handle.is_focused(window)
            && let Some(cursor) = prepaint.cursor.take()
        {
            window.paint_quad(cursor);
        }

        self.input.update(cx, |input, _cx| {
            input.last_layout = Some(line);
            input.last_multiline = None;
            input.last_bounds = Some(bounds);
        });
    }
}

impl Render for TextInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme: Theme = theme(cx).clone();
        let focused = self.focus_handle.is_focused(window);
        let height = self.style.height;
        let padding = self.style.padding_x;
        let font_size = self.style.font_size;
        let filled = self.style.filled;
        let bare = self.style.bare;
        let base_bg = if bare || !filled {
            gpui::transparent_black()
        } else {
            theme.surface_muted
        };
        let focus_border = theme.primary;
        let focus_bg = theme.surface;
        let hover_bg = theme.surface_hover;
        let prefix_color = if focused { theme.primary } else { theme.subtle };
        let placeholder_color = if focused {
            theme.subtle
        } else {
            theme.subtle.opacity(0.9)
        };
        let text_color = theme.foreground;
        let caret_color = theme.primary;
        let selection_color = theme.selection;
        let underline_color = if focused {
            theme.primary
        } else {
            theme.border_strong
        };

        let input = cx.entity();
        let prefix = self.prefix;
        let font_family = theme.font.clone();
        let multiline = self.style.multiline;
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .h(px(height))
            .w_full()
            .px(px(padding))
            .rounded_lg()
            .bg(base_bg)
            .when(!bare, |element| element.border_1())
            .when(!bare, |element| {
                element.border_color(if filled {
                    gpui::transparent_black()
                } else {
                    underline_color
                })
            })
            .text_color(text_color)
            .font_family(font_family.clone())
            .text_size(px(font_size))
            .cursor(CursorStyle::IBeam)
            .key_context(self.style.key_context)
            .track_focus(&self.focus_handle)
            .focus(move |style| {
                if bare {
                    style
                } else {
                    style.border_color(focus_border).bg(focus_bg)
                }
            })
            .hover(move |style| {
                if filled && !bare {
                    style.bg(hover_bg)
                } else {
                    style
                }
            })
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::backspace_word))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::delete_word))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::down))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::show_character_palette))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::submit))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .when_some(prefix, |element, prefix| {
                element.child(icon(
                    prefix,
                    if height <= 36.0 { 14.0 } else { 16.0 },
                    prefix_color,
                ))
            })
            .child(
                div()
                    .flex_1()
                    .h(px(height - 2.0))
                    .flex()
                    .when(multiline, |element| element.items_start().pt(px(8.0)))
                    .when(!multiline, |element| element.items_center())
                    .overflow_hidden()
                    .child(TextElement {
                        input,
                        font_size,
                        height: if multiline {
                            height - 18.0
                        } else {
                            font_size + 6.0
                        },
                        font_family,
                        text_color,
                        placeholder_color,
                        caret_color,
                        selection_color,
                    }),
            )
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

pub fn bind_text_input_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("backspace", Backspace, Some("TextInput")),
        KeyBinding::new("ctrl-backspace", BackspaceWord, Some("TextInput")),
        KeyBinding::new("alt-backspace", BackspaceWord, Some("TextInput")),
        KeyBinding::new("delete", Delete, Some("TextInput")),
        KeyBinding::new("ctrl-delete", DeleteWord, Some("TextInput")),
        KeyBinding::new("alt-delete", DeleteWord, Some("TextInput")),
        KeyBinding::new("left", Left, Some("TextInput")),
        KeyBinding::new("right", Right, Some("TextInput")),
        KeyBinding::new("up", Up, Some("TextInput")),
        KeyBinding::new("down", Down, Some("TextInput")),
        KeyBinding::new("shift-left", SelectLeft, Some("TextInput")),
        KeyBinding::new("shift-right", SelectRight, Some("TextInput")),
        KeyBinding::new("secondary-a", SelectAll, Some("TextInput")),
        KeyBinding::new("secondary-v", Paste, Some("TextInput")),
        KeyBinding::new("secondary-c", Copy, Some("TextInput")),
        KeyBinding::new("secondary-x", Cut, Some("TextInput")),
        KeyBinding::new("home", Home, Some("TextInput")),
        KeyBinding::new("end", End, Some("TextInput")),
        KeyBinding::new("enter", Submit, Some("TextInput")),
        KeyBinding::new("ctrl-cmd-space", ShowCharacterPalette, Some("TextInput")),
        // Added after the TextInput bindings so the search context wins for
        // these keys while the power-search box is focused.
        KeyBinding::new("ctrl-space", SearchOpenSuggestions, Some("SearchInput")),
        KeyBinding::new("tab", SearchComplete, Some("SearchInput")),
        KeyBinding::new("down", SearchNext, Some("SearchInput")),
        KeyBinding::new("up", SearchPrevious, Some("SearchInput")),
    ]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    struct MultilineHarness {
        input: Entity<TextInput>,
    }

    impl Render for MultilineHarness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(px(140.0)).child(self.input.clone())
        }
    }

    #[gpui::test]
    fn wrapped_lines_map_offsets_across_rows(cx: &mut TestAppContext) {
        cx.update(|cx| cx.set_global(crate::theme::Theme::light()));
        let (view, cx) = cx.add_window_view(|_, cx| {
            let input =
                cx.new(|cx| TextInput::new("Note", "W".repeat(80), InputStyle::multiline(), cx));
            MultilineHarness { input }
        });
        cx.refresh().expect("refresh");
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        cx.run_until_parked();

        view.update(cx, |harness, cx| {
            let input = harness.input.read(cx);
            let layout = input
                .last_multiline
                .as_ref()
                .expect("multiline layout is painted");
            let first = layout.position_for_offset(0).expect("first offset");
            let later = layout.position_for_offset(60).expect("wrapped offset");
            assert!(
                later.y > first.y,
                "later offsets inside a wrapped line must move to a lower row"
            );
        });
    }

    #[gpui::test]
    fn long_multiline_lines_soft_wrap(cx: &mut TestAppContext) {
        cx.update(|cx| cx.set_global(crate::theme::Theme::light()));
        let (view, cx) = cx.add_window_view(|_, cx| {
            let input =
                cx.new(|cx| TextInput::new("Note", "W".repeat(80), InputStyle::multiline(), cx));
            MultilineHarness { input }
        });
        cx.refresh().expect("refresh");
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        cx.run_until_parked();

        view.update(cx, |harness, cx| {
            let layout = harness
                .input
                .read(cx)
                .last_multiline
                .as_ref()
                .expect("multiline layout is painted");
            assert!(
                layout
                    .lines
                    .iter()
                    .any(|line| !line.wrap_boundaries().is_empty()),
                "a long logical line must produce soft-wrap boundaries"
            );
            assert!(
                layout.scroll_offset > 0.0,
                "the field scrolls to keep the cursor visible"
            );
        });
    }

    #[test]
    fn word_boundaries_delete_whole_words() {
        let text = "hello brave new world";
        // Cursor at the end: delete "world".
        assert_eq!(previous_word_boundary(text, text.len()), 16);
        // Cursor after a trailing space: delete the space and the word.
        assert_eq!(previous_word_boundary("hello world ", 12), 6);
        // Punctuation is its own run.
        assert_eq!(previous_word_boundary("hello,world", 6), 5);
        assert_eq!(previous_word_boundary("hello,world", 11), 6);
        // Nothing to delete at the start.
        assert_eq!(previous_word_boundary(text, 0), 0);

        assert_eq!(next_word_boundary(text, 0), 5);
        assert_eq!(next_word_boundary("hello world", 5), 11);
        assert_eq!(next_word_boundary("hello, world", 5), 6);
        assert_eq!(next_word_boundary(text, text.len()), text.len());
    }
}
