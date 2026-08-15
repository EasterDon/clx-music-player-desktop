use crate::model::{ClxState, MainTask, PlayMode, UiRefresh, APP_VERSION};
use crate::services::api::MusicItem;
use crate::services::hotkey::{self, HotkeyIds, RegisteredScriptHotkeys};
use crate::ui::lyrics_pane::LyricsPaneView;
use crate::ui::motion::{
    step_toward, DURATION_DRAWER_SECS, DURATION_LAYOUT_SECS, DURATION_MOBILE_PAGE_SECS,
    MOBILE_PAGE_SLIDE_PX,
};
use crate::ui::music_filter::MusicFilterInput;
use crate::ui::player_bar::PlayerBarView;
use crate::ui::settings_drawer::settings_drawer;
use crate::ui::theme::*;
use global_hotkey::GlobalHotKeyManager;
use gpui::prelude::*;
use gpui::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// 与 `app-tauri/src/config/index.ts` 中 `desktop_breakpoint` 一致。
const DESKTOP_MIN_WIDTH_PX: f32 = 1020.0;

/// 网格卡片固定外宽、外高（含 padding / 描边取整）。
const GRID_CARD_FIX_W: f32 = 138.0;
const GRID_CARD_FIX_H: f32 = 196.0;
/// 封面正方形边长（落在卡片内容区内）。
const GRID_CARD_COVER_SZ: f32 = 112.0;
/// 网格列/行间距（与列表 `flex-wrap` 容器 `.gap(px(...))` 一致）。
const GRID_GAP: f32 = 8.0;

pub struct RootView {
    focus_handle: FocusHandle,
    model: Entity<ClxState>,
    player_bar: Entity<PlayerBarView>,
    lyrics_pane: Entity<LyricsPaneView>,
    hotkey_mgr: GlobalHotKeyManager,
    script_hotkeys: Option<RegisteredScriptHotkeys>,
    hotkey_ids: Arc<Mutex<Option<HotkeyIds>>>,
    music_filter: Entity<MusicFilterInput>,
    /// 窄屏时：false 为列表页，true 为歌词页（对齐 App.vue `mobile_show_lyrics`）。
    mobile_show_lyrics: bool,
    /// 曲目列表滚动容器（用于按视口懒加载封面）。
    music_list_scroll: ScrollHandle,
    initial_viewport_covers: bool,
    /// 设置抽屉 0=关 1=开（插值，支持关闭动画）。
    settings_motion: f32,
    /// 窄屏歌词页 0=列表 1=歌词。
    mobile_lyrics_motion: f32,
    /// 桌面布局 0=窄屏单列 1=宽屏分栏。
    layout_desktop_motion: f32,
    layout_desktop_target: f32,
    /// 首帧将 motion 与当前窗口尺寸对齐，避免启动时布局插值为 0。
    motion_bootstrapped: bool,
}

fn dispatch_ui_refresh(
    refresh: UiRefresh,
    weak_root: &WeakEntity<RootView>,
    player_bar: &Entity<PlayerBarView>,
    lyrics_pane: &Entity<LyricsPaneView>,
    async_cx: &mut AsyncApp,
) {
    match refresh {
        UiRefresh::None => {}
        UiRefresh::Covers => {
            if let Some(root) = weak_root.upgrade() {
                let _ = async_cx.update_entity(&root, |_, c| c.notify());
            }
        }
        UiRefresh::Playback => {
            let _ = async_cx.update_entity(player_bar, |_, c| c.notify());
            let _ = async_cx.update_entity(lyrics_pane, |_, c| c.notify());
        }
        UiRefresh::Full => {
            if let Some(root) = weak_root.upgrade() {
                let _ = async_cx.update_entity(&root, |_, c| c.notify());
            }
            let _ = async_cx.update_entity(player_bar, |_, c| c.notify());
            let _ = async_cx.update_entity(lyrics_pane, |_, c| c.notify());
        }
    }
}

