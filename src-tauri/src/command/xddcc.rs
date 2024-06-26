use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use log::{debug, info};
use serde::Serialize;
use tauri::State;

use crate::{room::Room, tools::VideoPath, SessionsState};
#[derive(Serialize)]
pub struct RoomPair {
    pub(crate) name: String,
    pub(crate) code: String,
}
#[derive(Serialize)]
pub struct LiveUrlPair {
    name: String,
    room: RoomPair,
    live: VideoPath,
}
#[tauri::command]
pub async fn list_rooms(
    unames: Vec<String>,
    sessions_state: State<'_, SessionsState>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<RoomPair>, String> {
    debug!("call_from_js: list_rooms");
    let sessions = sessions_state.0.lock().unwrap();
    let mut accounts = Vec::new();
    for uname in unames {
        if let Some(account) = sessions.get(&uname) {
            accounts.push(account);
        }
    }
    let map = Arc::new(Mutex::new(HashMap::new()));
    Room::get_all_live_id(&accounts, Arc::clone(&map), Some(app_handle.clone()));
    let rooms = Arc::new(Mutex::new(HashMap::new()));
    if let Some(session) = accounts.first() {
        debug!(
            "call_from_js: list_rooms/id_to_rooms: {}",
            session.get_stu_name()
        );
        Room::id_to_rooms(
            map.clone(),
            (*session).clone(),
            rooms.clone(),
            Some(app_handle.clone()),
        );
    }
    let rooms =
        crate::tools::map_sort_by_key((Arc::into_inner(rooms).unwrap()).into_inner().unwrap())
            .into_iter();
    let vec = rooms.map(|(name, code)| RoomPair { name, code }).collect();
    Ok(vec)
}
#[tauri::command]
pub async fn code_to_video_path(
    code: String,
    sessions_state: State<'_, SessionsState>,
) -> Result<VideoPath, String> {
    let sessions = sessions_state.0.lock().unwrap();
    if sessions.len() == 0 {
        return Err("请至少登录一个账号！".to_string());
    }
    if let Some(session) = (*sessions).values().next() {
        return Ok(crate::tools::get_live_video_path(session, &code));
    }
    Err("未能成功获取！".into())
}
pub fn get_lives_now(
    unames: &Vec<String>,
    sessions_state: &State<'_, SessionsState>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<LiveUrlPair>, String> {
    let sessions = sessions_state.0.lock().unwrap();
    let mut accounts = Vec::new();
    for uname in unames {
        if let Some(account) = sessions.get(uname) {
            accounts.push(account);
        }
    }
    let urls = crate::live::Live::get_lives_now(accounts.into_iter(), app_handle)
        .into_iter()
        .map(|(_, (stu_name, room, url))| {
            info!("获取当前：{}", stu_name);
            LiveUrlPair {
                name: stu_name.to_owned(),
                room: room.into_room_pair(),
                live: url,
            }
        })
        .collect();
    Ok(urls)
}
#[tauri::command]
pub async fn get_video_pathes_now(
    unames: Vec<String>,
    sessions_state: State<'_, SessionsState>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<LiveUrlPair>, String> {
    get_lives_now(&unames, &sessions_state, app_handle)
}
