use cxlib::{
    activity::{Activity, RawSign},
    default_impl::{
        sign::Sign,
        signner::{
            DefaultGestureOrSigncodeSignner, DefaultLocationInfoGetter, DefaultLocationSignner,
            DefaultNormalOrRawSignner, DefaultPhotoSignner,
        },
        store::DataBase,
    },
    sign::{SignResult, SignTrait},
    signner::SignnerTrait,
    types::Course,
    user::Session,
};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::Arc,
};
use tauri::{Emitter, Listener};
use tauri_plugin_dialog::DialogExt;

use crate::{
    location_info_getter::TauriLocationInfoGetter, signner::TauriQrCodeSignner, AccountPair,
    CurrentSignState, CurrentSignUidSetState, DataBaseState, SessionsState,
};

#[derive(Serialize)]
pub struct RawSignPair {
    sign: RawSign,
    account_pairs: Vec<AccountPair>,
}

#[derive(Deserialize)]
pub struct LocationSignnerInfo {
    pub location_str: Option<String>,
}

impl FromStr for LocationSignnerInfo {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[tauri::command]
pub async fn list_course_activities(
    course: Course,
    db_state: tauri::State<'_, DataBaseState>,
    sessions_state: tauri::State<'_, SessionsState>,
) -> Result<Vec<RawSign>, String> {
    let db = db_state.0.lock().unwrap();
    let sessions = sessions_state.0.lock().unwrap();
    let r = if let Some((_uid, session)) = sessions.iter().next() {
        let v =
            Activity::get_course_activities(&*db, session, &course).map_err(|e| e.to_string())?;
        v.into_iter()
            .filter_map(|sign| match sign {
                Activity::RawSign(sign) => {
                    if sign.is_valid() {
                        Some(sign)
                    } else {
                        None
                    }
                }
                Activity::Other(_) => None,
            })
            .collect()
    } else {
        vec![]
    };
    Ok(r)
}

#[tauri::command]
pub async fn list_all_activities(
    db_state: tauri::State<'_, DataBaseState>,
    sessions_state: tauri::State<'_, SessionsState>,
) -> Result<Vec<RawSignPair>, String> {
    let db = db_state.0.lock().unwrap();
    let sessions = sessions_state.0.lock().unwrap();
    let r =
        Activity::get_all_activities(&*db, sessions.values(), false).map_err(|e| e.to_string())?;
    Ok(r.into_iter()
        .filter_map(|(sign, sessions)| match sign {
            Activity::RawSign(sign) => {
                if sign.is_valid() {
                    Some(RawSignPair {
                        sign,
                        account_pairs: sessions.into_iter().map(AccountPair::from).collect(),
                    })
                } else {
                    None
                }
            }
            Activity::Other(_) => None,
        })
        .collect())
}

#[tauri::command]
pub async fn prepare_sign(
    sign: RawSign,
    accounts: Vec<AccountPair>,
    sessions_state: tauri::State<'_, SessionsState>,
    sign_state: tauri::State<'_, CurrentSignState>,
) -> Result<(), String> {
    let sessions_ = sessions_state.0.lock().unwrap();
    let mut sessions = HashSet::new();
    for account in accounts {
        if let Some(session) = sessions_.get(account.get_uid()) {
            sessions.insert(session.clone());
        }
    }
    if let Some(session) = sessions.iter().next() {
        let sign = Sign::from_raw(sign, session);
        *sign_state.sign.lock().unwrap() = Some(sign);
        *sign_state.sessions.lock().unwrap() = sessions;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_sign_type(
    sign_state: tauri::State<'_, CurrentSignState>,
) -> Result<String, String> {
    let t = sign_state
        .sign
        .lock()
        .unwrap()
        .as_ref()
        .map(|s| match s {
            Sign::Photo(_) => "photo",
            Sign::Normal(_) => "normal",
            Sign::QrCode(_) => "qrcode",
            Sign::Gesture(_) => "gesture",
            Sign::Location(_) => "location",
            Sign::Signcode(_) => "signcode",
            Sign::Unknown(_) => "unknown",
        })
        .unwrap_or("unknown")
        .to_string();
    Ok(t)
}

fn handle_results(results: HashMap<&Session, SignResult>, app_handle: &tauri::AppHandle) {
    for (session, result) in results {
        match result {
            SignResult::Susses => {
                info!("签到成功：{}", session.get_stu_name());
                app_handle.emit("sign:susses", session.get_uid()).unwrap();
            }
            SignResult::Fail { msg } => {
                info!("签到失败：{}", session.get_stu_name());
                app_handle
                    .emit("sign:fail", [session.get_uid(), &msg])
                    .unwrap();
            }
        }
    }
}

#[tauri::command]
pub async fn sign_single(
    db_state: tauri::State<'_, DataBaseState>,
    sign_state: tauri::State<'_, CurrentSignState>,
    uid_set_state: tauri::State<'_, CurrentSignUidSetState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let sign = Arc::clone(&sign_state.sign);
    let sign = sign.lock().unwrap().clone();
    if let Some(sign) = sign {
        let db = Arc::clone(&db_state.0);
        let sessions = Arc::clone(&sign_state.sessions);
        let sign_name = sign.as_inner().name.clone();
        let app_handle_ = app_handle.clone();
        let uid_set = Arc::clone(&uid_set_state.0);
        match sign {
            Sign::Photo(sign) => {
                info!("签到[{sign_name}]为拍照签到。");
                app_handle.listen("sign:photo", move |p| {
                    if p.payload() == "\"quit\"" {
                        log::info!("quit");
                        app_handle_.unlisten(p.id());
                        return;
                    }
                    let sign = sign.clone();
                    let app = app_handle_.clone();
                    let sessions = Arc::clone(&sessions);
                    let uid_set_ = Arc::clone(&uid_set);
                    app_handle_
                        .dialog()
                        .file()
                        .add_filter("选取图片", &["png", "jpeg"])
                        .pick_file(move |file_response| {
                            let mut sign = sign.clone();
                            let sign = &mut sign;
                            let path = match file_response {
                                Some(fp) => match fp {
                                    tauri_plugin_dialog::FilePath::Url(_url) => None,
                                    tauri_plugin_dialog::FilePath::Path(path) => Some(path),
                                },
                                None => None,
                            };
                            let uid_set__ = uid_set_.lock().unwrap();
                            let sessions = sessions.lock().unwrap();
                            let results = DefaultPhotoSignner::new(&path).sign(
                                sign,
                                sessions.iter().filter(|a| uid_set__.contains(a.get_uid())),
                            );
                            if let Ok(results) = results {
                                handle_results(results, &app)
                            }
                        });
                });
            }
            Sign::Normal(sign) => {
                info!("签到[{sign_name}]为普通签到。");
                app_handle.listen("sign:normal", move |p| {
                    if p.payload() == "\"quit\"" {
                        log::info!("quit");
                        app_handle_.unlisten(p.id());
                        return;
                    }
                    let mut sign = sign.clone();
                    let sign = &mut sign;
                    let uid_set_ = uid_set.lock().unwrap();
                    let sessions = sessions.lock().unwrap();
                    if let Ok(results) = DefaultNormalOrRawSignner.sign(
                        sign,
                        sessions.iter().filter(|a| uid_set_.contains(a.get_uid())),
                    ) {
                        handle_results(results, &app_handle_)
                    }
                });
            }
            Sign::QrCode(sign) => {
                info!("签到[{sign_name}]为二维码签到。");
                let mut sign = sign.clone();
                let sign = &mut sign;
                let _ = TauriQrCodeSignner::<TauriLocationInfoGetter, DataBase>::new(
                    Arc::clone(&db),
                    app_handle_.clone(),
                )
                .sign(sign, None.iter());
            }
            Sign::Gesture(sign) => {
                info!("签到[{sign_name}]为手势签到。");
                app_handle.listen("sign:gesture", move |p| {
                    if p.payload() == "\"quit\"" {
                        log::info!("quit");
                        app_handle_.unlisten(p.id());
                        return;
                    }
                    let mut sign = sign.clone();
                    let sign = &mut sign;
                    let uid_set_ = uid_set.lock().unwrap();
                    let sessions = sessions.lock().unwrap();
                    if let Ok(results) =
                        DefaultGestureOrSigncodeSignner::new(p.payload().trim_matches(|c| c == '"'))
                            .sign(
                                sign,
                                sessions.iter().filter(|a| uid_set_.contains(a.get_uid())),
                            )
                    {
                        handle_results(results, &app_handle_)
                    }
                });
            }
            Sign::Location(sign) => {
                info!("签到[{sign_name}]为位置签到。");
                // sign_results = DefaultLocationSignner::new(db, 位置字符串, *是否禁用随机偏移)
                //     .sign(sign, sessions)?;
                app_handle.listen("sign:location", move |p| {
                    if p.payload() == "\"quit\"" {
                        log::info!("quit");
                        app_handle_.unlisten(p.id());
                        return;
                    }
                    let mut sign = sign.clone();
                    let sign = &mut sign;
                    let LocationSignnerInfo { location_str } = p.payload().parse().unwrap();
                    let uid_set_ = uid_set.lock().unwrap();
                    let sessions = sessions.lock().unwrap();
                    if let Ok(results) = DefaultLocationSignner::new(
                        DefaultLocationInfoGetter::from(&*db.lock().unwrap()),
                        &location_str,
                    )
                    .sign(
                        sign,
                        sessions.iter().filter(|a| uid_set_.contains(a.get_uid())),
                    ) {
                        handle_results(results, &app_handle_)
                    }
                });
            }
            Sign::Signcode(sign) => {
                info!("签到[{sign_name}]为签到码签到。");
                app_handle.listen("sign:signcode", move |p| {
                    if p.payload() == "\"quit\"" {
                        log::info!("quit");
                        app_handle_.unlisten(p.id());
                        return;
                    }
                    let mut sign = sign.clone();
                    let sign = &mut sign;
                    let uid_set_ = uid_set.lock().unwrap();
                    let sessions = sessions.lock().unwrap();
                    if let Ok(results) =
                        DefaultGestureOrSigncodeSignner::new(p.payload().trim_matches(|c| c == '"'))
                            .sign(
                                sign,
                                sessions.iter().filter(|a| uid_set_.contains(a.get_uid())),
                            )
                    {
                        handle_results(results, &app_handle_)
                    }
                });
            }
            Sign::Unknown(sign) => {
                warn!("签到[{}]为无效签到类型！", sign.name);
                app_handle.listen("sign:unknown", move |p| {
                    if p.payload() == "\"quit\"" {
                        log::info!("quit");
                        app_handle_.unlisten(p.id());
                        return;
                    }
                    let mut sign = sign.clone();
                    let sign = &mut sign;
                    let uid_set_ = uid_set.lock().unwrap();
                    let sessions = sessions.lock().unwrap();
                    if let Ok(results) = DefaultNormalOrRawSignner.sign(
                        sign,
                        sessions.iter().filter(|a| uid_set_.contains(a.get_uid())),
                    ) {
                        handle_results(results, &app_handle_)
                    }
                });
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn remove_uid(
    uid: String,
    state: tauri::State<'_, CurrentSignUidSetState>,
) -> Result<bool, String> {
    Ok(state.0.lock().unwrap().remove(&uid))
}

#[tauri::command]
pub async fn add_uid(
    uid: String,
    state: tauri::State<'_, CurrentSignUidSetState>,
) -> Result<bool, String> {
    Ok(state.0.lock().unwrap().insert(uid))
}

#[tauri::command]
pub async fn extent_uid_set(
    uid_vec: Vec<String>,
    state: tauri::State<'_, CurrentSignUidSetState>,
) -> Result<(), String> {
    info!("添加：{uid_vec:?}");
    state.0.lock().unwrap().extend(uid_vec);
    Ok(())
}

#[tauri::command]
pub async fn has_uid(
    uid: String,
    state: tauri::State<'_, CurrentSignUidSetState>,
) -> Result<bool, String> {
    Ok(state.0.lock().unwrap().contains(&uid))
}

#[tauri::command]
pub async fn clear_uid_set(state: tauri::State<'_, CurrentSignUidSetState>) -> Result<(), String> {
    state.0.lock().unwrap().clear();
    Ok(())
}
