// Release 下隐藏控制台窗口（与 app-tauri 一致）
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod assets;
mod lyrics;
mod model;
mod services;
mod ui;
mod util;

use gpui::prelude::*;
use gpui::*;
use model::MainTask;
use services::hotkey::HotkeyIds;
use std::sync::{Arc, Mutex, mpsc};
use ui::{FilterBackspace, RootView};

fn main() {
    let cfg = services::app_config::load();
    services::elevation::ensure_elevated_if_configured(cfg.launch_as_administrator);
    let _ = services::app_config::ensure_exists();

    let hotkey_mgr = match services::hotkey::try_init() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    let hotkey_ids = Arc::new(Mutex::new(None::<HotkeyIds>));

    Application::new()
        .with_assets(assets::Assets)
        .run(move |cx: &mut App| {
            cx.bind_keys([KeyBinding::new("backspace", FilterBackspace, None)]);

            let bounds = Bounds::centered(None, size(px(400.0), px(600.0)), cx);
            let (task_tx, task_rx) = mpsc::channel::<MainTask>();

            let _ = cx.open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some("一梦音乐播放器".into()),
                        appears_transparent: false,
                        ..Default::default()
                    }),
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_min_size: Some(size(px(400.0), px(600.0))),
                    ..Default::default()
                },
                move |window, cx| {
                    let model = cx.new(|_| model::ClxState::new(task_tx));
                    cx.new(|cx| {
                        RootView::new(model, task_rx, hotkey_mgr, hotkey_ids.clone(), window, cx)
                    })
                },
            );

            cx.activate(true);
        });
}
