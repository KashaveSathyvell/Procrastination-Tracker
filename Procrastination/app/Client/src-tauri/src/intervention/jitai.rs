use std::path::Path;
use chrono::Utc;
use rand::seq::IndexedRandom;
use rand::{Rng, RngExt};

use crate::models::table_structs::UserPreferences;
use crate::models::models::{ActivitySuggestion, PreferenceUpdate};
use crate::database::sqlite::{get_all_preferences, update_user_preferences};

pub fn suggest_activity(db_path: &Path) -> ActivitySuggestion {

    let preference_list = get_all_preferences(db_path).unwrap();
    let mut pref_list_2 = preference_list.clone();

    let mut chosen_activity: Option<UserPreferences> = None;

    let last = preference_list.iter()
        .max_by_key(|a| a.last_suggested);

    let mut untried_activities: Vec<UserPreferences> = preference_list.iter()
        .filter(|a| a.times_suggested == 0).cloned()
        .collect();

    if untried_activities.len() > 1 {
        if let Some(last_activity) = last {
            untried_activities.retain(|a| a.id != last_activity.id);
        }
    }

    if !untried_activities.is_empty() {
        chosen_activity = Some(untried_activities.choose(&mut rand::rng()).unwrap().clone());
    } else {
        let roll: f64 = rand::rng().random();

        if pref_list_2.len() > 1 {
            if let Some(last_activity) = last {
                pref_list_2.retain(|a| a.id != last_activity.id);
            }
        }

        if roll < 0.7 {
            let best = pref_list_2.iter()
                .max_by(|a, b| a.average_focus_score.partial_cmp(&b.average_focus_score).unwrap_or(std::cmp::Ordering::Equal));
            chosen_activity = Some(best.unwrap().clone());
        }
        else {
            // if pref_list_2.len() > 1 {
            //     if let Some(last_activity) = last {
            //         pref_list_2.retain(|a| a.id != last_activity.id);
            //     }
            // }
            chosen_activity = Some(pref_list_2.choose(&mut rand::rng()).unwrap().clone());
        }
    }

    let chosen = chosen_activity.unwrap();
    
    let updated_preference = PreferenceUpdate {
        preference_id: chosen.id.clone().expect("Preference.id should exist here"),
        times_suggested: chosen.times_suggested + 1,
        last_suggested: Utc::now().timestamp(),
    };
    
    update_user_preferences(db_path, updated_preference).expect("TODO: panic message");

    let random_duration = rand::rng().random_range(chosen.min_duration_minutes..=chosen.max_duration_minutes);

    ActivitySuggestion {
        preference_id: chosen.id.expect("Preference.id should exist here"),
        activity: chosen.activity_name.clone(),
        random_duration,
    }


    
}