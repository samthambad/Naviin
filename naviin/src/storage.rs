use super::entities::app_state::ActiveModel as AppStateActiveModel;
use super::entities::app_state::Entity as AppStateEntity;
use super::entities::holding::ActiveModel as HoldingActiveModel;
use super::entities::holding::Column as HoldingColumn;
use super::entities::holding::Entity as HoldingEntity;
use super::entities::open_order::ActiveModel as OpenOrderActiveModel;
use super::entities::open_order::Entity as OpenOrderEntity;
use super::entities::trade::ActiveModel as TradeActiveModel;
use super::entities::trade::Entity as TradeEntity;
use super::entities::watchlist::ActiveModel as WatchlistActiveModel;
use super::entities::watchlist::Entity as WatchlistEntity;
use crate::AppState::AppState;
use crate::Finance::{Holding, Symbol};
use crate::Orders::{OpenOrder, Side, Trade};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, DatabaseTransaction, DbErr,
    EntityTrait, IntoActiveModel, NotSet, QueryFilter, Set, TransactionTrait,
};
use std::{collections::HashMap, env, sync::Arc, sync::Mutex};

async fn load_app_state(db: &DatabaseConnection) -> Result<Option<rust_decimal::Decimal>, DbErr> {
    match AppStateEntity::find_by_id(1).one(db).await? {
        Some(model) => Ok(Some(model.cash_balance)),
        None => Ok(None),
    }
}

/// Loads all holdings from the database into a HashMap keyed by symbol.
async fn load_holdings(db: &DatabaseConnection) -> Result<HashMap<String, Holding>, DbErr> {
    let holdings_models = HoldingEntity::find().all(db).await?;
    let mut holdings_map: HashMap<String, Holding> = HashMap::new();
    for h in holdings_models {
        let holding = Holding::new(h.symbol.clone(), h.quantity, h.avg_cost);
        holdings_map.insert(h.symbol, holding);
    }
    Ok(holdings_map)
}

async fn load_trades(db: &DatabaseConnection) -> Result<Vec<Trade>, DbErr> {
    let trades_models = TradeEntity::find().all(db).await?;
    let trades: Vec<Trade> = trades_models
        .into_iter()
        .map(|t| {
            let side = match t.side.as_str() {
                "Buy" => Side::Buy,
                "Sell" => Side::Sell,
                _ => panic!("Unknown trade side: {}", t.side),
            };
            Trade::from_database(t.symbol, t.quantity, t.price_per, side, t.timestamp, t.order_type)
        })
        .collect();
    Ok(trades)
}

async fn load_open_orders(db: &DatabaseConnection) -> Result<Vec<OpenOrder>, DbErr> {
    let open_orders_models = OpenOrderEntity::find().all(db).await?;
    let open_orders: Vec<OpenOrder> = open_orders_models
        .into_iter()
        .map(|o| match o.order_type.as_str() {
            "BuyLimit" => OpenOrder::BuyLimit {
                symbol: o.symbol,
                quantity: o.quantity,
                price: o.price,
                timestamp: o.timestamp,
            },
            "StopLoss" => OpenOrder::StopLoss {
                symbol: o.symbol,
                quantity: o.quantity,
                price: o.price,
                timestamp: o.timestamp,
            },
            "TakeProfit" => OpenOrder::TakeProfit {
                symbol: o.symbol,
                quantity: o.quantity,
                price: o.price,
                timestamp: o.timestamp,
            },
            _ => panic!("Unknown order type: {}", o.order_type),
        })
        .collect();
    Ok(open_orders)
}

