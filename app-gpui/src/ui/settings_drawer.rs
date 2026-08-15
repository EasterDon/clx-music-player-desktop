use crate::model::{ClxState, PlayMode, APP_VERSION};
use crate::ui::motion::SETTINGS_DRAWER_W_PX;
use crate::ui::theme::*;
use crate::util::open_url_in_browser;
use gpui::prelude::*;
use gpui::*;

const DRAWER_PAD: f32 = 20.0;
const SECTION_GAP: f32 = 20.0;

pub fn settings_drawer(
    progress: f32,
    model: Entity<ClxState>,
    remote_ver: String,
    announcement: String,
    version_desc: String,
    download_link: String,
    has_update: bool,
    play_mode: PlayMode,
    is_elevated: bool,
    launch_as_administrator: bool,
    weak_root: WeakEntity<crate::ui::root::RootView>,
) -> impl IntoElement {
    let m_close = model.clone();
    let m_modes = model.clone();
    let link = download_link.clone();
    let scrim_a = OVERLAY_SCRIM_A * progress;
    let announcement = if announcement.trim().is_empty() {
        "暂无公告".to_string()
    } else {
        announcement
    };

    let scroll_body = div()
        .flex()
        .flex_col()
        .gap(px(SECTION_GAP))
        .pb(px(12.0))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(settings_section_title("公告"))
                .child(settings_card().child(
                    div()
                        .text_sm()
                        .text_color(rgb(TEXT_PRIMARY))
                        .child(announcement),
                )),
        )
        .child(settings_divider())
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(settings_section_title("演奏模式"))
                .child(
                    settings_card()
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(TEXT_TERTIARY))
                                .child("聆听模式仅播放音乐；演奏模式可使用 F1 / F2 快捷键自动演奏。"),
                        )
                        .child(settings_mode_segment(
                            play_mode,
                            m_modes.clone(),
                            PlayMode::Listen,
                            PlayMode::Script,
                            weak_root.clone(),
                        )),
                ),
        )
        .child(settings_divider())
        .child(settings_admin_section(
            is_elevated,
            launch_as_administrator,
            model.clone(),
        ))
        .child(settings_divider())
        .child({
            let mut version_card = settings_card();
            version_card =
                version_card.child(settings_kv_row("本地版本", APP_VERSION.to_string(), false));
            version_card = version_card.child(settings_kv_row(
                "服务端版本",
                remote_ver,
                has_update,
            ));
            if has_update {
                version_card = version_card.child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            div()
                                .px(px(8.0))
                                .py(px(3.0))
                                .rounded(px(6.0))
                                .bg(rgb(ACCENT))
                                .text_xs()
                                .text_color(rgb(0xffffff))
                                .child("有新版本"),
                        ),
                );
            }
            if !version_desc.is_empty() {
                version_card =
                    version_card.child(settings_multiline_field("更新说明", &version_desc));
            }
            if has_update && !download_link.is_empty() {
                let link_dl = link.clone();
                version_card = version_card.child(
                    div()
                        .w_full()
                        .mt(px(4.0))
                        .py(px(10.0))
                        .rounded(px(10.0))
                        .bg(rgb(ACCENT))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .text_sm()
                        .font_weight(FontWeight(600.0))
                        .text_color(rgb(0xffffff))
                        .child("下载新版本")
                        .id("settings-download")
                        .on_click(move |_, _, _| {
                            open_url_in_browser(&link_dl);
                        }),
                );
            }
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(settings_section_title("版本信息"))
                .child(version_card)
        });

    div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .occlude()
        .child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .bottom_0()
                .bg(hsla(0., 0., 0., scrim_a))
                .cursor_pointer()
                .id("settings-backdrop")
                .on_click(move |_, _, cx| {
                    let _ = cx.update_entity(&m_close, |st, c| {
                        st.show_settings = false;
                        c.notify();
                    });
                }),
        )
        .child(
            div()
                .absolute()
                .top_0()
                .left(px(-SETTINGS_DRAWER_W_PX * (1.0 - progress)))
                .bottom_0()
                .w(px(SETTINGS_DRAWER_W_PX))
                .flex()
                .flex_col()
                .bg(rgb(BG_ELEVATED))
                .border_r_1()
                .border_color(rgb(SEPARATOR))
                .shadow_lg()
                .text_color(rgb(TEXT_PRIMARY))
                .occlude()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .flex_shrink_0()
                        .px(px(DRAWER_PAD))
                        .pt(px(DRAWER_PAD))
                        .pb(px(12.0))
                        .border_b_1()
                        .border_color(rgb(SEPARATOR))
                        .child(
                            div()
                                .text_xl()
                                .font_weight(FontWeight(700.0))
                                .child("设置"),
                        )
                        .child(settings_close_button(model)),
                )
                .child(
                    div()
                        .flex_1()
                        .min_h(px(0.0))
                        .id("settings-scroll")
                        .overflow_y_scroll()
                        .px(px(DRAWER_PAD))
                        .pt(px(16.0))
                        .child(scroll_body),
                ),
        )
}

