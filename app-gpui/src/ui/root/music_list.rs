use crate::model::ClxState;
use crate::services::api::MusicItem;
use super::covers::{
    GRID_CARD_COVER_SZ, GRID_CARD_FIX_H, GRID_CARD_FIX_W,
};
use crate::ui::theme::*;
use gpui::prelude::*;
use gpui::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub(crate) fn music_list_panel(
    items: Vec<MusicItem>,
    current_id: Option<i64>,
    model: Entity<ClxState>,
    grid: bool,
    music_covers: Arc<HashMap<i64, Arc<Image>>>,
    music_cover_failed: HashSet<i64>,
) -> impl IntoElement {
    if grid {
        let covers_grid = Arc::clone(&music_covers);
        let failed_grid = music_cover_failed.clone();
        let mut grid_el = div()
            .w_full()
            .min_w(px(0.0))
            .flex()
            .flex_row()
            .flex_wrap()
            .justify_center()
            .gap(px(8.0))
            .children(items.into_iter().map(move |item| {
                let sel = current_id == Some(item.id);
                let loaded = covers_grid.get(&item.id).cloned();
                let cover_failed = failed_grid.contains(&item.id);
                music_list_item(
                    item,
                    sel,
                    model.clone(),
                    true,
                    loaded,
                    cover_failed,
                )
            }));
        grid_el.style().align_items = Some(AlignItems::FlexStart);
        grid_el
    } else {
        div()
            .w_full()
            .min_w(px(0.0))
            .flex()
            .flex_col()
            .gap(px(6.0))
            .children(items.into_iter().map(move |item| {
                let sel = current_id == Some(item.id);
                let loaded = music_covers.get(&item.id).cloned();
                let cover_failed = music_cover_failed.contains(&item.id);
                music_list_item(
                    item,
                    sel,
                    model.clone(),
                    false,
                    loaded,
                    cover_failed,
                )
            }))
    }
}

fn music_item_cover_square_top(
    loaded: Option<Arc<Image>>,
    cover_failed: bool,
    initial: SharedString,
) -> impl IntoElement {
    let init = initial.clone();
    let h = px(GRID_CARD_COVER_SZ);
    let inner = match loaded {
        Some(arc_img) => div()
            .w_full()
            .h(h)
            .overflow_hidden()
            .flex()
            .items_center()
            .justify_center()
            .child(
                img(arc_img)
                    .flex_none()
                    .object_fit(ObjectFit::Cover)
                    .w(h)
                    .h(h),
            ),
        None if cover_failed => div()
            .w_full()
            .h(h)
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(BG_PLACEHOLDER))
            .text_sm()
            .text_color(rgb(TEXT_SECONDARY))
            .child(init.clone()),
        None => div().w_full().h(h).bg(rgb(BG_PLACEHOLDER)),
    };
    div()
        .w_full()
        .h(h)
        .flex_shrink_0()
        .rounded(px(8.0))
        .overflow_hidden()
        .bg(rgb(BG_PLACEHOLDER))
        .child(inner)
}

fn music_item_cover_row_circle(
    loaded: Option<Arc<Image>>,
    cover_failed: bool,
    initial: SharedString,
) -> impl IntoElement {
    let init = initial.clone();
    let side = px(60.0);
    let inner = match loaded {
        Some(arc_img) => div()
            .w(side)
            .h(side)
            .overflow_hidden()
            .child(
                img(arc_img)
                    .flex_none()
                    .object_fit(ObjectFit::Cover)
                    .w(side)
                    .h(side),
            ),
        None if cover_failed => div()
            .w(side)
            .h(side)
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(BG_PLACEHOLDER))
            .text_xs()
            .text_color(rgb(TEXT_SECONDARY))
            .child(init.clone()),
        None => div().w(side).h(side).bg(rgb(BG_PLACEHOLDER)),
    };
    div()
        .w(side)
        .h(side)
        .flex_shrink_0()
        .rounded_full()
        .overflow_hidden()
        .bg(rgb(BG_PLACEHOLDER))
        .child(inner)
}

fn music_list_item(
    item: MusicItem,
    selected: bool,
    model: Entity<ClxState>,
    grid: bool,
    cover_loaded: Option<Arc<Image>>,
    cover_failed: bool,
) -> impl IntoElement {
    let row_id = ElementId::NamedInteger("music-item".into(), item.id as u64);
    let initial = SharedString::from(
        item.name
            .chars()
            .next()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "♪".into()),
    );

    if grid {
        let m = model.clone();
        let item_clone = item.clone();
        div()
            .w(px(GRID_CARD_FIX_W))
            .h(px(GRID_CARD_FIX_H))
            .min_w(px(GRID_CARD_FIX_W))
            .max_w(px(GRID_CARD_FIX_W))
            .min_h(px(GRID_CARD_FIX_H))
            .max_h(px(GRID_CARD_FIX_H))
            .flex_none()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(6.0))
            .p(px(8.0))
            .rounded(px(10.0))
            .overflow_hidden()
            .cursor_pointer()
            .when(selected, |d| d.bg(rgb(BG_SELECTED)))
            .when(!selected, |d| d.bg(rgb(BG_ELEVATED)))
            .id(row_id)
            .on_click(move |_, _, cx| {
                let _ = cx.update_entity(&m, |s, c| {
                    s.select_music(item_clone.clone());
                    c.notify();
                });
            })
            .child(music_item_cover_square_top(
                cover_loaded,
                cover_failed,
                initial.clone(),
            ))
            .child(
                div()
                    .w_full()
                    .min_w(px(0.0))
                    .text_center()
                    .text_xs()
                    .truncate()
                    .when(selected, |d| {
                        d.font_weight(FontWeight(600.0)).text_color(rgb(ACCENT))
                    })
                    .when(!selected, |d| d.text_color(rgb(TEXT_PRIMARY)))
                    .child(item.name.clone()),
            )
            .child(
                div()
                    .w_full()
                    .min_w(px(0.0))
                    .text_center()
                    .text_xs()
                    .text_color(rgb(TEXT_SECONDARY))
                    .truncate()
                    .child(item.author.clone()),
            )
    } else {
        let m = model;
        let item_clone = item.clone();
        div()
            .w_full()
            .min_w(px(0.0))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(20.0))
            .px(px(8.0))
            .py(px(6.0))
            .rounded(px(8.0))
            .cursor_pointer()
            .when(selected, |d| d.bg(rgb(BG_SELECTED)))
            .when(!selected, |d| d.bg(rgb(BG_ELEVATED)))
            .id(row_id)
            .on_click(move |_, _, cx| {
                let _ = cx.update_entity(&m, |s, c| {
                    s.select_music(item_clone.clone());
                    c.notify();
                });
            })
            .child(music_item_cover_row_circle(
                cover_loaded,
                cover_failed,
                initial,
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w(px(0.0))
                    .child(
                        div()
                            .text_sm()
                            .when(selected, |d| {
                                d.font_weight(FontWeight(600.0)).text_color(rgb(ACCENT))
                            })
                            .when(!selected, |d| d.text_color(rgb(TEXT_PRIMARY)))
                            .child(item.name.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(TEXT_SECONDARY))
                            .child(item.author.clone()),
                    ),
            )
    }
}
