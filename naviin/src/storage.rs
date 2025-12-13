use crate::AppState::AppState;
use std::fs;

const STATE_PATH: &str = "state.json";

pub fn username_checker(username: &String) -> bool {
    println!("Validating username: {username} against storage");
    true
}

pub fn save_state(state: &AppState) {
    match serde_json::to_string_pretty(state) {
        Ok(json) => match fs::write(STATE_PATH, json) {
            Ok(_) => (),
            Err(err) => eprintln!("Failed to save state: {err:?}"),
        },
        Err(err) => eprintln!("Failed to serialize state: {err:?}"),
    }
}

pub fn load_state() -> AppState {
    // Try to read file
    let data = match fs::read_to_string("state.json") {
        Ok(s) => s,
        Err(_) => return AppState::new(),
    };

    // Try to parse JSON
    match serde_json::from_str(&data) {
        Ok(s) => {
            println!("Found a save file, restoring...");
            s
        }
        Err(_) => AppState::new(),
    }
}

pub fn default_state(state: &mut AppState) {
    *state = AppState::new();
    save_state(state);
}
