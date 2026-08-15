use crate::services::mp3_probe;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// 临时缓存目录：`{temp}/clx-gpui/music/{id}.mp3`
pub fn cache_dir() -> PathBuf {
    std::env::temp_dir().join("clx-gpui").join("music")
}

pub fn mp3_path(music_id: i64) -> PathBuf {
    cache_dir().join(format!("{music_id}.mp3"))
}

pub fn ensure_cache_dir() -> Result<(), String> {
    std::fs::create_dir_all(cache_dir()).map_err(|e| format!("创建缓存目录失败: {e}"))
}

pub fn is_complete_file(path: &Path, expected_bytes: u64) -> bool {
    expected_bytes > 0
        && std::fs::metadata(path)
            .map(|m| m.len() == expected_bytes)
            .unwrap_or(false)
}

/// 从已缓存的 MP3 文件头部探测时长。
pub fn probe_duration_from_file(path: &Path) -> Option<f64> {
    const PROBE_MAX: usize = 512 * 1024;
    let mut file = File::open(path).ok()?;
    let total = file.metadata().ok().map(|m| m.len()).filter(|&n| n > 0);
    let mut buf = vec![0u8; PROBE_MAX];
    let n = file.read(&mut buf).ok()?;
    buf.truncate(n);
    mp3_probe::probe_duration_sec(&buf, total)
}
