use crate::services::api::MusicItem;
use gpui::Pixels;
use gpui::px;

pub(crate) const GRID_CARD_FIX_W: f32 = 138.0;
pub(crate) const GRID_CARD_FIX_H: f32 = 196.0;
pub(crate) const GRID_CARD_COVER_SZ: f32 = 112.0;
pub(crate) const GRID_GAP: f32 = 8.0;

const LIST_COVER_STRIDE_PX: f32 = 90.0;
const GRID_ROW_STRIDE_PX: f32 = GRID_CARD_FIX_H + GRID_GAP;
const COVER_PREFETCH_OVERSCAN_PX: f32 = 160.0;

fn grid_columns_for_width(w: Pixels) -> usize {
    let w = w.max(px(0.0)).to_f64() as f32;
    let slot = GRID_CARD_FIX_W + GRID_GAP;
    if w < slot {
        return 1;
    }
    (((w + GRID_GAP) / slot).floor() as i32).max(1) as usize
}

pub(crate) fn cover_ids_for_visible(
    items: &[MusicItem],
    grid: bool,
    grid_content_width: Pixels,
    scroll_offset_y: f32,
    viewport_h: f32,
) -> Vec<i64> {
    if items.is_empty() {
        return Vec::new();
    }
    let n = items.len();
    let scroll_top = (-scroll_offset_y).max(0.0);
    let top = scroll_top - COVER_PREFETCH_OVERSCAN_PX;
    let bot = scroll_top + viewport_h + COVER_PREFETCH_OVERSCAN_PX;

    if !grid {
        let i0 = ((top / LIST_COVER_STRIDE_PX).floor() as isize).max(0) as usize;
        let i1 = ((bot / LIST_COVER_STRIDE_PX).ceil() as isize).max(0) as usize;
        let i1 = i1.min(n - 1);
        if i0 > i1 {
            return Vec::new();
        }
        return items[i0..=i1].iter().map(|m| m.id).collect();
    }

    let per_row = grid_columns_for_width(grid_content_width).max(1);
    let r0 = ((top / GRID_ROW_STRIDE_PX).floor() as isize).max(0) as usize;
    let r1 = ((bot / GRID_ROW_STRIDE_PX).ceil() as isize).max(0) as usize;
    let i0 = (r0 * per_row).min(n.saturating_sub(1));
    let i1_ex = ((r1 + 1) * per_row).min(n);
    if i0 >= i1_ex {
        return Vec::new();
    }
    items[i0..i1_ex].iter().map(|m| m.id).collect()
}
