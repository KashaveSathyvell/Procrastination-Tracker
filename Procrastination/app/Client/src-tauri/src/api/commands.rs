use std::sync::mpsc::{Sender, Receiver, TryRecvError};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::Thread;
use chrono::Utc;
use ndarray::AssignElem;
use ort::editor::Model;
use rusqlite::params;
use rusqlite::Connection;
use rusqlite::fallible_iterator::FallibleIterator;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

use crate::capture::keyboard::{logging};
use crate::database::sqlite::{update_user_label, get_ids, assign_truth_label, insert_break_sessions, has_preferences, insert_user_preference, update_break, update_n_windows_before, get_retraining_stats, clear_old_events, get_user_saved_activities, get_prediction_stats, get_focus_score, get_prediction_history, delete_user_preference, get_setting, save_setting, get_predictions_count_today, get_break_plan, extend_break_planned_duration, prediction_corrected_n_windows};
use crate::features::feature_extractor::run_extractor;
use crate::PendingBreakData;
use tauri::{Emitter, Manager, State};
use crate::config::AppConfig;
use crate::models::table_structs::{BreakSessions, Input};
use crate::models::models::{ThreadStop, ModelState, UpdateIntervention, OnBreak, EndBreak, IdleFocusedPackage, RetrainingStats, RetrainingResult, StateDistribution, FocusScore, PredictionHistoryRow, ActivityScore, StreakSettings, BreakInitData};
use crate::intervention::activity::break_activities;
use crate::ml::inference::load_model;

#[tauri::command]
pub fn start_collect(app_handle: AppHandle, state: State<ThreadStop>, model_state: State<ModelState>, config: State<AppConfig>, on_break: State<OnBreak>) -> Result<(), String> {

    let session_clone = model_state.session.clone();

    let is_running = state.running_collect.swap(true, Ordering::Relaxed);

    if is_running {
        return Err(String::from("Collecting has already started"))
    }

    let running_clone1 = state.running_collect.clone();
    let running_clone2 = state.running_collect.clone();
    let running_clone3 = state.running_collect.clone();
    let db_path1 = config.paths.database_path.clone();
    let db_path2 = config.paths.database_path.clone();
    let user_break1 = on_break.on_break.clone();
    let user_break2 = on_break.on_break.clone();
    let break_end = on_break.break_ended.clone();
    let break_id = on_break.break_id.clone();


    println!("Collecting data");

    let (tx, rx) = mpsc::channel::<Input>();

    let handle1 = thread::spawn(move || {
        logging(tx, running_clone1, user_break1);
    });



    let handle2 = thread::spawn(move || {
        const BATCH_SIZE: usize = 500;
        let mut conn = match Connection::open(&db_path1) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to open DB connection \n                     in writer thread: {}", e);
                return;
            }
        };
        if let Err(e) = conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 5000;") {
            eprintln!("Failed to set pragmas: {}", e);
        }
        let mut batch: Vec<Input> = Vec::with_capacity(BATCH_SIZE);
        while running_clone2.load(Ordering::Relaxed) {
            match rx.try_recv() {
                Ok(received) => {
                    batch.push(received);

                    while batch.len() < BATCH_SIZE {
                        match rx.try_recv() {
                            Ok(next) => batch.push(next),
                            Err(TryRecvError::Empty) => break,
                            Err(TryRecvError::Disconnected) => break,
                        }
                    }

                    let tx = match conn.transaction() {
                        Ok(t) => t,
                        Err(e) => {
                            eprintln!("Failed to start batch transaction: {}", e);
                            batch.clear();
                            continue;
                        }
                    };

                    let mut stmt = match tx.prepare(
                        "INSERT INTO input_events(timestamp, event_type, \
                         event_action, key_code, mouse_x, mouse_y, \
                         wheel_x, wheel_y, button, active_window) \
                         Values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
                    ) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("Failed to prepare batch insert statement: {}", e);
                            batch.clear();
                            continue;
                        }
                    };

                    for input in &batch {
                        if let Err(e) = stmt.execute(params![
                            input.timestamp,
                            input.event_type,
                            input.event_action,
                            input.key_code.as_deref(),
                            input.mouse_x,
                            input.mouse_y,
                            input.wheel_x,
                            input.wheel_y,
                            input.button.as_deref(),
                            input.active_window
                        ]) {
                            eprintln!("insert_events batch row failed: {}", e);
                        }
                    }

                    drop(stmt);
                    if let Err(e) = tx.commit() {
                        eprintln!("Failed to commit event batch: {}", e);
                    }
                    batch.clear();
                }
                Err(TryRecvError::Empty) => { thread::sleep(Duration::from_millis(10)) }
                _ => {}
            }
        }
       //  for received in rx {
       //      insert_events(Path::new("behavior.db"), &received);
       // }
    });



    let handle3 = thread::spawn(move || {
            run_extractor(&db_path2, &running_clone3, &session_clone, &app_handle, &user_break2, &break_end, &break_id);
    });

    {
        let mut handles = state.handles.lock().unwrap();
        *handles = Some(vec![handle1, handle2, handle3]);
    }

    Ok(())
}


