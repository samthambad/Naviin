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

pub fn load_state() -> AppState {
    // only runs after login
    let data =
        fs::read_to_string("state.json").unwrap_or_else(|_| panic!("Could not read state.json"));
    serde_json::from_str(&data).unwrap_or_else(|_| panic!("Invalid state.json format"))
}

pub fn default_state() -> AppState {}
