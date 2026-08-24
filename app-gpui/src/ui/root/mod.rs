use crate::model::{ClxState, MainTask, PlayMode, UiRefresh, APP_VERSION};
use crate::services::hotkey::{self, HotkeyIds, RegisteredScriptHotkeys};
use crate::ui::lyrics_pane::LyricsPaneView;
use crate::ui::motion::{
    step_toward, DURATION_DRAWER_SECS, DURATION_LAYOUT_SECS, DURATION_MOBILE_PAGE_SECS,
};
use crate::ui::music_filter::MusicFilterInput;
use crate::ui::player_bar::PlayerBarView;
use crate::ui::root::covers::cover_ids_for_visible;
use crate::ui::root::mobile::{
    mobile_fullscreen_pane, mobile_list_back_overlay, mobile_lyrics_toggle_overlay,
    mobile_page_stack,
};
use crate::ui::root::music_list::music_list_panel;
use crate::ui::root::overlays::{script_modal, toast_overlay};
use crate::ui::settings_drawer::settings_drawer;
use crate::ui::theme::*;
use global_hotkey::GlobalHotKeyManager;
use gpui::prelude::*;
use gpui::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

mod covers;
mod mobile;
mod music_list;
mod overlays;

/// 与 `app-tauri/src/config/index.ts` 中 `desktop_breakpoint` 一致。
const DESKTOP_MIN_WIDTH_PX: f32 = 1020.0;

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
