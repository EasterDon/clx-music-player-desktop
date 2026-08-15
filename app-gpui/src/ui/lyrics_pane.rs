use crate::lyrics::LyricLine;
use crate::model::ClxState;
use crate::ui::motion::{step_toward, DURATION_LYRICS_SCROLL_SECS};
use crate::ui::theme::*;
use gpui::prelude::*;
use gpui::*;
use std::sync::Arc;
use std::time::Duration;

const LYRIC_LINE_MIN_H: f32 = 64.0;
const LYRIC_FONT_NORMAL: f32 = 16.0;
const LYRIC_FONT_CURRENT: f32 = 19.2;
/// 在 `LYRIC_LINE_MIN_H` 内允许的最大行数（超出则省略号）。
const LYRIC_LINE_CLAMP_NORMAL: usize = 3;
const LYRIC_LINE_CLAMP_CURRENT: usize = 2;
const LYRIC_PLACEHOLDER: u32 = 0x7a7a7a;
const LYRIC_MUTED: u32 = 0x808080;

pub struct LyricsPaneView {
    model: Entity<ClxState>,
    /// 仅用于读取视口高度，绝不调用 `set_offset`（正值会把内容推出视口）。
    viewport: ScrollHandle,
    viewport_height: f32,
    translate_y: f32,
    translate_y_target: f32,
    last_centered_index: Option<usize>,
    snap_scroll: bool,
    loaded_music_id: Option<i64>,
    loaded_lines_len: usize,
}

impl LyricsPaneView {
    pub fn new(model: Entity<ClxState>, cx: &mut Context<Self>) -> Self {
        let _ = cx.observe(&model, |_, _, cx| {
            cx.notify();
        });

        let weak = cx.weak_entity();
        cx.spawn(async move |_, async_cx| {
            loop {
                let _ = async_cx
                    .background_executor()
                    .timer(Duration::from_millis(16))
                    .await;
                let Some(pane) = weak.upgrade() else {
                    break;
                };
                let _ = async_cx.update_entity(&pane, |this, cx| {
                    if this.tick_scroll_motion(0.016) {
                        cx.notify();
                    }
                });
            }
        })
        .detach();

        Self {
            model,
            viewport: ScrollHandle::new(),
            viewport_height: 0.0,
            translate_y: 0.0,
            translate_y_target: 0.0,
            last_centered_index: None,
            snap_scroll: true,
            loaded_music_id: None,
            loaded_lines_len: 0,
        }
    }

    fn sync_track(&mut self, music_id: Option<i64>, lines_len: usize) {
        if self.loaded_music_id != music_id || self.loaded_lines_len != lines_len {
            self.loaded_music_id = music_id;
            self.loaded_lines_len = lines_len;
            self.last_centered_index = None;
            self.translate_y = 0.0;
            self.translate_y_target = 0.0;
            self.snap_scroll = true;
        }
    }

    fn translate_for_index(&self, index: usize) -> f32 {
        let vh = self.viewport_height;
        if vh <= 0.0 {
            return 0.0;
        }
        let line_center = index as f32 * LYRIC_LINE_MIN_H + LYRIC_LINE_MIN_H / 2.0;
        vh / 2.0 - line_center
    }

    fn center_current_lyric(&mut self, index: Option<usize>) {
        let Some(i) = index else {
            self.last_centered_index = None;
            return;
        };
        if self.last_centered_index == Some(i) {
            return;
        }
        let rewind = self
            .last_centered_index
            .is_some_and(|prev| i < prev);
        let target = self.translate_for_index(i);
        self.translate_y_target = target;
        if self.snap_scroll || rewind {
            self.translate_y = target;
            self.snap_scroll = false;
        }
        self.last_centered_index = Some(i);
    }

    fn tick_scroll_motion(&mut self, dt: f32) -> bool {
        if self.snap_scroll {
            return false;
        }
        let (v, animating) = step_toward(
            self.translate_y,
            self.translate_y_target,
            dt,
            DURATION_LYRICS_SCROLL_SECS,
        );
        if (v - self.translate_y).abs() > 0.001 {
            self.translate_y = v;
            true
        } else {
            animating
        }
    }

    fn refresh_viewport_height(&mut self, vh: f32) {
        if vh <= 0.0 {
            return;
        }
        if (self.viewport_height - vh).abs() > 0.5 {
            self.viewport_height = vh;
            self.last_centered_index = None;
        }
    }
}

fn lyric_line_row(line: &LyricLine, is_current: bool) -> impl IntoElement {
    let font_size = if is_current {
        LYRIC_FONT_CURRENT
    } else {
        LYRIC_FONT_NORMAL
    };
    let max_lines = if is_current {
        LYRIC_LINE_CLAMP_CURRENT
    } else {
        LYRIC_LINE_CLAMP_NORMAL
    };
    let color = if is_current {
        TEXT_PRIMARY
    } else {
        LYRIC_MUTED
    };

    div()
        .w_full()
        .min_w(px(0.0))
        .min_h(px(LYRIC_LINE_MIN_H))
        .px(px(8.0))
        .flex()
        .items_center()
        .justify_center()
        .overflow_hidden()
        .child(
            div()
                .w_full()
                .min_w(px(0.0))
                .text_center()
                .text_size(px(font_size))
                .text_color(rgb(color))
                .whitespace_normal()
                .line_clamp(max_lines)
                .child(line.content.clone()),
        )
}

impl Render for LyricsPaneView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let viewport = self.viewport.clone();
        let mut has_track = false;
        let mut lines = Arc::new(Vec::new());
        let mut idx = None;
        let mut music_id = None;
        cx.read_entity(&self.model, |s, _| {
            has_track = s.current.is_some();
            lines = s.lyrics.clone();
            idx = s.current_lyric_index;
            music_id = s.current.as_ref().map(|m| m.id);
        });

        self.sync_track(music_id, lines.len());

        let vh = viewport.bounds().size.height.to_f64() as f32;
        self.refresh_viewport_height(vh);

        let center_idx = idx.or_else(|| (!lines.is_empty()).then_some(0));
        self.center_current_lyric(center_idx);

        let translate_y = self.translate_y;

        div()
            .flex_1()
            .min_h(px(0.0))
            .h_full()
            .w_full()
            .overflow_hidden()
            .id("scroll-lyrics")
            .track_scroll(&viewport)
            .when(!has_track, |shell| {
                shell.flex().items_center().justify_center().child(
                    div()
                        .text_size(px(LYRIC_FONT_NORMAL))
                        .text_color(rgb(LYRIC_PLACEHOLDER))
                        .child("暂未选择歌曲"),
                )
            })
            .when(has_track && lines.is_empty(), |shell| {
                shell.flex().items_center().justify_center().child(
                    div()
                        .text_size(px(LYRIC_FONT_NORMAL))
                        .text_color(rgb(LYRIC_PLACEHOLDER))
                        .child("加载歌词中…"),
                )
            })
            .when(has_track && !lines.is_empty(), |shell| {
                shell.child(
                    div()
                        .relative()
                        .w_full()
                        .top(px(translate_y))
                        .flex()
                        .flex_col()
                        .w_full()
                        .text_center()
                        .children(lines.iter().enumerate().map(|(i, line)| {
                            lyric_line_row(line, Some(i) == idx)
                        })),
                )
            })
    }
}
