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