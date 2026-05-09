pub mod models;
pub mod capture;
pub mod database;
pub mod features;
pub mod api;
pub mod ml;
pub mod intervention;


use ml::inference::load_model;

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Manager;
mod config;
use config::AppConfig;
use api::commands::{start_collect, stop_collect, intervention_update, break_start, break_end, extend_break, open_break_window, close_break_window, get_break_init_data, preference_exist, get_preference, save_user_activity, update_label_streak, check_retraining_needed, trigger_retraining, get_saved_activities, get_analytics_stats, get_analytics_focus_score, get_history, get_activity_scores, get_recent_predictions, get_total_predictions_today, delete_activity, get_streak_settings, save_streak_settings, trigger_manual_intervention};
use models::models::{ThreadStop, ModelState, OnBreak, BreakInitData};
use crate::database::sqlite::initialize_database;

pub struct PendingBreakData {
    pub data: Mutex<Option<BreakInitData>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_config = AppConfig::new(&app.handle());
            app.manage(app_config.clone());

            let model_path = if app_config.paths.model_path.exists() {
                println!("Loading retrained model from: {:?}", app_config.paths.model_path);
                app_config.paths.model_path.clone()
            } else {
                println!("No retrained model found, loading bundled baseline...");
                app.path().resource_dir()
                    .expect("Failed to get resource path")
                    .join("resources/baseline_model.onnx")
            };

            initialize_database(&*app_config.paths.database_path).map_err(|e| {
                    eprintln!("Database initialization failed: {}", e);
                    e
            })?;


            app.manage(ModelState {session: Arc::new(Mutex::new(load_model(model_path)?))});

            app.manage(ThreadStop{ running_collect: Arc::new(<AtomicBool>::new(false)), handles: Mutex::new(None)});
            app.manage(OnBreak{ on_break: Arc::new(<AtomicBool>::new(false)), break_ended: Arc::new(AtomicBool::new(false)), break_id: Arc::new(Mutex::new(None)) });
            app.manage(PendingBreakData {
                data: Mutex::new(None),
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![start_collect, stop_collect, intervention_update, break_start, break_end, extend_break, open_break_window, close_break_window, get_break_init_data, preference_exist, get_preference, save_user_activity, update_label_streak, check_retraining_needed, trigger_retraining, get_saved_activities, get_analytics_stats, get_analytics_focus_score, get_history, get_activity_scores, get_recent_predictions, get_total_predictions_today, delete_activity, get_streak_settings, save_streak_settings, trigger_manual_intervention])
        .on_window_event(|window, event| {
            if !matches!(
                event,
                tauri::WindowEvent::CloseRequested { .. } | tauri::WindowEvent::Destroyed
            ) {
                return;
            }

            let app = window.app_handle();

            if window.label() == "main" {
                let thread_state = app.state::<ThreadStop>();
                thread_state.running_collect.store(false, Ordering::SeqCst);

                let break_state = app.state::<OnBreak>();
                break_state.on_break.store(false, Ordering::SeqCst);
                break_state.break_ended.store(false, Ordering::SeqCst);
                if let Ok(mut break_id) = break_state.break_id.lock() {
                    *break_id = None;
                }

                if let Some(w) = app.get_webview_window("popup") {
                    let _ = w.close();
                }
                if let Some(w) = app.get_webview_window("break") {
                    let _ = w.close();
                }
                app.exit(0);
                return;
            }

            if app.get_webview_window("main").is_none() {
                if let Some(w) = app.get_webview_window("popup") {
                    let _ = w.close();
                }
                if let Some(w) = app.get_webview_window("break") {
                    let _ = w.close();
                }
                app.exit(0);
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
