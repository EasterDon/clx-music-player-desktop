//! 颜色令牌与主题注入。
//!
//! 组件（gpui-kit / gpui-component）与自绘部分共用同一套配色：应用启动时把原有的
//! Apple Music 风格色值注入 gpui-kit 全局主题（[`install`]），之后所有组件都按
//! 这套色值渲染；[`Palette`] 只是把同一批颜色分发给渲染闭包。
//!
//! 注意：`cx.theme().tokens.*` 是可能带渐变的 `ThemeToken`，不能直接当作颜色用；
//! 这里统一从 `cx.theme().colors`（受 `Deref` 影响，可直接访问字段）取 `Hsla`。

use gpui_kit::component::{ActiveTheme as _, Theme};
use gpui_kit::*;

/// 应用主背景（系统浅灰）。
pub const BG_APP: u32 = 0xf5f5f7;
/// 抬升面板：侧栏、播放器、歌词区。
pub const BG_ELEVATED: u32 = 0xffffff;
/// 控件/搜索框/未激活按钮底。
pub const BG_SURFACE: u32 = 0xe8e8ed;
/// 列表选中、卡片高亮。
pub const BG_SELECTED: u32 = 0xe5e5ea;
/// 分隔线。
pub const SEPARATOR: u32 = 0xd1d1d6;
/// 主文字。
pub const TEXT_PRIMARY: u32 = 0x1d1d1f;
/// 次要文字（艺术家、时间等）。
pub const TEXT_SECONDARY: u32 = 0x86868b;
/// 占位、辅助说明。
pub const TEXT_TERTIARY: u32 = 0xaeaeb2;
/// Apple Music 粉红强调色。
pub const ACCENT: u32 = 0xfa2d48;
/// 强调色上的文字。
pub const ACCENT_FG: u32 = 0xffffff;
/// 更新提示角标。
pub const BADGE: u32 = 0xfa2d48;

fn c(v: u32) -> Hsla {
    rgb(v).into()
}

/// 把应用的色板写入 gpui-kit 全局主题。
///
/// 必须在 `gpui_kit::init` 之后、打开窗口之前调用；`Theme::update` 会同时刷新
/// `colors`、`tokens` 与各窗口，组件因此立即按本配色渲染。深色模式下同样适用
/// （当前应用只提供浅色设计）。
pub fn install(cx: &mut App) {
    Theme::update(cx, |theme| {
        // `theme.list` 是 ListSettings 配置，颜色在 `theme.colors.list`。
        theme.colors.list = c(BG_ELEVATED);
        theme.background = c(BG_APP);
        theme.foreground = c(TEXT_PRIMARY);
        theme.border = c(SEPARATOR);
        // accent 在 gpui-component 中是「选中高亮底色」，对应的品牌色在这里。
        theme.accent = c(ACCENT);
        theme.accent_foreground = c(ACCENT_FG);
        theme.muted = c(BG_SURFACE);
        theme.muted_foreground = c(TEXT_SECONDARY);
        theme.secondary = c(BG_ELEVATED);
        theme.secondary_foreground = c(TEXT_PRIMARY);
        theme.secondary_hover = c(BG_SELECTED);
        theme.secondary_active = c(BG_SELECTED);
        theme.list_active = c(BG_SELECTED);
        theme.list_hover = c(BG_SELECTED);
        theme.popover = c(BG_ELEVATED);
        theme.popover_foreground = c(TEXT_PRIMARY);
        theme.group_box = c(BG_SURFACE);
        theme.group_box_foreground = c(TEXT_PRIMARY);
        theme.button = c(BG_SURFACE);
        theme.button_hover = c(BG_SELECTED);
        theme.button_active = c(BG_SELECTED);
        theme.button_foreground = c(TEXT_PRIMARY);
        theme.primary = c(ACCENT);
        theme.primary_hover = c(0xe0263f);
        theme.primary_active = c(0xc41f36);
        theme.primary_foreground = c(ACCENT_FG);
        theme.caret = c(ACCENT);
        theme.selection = c(0xfa2d48);
        theme.slider_bar = c(BG_SURFACE);
        theme.slider_thumb = c(ACCENT);
        theme.danger = c(BADGE);
        theme.input = c(BG_SURFACE);
        theme.overlay = hsla(0.0, 0.0, 0.0, 0.28);
    });
}