impl RootView {
    pub fn new(
        model: Entity<ClxState>,
        task_rx: std::sync::mpsc::Receiver<MainTask>,
        hotkey_mgr: GlobalHotKeyManager,
        hotkey_ids: Arc<Mutex<Option<HotkeyIds>>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus_handle = cx.focus_handle();
        focus_handle.focus(window);
        let player_bar = cx.new(|cx| PlayerBarView::new(model.clone(), cx));
        let lyrics_pane =
            cx.new(|cx| LyricsPaneView::new(model.clone(), cx));

        let m_poll = model.clone();
        let weak_root = cx.weak_entity();
        let hotkey_ids_poll = hotkey_ids.clone();
        let music_filter = cx.new(|cx| {
            MusicFilterInput::new(model.clone(), weak_root.clone(), cx)
        });
        let player_spawn = player_bar.clone();
        let lyrics_spawn = lyrics_pane.clone();
        cx.spawn(async move |_, async_cx| {
            loop {
                let _ = async_cx
                    .background_executor()
                    .timer(Duration::from_millis(50))
                    .await;
                while let Ok(task) = task_rx.try_recv() {
                    let refresh = async_cx
                        .update_entity(&m_poll, |s, _| s.apply_main_task(task))
                        .unwrap_or(UiRefresh::None);
                    dispatch_ui_refresh(
                        refresh,
                        &weak_root,
                        &player_spawn,
                        &lyrics_spawn,
                        async_cx,
                    );
                }
                if let Ok(guard) = hotkey_ids_poll.lock() {
                    if let Some(ref ids) = *guard {
                        for k in hotkey::poll_hotkeys(ids) {
                            let refresh = async_cx
                                .update_entity(&m_poll, |s, _| {
                                    s.apply_main_task(MainTask::Hotkey(k))
                                })
                                .unwrap_or(UiRefresh::None);
                            dispatch_ui_refresh(
                                refresh,
                                &weak_root,
                                &player_spawn,
                                &lyrics_spawn,
                                async_cx,
                            );
                        }
                    }
                }
                let toast_dismissed = async_cx
                    .update_entity(&m_poll, |s, _| s.tick_toast_auto_dismiss())
                    .unwrap_or(false);
                if toast_dismissed {
                    dispatch_ui_refresh(
                        UiRefresh::Full,
                        &weak_root,
                        &player_spawn,
                        &lyrics_spawn,
                        async_cx,
                    );
                }
            }
        })
        .detach();

        let weak_motion = cx.weak_entity();
        cx.spawn(async move |_, async_cx| {
            loop {
                let _ = async_cx
                    .background_executor()
                    .timer(Duration::from_millis(16))
                    .await;
                let Some(root) = weak_motion.upgrade() else {
                    break;
                };
                let _ = async_cx.update_entity(&root, |this, cx| {
                    if this.tick_motions(0.016, cx) {
                        cx.notify();
                    }
                });
            }
        })
        .detach();

        let mut this = Self {
            focus_handle,
            model,
            player_bar,
            lyrics_pane,
            hotkey_mgr,
            script_hotkeys: None,
            hotkey_ids,
            music_filter,
            mobile_show_lyrics: false,
            music_list_scroll: ScrollHandle::new(),
            initial_viewport_covers: false,
            settings_motion: 0.0,
            mobile_lyrics_motion: 0.0,
            layout_desktop_motion: 0.0,
            layout_desktop_target: 0.0,
            motion_bootstrapped: false,
        };
        this.sync_play_mode_hotkeys(PlayMode::Listen);
        this
    }

    fn tick_motions(&mut self, dt: f32, cx: &mut Context<Self>) -> bool {
        let settings_target = if cx.read_entity(&self.model, |s, _| s.show_settings) {
            1.0
        } else {
            0.0
        };
        let mobile_target = if self.mobile_show_lyrics { 1.0 } else { 0.0 };

        let mut animating = false;
        let (v, a) = step_toward(
            self.settings_motion,
            settings_target,
            dt,
            DURATION_DRAWER_SECS,
        );
        self.settings_motion = v;
        animating |= a;

        let (v, a) = step_toward(
            self.mobile_lyrics_motion,
            mobile_target,
            dt,
            DURATION_MOBILE_PAGE_SECS,
        );
        self.mobile_lyrics_motion = v;
        animating |= a;

        let (v, a) = step_toward(
            self.layout_desktop_motion,
            self.layout_desktop_target,
            dt,
            DURATION_LAYOUT_SECS,
        );
        self.layout_desktop_motion = v;
        animating |= a;

        animating
    }

