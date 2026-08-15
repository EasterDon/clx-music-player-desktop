use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::str::FromStr;
use ureq::http::HeaderMap;

/// 整曲下载体积上限（`read_to_vec` 默认 10MB，音乐文件常更大）。
const MAX_MUSIC_DOWNLOAD_BYTES: u64 = 512 * 1024 * 1024;

fn header_first_str<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

fn get_allow_error_status(url: &str) -> Result<ureq::http::Response<ureq::Body>, String> {
    ureq::get(url)
        .config()
        .http_status_as_error(false)
        .build()
        .call()
        .map_err(|e| format!("请求失败: {e}"))
}

fn head_allow_error_status(url: &str) -> Result<ureq::http::Response<ureq::Body>, String> {
    ureq::head(url)
        .config()
        .http_status_as_error(false)
        .build()
        .call()
        .map_err(|e| format!("HEAD 失败: {e}"))
}

#[derive(Debug, Deserialize)]
pub struct ApiEnvelope<T> {
    pub data: T,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MusicItem {
    pub id: i64,
    pub author: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppInfo {
    #[allow(dead_code)]
    pub id: i64,
    pub announcement: String,
    pub app_version: String,
    pub app_download_link: String,
    pub app_version_description: String,
}

fn fetch_json<T: DeserializeOwned>(url: &str) -> Result<T, String> {
    let mut resp = get_allow_error_status(url)?;
    let status = resp.status();
    let text = resp
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("读取响应失败: {e}"))?;
    if status.is_client_error() || status.is_server_error() {
        let slice: String = text.chars().take(160).collect();
        return Err(format!("HTTP {}: {slice}", status.as_u16()));
    }
    serde_json::from_str(&text).map_err(|e| format!("JSON 解析失败: {e}"))
}

pub fn get_music_list(base: &str) -> Result<Vec<MusicItem>, String> {
    let url = format!("{base}/v3/music");
    let env: ApiEnvelope<Vec<MusicItem>> = fetch_json(&url)?;
    Ok(env.data)
}

/// 与 `app-tauri/src/config/index.ts` 中 `music_resource_url` + `music.jpg` 一致。
pub fn music_cover_jpeg_url(base: &str, music_id: i64) -> String {
    let b = base.trim_end_matches('/');
    format!("{b}/music/{music_id}/music.jpg")
}

pub fn get_music_lyrics(base: &str, music_id: i64) -> Result<String, String> {
    let url = format!("{base}/v3/music/{music_id}/lyrics");
    let env: ApiEnvelope<String> = fetch_json(&url)?;
    Ok(env.data)
}

pub fn get_app_info(base: &str) -> Result<AppInfo, String> {
    let url = format!("{base}/v3/app");
    let env: ApiEnvelope<AppInfo> = fetch_json(&url)?;
    Ok(env.data)
}

pub fn get_music_notation(base: &str, music_id: i64) -> Result<Vec<serde_json::Value>, String> {
    let url = format!("{base}/v3/music/{music_id}/notation");
    let env: ApiEnvelope<Vec<serde_json::Value>> = fetch_json(&url)?;
    Ok(env.data)
}

pub fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
    let mut resp = get_allow_error_status(url)?;
    let status = resp.status();
    if status.is_client_error() || status.is_server_error() {
        return Err(format!("HTTP {}", status.as_u16()));
    }
    resp.body_mut()
        .with_config()
        .limit(MAX_MUSIC_DOWNLOAD_BYTES)
        .read_to_vec()
        .map_err(|e| format!("读取二进制失败: {e}"))
}

/// `Content-Length` from HEAD（部分 CDN 不支持 HEAD，可忽略错误）。
pub fn head_content_length(url: &str) -> Result<Option<u64>, String> {
    let resp = head_allow_error_status(url)?;
    let status = resp.status();
    if status.is_client_error() || status.is_server_error() {
        return Err(format!("HEAD HTTP {}", status.as_u16()));
    }
    Ok(header_first_str(resp.headers(), "content-length").and_then(|s| u64::from_str(s.trim()).ok()))
}

fn parse_content_range_total(value: &str) -> Option<u64> {
    let rest = value.trim().strip_prefix("bytes ")?;
    let (_, total) = rest.split_once('/')?;
    if total == "*" {
        return None;
    }
    u64::from_str(total.trim()).ok()
}

