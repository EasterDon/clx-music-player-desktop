use crate::model::ClxState;
use crate::ui::theme::*;
use gpui_kit::component::Disableable;
use gpui_kit::component::button::Button;
use gpui_kit::component::slider::{Slider, SliderEvent, SliderState};
use gpui_kit::prelude::*;
use gpui_kit::Subscription;
use gpui_kit::*;

const TRANSPORT_SIDE: f32 = 36.0;
const TRANSPORT_MAIN: f32 = 44.0;
const ICON_SIDE: f32 = 18.0;
const ICON_MAIN: f32 = 22.0;

/// 播放控制栏。
///
/// 进度条由 `gpui_kit::component::slider::Slider` 承担：`SliderEvent::Change` 在
/// 拖动过程中连续触发，`SliderEvent::Release` 在松手时触发。拖动中的秒数记录在
/// `PlayerBarView::drag_frac`，避免与播放进度回写互相拉扯。
pub struct PlayerBarView {
    pub model: Entity<ClxState>,
    progress: Entity<SliderState>,
    /// 正在拖动进度条。
    progress_dragging: bool,
    /// 拖动中的实时进度比例 0..1（松手后复位为 None）。
    drag_frac: Option<f32>,
    /// 刚松开滑条、已发起 seek 但音频位置尚未跟上的目标比例。
    ///
    /// 松手瞬间 `current_time` 还是旧值，若立刻按它回写滑条，会把刚拖到的位置
    /// 拽回原处（表现为“点了又跳回去”）。此期间以目标比例为准。
    pending_seek: Option<f32>,
    /// 上一次写回 SliderState 的秒数，用于避免重复 set_value。
    synced_secs: f32,
    /// 必须持有：`Subscription` 一旦被 drop 就会取消订阅。
    _progress_sub: Subscription,
}

impl PlayerBarView {
    pub fn new(model: Entity<ClxState>, cx: &mut Context<Self>) -> Self {
        let progress = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(1.0)
                .step(0.1)
                .default_value(0.0)
        });

        let progress_sub = cx.subscribe(&progress, |this, _, event: &SliderEvent, cx| match event {
            SliderEvent::Change(value) => {
                this.progress_dragging = true;
                this.drag_frac = Some(value.end());
                cx.notify();
            }
            SliderEvent::Release(value) => {
                // 滑条量程是 0..1 的比例，必须乘回时长才是秒数。
                let frac = value.end();
                this.progress_dragging = false;
                this.drag_frac = None;
                this.synced_secs = frac;
                this.pending_seek = Some(frac);
                let model = this.model.clone();
                let _ = cx.update_entity(&model, |st, c| {
                    if st.duration > 0.0 {
                        let cap = st.buffered_end_secs().max(0.0);
                        let secs = (frac as f64 * st.duration).min(cap).max(0.0);
                        st.seek(secs);
                    }
                    c.notify();
                });
                cx.notify();
            }
        });

        let _ = cx.observe(&model, |_, _, cx| {
            cx.notify();
        });

        Self {
            model,
            progress,
            progress_dragging: false,
            drag_frac: None,
            pending_seek: None,
            synced_secs: -1.0,
            _progress_sub: progress_sub,
        }
    }
}

fn fmt_time(sec: f64) -> String {
    if !sec.is_finite() || sec < 0.0 {
        return "00:00".into();
    }
    let s = sec.floor() as u64;
    let mm = s / 60;
    let ss = s % 60;
    format!("{mm:02}:{ss:02}")
}

