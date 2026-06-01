use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use naviin::AppState::AppState;
use naviin::Finance::Holding;
use naviin::Orders::{OpenOrder, OrderType, Side, Trade};
use naviin::Storage;
use naviin::entities::{app_state, holding, open_order, trade, watchlist};
use rust_decimal_macros::dec;
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, EntityTrait, Schema};

async fn test_db() -> DatabaseConnection {
    let mut options = ConnectOptions::new("sqlite::memory:");
    options.max_connections(1);
    let db = Database::connect(options).await.unwrap();

    let schema = Schema::new(db.get_database_backend());
    for statement in [
        schema.create_table_from_entity(app_state::Entity),
        schema.create_table_from_entity(holding::Entity),
        schema.create_table_from_entity(trade::Entity),
        schema.create_table_from_entity(open_order::Entity),
        schema.create_table_from_entity(watchlist::Entity),
    ] {
        db.execute(&statement).await.unwrap();
    }

    db
}

#[tokio::test]
async fn save_state_persists_cash_holdings_trades_orders_and_watchlist() {
    let db = test_db().await;
    let mut app = AppState::new();
    app.deposit(dec!(5000));

    let mut holdings = HashMap::new();
    holdings.insert(
        "AAPL".to_string(),
        Holding::new("AAPL".to_string(), dec!(10), dec!(125)),
    );
    app.set_holdings_map(holdings).await;

    let mut trade = Trade::buy("AAPL".to_string(), dec!(10), dec!(125));
    trade.set_timestamp(1_704_067_200);
    app.add_trade(trade);

    app.add_open_order(OpenOrder::new(
        "MSFT".to_string(),
        dec!(5),
        dec!(300),
        OrderType::BuyLimit,
        Side::Buy,
    ))
    .unwrap();

    assert!(app.add_to_watchlist("AAPL".to_string()));
    assert!(app.add_to_watchlist("MSFT".to_string()));

    let state = Arc::new(Mutex::new(app));

    Storage::save_state(&state, &db).await;

    let app_state = app_state::Entity::find_by_id(1)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    let holdings = holding::Entity::find().all(&db).await.unwrap();
    let trades = trade::Entity::find().all(&db).await.unwrap();
    let open_orders = open_order::Entity::find().all(&db).await.unwrap();
    let watchlist = watchlist::Entity::find().all(&db).await.unwrap();

    assert_eq!(app_state.cash_balance, dec!(5000));
    assert_eq!(holdings.len(), 1);
    assert_eq!(holdings[0].symbol, "AAPL");
    assert_eq!(holdings[0].quantity, dec!(10));
    assert_eq!(holdings[0].avg_cost, dec!(125));
    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].symbol, "AAPL");
    assert_eq!(trades[0].quantity, dec!(10));
    assert_eq!(open_orders.len(), 1);
    assert_eq!(open_orders[0].symbol, "MSFT");
    assert_eq!(
        watchlist
            .iter()
            .map(|row| row.symbol.clone())
            .collect::<Vec<_>>(),
        vec!["AAPL".to_string(), "MSFT".to_string()]
    );
}

#[tokio::test]
async fn save_state_updates_existing_holdings_and_replaces_open_orders_and_watchlist() {
    let db = test_db().await;
    let mut app = AppState::new();
    app.deposit(dec!(1000));
    let mut holdings = HashMap::new();
    holdings.insert(
        "AAPL".to_string(),
        Holding::new("AAPL".to_string(), dec!(1), dec!(100)),
    );
    app.set_holdings_map(holdings).await;
    app.add_open_order(OpenOrder::new(
        "AAPL".to_string(),
        dec!(1),
        dec!(90),
        OrderType::BuyLimit,
        Side::Buy,
    ))
    .unwrap();
    assert!(app.add_to_watchlist("AAPL".to_string()));

    let state = Arc::new(Mutex::new(app));
    Storage::save_state(&state, &db).await;

    let mut replacement = AppState::new();
    replacement.set_cash_balance(dec!(2500));
    let mut holdings = HashMap::new();
    holdings.insert(
        "AAPL".to_string(),
        Holding::new("AAPL".to_string(), dec!(3), dec!(110)),
    );
    replacement.set_holdings_map(holdings).await;
    replacement.set_open_orders(vec![]);
    replacement.set_watchlist(vec!["MSFT".to_string()]);

    {
        let mut guard = state.lock().unwrap();
        *guard = replacement;
    }
    Storage::save_state(&state, &db).await;

    let app_state = app_state::Entity::find_by_id(1)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    let holdings = holding::Entity::find().all(&db).await.unwrap();
    let open_orders = open_order::Entity::find().all(&db).await.unwrap();
    let watchlist = watchlist::Entity::find().all(&db).await.unwrap();

    assert_eq!(app_state.cash_balance, dec!(2500));
    assert_eq!(holdings.len(), 1);
    assert_eq!(holdings[0].symbol, "AAPL");
    assert_eq!(holdings[0].quantity, dec!(3));
    assert_eq!(holdings[0].avg_cost, dec!(110));
    assert!(open_orders.is_empty());
    assert_eq!(
        watchlist
            .iter()
            .map(|row| row.symbol.clone())
            .collect::<Vec<_>>(),
        vec!["MSFT".to_string()]
    );
}

#[tokio::test]
async fn default_state_clears_database_and_recreates_empty_app_state() {
    let db = test_db().await;
    let state = Arc::new(Mutex::new(AppState::new()));

    {
        let mut guard = state.lock().unwrap();
        guard.deposit(dec!(5000));
        assert!(guard.add_to_watchlist("AAPL".to_string()));
    }
    Storage::save_state(&state, &db).await;

    Storage::default_state(&state, &db).await;

    {
        let guard = state.lock().unwrap();
        assert_eq!(guard.check_balance(), dec!(0));
        assert!(guard.get_holdings_map().is_empty());
        assert!(guard.get_open_orders().is_empty());
        assert!(guard.get_watchlist().is_empty());
    }

    let app_state = app_state::Entity::find_by_id(1)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(app_state.cash_balance, dec!(0));
    assert!(holding::Entity::find().all(&db).await.unwrap().is_empty());
    assert!(
        open_order::Entity::find()
            .all(&db)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(watchlist::Entity::find().all(&db).await.unwrap().is_empty());
}