#[tauri::command]
pub fn stop_collect(state: State<ThreadStop>, on_break: State<OnBreak>) -> Result<(), String> {
    state.running_collect.store(false, Ordering::Relaxed);
    on_break.on_break.store(false, Ordering::SeqCst);
    on_break.break_ended.store(false, Ordering::SeqCst);

    let mut break_id = on_break.break_id.lock().unwrap();
    *break_id = None;
    
    println!("Stopping threads");
    Ok(())
}


#[tauri::command]
pub fn get_recent_predictions(config: State<AppConfig>) -> Result<Vec<PredictionHistoryRow>, String> {
    crate::database::sqlite::get_recent_predictions(&config.paths.database_path, 10)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_total_predictions_today(config: State<AppConfig>) -> Result<i64, String> {
    get_predictions_count_today(&config.paths.database_path).map_err(|e| e.to_string())
}


#[tauri::command]
pub fn intervention_update(updated_intervention: UpdateIntervention, config: State<AppConfig>) -> Result<(), String>  {
    let db_path = &config.paths.database_path;
    let overwrite = updated_intervention.user_label != updated_intervention.predicted_label;

    let updated_truth_label = IdleFocusedPackage {
        timestamp: updated_intervention.timestamp,
        streak_windows: 3,
        label: updated_intervention.user_label.clone(),
        overwrite,
    };

    println!("Intervention update: {:?}", updated_intervention);
    update_user_label(db_path, &updated_intervention).expect("TODO: panic message");

    let (predictions_id, feature_vector_id) = match get_ids(db_path, updated_intervention.intervention_id) {
        Ok(result) => result,
        Err(err) => return Err(err.to_string())
    };

    if updated_intervention.dismissed == false {
        if (updated_intervention.user_label != updated_intervention.predicted_label) {
            if let Err(e) = prediction_corrected_n_windows(db_path, updated_intervention.timestamp, 3, true) {
                eprintln!("Failed to mark predictions as corrected: {}", e);
            }
        }
        update_n_windows_before(db_path, updated_truth_label).expect("TODO: panic message");
    }

    Ok(())
}


//ERROR: PopUp.tsx:84 start_break failed: invalid args `plannedDurationMins` for command `break_start`: command break_start missing required key plannedDurationMins
#[tauri::command]
pub fn break_start(intervention_id: i64, activity: String, planned_duration_mins: i64, preference_id: i64, config: State<AppConfig>, on_break: State<OnBreak>) -> Result<(i64), String> {

    let break_session = BreakSessions {
        intervention_id,
        start_time: Utc::now().timestamp(),
        end_time: 0,
        preference_id: Some(preference_id),
        activity,
        planned_duration_mins,
        returned_on_time: 0,
        post_break_focus_score: 0.0,
    };

    on_break.on_break.store(true, Ordering::SeqCst);

    println!("Break update: {:?}", break_session);

    let break_id = insert_break_sessions(&config.paths.database_path, &break_session).map_err(|e| e.to_string())?;

    Ok(break_id)
}

#[tauri::command]
pub fn break_end(end_break: EndBreak, on_break: State<OnBreak>, config: State<AppConfig>) -> Result<(), String> {
    on_break.on_break.store(false, Ordering::SeqCst);
    on_break.break_ended.store(true, Ordering::SeqCst);
    let end_time = Utc::now().timestamp();

    let (start_ts, planned_mins) = get_break_plan(&config.paths.database_path, end_break.break_session_id)
        .map_err(|e| e.to_string())?;
    let planned_end_ts = start_ts + (planned_mins * 60);
    let returned_on_time = end_time <= planned_end_ts;

    println!("Break ended: session {}, on time: {}", end_break.break_session_id, returned_on_time);
    let mut current_break_id = on_break.break_id.lock().unwrap();
    *current_break_id = Some(end_break.break_session_id);

    update_break(
        &config.paths.database_path,
        EndBreak { break_session_id: end_break.break_session_id, returned_on_time },
        end_time
    ).expect("TODO: panic message");

    Ok(())
}

#[tauri::command]
pub fn extend_break(break_session_id: i64, extra_minutes: i64, config: State<AppConfig>) -> Result<(), String> {
    if extra_minutes <= 0 {
        return Err("extra_minutes must be > 0".to_string());
    }
    extend_break_planned_duration(&config.paths.database_path, break_session_id, extra_minutes)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_break_window(
    app_handle: AppHandle,
    activity: String,
    duration: i64,
    break_session_id: i64,
    intervention_id: i64,
) -> Result<(), String> {
    use tauri::WebviewUrl;
    use tauri::WebviewWindowBuilder;

    let init_data = BreakInitData {
        activity,
        duration,
        break_session_id,
        intervention_id,
    };

    let pending_state = app_handle.state::<PendingBreakData>();
    {
        let mut slot = pending_state.data.lock().map_err(|e| e.to_string())?;
        *slot = Some(init_data.clone());
    }

    println!("Opening break window from configured app window");
    if let Some(window) = app_handle.get_webview_window("break") {
        let _ = window.unminimize();
        let _ = window.center();
        window.show().map_err(|e| e.to_string())?;
        let _ = window.set_focus();
        let _ = app_handle.emit_to("break", "break_init_data", init_data);
        return Ok(());
    }

    let break_url = tauri::WebviewUrl::App("break.html".into());
    let _window = WebviewWindowBuilder::new(&app_handle, "break", break_url)
        .title("Break Time")
        .inner_size(380.0, 280.0)
        .resizable(false)
        .center()
        .always_on_top(false)
        .skip_taskbar(false)
        .build()
        .map_err(|e| e.to_string())?;
    let _ = app_handle.emit_to("break", "break_init_data", init_data);

    Ok(())
}

#[tauri::command]
pub fn get_break_init_data(pending: State<PendingBreakData>) -> Result<Option<BreakInitData>, String> {
    let mut data = pending.data.lock().map_err(|e| e.to_string())?;
    Ok(data.take())
}

#[tauri::command]
pub fn close_break_window(app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("break") {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}


#[tauri::command]
pub fn trigger_manual_intervention(
    label: String,
    timestamp: i64,
    app_handle: AppHandle,
    config: State<AppConfig>,
) -> Result<(), String> {
    use crate::intervention::jitai::suggest_activity;
    use crate::database::sqlite::insert_interventions;
    use crate::models::table_structs::Interventions;
    use crate::models::models::InterventionPackage;

    // Create an intervention record in the DB
    let intervention = Interventions {
        predictions_id: 0, // no specific prediction triggered this
        timestamp,
        intervention_type: "ManualCorrection".to_string(),
        prediction_label: label.clone(),
        user_label: Some(label.clone()),
        dismissed: false,
    };

    let intervention_id = insert_interventions(&config.paths.database_path, &intervention)
        .map_err(|e| e.to_string())?;

    let activity_suggestion = suggest_activity(&config.paths.database_path);

    let payload = InterventionPackage {
        intervention_id,
        timestamp,
        intervention_type: "ManualCorrection".to_string(),
        prediction_label: label,
        confidence: 0.9, // user confirmed, so treat as high confidence
        suggested_activity: Some(activity_suggestion.activity),
        suggested_duration: Some(activity_suggestion.random_duration),
        preference_id: Some(activity_suggestion.preference_id),
    };

    // Show the popup window and emit the event — PopUp.tsx handles the rest
    if let Some(popup_window) = app_handle.get_webview_window("popup") {
        if let Some(monitor) = popup_window.current_monitor().unwrap_or(None) {
            let screen_size = monitor.size();
            let scale = monitor.scale_factor();
            let popup_w = (360.0 * scale) as i32;
            let popup_h = (480.0 * scale) as i32;
            let margin = (16.0 * scale) as i32;
            let x = (screen_size.width as i32) - popup_w - margin;
            let y = (screen_size.height as i32) - popup_h - margin;
            let _ = popup_window.set_position(tauri::PhysicalPosition::new(x, y));
        }
        let _ = popup_window.show();
        let _ = popup_window.set_focus();
    }

    app_handle.emit("new_intervention", payload).map_err(|e| e.to_string())?;

    Ok(())
}


#[tauri::command]
pub fn preference_exist(config: State<AppConfig>) -> Result<bool, String> {
    let preference_count = has_preferences(&*config.paths.database_path);

    let count = preference_count.map_err(|e| e.to_string())?;

    Ok(count > 0)
}

#[tauri::command]
pub fn get_preference() -> Result<Vec<String>, String> {
    let preference_list = break_activities().into_iter().map(|row| row.activity_name).collect::<Vec<String>>();

    Ok(preference_list)
}

#[tauri::command]
pub fn save_user_activity(config: State<AppConfig>, chosen_list: Vec<String>) -> Result<(), String> {
    let activities = break_activities();

    for chosen in &chosen_list {
        if let Some(activity) = activities.iter().find(|row| row.activity_name == *chosen) {
            insert_user_preference(&config.paths.database_path, activity).map_err(|err| err.to_string())?;
        };
    };

    Ok(())
}

#[tauri::command]
pub fn get_saved_activities(config: State<AppConfig>) -> Result<Vec<String>, String> {
    get_user_saved_activities(&config.paths.database_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_activity(activity_name: String, config: State<AppConfig>) -> Result<(), String> {
    delete_user_preference(&config.paths.database_path, activity_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_activity_scores(config: State<AppConfig>) -> Result<Vec<ActivityScore>, String> {
    crate::database::sqlite::get_activity_scores(&config.paths.database_path)
        .map_err(|e| e.to_string())
}


#[tauri::command]
pub fn update_label_streak(config: State<AppConfig>, state_confirmation: IdleFocusedPackage) -> Result<(), String> {
    if state_confirmation.overwrite {
        if let Err(e) = prediction_corrected_n_windows(
            &config.paths.database_path,
            state_confirmation.timestamp,
            state_confirmation.streak_windows as i64,
            true,
        ) {
            eprintln!("Failed to mark predictions corrected in streak: {}", e);
        }
    }

    update_n_windows_before(&config.paths.database_path, state_confirmation).map_err(|err| err.to_string())?;

    Ok(())
}


#[tauri::command]
pub fn get_streak_settings(config: State<AppConfig>) -> Result<StreakSettings, String> {
    let value = get_setting(&config.paths.database_path, "focused_streak_window")
        .map_err(|e| e.to_string())?;

    let focused_streak_window = value.and_then(|v| v.parse::<i64>().ok()).unwrap_or(15);

    Ok(StreakSettings { focused_streak_window })
}

#[tauri::command]
pub fn save_streak_settings(settings: StreakSettings, config: State<AppConfig>) -> Result<(), String> {
    save_setting(&config.paths.database_path, "focused_streak_window", &settings.focused_streak_window.to_string())
        .map_err(|e| e.to_string())
}


#[tauri::command]
pub fn check_retraining_needed(config: State<AppConfig>) -> Result<RetrainingStats, String> {
    let (correction_rate, labelled_count) = get_retraining_stats(&config.paths.database_path)
        .map_err(|e| e.to_string())?;

    let retraining_needed = correction_rate > 0.25 && labelled_count >= 50;

    println!(
        "Retraining check — correction rate: {:.2}%, labelled rows: {}, needed: {}",
        correction_rate * 100.0,
        labelled_count,
        retraining_needed
    );

    Ok(RetrainingStats {
        correction_rate,
        labelled_count,
        retraining_needed,
    })
}




#[tauri::command]
pub fn trigger_retraining(app_handle: AppHandle, config: State<AppConfig>, model_state: State<ModelState>) -> Result<RetrainingResult, String> {
    let db_path = config.paths.database_path.to_str().ok_or("Invalid database path")?.to_string();

    let model_output_path = config.paths.model_path.to_str().ok_or("Invalid model path")?.to_string();

    println!("Triggering retraining via sidecar...");
    println!("DB path: {}", db_path);
    println!("Model output: {}", model_output_path);

    // Spawn the sidecar — Tauri resolves the correct binary for the current platform
    let sidecar_command = app_handle.shell().sidecar("retrain")
        .map_err(|e| format!("Failed to find retrain sidecar: {}", e))?.args([&db_path, &model_output_path]);

    let output = tauri::async_runtime::block_on(sidecar_command.output())
        .map_err(|e| format!("Failed to run retrain sidecar: {}", e))?;


    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("Sidecar stdout:\n{}", stdout);
    if !stderr.is_empty() {
        println!("Sidecar stderr:\n{}", stderr);
    }

    if !output.status.success() {
        return Ok(RetrainingResult {
            success: false,
            message: format!("Retraining failed: {}", stderr),
        });
    }

    // Reload model
    println!("Reloading model from: {}", model_output_path);
    let new_session = load_model(std::path::PathBuf::from(&model_output_path))
        .map_err(|e| format!("Failed to load retrained model: {}", e))?;

    {
        let mut session_guard = model_state.session.lock()
            .map_err(|e| format!("Failed to lock model session: {}", e))?;
        *session_guard = new_session;
        println!("Model session replaced successfully");
    }

    clear_old_events(&config.paths.database_path)
        .map_err(|e| format!("Failed to clear old events: {}", e))?;

    Ok(RetrainingResult {
        success: true,
        message: "Model retrained successfully and loaded. Old input events cleared.".to_string(),
    })
}




//analyticsn  history
#[tauri::command]
pub fn get_analytics_stats(range_days: i64, config: State<AppConfig>) -> Result<StateDistribution, String> {
    let since = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64 - (range_days * 24 * 60 * 60);

    get_prediction_stats(&config.paths.database_path, since)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_analytics_focus_score(range_days: i64, config: State<AppConfig>) -> Result<FocusScore, String> {
    let since = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64 - (range_days * 24 * 60 * 60);

    get_focus_score(&config.paths.database_path, since)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_history(range_days: i64, state_filter: Option<String>, config: State<AppConfig>) -> Result<Vec<PredictionHistoryRow>, String> {
    let since = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64 - (range_days * 24 * 60 * 60);

    get_prediction_history(&config.paths.database_path, since, state_filter, 1000)
        .map_err(|e| e.to_string())
}