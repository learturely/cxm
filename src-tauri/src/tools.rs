use log::{debug, error};
use serde::{Serialize};
use std::{
    sync::atomic::{AtomicU64},
};
use tauri::{AppHandle, Emitter};
use xddcc::{ProgressState, ProgressTracker, ProgressTrackerHolder};
use crate::state::PROG_STATE;

pub trait UnWrapEmit {
    fn unwrap_emit<S: Serialize + Clone>(
        &self,
        event: &str,
        payload: S,
    ) -> Result<(), tauri::Error>;
}

impl UnWrapEmit for Option<AppHandle> {
    fn unwrap_emit<S: Serialize + Clone>(
        &self,
        event: &str,
        payload: S,
    ) -> Result<(), tauri::Error> {
        if let Some(app) = self {
            app.emit(event, payload)
        } else {
            xddcc::out(&payload, None);
            Ok(())
        }
    }
}

impl UnWrapEmit for AppHandle {
    fn unwrap_emit<S: Serialize + Clone>(
        &self,
        event: &str,
        payload: S,
    ) -> Result<(), tauri::Error> {
        self.emit(event, payload)
    }
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
pub fn capture_screen_for_enc() -> Option<String> {
    let screens = xcap::Monitor::all().unwrap_or_else(|e| panic!("{e:?}"));
    for screen in screens {
        // 先截取整个屏幕。
        let pic = screen.capture_image().unwrap_or_else(|e| panic!("{e:?}"));
        log::info!("已截屏。");
        // 如果成功识别到二维码。
        use std::collections::HashMap;
        let results = cxsign::qrcode_utils::scan_qrcode(
            xcap::image::DynamicImage::from(pic),
            &mut HashMap::new(),
        );
        let results = if let Ok(results) = results {
            results
        } else {
            continue;
        };
        // 在结果中寻找。
        for r in &results {
            let url = r.getText();
            // 如果符合要求的二维码。
            if !cxsign::qrcode_utils::is_enc_qrcode_url(url) {
                log::warn!("{url:?}不是有效的签到二维码！");
                continue;
            }
            log::info!("存在签到二维码。");
            // 如果不是精确截取的二维码，则不需要提示。
            return cxsign::utils::find_qrcode_sign_enc_in_url(url);
        }
    }
    None
}

pub struct ProgressBar {
    signal: String,
    app: AppHandle,
    total: u64,
    done: AtomicU64,
}

impl ProgressTracker for ProgressBar {
    fn inc(&self, delta: u64) {
        self.done.fetch_add(delta, std::sync::atomic::Ordering::Relaxed);
        debug!("emit `{}:set-progress`.",self.signal);
        self.app.unwrap_emit(
            &format!("{}:set-progress", self.signal),
            self.done.load(std::sync::atomic::Ordering::Relaxed) as f32 / self.total as f32 * 100.0,
        )
            .unwrap_or_else(|_| {
                error!("emit `{}:set-progress` failed.",self.signal);
                panic!()
            });
    }

    fn go_on(&self) -> bool {
        PROG_STATE.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn finish(&self, data: ProgressState) {
        if self.go_on() {
            self.app.unwrap_emit(
                &format!("{}:set-progress", self.signal), 100.0)
                .unwrap_or_else(|_| {
                    error!("emit `{}:set-progress` failed.",self.signal);
                    panic!()
                });
        }
        match data {
            ProgressState::GetLiveUrls | ProgressState::GetDeviceCodes => {
                self.app.unwrap_emit(
                    &format!("{}:stopped", self.signal), 114514)
                    .unwrap_or_else(|_| {
                        error!("emit `{}:stopped` failed.",self.signal);
                        panic!()
                    });
            }
            _ => {}
        }
    }
}

impl ProgressTrackerHolder<ProgressBar> for AppHandle {
    fn init(&self, total: u64, data: ProgressState) -> ProgressBar {
        PROG_STATE.store(true, std::sync::atomic::Ordering::Relaxed);
        match data {
            ProgressState::GetLiveIds => {
                debug!("list_rooms/get_all_live_id: set_prog_state_to_true.");
            }
            ref r @ (ProgressState::GetLiveUrls | ProgressState::GetDeviceCodes) => {
                if let ProgressState::GetDeviceCodes = r {
                    debug!("list_rooms/id_to_rooms: set_prog_state_to_true.");
                }
                debug!("emit `step2:started`.");
                self.unwrap_emit("step2:started", 114514)
                    .expect("emit `step2:started` failed.");
            }
            _ => {}
        };
        let signal = match data {
            ProgressState::GetRecordingLives => { "bug" }
            ProgressState::GetLiveIds => { "step1" }
            ProgressState::GetLiveUrls | ProgressState::GetDeviceCodes => { "step2" }
            _ => { "bug" }
        }.to_string();
        let done = AtomicU64::new(0);
        ProgressBar {
            signal,
            app: self.clone(),
            total,
            done,
        }
    }

    fn remove_progress(&self, _: &ProgressBar) {}
}