use crate::models::table_structs::UserPreferences;

pub fn break_activities() -> Vec<UserPreferences> {
    vec![
        UserPreferences { id: Some(0), activity_name: "Walking".to_string(), min_duration_minutes: 5, max_duration_minutes: 10, times_suggested: 0, times_completed: 0, average_focus_score: 0.0, last_suggested: 0 },
        UserPreferences { id: Some(0), activity_name: "Light Exercise".to_string(), min_duration_minutes: 10, max_duration_minutes: 20, times_suggested: 0, times_completed: 0, average_focus_score: 0.0, last_suggested: 0 },
        UserPreferences { id: Some(0), activity_name: "Gaming".to_string(), min_duration_minutes: 10, max_duration_minutes: 25, times_suggested: 0, times_completed: 0, average_focus_score: 0.0, last_suggested: 0 },
        UserPreferences { id: Some(0), activity_name: "Reading".to_string(), min_duration_minutes: 10, max_duration_minutes: 20, times_suggested: 0, times_completed: 0, average_focus_score: 0.0, last_suggested: 0 },
        UserPreferences { id: Some(0), activity_name: "Meditation".to_string(), min_duration_minutes: 5, max_duration_minutes: 10, times_suggested: 0, times_completed: 0, average_focus_score: 0.0, last_suggested: 0 },
        UserPreferences { id: Some(0), activity_name: "Stretching".to_string(), min_duration_minutes: 5, max_duration_minutes: 10, times_suggested: 0, times_completed: 0, average_focus_score: 0.0, last_suggested: 0 },
        UserPreferences { id: Some(0), activity_name: "Crochet".to_string(), min_duration_minutes: 15, max_duration_minutes: 20, times_suggested: 0, times_completed: 0, average_focus_score: 0.0, last_suggested: 0 },
        UserPreferences { id: Some(0), activity_name: "Snack break".to_string(), min_duration_minutes: 5, max_duration_minutes: 10, times_suggested: 0, times_completed: 0, average_focus_score: 0.0, last_suggested: 0 },
        UserPreferences { id: Some(0), activity_name: "Youtube".to_string(), min_duration_minutes: 10, max_duration_minutes: 15, times_suggested: 0, times_completed: 0, average_focus_score: 0.0, last_suggested: 0 },
        UserPreferences { id: Some(0), activity_name: "Scrolling reels".to_string(), min_duration_minutes: 5, max_duration_minutes: 10, times_suggested: 0, times_completed: 0, average_focus_score: 0.0, last_suggested: 0 },
    ]
}