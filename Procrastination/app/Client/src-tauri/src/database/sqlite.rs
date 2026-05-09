// src-tauri/src/database/sqlite.rs
use rusqlite::{params, Connection, Result};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::menu::NativeIcon::User;
use crate::models::models::{ActivityScore, EndBreak, FocusScore, IdleFocusedPackage, PredictionHistoryRow, PreferenceUpdate, StateDistribution, UpdateIntervention};
use crate::models::table_structs::{FeatureVectors, Input, Predictions, Interventions, UserPreferences, BreakSessions};

fn open_connection(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;
         PRAGMA busy_timeout = 5000;",
    )?;
    Ok(conn)
}

pub fn initialize_database(db_path: &Path) -> Result<Connection> {
    let conn = open_connection(db_path)?;

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
            scroll_velocity REAL,
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

        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_input_events_timestamp
            ON input_events(timestamp);

        CREATE INDEX IF NOT EXISTS idx_feature_vectors_timestamp
            ON feature_vectors(timestamp);

        CREATE INDEX IF NOT EXISTS idx_predictions_timestamp
            ON predictions(timestamp);

        CREATE INDEX IF NOT EXISTS idx_interventions_timestamp
            ON interventions(timestamp);
        "
    )?;

    println!("Database schema initialized successfully.");
    Ok(conn)
}


pub fn insert_events(db_path: &Path, input: &Input) -> Result<()> {
    let conn = open_connection(db_path)?;

    conn.execute(
        "INSERT INTO input_events(timestamp, event_type, event_action, key_code, mouse_x, mouse_y, wheel_x, wheel_y, button, active_window) \
        Values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![input.timestamp, input.event_type, input.event_action, input.key_code.as_deref(), input.mouse_x, input.mouse_y, input.wheel_x, input.wheel_y, input.button.as_deref(), input.active_window]
    )?;

    Ok(())
}

pub fn insert_events_conn(conn: &Connection, input: &Input) -> Result<()> {
    conn.execute(
        "INSERT INTO input_events(timestamp, event_type, \
         event_action, key_code, mouse_x, mouse_y, \
         wheel_x, wheel_y, button, active_window) \
         Values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            input.timestamp,
            input.event_type,
            input.event_action,
            input.key_code.as_deref(),
            input.mouse_x,
            input.mouse_y,
            input.wheel_x,
            input.wheel_y,
            input.button.as_deref(),
            input.active_window
        ],
    )?;
    Ok(())
}

pub fn insert_features(db_path: &Path, features: &FeatureVectors) -> Result<(i64)> {
    let conn = open_connection(db_path)?;

    conn.execute(
        "INSERT INTO feature_vectors(timestamp, typing_speed, repetitive_key_ratio, mouse_velocity, idle_ratio, window_switch_frequency, scroll_velocity) \
        Values (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![features.timestamp, features.typing_speed, features.repetitive_key_ratio, features.mouse_velocity, features.idle_ratio, features.window_switch_frequency, features.scroll_velocity]
    )?;

    println!("Data added into FEATURE database: {:?}", features);

    let id = conn.last_insert_rowid();
    Ok((id))
}

pub fn insert_predictions(db_path: &Path, predictions: &Predictions) -> Result<(i64)> {
    let conn = open_connection(db_path)?;

    conn.execute(
        "INSERT INTO predictions(feature_vector_id, timestamp, predicted_state, confidence, window_size_seconds, was_corrected) \
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![predictions.feature_vectors_id ,predictions.timestamp, predictions.predicted_state, predictions.confidence, predictions.window_size_seconds, predictions.was_corrected]
    )?;

    let id = conn.last_insert_rowid();

    Ok((id))
}

pub fn insert_interventions(db_path: &Path, interventions: &Interventions) -> Result<(i64)> {
    let conn = open_connection(db_path)?;

    conn.execute(
        "INSERT INTO interventions(predictions_id, timestamp, intervention_type, prediction_label, user_label, dismissed)\
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![interventions.predictions_id, interventions.timestamp, interventions.intervention_type, interventions.prediction_label, interventions.user_label, interventions.dismissed]
    )?;

    let id = conn.last_insert_rowid();
    Ok(id)
}