    /// 按播放模式注册全局热键：聆听不占用；演奏为 F1/F2。
    pub fn sync_play_mode_hotkeys(&mut self, mode: PlayMode) {
        match mode {
            PlayMode::Listen => {
                if let Some(keys) = self.script_hotkeys.take() {
                    if let Err(e) = hotkey::unregister_script_hotkeys(&self.hotkey_mgr, &keys) {
                        eprintln!("{e}");
                    }
                    if let Ok(mut guard) = self.hotkey_ids.lock() {
                        *guard = None;
                    }
                }
            }
            PlayMode::Script => {
                if self.script_hotkeys.is_some() {
                    return;
                }
                match hotkey::register_script_hotkeys(&self.hotkey_mgr) {
                    Ok(keys) => {
                        let ids = keys.ids();
                        if let Ok(mut guard) = self.hotkey_ids.lock() {
                            *guard = Some(ids);
                        }
                        self.script_hotkeys = Some(keys);
                    }
                    Err(e) => eprintln!("{e}"),
                }
            }
        }
    }
}

/// 固定宽卡片 + `gap` 时，宽度 `w` 下每行约可放几列（与 `flex-wrap` 换行一致，供封面预取估算行号）。
fn grid_columns_for_width(w: Pixels) -> usize {
    let w = w.max(px(0.0)).to_f64() as f32;
    let slot = GRID_CARD_FIX_W + GRID_GAP;
    if w < slot {
        return 1;
    }
    (((w + GRID_GAP) / slot).floor() as i32).max(1) as usize
}

/// 列表行高约估（含 `gap(6)`）；与 `music_list_item` 窄屏行高同量级即可。
const LIST_COVER_STRIDE_PX: f32 = 90.0;
/// 网格行高约估（封面 + 文 + padding + `gap(8)`）。
const GRID_ROW_STRIDE_PX: f32 = GRID_CARD_FIX_H + GRID_GAP;
const COVER_PREFETCH_OVERSCAN_PX: f32 = 160.0;

/// 按当前滚动与视口估算应预取的封面 id（`scroll_offset_y` 为 `ScrollHandle::offset().y`）。
fn cover_ids_for_visible(
    items: &[MusicItem],
    grid: bool,
    grid_content_width: Pixels,
    scroll_offset_y: f32,
    viewport_h: f32,
) -> Vec<i64> {
    if items.is_empty() {
        return Vec::new();
    }
    let n = items.len();
    let scroll_top = (-scroll_offset_y).max(0.0);
    let top = scroll_top - COVER_PREFETCH_OVERSCAN_PX;
    let bot = scroll_top + viewport_h + COVER_PREFETCH_OVERSCAN_PX;

    if !grid {
        let i0 = ((top / LIST_COVER_STRIDE_PX).floor() as isize).max(0) as usize;
        let i1 = ((bot / LIST_COVER_STRIDE_PX).ceil() as isize).max(0) as usize;
        let i1 = i1.min(n - 1);
        if i0 > i1 {
            return Vec::new();
        }
        return items[i0..=i1].iter().map(|m| m.id).collect();
    }

    let per_row = grid_columns_for_width(grid_content_width).max(1);
    let r0 = ((top / GRID_ROW_STRIDE_PX).floor() as isize).max(0) as usize;
    let r1 = ((bot / GRID_ROW_STRIDE_PX).ceil() as isize).max(0) as usize;
    let i0 = (r0 * per_row).min(n.saturating_sub(1));
    let i1_ex = ((r1 + 1) * per_row).min(n);
    if i0 >= i1_ex {
        return Vec::new();
    }
    items[i0..i1_ex].iter().map(|m| m.id).collect()
}

