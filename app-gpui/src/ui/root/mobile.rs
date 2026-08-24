use crate::ui::motion::MOBILE_PAGE_SLIDE_PX;
use crate::ui::theme::*;
use gpui::prelude::*;
use gpui::*;

pub(crate) const MOBILE_FLOAT_BTN: f32 = 32.0;
pub(crate) const MOBILE_LYRICS_TOGGLE_BOTTOM: f32 = 14.0;

fn mobile_floating_icon_btn(
    id: &'static str,
    icon: &'static str,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    div()
        .w(px(MOBILE_FLOAT_BTN))
        .h(px(MOBILE_FLOAT_BTN))
        .min_w(px(MOBILE_FLOAT_BTN))
        .min_h(px(MOBILE_FLOAT_BTN))
        .max_w(px(MOBILE_FLOAT_BTN))
        .max_h(px(MOBILE_FLOAT_BTN))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .border_1()
        .border_color(rgb(SEPARATOR))
        .bg(rgb(BG_ELEVATED))
        .text_sm()
        .text_color(rgb(TEXT_SECONDARY))
        .cursor_pointer()
        .occlude()
        .child(icon)
        .id(id)
        .on_click(on_click)
}

pub(crate) fn mobile_lyrics_toggle_overlay(
    on_open_lyrics: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    mobile_floating_icon_btn("mobile-open-lyrics", "♬", on_open_lyrics)
        .absolute()
        .bottom(px(MOBILE_LYRICS_TOGGLE_BOTTOM))
        .right(px(10.0))
}

pub(crate) fn mobile_list_back_overlay(
    on_back: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    mobile_floating_icon_btn("mobile-back-list", "‹", on_back)
        .absolute()
        .top(px(10.0))
        .left(px(10.0))
}

pub(crate) fn mobile_fullscreen_pane(content: impl IntoElement) -> impl IntoElement {
    div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .child(content)
}

pub(crate) fn mobile_page_stack(
    list_pane: impl IntoElement,
    lyrics_pane: impl IntoElement,
    mobile_t: f32,
) -> impl IntoElement {
    let list_opacity = 1.0 - mobile_t;
    let lyrics_opacity = mobile_t;
    let list_shift = MOBILE_PAGE_SLIDE_PX * mobile_t;
    let lyrics_shift = MOBILE_PAGE_SLIDE_PX * (1.0 - mobile_t);

    let lyrics_layer = div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .opacity(lyrics_opacity)
        .left(px(-lyrics_shift))
        .child(lyrics_pane);

    let list_layer = div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .opacity(list_opacity)
        .left(px(list_shift))
        .child(list_pane);

    let mut stack = div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .overflow_hidden();

    if mobile_t < 0.5 {
        stack = stack.child(lyrics_layer).child(list_layer.occlude());
    } else {
        stack = stack.child(list_layer).child(lyrics_layer.occlude());
    }
    stack
}
