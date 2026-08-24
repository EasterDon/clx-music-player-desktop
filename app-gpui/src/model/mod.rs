use crate::lyrics::{self, LyricLine};
use crate::services::api::{self, AppInfo, MusicItem};
use crate::services::app_config::{self, AppConfig};
use crate::services::audio::{AudioBridge, AudioToUi, UiToAudio};
use crate::services::cover_queue::CoverQueue;
use clx_core::elevation;
use crate::services::hotkey::HotkeyKind;
use crate::services::music_cache;
use gpui::{Image, ImageFormat};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

pub const APP_VERSION: &str = env!("CLX_APP_VERSION");

/// Toast 显示后自动关闭的最长时间。
pub const TOAST_AUTO_DISMISS_SECS: u64 = 10;

/// 主任务应用后应刷新哪些 UI 区域（避免播放 Tick 触发整页重绘）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiRefresh {
    #[default]
    None,
    /// 列表、设置、弹窗、歌词文本等
    Full,
    /// 进度条、歌词高亮与滚动
    Playback,
    /// 仅封面图块
    Covers,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayMode {
    Listen,
    Script,
}

pub enum MainTask {
    MusicList(Result<Vec<MusicItem>, String>),
    AppInfo(Result<AppInfo, String>),
    Lyrics {
        music_id: i64,
        text: Result<String, String>,
    },
    DurationHint {
        music_id: i64,
        duration_secs: f64,
    },
    /// 当前曲目 MP3 缓存下载进度（字节）。
    LoadProgress {
        music_id: i64,
        bytes_loaded: u64,
        total_bytes: Option<u64>,
    },
    // 前缀已写入本地缓存，可开始流式播放；剩余字节在后台继续下载。
    StreamReady {
        music_id: i64,
        path: std::path::PathBuf,
        duration_secs: f64,
    },

    Audio(AudioToUi),
    Hotkey(HotkeyKind),
    ScriptFinished,
    Toast(String),
    MusicCover {
        id: i64,
        bytes: Result<Vec<u8>, ()>,
    },
}

fn send_task(tx: &Sender<MainTask>, task: MainTask) {
    if let Err(e) = tx.send(task) {
        eprintln!("UI 通道已关闭: {e}");
    }
}

pub struct ClxState {
    pub ui_tx: Sender<MainTask>,
    pub base_url: String,
    pub audio: AudioBridge,
    pub script_continue: Arc<AtomicBool>,
    /// 切歌时递增；后台拉取在发送结果前校验，丢弃过期任务。
    load_generation: Arc<AtomicU64>,
    cover_queue: CoverQueue,

    pub music_list: Vec<MusicItem>,
    /// 与 Tauri 一致：按曲名筛选后的展示列表（避免每帧 clone 全库）。
    pub filtered_display: Vec<MusicItem>,
    pub app_info: AppInfo,
    pub filter: String,
    pub current: Option<MusicItem>,
    pub lyrics: Arc<Vec<LyricLine>>,
    pub current_lyric_index: Option<usize>,
    pub play_mode: PlayMode,
    /// 下次启动是否以管理员身份运行（Windows）。
    pub launch_as_administrator: bool,
    pub is_playing: bool,
    pub current_time: f64,
    pub duration: f64,
    /// 当前曲目已写入缓存的字节数。
    pub load_bytes: u64,
    /// 当前曲目 MP3 总字节数（已知时）。
    pub load_total_bytes: Option<u64>,
    /// seek 后短暂忽略与目标偏差较大的 Tick，避免进度条回跳。
    pending_seek_time: Option<f64>,
    pending_seek_at: Option<Instant>,
    pub show_settings: bool,
    pub modal_script: bool,
    pub toast: Option<String>,
    toast_queue: VecDeque<String>,
    toast_shown_at: Option<Instant>,
    pub music_covers: Arc<HashMap<i64, Arc<Image>>>,
    pub music_cover_failed: HashSet<i64>,
    pub music_cover_inflight: HashSet<i64>,
}

