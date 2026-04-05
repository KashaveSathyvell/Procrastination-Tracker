#[derive(Debug)]
pub struct FeatureVector {
    pub typing_speed: f32,
    pub mouse_velocity: f32,
    pub idle_ratio: f32,
    pub window_switch_frequency: f32,
}

#[derive(Debug)]
pub struct InputEvent {
    pub timestamp: i64,
    pub event_type: String,
    pub event_action: String,
    pub delta_x: Option<i32>,
    pub delta_y: Option<i32>,
    pub active_window: Option<String>,
}

pub fn compute_features(mut events: Vec<InputEvent>, window_seconds: i64) -> FeatureVector {
    // Ensure events are sorted (VERY IMPORTANT)
    events.sort_by_key(|e| e.timestamp);

    // Edge Case: no activity at all
    if events.is_empty() {
        return FeatureVector {
            typing_speed: 0.0,
            mouse_velocity: 0.0,
            idle_ratio: 1.0,
            window_switch_frequency: 0.0,
        };
    }

    let mut key_presses = 0;
    let mut total_mouse_distance = 0.0;
    let mut idle_time = 0;
    let mut switches = 0;

    let idle_threshold = 2; // seconds

    let mut prev_timestamp = events[0].timestamp;
    let mut prev_window = events[0].active_window.clone().unwrap_or_default();

    for event in &events {
        // -------------------------
        // 1. Idle Time Calculation
        // -------------------------
        let gap = event.timestamp - prev_timestamp;
        if gap > idle_threshold {
            idle_time += gap;
        }
        prev_timestamp = event.timestamp;

        // -------------------------
        // 2. Keyboard Events
        // -------------------------
        if event.event_type == "keyboard" && event.event_action == "down" {
            key_presses += 1;
        }

        // -------------------------
        // 3. Mouse Movement
        // -------------------------
        if event.event_type == "mouse" && event.event_action == "move" {
            let dx = event.delta_x.unwrap_or(0) as f32;
            let dy = event.delta_y.unwrap_or(0) as f32;

            let distance = (dx * dx + dy * dy).sqrt();
            total_mouse_distance += distance;
        }

        // -------------------------
        // 4. Window Switching
        // -------------------------
        if let Some(current_window) = &event.active_window {
            if current_window != &prev_window {
                switches += 1;
                prev_window = current_window.clone();
            }
        }
    }

    let window_f32 = window_seconds as f32;

    FeatureVector {
        typing_speed: key_presses as f32 / window_f32,
        mouse_velocity: total_mouse_distance / window_f32,

        // Clamp to avoid invalid values
        idle_ratio: (idle_time as f32 / window_f32).clamp(0.0, 1.0),

        window_switch_frequency: switches as f32 / window_f32,
    }
}