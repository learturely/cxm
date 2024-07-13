#![feature(let_chains)]
#![allow(incomplete_features)]
#![feature(specialization)]
#![feature(iter_next_chunk)]
#![feature(iter_array_chunks)]
#![feature(map_try_insert)]

mod command;
mod live;
mod protocol;
mod room;
mod location_info_getter;
mod signner;
mod state;
mod tools;

use cxsign::default_impl::store::{
    AccountTable, AliasTable, DataBase, ExcludeTable, LocationTable,
};
use log::{debug, error, info, trace};
use state::*;
use tauri::Listener;
use std::sync::Arc;
use std::sync::Mutex;
use cxsign::types::Location;
use tauri::Manager;

use command::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    Location::set_boxed_location_preprocessor(Box::new(xdsign_data::LocationPreprocessor))
        .unwrap_or_else(|e| error!("{e}"));
    #[cfg(mobile)]
        let default_builder = tauri::Builder::default().plugin(tauri_plugin_barcode_scanner::init());
    #[cfg(not(mobile))]
        let default_builder = tauri::Builder::default();
    // #[cfg(target_os = "android")]
    // let default_builder = default_builder.plugin(file_picker_android::init());
    // #[cfg(not(target_os = "android"))]
    let default_builder = default_builder.plugin(tauri_plugin_dialog::init());
    default_builder
        .setup(|app| {
            #[cfg(mobile)]
            cxsign::dir::Dir::set_config_dir(Box::new(
                app.path()
                    .resolve("", tauri::path::BaseDirectory::AppLocalData)?
                    .into(),
            ));
            #[cfg(not(mobile))]
            cxsign::dir::Dir::set_config_dir_info("TEST_CXSIGN", "up.workso", "Worksoup", "csm");
            let db = DataBase::new();
            db.add_table::<AccountTable>();
            db.add_table::<ExcludeTable>();
            db.add_table::<AliasTable>();
            db.add_table::<LocationTable>();
            app.manage(CoursesState::default());
            app.manage(SessionsState::default());
            app.manage(DataBaseState(Arc::new(Mutex::new(db))));
            app.manage(CurrentSignState::default());
            app.manage(CurrentSignUnamesState::default());
            debug!("程序加载。");
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                debug!("开始监听。");
                app_handle.listen("list-rooms:next-step", move |_| {
                    trace!("list-rooms:next-step: set_prog_state_to_false.");
                    PROG_STATE.fetch_and(false, std::sync::atomic::Ordering::Relaxed);
                    info!("操作已取消，准备下一步。");
                });
            });
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            has_accounts,
            add_account,
            refresh_accounts,
            delete_accounts,
            get_config_dir,
            list_accounts,
            load_accounts,
            list_rooms,
            code_to_video_path,
            get_video_pathes_now,
            load_courses,
            list_courses,
            list_course_activities,
            list_all_activities,
            prepare_sign,
            get_sign_type,
            sign_single,
            scan_image,
            remove_uname,
            add_uname,
            has_uname,
            add_unames,
            clear_unames,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        for i in 0..360 {
            let (y, t, w) = crate::tools::date_count_to_year_term_week(2024, i);
            println!("{y}, {t}, {w}");
        }
    }
}
