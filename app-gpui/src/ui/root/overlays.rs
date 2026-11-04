use gpui_kit::component::WindowExt as _;
use gpui_kit::component::notification::Notification;
use gpui_kit::component::dialog::{DialogDescription, DialogTitle};
use gpui_kit::prelude::*;
use gpui_kit::*;

/// 脚本演奏提示框。
///
/// 使用组件库的 `Dialog`，由窗口层统一绘制遮罩、圆角与动画，
/// 关闭时调用 `window.close_dialog`。
pub(crate) fn open_script_dialog(
    window: &mut Window,
    cx: &mut App,
    title: String,
    author: String,
) {
    window.open_dialog(cx, move |dialog, _, _| {
        dialog
            .title(DialogTitle::new().child("自动演奏中…"))
            .child(
                DialogDescription::new()
                    .child(format!("当前演奏歌曲：{title}")),
            )
            .child(DialogDescription::new().child(format!("作者：{author}")))
            .child(DialogDescription::new().child("按 F2 可停止"))
    });
}

/// 关闭脚本演奏提示框。
pub(crate) fn close_script_dialog(window: &mut Window, cx: &mut App) {
    if window.has_active_dialog(cx) {
        window.close_dialog(cx);
    }
}


/// 应用级 toast 的唯一标识类型。
struct ClxToast;

/// 用组件库的通知层展示一条 toast。
///
/// 沿用 `ClxState` 的 toast 队列与自动消失计时（数据来源不变），
/// 只把展示交给 `NotificationList`；同一 `ClxToast` 类型下最多保留一条。
pub(crate) fn push_toast_notification(window: &mut Window, cx: &mut App, msg: String) {
    window.push_notification(
        Notification::info(msg).id::<ClxToast>(),
        cx,
    );
}

/// 没有新 toast 时清掉仍在显示的旧通知。
pub(crate) fn clear_toast_notifications(window: &mut Window, cx: &mut App) {
    window.remove_notification::<ClxToast>(cx);
}
