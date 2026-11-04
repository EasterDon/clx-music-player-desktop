//! 搜索框占位提示中的快捷键文案。
//!
//! 平台判断集中在 `clx_core::fkey_hint`，GPUI 与 Tauri 两端共用同一实现。
//! 目前只有搜索框需要带上 `fn` 提示；设置面板与演奏对话框使用固定文案。

/// 组合键提示中 F1 的写法，如 `"fn+F1"` 或 `"F1"`。
pub fn f1_label() -> String {
    clx_core::fkey_hint::fkey_label("F1")
}

/// 组合键提示中 F2 的写法。
pub fn f2_label() -> String {
    clx_core::fkey_hint::fkey_label("F2")
}
