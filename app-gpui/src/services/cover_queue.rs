use crate::model::MainTask;
use crate::services::api;
use std::sync::mpsc::{self, Sender};

struct CoverJob {
    id: i64,
    base: String,
}

/// 单线程封面下载队列，避免滚动时短时间拉起大量 `thread::spawn`。
pub struct CoverQueue(Sender<CoverJob>);

impl CoverQueue {
    pub fn new(result_tx: Sender<MainTask>) -> Self {
        let (tx, rx) = mpsc::channel::<CoverJob>();
        std::thread::spawn(move || {
            while let Ok(job) = rx.recv() {
                let url = api::music_cover_jpeg_url(&job.base, job.id);
                let bytes = api::fetch_bytes(&url).ok().and_then(|b| {
                    (b.len() >= 3 && b[0] == 0xFF && b[1] == 0xD8).then_some(b)
                });
                let res = bytes.ok_or(());
                if result_tx
                    .send(MainTask::MusicCover {
                        id: job.id,
                        bytes: res,
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        Self(tx)
    }

    pub fn enqueue(&self, id: i64, base: String) {
        let _ = self.0.send(CoverJob { id, base });
    }
}
