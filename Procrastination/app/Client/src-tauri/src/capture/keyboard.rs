use std::sync::mpsc::Sender;
use std::time::{UNIX_EPOCH};
use rdev::{listen, Event, EventType};
use active_win_pos_rs::get_active_window;
use chrono::{DateTime};

use crate::models::input_event::Input;


pub fn callback(event: Event, tx: Sender<Input>) {

    // println!("My callback {:?}", event);

    let sys_time = event.time;
    let difference = sys_time.duration_since(UNIX_EPOCH).unwrap().as_secs();
    // println!("{difference:?}");

    let secs = i64::try_from(difference).unwrap();
    let dt = DateTime::from_timestamp(secs, 0).unwrap();
    let timestamp = dt.format("%Y-%m-%d %H:%M:%S");
    println!("Timestamp: {} ", timestamp);

    // match event.name{
    //     Some(string) => println!("User writes {:?}", string),
    //     None => ()
    // }

    // match event.event_type {
    //     // EventType::from(event_type) => println!("Event Type: {:?}", event.event_type),
    //     EventType::ButtonPress(Button) => println!("User pressed {:?}", Button),
    //     EventType::ButtonRelease(Button) => println!("User released {:?}", Button),
    //     EventType::KeyPress(key) => println!("User key {:?}", key),
    //     EventType::KeyRelease(key) => println!("User key released {:?}", key),
    //     EventType::MouseMove {x, y} => println!("User moved to {:?} {:?}", x, y),
    //     EventType::Wheel {delta_x, delta_y} => println!("User scrolled wheel to: {} {}", delta_x, delta_y),
    //     _ => {}
    // }

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
        // {
        //     println!("active window: {:#?}", active_window.app_name);
        // }
        // Err(()) => {
        //     println!("error occurred while getting the active window");
        // }
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