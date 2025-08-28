use enigo::{Enigo, Key, Keyboard, Settings};
use std::sync::{Arc, Mutex};
use tauri::State;

// 全局状态保存 Enigo 实例
struct EnigoState(Arc<Mutex<Enigo>>);

#[tauri::command]
fn click(enigo_state: State<EnigoState>, key: &str) {
    let s: char = key.chars().next().unwrap();
    let mut enigo = enigo_state.0.lock().unwrap();
    enigo.key(Key::Unicode(s), enigo::Direction::Click).unwrap();
}

#[tauri::command]
fn click_down(enigo_state: State<EnigoState>, key: &str) {
    let s: char = key.chars().next().unwrap();
    let mut enigo = enigo_state.0.lock().unwrap();
    enigo.key(Key::Unicode(s), enigo::Direction::Press).unwrap();
}

#[tauri::command]
fn click_up(enigo_state: State<EnigoState>, key: &str) {
    let s: char = key.chars().next().unwrap();
    let mut enigo = enigo_state.0.lock().unwrap();
    enigo
        .key(Key::Unicode(s), enigo::Direction::Release)
        .unwrap();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .manage(EnigoState(Arc::new(Mutex::new(
            Enigo::new(&Settings::default()).unwrap(),
        ))))
        .invoke_handler(tauri::generate_handler![click, click_down, click_up])
        .run(tauri::generate_context!())
        .expect("运行应用失败");
}
