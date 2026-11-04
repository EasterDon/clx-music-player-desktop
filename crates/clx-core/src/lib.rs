//! 可被 Tauri、GPUI 等壳复用的应用逻辑。

use enigo::{Direction, Key, Keyboard, Settings};
use std::sync::{Arc, Mutex};

pub mod app_config;
pub mod elevation;
pub mod fkey_hint;

pub use enigo::{Enigo, InputError, NewConError};

/// 可在线程间共享的 Enigo 实例；Tauri 的 `State` 或 GPUI 模型均可持有同构类型。
pub type SharedEnigo = Arc<Mutex<Enigo>>;

pub fn new_shared_enigo() -> Result<SharedEnigo, NewConError> {
    Ok(Arc::new(Mutex::new(Enigo::new(&Settings::default())?)))
}

fn first_char(key: &str) -> char {
    key.chars().next().expect("按键字符串不能为空")
}

/// 在主线程上执行按键模拟，并同步等待结果。
///
/// macOS 上 enigo 把字符换算成键码时会调用 Text Input Source（TIS/TSM）接口，
/// 这些接口内部有 `dispatch_assert_queue` 断言，要求在框架指定的队列上调用。
/// 自动演奏跑在后台线程里，直接调用会触发 SIGTRAP 崩溃
/// （崩溃栈：`TSMGetInputSourceProperty` -> `dispatch_assert_queue_fail`）。
///
/// 因此 macOS 下统一派发到主队列执行；其它平台直接在当前线程执行，
/// 避免为 Windows/Linux 引入额外依赖（Windows 上跨线程模拟输入本就不可靠，
/// 但那里的调用方本就在同一线程创建并使用 Enigo）。
#[cfg(target_os = "macos")]
fn on_main_thread<R: Send>(f: impl FnOnce() -> R + Send) -> R {
    use dispatch2::DispatchQueue;
    use std::sync::mpsc;

    // 已在主线程时直接执行：同步派发到自身会死锁。
    if is_main_thread() {
        return f();
    }

    let (tx, rx) = mpsc::channel();
    DispatchQueue::main().exec_sync(move || {
        let _ = tx.send(f());
    });
    rx.recv().expect("主线程派发的任务被丢弃")
}

#[cfg(target_os = "macos")]
fn is_main_thread() -> bool {
    extern "C" {
        fn pthread_main_np() -> std::os::raw::c_int;
    }
    // SAFETY: pthread_main_np 是 libSystem 提供的无副作用查询函数。
    unsafe { pthread_main_np() != 0 }
}

#[cfg(not(target_os = "macos"))]
fn on_main_thread<R: Send>(f: impl FnOnce() -> R + Send) -> R {
    f()
}

pub fn click(enigo: &mut Enigo, key: &str) -> Result<(), InputError> {
    let s = first_char(key);
    on_main_thread(move || enigo.key(Key::Unicode(s), Direction::Click))
}

pub fn click_down(enigo: &mut Enigo, key: &str) -> Result<(), InputError> {
    let s = first_char(key);
    on_main_thread(move || enigo.key(Key::Unicode(s), Direction::Press))
}

pub fn click_up(enigo: &mut Enigo, key: &str) -> Result<(), InputError> {
    let s = first_char(key);
    on_main_thread(move || enigo.key(Key::Unicode(s), Direction::Release))
}