impl Render for RootView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let is_desktop = window.bounds().size.width >= px(DESKTOP_MIN_WIDTH_PX);
        self.layout_desktop_target = if is_desktop { 1.0 } else { 0.0 };

        if !self.motion_bootstrapped {
            self.layout_desktop_motion = self.layout_desktop_target;
            self.mobile_lyrics_motion = if self.mobile_show_lyrics {
                1.0
            } else {
                0.0
            };
            self.settings_motion = if cx.read_entity(&self.model, |s, _| s.show_settings) {
                1.0
            } else {
                0.0
            };
            self.motion_bootstrapped = true;
        }

        let settings_motion = self.settings_motion;
        let mobile_lyrics_motion = self.mobile_lyrics_motion;
        let mobile_show_lyrics = self.mobile_show_lyrics;
        let mobile_lyrics_target = if mobile_show_lyrics { 1.0 } else { 0.0 };
        let mobile_page_animating =
            (mobile_lyrics_motion - mobile_lyrics_target).abs() > 0.02;
        let layout_desktop_motion = self.layout_desktop_motion;
        let layout_desktop_target = self.layout_desktop_target;

        let open_mobile_lyrics = cx.listener(|this, _: &ClickEvent, _, cx| {
            this.mobile_show_lyrics = true;
            cx.notify();
        });
        let back_to_music_list = cx.listener(|this, _: &ClickEvent, _, cx| {
            this.mobile_show_lyrics = false;
            cx.notify();
        });

        let model = self.model.clone();
        let music_filter_entity = self.music_filter.clone();
        let player_bar_entity = self.player_bar.clone();
        let lyrics_pane_entity = self.lyrics_pane.clone();

        if !self.initial_viewport_covers {
            self.initial_viewport_covers = true;
            let is_desktop_w = is_desktop;
            let grid_w = (window.bounds().size.width
                - px(480.0)
                - px(1.0)
                - px(20.0)
                - px(40.0))
            .max(px(0.0));
            let sy = self.music_list_scroll.offset().y.to_f64() as f32;
            let vh = self
                .music_list_scroll
                .bounds()
                .size
                .height
                .to_f64() as f32;
            let _ = cx.update_entity(&model, |st, c| {
                let ids = cover_ids_for_visible(
                    &st.filtered_display,
                    is_desktop_w,
                    grid_w,
                    sy,
                    vh.max(1.0),
                );
                st.enqueue_music_cover_fetches(ids);
                c.notify();
            });
        }

        let focus_handle = self.focus_handle.clone();
        let weak_root_settings = cx.weak_entity();
        let music_list_scroll_handle = self.music_list_scroll.clone();
        let prefetch_music_covers_on_scroll = cx.listener({
            let model = model.clone();
            let scroll_h = music_list_scroll_handle.clone();
            move |_, _: &ScrollWheelEvent, window, cx| {
                let is_desktop_w = window.bounds().size.width >= px(DESKTOP_MIN_WIDTH_PX);
                let grid_w = (window.bounds().size.width
                    - px(480.0)
                    - px(1.0)
                    - px(20.0)
                    - px(40.0))
                .max(px(0.0));
                let sy = scroll_h.offset().y.to_f64() as f32;
                let vh = scroll_h.bounds().size.height.to_f64() as f32;
                let _ = cx.update_entity(&model, |st, c| {
                    let ids = cover_ids_for_visible(
                        &st.filtered_display,
                        is_desktop_w,
                        grid_w,
                        sy,
                        vh.max(1.0),
                    );
                    st.enqueue_music_cover_fetches(ids);
                    c.notify();
                });
            }
        });

        cx.read_entity(&self.model, move |s, _| {
            let list = s.filtered_display.clone();
            let current_id = s.current.as_ref().map(|m| m.id);
            let modal_script = s.modal_script;
            let toast = s.toast.clone();
            let cur = s.current.clone();
            let play_mode = s.play_mode;
            let launch_as_administrator = s.launch_as_administrator;
            let is_elevated = ClxState::is_elevated_now();
            let update_badge = s.app_info.app_version != APP_VERSION;
            let announcement = s.app_info.announcement.clone();
            let remote_ver = s.app_info.app_version.clone();
            let version_desc = s.app_info.app_version_description.clone();
            let download_link = s.app_info.app_download_link.clone();
            let music_covers = Arc::clone(&s.music_covers);
            let music_cover_failed = s.music_cover_failed.clone();

            let list_pane = {
                let m = model.clone();
                let list_header = {
                    let m2 = m.clone();
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_center()
                        .gap(px(12.0))
                        .w_full()
                        .flex_shrink_0()
                        .when(!is_desktop, |h| h.px(px(10.0)).pt(px(10.0)).pb(px(8.0)))
                        .child({
                            let m3 = m2.clone();
                            div()
                                .cursor_pointer()
                                .text_xl()
                                .child("☰")
                                .text_color(rgb(TEXT_PRIMARY))
                                .when(update_badge, |d| d.text_color(rgb(BADGE)))
                                .id("settings-menu")
                                .on_click(move |_, _, cx| {
                                    let _ = cx.update_entity(&m3, |st, c| {
                                        st.show_settings = !st.show_settings;
                                        c.notify();
                                    });
                                })
                        })
                        .child(music_filter_entity.clone())
                };

                let list_scroll = div()
                    .flex_1()
                    .min_h(px(0.0))
                    .min_w(px(0.0))
                    .w_full()
                    .id("scroll-music-list")
                    .overflow_y_scroll()
                    .track_scroll(&music_list_scroll_handle)
                    .on_scroll_wheel(prefetch_music_covers_on_scroll)
                    .when(!is_desktop, |s| s.px(px(10.0)))
                    .child(
                        div()
                            .w_full()
                            .min_w(px(0.0))
                            .child(music_list_panel(
                                list,
                                current_id,
                                m.clone(),
                                is_desktop,
                                music_covers,
                                music_cover_failed,
                            )),
                    );

                let mut pane = div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h(px(0.0))
                    .min_w(px(0.0))
                    .w_full()
                    .text_color(rgb(TEXT_PRIMARY))
                    .when(is_desktop, |d| d.gap(px(8.0)).p(px(10.0)))
                    .when(!is_desktop, |d| d.h_full())
                    .child(list_header)
                    .child(list_scroll);

                if !is_desktop {
                    pane = pane.child(
                        div()
                            .w_full()
                            .flex_shrink_0()
                            .flex_none()
                            .child(player_bar_entity.clone()),
                    );
                }
                pane
            };

            let lyrics_pane = {
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h(px(0.0))
                    .min_w(px(0.0))
                    .when(!is_desktop, |d| d.w_full().h_full())
                    .when(is_desktop, |d| {
                        d.w(px(480.0))
                            .flex_shrink_0()
                            .bg(rgb(BG_ELEVATED))
                            .border_l_1()
                            .border_color(rgb(SEPARATOR))
                    })
                    .when(!is_desktop, |d| d.bg(rgb(BG_ELEVATED)))
                    .child(
                        div()
                            .flex_1()
                            .min_h(px(0.0))
                            .min_w(px(0.0))
                            .w_full()
                            .child(lyrics_pane_entity.clone()),
                    )
                    .child(
                        div()
                            .w_full()
                            .flex_shrink_0()
                            .flex_none()
                            .child(player_bar_entity.clone()),
                    )
            };

            let layout_animating =
                (layout_desktop_motion - layout_desktop_target).abs() > 0.02;

            let mut main_body = if is_desktop {
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .min_w(px(0.0))
                    .min_h(px(0.0))
                    .child(list_pane)
                    .child(lyrics_pane)
            } else {
                let mut mobile_shell = div()
                    .relative()
                    .flex_1()
                    .min_w(px(0.0))
                    .min_h(px(0.0))
                    .overflow_hidden();
                if mobile_page_animating {
                    mobile_shell = mobile_shell.child(mobile_page_stack(
                        list_pane,
                        lyrics_pane,
                        mobile_lyrics_motion,
                    ));
                } else if mobile_show_lyrics {
                    mobile_shell = mobile_shell.child(mobile_fullscreen_pane(lyrics_pane));
                } else {
                    mobile_shell = mobile_shell.child(mobile_fullscreen_pane(list_pane));
                }
                mobile_shell
            };

            if layout_animating {
                let fade = 0.86
                    + 0.14
                        * (1.0
                            - (layout_desktop_motion - layout_desktop_target).abs());
                main_body = main_body.opacity(fade.clamp(0.86, 1.0));
            }

            div()
                .track_focus(&focus_handle)
                .flex()
                .flex_col()
                .size_full()
                .bg(rgb(BG_APP))
                .text_color(rgb(TEXT_PRIMARY))
                .relative()
                .child(main_body)
                .when(!is_desktop && mobile_lyrics_motion < 0.5, |root| {
                    root.child(mobile_lyrics_toggle_overlay(open_mobile_lyrics))
                })
                .when(!is_desktop && mobile_lyrics_motion >= 0.5, |root| {
                    root.child(mobile_list_back_overlay(back_to_music_list))
                })
                .when(settings_motion > 0.001, |root| {
                    root.child(settings_drawer(
                        settings_motion,
                        model.clone(),
                        remote_ver,
                        announcement,
                        version_desc,
                        download_link,
                        update_badge,
                        play_mode,
                        is_elevated,
                        launch_as_administrator,
                        weak_root_settings.clone(),
                    ))
                })
                .when(modal_script, |root| {
                    let title = cur.as_ref().map(|c| c.name.clone()).unwrap_or_default();
                    let author = cur.as_ref().map(|c| c.author.clone()).unwrap_or_default();
                    root.child(script_modal(model.clone(), title, author))
                })
                .when(toast.is_some(), |root| {
                    let msg = toast.clone().unwrap_or_default();
                    let m = model.clone();
                    root.child(toast_overlay(msg, move |_, _, cx| {
                        let _ = cx.update_entity(&m, |st, c| {
                            st.dismiss_toast();
                            c.notify();
                        });
                    }))
                })
        })
    }
}