/// Synchronizes the holdings in the database with the provided list, updating existing or inserting new ones.
async fn sync_holdings(
    txn: &DatabaseTransaction,
    holdings: &[(String, rust_decimal::Decimal, rust_decimal::Decimal)],
) -> Result<(), DbErr> {
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

/// Synchronizes the trades in the database, inserting new ones if they don't exist.
async fn sync_trades(txn: &DatabaseTransaction, trades: &[Trade]) -> Result<(), DbErr> {
    let existing_trades = TradeEntity::find().all(txn).await?;

    for trade in trades {
        let side_str = match trade.get_side() {
            Side::Buy => "Buy",
            Side::Sell => "Sell",
        };

        let already_exists = existing_trades.iter().any(|t| {
            t.symbol == *trade.get_symbol()
                && t.quantity == trade.get_quantity()
                && t.price_per == trade.get_price_per()
                && t.side == side_str
                && t.timestamp == trade.get_timestamp()
        });

        if !already_exists {
            let db_trade = TradeActiveModel {
                id: NotSet,
                symbol: Set(trade.get_symbol().clone()),
                quantity: Set(trade.get_quantity()),
                price_per: Set(trade.get_price_per()),
                side: Set(side_str.to_string()),
                order_type: Set(trade.get_order_type().clone()),
                timestamp: Set(trade.get_timestamp()),
            };
            db_trade.insert(txn).await?;
        }
    }
    Ok(())
}

/// Synchronizes the open orders in the database by deleting all and re-inserting.
async fn sync_open_orders(
    txn: &DatabaseTransaction,
    open_orders: &[OpenOrder],
) -> Result<(), DbErr> {
    OpenOrderEntity::delete_many().exec(txn).await?;

    for open_order in open_orders {
        let (order_type_str, symbol, quantity, price, timestamp) = match open_order {
            OpenOrder::BuyLimit {
                symbol,
                quantity,
                price,
                timestamp,
            } => ("BuyLimit", symbol, quantity, price, timestamp),
            OpenOrder::StopLoss {
                symbol,
                quantity,
                price,
                timestamp,
            } => ("StopLoss", symbol, quantity, price, timestamp),
            OpenOrder::TakeProfit {
                symbol,
                quantity,
                price,
                timestamp,
            } => ("TakeProfit", symbol, quantity, price, timestamp),
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

async fn load_watchlist(db: &DatabaseConnection) -> Result<Vec<Symbol>, DbErr> {
    let watchlist_models = WatchlistEntity::find().all(db).await?;
    let watchlist: Vec<Symbol> = watchlist_models.into_iter().map(|w| w.symbol).collect();
    Ok(watchlist)
}

/// Synchronizes the watchlist in the database by deleting all and re-inserting.
async fn sync_watchlist(
    txn: &DatabaseTransaction,
    watchlist: &[Symbol],
) -> Result<(), DbErr> {
    WatchlistEntity::delete_many().exec(txn).await?;

    for symbol in watchlist {
        let db_watchlist = WatchlistActiveModel {
            id: NotSet,
            symbol: Set(symbol.clone()),
        };
        db_watchlist.insert(txn).await?;
    }
    Ok(())
}

pub fn username_checker(username: &String) -> bool {
    println!("Validating username: {username} against storage");
    true
}

/// Saves the current app state to the database.
pub async fn save_state(state: &Arc<Mutex<AppState>>, db: &DatabaseConnection) {
    // No cloning of arc mutex needed here, only required for threads
    // get relevant data first to not block more than required
    let (cash, current_holdings, trades, open_orders, watchlist) = {
        let state_guard = state.lock().unwrap();
        let cash = state_guard.check_balance();

        // Collect holdings into a vector of simple data tuples
        let holdings = state_guard
            .get_holdings_map()
            .iter()
            .map(|(symbol, holding)| (symbol.clone(), holding.get_qty(), holding.get_avg_price()))
            .collect::<Vec<_>>();

        let trades = state_guard.get_trades();

        let open_orders = state_guard.get_open_orders();
        let watchlist = state_guard.get_watchlist();
        (cash, holdings, trades, open_orders, watchlist)
    };

    let _txn_result = db
        .transaction::<_, _, DbErr>(|txn| {
            Box::pin(async move {
                let app_state_opt = AppStateEntity::find_by_id(1).one(txn).await?;
                if let Some(model) = app_state_opt {
                    let mut active_model = model.into_active_model();
                    active_model.cash_balance = Set(cash);
                    active_model.updated_at = Set(chrono::Utc::now().timestamp());
                    active_model.update(txn).await?;
                } else {
                    let new_app_state = AppStateActiveModel {
                        id: Set(1),
                        cash_balance: Set(cash),
                        updated_at: Set(chrono::Utc::now().timestamp()),
                    };
                    new_app_state.insert(txn).await?;
                }

                sync_holdings(txn, &current_holdings).await?;
                sync_trades(txn, &trades).await?;
                sync_open_orders(txn, &open_orders).await?;
                sync_watchlist(txn, &watchlist).await?;

                Ok(())
            })
        })
        .await
        .ok();
}

/// Loads the app state from the database, or initializes a new one if not found.
pub async fn load_state() -> Arc<Mutex<AppState>> {
    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://db.sqlite".to_string());

    match Database::connect(&database_url).await {
        Ok(db) => match load_app_state(&db).await {
            Ok(Some(cash_balance)) => {
                let holdings_map = load_holdings(&db).await.unwrap_or_default();
                let trades = load_trades(&db).await.unwrap_or_default();
                let open_orders = load_open_orders(&db).await.unwrap_or_default();
                let watchlist = load_watchlist(&db).await.unwrap_or_default();

                let mut state = AppState::new();
                state.set_cash_balance(cash_balance);
                state.set_holdings_map(holdings_map).await;
                state.set_trades(trades);
                state.set_open_orders(open_orders);
                state.set_watchlist(watchlist);

                Arc::new(Mutex::new(state))
            }
            Ok(None) => {
                println!("No app state found in database, initializing new state");
                Arc::new(Mutex::new(AppState::new()))
            }
            Err(e) => {
                eprintln!("Error loading state from database: {}", e);
                Arc::new(Mutex::new(AppState::new()))
            }
        },
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            Arc::new(Mutex::new(AppState::new()))
        }
    }
}

/// Resets the app state to default and clears the database.
pub async fn default_state(state: &Arc<Mutex<AppState>>, db: &DatabaseConnection) {
    {
        let mut state_guard = state.lock().unwrap();
        *state_guard = AppState::new();
    }

    let _ = db
        .transaction::<_, _, DbErr>(|txn| {
            Box::pin(async move {
                AppStateEntity::delete_many().exec(txn).await?;
                HoldingEntity::delete_many().exec(txn).await?;
                TradeEntity::delete_many().exec(txn).await?;
                OpenOrderEntity::delete_many().exec(txn).await?;
                WatchlistEntity::delete_many().exec(txn).await?;
                Ok(())
            })
        })
        .await;

    save_state(state, db).await;
}