// Learn more aboupub(crate)pub(crate)t Tauri commands at https://tauri.app/develop/calling-rust/

pub mod models;
pub mod capture;
pub mod database;
pub mod features;
pub mod api;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tauri::Manager;
mod config;
use config::AppConfig;
use api::commands::{start_collect, stop_collect};
use models::models::ThreadStop;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_config = AppConfig::new(&app.handle());

            app.manage(app_config);
            app.manage(ThreadStop{ running_collect: Arc::new(<AtomicBool>::new(false))});
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![start_collect, stop_collect])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