/// 移动端浮层图标按钮尺寸（不参与 flex 布局，仅 absolute 定位）。
const MOBILE_FLOAT_BTN: f32 = 32.0;
/// 相对窗口底边，贴近播放条控制钮行（播放条总高约 88px）。
const MOBILE_LYRICS_TOGGLE_BOTTOM: f32 = 14.0;

fn mobile_floating_icon_btn(
    id: &'static str,
    icon: &'static str,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    div()
        .w(px(MOBILE_FLOAT_BTN))
        .h(px(MOBILE_FLOAT_BTN))
        .min_w(px(MOBILE_FLOAT_BTN))
        .min_h(px(MOBILE_FLOAT_BTN))
        .max_w(px(MOBILE_FLOAT_BTN))
        .max_h(px(MOBILE_FLOAT_BTN))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .border_1()
        .border_color(rgb(SEPARATOR))
        .bg(rgb(BG_ELEVATED))
        .text_sm()
        .text_color(rgb(TEXT_SECONDARY))
        .cursor_pointer()
        .occlude()
        .child(icon)
        .id(id)
        .on_click(on_click)
}

/// 列表页：浮于播放条右侧的歌词入口（不占用播放条布局）。
fn mobile_lyrics_toggle_overlay(
    on_open_lyrics: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    mobile_floating_icon_btn("mobile-open-lyrics", "♬", on_open_lyrics)
        .absolute()
        .bottom(px(MOBILE_LYRICS_TOGGLE_BOTTOM))
        .right(px(10.0))
}

