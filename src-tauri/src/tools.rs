use chrono::{Local, Timelike};
use log::debug;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash};
use cxsign::user::Session;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Default, Debug, Clone)]
pub struct VideoPath {
    ppt_video: Option<String>,
    teacher_full: Option<String>,
    teacher_track: Option<String>,
    student_full: Option<String>,
}

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
            out(&payload, None);
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

#[derive(Serialize, Default, Debug, Clone)]
struct WebUrl {
    url: String,
}

fn web_url_to_video_path(url: &WebUrl) -> VideoPath {
    let url = url.url.split("?info=").collect::<Vec<_>>()[1];
    let url = percent_encoding::percent_decode_str(url)
        .decode_utf8()
        .unwrap()
        .to_string();
    #[derive(Deserialize)]
    struct VideoPathInternal {
        #[serde(rename = "pptVideo")]
        ppt_video: Option<String>,
        #[serde(rename = "teacherFull")]
        teacher_full: Option<String>,
        #[serde(rename = "teacherTrack")]
        teacher_track: Option<String>,
        #[serde(rename = "studentFull")]
        student_full: Option<String>,
    }
    #[derive(Deserialize)]
    struct Info {
        #[serde(rename = "videoPath")]
        video_path: VideoPathInternal,
    }
    let Info {
        video_path:
        VideoPathInternal {
            ppt_video,
            teacher_full,
            teacher_track,
            student_full,
        },
    } = ureq::serde_json::from_str(&url).unwrap();
    VideoPath {
        ppt_video,
        teacher_full,
        teacher_track,
        student_full,
    }
}

fn get_live_web_url(session: &Session, device_code: &str) -> WebUrl {
    debug!("get_live_url, device_code: {device_code}");
    let url = crate::protocol::get_live_url(session, device_code)
        .unwrap()
        .into_string()
        .unwrap();
    WebUrl { url }
}

pub fn get_live_video_path(session: &Session, device_code: &str) -> VideoPath {
    let url = get_live_web_url(session, device_code);
    web_url_to_video_path(&url)
}

pub fn year_to_semester_id(year: i32, term: i32) -> i32 {
    let mut r = 2 * year - 4035 + term;
    if year == 2018 {
        r -= 1;
    } else if r < 1 {
        r = 1;
    }
    r
}

pub fn date_count_to_year_term_week(now_year: i32, date_count: i32) -> (i32, i32, i64) {
    (
        now_year - 6 + (date_count / 30) % 2 + date_count / 60,
        2 - (date_count / 30) % 2,
        date_count as i64 % 30 + 1,
    )
}

// pub fn out<S: Serialize>(contents: &S, path: Option<PathBuf>) {
//     let contents = toml::to_string_pretty(contents).unwrap();
//     if let Some(path) = path {
//         std::fs::write(path, contents).expect("写入内容出错！");
//     } else {
//         debug!("{contents}")
//     }
// }
pub fn now_to_jie() -> i32 {
    let date_time = Local::now();
    let s1 = Local::now().with_hour(10).unwrap().with_minute(5).unwrap();
    let s3 = Local::now().with_hour(12).unwrap().with_minute(0).unwrap();
    let s5 = Local::now().with_hour(15).unwrap().with_minute(35).unwrap();
    let s7 = Local::now().with_hour(17).unwrap().with_minute(30).unwrap();
    if date_time < s1 {
        1
    } else if date_time >= s1 && date_time < s3 {
        3
    } else if date_time >= s3 && date_time < s5 {
        5
    } else if date_time >= s5 && date_time < s7 {
        7
    } else {
        9
    }
}

pub fn map_sort_by_key<K: Ord + Hash, V>(map: HashMap<K, V>) -> Vec<(K, V)> {
    let mut map = map.into_iter().collect::<Vec<_>>();
    map.sort_by(|x, y| x.0.cmp(&y.0));
    map.into_iter().collect()
}

pub fn term_year_detial(session: &Session) -> (i32, i32, i64) {
    let data_time = chrono::DateTime::<chrono::Local>::from(std::time::SystemTime::now());
    let year = chrono::Datelike::year(&data_time);
    let semester_id = year_to_semester_id(year - 1, 2);

    #[derive(Deserialize)]
    struct WeekDetail {
        date1: String,
    }
    let WeekDetail { date1, .. } = crate::protocol::get_week_detail(session, 1, semester_id)
        .unwrap()
        .into_json()
        .unwrap();
    let date = date1.split('-').map(|s| s.trim()).collect::<Vec<_>>();
    let month = date[0].parse::<u32>().unwrap();
    let day = date[1].parse::<u32>().unwrap();
    let term_begin_data_time = <chrono::DateTime<chrono::Local> as std::str::FromStr>::from_str(
        &format!("{year}-{month}-{day}T00:00:00.0+08:00"),
    )
        .unwrap();
    let week = data_time
        .signed_duration_since(term_begin_data_time)
        .num_weeks()
        + 1;
    let (term_year, term) = if chrono::Datelike::month(&data_time) * 100
        + chrono::Datelike::day(&data_time)
        > month * 100 + day
        && chrono::Datelike::month(&data_time) * 100 + chrono::Datelike::day(&data_time) < 700
    {
        (year - 1, 2)
    } else {
        (year, 1)
    };
    (term_year, term, week)
}

pub struct PairVec<K, V> {
    vec: Vec<(K, V)>,
}

impl<K, V> PairVec<K, V> {
    pub fn new(vec: Vec<(K, V)>) -> Self {
        Self { vec }
    }
}
impl<K: Serialize, V: Serialize> Serialize for PairVec<K, V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.vec.len()))?;
        for (k, v) in &self.vec {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

pub fn out<S: Serialize>(contents: &S, path: Option<std::path::PathBuf>) {
    let contents = serde_json::to_string_pretty(contents).unwrap();
    if let Some(path) = path {
        std::fs::write(path, contents).expect("写入内容出错！");
    } else {
        println!("{contents}")
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
