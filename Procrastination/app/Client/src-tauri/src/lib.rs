// Learn more aboupub(crate)pub(crate)t Tauri commands at https://tauri.app/develop/calling-rust/

pub mod models;
pub mod capture;
pub mod database;
pub mod features;
pub mod api;
pub mod ml;
pub mod intervention;

use std::path::Path;
use ml::inference::load_model;

use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use tauri::Manager;
mod config;
use config::AppConfig;
use api::commands::{start_collect, stop_collect, intervention_update, break_start, break_end, preference_exist, get_preference, save_user_activity, update_label_streak};
use models::models::{ThreadStop, ModelState, OnBreak};
use crate::database::sqlite::initialize_database;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_config = AppConfig::new(&app.handle());

            let resource_path = app.path().resource_dir().expect("Failed to get resource path").join("resources/baseline_model.onnx");

            initialize_database(&*app_config.paths.database_path).expect("TODO: panic message");


            app.manage(ModelState {session: Arc::new(Mutex::new(load_model(resource_path)?))});
            app.manage(app_config);
            app.manage(ThreadStop{ running_collect: Arc::new(<AtomicBool>::new(false)), handles: Mutex::new(None)});
            app.manage(OnBreak{ on_break: Arc::new(<AtomicBool>::new(false)), break_ended: Arc::new(AtomicBool::new(false)), break_id: Arc::new(Mutex::new(None)) });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![start_collect, stop_collect, intervention_update, break_start, break_end, preference_exist, get_preference, save_user_activity, update_label_streak])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
