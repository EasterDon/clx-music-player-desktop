//! 可被 Tauri、GPUI 等壳复用的应用逻辑。

use enigo::{Direction, Key, Keyboard, Settings};
use std::sync::{Arc, Mutex};

pub mod app_config;
pub mod elevation;

pub use enigo::{Enigo, InputError, NewConError};

/// 可在线程间共享的 Enigo 实例；Tauri 的 `State` 或 GPUI 模型均可持有同构类型。
pub type SharedEnigo = Arc<Mutex<Enigo>>;

pub fn new_shared_enigo() -> Result<SharedEnigo, NewConError> {
    Ok(Arc::new(Mutex::new(Enigo::new(&Settings::default())?)))
}

fn first_char(key: &str) -> char {
    key.chars().next().expect("按键字符串不能为空")
}

pub fn click(enigo: &mut Enigo, key: &str) -> Result<(), InputError> {
    let s = first_char(key);
    enigo.key(Key::Unicode(s), Direction::Click)
}

pub fn click_down(enigo: &mut Enigo, key: &str) -> Result<(), InputError> {
    let s = first_char(key);
    enigo.key(Key::Unicode(s), Direction::Press)
}

pub fn click_up(enigo: &mut Enigo, key: &str) -> Result<(), InputError> {
    let s = first_char(key);
    enigo.key(Key::Unicode(s), Direction::Release)
}
