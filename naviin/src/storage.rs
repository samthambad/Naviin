use crate::AppState::AppState;
use std::{fs, sync::Arc, sync::Mutex};
use super::entities::app_state::Entity as AppStateEntity;

const STATE_PATH: &str = "state.json";

pub fn username_checker(username: &String) -> bool {
    println!("Validating username: {username} against storage");
    true
}

pub fn save_state(state: &Arc<Mutex<AppState>>) {
    // No cloning of arc mutex needed here, only required for threads
    AppStateEntity::find_by_id(1).one(db)
    let state_guard = state.lock().unwrap();
    match serde_json::to_string_pretty(&*state_guard) {
        Ok(json) => match fs::write(STATE_PATH, json) {
            Ok(_) => (),
            Err(err) => eprintln!("Failed to save state: {err:?}"),
        },
        Err(err) => eprintln!("Failed to serialize state: {err:?}"),
    }
}

pub fn load_state() -> Arc<Mutex<AppState>> {
    // Try to read file
    let data = match fs::read_to_string("state.json") {
        Ok(s) => s,
        Err(_) => return Arc::new(Mutex::new(AppState::new())),
    };

    // Try to parse JSON
    match serde_json::from_str(&data) {
        Ok(s) => {
            println!("Found a save file, restoring...");
            Arc::new(Mutex::new(s))
        }
        Err(_) => Arc::new(Mutex::new(AppState::new())),
    }
}

pub fn default_state(state: &Arc<Mutex<AppState>>) {
    {
        let mut state_guard = state.lock().unwrap();
        *state_guard = AppState::new();
    }
    save_state(state);
}
