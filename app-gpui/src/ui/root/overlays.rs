use crate::ui::theme::*;
use gpui::prelude::*;
use gpui::*;

pub(crate) fn script_modal(_model: Entity<crate::model::ClxState>, title: String, author: String) -> impl IntoElement {
    div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(hsla(0., 0., 0., OVERLAY_SCRIM_A))
        .child(
            div()
                .w(px(400.0))
                .bg(rgb(BG_MODAL))
                .border_1()
                .border_color(rgb(SEPARATOR))
                .rounded(px(14.0))
                .p(px(24.0))
                .flex()
                .flex_col()
                .gap(px(12.0))
                .items_center()
                .text_color(rgb(TEXT_PRIMARY))
                .child(div().font_weight(FontWeight(700.0)).child("自动演奏中…"))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(TEXT_SECONDARY))
                        .child(format!("当前演奏歌曲：{title}")),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(TEXT_SECONDARY))
                        .child(format!("作者：{author}")),
                )
                .child(
                    div()
                        .mt(px(8.0))
                        .text_xs()
                        .text_color(rgb(TEXT_TERTIARY))
                        .child("按 F2 可停止"),
                ),
        )
}

pub(crate) fn toast_overlay(
    msg: String,
    dismiss: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .absolute()
        .bottom(px(24.0))
        .left_0()
        .right_0()
        .flex()
        .flex_col()
        .items_center()
        .child(
            div()
                .max_w(px(520.0))
                .bg(rgb(BG_SURFACE))
                .text_color(rgb(TEXT_PRIMARY))
                .rounded(px(20.0))
                .px(px(16.0))
                .py(px(10.0))
                .text_sm()
                .cursor_pointer()
                .child(msg)
                .id("toast-dismiss")
                .on_click(dismiss),
        )
}
