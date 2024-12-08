use cxlib::{
    default_impl::store::{AccountData, AccountTable},
    error::LoginError,
    store::Dir,
    user::{DefaultLoginSolver, LoginSolverTrait, LoginSolverWrapper, Session},
};
use serde::{Deserialize, Serialize};

use crate::state::{DataBaseState, SessionsState};

#[tauri::command]
pub async fn has_accounts(db_state: tauri::State<'_, DataBaseState>) -> Result<bool, String> {
    let db = db_state.0.lock().unwrap();
    Ok(!AccountTable::get_accounts(&db).is_empty())
}

#[tauri::command]
pub async fn get_config_dir() -> Result<String, String> {
    Ok(Dir::get_config_dir().to_str().unwrap_or("").to_owned())
}

#[tauri::command]
pub async fn add_account(
    uname: String,
    pwd: String,
    db_state: tauri::State<'_, DataBaseState>,
    sessions_state: tauri::State<'_, SessionsState>,
) -> Result<(), String> {
    let uname_and_login_type = uname.split_once(':');
    let (login_type, uname) = if let Some(v) = uname_and_login_type {
        v
    } else {
        (DefaultLoginSolver.login_type(), uname.as_str())
    };
    let solver = LoginSolverWrapper::new(login_type);
    let db = db_state.0.lock().unwrap();
    let enc_pwd = solver.pwd_enc(pwd).map_err(|e| e.to_string())?;
    let session = Session::relogin(uname, &enc_pwd, &solver).map_err(|e: LoginError| {
        eprint!("添加账号错误！");
        e.to_string()
    })?;
    AccountTable::add_account_or(
        &db,
        &AccountData::new(
            session.get_uid().to_owned(),
            uname.to_owned(),
            enc_pwd,
            login_type.to_owned(),
        ),
        AccountTable::update_account,
    );
    sessions_state
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .insert(session.get_uid().to_owned(), session);
    Ok(())
}

#[tauri::command]
pub async fn refresh_accounts(
    uid_vec: Vec<String>,
    db_state: tauri::State<'_, DataBaseState>,
    sessions_state: tauri::State<'_, SessionsState>,
) -> Result<(), String> {
    let db = db_state.0.lock().unwrap();
    for uid in uid_vec {
        if let Some(account) = AccountTable::get_account(&db, &uid) {
            AccountTable::delete_account(&db, &uid);
            sessions_state
                .0
                .lock()
                .map_err(|e| e.to_string())?
                .remove(&uid);
            if let Ok(session) = Session::relogin(
                account.uname(),
                account.enc_pwd(),
                &LoginSolverWrapper::new(account.login_type()),
            ) {
                AccountTable::add_account_or(&db, &account, AccountTable::update_account);
                sessions_state
                    .0
                    .lock()
                    .map_err(|e| e.to_string())?
                    .insert(uid, session);
            } else {
                eprint!("添加账号错误！");
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn delete_accounts(
    uid_vec: Vec<String>,
    db_state: tauri::State<'_, DataBaseState>,
    sessions_state: tauri::State<'_, SessionsState>,
) -> Result<(), String> {
    let db = db_state.0.lock().unwrap();
    for uid in uid_vec {
        AccountTable::delete_account(&db, &uid);
        sessions_state
            .0
            .lock()
            .map_err(|e| e.to_string())?
            .remove(&uid);
    }
    Ok(())
}

#[derive(Serialize, Clone, Deserialize, Hash)]
pub struct AccountPair {
    uid: String,
    name: String,
    avatar: String,
}

impl AccountPair {
    pub fn get_uid(&self) -> &str {
        &self.uid
    }
    // pub fn get_name(&self) -> &str {
    //     &self.name
    // }
    // pub fn get_avatar(&self) -> &str {
    //     &self.avatar
    // }
    pub fn new(uid: String, name: String, avatar: String) -> Self {
        Self { uid, name, avatar }
    }
    fn from_internal(session: &Session) -> Self {
        AccountPair::new(
            session.get_uid().to_string(),
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
    let vec = sessions.iter().map(|(uid, session)| AccountPair {
        uid: uid.clone(),
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
    let mut sessions = sessions_state.0.lock().unwrap();
    let load_sessions = AccountTable::get_sessions(&db);
    for (k, v) in load_sessions {
        sessions.insert(k, v);
    }
    Ok(())
}