/// 一次性取出的颜色快照。
///
/// 渲染闭包会 move 走 `cx`（例如 `cx.read_entity(.., move |..| ..)`），
/// 在进入闭包前把用到的颜色复制到此结构体，闭包内直接取字段即可。
#[derive(Clone, Copy)]
pub struct Palette {
    /// 应用主背景。
    pub bg_app: Hsla,
    /// 抬升面板。
    pub bg_elevated: Hsla,
    /// 控件底。
    pub bg_surface: Hsla,
    /// 列表选中。
    pub bg_selected: Hsla,
    /// 封面占位。
    pub bg_placeholder: Hsla,
    /// 分隔线。
    pub separator: Hsla,
    /// 主文字。
    pub text_primary: Hsla,
    /// 次要文字。
    pub text_secondary: Hsla,
    /// 辅助文字。
    pub text_tertiary: Hsla,
    /// 强调色。
    pub accent: Hsla,
    /// 强调色上的文字。
    pub accent_fg: Hsla,
    /// 更新角标。
    pub badge: Hsla,
    /// 控制按钮底。
    pub ctrl_btn_bg: Hsla,
    /// 模态遮罩。
    pub overlay_scrim: Hsla,
}

impl Palette {
    pub fn from(cx: &App) -> Self {
        Self {
            bg_app: bg_app(cx),
            bg_elevated: bg_elevated(cx),
            bg_surface: bg_surface(cx),
            bg_selected: bg_selected(cx),
            bg_placeholder: bg_placeholder(cx),
            separator: separator(cx),
            text_primary: text_primary(cx),
            text_secondary: text_secondary(cx),
            text_tertiary: text_tertiary(cx),
            accent: accent(cx),
            accent_fg: accent_fg(cx),
            badge: badge(cx),
            ctrl_btn_bg: ctrl_btn_bg(cx),
            overlay_scrim: overlay_scrim(cx),
        }
    }
}

/// 应用主背景。
pub fn bg_app(cx: &App) -> Hsla {
    cx.theme().background
}

/// 抬升面板：侧栏、播放器、歌词区。
pub fn bg_elevated(cx: &App) -> Hsla {
    cx.theme().secondary
}

/// 控件/搜索框/未激活按钮底。
pub fn bg_surface(cx: &App) -> Hsla {
    cx.theme().muted
}

/// 列表选中、卡片高亮。
pub fn bg_selected(cx: &App) -> Hsla {
    cx.theme().list_active
}

/// 封面占位。
pub fn bg_placeholder(cx: &App) -> Hsla {
    cx.theme().muted
}


/// 分隔线。
pub fn separator(cx: &App) -> Hsla {
    cx.theme().border
}

/// 主文字。
pub fn text_primary(cx: &App) -> Hsla {
    cx.theme().foreground
}

/// 次要文字（艺术家、时间等）。
pub fn text_secondary(cx: &App) -> Hsla {
    cx.theme().muted_foreground
}

/// 占位、辅助说明。
pub fn text_tertiary(_cx: &App) -> Hsla {
    c(TEXT_TERTIARY)
}

/// 强调色（当前曲目、选中态、播放主按钮底）。
///
/// 取 `primary` 而非 `accent`：gpui-component 里 `accent` 表示「选中高亮底色」，
/// 品牌色对应的语义是 `primary`（已在 [`install`] 中注入为粉红）。
pub fn accent(cx: &App) -> Hsla {
    cx.theme().primary
}

/// 强调色上的文字。
pub fn accent_fg(cx: &App) -> Hsla {
    cx.theme().primary_foreground
}

/// 更新提示角标。
pub fn badge(cx: &App) -> Hsla {
    cx.theme().danger
}

/// 播放控制圆形按钮底。
pub fn ctrl_btn_bg(cx: &App) -> Hsla {
    cx.theme().button
}

/// 模态遮罩。
pub fn overlay_scrim(cx: &App) -> Hsla {
    cx.theme().overlay
}
