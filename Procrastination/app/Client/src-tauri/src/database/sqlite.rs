// src-tauri/src/database/sqlite.rs
use rusqlite::{params, Connection, Result};
use std::path::Path;
use tauri::menu::NativeIcon::User;
use crate::models::models::{EndBreak, PreferenceUpdate, UpdateIntervention};
use crate::models::table_structs::{FeatureVectors, Input, Predictions, Interventions, UserPreferences, BreakSessions};
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
            min_duration_minutes INTEGER NOT NULL,
            max_duration_minutes INTEGER NOT NULL,
            times_suggested INTEGER default 0,
            times_completed INTEGER default 0,
            average_focus_score REAL default 0.0,
            last_suggested INTEGER
        );

        CREATE TABLE IF NOT EXISTS break_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            intervention_id INTEGER,
            start_timestamp INTEGER NOT NULL,
            end_timestamp INTEGER,
            preference_id INTEGER,
            activity TEXT,
            planned_duration_minutes INTEGER,
            returned_on_time INTEGER default 0,
            post_break_focus_score REAL,
            FOREIGN KEY(intervention_id) REFERENCES interventions(id)
            FOREIGN KEY(preference_id) REFERENCES user_preferences(id)
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

pub fn insert_interventions(db_path: &Path, interventions: &Interventions) -> Result<(i64)> {
    let conn = Connection::open(db_path)?;

    conn.execute(
        "INSERT INTO interventions(predictions_id, timestamp, intervention_type, prediction_label, user_label, dismissed)\
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![interventions.predictions_id, interventions.timestamp, interventions.intervention_type, interventions.prediction_label, interventions.user_label, interventions.dismissed]
    )?;

    let id = conn.last_insert_rowid();
    Ok(id)
}

pub fn insert_user_preference(db_path: &Path, preference: &UserPreferences) -> Result<()> {
    let conn = Connection::open(db_path)?;

    conn.execute(
        "INSERT INTO user_preferences(activity_name, min_duration_minutes, max_duration_minutes, times_suggested, times_completed, average_focus_score, last_suggested) \
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![&preference.activity_name, preference.min_duration_minutes, preference.max_duration_minutes, preference.times_suggested, preference.times_completed, preference.average_focus_score, preference.last_suggested]
    )?;

    Ok(())
}

pub fn insert_break_sessions(db_path: &Path, break_sessions: &BreakSessions) -> Result<(i64)> {
    let conn = Connection::open(db_path)?;

    conn.execute(
        "INSERT INTO break_sessions(intervention_id, start_timestamp, end_timestamp, preference_id, activity, planned_duration_minutes, returned_on_time, post_break_focus_score) \
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![break_sessions.intervention_id, break_sessions.start_time, break_sessions.end_time, &break_sessions.preference_id, &break_sessions.activity, break_sessions.planned_duration_mins, break_sessions.returned_on_time, break_sessions.post_break_focus_score]
    )?;

    let id = conn.last_insert_rowid();

    Ok(id)
}

pub fn update_break(db_path: &Path, updated_break: EndBreak, end_time: i64) -> Result<()> {
    let conn = Connection::open(db_path)?;

    conn.execute("UPDATE break_sessions \
        SET returned_on_time = ?1, end_timestamp = ?2 \
        WHERE id = ?3",
        params![updated_break.returned_on_time, end_time, updated_break.break_session_id]
    )?;

    Ok(())
}

pub fn update_user_label(db_path: &Path, updated: &UpdateIntervention) -> Result<()> {
    let conn = Connection::open(db_path)?;

    conn.execute(
        "UPDATE interventions \
         SET user_label = ?1, dismissed = ?2 \
         WHERE id=?3",
        params![updated.user_label, updated.dismissed, updated.intervention_id]
    )?;

    Ok(())
}

pub fn prediction_corrected(db_path:&Path, prediction_id: i64, corrected: bool) -> Result<()> {
    let conn = Connection::open(db_path)?;

    conn.execute("UPDATE predictions \
    SET was_corrected = ?1 \
    WHERE id = ?2"
    , params![corrected, prediction_id])?;

    Ok(())
}

pub fn assign_truth_label(db_path: &Path, feature_vector_id: i64, truth_label: String) -> Result<()> {
    let conn = Connection::open(db_path)?;

    conn.execute("UPDATE feature_vectors \
    SET truth_label = ?1 \
    WHERE id = ?2",
    params!(truth_label, feature_vector_id))?;

    Ok(())
}

pub fn get_ids(db_path: &Path, intervention_id: i64) -> Result<(i64, i64)> {
    let conn = Connection::open(db_path)?;

    conn.query_row(
        "SELECT predictions.id, predictions.feature_vector_id \
        FROM interventions \
        JOIN predictions ON interventions.predictions_id = predictions.id \
        WHERE interventions.id = ?1",
        params![intervention_id],
        |row| Ok((row.get(0)?, row.get(1)?))
    )
}

pub fn has_preferences(db_path:&Path) -> Result<i64> {
    let conn = Connection::open(db_path)?;
    
    let total: i64 = conn.query_row("SELECT COUNT(*) FROM user_preferences",
                             params![], |row| row.get(0))?;
    
    Ok(total)
}

pub fn get_all_preferences(db_path: &Path) -> Result<Vec<UserPreferences>> {
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare(
        "SELECT * \
        from user_preferences",
    )?;

    let rows = stmt.query_map(params![], |row| {
        Ok(UserPreferences {
            id: row.get(0)?,
            activity_name: row.get(1)?,
            min_duration_minutes: row.get(2)?,
            max_duration_minutes: row.get(3)?,
            times_suggested: row.get(4)?,
            times_completed: row.get(5)?,
            average_focus_score: row.get(6)?,
            last_suggested: row.get(7)?
        })
    })?;

    let mut preferences = Vec::new();
    for pref in rows {
        preferences.push(pref?);
    }

    Ok(preferences)
}

pub fn update_user_preferences(db_path: &Path, updated_preference: PreferenceUpdate) -> Result<()> {
    let conn = Connection::open(db_path)?;
    
    conn.execute(
        "UPDATE user_preferences \
        SET last_suggested = ?1, times_suggested = ?2 \
        WHERE id = ?3",
        params![updated_preference.last_suggested, updated_preference.times_suggested, updated_preference.preference_id],
    )?;
    
    Ok(())
}

pub fn update_break_focus_score(db_path: &Path, break_id: i64, score: f64) -> Result<()> {
    let conn = Connection::open(db_path)?;

    conn.execute(
        "UPDATE break_sessions \
        SET post_break_focus_score = ?1 \
        WHERE id = ?2",
        params![score, break_id],
    )?;

    Ok(())
}

pub fn update_pref_focus_score(db_path: &Path, break_session_id: i64, new_score: f64) -> Result<()> {
    let conn = Connection::open(db_path)?;

    let preference_id: i64 = conn.query_row(
        "SELECT preference_id FROM break_sessions \
        WHERE id = ?1",
        params![break_session_id],
        |row| row.get(0)
    )?;

    let (average_score, times_completed): (f64, i64) = conn.query_row(
        "SELECT average_focus_score, times_completed \
        FROM user_preferences \
        WHERE id = ?1",
        params![preference_id],
        |row| Ok((row.get(0)?, row.get(1)?))
    )?;

    let new_average_score = (average_score * times_completed as f64 + new_score) / (times_completed + 1) as f64;
    
    conn.execute(
        "UPDATE user_preferences \
        SET average_focus_score = ?1, times_completed = ?2 \
        WHERE id = ?3",
        params![new_average_score, times_completed+1, preference_id],
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