impl ClxState {
    pub fn new(ui_tx: Sender<MainTask>) -> Self {
        let audio = AudioBridge::spawn(ui_tx.clone());
        let cover_queue = CoverQueue::new(ui_tx.clone());
        // 服务端地址只从项目根目录 .env 读取，源码中不写死线上地址。
        let base_url = app_config::resolve_base_url();
        let tx = ui_tx.clone();
        if base_url.trim().is_empty() {
            send_task(
                &tx,
                MainTask::Toast(
                    "未配置服务器地址：请在项目根目录 .env 填写 CLX_BASE_URL 后重新构建".into(),
                ),
            );
        } else {
            let b = base_url.clone();
            thread::spawn(move || {
                send_task(&tx, MainTask::MusicList(api::get_music_list(&b)));
                send_task(&tx, MainTask::AppInfo(api::get_app_info(&b)));
            });
        }

        let app_cfg = app_config::load();
        let launch_as_administrator = app_cfg.launch_as_administrator;
        let mut toast_queue = VecDeque::new();
        if let Some(msg) = elevation::startup_elevation_toast(launch_as_administrator) {
            toast_queue.push_back(msg);
        }
        let initial_toast = toast_queue.pop_front();
        let toast_shown_at = initial_toast.as_ref().map(|_| Instant::now());

        Self {
            ui_tx,
            base_url,
            audio,
            script_continue: Arc::new(AtomicBool::new(true)),
            load_generation: Arc::new(AtomicU64::new(0)),
            cover_queue,
            music_list: Vec::new(),
            filtered_display: Vec::new(),
            app_info: AppInfo {
                id: 1,
                announcement: String::new(),
                app_version: APP_VERSION.to_string(),
                app_download_link: String::new(),
                app_version_description: String::new(),
            },
            filter: String::new(),
            current: None,
            lyrics: Arc::new(Vec::new()),
            current_lyric_index: None,
            play_mode: PlayMode::Listen,
            launch_as_administrator,
            is_playing: false,
            current_time: 0.0,
            duration: 0.0,
            load_bytes: 0,
            load_total_bytes: None,
            pending_seek_time: None,
            pending_seek_at: None,
            show_settings: false,
            modal_script: false,
            toast: initial_toast,
            toast_queue,
            toast_shown_at,
            music_covers: Arc::new(HashMap::new()),
            music_cover_failed: HashSet::new(),
            music_cover_inflight: HashSet::new(),
        }
    }

    pub fn rebuild_filtered_display(&mut self) {
        let q = self.filter.trim().to_lowercase();
        if q.is_empty() {
            self.filtered_display.clone_from(&self.music_list);
            return;
        }
        self.filtered_display = self
            .music_list
            .iter()
            .filter(|m| m.name.to_lowercase().contains(&q))
            .cloned()
            .collect();
    }

    pub fn set_filter(&mut self, filter: String) {
        self.filter = filter;
        self.rebuild_filtered_display();
    }

    pub(crate) fn push_toast(&mut self, msg: String) {
        if self.toast.is_none() {
            self.toast = Some(msg);
            self.toast_shown_at = Some(Instant::now());
        } else {
            self.toast_queue.push_back(msg);
        }
    }

    pub fn dismiss_toast(&mut self) {
        self.toast = self.toast_queue.pop_front();
        self.toast_shown_at = self.toast.as_ref().map(|_| Instant::now());
    }

    /// 若当前 Toast 已超过 [`TOAST_AUTO_DISMISS_SECS`]，则关闭并返回 `true`。
    pub fn tick_toast_auto_dismiss(&mut self) -> bool {
        if self.toast.is_none() {
            self.toast_shown_at = None;
            return false;
        }
        let shown_at = self.toast_shown_at.get_or_insert_with(Instant::now);
        if shown_at.elapsed() >= Duration::from_secs(TOAST_AUTO_DISMISS_SECS) {
            self.dismiss_toast();
            return true;
        }
        false
    }

    fn insert_cover(&mut self, id: i64, image: Image) {
        Arc::make_mut(&mut self.music_covers).insert(id, Arc::new(image));
    }

    pub fn enqueue_music_cover_fetches(&mut self, ids: Vec<i64>) {
        let base = self.base_url.clone();
        for id in ids {
            if self.music_covers.contains_key(&id) {
                continue;
            }
            if self.music_cover_failed.contains(&id) {
                continue;
            }
            if !self.music_cover_inflight.insert(id) {
                continue;
            }
            self.cover_queue.enqueue(id, base.clone());
        }
    }

