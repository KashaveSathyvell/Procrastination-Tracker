use std::sync::mpsc::{Sender, Receiver, TryRecvError};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::Thread;
use chrono::Utc;
use ndarray::AssignElem;
use ort::editor::Model;
use rusqlite::fallible_iterator::FallibleIterator;
use tauri::async_runtime::handle;
use tauri::AppHandle;

use crate::capture::keyboard::{callback, logging};
use crate::database::sqlite::{initialize_database, insert_events, update_user_label, get_ids, prediction_corrected, assign_truth_label, insert_break_sessions, has_preferences, insert_user_preference, update_break, update_n_windows_before};
use crate::features::feature_extractor::run_extractor;
use tauri::{Manager, State};
use tauri::WebviewUrl::App;
use crate::config::AppConfig;
use crate::models::table_structs::{BreakSessions, Input, UserPreferences};
use crate::models::models::{ThreadStop, ModelState, UpdateIntervention, OnBreak, EndBreak, IdleFocusedPackage};
use crate::intervention::activity::break_activities;


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
        while running_clone2.load(Ordering::Relaxed) {
            match rx.try_recv() {
                Ok(received) => { insert_events(&db_path1, &received); }
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
            prediction_corrected(db_path, predictions_id, true).expect("TODO: panic message");
        }
        update_n_windows_before(db_path, updated_truth_label).expect("TODO: panic message");
        // assign_truth_label(db_path, feature_vector_id, updated_intervention.user_label.clone()).expect("TODO: panic message");
    }
    
    // if updated_intervention.dismissed == false {
    //     assign_truth_label(db_path, feature_vector_id, updated_intervention.user_label).expect("TODO: panic message");
    // }
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

    println!("Break ended: session {}, on time: {}", end_break.break_session_id, end_break.returned_on_time);
    let mut current_break_id = on_break.break_id.lock().unwrap();
    *current_break_id = Some(end_break.break_session_id);

    update_break(&config.paths.database_path, end_break, end_time).expect("TODO: panic message");

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
pub fn update_label_streak(config: State<AppConfig>, state_confirmation: IdleFocusedPackage) -> Result<(), String> {

    update_n_windows_before(&config.paths.database_path, state_confirmation).map_err(|err| err.to_string())?;

    Ok(())
}