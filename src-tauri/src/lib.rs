pub mod ai;
pub mod commands;
pub mod db;

use rusqlite::Connection;
use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    pub db: Mutex<Option<Connection>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 1. Initialize Stronghold with secure Argon2 hashing
            let local_data_dir = app
                .path()
                .app_local_data_dir()
                .expect("could not resolve app local data path");
            
            std::fs::create_dir_all(&local_data_dir).expect("Failed to create local data dir");
            
            let salt_path = local_data_dir.join("salt.txt");

            app.handle()
                .plugin(tauri_plugin_stronghold::Builder::with_argon2(&salt_path).build())
                .expect("Failed to initialize Stronghold");

            // 2. Initialize SQLite Database
            let conn = db::init_db(app.handle()).expect("Failed to init DB");
            app.manage(AppState {
                db: Mutex::new(Some(conn)),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::save_model_pref,
            commands::settings::get_model_pref,
            commands::jobs::parse_and_save_job,
            commands::jobs::tailor_resume,
            commands::resumes::get_all_resumes,
            commands::resumes::get_resume_by_id,
            commands::resumes::create_new_resume,
            commands::resumes::update_resume,
            commands::resumes::delete_resume
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
