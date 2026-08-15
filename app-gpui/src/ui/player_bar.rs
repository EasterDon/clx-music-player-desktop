use crate::model::ClxState;
use crate::ui::theme::*;
use gpui::prelude::*;
use gpui::*;

const ICON_PREV: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/skip-previous.svg");
const ICON_NEXT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/skip-next.svg");
const ICON_PLAY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/play.svg");
const ICON_PAUSE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icons/pause.svg");

const TRANSPORT_SIDE: f32 = 36.0;
const TRANSPORT_MAIN: f32 = 44.0;
const ICON_SIDE: f32 = 18.0;
const ICON_MAIN: f32 = 22.0;

pub struct PlayerBarView {
    pub model: Entity<ClxState>,
    pub progress_dragging: bool,
    pub progress_track_bounds: Option<Bounds<Pixels>>,
}

impl PlayerBarView {
    pub fn new(model: Entity<ClxState>, cx: &mut Context<Self>) -> Self {
        let _ = cx.observe(&model, |_, _, cx| {
            cx.notify();
        });
        Self {
            model,
            progress_dragging: false,
            progress_track_bounds: None,
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

fn seek_secs_at(pos: Point<Pixels>, bounds: Bounds<Pixels>, dur: f64, max_seek: f64) -> f64 {
    if dur <= 0.0 {
        return 0.0;
    }
    let width = bounds.size.width.to_f64();
    if width <= 0.0 {
        return 0.0;
    }
    let x = (pos.x - bounds.origin.x).to_f64();
    let frac = (x / width).clamp(0.0, 1.0);
    (frac * dur).min(max_seek.max(0.0))
}

struct ProgressTrackElement {
    player: Entity<PlayerBarView>,
    model: Entity<ClxState>,
    play_frac: f32,
    buffered_frac: f32,
    dur: f64,
    max_seek: f64,
    interactive: bool,
}

impl Element for ProgressTrackElement {
    type RequestLayoutState = ();
    type PrepaintState = Option<Hitbox>;

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
        style.flex_grow = 1.0;
        style.flex_shrink = 1.0;
        style.flex_basis = relative(0.).into();
        style.size.width = relative(1.).into();
        style.size.height = px(16.).into();
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
        let _ = self.player.update(cx, |player, _| {
            player.progress_track_bounds = Some(bounds);
        });
        if self.interactive {
            Some(window.insert_hitbox(bounds, HitboxBehavior::Normal))
        } else {
            None
        }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        hitbox: &mut Self::PrepaintState,
        window: &mut Window,
        _cx: &mut App,
    ) {
        let bar_h = px(6.0);
        let bar_top = bounds.top() + (bounds.size.height - bar_h) / 2.;
        let bar_bounds = Bounds::new(
            point(bounds.left(), bar_top),
            size(bounds.size.width, bar_h),
        );
        window.paint_quad(fill(bar_bounds, rgb(PROGRESS_TRACK)));

        let buffered_w = bounds.size.width * self.buffered_frac;
        if buffered_w > px(0.0) {
            let buffered_bounds = Bounds::new(
                point(bounds.left(), bar_top),
                size(buffered_w.max(px(2.0)), bar_h),
            );
            window.paint_quad(fill(buffered_bounds, rgb(PROGRESS_BUFFERED)));
        }

        let fill_w = bounds.size.width * self.play_frac;
        if fill_w > px(0.0) {
            let fill_bounds = Bounds::new(
                point(bounds.left(), bar_top),
                size(fill_w.max(px(2.0)), bar_h),
            );
            window.paint_quad(fill(fill_bounds, rgb(PROGRESS_FILL)));
        }

        let Some(hitbox) = hitbox.clone() else {
            return;
        };

        let player = self.player.clone();
        let model = self.model.clone();
        let dur = self.dur;
        let max_seek = self.max_seek;

        window.on_mouse_event({
            let hitbox = hitbox.clone();
            let player = player.clone();
            let model = model.clone();
            move |event: &MouseDownEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble
                    || event.button != MouseButton::Left
                    || !hitbox.is_hovered(window)
                {
                    return;
                }
                let Some(bounds) =
                    cx.read_entity(&player, |p, _| p.progress_track_bounds)
                else {
                    return;
                };
                let sec = seek_secs_at(event.position, bounds, dur, max_seek);
                let _ = cx.update_entity(&player, |p, c| {
                    p.progress_dragging = true;
                    c.notify();
                });
                let _ = cx.update_entity(&model, |st, c| {
                    st.seek(sec);
                    c.notify();
                });
            }
        });

        window.on_mouse_event({
            let player = player.clone();
            move |_: &MouseUpEvent, phase, _, cx| {
                if phase != DispatchPhase::Bubble {
                    return;
                }
                let _ = cx.update_entity(&player, |p, c| {
                    p.progress_dragging = false;
                    c.notify();
                });
            }
        });

        window.on_mouse_event({
            let player = player.clone();
            let model = model.clone();
            move |event: &MouseMoveEvent, phase, _window, cx| {
                if phase != DispatchPhase::Bubble {
                    return;
                }
                let dragging = cx.read_entity(&player, |p, _| p.progress_dragging);
                if !dragging {
                    return;
                }
                let Some(bounds) =
                    cx.read_entity(&player, |p, _| p.progress_track_bounds)
                else {
                    return;
                };
                let sec = seek_secs_at(event.position, bounds, dur, max_seek);
                let _ = cx.update_entity(&model, |st, c| {
                    st.seek(sec);
                    c.notify();
                });
            }
        });
    }
}

impl IntoElement for ProgressTrackElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Render for PlayerBarView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let player = cx.entity();
        let model = self.model.clone();

