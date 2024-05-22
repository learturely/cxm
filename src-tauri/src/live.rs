use std::{
    collections::{HashMap, HashSet},
    sync::{atomic::AtomicI32, Arc},
};
use cxsign::user::Session;
use log::debug;
use serde::{Deserialize, Serialize};

use crate::{
    room::Room,
    tools::{UnWrapEmit, VideoPath},
    PROG_STATE,
};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Live {
    place: String,
    id: i32,
    #[serde(rename = "weekDay")]
    week_day: u32,
    jie: i32,
}

impl Live {
    pub fn get_id(&self) -> i32 {
        self.id
    }
    pub fn get_week_day(&self) -> u32 {
        self.week_day
    }
    fn get_jie(&self) -> i32 {
        self.jie
    }
    pub fn get_lives(
        session: &Session,
        week: i64,
        term_year: i32,
        term: i32,
    ) -> Result<HashMap<String, i32>, Box<ureq::Error>> {
        debug!("list_student_course_live_page, term_year: {term_year}, term: {term}, week: {week}");
        let vec = crate::protocol::list_student_course_live_page(session, week, term_year, term)?
            .into_json::<Vec<Live>>()
            .unwrap();
        let mut map = HashMap::new();
        for i in vec {
            map.insert(i.place, i.id);
        }
        Ok(map)
    }
    fn get_lives_by_time(
        session: &Session,
        term_year: i32,
        term: i32,
        week: i64,
        week_day: u32,
        jie: i32,
    ) -> Result<Option<Live>, Box<ureq::Error>> {
        debug!("list_student_course_live_page, term_year: {term_year}, term: {term}, week: {week}");
        let vec = crate::protocol::list_student_course_live_page(session, week, term_year, term)?
            .into_json::<Vec<Live>>()
            .unwrap();
        debug!("{vec:?}");
        debug!("jie: {jie}");
        let iter = vec
            .into_iter()
            .filter(|live| (live.get_week_day() == week_day) && (live.get_jie() >= jie));
        let mut vec = iter.collect::<Vec<_>>();
        vec.sort_by_key(Live::get_jie);
        Ok(vec.first().cloned())
    }
    pub fn get_lives_now<'a, Iter: Iterator<Item=&'a Session> + Clone>(
        sessions: Iter,
        app: tauri::AppHandle,
    ) -> HashMap<&'a str, (&'a str, Room, VideoPath)> {
        PROG_STATE.fetch_or(true, std::sync::atomic::Ordering::Relaxed);
        let sessions = sessions.collect::<Vec<_>>();
        let done = Arc::new(AtomicI32::new(0));
        let total = sessions.len() as u64;
        let data_time = chrono::DateTime::<chrono::Local>::from(std::time::SystemTime::now());
        let mut term_year = 0;
        let mut term = 0;
        let mut week = 0;
        let mut first = true;
        let week_day = chrono::Datelike::weekday(&data_time).number_from_monday();
        let mut lives_map = HashMap::new();
        // let sty = indicatif::ProgressStyle::with_template(
        //     "获取直播号：[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        // )
        // .unwrap();
        // let pb = indicatif::ProgressBar::new(total);
        // pb.set_style(sty);
        for session in sessions.clone() {
            if !PROG_STATE.load(std::sync::atomic::Ordering::Relaxed) {
                debug!("list_rooms/get_all_live_id: break.");
                break;
            }
            if first {
                (term_year, term, week) = crate::tools::term_year_detial(session);
                first = false;
            }
            let jie = crate::tools::now_to_jie();
            let live = Live::get_lives_by_time(session, term_year, term, week, week_day, jie);
            if let Ok(Some(live)) = live {
                lives_map.insert(session, live);
            }
            done.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            debug!("emit `step1:set-progress`.");
            app.unwrap_emit(
                "step1:set-progress",
                done.load(std::sync::atomic::Ordering::Relaxed) as f32 / total as f32 * 100.0,
            )
                .expect("emit `step1:set-progress` failed.");
        }
        if PROG_STATE.load(std::sync::atomic::Ordering::Relaxed) {
            app.unwrap_emit("step1:set-progress", 100.0)
                .expect("emit `step1:set-progress` failed.");
        }
        // pb.finish_with_message("获取直播号完成。");
        PROG_STATE.fetch_or(true, std::sync::atomic::Ordering::Relaxed);
        app.unwrap_emit("step2:started", 114514)
            .expect("emit `step2:started` failed.");
        let mut lives = HashSet::new();
        for live in lives_map.values() {
            lives.insert(live.get_id());
        }
        let mut rooms = HashMap::new();
        // let sty = indicatif::ProgressStyle::with_template(
        //     "获取地址中：[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        // )
        // .unwrap();
        // let pb = indicatif::ProgressBar::new(lives.len() as u64 * 2);
        // pb.set_style(sty);
        // pb.inc(0);
        if let Some(session) = sessions.first() {
            for live in lives {
                if !PROG_STATE.load(std::sync::atomic::Ordering::Relaxed) {
                    debug!("list_rooms/id_to_rooms: break.");
                    break;
                }
                if let Some(room) = Room::get_rooms(session, live).unwrap() {
                    done.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    debug!("emit `step2:set-progress`.");
                    app.unwrap_emit(
                        "step2:set-progress",
                        done.load(std::sync::atomic::Ordering::Relaxed) as f32 / total as f32
                            * 100.0,
                    )
                        .expect("emit `step2:set-progress` failed.");
                    let video_path = room.get_live_video_path(session);
                    done.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    debug!("emit `step2:set-progress`.");
                    app.unwrap_emit(
                        "step2:set-progress",
                        done.load(std::sync::atomic::Ordering::Relaxed) as f32 / total as f32
                            * 100.0,
                    )
                        .expect("emit `step2:set-progress` failed.");
                    rooms.insert(live, (room, video_path));
                } else {
                    done.fetch_add(2, std::sync::atomic::Ordering::Relaxed);
                    debug!("emit `step2:set-progress`.");
                    app.unwrap_emit(
                        "step2:set-progress",
                        done.load(std::sync::atomic::Ordering::Relaxed) as f32 / total as f32
                            * 100.0,
                    )
                        .expect("emit `step2:set-progress` failed.");
                }
            }
        }
        // pb.finish_with_message("已获取直播地址。");
        let mut results = HashMap::new();
        for (session, live) in lives_map {
            if let Some((room, video_path)) = rooms.get(&live.get_id()) {
                results.insert(
                    session.get_uid(),
                    (session.get_stu_name(), room.clone(), video_path.clone()),
                );
            }
        }
        if PROG_STATE.load(std::sync::atomic::Ordering::Relaxed) {
            app.unwrap_emit("step2:set-progress", 100.0)
                .expect("emit `step2:set-progress` failed.");
        }
        app.unwrap_emit("step2:stopped", 114514)
            .expect("emit `step2:started` failed.");
        results
    }
}
