use cxlib::types::Course;
use serde::Serialize;

use crate::{AccountPair, CoursesState, SessionsState};
#[derive(Serialize)]
pub struct CoursePair {
    course: Course,
    account_pairs: Vec<AccountPair>,
}
// TODO:
// #[derive(Serialize)]
// pub struct CoursePairs {
//     ok: Vec<CoursePair>,
//     excluded: Vec<CoursePair>,
// }
#[tauri::command]
pub async fn load_courses(
    sessions_state: tauri::State<'_, SessionsState>,
    courses_state: tauri::State<'_, CoursesState>,
) -> Result<(), String> {
    let sessions = sessions_state.0.lock().unwrap();
    let mut courses = courses_state.0.lock().unwrap();
    courses.clear();
    let courses_ = Course::get_courses(sessions.values()).map_err(|e| e.to_string())?;
    for (course, session_vec) in courses_ {
        courses.insert(
            course,
            session_vec.into_iter().map(AccountPair::from).collect(),
        );
    }
    Ok(())
}
#[tauri::command]
pub async fn list_courses(
    courses_state: tauri::State<'_, CoursesState>,
) -> Result<Vec<CoursePair>, String> {
    let courses = &courses_state.0;
    let mut course_pairs = Vec::<CoursePair>::new();
    for (course, account_pairs) in courses.lock().unwrap().iter() {
        course_pairs.push(CoursePair {
            course: course.clone(),
            account_pairs: account_pairs.clone(),
        })
    }
    Ok(course_pairs)
}
