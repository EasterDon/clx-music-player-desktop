//! 功能键（F1/F2）提示文案：按系统环境决定是否需要配合 `fn`。
//!
//! macOS 默认把 F1–F12 当作媒体键（亮度、音量等），此时需按住 `fn` 才能发出
//! 真正的 F1/F2 键码。系统设置「键盘 → 将 F1、F2 等键用作标准功能键」会把
//! `com.apple.keyboard.fnState` 置为 true，此时直接按 F1/F2 即可。
//!
//! Windows / Linux 上 F1–F12 本就是标准功能键，不需要 `fn`。
//!
//! GPUI 端直接调用本模块；Tauri 端通过 `get_fkey_hint` 命令取同样的结果。

/// 是否需要按住 `fn` 才能触发功能键。
///
/// 结果在首次调用时检测并缓存：该偏好只在启动时读取一次，
/// 用户修改系统设置后重启应用即生效。
pub fn needs_fn_modifier() -> bool {
    use std::sync::OnceLock;
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(detect_needs_fn_modifier)
}

/// macOS 上探测 `com.apple.keyboard.fnState`；其它平台固定返回 false。
#[cfg(target_os = "macos")]
fn detect_needs_fn_modifier() -> bool {
    // 该键不存在时系统采用默认行为：F1–F12 作为媒体键。
    let out = std::process::Command::new("defaults")
        .args(["read", "-g", "com.apple.keyboard.fnState"])
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let v = String::from_utf8_lossy(&o.stdout);
            // fnState = 1 表示已启用标准功能键，无需按住 fn。
            !v.trim().eq_ignore_ascii_case("1")
        }
        // 读取失败（键不存在或命令不可用）按默认媒体键行为处理。
        _ => true,
    }
}

#[cfg(not(target_os = "macos"))]
fn detect_needs_fn_modifier() -> bool {
    false
}

/// 渲染指定功能键的显示名：`"F1"` 或 `"fn+F1"`。
pub fn fkey_label(key: &str) -> String {
    if needs_fn_modifier() {
        format!("fn+{key}")
    } else {
        key.to_string()
    }
}

/// 「开始自动演奏」提示，如 `fn+F1开始自动演奏`。
pub fn start_script_hint() -> String {
    format!("{}开始自动演奏", fkey_label("F1"))
}

/// 「停止演奏」提示，如 `按 fn+F2 可停止`。
pub fn stop_script_hint() -> String {
    format!("按 {} 可停止", fkey_label("F2"))
}