impl Render for PlayerBarView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let model = self.model.clone();
        let colors = Palette::from(cx);

        let (is_playing, ct, dur, play_disabled, max_seek) =
            cx.read_entity(&self.model, |s, _| {
                (
                    s.is_playing,
                    s.current_time,
                    s.duration,
                    s.current.is_none(),
                    s.buffered_end_secs(),
                )
            });

        // 量程固定为 0..1（归一化），进度换算成比例写回 SliderState；
        // 这样时长变化时无需重建 slider 实体。
        // 与原始实现一致：只有在有歌曲、有时长、且已有可 seek 的缓冲时才允许拖动。
        // 否则刚加载时 buffer 接近 0，seek 会被 clamp 到开头（表现为“点一下就回到第一秒”）。
        let seekable = !play_disabled && dur > 0.0 && max_seek > 0.0;

        // 松手后的目标比例：等音频位置真正跟上（误差 < 1%）再交还给播放进度。
        if let Some(target) = self.pending_seek {
            let reached = dur > 0.0 && ((ct / dur) as f32 - target).abs() < 0.01;
            if reached || !seekable {
                self.pending_seek = None;
            }
        }

        let frac = if seekable {
            match self.drag_frac.or(self.pending_seek) {
                // 拖动中/刚松手：用滑条比例，避免与播放进度互相拉扯。
                Some(f) => f.clamp(0.0, 1.0),
                None => (ct / dur).clamp(0.0, 1.0) as f32,
            }
        } else {
            0.0
        };

        if !self.progress_dragging && (self.synced_secs - frac).abs() > 0.001 {
            self.synced_secs = frac;
            self.progress.update(cx, |state, cx| {
                state.set_value(frac, window, cx);
            });
        }

        // 左侧时间文案：拖动中按比例换回秒数显示。
        let shown_secs = match self.drag_frac.or(self.pending_seek) {
            Some(f) => (f as f64 * dur).clamp(0.0, dur),
            None => ct,
        };

        let side_color = colors.text_secondary;
        let main_color = if play_disabled {
            // 禁用态仍要能看出这是一个播放按钮：用次要文字色而非最浅的辅助色。
            colors.text_secondary
        } else {
            colors.accent_fg
        };

        let controls = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .gap(px(20.0))
            .child({
                let m = model.clone();
                transport_button(
                    &colors,
                    "player-prev",
                    IconKind::Prev,
                    ICON_SIDE,
                    side_color,
                    TRANSPORT_SIDE,
                    TransportKind::Side,
                    false,
                    move |_, _, cx| {
                        let _ = cx.update_entity(&m, |s, c| {
                            s.previous_track();
                            c.notify();
                        });
                    },
                )
            })
            .child({
                let m = model.clone();
                let kind = if is_playing { IconKind::Pause } else { IconKind::Play };
                let id = if is_playing { "player-pause" } else { "player-play" };
                transport_button(
                    &colors,
                    id,
                    kind,
                    ICON_MAIN,
                    main_color,
                    TRANSPORT_MAIN,
                    TransportKind::Main,
                    play_disabled,
                    move |_, _, cx| {
                        let _ = cx.update_entity(&m, |st, c| {
                            if st.is_playing {
                                st.pause_playback();
                            } else {
                                st.play_playback();
                            }
                            c.notify();
                        });
                    },
                )
            })
            .child({
                let m = model.clone();
                transport_button(
                    &colors,
                    "player-next",
                    IconKind::Next,
                    ICON_SIDE,
                    side_color,
                    TRANSPORT_SIDE,
                    TransportKind::Side,
                    false,
                    move |_, _, cx| {
                        let _ = cx.update_entity(&m, |s, c| {
                            s.next_track();
                            c.notify();
                        });
                    },
                )
            });

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .p(px(12.0))
            .bg(colors.bg_elevated)
            .border_t_1()
            .border_color(colors.separator)
            .text_color(colors.text_primary)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(8.0))
                    .w_full()
                    .min_w(px(0.0))
                    .child(
                        div()
                            .flex_none()
                            .w(px(40.0))
                            .text_xs()
                            .text_color(colors.text_secondary)
                            .child(fmt_time(shown_secs)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .child(Slider::new(&self.progress).disabled(!seekable)),
                    )
                    .child(
                        div()
                            .flex_none()
                            .w(px(40.0))
                            .text_xs()
                            .text_color(colors.text_secondary)
                            .child(fmt_time(dur)),
                    ),
            )
            .child(controls)
    }
}

#[derive(Clone, Copy)]
enum IconKind {
    Prev,
    Next,
    Play,
    Pause,
}

#[derive(Clone, Copy)]
enum TransportKind {
    Side,
    Main,
}

fn icon_path(kind: IconKind) -> &'static str {
    match kind {
        IconKind::Prev => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/skip-previous.svg"),
        IconKind::Next => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/skip-next.svg"),
        IconKind::Play => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/play.svg"),
        IconKind::Pause => concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/pause.svg"),
    }
}

/// 用组件库 `Button` 承载播放控制的圆形图标按钮。
#[allow(clippy::too_many_arguments)]
fn transport_button(
    colors: &Palette,
    id: &'static str,
    kind: IconKind,
    icon_size: f32,
    icon_color: Hsla,
    size: f32,
    transport: TransportKind,
    disabled: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let btn = Button::new(id)
        .child(
            svg()
                .path(icon_path(kind))
                .size(px(icon_size))
                .text_color(icon_color),
        )
        .disabled(disabled)
        .w(px(size))
        .h(px(size))
        .min_w(px(size))
        .min_h(px(size))
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .rounded_full()
        .on_click(on_click);

    match transport {
        TransportKind::Main => btn.bg(if disabled { colors.bg_selected } else { colors.accent }),
        TransportKind::Side => btn.bg(colors.ctrl_btn_bg),
    }
}
