use crate::AppState::AppState;
use std::{fs, io, path::Path};

const STATE_PATH: &str = "state.json";

pub fn username_checker(username: &String) -> bool {
    println!("Validating username: {username} against storage");
    true
}

pub fn save_state(state: &AppState) {}

pub fn load_state() -> AppState {}

pub fn default_state() -> AppState {}
