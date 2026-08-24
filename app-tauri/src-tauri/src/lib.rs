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

#[tauri::command]
fn set_launch_as_administrator(enabled: bool) -> Result<(), String> {
    app_config::save(&app_config::AppConfig {
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
        ])
        .run(tauri::generate_context!())
        .expect("运行应用失败");
}
