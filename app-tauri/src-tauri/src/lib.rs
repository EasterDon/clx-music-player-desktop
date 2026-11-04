use clx_core::SharedEnigo;
use clx_core::{app_config, elevation};
use serde::Serialize;
use tauri::State;

struct EnigoState(SharedEnigo);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ElevationState {
    is_elevated: bool,
    launch_as_administrator: bool,
}

#[tauri::command]
fn get_elevation_state() -> ElevationState {
    let cfg = app_config::load();
    ElevationState {
        is_elevated: elevation::is_elevated(),
        launch_as_administrator: cfg.launch_as_administrator,
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FkeyHint {
    /// 是否需要配合 `fn`（macOS 未启用「标准功能键」时为 true）。
    needs_fn: bool,
    f1_label: String,
    f2_label: String,
    start_hint: String,
    stop_hint: String,
}

/// 供前端渲染 F1/F2 提示文案；平台判断与 GPUI 端共用 `clx_core::fkey_hint`。
#[tauri::command]
fn get_fkey_hint() -> FkeyHint {
    FkeyHint {
        needs_fn: clx_core::fkey_hint::needs_fn_modifier(),
        f1_label: clx_core::fkey_hint::fkey_label("F1"),
        f2_label: clx_core::fkey_hint::fkey_label("F2"),
        start_hint: clx_core::fkey_hint::start_script_hint(),
        stop_hint: clx_core::fkey_hint::stop_script_hint(),
    }
}

#[tauri::command]
fn set_launch_as_administrator(enabled: bool) -> Result<(), String> {    app_config::save(&app_config::AppConfig {
        launch_as_administrator: enabled,
    })
}

#[tauri::command]
fn restart_as_admin(app: tauri::AppHandle) -> Result<(), String> {
    elevation::restart_as_admin()?;
    app.exit(0);
    Ok(())
}

#[tauri::command]
fn take_startup_elevation_toast() -> Option<String> {
    let cfg = app_config::load();
    elevation::startup_elevation_toast(cfg.launch_as_administrator)
}

#[tauri::command]
fn click(enigo_state: State<EnigoState>, key: String) -> Result<(), String> {
    let mut enigo = enigo_state.0.lock().map_err(|e| e.to_string())?;
    clx_core::click(&mut *enigo, &key).map_err(|e| e.to_string())
}

#[tauri::command]
fn click_down(enigo_state: State<EnigoState>, key: String) -> Result<(), String> {
    let mut enigo = enigo_state.0.lock().map_err(|e| e.to_string())?;
    clx_core::click_down(&mut *enigo, &key).map_err(|e| e.to_string())
}

#[tauri::command]
fn click_up(enigo_state: State<EnigoState>, key: String) -> Result<(), String> {
    let mut enigo = enigo_state.0.lock().map_err(|e| e.to_string())?;
    clx_core::click_up(&mut *enigo, &key).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cfg = app_config::load();
    elevation::ensure_elevated_if_configured(cfg.launch_as_administrator);
    let _ = app_config::ensure_exists();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .manage(EnigoState(
            clx_core::new_shared_enigo().expect("初始化 Enigo 失败"),
        ))
        .invoke_handler(tauri::generate_handler![
            click,
            click_down,
            click_up,
            get_elevation_state,
            set_launch_as_administrator,
            restart_as_admin,
            take_startup_elevation_toast,
            get_fkey_hint,
        ])
        .run(tauri::generate_context!())
        .expect("运行应用失败");
}
