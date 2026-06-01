use std::fs;
use std::sync::{Arc, Mutex};

use naviin::AppState::AppState;
use naviin::import::import_trades_from_csv;
use rust_decimal_macros::dec;

fn unique_temp_csv(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "naviin_{name}_{}_{}.csv",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    ))
}

#[tokio::test]
async fn csv_import_rebuilds_holdings_and_trade_history_from_valid_rows() {
    let path = unique_temp_csv("valid_import");
    fs::write(
        &path,
        "date,asset,asset_type,side,quantity,price,currency\n\
         2024-01-02,AAPL,STOCK,BUY,10,100,USD\n\
         2024-01-03,aapl,STOCK,BUY,5,160,USD\n\
         2024-01-04,AAPL,STOCK,SELL,4,180,USD\n",
    )
    .unwrap();

    let state = Arc::new(Mutex::new(AppState::new()));
    let report = import_trades_from_csv(&state, path.to_str().unwrap())
        .await
        .unwrap();

    assert_eq!(report, "Imported 3 trades (0 skipped).");
    let guard = state.lock().unwrap();
    assert_eq!(guard.get_trades().len(), 3);
    assert_eq!(guard.get_ticker_holdings_qty(&"AAPL".to_string()), dec!(11));

    let holding = guard.get_holdings_map().get("AAPL").unwrap().clone();
    assert_eq!(holding.get_qty(), dec!(11));
    assert_eq!(holding.get_avg_price().round_dp(4), dec!(120.0000));

    fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn csv_import_skips_invalid_rows_without_applying_partial_side_effects() {
    let path = unique_temp_csv("invalid_import");
    fs::write(
        &path,
        "date,asset,asset_type,side,quantity,price,currency\n\
         2024-01-02,AAPL,STOCK,BUY,10,100,USD\n\
         2024-01-03,MSFT,STOCK,SELL,1,200,USD\n\
         2024-01-04,TSLA,STOCK,BUY,-2,300,USD\n",
    )
    .unwrap();

    let state = Arc::new(Mutex::new(AppState::new()));
    let report = import_trades_from_csv(&state, path.to_str().unwrap())
        .await
        .unwrap();

    assert!(report.starts_with("Imported 1 trades (2 skipped). 2 errors."));
    assert!(report.contains("Insufficient holdings for MSFT"));
    assert!(report.contains("Quantity must be positive"));

    let guard = state.lock().unwrap();
    assert_eq!(guard.get_trades().len(), 1);
    assert_eq!(guard.get_ticker_holdings_qty(&"AAPL".to_string()), dec!(10));
    assert_eq!(guard.get_ticker_holdings_qty(&"MSFT".to_string()), dec!(0));
    assert_eq!(guard.get_ticker_holdings_qty(&"TSLA".to_string()), dec!(0));

    fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn csv_import_reports_missing_required_columns_before_mutating_state() {
    let path = unique_temp_csv("missing_columns");
    fs::write(
        &path,
        "date,asset,side,quantity,price\n2024-01-02,AAPL,BUY,10,100\n",
    )
    .unwrap();

    let state = Arc::new(Mutex::new(AppState::new()));
    let error = import_trades_from_csv(&state, path.to_str().unwrap())
        .await
        .unwrap_err();

    assert_eq!(error, "Missing required column: asset_type");
    let guard = state.lock().unwrap();
    assert!(guard.get_trades().is_empty());
    assert!(guard.get_holdings_map().is_empty());

    fs::remove_file(path).unwrap();
}
