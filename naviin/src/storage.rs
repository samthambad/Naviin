use crate::AppState::AppState;
use crate::Orders::{OpenOrder, Trade, Side};
use std::{fs, sync::Arc, sync::Mutex};
use sea_orm::{ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, Set, ActiveModelTrait, IntoActiveModel, NotSet, TransactionTrait, DbErr, QuerySelect, QueryFilter};
use super::entities::app_state::Entity as AppStateEntity;
use super::entities::holding::Entity as HoldingEntity;
use super::entities::holding::ActiveModel as HoldingActiveModel;
use super::entities::holding::Column as HoldingColumn;
use super::entities::trade::ActiveModel as TradeActiveModel;
use super::entities::open_order::ActiveModel as OpenOrderActiveModel;


const STATE_PATH: &str = "state.json";

async fn sync_holdings(txn: &DatabaseTransaction, holdings: &[(String, rust_decimal::Decimal, rust_decimal::Decimal)]) -> Result<(), DbErr> {
    for (symbol, quantity, avg_price) in holdings {
        let existing = HoldingEntity::find()
            .filter(HoldingColumn::Symbol.eq(symbol))
            .one(txn)
            .await?;
        
        if let Some(model) = existing {
            let mut active_model = model.into_active_model();
            active_model.quantity = Set(*quantity);
            active_model.avg_cost = Set(*avg_price);
            active_model.update(txn).await?;
        } else {
            let holding = HoldingActiveModel {
                id: NotSet,
                symbol: Set(symbol.clone()),
                quantity: Set(*quantity),
                avg_cost: Set(*avg_price),
            };
            holding.insert(txn).await?;
        }
    }
    Ok(())
}

async fn sync_trades(txn: &DatabaseTransaction, trades: &[Trade]) -> Result<(), DbErr> {
    for trade in trades {
        let side_str = match trade.get_side(){
            Side::Buy => "Buy",
            Side::Sell => "Sell",
        };
        let db_trade = TradeActiveModel {
            id: NotSet,
            symbol: Set(trade.get_symbol().clone()),
            quantity: Set(trade.get_quantity()),
            price_per: Set(trade.get_price_per()),
            side: Set(side_str.to_string()),
            order_type: Set("Market".to_string()),
            timestamp: Set(trade.get_timestamp()),
        };
        db_trade.insert(txn).await?;
    }
    Ok(())
}

async fn sync_open_orders(txn: &DatabaseTransaction, open_orders: &[OpenOrder]) -> Result<(), DbErr> {
    for open_order in open_orders {
        let (order_type_str, symbol, quantity, price, timestamp) = match open_order {
            OpenOrder::BuyLimit { symbol, quantity, price, timestamp } => {
                ("BuyLimit", symbol, quantity, price, timestamp)
            }
            OpenOrder::StopLoss { symbol, quantity, price, timestamp } => {
                ("StopLoss", symbol, quantity, price, timestamp)
            }
            OpenOrder::TakeProfit { symbol, quantity, price, timestamp } => {
                ("TakeProfit", symbol, quantity, price, timestamp)
            }
        };
        let db_order = OpenOrderActiveModel {
            id: NotSet,
            order_type: Set(order_type_str.to_string()),
            symbol: Set(symbol.clone()),
            quantity: Set(*quantity),
            price: Set(*price),
            timestamp: Set(*timestamp),
        };
        db_order.insert(txn).await?;
    }
    Ok(())
}

pub fn username_checker(username: &String) -> bool {
    println!("Validating username: {username} against storage");
    true
}

pub async fn save_state(state: &Arc<Mutex<AppState>>, db: &DatabaseConnection) {
    // No cloning of arc mutex needed here, only required for threads
    // get relevant data first to not block more than required
    let (cash, current_holdings, trades, open_orders) = {
        let state_guard = state.lock().unwrap();
        let cash = state_guard.get_available_cash();

        // Collect holdings into a vector of simple data tuples
        let holdings = state_guard.get_holdings_map()
            .iter()
            .map(|(symbol, holding)| (symbol.clone(), holding.get_qty(), holding.get_avg_price()))
            .collect::<Vec<_>>();

        let trades = state_guard.get_trades();

        let open_orders = state_guard.get_open_orders();
        (cash, holdings, trades, open_orders)
    };

    let txn_result = db.transaction::<_,_, DbErr>(|txn| Box::pin(async move {
        let app_state_opt = AppStateEntity::find_by_id(1).one(txn).await?;
        if let Some(model) = app_state_opt {
            let mut active_model = model.into_active_model();
            active_model.cash_balance = Set(cash);
            active_model.updated_at = Set(chrono::Utc::now().timestamp());
            active_model.update(txn).await?;
        } else {
            eprintln!("AppState row not found during saving")
        }
        
        sync_holdings(txn, &current_holdings).await?;
        sync_trades(txn, &trades).await?;
        sync_open_orders(txn, &open_orders).await?;

        Ok(())
    })).await;


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
