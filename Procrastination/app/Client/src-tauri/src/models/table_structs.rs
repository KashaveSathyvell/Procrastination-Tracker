use serde::Serialize;

#[derive(Debug)]
pub struct Input {
    pub timestamp: i64,
    pub event_type: String,
    pub event_action: String,
    pub key_code: Option<String>,
    pub mouse_x: Option<f64>,
    pub mouse_y: Option<f64>,
    pub wheel_x: Option<i64>,
    pub wheel_y: Option<i64>,
    pub button: Option<String>,
    pub active_window: String,
}

#[derive(Debug)]
pub struct FeatureVectors {
    pub timestamp: i64,
    pub typing_speed: f64,
    pub repetitive_key_ratio: f64,
    pub mouse_velocity: f64,
    pub idle_ratio: f64,
    pub window_switch_frequency: f64,
}

#[derive(Debug)]
pub struct Predictions {
    pub feature_vectors_id: i64,
    pub timestamp: i64,
    pub predicted_state: String,
    pub confidence: f64,
    pub window_size_seconds: i64,
    pub was_corrected: bool,
}


#[derive(Debug, Serialize, Clone)]
pub struct Interventions {
    pub predictions_id: i64,
    pub timestamp: i64,
    pub intervention_type: String,
    pub prediction_label: String,
    pub user_label: Option<String>,
    pub dismissed: bool,
}