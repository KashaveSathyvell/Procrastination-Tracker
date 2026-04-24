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