/// 歌词页：浮于左上角的返回列表入口（不占用歌词区布局）。
fn mobile_list_back_overlay(
    on_back: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    mobile_floating_icon_btn("mobile-back-list", "‹", on_back)
        .absolute()
        .top(px(10.0))
        .left(px(10.0))
}

/// 窄屏静止态：单页铺满主区域（与改版前结构一致，避免高度塌陷）。
fn mobile_fullscreen_pane(content: impl IntoElement) -> impl IntoElement {
    div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .child(content)
}

/// 窄屏切换中：列表 / 歌词叠层（滑动 + 淡入淡出）。
fn mobile_page_stack(
    list_pane: impl IntoElement,
    lyrics_pane: impl IntoElement,
    mobile_t: f32,
) -> impl IntoElement {
    let list_opacity = 1.0 - mobile_t;
    let lyrics_opacity = mobile_t;
    let list_shift = MOBILE_PAGE_SLIDE_PX * mobile_t;
    let lyrics_shift = MOBILE_PAGE_SLIDE_PX * (1.0 - mobile_t);

    let lyrics_layer = div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .opacity(lyrics_opacity)
        .left(px(-lyrics_shift))
        .child(lyrics_pane);

    let list_layer = div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .opacity(list_opacity)
        .left(px(list_shift))
        .child(list_pane);

    let mut stack = div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .overflow_hidden();

    if mobile_t < 0.5 {
        stack = stack.child(lyrics_layer).child(list_layer.occlude());
    } else {
        stack = stack.child(list_layer).child(lyrics_layer.occlude());
    }
    stack
}

