use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use ort::session::Session;
use tauri::{AppHandle, Emitter, Manager};
use crate::database::sqlite::{collect_events, insert_features, insert_predictions, insert_interventions, update_break_focus_score, update_pref_focus_score, get_setting};
use crate::intervention::jitai::suggest_activity;
use crate::ml::inference::run_inference;
use crate::models::table_structs::{FeatureVectors, Input, Predictions, Interventions};
use crate::models::models::{IdleFocusedPackage, InterventionPackage, PredictionPackage};

fn show_popup_window(app_handle: &AppHandle) {
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
}

pub fn run_extractor(db_path: &Path, running: &Arc<AtomicBool>, session: &Arc<Mutex<Session>>, app_handle: &AppHandle, on_break: &Arc<AtomicBool>, end_break: &Arc<AtomicBool>, break_id: &Arc<Mutex<Option<i64>>>) {
    let confidence_threshold = 0.75;  //Should I reduce the score? Ask spv
    let mut prediction_counter = 0;
    let mut focused_counter = 0;
    let mut idle_counter = 0;

    let focused_streak_threshold: i32 = get_setting(db_path, "focused_streak_window").unwrap_or(None).and_then(|v| v.parse::<i32>().ok()).unwrap_or(15);

    let mut post_break_remaining_windows = 0;
    let mut post_break_scores: Vec<f64> = Vec::new();
    let mut resume_from_ts: Option<i64> = None;

    thread::sleep(Duration::from_secs(60));

    loop {

        if !running.load(std::sync::atomic::Ordering::SeqCst) { break; }

        if on_break.load(std::sync::atomic::Ordering::SeqCst) {
            prediction_counter = 0;
            continue;
        }

        if end_break.load(std::sync::atomic::Ordering::SeqCst) {
            post_break_remaining_windows = 5;
            post_break_scores.clear();
            end_break.store(false, std::sync::atomic::Ordering::SeqCst);
            // make sure window with  break time event is not computed
            //  only resume predictions 1 min after user hits "I'm back".
            resume_from_ts = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64);
        }

        let start_time = std::time::Instant::now();
        let window_end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        let window_start = window_end - 60;

        let app_handle = app_handle.clone();

        if let Some(resume_ts) = resume_from_ts {
            if window_start < resume_ts {
                // Not enough post-break time has passed to form a proper 60s window.
                let elapsed = start_time.elapsed();
                let sleep = Duration::from_secs(60).saturating_sub(elapsed);
                thread::sleep(sleep);
                continue;
            } else {
                resume_from_ts = None;
            }
        }

        let events = match collect_events(db_path, window_start, window_end) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Failed to collect events: {}", e);
                continue;
            }
        };


        let features = extract_features(events, window_start, window_end);

        let feature_id = match insert_features(db_path, &features) {
            Ok(id) => id,
            Err(e) => {
                eprintln!("Failed to insert features: {}", e);
                continue;
            }
        };

        let mut session_guard = match session.lock() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Session mutex poisoned, skipping window: {}", e);
                let elapsed = start_time.elapsed();
                let sleep = Duration::from_secs(60).saturating_sub(elapsed);
                thread::sleep(sleep);
                continue;
            }
        };
        let (label, confidence) = match run_inference(&mut session_guard, &features){
            Ok(result) => result,
            Err(err) => {eprintln!("Inference failed: {}", err); continue}
        };

        let prediction = Predictions {
            feature_vectors_id: feature_id,
            timestamp: window_end,
            predicted_state: label,
            confidence,
            window_size_seconds: 60,
            was_corrected: false,
        };

        let prediction_id = match insert_predictions(db_path, &prediction) {
            Ok(id) => id,
            Err(e) => {
                eprintln!("Failed to insert prediction: {}", e);
                continue;
            }
        };

        //for dashbaord live prediction view
        let prediction_payload = PredictionPackage {
            prediction_id,
            feature_vector_id: feature_id,
            prediction_label: prediction.predicted_state.clone(),
            confidence,
            timestamp: window_end * 1000,
        };
        if let Err(e) = app_handle.emit("new_prediction", prediction_payload) {
            eprintln!("Failed to emit new_prediction: {}", e);
        }

        if post_break_remaining_windows > 0 {
            if prediction.predicted_state == "Focused" {
                post_break_scores.push(confidence.clone())
            }
            else {
                post_break_scores.push(0.0)
            }

            post_break_remaining_windows -= 1
        }
        else if post_break_remaining_windows == 0 && !post_break_scores.is_empty() {

            let post_break_average = post_break_scores.iter().sum::<f64>() / 5.0;

            let break_session_id = match break_id.lock() {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("break_id mutex poisoned in post-break scoring: {}", e);
                    post_break_scores.clear();
                    let elapsed = start_time.elapsed();
                    let sleep = Duration::from_secs(60).saturating_sub(elapsed);
                    thread::sleep(sleep);
                    continue;
                }
            };
            if let Some(break_sess_id) = *break_session_id {
                if let Err(e) = update_break_focus_score(&db_path, break_sess_id, post_break_average) {
                    eprintln!("Failed to update break focus score: {}", e);
                }
                if let Err(e) = update_pref_focus_score(&db_path, break_sess_id, post_break_average) {
                    eprintln!("Failed to update preference focus score: {}", e);
                }
            }

            post_break_scores.clear();
        }

        //Logic for intervention popups
        let intervention_confidence = &prediction.confidence >= &confidence_threshold;

        if (&prediction.predicted_state == "Procrastinating" || &prediction.predicted_state == "At Risk") {
            focused_counter = 0;
            idle_counter = 0;

            if intervention_confidence {
                prediction_counter += 1;
            }
        }
        else if &prediction.predicted_state == "Focused" {
            prediction_counter = 0;
            idle_counter = 0;

            if intervention_confidence {
                focused_counter += 1;
            }
        }
        else if &prediction.predicted_state == "Idle" {
            focused_counter = 0;
            prediction_counter = 0;

            if intervention_confidence {
                idle_counter += 1;
            }
        }
        else {
            prediction_counter = 0;
            focused_counter = 0;
            idle_counter = 0;
        }


        if prediction_counter == 3 {
            prediction_counter = 0;
            let intervention = Interventions {
                predictions_id: prediction_id,
                timestamp: window_end,
                intervention_type: "PopUp".parse().unwrap(),
                prediction_label: prediction.predicted_state.clone(),
                user_label: None,
                dismissed: false,
            };

            let intervention_id = match insert_interventions(db_path, &intervention) {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("Failed to insert intervention: {}", e);
                    continue;
                }
            };

            let activity_suggestion = suggest_activity(db_path);

            let payload = InterventionPackage {
                intervention_id,
                timestamp: window_end,
                intervention_type: "PopUp".to_string(),
                prediction_label: prediction.predicted_state,
                confidence,
                suggested_activity: Option::from(activity_suggestion.activity),
                suggested_duration: Option::from(activity_suggestion.random_duration),
                preference_id: Option::from(activity_suggestion.preference_id),
            };

            show_popup_window(&app_handle);

            if let Err(e) = app_handle.emit("new_intervention", payload) {
                eprintln!("Failed to emit new_intervention: {}", e);
            }
        }
        else if focused_counter == focused_streak_threshold {

            let focused_payload = IdleFocusedPackage {
                timestamp: window_end,
                streak_windows: focused_counter,
                label: "Focused".to_string(),
                overwrite: false,
            };

            focused_counter = 0;

            show_popup_window(&app_handle);
            if let Err(e) = app_handle.emit("focus_check", focused_payload) {
                eprintln!("Failed to emit focus_check: {}", e);
            }

        }
        else if idle_counter == 10 {

            let idle_payload = IdleFocusedPackage {
                timestamp: window_end,
                streak_windows: idle_counter,
                label: "Idle".to_string(),
                overwrite: false,
            };

            idle_counter = 0;

            show_popup_window(&app_handle);
            if let Err(e) = app_handle.emit("idle_check", idle_payload) {
                eprintln!("Failed to emit idle_check: {}", e);
            }
            
        }


        let elapsed = start_time.elapsed();
        let sleep = Duration::from_secs(60).saturating_sub(elapsed);
        thread::sleep(sleep);
    }
}

