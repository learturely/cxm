use std::{
    collections::HashMap,
    sync::{atomic::AtomicI32, Arc, Mutex},
};

use chrono::{Datelike, Local};
use cxsign::Session;
use log::debug;
use serde::{Deserialize, Serialize};

use crate::{
    live::Live,
    tools::{UnWrapEmit, VideoPath},
    RoomPair, PROG_STATE,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Room {
    #[serde(rename = "schoolRoomName")]
    pub(crate) name: String,
    #[serde(rename = "deviceCode")]
    pub(crate) device_code: String,
    #[serde(rename = "schoolRoomId")]
    pub(crate) room_id: i32,
    pub(crate) id: i32,
}

impl Room {
    pub fn into_room_pair(self) -> RoomPair {
        RoomPair {
            name: self.name,
            code: self.device_code,
        }
    }
    fn trim_end(mut self) -> Self {
        let mut i = self.name.len();
        for char in self.name.chars().rev() {
            if char.is_whitespace() {
                i -= char.len_utf8();
            } else {
                break;
            }
        }
        self.name.truncate(i);
        self
    }
    pub fn get_live_video_path(&self, session: &Session) -> VideoPath {
        crate::tools::get_live_video_path(session, &self.device_code)
    }
    // pub fn get_live_video_path(&self, session: &Session) -> VideoPath {
    //     crate::tools::get_live_video_path(session, &self.device_code)
    // }
    // pub fn get_live_url(&self, session: &Session) -> WebUrl {
    //     crate::tools::get_live_web_url(session, &self.device_code)
    // }
    pub fn get_rooms(session: &Session, live_id: i32) -> Result<Option<Room>, Box<ureq::Error>> {
        debug!("list_single_course, live_id: {live_id}");
        let rooms: Vec<Room> = crate::protocol::list_single_course(session, live_id)?
            .into_json()
            .unwrap();
        Ok(rooms
            .into_iter()
            .find(|r| r.id == live_id)
            .map(Self::trim_end))
    }
    // pub fn get_all_rooms<'a, Iter: Iterator<Item = &'a Session> + Clone>(
    //     mut sessions: Iter,
    // ) -> HashMap<String, String> {
    //     let map = Arc::new(Mutex::new(HashMap::new()));
    //     Room::get_all_live_id(
    //         &sessions.clone().collect::<Vec<_>>(),
    //         Arc::clone(&map),
    //         None,
    //     );
    //     let rooms = Arc::new(Mutex::new(HashMap::new()));
    //     if let Some(session) = sessions.next() {
    //         debug!(
    //             "call_from_js: list_rooms/id_to_rooms: {}",
    //             session.get_stu_name()
    //         );
    //         Room::id_to_rooms(map.clone(), (*session).clone(), rooms.clone(), None);
    //     }
    //     Arc::into_inner(rooms).unwrap().into_inner().unwrap()
    // }
    pub fn get_all_live_id(
        sessions: &[&Session],
        id_map: Arc<Mutex<HashMap<String, i32>>>,
        app: Option<tauri::AppHandle>,
    ) {
        debug!("list_rooms/get_all_live_id: set_prog_state_to_true.");
        PROG_STATE.fetch_or(true, std::sync::atomic::Ordering::Relaxed);
        // let _ = app
        //     .emit("step1:started", 114514)
        //     .expect("emit `step1:started` failed.");
        let now_year = Local::now().year();
        let done = Arc::new(AtomicI32::new(0));
        let thread_count = 64 / sessions.len() as i32;
        let week_total = 6 * 60;
        let total = (week_total * sessions.len() as i32) as f32;
        let mut handles = Vec::new();
        for session in sessions.iter() {
            let week_thread = week_total / (thread_count - 1) + 1;
            let thread_count = week_total / week_thread + 1;
            let week_rest = week_total % week_thread;
            for i in 0..thread_count {
                if !PROG_STATE.load(std::sync::atomic::Ordering::Relaxed) {
                    debug!("list_rooms/get_all_live_id: break.");
                    break;
                }
                let session = (*session).clone();
                let id_map = Arc::clone(&id_map);
                let done = Arc::clone(&done);
                let app = app.clone();
                let handle = std::thread::spawn(move || {
                    for date_count in i * week_thread..if i != thread_count - 1 {
                        (i + 1) * week_thread
                    } else {
                        i * week_thread + week_rest
                    } {
                        if !PROG_STATE.load(std::sync::atomic::Ordering::Relaxed) {
                            debug!("list_rooms/get_all_live_id: break.");
                            break;
                        }
                        let (year, term, week) =
                            crate::tools::date_count_to_year_term_week(now_year, date_count);
                        let lives = Live::get_lives(&session, week, year, term).unwrap();
                        for live in lives {
                            id_map.lock().unwrap().insert(live.0, live.1);
                        }
                        done.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        debug!("emit `step1:set-progress`.");
                        app.unwrap_emit(
                            "step1:set-progress",
                            done.load(std::sync::atomic::Ordering::Relaxed) as f32 / total * 100.0,
                        )
                            .expect("emit `step1:set-progress` failed.");
                    }
                });
                handles.push(handle);
            }
        }
        for handle in handles {
            handle.join().unwrap();
        }
        if PROG_STATE.load(std::sync::atomic::Ordering::Relaxed) {
            app.unwrap_emit("step1:set-progress", 100.0)
                .expect("emit `step1:set-progress` failed.");
        }
    }
    pub fn id_to_rooms(
        id_map: Arc<Mutex<HashMap<String, i32>>>,
        session: Session,
        rooms: Arc<Mutex<HashMap<String, String>>>,
        app: Option<tauri::AppHandle>,
    ) {
        debug!("list_rooms/id_to_rooms: set_prog_state_to_true.");
        PROG_STATE.fetch_or(true, std::sync::atomic::Ordering::Relaxed);
        app.unwrap_emit("step2:started", 114514)
            .expect("emit `step2:started` failed.");
        let ids = id_map.lock().unwrap().values().copied().collect::<Vec<_>>();
        let len = ids.len() as i32;
        let total = len as f32;
        let thread_count = 64;
        let chunk_rest = len % thread_count;
        let chunk_count = len / thread_count + if chunk_rest == 0 { 0 } else { 1 };
        let done = Arc::new(AtomicI32::new(0));
        for i in 0..chunk_count {
            let mut handles = Vec::new();
            if !PROG_STATE.load(std::sync::atomic::Ordering::Relaxed) {
                debug!("list_rooms/id_to_rooms: break.");
                break;
            }
            let ids = &ids[(i * thread_count) as usize..if i != chunk_count - 1 {
                (i + 1) * thread_count
            } else {
                len
            } as usize];
            for id in ids {
                let id = *id;
                let session = session.clone();
                let rooms = rooms.clone();
                let done = done.clone();
                let app = app.clone();
                let handle = std::thread::spawn(move || {
                    if !PROG_STATE.load(std::sync::atomic::Ordering::Relaxed) {
                        debug!("list_rooms/id_to_rooms: break.");
                        return;
                    }
                    let room = Room::get_rooms(&session, id).unwrap();
                    if let Some(room) = room {
                        rooms.lock().unwrap().insert(room.name, room.device_code);
                    }
                    done.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    debug!("emit `step2:set-progress`.");
                    app.unwrap_emit(
                        "step2:set-progress",
                        done.load(std::sync::atomic::Ordering::Relaxed) as f32 / total * 100.0,
                    )
                        .expect("emit `step2:set-progress` failed.");
                });
                handles.push(handle)
            }
            for handle in handles {
                handle.join().unwrap();
            }
        }
        if PROG_STATE.load(std::sync::atomic::Ordering::Relaxed) {
            app.unwrap_emit("step2:set-progress", 100.0)
                .expect("emit `step2:set-progress` failed.");
        }
        app.unwrap_emit("step2:stopped", 114514)
            .expect("emit `step2:started` failed.");
    }
}
