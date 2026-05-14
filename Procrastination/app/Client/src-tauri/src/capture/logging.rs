use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::time::{UNIX_EPOCH};
use std::sync::{Mutex, OnceLock};
use rdev::{listen, Event, EventType};
use active_win_pos_rs::get_active_window;
use chrono::{DateTime};
use crate::models::table_structs::Input;

static LAST_WINDOW: OnceLock<Mutex<String>> = OnceLock::new();
static LAST_ON_BREAK_STATE: OnceLock<Mutex<Option<bool>>> = OnceLock::new();


pub fn callback(event: Event, tx: Sender<Input>, running: &Arc<AtomicBool>, on_break: &Arc<AtomicBool>) {
    if !running.load(Ordering::Relaxed) {
        return;
    }

    let is_on_break = on_break.load(Ordering::SeqCst);
    if let Ok(mut last_state) = LAST_ON_BREAK_STATE.get_or_init(|| Mutex::new(None)).lock() {
        if last_state.map(|state| state != is_on_break).unwrap_or(true) {
            *last_state = Some(is_on_break);
        }
    }

    if is_on_break {
        return;
    }


    let sys_time = event.time;
    let difference = match sys_time.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs(),
        Err(_) => return,
    };

    let secs = match i64::try_from(difference) {
        Ok(v) => v,
        Err(_) => return,
    };

    let _dt = match DateTime::from_timestamp(secs, 0) {
        Some(v) => v,
        None => return,
    };


    let (event_type, event_action, key_code, mouse_x, mouse_y, wheel_x, wheel_y, button) = match event.event_type {
        EventType::KeyPress(key) => ("keyboard", "KeyPress", Some(format!("{:?}", key)), None, None, None, None, None),
        EventType::KeyRelease(key) => ("keyboard", "KeyRelease", Some(format!("{:?}", key)), None, None, None, None, None),
        EventType::MouseMove {x, y} => ("Mouse", "MouseMove", None, Some(x), Some(y), None, None, None),
        EventType::Wheel {delta_x, delta_y} => ("Mouse", "WheelScroll", None, None, None, Some(delta_x), Some(delta_y), None),
        EventType::ButtonPress(btn) => ("Mouse", "ButtonPress", None, None, None, None, None, Some(format!("{:?}", btn))),
        EventType::ButtonRelease(btn) => ("Mouse", "ButtonRelease", None, None, None, None, None, Some(format!("{:?}", btn))),
    };

    let active_window = if matches!(event.event_type, EventType::MouseMove { .. }) {
        LAST_WINDOW
            .get_or_init(|| Mutex::new(String::new()))
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    } else if event_action == "WheelScroll" {
        String::from("")
    } else {
        let window = get_active_window()
            .map(|w| w.app_name)
            .unwrap_or_else(|()| String::from("Unknown window"));

        if let Ok(mut cached) = LAST_WINDOW.get_or_init(|| Mutex::new(String::new())).lock() {
            *cached = window.clone();
        }

        window
    };


    let input = Input {
        timestamp: secs,
        event_type: event_type.to_string(),
        event_action: event_action.to_string(),
        key_code,
        mouse_x,
        mouse_y,
        wheel_x,
        wheel_y,
        button,
        active_window,
    };

    if let Err(error) = tx.send(input) {
        eprintln!("tx.send failed: {}", error);
    }
}

pub fn logging(tx: Sender<Input>, running: Arc<AtomicBool>, on_break: Arc<AtomicBool>) {
    let user_break = Arc::clone(&on_break);
    
    if let Err(error) = listen(move |event| {
        callback(event, tx.clone(), &running, &user_break);
    }) {
        println!("Error: {:?}", error)
    }
}