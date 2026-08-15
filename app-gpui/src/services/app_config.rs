use std::path::PathBuf;

const CONFIG_FILE: &str = "config.ini";
const KEY_LOAD_BY_ADMINISTRATOR: &str = "loadByAdministrator";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppConfig {
    /// 下次启动时是否请求管理员权限（Windows 自动演奏模拟输入常需要）。
    pub launch_as_administrator: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            launch_as_administrator: false,
        }
    }
}

pub fn config_path() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    base.join("clx-gpui")
}

fn config_file() -> PathBuf {
    config_path().join(CONFIG_FILE)
}

fn strip_bom(s: &str) -> &str {
    s.strip_prefix('\u{feff}').unwrap_or(s)
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" => Some(true),
        "false" | "0" | "no" => Some(false),
        _ => None,
    }
}

fn parse_ini(content: &str) -> AppConfig {
    let mut launch_as_administrator = false;
    for line in strip_bom(content).lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if strip_bom(key.trim()) == KEY_LOAD_BY_ADMINISTRATOR {
            if let Some(v) = parse_bool(value) {
                launch_as_administrator = v;
            }
        }
    }
    AppConfig {
        launch_as_administrator,
    }
}

fn format_ini(config: &AppConfig) -> String {
    format!(
        "; 一梦音乐播放器本地配置\r\n{KEY_LOAD_BY_ADMINISTRATOR}={}\r\n",
        config.launch_as_administrator
    )
}

/// 解析服务端地址：构建时由 `build.rs` 从项目根目录 `.env` 的 `CLX_BASE_URL`
/// 编译进二进制（`CLX_EMBEDDED_BASE_URL`），运行时不再读取任何配置文件。
/// 未配置时返回空串（应用会提示「未配置服务器地址」）。
pub fn resolve_base_url() -> String {
    option_env!("CLX_EMBEDDED_BASE_URL")
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

/// 读取配置（不创建文件）。
pub fn load() -> AppConfig {
    let path = config_file();
    let Ok(data) = std::fs::read_to_string(&path) else {
        return AppConfig::default();
    };
    parse_ini(&data)
}

/// 若 `%LOCALAPPDATA%\clx-gpui\config.ini` 不存在，则创建默认文件。
pub fn ensure_exists() -> Result<(), String> {
    if config_file().is_file() {
        return Ok(());
    }
    save(&AppConfig::default())
}

pub fn save(config: &AppConfig) -> Result<(), String> {
    let dir = config_path();
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建配置目录失败: {e}"))?;
    std::fs::write(config_file(), format_ini(config)).map_err(|e| format!("写入配置失败: {e}"))?;
    Ok(())
}
