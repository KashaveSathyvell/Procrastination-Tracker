use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use ort::session::Session;
use serde::Serialize;

#[derive(Debug)]
pub struct ThreadStop {
    pub running_collect: Arc<AtomicBool>,
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


#[derive(Debug)]
pub struct UpdateIntervention {
    pub intervention_id: i64,
    pub user_label: String, 
    pub dismissed: bool,
}