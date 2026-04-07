use std::sync::mpsc::Sender;
use std::time::{UNIX_EPOCH};
use rdev::{listen, Event, EventType};
use active_win_pos_rs::get_active_window;
use chrono::{DateTime};

use crate::models::input_event::Input;


pub fn callback(event: Event, tx: Sender<Input>) {

    let sys_time = event.time;
    let difference = sys_time.duration_since(UNIX_EPOCH).unwrap().as_secs();

    let secs = i64::try_from(difference).unwrap();
    let dt = DateTime::from_timestamp(secs, 0).unwrap();
    let timestamp = dt.format("%Y-%m-%d %H:%M:%S");
    println!("Timestamp: {} ", timestamp);

    let (event_type, event_action, key_code, mouse_X, mouse_Y, wheel_X, wheel_Y, button) = match event.event_type {
        EventType::KeyPress(key) => ("keyboard", "KeyPress", Some(format!("{:?}", key)), None, None, None, None, None),
        EventType::KeyRelease(key) => ("keyboard", "KeyRelease", Some(format!("{:?}", key)), None, None, None, None, None),
        EventType::MouseMove {x, y} => ("Mouse", "MouseMove", None, Some(x), Some(y), None, None, None),
        EventType::Wheel {delta_x, delta_y} => ("Mouse", "WheelScroll", None, None, None, Some(delta_x), Some(delta_y), None),
        EventType::ButtonPress(btn) => ("Mouse", "ButtonPress", None, None, None, None, None, Some(format!("{:?}", btn))),
        EventType::ButtonRelease(btn) => ("Mouse", "ButtonRelease", None, None, None, None, None, Some(format!("{:?}", btn))),

        _ => ("Unknown", "Unknown", None, None, None, None, None, None)
    };

    let activeWindow = match get_active_window() {
        Ok(active_window) => active_window.app_name,
        Err(()) => String::from("Unknown window")
    };


    let input = Input {
        timestamp: secs,
        event_type: event_type.to_string(),
        event_action: event_action.to_string(),
        key_code: key_code,
        mouse_x: mouse_X,
        mouse_y: mouse_Y,
        wheel_x: wheel_X,
        wheel_y: wheel_Y,
        button: button,
        active_window: activeWindow,
    };

    // println!("Input: {:?}", input);
    tx.send(input).unwrap();
}

pub fn logging(tx: Sender<Input>){
    // This will block.
    // if let Err(error) = listen(callback) {
    //     println!("Error: {:?}", error)
    // }
    
    if let Err(error) = listen(move |event| {
        callback(event, tx.clone());
    }) {
        println!("Error: {:?}", error)
    }
}