/// 大屏：`flex` + `flex-wrap` + `gap`，由布局按真实宽度换行（对齐浏览器里 gap + wrap 的体验）。
/// 窄屏：纵向列表（与 `App.vue` 窄屏列表一致）。
fn music_list_panel(
    items: Vec<MusicItem>,
    current_id: Option<i64>,
    model: Entity<ClxState>,
    grid: bool,
    music_covers: Arc<HashMap<i64, Arc<Image>>>,
    music_cover_failed: HashSet<i64>,
) -> impl IntoElement {
    if grid {
        let covers_grid = Arc::clone(&music_covers);
        let failed_grid = music_cover_failed.clone();
        let mut grid_el = div()
            .w_full()
            .min_w(px(0.0))
            .flex()
            .flex_row()
            .flex_wrap()
            .justify_center()
            .gap(px(8.0))
            .children(items.into_iter().map(move |item| {
                let sel = current_id == Some(item.id);
                let loaded = covers_grid.get(&item.id).cloned();
                let cover_failed = failed_grid.contains(&item.id);
                music_list_item(
                    item,
                    sel,
                    model.clone(),
                    true,
                    loaded,
                    cover_failed,
                )
            }));
        grid_el.style().align_items = Some(AlignItems::FlexStart);
        grid_el
    } else {
        div()
            .w_full()
            .min_w(px(0.0))
            .flex()
            .flex_col()
            .gap(px(6.0))
            .children(items.into_iter().map(move |item| {
                let sel = current_id == Some(item.id);
                let loaded = music_covers.get(&item.id).cloned();
                let cover_failed = music_cover_failed.contains(&item.id);
                music_list_item(
                    item,
                    sel,
                    model.clone(),
                    false,
                    loaded,
                    cover_failed,
                )
            }))
    }
}

/// 网格顶区封面：`Img` 须固定像素尺寸 + `flex_none()`，避免解码后 `aspect_ratio` 与百分比宽把布局撑爆。
fn music_item_cover_square_top(
    loaded: Option<Arc<Image>>,
    cover_failed: bool,
    initial: SharedString,
) -> impl IntoElement {
    let init = initial.clone();
    let h = px(GRID_CARD_COVER_SZ);
    let inner = match loaded {
        Some(arc_img) => div()
            .w_full()
            .h(h)
            .overflow_hidden()
            .flex()
            .items_center()
            .justify_center()
            .child(
                img(arc_img)
                    .flex_none()
                    .object_fit(ObjectFit::Cover)
                    .w(h)
                    .h(h),
            ),
        None if cover_failed => div()
            .w_full()
            .h(h)
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(BG_PLACEHOLDER))
            .text_sm()
            .text_color(rgb(TEXT_SECONDARY))
            .child(init.clone()),
        None => div().w_full().h(h).bg(rgb(BG_PLACEHOLDER)),
    };
    div()
        .w_full()
        .h(h)
        .flex_shrink_0()
        .rounded(px(8.0))
        .overflow_hidden()
        .bg(rgb(BG_PLACEHOLDER))
        .child(inner)
}

fn music_item_cover_row_circle(
    loaded: Option<Arc<Image>>,
    cover_failed: bool,
    initial: SharedString,
) -> impl IntoElement {
    let init = initial.clone();
    let side = px(60.0);
    let inner = match loaded {
        Some(arc_img) => div()
            .w(side)
            .h(side)
            .overflow_hidden()
            .child(
                img(arc_img)
                    .flex_none()
                    .object_fit(ObjectFit::Cover)
                    .w(side)
                    .h(side),
            ),
        None if cover_failed => div()
            .w(side)
            .h(side)
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(BG_PLACEHOLDER))
            .text_xs()
            .text_color(rgb(TEXT_SECONDARY))
            .child(init.clone()),
        None => div().w(side).h(side).bg(rgb(BG_PLACEHOLDER)),
    };
    div()
        .w(side)
        .h(side)
        .flex_shrink_0()
        .rounded_full()
        .overflow_hidden()
        .bg(rgb(BG_PLACEHOLDER))
        .child(inner)
}