    pub fn select_music(&mut self, item: MusicItem) {
        if self.current.as_ref().map(|c| c.id) == Some(item.id) {
            return;
        }
        // 切换曲目时彻底停止旧声音并清空音频线程状态，
        // 防止旧曲在加载新曲期间被 Resume 重新播放而叠加。
        let _ = self.audio.to_audio.send(UiToAudio::Stop);
        self.is_playing = false;
        self.current = Some(item.clone());
        self.lyrics = Arc::new(Vec::new());
        self.current_lyric_index = None;
        self.current_time = 0.0;
        self.duration = 0.0;
        self.load_bytes = 0;
        self.load_total_bytes = None;
        self.pending_seek_time = None;
        self.pending_seek_at = None;
        let load_gen = self.load_generation.fetch_add(1, Ordering::SeqCst) + 1;
        let base = self.base_url.clone();
        let id = item.id;
        let tx = self.ui_tx.clone();
        let gen_check = self.load_generation.clone();
        thread::spawn(move || {
            let tx_l = tx.clone();
            let base_l = base.clone();
            let gen_lyrics = gen_check.clone();
            thread::spawn(move || {
                if gen_lyrics.load(Ordering::SeqCst) != load_gen {
                    return;
                }
                let lyrics = api::get_music_lyrics(&base_l, id);
                if gen_lyrics.load(Ordering::SeqCst) != load_gen {
                    return;
                }
                send_task(
                    &tx_l,
                    MainTask::Lyrics {
                        music_id: id,
                        text: lyrics,
                    },
                );
            });

            let mp3_url = format!("{}/music/{id}/music.mp3", base.trim_end_matches('/'));
            const PREFIX_BYTES: usize = 512 * 1024;

            let still_current = || gen_check.load(Ordering::SeqCst) == load_gen;

            if !still_current() {
                return;
            }
            let _ = music_cache::ensure_cache_dir();
            let cache_path = music_cache::mp3_path(id);

            let head_total = api::head_content_length(&mp3_url).ok().flatten();

            if still_current()
                && head_total.is_some_and(|t| music_cache::is_complete_file(&cache_path, t))
            {
                let duration_secs = music_cache::probe_duration_from_file(&cache_path)
                    .filter(|d| *d > 0.0)
                    .unwrap_or(0.0);
                if duration_secs > 0.0 {
                    send_task(
                        &tx,
                        MainTask::DurationHint {
                            music_id: id,
                            duration_secs,
                        },
                    );
                }
                if still_current() {
                    if let Some(total) = head_total.filter(|t| *t > 0) {
                        send_task(
                            &tx,
                            MainTask::LoadProgress {
                                music_id: id,
                                bytes_loaded: total,
                                total_bytes: Some(total),
                            },
                        );
                    }
                    send_task(
                        &tx,
                        MainTask::StreamReady {
                            music_id: id,
                            path: cache_path,
                            duration_secs,
                        },
                    );
                }
                return;
            }

            let prefix = match api::write_mp3_prefix_to_file(&mp3_url, &cache_path, PREFIX_BYTES) {
                Ok(p) => p,
                Err(e) => {
                    if still_current() {
                        send_task(&tx, MainTask::Toast(format!("加载音频: {e}")));
                    }
                    return;
                }
            };

            if !still_current() {
                let _ = std::fs::remove_file(&cache_path);
                return;
            }

            let total = prefix.total_bytes.or(head_total);
            if let Some(secs) = prefix.duration_secs.filter(|d| *d > 0.0) {
                send_task(
                    &tx,
                    MainTask::DurationHint {
                        music_id: id,
                        duration_secs: secs,
                    },
                );
            }

            let duration_secs = prefix.duration_secs.unwrap_or(0.0);
            send_task(
                &tx,
                MainTask::LoadProgress {
                    music_id: id,
                    bytes_loaded: prefix.bytes_written,
                    total_bytes: total,
                },
            );
            send_task(
                &tx,
                MainTask::StreamReady {
                    music_id: id,
                    path: cache_path.clone(),
                    duration_secs,
                },
            );

            if still_current() {
                let mut last_report_bytes = prefix.bytes_written;
                let mut last_report_at = Instant::now();
                let mut report = |loaded: u64| {
                    if !still_current() {
                        return;
                    }
                    let done = total.is_some_and(|t| loaded >= t);
                    let delta = loaded.saturating_sub(last_report_bytes);
                    let due = delta >= 256 * 1024
                        || last_report_at.elapsed() >= Duration::from_millis(250)
                        || done;
                    if !due {
                        return;
                    }
                    last_report_bytes = loaded;
                    last_report_at = Instant::now();
                    send_task(
                        &tx,
                        MainTask::LoadProgress {
                            music_id: id,
                            bytes_loaded: loaded,
                            total_bytes: total,
                        },
                    );
                };
                if let Err(e) = api::append_mp3_remainder(
                    &mp3_url,
                    &cache_path,
                    prefix.bytes_written,
                    total,
                    Some(&mut report),
                ) {
                    if still_current() {
                        send_task(&tx, MainTask::Toast(format!("补全音频: {e}")));
                    }
                } else if still_current() {
                    if let Some(t) = total.filter(|t| *t > 0) {
                        send_task(
                            &tx,
                            MainTask::LoadProgress {
                                music_id: id,
                                bytes_loaded: t,
                                total_bytes: Some(t),
                            },
                        );
                    }
                }
            }
        });
    }

