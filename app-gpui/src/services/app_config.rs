pub use clx_core::app_config::*;

/// 解析服务端地址：构建时由 `build.rs` 从项目根目录 `.env` 的 `CLX_BASE_URL`
/// 编译进二进制（`CLX_EMBEDDED_BASE_URL`），运行时不再读取任何配置文件。
pub fn resolve_base_url() -> String {
    option_env!("CLX_EMBEDDED_BASE_URL")
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}