fn music_list_item(
    item: MusicItem,
    selected: bool,
    model: Entity<ClxState>,
    grid: bool,
    cover_loaded: Option<Arc<Image>>,
    cover_failed: bool,
) -> impl IntoElement {
    let row_id = ElementId::NamedInteger("music-item".into(), item.id as u64);
    let initial = SharedString::from(
        item.name
            .chars()
            .next()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "♪".into()),
    );

    if grid {
        let m = model.clone();
        let item_clone = item.clone();
        div()
            .w(px(GRID_CARD_FIX_W))
            .h(px(GRID_CARD_FIX_H))
            .min_w(px(GRID_CARD_FIX_W))
            .max_w(px(GRID_CARD_FIX_W))
            .min_h(px(GRID_CARD_FIX_H))
            .max_h(px(GRID_CARD_FIX_H))
            .flex_none()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(6.0))
            .p(px(8.0))
            .rounded(px(10.0))
            .overflow_hidden()
            .cursor_pointer()
            .when(selected, |d| d.bg(rgb(BG_SELECTED)))
            .when(!selected, |d| d.bg(rgb(BG_ELEVATED)))
            .id(row_id)
            .on_click(move |_, _, cx| {
                let _ = cx.update_entity(&m, |s, c| {
                    s.select_music(item_clone.clone());
                    c.notify();
                });
            })
            .child(music_item_cover_square_top(
                cover_loaded,
                cover_failed,
                initial.clone(),
            ))
            .child(
                div()
                    .w_full()
                    .min_w(px(0.0))
                    .text_center()
                    .text_xs()
                    .truncate()
                    .when(selected, |d| {
                        d.font_weight(FontWeight(600.0)).text_color(rgb(ACCENT))
                    })
                    .when(!selected, |d| d.text_color(rgb(TEXT_PRIMARY)))
                    .child(item.name.clone()),
            )
            .child(
                div()
                    .w_full()
                    .min_w(px(0.0))
                    .text_center()
                    .text_xs()
                    .text_color(rgb(TEXT_SECONDARY))
                    .truncate()
                    .child(item.author.clone()),
            )
    } else {
        let m = model;
        let item_clone = item.clone();
        div()
            .w_full()
            .min_w(px(0.0))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(20.0))
            .px(px(8.0))
            .py(px(6.0))
            .rounded(px(8.0))
            .cursor_pointer()
            .when(selected, |d| d.bg(rgb(BG_SELECTED)))
            .when(!selected, |d| d.bg(rgb(BG_ELEVATED)))
            .id(row_id)
            .on_click(move |_, _, cx| {
                let _ = cx.update_entity(&m, |s, c| {
                    s.select_music(item_clone.clone());
                    c.notify();
                });
            })
            .child(music_item_cover_row_circle(
                cover_loaded,
                cover_failed,
                initial,
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w(px(0.0))
                    .child(
                        div()
                            .text_sm()
                            .when(selected, |d| {
                                d.font_weight(FontWeight(600.0)).text_color(rgb(ACCENT))
                            })
                            .when(!selected, |d| d.text_color(rgb(TEXT_PRIMARY)))
                            .child(item.name.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(TEXT_SECONDARY))
                            .child(item.author.clone()),
                    ),
            )
    }
}

fn script_modal(_model: Entity<ClxState>, title: String, author: String) -> impl IntoElement {
    div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(hsla(0., 0., 0., OVERLAY_SCRIM_A))
        .child(
            div()
                .w(px(400.0))
                .bg(rgb(BG_MODAL))
                .border_1()
                .border_color(rgb(SEPARATOR))
                .rounded(px(14.0))
                .p(px(24.0))
                .flex()
                .flex_col()
                .gap(px(12.0))
                .items_center()
                .text_color(rgb(TEXT_PRIMARY))
                .child(div().font_weight(FontWeight(700.0)).child("自动演奏中…"))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(TEXT_SECONDARY))
                        .child(format!("当前演奏歌曲：{title}")),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(TEXT_SECONDARY))
                        .child(format!("作者：{author}")),
                )
                .child(
                    div()
                        .mt(px(8.0))
                        .text_xs()
                        .text_color(rgb(TEXT_TERTIARY))
                        .child("按 F2 可停止"),
                ),
        )
}

fn toast_overlay(
    msg: String,
    dismiss: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .absolute()
        .bottom(px(24.0))
        .left_0()
        .right_0()
        .flex()
        .flex_col()
        .items_center()
        .child(
            div()
                .max_w(px(520.0))
                .bg(rgb(BG_SURFACE))
                .text_color(rgb(TEXT_PRIMARY))
                .rounded(px(20.0))
                .px(px(16.0))
                .py(px(10.0))
                .text_sm()
                .cursor_pointer()
                .child(msg)
                .id("toast-dismiss")
                .on_click(dismiss),
        )
}
