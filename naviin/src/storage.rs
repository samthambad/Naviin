use crate::AppState::AppState;
use std::{fs, sync::Arc, sync::Mutex};
use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait, IntoActiveModel, NotSet, TransactionTrait, DbErr};
use migration::OnConflict;
use super::entities::app_state::Entity as AppStateEntity;
use super::entities::holding::Entity as HoldingEntity;
use super::entities::holding::ActiveModel as HoldingActiveModel;
use super:: entities::holding::Column as HoldingColumn;


const STATE_PATH: &str = "state.json";

pub fn username_checker(username: &String) -> bool {
    println!("Validating username: {username} against storage");
    true
}

pub async fn save_state(state: &Arc<Mutex<AppState>>, db: &DatabaseConnection) {
    // No cloning of arc mutex needed here, only required for threads
    // get relevant data first to not block more than required
    let (cash, current_holdings) = {
        let state_guard = state.lock().unwrap();
        let cash = state_guard.get_available_cash();

        // Collect holdings into a vector of simple data tuples
        let holdings = state_guard.get_holdings_map()
            .iter()
            .map(|(symbol, holding)| (symbol.clone(), holding.get_qty(), holding.get_avg_price()))
            .collect::<Vec<_>>();

        (cash, holdings)
    };

    let txn_result = db.transaction::<_,_, DbErr>(|txn|)


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