fn settings_close_button(model: Entity<ClxState>) -> impl IntoElement {
    div()
        .w(px(32.0))
        .h(px(32.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(rgb(BG_SURFACE))
        .text_sm()
        .text_color(rgb(TEXT_SECONDARY))
        .cursor_pointer()
        .child("✕")
        .id("settings-close")
        .on_click(move |_, _, cx| {
            let _ = cx.update_entity(&model, |st, c| {
                st.show_settings = false;
                c.notify();
            });
        })
}

fn settings_section_title(title: &'static str) -> impl IntoElement {
    div()
        .text_xs()
        .font_weight(FontWeight(600.0))
        .text_color(rgb(TEXT_SECONDARY))
        .child(title)
}

fn settings_card() -> Stateful<Div> {
    div()
        .id("settings-card")
        .w_full()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .p(px(14.0))
        .rounded(px(12.0))
        .bg(rgb(BG_SURFACE))
}

fn settings_divider() -> impl IntoElement {
    div().h(px(1.0)).w_full().bg(rgb(SEPARATOR))
}

fn settings_kv_row(label: &'static str, value: String, highlight: bool) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_start()
        .justify_between()
        .gap(px(12.0))
        .child(
            div()
                .flex_shrink_0()
                .text_sm()
                .text_color(rgb(TEXT_SECONDARY))
                .child(label),
        )
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .text_sm()
                .text_align(TextAlign::Right)
                .when(highlight, |d| d.text_color(rgb(BADGE)).font_weight(FontWeight(600.0)))
                .when(!highlight, |d| d.text_color(rgb(TEXT_PRIMARY)))
                .child(value),
        )
}

fn settings_multiline_field(label: &'static str, body: &str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(
            div()
                .text_xs()
                .text_color(rgb(TEXT_SECONDARY))
                .child(label),
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(TEXT_PRIMARY))
                .child(body.to_string()),
        )
}

fn settings_mode_segment(
    active: PlayMode,
    model: Entity<ClxState>,
    listen: PlayMode,
    script: PlayMode,
    weak_root: WeakEntity<crate::ui::root::RootView>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .p(px(3.0))
        .gap(px(4.0))
        .rounded(px(10.0))
        .bg(rgb(BG_ELEVATED))
        .border_1()
        .border_color(rgb(SEPARATOR))
        .child(settings_mode_button(
            "mode-listen",
            "聆听",
            active == listen,
            model.clone(),
            listen,
            weak_root.clone(),
        ))
        .child(settings_mode_button(
            "mode-script",
            "演奏",
            active == script,
            model,
            script,
            weak_root,
        ))
}

