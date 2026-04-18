use std::sync::mpsc::{Sender, Receiver, TryRecvError};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::Thread;
use ort::editor::Model;
use rusqlite::fallible_iterator::FallibleIterator;
use tauri::async_runtime::handle;
use tauri::AppHandle;

use crate::capture::keyboard::{callback, logging};
use crate::database::sqlite::{initialize_database, insert_events, update_user_label};
use crate::features::feature_extractor::run_extractor;
use tauri::{Manager, State};
use tauri::WebviewUrl::App;
use crate::models::table_structs::Input;
use crate::models::models::{ThreadStop, ModelState, UpdateIntervention};


#[tauri::command]
pub fn start_collect(app_handle: AppHandle, state: State<ThreadStop>, model_state: State<ModelState>) -> Result<(), String> {
    let session_clone = model_state.session.clone();

    let is_running = state.running_collect.swap(true, Ordering::Relaxed);

    if is_running {
        return Err(String::from("Collecting has already started"))
    }

    let running_clone1 = state.running_collect.clone();
    let running_clone2 = state.running_collect.clone();
    let running_clone3 = state.running_collect.clone();

    println!("Collecting data");
    initialize_database(Path::new("behavior.db"));
    println!("{:?}", std::env::current_dir().unwrap());

    let (tx, rx) = mpsc::channel::<Input>();
     // let tx1 = tx.clone();
    let handle1 = thread::spawn(move || {
        logging(tx, running_clone1);
    });

    let handle2 = thread::spawn(move || {
        while running_clone2.load(Ordering::Relaxed) {
            match rx.try_recv() {
                Ok(received) => { insert_events(Path::new("behavior.db"), &received); }
                Err(TryRecvError::Empty) => { thread::sleep(Duration::from_millis(10)) }
                _ => {}
            }
        }
       //  for received in rx {
       //      insert_events(Path::new("behavior.db"), &received);
       // }
    });

    let handle3 = thread::spawn(move || {
            run_extractor(Path::new("behavior.db"), &running_clone3, &session_clone, &app_handle);
    });

    Ok(())
}


#[tauri::command]
pub fn stop_collect(state: State<ThreadStop>) -> Result<(), String> {

    state.running_collect.store(false, Ordering::Relaxed);
    println!("Stopping threads");

    Ok(())
}


#[tauri::command]
pub fn intervention_update(updated: UpdateIntervention) {
    println!("Intervention update");
    update_user_label(Path::new("behavior.db"), updated).expect("TODO: panic message");
}