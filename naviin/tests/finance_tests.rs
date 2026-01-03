use naviin::AppState::AppState;
use naviin::Finance::{Holding, OpenOrder, Side, Trade};

// ===== Holding Tests =====

#[test]
fn test_holding_creation() {
    let holding = Holding::new("AAPL".to_string(), 10.0, 150.0);

    assert_eq!(holding.get_qty(), 10.0);
    assert_eq!(holding.get_avg_price(), 150.0);
}

#[test]
fn test_holding_getters() {
    let holding = Holding::new("GOOGL".to_string(), 5.5, 2800.50);

    assert_eq!(holding.get_qty(), 5.5);
    assert_eq!(holding.get_avg_price(), 2800.50);
}

#[test]
fn test_holding_with_zero_quantity() {
    let holding = Holding::new("TSLA".to_string(), 0.0, 100.0);

    assert_eq!(holding.get_qty(), 0.0);
}

// ===== Trade Tests =====

#[test]
fn test_trade_buy_creation() {
    let trade = Trade::buy("AAPL".to_string(), 10.0, 150.0);

    assert_eq!(trade.get_symbol(), "AAPL");
    assert_eq!(trade.get_quantity(), 10.0);
    assert_eq!(trade.get_price_per(), 150.0);

    // Verify it's a buy trade
    match trade.get_side() {
        Side::Buy => assert!(true),
        Side::Sell => panic!("Expected Buy, got Sell"),
    }

    // Timestamp should be recent (within last minute)
    let now = chrono::Utc::now().timestamp();
    assert!(trade.get_timestamp() <= now);
    assert!(trade.get_timestamp() > now - 60);
}

#[test]
fn test_trade_sell_creation() {
    let trade = Trade::sell("GOOGL".to_string(), 5.0, 2800.0);

    assert_eq!(trade.get_symbol(), "GOOGL");
    assert_eq!(trade.get_quantity(), 5.0);
    assert_eq!(trade.get_price_per(), 2800.0);

    // Verify it's a sell trade
    match trade.get_side() {
        Side::Sell => assert!(true),
        Side::Buy => panic!("Expected Sell, got Buy"),
    }
}

#[test]
fn test_trade_getters() {
    let trade = Trade::buy("TSLA".to_string(), 15.5, 250.75);

    assert_eq!(trade.get_symbol(), "TSLA");
    assert_eq!(trade.get_quantity(), 15.5);
    assert_eq!(trade.get_price_per(), 250.75);
    assert!(trade.get_timestamp() > 0);
}

#[test]
fn test_trade_with_fractional_shares() {
    let trade = Trade::buy("AAPL".to_string(), 0.5, 150.0);

    assert_eq!(trade.get_quantity(), 0.5);
}

// ===== LimitOrder Tests =====

#[test]
fn test_limit_order_creation() {
    let order = OpenOrder::new("AAPL".to_string(), 10.0, 145.0, Side::Buy);

    assert_eq!(order.get_symbol(), "AAPL");
    assert_eq!(order.get_qty(), 10.0);
    assert_eq!(order.get_price_per(), 145.0);

    match order.get_side() {
        Side::Buy => assert!(true),
        Side::Sell => panic!("Expected Buy, got Sell"),
    }
}

#[test]
fn test_limit_order_sell() {
    let order = OpenOrder::new("GOOGL".to_string(), 5.0, 2900.0, Side::Sell);

    assert_eq!(order.get_symbol(), "GOOGL");

    match order.get_side() {
        Side::Sell => assert!(true),
        Side::Buy => panic!("Expected Sell, got Buy"),
    }
}

#[test]
fn test_limit_order_getters() {
    let order = OpenOrder::new("MSFT".to_string(), 20.0, 350.0, Side::Buy);

    assert_eq!(order.get_symbol(), "MSFT");
    assert_eq!(order.get_qty(), 20.0);
    assert_eq!(order.get_price_per(), 350.0);
    assert!(order.get_timestamp() > 0);
}

#[test]
fn test_limit_order_timestamp_is_recent() {
    let order = OpenOrder::new("AAPL".to_string(), 10.0, 150.0, Side::Buy);

    let now = chrono::Utc::now().timestamp();
    assert!(order.get_timestamp() <= now);
    assert!(order.get_timestamp() > now - 60); // Within last minute
}