    pub fn play_playback(&mut self) {
        if self.current.is_none() {
            self.push_toast("请先选择一首歌曲哦".into());
            return;
        }
        let _ = self.audio.to_audio.send(UiToAudio::Resume);
        self.is_playing = true;
    }

    pub fn pause_playback(&mut self) {
        let _ = self.audio.to_audio.send(UiToAudio::Pause);
        self.is_playing = false;
    }

    /// 曲目自然播放结束：重置进度、播放状态与歌词位置（对齐 Tauri `audio_end`）。
    fn on_track_ended(&mut self) {
        self.is_playing = false;
        self.pending_seek_time = None;
        self.pending_seek_at = None;
        self.current_time = 0.0;
        self.current_lyric_index = if self.lyrics.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    /// 按已缓存字节比例估算可 seek 的最大时间点。
    pub fn buffered_end_secs(&self) -> f64 {
        if self.duration <= 0.0 {
            return 0.0;
        }
        let Some(total) = self.load_total_bytes.filter(|t| *t > 0) else {
            return if self.load_bytes > 0 {
                self.duration
            } else {
                0.0
            };
        };
        if self.load_bytes >= total {
            return self.duration;
        }
        self.duration * (self.load_bytes as f64 / total as f64)
    }

    pub fn seek(&mut self, sec: f64) {
        let max = self.buffered_end_secs().max(0.0);
        let t = sec.clamp(0.0, max);
        let _ = self.audio.to_audio.send(UiToAudio::Seek(t));
        self.pending_seek_time = Some(t);
        self.pending_seek_at = Some(Instant::now());
        self.current_time = t;
        self.sync_lyric_index();
    }

    /// seek 后解码器位置可能滞后；在短窗口内丢弃与目标偏差较大的 Tick，避免进度条回跳。
    fn apply_playback_tick(&mut self, position_secs: f64, duration_secs: f64) -> bool {
        if let (Some(target), Some(at)) = (self.pending_seek_time, self.pending_seek_at) {
            if at.elapsed() < Duration::from_millis(400) {
                if (position_secs - target).abs() > 0.4 {
                    return false;
                }
            } else {
                self.pending_seek_time = None;
                self.pending_seek_at = None;
            }
        }
        self.current_time = position_secs;
        self.duration = duration_secs;
        self.sync_lyric_index();
        true
    }

    pub fn previous_track(&mut self) {
        let Some(cur_id) = self.current.as_ref().map(|m| m.id) else {
            return;
        };
        self.pause_playback();
        let list = &self.music_list;
        if list.is_empty() {
            return;
        }
        let idx = list.iter().position(|m| m.id == cur_id).unwrap_or(0);
        let pre = if idx > 0 {
            list[idx - 1].clone()
        } else {
            list[list.len() - 1].clone()
        };
        self.select_music(pre);
    }

    pub fn next_track(&mut self) {
        let Some(cur_id) = self.current.as_ref().map(|m| m.id) else {
            return;
        };
        self.pause_playback();
        let list = &self.music_list;
        if list.is_empty() {
            return;
        }
        let idx = list.iter().position(|m| m.id == cur_id).unwrap_or(0);
        let nxt = if idx + 1 < list.len() {
            list[idx + 1].clone()
        } else {
            list[0].clone()
        };
        self.select_music(nxt);
    }

    pub fn sync_lyric_index(&mut self) {
        self.current_lyric_index = lyrics::lyric_index_at_time(&self.lyrics, self.current_time);
    }

    pub fn is_elevated_now() -> bool {
        elevation::is_elevated()
    }

    pub fn set_launch_as_administrator(&mut self, enabled: bool) -> Result<(), String> {
        self.launch_as_administrator = enabled;
        app_config::save(&AppConfig {
            launch_as_administrator: enabled,
        })
    }

    pub fn try_restart_as_admin() -> Result<(), String> {
        elevation::restart_as_admin()
    }

    pub fn handle_hotkey(&mut self, k: HotkeyKind) {
        match k {
            HotkeyKind::F1 => {
                if self.play_mode != PlayMode::Script {
                    return;
                }
                if self.current.is_none() {
                    return;
                }
                if self.modal_script {
                    return;
                }
                self.start_script_playback();
            }
            HotkeyKind::F2 => {
                if self.play_mode != PlayMode::Script {
                    return;
                }
                self.script_continue.store(false, Ordering::SeqCst);
            }
        }
    }

    fn start_script_playback(&mut self) {
        let Some(cur) = self.current.clone() else {
            return;
        };
        self.script_continue.store(true, Ordering::SeqCst);
        self.modal_script = true;
        let base = self.base_url.clone();
        let tx = self.ui_tx.clone();
        let flag = self.script_continue.clone();
        thread::spawn(move || {
            // Enigo 须在执行按键的线程上创建（Windows 上跨线程模拟输入常失效）。
            let enigo = match clx_core::new_shared_enigo() {
                Ok(e) => e,
                Err(e) => {
                    send_task(&tx, MainTask::Toast(format!("初始化键鼠模拟失败: {e}")));
                    send_task(&tx, MainTask::ScriptFinished);
                    return;
                }
            };
            let notation = match api::get_music_notation(&base, cur.id) {
                Ok(n) => n,
                Err(e) => {
                    send_task(&tx, MainTask::Toast(e));
                    send_task(&tx, MainTask::ScriptFinished);
                    return;
                }
            };
            run_script_notation(&notation, &enigo, &flag, &tx);
            send_task(&tx, MainTask::ScriptFinished);
        });
    }
}

fn run_script_notation(
    notation: &[serde_json::Value],
    enigo: &clx_core::SharedEnigo,
    flag: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    tx: &Sender<MainTask>,
) {
    for v in notation {
        if !flag.load(Ordering::SeqCst) {
            break;
        }
        if let Some(ms) = v.as_f64() {
            if ms > 0.0 {
                thread::sleep(Duration::from_millis(ms as u64));
            }
        } else if let Some(ms) = v.as_u64() {
            thread::sleep(Duration::from_millis(ms));
        } else if let Some(ms) = v.as_i64() {
            if ms > 0 {
                thread::sleep(Duration::from_millis(ms as u64));
            }
        } else if let Some(s) = v.as_str() {
            for ch in s.chars() {
                if !flag.load(Ordering::SeqCst) {
                    break;
                }
                let key = ch.to_string();
                if let Ok(mut g) = enigo.lock() {
                    if let Err(e) = clx_core::click(&mut *g, &key) {
                        send_task(&tx, MainTask::Toast(format!("按键失败: {e}")));
                    }
                } else {
                    send_task(&tx, MainTask::Toast("键鼠模拟器被占用".into()));
                }
            }
        }
    }
}

impl ClxState {
    pub fn apply_main_task(&mut self, task: MainTask) -> UiRefresh {
        match task {
            MainTask::MusicList(res) => match res {
                Ok(list) => {
                    self.music_list = list;
                    self.rebuild_filtered_display();
                    self.music_covers = Arc::new(HashMap::new());
                    self.music_cover_failed.clear();
                    self.music_cover_inflight.clear();
                    let preload: Vec<i64> = self.music_list.iter().take(40).map(|m| m.id).collect();
                    self.enqueue_music_cover_fetches(preload);
                    UiRefresh::Full
                }
                Err(e) => {
                    self.push_toast(e);
                    UiRefresh::Full
                }
            },
            MainTask::MusicCover { id, bytes } => {
                self.music_cover_inflight.remove(&id);
                match bytes {
                    Ok(b) => {
                        self.insert_cover(id, Image::from_bytes(ImageFormat::Jpeg, b));
                    }
                    Err(()) => {
                        self.music_cover_failed.insert(id);
                    }
                }
                UiRefresh::Covers
            }
            MainTask::AppInfo(res) => {
                match res {
                    Ok(info) => self.app_info = info,
                    Err(e) => self.push_toast(e),
                }
                UiRefresh::Full
            }
            MainTask::Lyrics { music_id, text } => {
                if self.current.as_ref().map(|c| c.id) != Some(music_id) {
                    return UiRefresh::None;
                }
                match text {
                    Ok(s) => {
                        self.lyrics = Arc::new(lyrics::parse_lrc_text(&s));
                        self.current_lyric_index = lyrics::lyric_index_at_time(&self.lyrics, 0.0);
                    }
                    Err(e) => self.push_toast(e),
                }
                UiRefresh::Full
            }
            MainTask::DurationHint {
                music_id,
                duration_secs,
            } => {
                if self.current.as_ref().map(|c| c.id) != Some(music_id) {
                    return UiRefresh::None;
                }
                self.duration = duration_secs;
                UiRefresh::Playback
            }
            MainTask::LoadProgress {
                music_id,
                bytes_loaded,
                total_bytes,
            } => {
                if self.current.as_ref().map(|c| c.id) != Some(music_id) {
                    return UiRefresh::None;
                }
                self.load_bytes = bytes_loaded;
                if let Some(t) = total_bytes.filter(|t| *t > 0) {
                    self.load_total_bytes = Some(t);
                }
                UiRefresh::Playback
            }
            MainTask::StreamReady {
                music_id,
                path,
                duration_secs,
            } => {
                if self.current.as_ref().map(|c| c.id) != Some(music_id) {
                    return UiRefresh::None;
                }
                let _ = self.audio.to_audio.send(UiToAudio::LoadStream {
                    music_id,
                    path,
                    duration_secs,
                });
                if self.is_playing {
                    let _ = self.audio.to_audio.send(UiToAudio::Resume);
                }
                UiRefresh::None
            }
            MainTask::Audio(a) => match a {
                AudioToUi::Tick {
                    music_id,
                    position_secs,
                    duration_secs,
                } => {
                    if self.current.as_ref().map(|c| c.id) != Some(music_id) {
                        return UiRefresh::None;
                    }
                    if self.apply_playback_tick(position_secs, duration_secs) {
                        UiRefresh::Playback
                    } else {
                        UiRefresh::None
                    }
                }
                AudioToUi::Loaded {
                    music_id,
                    duration_secs,
                } => {
                    if self.current.as_ref().map(|c| c.id) != Some(music_id) {
                        return UiRefresh::None;
                    }
                    self.duration = duration_secs;
                    if self.is_playing {
                        let _ = self.audio.to_audio.send(UiToAudio::Resume);
                    }
                    UiRefresh::Playback
                }
                AudioToUi::Ended { music_id, .. } => {
                    if self.current.as_ref().map(|c| c.id) != Some(music_id) {
                        return UiRefresh::None;
                    }
                    self.on_track_ended();
                    UiRefresh::Playback
                }
                AudioToUi::Error(e) => {
                    self.push_toast(e);
                    UiRefresh::Full
                }
            },
            MainTask::Hotkey(k) => {
                self.handle_hotkey(k);
                UiRefresh::Full
            }
            MainTask::ScriptFinished => {
                self.modal_script = false;
                self.script_continue.store(false, Ordering::SeqCst);
                UiRefresh::Full
            }
            MainTask::Toast(s) => {
                self.push_toast(s);
                UiRefresh::Full
            }
        }
    }
}
