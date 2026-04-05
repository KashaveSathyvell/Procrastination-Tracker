// src-tauri/src/database/sqlite.rs
use rusqlite::{params, Connection, Result};
use std::path::Path;

use crate::models::input_event::Input;
pub fn initialize_database(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS input_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL,
            event_type TEXT NOT NULL,
            event_action TEXT NOT NULL,
            key_code TEXT,
            mouse_x REAL,
            mouse_y REAL,
            wheel_x INTEGER,
            wheel_y INTEGER,
            button TEXT,
            active_window TEXT
        );

        CREATE TABLE IF NOT EXISTS feature_vectors (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL,
            typing_speed REAL,
            mouse_velocity REAL,
            idle_ratio REAL,
            window_switch_frequency REAL,
            state_label TEXT
        );

        CREATE TABLE IF NOT EXISTS predictions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL,
            state TEXT NOT NULL,
            confidence REAL,
            window_size_seconds INTEGER
        );

        CREATE TABLE IF NOT EXISTS interventions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL,
            intervention_type TEXT,
            reason TEXT
        );
        "
    )?;

    println!("Database schema initialized successfully.");
    Ok(conn)
}


pub fn insert_events(db_path: &Path, input: &Input) -> Result<()> {
    let conn = Connection::open(db_path)?;
    println!("Openning database for write {:?}", db_path);

    conn.execute(
        "INSERT INTO input_events(timestamp, event_type, event_action, key_code, mouse_x, mouse_y, wheel_x, wheel_y, button, active_window) \
        Values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![input.timestamp, input.event_type, input.event_action, input.key_code.as_deref(), input.mouse_x, input.mouse_y, input.wheel_x, input.wheel_y, input.button.as_deref(), input.active_window]
    )?;

    println!("Data added into database: {:?}", input);
    Ok(())
}