/// 使用 `Range` 拉取文件前缀用于时长探测；若服务端忽略 Range 返回 200，只读前 `max_bytes` 字节。
/// 返回 `(数据, 资源总字节数)`，总长度来自 `Content-Range` / `Content-Length`。
pub fn fetch_bytes_range_prefix(url: &str, max_bytes: usize) -> Result<(Vec<u8>, Option<u64>), String> {
    if max_bytes == 0 {
        return Ok((Vec::new(), None));
    }
    let last = (max_bytes as u64).saturating_sub(1);
    let mut resp = ureq::get(url)
        .header("Range", format!("bytes=0-{last}"))
        .config()
        .http_status_as_error(false)
        .build()
        .call()
        .map_err(|e| format!("请求失败: {e}"))?;
    let status = resp.status();
    if status.is_client_error() || status.is_server_error() {
        return Err(format!("HTTP {}", status.as_u16()));
    }
    let mut total = header_first_str(resp.headers(), "content-range").and_then(parse_content_range_total);
    if total.is_none() {
        total = header_first_str(resp.headers(), "content-length")
            .and_then(|s| u64::from_str(s.trim()).ok());
    }
    let mut buf = Vec::new();
    resp.body_mut()
        .as_reader()
        .take(max_bytes as u64)
        .read_to_end(&mut buf)
        .map_err(|e| format!("读取二进制失败: {e}"))?;
    Ok((buf, total))
}

/// 前缀下载结果：用于边下边播，先落盘前缀再开始解码。
pub struct Mp3PrefixDownload {
    pub bytes_written: u64,
    pub total_bytes: Option<u64>,
    pub duration_secs: Option<f64>,
}

/// 将 MP3 前缀写入 `dest`（截断重建）。返回写入字节数与探测到的时长。
pub fn write_mp3_prefix_to_file(
    url: &str,
    dest: &Path,
    max_prefix_bytes: usize,
) -> Result<Mp3PrefixDownload, String> {
    if max_prefix_bytes == 0 {
        return Err("前缀大小为 0".into());
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    let (bytes, total) = fetch_bytes_range_prefix(url, max_prefix_bytes)?;
    let duration_secs =
        crate::services::mp3_probe::probe_duration_sec(&bytes, total);
    let mut file = File::create(dest).map_err(|e| format!("创建缓存文件失败: {e}"))?;
    file.write_all(&bytes)
        .map_err(|e| format!("写入缓存失败: {e}"))?;
    file.flush().map_err(|e| format!("刷新缓存失败: {e}"))?;
    Ok(Mp3PrefixDownload {
        bytes_written: bytes.len() as u64,
        total_bytes: total,
        duration_secs,
    })
}

/// 从 `start_byte` 起追加剩余字节到已存在文件（需服务端支持 Range）。
/// `on_progress` 收到当前文件总字节数（`start_byte` + 本次已写入）。
pub fn append_mp3_remainder(
    url: &str,
    dest: &Path,
    start_byte: u64,
    total_bytes: Option<u64>,
    mut on_progress: Option<&mut dyn FnMut(u64)>,
) -> Result<(), String> {
    if let Some(total) = total_bytes {
        if start_byte >= total {
            return Ok(());
        }
    }
    let mut resp = ureq::get(url)
        .header("Range", format!("bytes={start_byte}-"))
        .config()
        .http_status_as_error(false)
        .build()
        .call()
        .map_err(|e| format!("请求失败: {e}"))?;
    let status = resp.status();
    if status.is_client_error() || status.is_server_error() {
        return Err(format!("HTTP {}", status.as_u16()));
    }
    let mut file = OpenOptions::new()
        .append(true)
        .open(dest)
        .map_err(|e| format!("打开缓存失败: {e}"))?;
    let limit = total_bytes.map(|t| t.saturating_sub(start_byte));
    let mut reader = resp.body_mut().as_reader();
    let mut buf = [0u8; 64 * 1024];
    let mut written = 0u64;
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| format!("读取网络数据失败: {e}"))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])
            .map_err(|e| format!("追加缓存失败: {e}"))?;
        written += n as u64;
        if let Some(ref mut f) = on_progress {
            f(start_byte.saturating_add(written));
        }
        if let Some(max) = limit {
            if written >= max {
                break;
            }
        }
        if written > MAX_MUSIC_DOWNLOAD_BYTES {
            return Err("下载体积超过上限".into());
        }
    }
    file.flush().map_err(|e| format!("刷新缓存失败: {e}"))?;
    Ok(())
}
