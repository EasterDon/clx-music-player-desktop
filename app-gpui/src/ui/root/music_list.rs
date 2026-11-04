use crate::model::ClxState;
use crate::services::api::MusicItem;
use super::covers::{
    GRID_CARD_COVER_SZ, GRID_CARD_FIX_H, GRID_CARD_FIX_W,
};
use crate::ui::theme::*;
use gpui_kit::prelude::*;
use gpui_kit::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub(crate) fn music_list_panel(
    colors: &Palette,
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
                    colors,
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
                    colors,
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
                colors: &Palette,
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
            .bg(colors.bg_placeholder)
            .text_sm()
            .text_color(colors.text_secondary)
            .child(init.clone()),
        None => div().w_full().h(h).bg(colors.bg_placeholder),
    };
    div()
        .w_full()
        .h(h)
        .flex_shrink_0()
        .rounded(px(8.0))
        .overflow_hidden()
        .bg(colors.bg_placeholder)
        .child(inner)
}

fn music_item_cover_row_circle(
                colors: &Palette,
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
            .bg(colors.bg_placeholder)
            .text_xs()
            .text_color(colors.text_secondary)
            .child(init.clone()),
        None => div().w(side).h(side).bg(colors.bg_placeholder),
    };
    div()
        .w(side)
        .h(side)
        .flex_shrink_0()
        .rounded_full()
        .overflow_hidden()
        .bg(colors.bg_placeholder)
        .child(inner)
}

fn music_list_item(
    colors: &Palette,
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

    let element: AnyElement = if grid {
        let m = model.clone();
        let item_clone = item.clone();
        // 网格卡片：尺寸固定，交由外层 flex_wrap 排版。
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
            .when(selected, |d| d.bg(colors.bg_selected))
            .when(!selected, |d| d.bg(colors.bg_elevated))
            .id(row_id)
            .on_click(move |_, _, cx| {
                let _ = cx.update_entity(&m, |s, c| {
                    s.select_music(item_clone.clone());
                    c.notify();
                });
            })
            .child(music_item_cover_square_top(
                colors,
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
                        d.font_weight(FontWeight(600.0)).text_color(colors.accent)
                    })
                    .when(!selected, |d| d.text_color(colors.text_primary))
                    .child(item.name.clone()),
            )
            .child(
                div()
                    .w_full()
                    .min_w(px(0.0))
                    .text_center()
                    .text_xs()
                    .text_color(colors.text_secondary)
                    .truncate()
                    .child(item.author.clone()),
            )
            .into_any_element()
    } else {
        let m = model;
        let item_clone = item.clone();
        // 列表行：ListItem 内部用 h_flex() 并叠加自己的 padding/gap 默认样式，
        // 与这里需要的行布局冲突（实测会变成竖向堆叠），因此保持自绘容器。
        div()
            .id(row_id)
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
            .when(selected, |d| d.bg(colors.bg_selected))
            .when(!selected, |d| d.bg(colors.bg_elevated))
            .on_click(move |_, _, cx| {
                let _ = cx.update_entity(&m, |s, c| {
                    s.select_music(item_clone.clone());
                    c.notify();
                });
            })
            .child(music_item_cover_row_circle(
                colors,
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
                                d.font_weight(FontWeight(600.0)).text_color(colors.accent)
                            })
                            .when(!selected, |d| d.text_color(colors.text_primary))
                            .child(item.name.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors.text_secondary)
                            .child(item.author.clone()),
                    ),
            )
            .into_any_element()
    };

    element
}
