use cxsign::{
    store::{tables::AccountTable, DataBaseTableTrait},
    Session, UnameAndEncPwdPair,
};
use serde::{Deserialize, Serialize};

use crate::state::{DataBaseState, SessionsState};

#[tauri::command]
pub async fn has_accounts(db_state: tauri::State<'_, DataBaseState>) -> Result<bool, String> {
    let db = db_state.0.lock().unwrap();
    let table = AccountTable::from_ref(&db);
    Ok(!table.get_accounts().is_empty())
}

#[tauri::command]
pub async fn get_config_dir(
    dir_state: tauri::State<'_, cxsign::utils::Dir>,
) -> Result<String, String> {
    Ok(dir_state.get_config_dir().to_str().unwrap_or("").to_owned())
}

#[tauri::command]
pub async fn add_account(
    uname: String,
    pwd: String,
    db_state: tauri::State<'_, DataBaseState>,
    dir_state: tauri::State<'_, cxsign::utils::Dir>,
    sessions_state: tauri::State<'_, SessionsState>,
) -> Result<(), String> {
    let db = db_state.0.lock().unwrap();
    let enc_pwd = cxsign::utils::des_enc(&pwd);
    let session = Session::login(&dir_state, &uname, &enc_pwd).map_err(|e: cxsign::Error| {
        eprint!("添加账号错误！");
        match e {
            cxsign::Error::LoginError(e) => e,
            cxsign::Error::AgentError(e) => e.to_string(),
            _ => unreachable!(),
        }
    })?;
    let name = session.get_stu_name();
    let table = AccountTable::from_ref(&db);
    table.add_account_or(&uname, &enc_pwd, name, AccountTable::update_account);
    sessions_state
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .insert(uname, session);
    Ok(())
}

#[tauri::command]
pub async fn refresh_accounts(
    unames: Vec<String>,
    db_state: tauri::State<'_, DataBaseState>,
    dir_state: tauri::State<'_, cxsign::utils::Dir>,
    sessions_state: tauri::State<'_, SessionsState>,
) -> Result<(), String> {
    let db = db_state.0.lock().unwrap();
    let table = AccountTable::from_ref(&db);
    for uname in unames {
        if let Some((UnameAndEncPwdPair { uname, enc_pwd }, _)) = table.get_account(&uname) {
            table.delete_account(&uname);
            sessions_state
                .0
                .lock()
                .map_err(|e| e.to_string())?
                .remove(&uname);
            if let Ok(session) = Session::login(&dir_state, &uname, &enc_pwd) {
                let name = session.get_stu_name();
                table.add_account_or(&uname, &enc_pwd, name, AccountTable::update_account);
                sessions_state
                    .0
                    .lock()
                    .map_err(|e| e.to_string())?
                    .insert(uname, session);
            } else {
                eprint!("添加账号错误！");
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn delete_accounts(
    unames: Vec<String>,
    db_state: tauri::State<'_, DataBaseState>,
    sessions_state: tauri::State<'_, SessionsState>,
) -> Result<(), String> {
    let db = db_state.0.lock().unwrap();
    let table = AccountTable::from_ref(&db);
    for uname in unames {
        table.delete_account(&uname);
        sessions_state
            .0
            .lock()
            .map_err(|e| e.to_string())?
            .remove(&uname);
    }
    Ok(())
}

#[derive(Serialize, Clone, Deserialize, Hash)]
pub struct AccountPair {
    uname: String,
    name: String,
    avatar: String,
}

impl AccountPair {
    pub fn get_uname(&self) -> &str {
        &self.uname
    }
    // pub fn get_name(&self) -> &str {
    //     &self.name
    // }
    // pub fn get_avatar(&self) -> &str {
    //     &self.avatar
    // }
    pub fn new(uname: String, name: String, avatar: String) -> Self {
        Self {
            uname,
            name,
            avatar,
        }
    }
    fn from_internal(session: &Session) -> Self {
        AccountPair::new(
            session.get_uname().to_string(),
            session.get_stu_name().to_string(),
            session.get_avatar_url(128).to_string(),
        )
    }
}

impl From<&Session> for AccountPair {
    fn from(session: &Session) -> Self {
        AccountPair::from_internal(session)
    }
}

impl From<Session> for AccountPair {
    fn from(session: Session) -> Self {
        AccountPair::from_internal(&session)
    }
}

#[tauri::command]
pub async fn list_accounts(
    sessions_state: tauri::State<'_, SessionsState>,
) -> Result<Vec<AccountPair>, String> {
    let sessions = sessions_state.0.lock().unwrap();
    let vec = sessions.iter().map(|(uname, session)| AccountPair {
        uname: uname.clone(),
        name: session.get_stu_name().to_string(),
        avatar: session.get_avatar_url(128),
    });
    Ok(vec.collect())
}

#[tauri::command]
pub async fn load_accounts(
    db_state: tauri::State<'_, DataBaseState>,
    sessions_state: tauri::State<'_, SessionsState>,
) -> Result<(), String> {
    let db = db_state.0.lock().unwrap();
    let table = AccountTable::from_ref(&db);
    let mut sessions = sessions_state.0.lock().unwrap();
    let load_sessions = table.get_sessions();
    for (k, v) in load_sessions {
        sessions.insert(k, v);
    }
    Ok(())
}
