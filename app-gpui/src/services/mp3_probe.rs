//! 用 MP3 文件前缀（配合 HTTP `Content-Length` / `Content-Range`）尽快估算总时长。
//! 优先使用 `mp3-duration`（遇 Xing/Info 可立即返回）；否则在已知文件总大小时用 CBR 公式估算。

use std::io::Cursor;
use std::time::Duration;

fn duration_to_f64(d: Duration) -> f64 {
    d.as_secs_f64() + d.subsec_nanos() as f64 * 1e-9
}

fn id3v2_skip_len(buf: &[u8]) -> Option<usize> {
    if buf.len() < 10 || buf[0] != b'I' || buf[1] != b'D' || buf[2] != b'3' {
        return None;
    }
    let flags = buf[5];
    let footer = if flags & 0x10 != 0 { 10 } else { 0 };
    let tag_body = ((buf[9] as usize) | ((buf[8] as usize) << 7) | ((buf[7] as usize) << 14)
        | ((buf[6] as usize) << 21))
        + 10
        + footer;
    Some(tag_body)
}

/// MPEG1 Layer III bit rates (kbps), index = `bitrate_index` from header.
const L3_BITRATE_MPEG1_KBPS: [u32; 16] =
    [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0];
/// MPEG2 / MPEG2.5 Layer III (kbps).
const L3_BITRATE_MPEG23_KBPS: [u32; 16] =
    [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0];

const SAMPLE_RATE: [[u32; 3]; 3] = [
    [44100, 48000, 32000],
    [22050, 24000, 16000],
    [11025, 12000, 8000],
];

fn layer3_bitrate_kbps(version_idx: usize, br_index: u8) -> Option<u32> {
    let t = match version_idx {
        0 => L3_BITRATE_MPEG1_KBPS,
        1 | 2 => L3_BITRATE_MPEG23_KBPS,
        _ => return None,
    };
    let v = *t.get(br_index as usize)?;
    if v == 0 {
        None
    } else {
        Some(v)
    }
}

fn sample_rate_hz(version_idx: usize, sr_index: u8) -> Option<u32> {
    if sr_index >= 3 {
        return None;
    }
    SAMPLE_RATE.get(version_idx)?.get(sr_index as usize).copied()
}

/// 首帧 Layer III 起始偏移与比特率（bps）。仅用于 CBR 估算。
fn first_l3_frame_offset_and_bitrate_bps(buf: &[u8]) -> Option<(usize, u32)> {
    let mut i = id3v2_skip_len(buf).unwrap_or(0);
    i = i.min(buf.len());

    while i + 4 <= buf.len() {
        let h = u32::from_be_bytes([buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]);
        if h >> 21 != 0x7FF {
            i += 1;
            continue;
        }
        let version_raw = (h >> 19) & 0b11;
        if version_raw == 1 {
            i += 1;
            continue;
        }
        let v_idx = match version_raw {
            3 => 0usize,
            2 => 1,
            0 => 2,
            _ => {
                i += 1;
                continue;
            }
        };
        let layer_raw = (h >> 17) & 0b11;
        if layer_raw != 1 {
            // 只处理 Layer III
            i += 1;
            continue;
        }
        let br_index = ((h >> 12) & 0xF) as u8;
        if br_index == 0 || br_index >= 15 {
            i += 1;
            continue;
        }
        let sr_index = ((h >> 10) & 0b11) as u8;
        if sample_rate_hz(v_idx, sr_index).is_none() {
            i += 1;
            continue;
        }
        let kbps = layer3_bitrate_kbps(v_idx, br_index)?;
        let bps = kbps.saturating_mul(1000);
        return Some((i, bps));
    }
    None
}

fn cbr_duration_from_total_size(prefix: &[u8], total_bytes: u64) -> Option<f64> {
    let (first_off, bitrate_bps) = first_l3_frame_offset_and_bitrate_bps(prefix)?;
    let audio_bytes = total_bytes.checked_sub(first_off as u64)?;
    let secs = (audio_bytes as f64) * 8.0 / (bitrate_bps as f64);
    if secs.is_finite() && secs > 0.0 {
        Some(secs)
    } else {
        None
    }
}

/// `prefix`：文件开头一段字节；`total_file_bytes`：整文件长度（来自 `Content-Range` / `Content-Length` / HEAD）。
pub fn probe_duration_sec(prefix: &[u8], total_file_bytes: Option<u64>) -> Option<f64> {
    if prefix.is_empty() {
        return None;
    }

    let mut cur = Cursor::new(prefix);
    match mp3_duration::from_read(&mut cur) {
        Ok(d) => {
            let consumed = cur.position() as usize;
            let secs = duration_to_f64(d);
            // Xing/Info 等路径会提前返回，不会读完整个 prefix
            if consumed < prefix.len() {
                return Some(secs);
            }
            if let Some(total) = total_file_bytes {
                if total > 0 && total == prefix.len() as u64 {
                    return Some(secs);
                }
                return cbr_duration_from_total_size(prefix, total);
            }
            None
        }
        Err(_) => total_file_bytes.and_then(|t| cbr_duration_from_total_size(prefix, t)),
    }
}
