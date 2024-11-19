use std::{
    collections::{HashMap, HashSet},
    sync::{atomic::AtomicBool, Arc, Mutex},
};

use cxlib::{
    default_impl::{sign::Sign, store::DataBase},
    types::Course,
    user::Session,
};

use crate::AccountPair;

pub struct DataBaseState(pub(crate) Arc<Mutex<DataBase>>);

#[derive(Default)]
pub struct SessionsState(pub(crate) Mutex<HashMap<String, Session>>);

pub(crate) static PROG_STATE: AtomicBool = AtomicBool::new(false);

#[derive(Default)]
pub struct CoursesState(pub(crate) Mutex<HashMap<Course, Vec<AccountPair>>>);

#[derive(Default)]
pub struct CurrentSignState {
    pub(crate) sign: Arc<Mutex<Option<Sign>>>,
    pub(crate) sessions: Arc<Mutex<HashSet<Session>>>,
}

#[derive(Default)]
pub struct CurrentSignUidSetState(pub(crate) Arc<Mutex<HashSet<String>>>);
