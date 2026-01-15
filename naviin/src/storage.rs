use crate::AppState::AppState;
use std::{fs, sync::Arc, sync::Mutex};
use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait, IntoActiveModel};
use super::entities::app_state::Entity as AppStateEntity;

const STATE_PATH: &str = "state.json";

pub fn username_checker(username: &String) -> bool {
    println!("Validating username: {username} against storage");
    true
}

pub async fn save_state(state: &Arc<Mutex<AppState>>, db: &DatabaseConnection) {
    // No cloning of arc mutex needed here, only required for threads
    let state_guard = state.lock().unwrap();
    match AppStateEntity::find_by_id(1).one(db).await {
        Ok(Some(model)) => {
            let mut active_model = model.into_active_model();
            active_model.cash_balance = Set(state_guard.get_available_cash());
            active_model.updated_at = Set(chrono::Utc::now().timestamp());
            if let Err(e) = active_model.update(db).await {
                eprintln!("Failed to update database: {e}");
            }
            // todo: save the other fields as needed
        },
        Ok(None) => {eprintln!("Record not found")}
        Err(e) => {eprintln!("Database error: {e}.")},
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

pub async fn default_state(state: &Arc<Mutex<AppState>>, db: &DatabaseConnection) {
    {
        let mut state_guard = state.lock().unwrap();
        *state_guard = AppState::new();
    }
    save_state(state, db).await;
}
