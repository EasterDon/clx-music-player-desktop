use kira::{
    AudioManager, AudioManagerSettings, DefaultBackend, Tween,
    sound::PlaybackState,
    sound::streaming::{StreamingSoundData, StreamingSoundHandle},
};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

pub enum AudioToUi {
    Tick {
        music_id: i64,
        position_secs: f64,
        duration_secs: f64,
    },
    Loaded {
        music_id: i64,
        duration_secs: f64,
    },
    /// 当前曲目自然播放结束（解码器进入 Stopped）。
    Ended {
        music_id: i64,
    },
    Error(String),
}

pub enum UiToAudio {
    LoadStream {
        music_id: i64,
        path: PathBuf,
        duration_secs: f64,
    },
    /// 立即停止当前声音并清空内部状态（切换曲目时发送）。
    Stop,
    Resume,
    Pause,
    Seek(f64),
}

pub struct AudioBridge {
    pub to_audio: Sender<UiToAudio>,
}

impl AudioBridge {
    pub fn spawn(ui_tx: std::sync::mpsc::Sender<crate::model::MainTask>) -> Self {
        let (to_audio, from_ui) = mpsc::channel::<UiToAudio>();
        std::thread::spawn(move || {
            run_audio_loop(from_ui, ui_tx);
        });
        AudioBridge { to_audio }
    }
}

type StreamHandle = StreamingSoundHandle<kira::sound::FromFileError>;

fn play_stream(
    manager: &mut AudioManager<DefaultBackend>,
    path: &Path,
    hinted_duration: f64,
) -> Result<(StreamHandle, f64), String> {
    let data = StreamingSoundData::from_file(path).map_err(|e| format!("解码音频失败: {e}"))?;
    let duration_secs = if hinted_duration > 0.0 {
        hinted_duration
    } else {
        data.duration().as_secs_f64()
    };
    let handle = manager.play(data).map_err(|e| format!("播放失败: {e}"))?;
    Ok((handle, duration_secs))
}

fn send_tick(ui_tx: &Sender<crate::model::MainTask>, id: i64, pos: f64, dur: f64) {
    let _ = ui_tx.send(crate::model::MainTask::Audio(AudioToUi::Tick {
        music_id: id,
        position_secs: pos,
        duration_secs: dur,
    }));
}

fn at_track_end(position_secs: f64, duration_secs: f64) -> bool {
    duration_secs > 0.0 && position_secs >= duration_secs - 0.05
}

fn resume_or_replay(
    manager: &mut AudioManager<DefaultBackend>,
    handle: &mut Option<StreamHandle>,
    stream_path: &Option<PathBuf>,
    track_id: Option<i64>,
    duration_secs: f64,
    ended_notified: &mut bool,
    ui_tx: &Sender<crate::model::MainTask>,
) {
    let Some(id) = track_id else {
        return;
    };
    let Some(h) = handle.as_mut() else {
        return;
    };

    if h.state() == PlaybackState::Stopped {
        let Some(path) = stream_path.clone() else {
            return;
        };
        match play_stream(manager, &path, duration_secs) {
            Ok((mut new_h, dur)) => {
                // 旧声音虽已自然结束，但仍可能驻留在音频引擎中，
                // 需显式 stop 后再丢弃，避免与新声音叠加播放。
                if let Some(old) = handle.as_mut() {
                    old.stop(Tween::default());
                }
                *ended_notified = false;
                let _ = new_h.resume(Tween::default());
                *handle = Some(new_h);
                send_tick(ui_tx, id, 0.0, dur);
            }
            Err(e) => {
                let _ = ui_tx.send(crate::model::MainTask::Audio(AudioToUi::Error(e)));
            }
        }
        return;
    }

    if at_track_end(h.position(), duration_secs) {
        h.seek_to(0.0);
        send_tick(ui_tx, id, 0.0, duration_secs);
    }
    let _ = h.resume(Tween::default());
}

