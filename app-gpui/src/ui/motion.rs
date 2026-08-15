//! UI 过渡动画（对齐 `app-tauri` 中 cubic-bezier(0.22, 1, 0.36, 1) 的时长与手感）。

use gpui::ease_out_quint;

/// 设置抽屉开关（约 0.32s，贴近 Ant Design Drawer）。
pub const DURATION_DRAWER_SECS: f32 = 0.32;
/// 窄屏列表 / 歌词页切换（`App.vue` 0.6s）。
pub const DURATION_MOBILE_PAGE_SECS: f32 = 0.6;
/// 窄宽布局切换（`MusicList` 0.48s）。
pub const DURATION_LAYOUT_SECS: f32 = 0.48;
/// 歌词行切换居中（`Lyrics/index.vue` `transition: all 0.5s`）。
pub const DURATION_LYRICS_SCROLL_SECS: f32 = 0.5;

pub const SETTINGS_DRAWER_W_PX: f32 = 320.0;
/// 移动端页面切换时的水平位移（替代 CSS 3D 翻页）。
pub const MOBILE_PAGE_SLIDE_PX: f32 = 56.0;

/// 与 Tauri `cubic-bezier(0.22, 1, 0.36, 1)` 接近的缓动（GPUI 内置 ease-out-quint）。
pub fn ease_standard(t: f32) -> f32 {
    ease_out_quint()(t.clamp(0.0, 1.0))
}

/// 将 `current` 向 `target` 推进一帧；返回 `(新值, 是否仍在动画中)`。
pub fn step_toward(current: f32, target: f32, dt_secs: f32, duration_secs: f32) -> (f32, bool) {
    if (current - target).abs() < 0.004 {
        return (target, false);
    }
    let rate = (dt_secs / duration_secs).clamp(0.0, 1.0);
    let next = current + (target - current) * ease_standard(rate);
    (next, true)
}
