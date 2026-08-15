use crate::model::{ClxState, PlayMode};
use crate::ui::root::RootView;
use crate::ui::theme::*;
use gpui::prelude::*;
use gpui::*;
use std::ops::Range;
use std::time::Duration;

fn filter_placeholder(mode: PlayMode) -> &'static str {
    match mode {
        PlayMode::Listen => "聆听模式仅可播放；可在设置中切换为「演奏」",
        PlayMode::Script => "F1开始自动演奏，F2停止演奏",
    }
}
const CURSOR_BLINK_MS: u64 = 530;
const CURSOR_WIDTH_PX: f32 = 1.5;
const CURSOR_HEIGHT_RATIO: f32 = 0.78;
const CURSOR_RADIUS_PX: f32 = 1.0;
const CURSOR_COLOR: u32 = ACCENT;
const SELECTION_FILL: u32 = ACCENT_SOFT;

actions!(
    music_filter,
    [FilterBackspace, FilterDelete, FilterClear]
);

/// 曲库搜索框：通过平台 IME / `EntityInputHandler` 接收输入，并同步 `ClxState::filter`。
pub struct MusicFilterInput {
    focus_handle: FocusHandle,
    model: Entity<ClxState>,
    weak_root: WeakEntity<RootView>,
    content: SharedString,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    last_text_origin_x: Option<Pixels>,
    cursor_blink_on: bool,
    blink_loop_started: bool,
}

fn text_cursor_quad(bounds: Bounds<Pixels>, cursor_x: Pixels, alpha: f32) -> PaintQuad {
    let h = bounds.size.height * CURSOR_HEIGHT_RATIO;
    let top = bounds.top() + (bounds.size.height - h) / 2.;
    let w = px(CURSOR_WIDTH_PX);
    let mut c = rgb(CURSOR_COLOR);
    c.a = alpha;
    fill(
        Bounds::new(point(cursor_x - w / 2., top), size(w, h)),
        c,
    )
    .corner_radii(px(CURSOR_RADIUS_PX))
}

