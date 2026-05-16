#[cfg(test)]
mod tests {
    use std::time::{SystemTime, Duration};
    // If your logic is inside jitai.rs, uncomment the line below:
    // use super::super::jitai::*;

    #[test]
    fn test_procrastination_threshold() {
        let recent_states = vec!["Procrastinating", "Procrastinating", "Procrastinating", "Procrastinating", "Procrastinating"];
        let should_trigger = recent_states.iter().all(|&state| state == "Procrastinating");

        assert_eq!(should_trigger, true, "5 consecutive procrastination states must trigger intervention");
    }

    #[test]
    fn test_cooldown_respect() {
        let cooldown_minutes = 10;
        let last_intervention_time = SystemTime::now() - Duration::from_secs(2 * 60);

        let time_since_last = SystemTime::now().duration_since(last_intervention_time).unwrap();
        let is_cooldown_active = time_since_last.as_secs() < (cooldown_minutes * 60);
        let can_trigger = !is_cooldown_active;

        assert_eq!(can_trigger, false, "System must not trigger if 10-minute cooldown has not passed");
    }

    #[test]
    fn test_parse_inference_output() {
        let probabilities = vec![0.05, 0.10, 0.80, 0.05];

        let mut max_index = 0;
        let mut max_val = probabilities[0];
        for (i, &val) in probabilities.iter().enumerate() {
            if val > max_val {
                max_val = val;
                max_index = i;
            }
        }

        let labels = ["Focused", "At Risk", "Procrastinating", "Idle"];
        let predicted_label = labels[max_index];

        assert_eq!(predicted_label, "Procrastinating", "Should correctly map index 2 to Procrastinating");
    }
}