fn settings_admin_section(
    is_elevated: bool,
    launch_as_administrator: bool,
    model: Entity<ClxState>,
) -> impl IntoElement {
    let elevated_label = if is_elevated {
        "管理员"
    } else {
        "普通用户"
    };
    let elevated_highlight = is_elevated;
    let need_restart = launch_as_administrator && !is_elevated;

    let mut card = settings_card();
    card = card
        .child(
            div()
                .text_xs()
                .text_color(rgb(TEXT_TERTIARY))
                .child("如自动演奏未生效，可尝试使用管理员权限运行本软件。开启后，若当前不是管理员，下次启动会弹出 UAC；若已是管理员则不会重复提示。也可点击下方按钮立即重启。"),
        )
        .child(settings_kv_row("当前权限", elevated_label.to_string(), elevated_highlight))
        .child(settings_kv_row(
            "下次启动",
            if launch_as_administrator {
                "以管理员身份"
            } else {
                "普通用户"
            }
            .to_string(),
            launch_as_administrator,
        ))
        .child(settings_admin_toggle(
            launch_as_administrator,
            model.clone(),
        ));

    let mut section = div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(settings_section_title("权限与启动"))
        .child(card);

    if need_restart {
        let m_restart = model;
        section = section.child(
            div()
                .w_full()
                .py(px(10.0))
                .rounded(px(10.0))
                .bg(rgb(ACCENT))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .text_sm()
                .font_weight(FontWeight(600.0))
                .text_color(rgb(0xffffff))
                .child("立即以管理员重启")
                .id("settings-restart-admin")
                .on_click(move |_, _, cx| {
                    match ClxState::try_restart_as_admin() {
                        Ok(()) => {
                            let _ = cx.update_entity(&m_restart, |st, c| {
                                st.push_toast("正在以管理员身份重新启动…".into());
                                c.notify();
                            });
                            std::process::exit(0);
                        }
                        Err(e) => {
                            let _ = cx.update_entity(&m_restart, |st, c| {
                                st.push_toast(e);
                                c.notify();
                            });
                        }
                    }
                }),
        );
    }

    section
}

fn settings_admin_toggle(
    launch_as_administrator: bool,
    model: Entity<ClxState>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(
            div()
                .text_xs()
                .text_color(rgb(TEXT_SECONDARY))
                .child("以管理员身份启动"),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .p(px(3.0))
                .gap(px(4.0))
                .rounded(px(10.0))
                .bg(rgb(BG_ELEVATED))
                .border_1()
                .border_color(rgb(SEPARATOR))
                .child(settings_admin_option(
                    "admin-launch-off",
                    "关闭",
                    !launch_as_administrator,
                    model.clone(),
                    false,
                ))
                .child(settings_admin_option(
                    "admin-launch-on",
                    "开启",
                    launch_as_administrator,
                    model,
                    true,
                )),
        )
}

fn settings_admin_option(
    id: &'static str,
    label: &'static str,
    active: bool,
    model: Entity<ClxState>,
    enabled: bool,
) -> impl IntoElement {
    div()
        .flex_1()
        .py(px(8.0))
        .rounded(px(8.0))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .text_sm()
        .when(active, |d| {
            d.bg(rgb(ACCENT))
                .text_color(rgb(0xffffff))
                .font_weight(FontWeight(600.0))
        })
        .when(!active, |d| d.text_color(rgb(TEXT_SECONDARY)))
        .child(label)
        .id(id)
        .on_click(move |_, _, cx| {
            if active {
                return;
            }
            let _ = cx.update_entity(&model, |st, c| {
                match st.set_launch_as_administrator(enabled) {
                    Ok(()) => {
                        if enabled && !ClxState::is_elevated_now() {
                            st.push_toast(
                                "已开启：请点击下方按钮以管理员重启，或下次启动时自动提升。"
                                    .into(),
                            );
                        } else if !enabled && ClxState::is_elevated_now() {
                            st.push_toast(
                                "已关闭：请退出后按普通方式重新打开应用。".into(),
                            );
                        }
                    }
                    Err(e) => st.push_toast(e),
                }
                c.notify();
            });
        })
}

fn settings_mode_button(
    id: &'static str,
    label: &'static str,
    active: bool,
    model: Entity<ClxState>,
    mode: PlayMode,
    weak_root: WeakEntity<crate::ui::root::RootView>,
) -> impl IntoElement {
    div()
        .flex_1()
        .py(px(8.0))
        .rounded(px(8.0))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .text_sm()
        .when(active, |d| {
            d.bg(rgb(ACCENT))
                .text_color(rgb(0xffffff))
                .font_weight(FontWeight(600.0))
        })
        .when(!active, |d| d.text_color(rgb(TEXT_SECONDARY)))
        .child(label)
        .id(id)
        .on_click(move |_, _, cx| {
            let _ = cx.update_entity(&model, |s, c| {
                s.play_mode = mode;
                c.notify();
            });
            if let Some(root) = weak_root.upgrade() {
                let _ = cx.update_entity(&root, |r, c| {
                    r.sync_play_mode_hotkeys(mode);
                    c.notify();
                });
            }
        })
}
