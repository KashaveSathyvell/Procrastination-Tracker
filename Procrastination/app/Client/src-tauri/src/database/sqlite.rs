// src-tauri/src/database/sqlite.rs
use rusqlite::{params, Connection, Result};
use std::path::Path;
use crate::models::models::UpdateIntervention;
use crate::models::table_structs::{FeatureVectors, Input, Predictions, Interventions};
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
            repetitive_key_ratio REAL,
            mouse_velocity REAL,
            idle_ratio REAL,
            window_switch_frequency REAL,
            truth_label TEXT
        );

        CREATE TABLE IF NOT EXISTS predictions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            feature_vector_ID INTEGER,
            timestamp INTEGER NOT NULL,
            predicted_state TEXT NOT NULL,
            confidence REAL,
            window_size_seconds INTEGER,
            was_corrected INTEGER DEFAULT 0,
            FOREIGN KEY(feature_vector_ID) REFERENCES feature_vectors(id)
        );

        CREATE TABLE IF NOT EXISTS interventions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            predictions_id INTEGER,
            timestamp INTEGER NOT NULL,
            intervention_type TEXT,
            prediction_label TEXT,
            user_label TEXT,
            dismissed INTEGER default 0,
            FOREIGN KEY(predictions_id) REFERENCES predictions(id)
        );

        CREATE TABLE IF NOT EXISTS user_preferences (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            activity_name TEXT NOT NULL,
            duration_minutes INTEGER NOT NULL,
            last_suggested INTEGER
        );

        CREATE TABLE IF NOT EXISTS break_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            intervention_id INTEGER,
            start_timestamp INTEGER NOT NULL,
            end_timestamp INTEGER,
            activity TEXT,
            FOREIGN KEY(intervention_id) REFERENCES interventions(id)
        );
        "
    )?;

    println!("Database schema initialized successfully.");
    Ok(conn)
}


pub fn insert_events(db_path: &Path, input: &Input) -> Result<()> {
    let conn = Connection::open(db_path)?;

    conn.execute(
        "INSERT INTO input_events(timestamp, event_type, event_action, key_code, mouse_x, mouse_y, wheel_x, wheel_y, button, active_window) \
        Values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![input.timestamp, input.event_type, input.event_action, input.key_code.as_deref(), input.mouse_x, input.mouse_y, input.wheel_x, input.wheel_y, input.button.as_deref(), input.active_window]
    )?;

    // println!("Data added into INPUT database: {:?}", input);
    Ok(())
}

pub fn insert_features(db_path: &Path, features: &FeatureVectors) -> Result<(i64)> {
    let conn = Connection::open(db_path)?;

    conn.execute(
        "INSERT INTO feature_vectors(timestamp, typing_speed, repetitive_key_ratio, mouse_velocity, idle_ratio, window_switch_frequency) \
        Values (?1, ?2, ?3, ?4, ?5, ?6)",
        params![features.timestamp, features.typing_speed, features.repetitive_key_ratio, features.mouse_velocity, features.idle_ratio, features.window_switch_frequency]
    )?;

    println!("Data added into FEATURE database: {:?}", features);

    let id = conn.last_insert_rowid();
    Ok((id))
}

pub fn insert_predictions(db_path: &Path, predictions: &Predictions) -> Result<(i64)> {
    let conn = Connection::open(db_path)?;

    conn.execute(
        "INSERT INTO predictions(feature_vector_id, timestamp, predicted_state, confidence, window_size_seconds, was_corrected) \
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![predictions.feature_vectors_id ,predictions.timestamp, predictions.predicted_state, predictions.confidence, predictions.window_size_seconds, predictions.was_corrected]
    )?;

    let id = conn.last_insert_rowid();

    Ok((id))
}

pub fn insert_interventions(db_path: &Path, interventions: &Interventions) -> Result<()> {
    let conn = Connection::open(db_path)?;

    conn.execute(
        "INSERT INTO interventions(predictions_id, timestamp, intervention_type, prediction_label, user_label, dismissed)\
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![interventions.predictions_id, interventions.timestamp, interventions.intervention_type, interventions.prediction_label, interventions.user_label, interventions.dismissed]
    )?;

    Ok(())
}

pub fn update_user_label(db_path: &Path, updated: UpdateIntervention) -> Result<()> {
    let conn = Connection::open(db_path)?;

    conn.execute(
        "UPDATE interventions\
         SET user_label = ?1, dismissed = ?2 \
         WHERE interventions_id=?3",
        params![updated.user_label, updated.dismissed, updated.intervention_id]
    )?;
    
    Ok(())
}


pub fn collect_events(db_path: &Path, window_start: i64, window_end: i64) -> Result<Vec<Input>> {
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare(
        "SELECT timestamp, event_type, event_action, key_code, mouse_x, mouse_y, wheel_x, wheel_y, button, active_window FROM input_events \
        WHERE timestamp BETWEEN ?1 AND ?2 \
        ORDER BY timestamp ASC",
        // params![windowStart, windowEnd],
    )?;

    let rows = stmt.query_map(params![window_start, window_end], |row| {
        Ok(Input{
            timestamp: row.get(0)?,
            event_type: row.get(1)?,
            event_action: row.get(2)?,
            key_code: row.get(3)?,
            mouse_x: row.get(4)?,
            mouse_y: row.get(5)?,
            wheel_x: row.get(6)?,
            wheel_y: row.get(7)?,
            button: row.get(8)?,
            active_window: row.get(9)?
        })
    })?;

    let mut features = Vec::new();
    for feature_vector in rows {
        features.push(feature_vector?);
    }

    Ok(features)
}