fn run_audio_loop(
    from_ui: Receiver<UiToAudio>,
    ui_tx: std::sync::mpsc::Sender<crate::model::MainTask>,
) {
    let mut manager = match AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()) {
        Ok(m) => m,
        Err(e) => {
            let _ = ui_tx.send(crate::model::MainTask::Audio(AudioToUi::Error(format!(
                "音频引擎初始化失败: {e}"
            ))));
            return;
        }
    };

    let mut handle: Option<StreamHandle> = None;
    let mut duration_secs: f64 = 0.0;
    let mut track_id: Option<i64> = None;
    let mut stream_path: Option<PathBuf> = None;
    let mut want_playing = false;
    let mut ended_notified = false;
    let mut last_tick = Instant::now();
    const TICK_INTERVAL: Duration = Duration::from_millis(200);

    loop {
        while let Ok(cmd) = from_ui.try_recv() {
            match cmd {
                UiToAudio::LoadStream {
                    music_id,
                    path,
                    duration_secs: hinted_duration,
                } => {
                    // 丢弃旧声音前先显式停止：Kira 的 StreamingSoundHandle
                    // 在 drop 时不会停止声音，否则旧曲会继续在引擎中播放。
                    if let Some(old) = handle.as_mut() {
                        old.stop(Tween::default());
                    }
                    handle = None;
                    stream_path = Some(path.clone());
                    ended_notified = false;
                    match play_stream(&mut manager, &path, hinted_duration) {
                        Ok((mut h, dur)) => {
                            duration_secs = dur;
                            let _ = ui_tx.send(crate::model::MainTask::Audio(AudioToUi::Loaded {
                                music_id,
                                duration_secs,
                            }));
                            if !want_playing {
                                let _ = h.pause(Tween::default());
                            }
                            handle = Some(h);
                            track_id = Some(music_id);
                        }
                        Err(e) => {
                            track_id = None;
                            let _ = ui_tx.send(crate::model::MainTask::Audio(AudioToUi::Error(e)));
                        }
                    }
                }
                UiToAudio::Stop => {
                    want_playing = false;
                    ended_notified = false;
                    if let Some(mut old) = handle.take() {
                        old.stop(Tween::default());
                    }
                    track_id = None;
                    stream_path = None;
                    duration_secs = 0.0;
                }
                UiToAudio::Resume => {
                    want_playing = true;
                    resume_or_replay(
                        &mut manager,
                        &mut handle,
                        &stream_path,
                        track_id,
                        duration_secs,
                        &mut ended_notified,
                        &ui_tx,
                    );
                }
                UiToAudio::Pause => {
                    want_playing = false;
                    if let Some(ref mut h) = handle {
                        let _ = h.pause(Tween::default());
                    }
                }
                UiToAudio::Seek(sec) => {
                    if let Some(ref mut h) = handle {
                        if h.state() == PlaybackState::Stopped {
                            continue;
                        }
                        let t = sec.max(0.0);
                        h.seek_to(t);
                        ended_notified = false;
                        last_tick = Instant::now();
                        if let Some(id) = track_id {
                            send_tick(&ui_tx, id, t, duration_secs);
                        }
                    }
                }
            }
        }

        if let (Some(id), Some(ref h)) = (track_id, handle.as_ref()) {
            if h.state() == PlaybackState::Stopped {
                if !ended_notified {
                    ended_notified = true;
                    want_playing = false;
                    let _ = ui_tx.send(crate::model::MainTask::Audio(AudioToUi::Ended {
                        music_id: id,
                    }));
                }
            } else if h.state() == PlaybackState::Playing && last_tick.elapsed() >= TICK_INTERVAL {
                last_tick = Instant::now();
                send_tick(&ui_tx, id, h.position(), duration_secs);
            }
        }

        std::thread::sleep(Duration::from_millis(32));
    }
}
