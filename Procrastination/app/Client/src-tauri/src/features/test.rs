#[cfg(test)]
mod tests {
    use crate::features::feature_extractor::extract_features;
    use crate::models::table_structs::Input;

    fn make_key_event(timestamp: i64, key: &str) -> Input {
        Input {
            timestamp,
            event_type: "Keyboard".to_string(),
            event_action: "KeyPress".to_string(),
            key_code: Some(key.to_string()),
            mouse_x: None,
            mouse_y: None,
            wheel_x: None,
            wheel_y: None,
            button: None,
            active_window: "TestApp".to_string(),
        }
    }

    fn make_mouse_event(timestamp: i64, x: f64, y: f64) -> Input {
        Input {
            timestamp,
            event_type: "Mouse".to_string(),
            event_action: "MouseMove".to_string(),
            key_code: None,
            mouse_x: Some(x),
            mouse_y: Some(y),
            wheel_x: None,
            wheel_y: None,
            button: None,
            active_window: "TestApp".to_string(),
        }
    }

    #[test]
    fn test_empty_events_gives_full_idle() {
        let features = extract_features(vec![], 0, 60);
        assert_eq!(features.idle_ratio, 1.0,
                   "Empty event window should be fully idle");
        assert_eq!(features.typing_speed, 0.0);
        assert_eq!(features.mouse_velocity, 0.0);
        assert_eq!(features.repetitive_key_ratio, 0.0);
    }

    #[test]
    fn test_typing_speed_calculation() {
        // 60 keypresses over a 60-second window → speed = 1.0
        let events: Vec<Input> = (0..60)
            .map(|i| make_key_event(i, "a"))
            .collect();
        let features = extract_features(events, 0, 60);
        assert!(
            (features.typing_speed - 1.0).abs() < 0.001,
            "60 keypresses / 60s should give typing_speed ~1.0, got {}",
            features.typing_speed
        );
    }

    #[test]
    fn test_repetitive_key_ratio_all_same_key() {
        // All same key → repetitive_ratio should be close to 1.0
        let events: Vec<Input> = (0..10)
            .map(|i| make_key_event(i, "a"))
            .collect();
        let features = extract_features(events, 0, 60);
        assert!(
            features.repetitive_key_ratio > 0.8,
            "All-same keypresses should give high repetitive ratio, got {}",
            features.repetitive_key_ratio
        );
    }

    #[test]
    fn test_mouse_velocity_stationary() {
        // Mouse not moving → velocity should be 0
        let events: Vec<Input> = (0..5)
            .map(|i| make_mouse_event(i * 10, 100.0, 100.0))
            .collect();
        let features = extract_features(events, 0, 60);
        assert_eq!(features.mouse_velocity, 0.0,
                   "Stationary mouse should give velocity 0");
    }

    #[test]
    fn test_mouse_velocity_moving() {
        // Mouse moves 300px right over the window
        let events = vec![
            make_mouse_event(0,  0.0, 0.0),
            make_mouse_event(30, 300.0, 0.0),
        ];
        let features = extract_features(events, 0, 60);
        // Distance = 300, velocity = 300/60 = 5.0
        assert!(
            (features.mouse_velocity - 5.0).abs() < 0.001,
            "Expected mouse_velocity ~5.0, got {}",
            features.mouse_velocity
        );
    }

    #[test]
    fn test_idle_ratio_capped_at_one() {
        // Even if gap calculations exceed 60s, ratio must not exceed 1.0
        let events = vec![make_key_event(30, "a")]; // one event in the middle
        let features = extract_features(events, 0, 60);
        assert!(
            features.idle_ratio <= 1.0,
            "Idle ratio must never exceed 1.0, got {}",
            features.idle_ratio
        );
    }
}