// src-tauri/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::path::Path;
use tauri::async_runtime::handle;

use client_lib::capture::keyboard::{callback, logging};
use client_lib::features::feature_extractor::run_extractor;

mod config;
use client_lib::database::sqlite::{initialize_database, insert_events};

use config::AppConfig;
use tauri::Manager;
use client_lib::models::input_event::Input;

fn main() {

    initialize_database(Path::new("behavior.db"));
    println!("{:?}", std::env::current_dir().unwrap());

    let (tx, rx) = mpsc::channel::<Input>();

    // let tx1 = tx.clone();
    let handle = thread::spawn(move || {
        logging(tx);
    });

    let handle = thread::spawn(move || {
        for received in rx {
            // println!("Got: {:?}", received);
            insert_events(Path::new("behavior.db"), &received);
        }
    });
    
    let handle = thread::spawn(move || {
        run_extractor(Path::new("behavior.db"));
    });

    handle.join().unwrap();

    // tauri::Builder::default()
    //     .setup(|app| {
    //         let app_config = AppConfig::new(&app.handle());
    //
    //         // Call the sqlite initialization function
    //         match database::sqlite::initialize_database(&app_config.paths.database_path) {
    //             Ok(_conn) => println!("Database is ready to use!"),
    //             Err(e) => eprintln!("Failed to initialize database: {}", e),
    //         }
    //
    //         app.manage(app_config);
    //         Ok(())
    //     })
    //     .run(tauri::generate_context!())
    //     .expect("error while running tauri application");
}