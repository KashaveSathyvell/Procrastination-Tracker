use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::thread::JoinHandle;
use ort::session::Session;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ThreadStop {
    pub running_collect: Arc<AtomicBool>,
    pub handles: Mutex<Option<Vec<JoinHandle<()>>>>,
}

#[derive(Debug)]
pub struct OnBreak {
    pub on_break: Arc<AtomicBool>,
    pub break_ended: Arc<AtomicBool>,
    pub break_id: Arc<Mutex<Option<i64>>>,
}

#[derive(Debug)]
pub struct ModelState {
    pub session: Arc<Mutex<Session>>,
}

#[derive(Serialize, Clone)]
pub struct PredictionPackage {
    pub prediction_id: i64,
    pub feature_vector_id: i64,
    pub prediction_label: String,
    pub confidence: f64,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct InterventionPackage {
    pub intervention_id: i64,
    pub timestamp: i64,
    pub intervention_type: String,
    pub prediction_label: String,
    pub confidence: f64,
    pub suggested_activity: Option<String>, 
    pub suggested_duration: Option<i64>,
    pub preference_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIntervention {
    pub timestamp: i64,
    pub intervention_id: i64,
    pub user_label: String,
    pub dismissed: bool,
    pub predicted_label: String,
}


#[derive(Debug)]
pub struct ActivitySuggestion {
    pub preference_id: i64,
    pub activity: String,
    pub random_duration: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct ActivityScore {
    pub activity_name: String,
    pub average_focus_score: f64,
    pub times_completed: i64,
    pub times_suggested: i64,
}

#[derive(Debug)]
pub struct PreferenceUpdate {
    pub preference_id: i64,
    pub times_suggested: i64,
    pub last_suggested: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndBreak {
    pub break_session_id: i64,
    pub returned_on_time: bool,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IdleFocusedPackage {
    pub timestamp: i64,
    pub streak_windows: i32,
    pub label: String,
    pub overwrite: bool,
}


#[derive(Debug, Serialize)]
pub struct RetrainingStats {
    pub correction_rate: f64,
    pub labelled_count: i64,
    pub retraining_needed: bool,
}

#[derive(Debug, Serialize)]
pub struct RetrainingResult {
    pub success: bool,
    pub message: String,
}


//analytics n history
#[derive(Debug, Serialize, Clone)]
pub struct StateDistribution {
    pub focused: f64,
    pub at_risk: f64,
    pub procrastinating: f64,
    pub idle: f64,
    pub focused_count: i64,
    pub at_risk_count: i64,
    pub procrastinating_count: i64,
    pub idle_count: i64,
    pub total: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct FocusScore {
    pub score: f64,           // 0-100
    pub average_confidence: f64,
    pub focused_percentage: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct PredictionHistoryRow {
    pub prediction_id: i64,
    pub timestamp: i64,
    pub predicted_state: String,
    pub confidence: f64,
    pub was_corrected: bool,
    pub user_label: Option<String>,
}