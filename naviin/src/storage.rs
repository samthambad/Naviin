use crate::AppState::AppState;
use std::fs;

const STATE_PATH: &str = "state.json";

pub fn username_checker(username: &String) -> bool {
    println!("Validating username: {username} against storage");
    true
}

pub fn save_state(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(state)?;
    fs::write(STATE_PATH, json)?;
    Ok(())
}

// pub fn load_state() -> AppState {}

// pub fn default_state() -> AppState {}
