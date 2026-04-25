use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use ort::session::Session;
use tauri::{AppHandle, Emitter};
use crate::database::sqlite::{collect_events, insert_features, insert_predictions, insert_interventions, update_break_focus_score, update_pref_focus_score};
use crate::intervention::jitai::suggest_activity;
use crate::ml::inference::run_inference;
use crate::models::table_structs::{FeatureVectors, Input, Predictions, Interventions};
use crate::models::models::InterventionPackage;

pub fn run_extractor(db_path: &Path, running: &Arc<AtomicBool>, session: &Arc<Mutex<Session>>, app_handle: &AppHandle, on_break: &Arc<AtomicBool>, end_break: &Arc<AtomicBool>, break_id: &Arc<Mutex<Option<i64>>>) {
    let confidence_threshold = 0.75;
    let mut prediction_counter = 0;
    let mut post_break_remaining_windows = 0;
    let mut post_break_scores: Vec<f64> = Vec::new();

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
        }

        let start_time = std::time::Instant::now();
        let window_end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let window_start = window_end - 60;

        let app_handle = app_handle.clone();

        let events = collect_events(db_path, window_start, window_end).unwrap();

        //DEBUG
        println!("Events in window: {}", events.len());
        if let Some(first) = events.first() {
            println!("First event timestamp: {}", first.timestamp);
        }
        if let Some(last) = events.last() {
            println!("Last event timestamp: {}", last.timestamp);
        }
        println!("Window: {} to {}", window_start, window_end);



        let features = extract_features(events, window_start, window_end);

        let feature_id = insert_features(db_path, &features).expect("TODO: panic message");

        let mut session_guard = session.lock().unwrap();
        let (label, confidence) = match run_inference(&mut session_guard, &features){
            Ok((result)) => (result),
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

        let prediction_id = insert_predictions(db_path, &prediction).expect("TODO: panic message");

        println!("PREDICTION: {:?} Confidence: {:?}", &prediction.predicted_state, &prediction.confidence);

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
            let break_session_id = break_id.lock().unwrap();
            let break_sess_id = break_session_id.unwrap();
            update_break_focus_score(&db_path, break_sess_id, post_break_average).expect("TODO: panic message");

            update_pref_focus_score(&db_path, break_sess_id, post_break_average).expect("TODO: panic message");

            post_break_scores.clear();
        }

        

        let intervention_confidence = &prediction.confidence >= &confidence_threshold;

        if intervention_confidence && (&prediction.predicted_state == "Procrastinating" || &prediction.predicted_state == "At Risk") {
            prediction_counter += 1;
        }
        else {
            prediction_counter = 0;
        }

        if prediction_counter == 1 {
            prediction_counter = 0;
            let intervention = Interventions {
                predictions_id: prediction_id,
                timestamp: window_end,
                intervention_type: "PopUp".parse().unwrap(),
                prediction_label: prediction.predicted_state.clone(),
                user_label: None,
                dismissed: false,
            };
            
            let intervention_id = insert_interventions(db_path, &intervention).expect("TODO: panic message");

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

            app_handle.emit("new_intervention", payload).expect("TODO: panic message");
        }

        let elapsed = start_time.elapsed();
        let sleep = Duration::from_secs(60).saturating_sub(elapsed);
        thread::sleep(sleep);
    }
}

pub fn extract_features(events: Vec<Input>, window_start: i64, window_end: i64) -> FeatureVectors {
    let window_time = 60;

    //get the typing speed. sum of key press events / 60
    let key_press: Vec<&Input> = events.iter().filter(|event| event.event_action == "KeyPress").collect();
    let key_count = key_press.len() as f64;
    let typing_speed = key_count/60.0;
    println!("Amount of key presses: {}", key_count);

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

    let mut mouse_dist = if mouse_move.len() <= 1 {
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

    let mouse_velocity = mouse_dist / 60.0;

    //idle ratio
    let mut idle = if events.len() == 0 {
        60.0
    }
    else {
        let first_gap = (events[0].timestamp - window_start) as f64;
        let first_idle = if first_gap > 2.0 {first_gap} else {0.0};
        let last_gap = (window_end - events[events.len() - 1].timestamp) as f64;
        let last_idle = if last_gap > 2.0 {last_gap} else { 0.0 };
        let event_idle = events.windows(2).map(|pair| {
            (pair[1].timestamp - pair[0].timestamp) as f64
        }).filter(|gap| *gap > 2.0).sum::<f64>();

        //DEBUG
        println!("First Gap: {}, Event Gap: {}, Last Gap: {}", first_idle, event_idle, last_idle);

        first_idle + event_idle + last_idle
    };

    let idle_ratio = (idle / 60.0).min(1.0);

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

    let feature_vector = FeatureVectors {
        timestamp: window_end,
        typing_speed,
        repetitive_key_ratio: repetitive_ratio,
        mouse_velocity,
        idle_ratio,
        window_switch_frequency: window_switch_ratio,
    };
    println!("Features calculated: {:?}", feature_vector);
    feature_vector
}