pub fn extract_features(events: Vec<Input>, window_start: i64, window_end: i64) -> FeatureVectors {
    let window_time = 60.0;

    //get the typing speed. sum of key press events / 60
    let key_press: Vec<&Input> = events.iter().filter(|event| event.event_action == "KeyPress").collect();
    let key_count = key_press.len() as f64;
    let typing_speed = key_count/window_time;

    //get the repetitive key ratio(multiple smae key clicks or not?), divide 60(sliding window time)
    let repetitive_key = if key_count == 0.0 {
         0.0
    }
    else {
        key_press.windows(2).filter(|pair| {
            matches!((&pair[0].key_code, &pair[1].key_code), (Some(a), Some(b)) if a == b)
        }).count() as f64
    };

    let repetitive_ratio = if key_count == 0.0 { 0.0 } else {repetitive_key / key_count};

    //mouse vel. cal distancebetween each mouse move event. divide 60 to find ratio
    let mouse_move: Vec<&Input> = events.iter().filter(|event| event.event_action == "MouseMove").collect();

    let mouse_dist = if mouse_move.len() <= 1 {
        0.0
    }
    else {
        mouse_move.windows(2).map(|pair| {
            let dx = pair[1].mouse_x.unwrap_or(0.0) - pair[0].mouse_x.unwrap_or(0.0);
            let dy = pair[1].mouse_y.unwrap_or(0.0) - pair[0].mouse_y.unwrap_or(0.0);
            let dxdy = (dx*dx) + (dy*dy);
            dxdy.sqrt()
        }).sum::<f64>()
    };

    let mouse_velocity = mouse_dist / window_time;

    //idle ratio
    let idle = if events.len() == 0 {
        window_time
    }
    else {
        let first_gap = (events[0].timestamp - window_start) as f64;
        let first_idle = if first_gap > 2.0 {first_gap} else {0.0};
        let last_gap = (window_end - events[events.len() - 1].timestamp) as f64;
        let last_idle = if last_gap > 2.0 {last_gap} else { 0.0 };
        let event_idle = events.windows(2).map(|pair| {
            (pair[1].timestamp - pair[0].timestamp) as f64
        }).filter(|gap| *gap > 2.0).sum::<f64>();

        first_idle + event_idle + last_idle
    };

    let idle_ratio = (idle / window_time).min(1.0);

    let window_activity: Vec<&Input> = events.iter().filter(|event| {
        event.event_action != "MouseMove" &&
        event.event_action != "WheelScroll"
    }).collect();

    let window_switch = if window_activity.len() <= 1 {
        0.0
    }
    else {
        window_activity.windows(2).filter(|pair| pair[0].active_window != pair[1].active_window).count() as f64
    };

    let window_switch_ratio = window_switch / 1.0;

    //wheel scrooling
    let scroll_events: Vec<&Input> = events.iter().filter(|event| event.event_action == "WheelScroll").collect();

    let scroll_velocity = scroll_events.iter().map(|event| event.wheel_y.unwrap_or(0).abs() as f64)
        .sum::<f64>() / window_time;


    let feature_vector = FeatureVectors {
        timestamp: window_end,
        typing_speed,
        repetitive_key_ratio: repetitive_ratio,
        mouse_velocity,
        idle_ratio,
        window_switch_frequency: window_switch_ratio,
        scroll_velocity,
    };
    
    feature_vector
}