fn shape_filter_line(window: &Window, text: SharedString, color: Hsla) -> ShapedLine {
    let style = window.text_style();
    let run = TextRun {
        len: text.len(),
        font: style.font(),
        color,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let font_size = style.font_size.to_pixels(window.rem_size());
    window
        .text_system()
        .shape_line(text, font_size, &[run], None)
}

impl MusicFilterInput {
    pub fn new(
        model: Entity<ClxState>,
        weak_root: WeakEntity<RootView>,
        cx: &mut Context<Self>,
    ) -> Self {
        let content = cx.read_entity(&model, |s, _| s.filter.clone());
        let end = content.len();
        Self {
            focus_handle: cx.focus_handle(),
            model,
            weak_root,
            content: content.into(),
            selected_range: end..end,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            last_text_origin_x: None,
            cursor_blink_on: true,
            blink_loop_started: false,
        }
    }

    fn start_blink_loop(&mut self, cx: &mut Context<Self>) {
        if self.blink_loop_started {
            return;
        }
        self.blink_loop_started = true;
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(CURSOR_BLINK_MS))
                    .await;
                let _ = this.update(cx, |input, cx| {
                    input.cursor_blink_on = !input.cursor_blink_on;
                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn notify_root(&self, cx: &mut Context<Self>) {
        if let Some(root) = self.weak_root.upgrade() {
            let _ = root.update(cx, |_, cx| cx.notify());
        }
    }

    fn apply_filter(&mut self, text: String, cx: &mut Context<Self>) {
        self.content = text.clone().into();
        self.selected_range = text.len()..text.len();
        self.marked_range = None;
        let model = self.model.clone();
        let _ = cx.update_entity(&model, |st, c| {
            st.set_filter(text);
            let ids: Vec<i64> = st
                .filtered_display
                .iter()
                .take(32)
                .map(|m| m.id)
                .collect();
            st.enqueue_music_cover_fetches(ids);
            c.notify();
        });
        self.notify_root(cx);
        cx.notify();
    }

    fn clear(&mut self, cx: &mut Context<Self>) {
        self.apply_filter(String::new(), cx);
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.content
            .char_indices()
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.content
            .char_indices()
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content.len())
    }

    fn backspace(&mut self, _: &FilterBackspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let cursor = self.cursor_offset();
            self.selected_range = self.previous_boundary(cursor)..cursor;
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    fn delete(&mut self, _: &FilterDelete, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let cursor = self.cursor_offset();
            self.selected_range = cursor..self.next_boundary(cursor);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    fn on_clear(&mut self, _: &FilterClear, _: &mut Window, cx: &mut Context<Self>) {
        self.clear(cx);
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
}

impl EntityInputHandler for MusicFilterInput {
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

    fn unmark_text(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.marked_range.take().is_some() {
            self.apply_filter(self.content.to_string(), cx);
        }
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
            .map(|r| self.range_from_utf16(r))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range.clone());

        let next = (self.content[0..range.start].to_owned() + new_text
            + &self.content[range.end..])
            .replace('\n', " ");
        self.apply_filter(next, cx);
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
            .map(|r| self.range_from_utf16(r))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range.clone());

        let next = (self.content[0..range.start].to_owned() + new_text
            + &self.content[range.end..])
            .replace('\n', " ");
        self.content = next.clone().into();
        if !new_text.is_empty() {
            self.marked_range = Some(range.start..range.start + new_text.len());
        } else {
            self.marked_range = None;
        }
        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|r| self.range_from_utf16(r))
            .map(|r| r.start + range.start..r.end + range.end)
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
        let last_layout = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        Some(Bounds::from_corners(
            point(
                bounds.left() + last_layout.x_for_index(range.start),
                bounds.top(),
            ),
            point(
                bounds.left() + last_layout.x_for_index(range.end),
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
        if self.content.is_empty() {
            return Some(0);
        }
        self.last_bounds?.localize(&point)?;
        let origin = self.last_text_origin_x?;
        let last_layout = self.last_layout.as_ref()?;
        let utf8_index = last_layout.index_for_x(point.x - origin)?;
        Some(self.offset_to_utf16(utf8_index))
    }
}

struct FilterTextElement {
    input: Entity<MusicFilterInput>,
}

struct FilterPrepaint {
    display_line: ShapedLine,
    content_line: ShapedLine,
    text_origin_x: Pixels,
    cursor_x: Option<Pixels>,
    selection: Option<PaintQuad>,
}

impl Element for FilterTextElement {
    type RequestLayoutState = ();
    type PrepaintState = FilterPrepaint;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let placeholder = cx.read_entity(&input.model, |s, _| filter_placeholder(s.play_mode));
        let text_color = window.text_style().color;
        let content_line = shape_filter_line(window, input.content.clone(), text_color);
        let (display_text, display_color) = if input.content.is_empty() {
            (SharedString::from(placeholder), hsla(0., 0., 0., 0.35))
        } else {
            (input.content.clone(), text_color)
        };
        let display_line = shape_filter_line(window, display_text, display_color);

        let text_origin_x =
            bounds.left() + ((bounds.size.width - display_line.width) / 2.).max(px(0.0));

        let cursor_idx = input
            .cursor_offset()
            .min(if input.content.is_empty() {
                0
            } else {
                input.content.len()
            });
        let selected = input.selected_range.clone();

        let (selection, cursor) = if !selected.is_empty() && !input.content.is_empty() {
            (
                Some(fill(
                    Bounds::from_corners(
                        point(
                            text_origin_x + display_line.x_for_index(selected.start),
                            bounds.top(),
                        ),
                        point(
                            text_origin_x + display_line.x_for_index(selected.end),
                            bounds.bottom(),
                        ),
                    ),
                    rgba(SELECTION_FILL),
                )),
                None,
            )
        } else {
            let cursor_x = if input.content.is_empty() {
                bounds.left() + bounds.size.width / 2.
            } else {
                text_origin_x + display_line.x_for_index(cursor_idx)
            };
            (None, Some(cursor_x))
        };

        FilterPrepaint {
            display_line,
            content_line,
            text_origin_x,
            cursor_x: cursor,
            selection,
        }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
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
        if let Some(selection) = prepaint.selection.take() {
            window.paint_quad(selection);
        }
        let origin = point(prepaint.text_origin_x, bounds.top());
        prepaint
            .display_line
            .paint(origin, window.line_height(), window, cx)
            .ok();
        if focus_handle.is_focused(window) {
            let blink_on = self.input.read(cx).cursor_blink_on;
            if blink_on {
                if let Some(cursor_x) = prepaint.cursor_x {
                    window.paint_quad(text_cursor_quad(bounds, cursor_x, 1.0));
                }
            }
        }
        let content_line = prepaint.content_line.clone();
        let text_origin_x = prepaint.text_origin_x;
        self.input.update(cx, |input, _| {
            input.last_layout = Some(content_line);
            input.last_bounds = Some(bounds);
            input.last_text_origin_x = Some(text_origin_x);
        });
    }
}

impl IntoElement for FilterTextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Focusable for MusicFilterInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for MusicFilterInput {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.start_blink_loop(cx);
        div()
            .flex_1()
            .min_w(px(120.0))
            .max_w(px(420.0))
            .px(px(14.0))
            .py(px(8.0))
            .bg(rgb(BG_SURFACE))
            .rounded_full()
            .text_sm()
            .text_color(rgb(TEXT_PRIMARY))
            .tab_stop(true)
            .cursor(CursorStyle::IBeam)
            .key_context("MusicFilter")
            .track_focus(&self.focus_handle(cx))
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::on_clear))
            .child(
                div()
                    .w_full()
                    .child(FilterTextElement {
                        input: cx.entity(),
                    }),
            )
    }
}