        cx.read_entity(&self.model, move |s, _| {
            let is_playing = s.is_playing;
            let ct = s.current_time;
            let dur = s.duration;
            let play_disabled = s.current.is_none();
            let m = model.clone();

            let side_icon = rgb(TEXT_SECONDARY);
            let main_icon = if play_disabled {
                rgb(TEXT_TERTIARY)
            } else {
                rgb(0xffffff)
            };

            let controls = div()
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .gap(px(20.0))
                .child({
                    let m_prev = m.clone();
                    transport_side_btn(
                        "player-prev",
                        ICON_PREV,
                        side_icon,
                        move |_, _, cx| {
                            let _ = cx.update_entity(&m_prev, |s, c| {
                                s.previous_track();
                                c.notify();
                            });
                        },
                    )
                })
                .child(if is_playing {
                    let m = m.clone();
                    transport_main_btn(
                        "player-pause",
                        ICON_PAUSE,
                        play_disabled,
                        main_icon,
                        move |_, _, cx| {
                            let _ = cx.update_entity(&m, |st, c| {
                                st.pause_playback();
                                c.notify();
                            });
                        },
                    )
                    .into_any_element()
                } else {
                    let m = m.clone();
                    transport_main_btn(
                        "player-play",
                        ICON_PLAY,
                        play_disabled,
                        main_icon,
                        move |_, _, cx| {
                            let _ = cx.update_entity(&m, |st, c| {
                                st.play_playback();
                                c.notify();
                            });
                        },
                    )
                    .into_any_element()
                })
                .child({
                    let m_next = m.clone();
                    transport_side_btn(
                        "player-next",
                        ICON_NEXT,
                        side_icon,
                        move |_, _, cx| {
                            let _ = cx.update_entity(&m_next, |s, c| {
                                s.next_track();
                                c.notify();
                            });
                        },
                    )
                });

            let play_frac = if dur > 0.0 {
                (ct / dur).clamp(0.0, 1.0) as f32
            } else {
                0.0
            };
            let max_seek = s.buffered_end_secs();
            let buffered_frac = if dur > 0.0 {
                (max_seek / dur).clamp(0.0, 1.0) as f32
            } else if s.load_total_bytes.is_some_and(|t| t > 0) {
                (s.load_bytes as f64 / s.load_total_bytes.unwrap_or(1) as f64)
                    .clamp(0.0, 1.0) as f32
            } else {
                0.0
            };

            let track = ProgressTrackElement {
                player: player.clone(),
                model: m.clone(),
                play_frac,
                buffered_frac,
                dur,
                max_seek,
                interactive: !play_disabled && dur > 0.0 && max_seek > 0.0,
            };

            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .p(px(12.0))
                .bg(rgb(BG_ELEVATED))
                .border_t_1()
                .border_color(rgb(SEPARATOR))
                .text_color(rgb(TEXT_PRIMARY))
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
                                .text_color(rgb(TEXT_SECONDARY))
                                .child(fmt_time(ct)),
                        )
                        .child(track)
                        .child(
                            div()
                                .flex_none()
                                .w(px(40.0))
                                .text_xs()
                                .text_color(rgb(TEXT_SECONDARY))
                                .child(fmt_time(dur)),
                        ),
                )
                .child(controls)
        })
    }
}

fn transport_icon(path: &'static str, size: f32, color: impl Into<Hsla>) -> impl IntoElement {
    svg()
        .path(path)
        .size(px(size))
        .text_color(color)
}

fn transport_side_btn(
    id: &'static str,
    icon_path: &'static str,
    icon_color: impl Into<Hsla>,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .w(px(TRANSPORT_SIDE))
        .h(px(TRANSPORT_SIDE))
        .min_w(px(TRANSPORT_SIDE))
        .min_h(px(TRANSPORT_SIDE))
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(rgb(CTRL_BTN_BG))
        .cursor_pointer()
        .child(transport_icon(icon_path, ICON_SIDE, icon_color))
        .id(id)
        .on_click(on_click)
}

fn transport_main_btn(
    id: &'static str,
    icon_path: &'static str,
    disabled: bool,
    icon_color: impl Into<Hsla>,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let mut btn = div()
        .w(px(TRANSPORT_MAIN))
        .h(px(TRANSPORT_MAIN))
        .min_w(px(TRANSPORT_MAIN))
        .min_h(px(TRANSPORT_MAIN))
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .rounded_full()
        .child(transport_icon(icon_path, ICON_MAIN, icon_color))
        .id(id);
    if disabled {
        btn = btn.bg(rgb(BG_SURFACE)).cursor_default();
    } else {
        btn = btn.bg(rgb(ACCENT)).cursor_pointer().on_click(on_click);
    }
    btn
}