pub fn insert_user_preference(db_path: &Path, preference: &UserPreferences) -> Result<()> {
    let conn = open_connection(db_path)?;

    conn.execute(
        "INSERT INTO user_preferences(activity_name, min_duration_minutes, max_duration_minutes, times_suggested, times_completed, average_focus_score, last_suggested) \
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![&preference.activity_name, preference.min_duration_minutes, preference.max_duration_minutes, preference.times_suggested, preference.times_completed, preference.average_focus_score, preference.last_suggested]
    )?;

    Ok(())
}

pub fn insert_break_sessions(db_path: &Path, break_sessions: &BreakSessions) -> Result<(i64)> {
    let conn = open_connection(db_path)?;

    conn.execute(
        "INSERT INTO break_sessions(intervention_id, start_timestamp, end_timestamp, preference_id, activity, planned_duration_minutes, returned_on_time, post_break_focus_score) \
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![break_sessions.intervention_id, break_sessions.start_time, break_sessions.end_time, &break_sessions.preference_id, &break_sessions.activity, break_sessions.planned_duration_mins, break_sessions.returned_on_time, break_sessions.post_break_focus_score]
    )?;

    let id = conn.last_insert_rowid();

    Ok(id)
}

pub fn update_break(db_path: &Path, updated_break: EndBreak, end_time: i64) -> Result<()> {
    let conn = open_connection(db_path)?;

    conn.execute("UPDATE break_sessions \
        SET returned_on_time = ?1, end_timestamp = ?2 \
        WHERE id = ?3",
        params![updated_break.returned_on_time, end_time, updated_break.break_session_id]
    )?;

    Ok(())
}

