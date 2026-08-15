//! Apple Music 风格视觉令牌（浅色主题）。

/// 应用主背景（系统浅灰）。
pub const BG_APP: u32 = 0xf5f5f7;
/// 抬升面板：侧栏、播放器、歌词区。
pub const BG_ELEVATED: u32 = 0xffffff;
/// 控件/搜索框/未激活按钮底。
pub const BG_SURFACE: u32 = 0xe8e8ed;
/// 列表选中、卡片高亮。
pub const BG_SELECTED: u32 = 0xe5e5ea;
/// 封面占位。
pub const BG_PLACEHOLDER: u32 = 0xe8e8ed;

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
/// 文本选区、半透明强调。
pub const ACCENT_SOFT: u32 = 0xfa2d4828;
/// 更新提示。
pub const BADGE: u32 = 0xfa2d48;

/// 进度条轨道 / 未填充。
pub const PROGRESS_TRACK: u32 = 0xd1d1d6;
/// 进度条已缓存（可 seek），未播放部分。
pub const PROGRESS_BUFFERED: u32 = 0xfcc2cb;
/// 进度条已播放。
pub const PROGRESS_FILL: u32 = 0xfa2d48;

/// 播放控制圆形按钮底。
pub const CTRL_BTN_BG: u32 = 0xe8e8ed;

/// 弹窗面板底色。
pub const BG_MODAL: u32 = 0xffffff;

/// 模态遮罩不透明度（与 `hsla(0., 0., 0., OVERLAY_SCRIM_A)` 配合）。
pub const OVERLAY_SCRIM_A: f32 = 0.28;