pub fn get_break_plan(db_path: &Path, break_session_id: i64) -> Result<(i64, i64)> {
    let conn = open_connection(db_path)?;

    conn.query_row(
        "SELECT start_timestamp, planned_duration_minutes
         FROM break_sessions
         WHERE id = ?1",
        params![break_session_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
}

pub fn extend_break_planned_duration(db_path: &Path, break_session_id: i64, extra_minutes: i64) -> Result<()> {
    let conn = open_connection(db_path)?;

    conn.execute(
        "UPDATE break_sessions
         SET planned_duration_minutes = planned_duration_minutes + ?1
         WHERE id = ?2",
        params![extra_minutes, break_session_id],
    )?;

    Ok(())
}

pub fn update_user_label(db_path: &Path, updated: &UpdateIntervention) -> Result<()> {
    let conn = open_connection(db_path)?;

    conn.execute(
        "UPDATE interventions \
         SET user_label = ?1, dismissed = ?2 \
         WHERE id=?3",
        params![updated.user_label, updated.dismissed, updated.intervention_id]
    )?;

    Ok(())
}

pub fn prediction_corrected_n_windows(db_path: &Path, timestamp: i64, streak_windows: i64, corrected: bool, ) -> Result<()> {
    let conn = open_connection(db_path)?;

    conn.execute(
        "UPDATE predictions
         SET was_corrected = ?1
         WHERE timestamp <= ?2
         AND timestamp >= (?2 - (?3 * 60) - 30)",
        params![corrected, timestamp, streak_windows],
    )?;

    Ok(())
}

pub fn assign_truth_label(db_path: &Path, feature_vector_id: i64, truth_label: String) -> Result<()> {
    let conn = open_connection(db_path)?;

    conn.execute("UPDATE feature_vectors \
    SET truth_label = ?1 \
    WHERE id = ?2",
    params!(truth_label, feature_vector_id))?;

    Ok(())
}

pub fn get_ids(db_path: &Path, intervention_id: i64) -> Result<(i64, i64)> {
    let conn = open_connection(db_path)?;

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
    let conn = open_connection(db_path)?;
    
    let total: i64 = conn.query_row("SELECT COUNT(*) FROM user_preferences",
                             params![], |row| row.get(0))?;
    
    Ok(total)
}

pub fn get_all_preferences(db_path: &Path) -> Result<Vec<UserPreferences>> {
    let conn = open_connection(db_path)?;

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


pub fn get_recent_predictions(db_path: &Path, limit: i64) -> Result<Vec<PredictionHistoryRow>> {
    let conn = open_connection(db_path)?;

    let mut stmt = conn.prepare(
        "SELECT
            p.id,
            p.timestamp,
            p.predicted_state,
            p.confidence,
            p.was_corrected,
            i.user_label
         FROM predictions p
         LEFT JOIN interventions i ON i.predictions_id = p.id
         ORDER BY p.timestamp DESC
         LIMIT ?1"
    )?;

    let rows = stmt.query_map(params![limit], |row| {
        Ok(PredictionHistoryRow {
            prediction_id: row.get(0)?,
            timestamp: row.get(1)?,
            predicted_state: row.get(2)?,
            confidence: row.get(3)?,
            was_corrected: row.get::<_, i64>(4)? != 0,
            user_label: row.get(5)?,
        })
    })?
        .filter_map(|r| r.ok())
        .collect::<Vec<PredictionHistoryRow>>();

    Ok(rows)
}

pub fn get_user_saved_activities(db_path: &Path) -> Result<Vec<String>> {
    let conn = open_connection(db_path)?;

    let mut stmt = conn.prepare(
        "SELECT activity_name FROM user_preferences ORDER BY id ASC"
    )?;

    let activities = stmt.query_map(params![], |row| {
        row.get(0)
    })?
        .filter_map(|r| r.ok())
        .collect::<Vec<String>>();

    Ok(activities)
}

pub fn get_activity_scores(db_path: &Path) -> Result<Vec<ActivityScore>> {
    let conn = open_connection(db_path)?;

    let mut stmt = conn.prepare(
        "SELECT activity_name, average_focus_score, times_completed, times_suggested
         FROM user_preferences
         ORDER BY average_focus_score DESC"
    )?;

    let rows = stmt.query_map(params![], |row| {
        Ok(ActivityScore {
            activity_name: row.get(0)?,
            average_focus_score: row.get(1)?,
            times_completed: row.get(2)?,
            times_suggested: row.get(3)?,
        })
    })?
        .filter_map(|r| r.ok())
        .collect::<Vec<ActivityScore>>();

    Ok(rows)
}

pub fn update_user_preferences(db_path: &Path, updated_preference: PreferenceUpdate) -> Result<()> {
    let conn = open_connection(db_path)?;
    
    conn.execute(
        "UPDATE user_preferences \
        SET last_suggested = ?1, times_suggested = ?2 \
        WHERE id = ?3",
        params![updated_preference.last_suggested, updated_preference.times_suggested, updated_preference.preference_id],
    )?;
    
    Ok(())
}

pub fn delete_user_preference(db_path: &Path, activity_name: String) -> Result<()> {
    let conn = open_connection(db_path)?;

    conn.execute(
        "DELETE FROM user_preferences WHERE activity_name = ?1",
        params![activity_name],
    )?;

    Ok(())
}

pub fn update_break_focus_score(db_path: &Path, break_id: i64, score: f64) -> Result<()> {
    let conn = open_connection(db_path)?;

    conn.execute(
        "UPDATE break_sessions \
        SET post_break_focus_score = ?1 \
        WHERE id = ?2",
        params![score, break_id],
    )?;

    Ok(())
}

pub fn update_pref_focus_score(db_path: &Path, break_session_id: i64, new_score: f64) -> Result<()> {
    let conn = open_connection(db_path)?;

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
    let conn = open_connection(db_path)?;

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

pub fn update_n_windows_before(db_path: &Path, update_package: IdleFocusedPackage) -> Result<()> {
    let conn = open_connection(db_path)?;

    conn.execute(
        "UPDATE feature_vectors \
        SET truth_label = ?1 \
        WHERE id IN ( \
            SELECT id FROM feature_vectors \
            WHERE timestamp <= ?2 AND timestamp >= (?2 - (?4 * 60) - 30) \
            AND (truth_label IS NULL OR ?3 = 1) \
            ORDER BY timestamp DESC \
            LIMIT ?4\
        )",
        params![update_package.label, update_package.timestamp, update_package.overwrite, update_package.streak_windows as i64]
    )?;

    Ok(())
}



//for checking if enough for retraining
pub fn get_retraining_stats(db_path: &Path) -> Result<(f64, i64)> {
    let conn = open_connection(db_path)?;

    let two_days_ago = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64 - (48 * 60 * 60);

    let total_predictions: i64 = conn.query_row(
        "SELECT COUNT(*) FROM predictions WHERE timestamp >= ?1",
        params![two_days_ago],
        |row| row.get(0)
    )?;

    let correction_rate = if total_predictions == 0 {
        0.0
    } else {
        let corrected: i64 = conn.query_row(
            "SELECT COUNT(*) FROM predictions
             WHERE timestamp >= ?1
             AND was_corrected = 1",
            params![two_days_ago],
            |row| row.get(0)
        )?;
        corrected as f64 / total_predictions as f64
    };

    let labelled_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM feature_vectors
         WHERE truth_label IS NOT NULL
         AND truth_label != 'Break'",
        params![],
        |row| row.get(0)
    )?;

    Ok((correction_rate, labelled_count))
}



//clear input events after retrainng
pub fn clear_old_events(db_path: &Path) -> Result<()> {
    let conn = open_connection(db_path)?;

    let seven_days_ago = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64 - (5 * 24 * 60 * 60);//days * hours * mins * secs

    let deleted = conn.execute(
        "DELETE FROM input_events WHERE timestamp < ?1",
        params![seven_days_ago],
    )?;

    println!("Cleared {} old input_events rows", deleted);
    Ok(())
}


pub fn get_setting(db_path: &Path, key: &str) -> Result<Option<String>> {
    let conn = open_connection(db_path)?;

    let result = conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        params![key],
        |row| row.get(0)
    );

    match result {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn save_setting(db_path: &Path, key: &str, value: &str) -> Result<()> {
    let conn = open_connection(db_path)?;

    conn.execute(
        "INSERT OR REPLACE INTO app_settings(key, value) VALUES(?1, ?2)",
        params![key, value],
    )?;

    Ok(())
}

pub fn get_predictions_count_today(db_path: &Path) -> Result<i64> {
    let conn = open_connection(db_path)?;

    conn.query_row(
        "SELECT COUNT(*) FROM predictions
         WHERE timestamp >= strftime('%s', 'now', 'localtime', 'start of day')",
        params![],
        |row| row.get(0),
    )
}


//analytics n history
pub fn get_prediction_stats(db_path: &Path, since_timestamp: i64) -> Result<StateDistribution> {
    let conn = open_connection(db_path)?;

    // Count each state within the time range
    let focused: i64 = conn.query_row(
        "SELECT COUNT(*) FROM predictions
         WHERE predicted_state = 'Focused'
         AND timestamp >= ?1",
        params![since_timestamp],
        |row| row.get(0)
    )?;

    let at_risk: i64 = conn.query_row(
        "SELECT COUNT(*) FROM predictions
         WHERE predicted_state = 'At Risk'
         AND timestamp >= ?1",
        params![since_timestamp],
        |row| row.get(0)
    )?;

    let procrastinating: i64 = conn.query_row(
        "SELECT COUNT(*) FROM predictions
         WHERE predicted_state = 'Procrastinating'
         AND timestamp >= ?1",
        params![since_timestamp],
        |row| row.get(0)
    )?;

    let idle: i64 = conn.query_row(
        "SELECT COUNT(*) FROM predictions
         WHERE predicted_state = 'Idle'
         AND timestamp >= ?1",
        params![since_timestamp],
        |row| row.get(0)
    )?;

    let total = focused + at_risk + procrastinating + idle;

    // Convert to percentages, handle zero total
    let pct = |count: i64| -> f64 {
        if total == 0 { 0.0 } else { (count as f64 / total as f64) * 100.0 }
    };

    Ok(StateDistribution {
        focused: pct(focused),
        at_risk: pct(at_risk),
        procrastinating: pct(procrastinating),
        idle: pct(idle),
        focused_count: focused,
        at_risk_count: at_risk,
        procrastinating_count: procrastinating,
        idle_count: idle,
        total,
    })
}


pub fn get_focus_score(db_path: &Path, since_timestamp: i64) -> Result<FocusScore> {
    let conn = open_connection(db_path)?;

    // Average confidence of ALL predictions in range
    let avg_confidence: f64 = conn.query_row(
        "SELECT COALESCE(AVG(confidence), 0.0)
         FROM predictions
         WHERE timestamp >= ?1",
        params![since_timestamp],
        |row| row.get(0)
    )?;

    // Total predictions and focused count
    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM predictions WHERE timestamp >= ?1",
        params![since_timestamp],
        |row| row.get(0)
    )?;

    let focused_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM predictions
         WHERE predicted_state = 'Focused'
         AND timestamp >= ?1",
        params![since_timestamp],
        |row| row.get(0)
    )?;

    let focused_percentage = if total == 0 {
        0.0
    } else {
        (focused_count as f64 / total as f64) * 100.0
    };

    // Focus score is weighted combination:
    // 70% how often you were focused, 30% average confidence
    // Both scaled to 0-100
    let score = (focused_percentage * 0.7) + (avg_confidence * 100.0 * 0.3);

    Ok(FocusScore {
        score,
        average_confidence: avg_confidence,
        focused_percentage,
    })
}


pub fn get_prediction_history(db_path: &Path, since_timestamp: i64, state_filter: Option<String>, limit: i64) -> Result<Vec<PredictionHistoryRow>> {
    let conn = open_connection(db_path)?;

    // Build query dynamically based on whether a state filter is applied
    // We LEFT JOIN interventions to get the user_label if a correction was made
    let rows = if let Some(state) = state_filter {
        let mut stmt = conn.prepare(
            "SELECT
                p.id,
                p.timestamp,
                p.predicted_state,
                p.confidence,
                p.was_corrected,
                i.user_label
             FROM predictions p
             LEFT JOIN interventions i ON i.predictions_id = p.id
             WHERE p.timestamp >= ?1
             AND p.predicted_state = ?2
             ORDER BY p.timestamp DESC
             LIMIT ?3"
        )?;

        let mapped_rows = stmt.query_map(params![since_timestamp, state, limit], |row| {
            Ok(PredictionHistoryRow {
                prediction_id: row.get(0)?,
                timestamp: row.get(1)?,
                predicted_state: row.get(2)?,
                confidence: row.get(3)?,
                was_corrected: row.get::<_, i64>(4)? != 0,
                user_label: row.get(5)?,
            })
        })?;

        mapped_rows
            .filter_map(|r| r.ok())
            .collect::<Vec<PredictionHistoryRow>>()
    } else {
        let mut stmt = conn.prepare(
            "SELECT
                p.id,
                p.timestamp,
                p.predicted_state,
                p.confidence,
                p.was_corrected,
                i.user_label
             FROM predictions p
             LEFT JOIN interventions i ON i.predictions_id = p.id
             WHERE p.timestamp >= ?1
             ORDER BY p.timestamp DESC
             LIMIT ?2"
        )?;

        let mapped_rows = stmt.query_map(params![since_timestamp, limit], |row| {
            Ok(PredictionHistoryRow {
                prediction_id: row.get(0)?,
                timestamp: row.get(1)?,
                predicted_state: row.get(2)?,
                confidence: row.get(3)?,
                was_corrected: row.get::<_, i64>(4)? != 0,
                user_label: row.get(5)?,
            })
        })?;

        mapped_rows
            .filter_map(|r| r.ok())
            .collect::<Vec<PredictionHistoryRow>>()
    };

    Ok(rows)
}




