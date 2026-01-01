use naviin::AppState::AppState;
use naviin::Finance::{Trade, LimitOrder, Side, Holding};
use naviin::Storage;
use std::sync::{Arc, Mutex};

// ===== Integration Tests for AppState + Finance =====

#[test]
fn test_complete_trading_workflow() {
    let mut state = AppState::new();
    
    // 1. Fund account
    state.deposit(10000.0);
    assert_eq!(state.check_balance(), 10000.0);
    
    // 2. Simulate a purchase
    state.withdraw_purchase(1500.0);
    assert_eq!(state.check_balance(), 8500.0);
    
    // 3. Add a trade
    let trade = Trade::buy("AAPL".to_string(), 10.0, 150.0);
    state.add_trade(trade);
    
    // 4. Simulate a sale
    state.deposit_sell(1600.0);
    assert_eq!(state.check_balance(), 10100.0);
}

#[test]
fn test_limit_order_management() {
    let mut state = AppState::new();
    
    // Add multiple limit orders
    let order1 = LimitOrder::new("AAPL".to_string(), 10.0, 145.0, Side::Buy);
    let order2 = LimitOrder::new("GOOGL".to_string(), 5.0, 2800.0, Side::Buy);
    let order3 = LimitOrder::new("MSFT".to_string(), 15.0, 340.0, Side::Buy);
    
    state.add_open_order(order1.clone());
    state.add_open_order(order2.clone());
    state.add_open_order(order3.clone());
    
    // Verify all orders are present
    let orders = state.get_open_orders();
    assert_eq!(orders.len(), 3);
    
    // Remove one order
    state.remove_from_open_orders(order2);
    
    // Verify correct order was removed
    let orders = state.get_open_orders();
    assert_eq!(orders.len(), 2);
    
    // Verify remaining orders are correct
    assert_eq!(orders[0].get_symbol(), "AAPL");
    assert_eq!(orders[1].get_symbol(), "MSFT");
}

#[test]
fn test_multiple_trades_tracking() {
    let mut state = AppState::new();
    
    // Execute multiple trades
    let trade1 = Trade::buy("AAPL".to_string(), 10.0, 150.0);
    let trade2 = Trade::sell("GOOGL".to_string(), 5.0, 2800.0);
    let trade3 = Trade::buy("TSLA".to_string(), 8.0, 250.0);
    
    state.add_trade(trade1);
    state.add_trade(trade2);
    state.add_trade(trade3);
    
    // State should still be valid
    assert_eq!(state.check_balance(), 0.0);
}

#[test]
fn test_fund_withdraw_and_reset() {
    let state = Arc::new(Mutex::new(AppState::new()));
    
    // Fund account
    {
        let mut guard = state.lock().unwrap();
        guard.deposit(5000.0);
    }
    
    // Add some orders
    {
        let mut guard = state.lock().unwrap();
        let order = LimitOrder::new("AAPL".to_string(), 10.0, 150.0, Side::Buy);
        guard.add_open_order(order);
    }
    
    // Reset state
    Storage::default_state(&state);
    
    // Verify everything is reset
    let guard = state.lock().unwrap();
    assert_eq!(guard.check_balance(), 0.0);
    assert!(guard.get_open_orders().is_empty());
}

#[test]
fn test_concurrent_balance_operations() {
    let mut state = AppState::new();
    
    // Multiple deposits and withdrawals
    state.deposit(1000.0);
    state.withdraw(200.0);
    state.deposit_sell(500.0);
    state.withdraw_purchase(300.0);
    state.deposit(100.0);
    
    // Expected: 1000 - 200 + 500 - 300 + 100 = 1100
    assert_eq!(state.check_balance(), 1100.0);
}

#[test]
fn test_order_removal_with_multiple_identical_symbols() {
    let mut state = AppState::new();
    
    // Add multiple orders for same symbol but different prices
    let order1 = LimitOrder::new("AAPL".to_string(), 10.0, 145.0, Side::Buy);
    let order2 = LimitOrder::new("AAPL".to_string(), 10.0, 150.0, Side::Buy);
    let order3 = LimitOrder::new("AAPL".to_string(), 10.0, 155.0, Side::Buy);
    
    state.add_open_order(order1.clone());
    state.add_open_order(order2.clone());
    state.add_open_order(order3.clone());
    
    // Remove middle order
    state.remove_from_open_orders(order2);
    
    let orders = state.get_open_orders();
    assert_eq!(orders.len(), 2);
    assert_eq!(orders[0].get_price_per(), 145.0);
    assert_eq!(orders[1].get_price_per(), 155.0);
}

#[test]
fn test_empty_state_operations() {
    let state = AppState::new();
    
    // Operations on empty state should not panic
    assert_eq!(state.check_balance(), 0.0);
    assert!(state.get_holdings_map().is_empty());
    assert!(state.get_open_orders().is_empty());
    assert_eq!(state.get_ticker_holdings_qty(&"AAPL".to_string()), 0.0);
}

#[test]
fn test_trade_creation_preserves_data() {
    let symbol = "AAPL".to_string();
    let qty = 10.5;
    let price = 150.75;
    
    let buy_trade = Trade::buy(symbol.clone(), qty, price);
    
    assert_eq!(buy_trade.get_symbol(), &symbol);
    assert_eq!(buy_trade.get_quantity(), qty);
    assert_eq!(buy_trade.get_price_per(), price);
    
    let sell_trade = Trade::sell(symbol.clone(), qty, price);
    assert_eq!(sell_trade.get_symbol(), &symbol);
}

#[test]
fn test_holding_initialization() {
    let holding = Holding::new("TSLA".to_string(), 25.0, 240.50);
    
    assert_eq!(holding.get_qty(), 25.0);
    assert_eq!(holding.get_avg_price(), 240.50);
}

// ===== Edge Case Tests =====

#[test]
fn test_zero_balance_withdrawal_protection() {
    let mut state = AppState::new();
    
    // Try to withdraw with zero balance
    state.withdraw(100.0);
    
    // Balance should remain zero due to insufficient funds check
    assert_eq!(state.check_balance(), 0.0);
}

#[test]
fn test_order_removal_nonexistent_order() {
    let mut state = AppState::new();
    
    let order1 = LimitOrder::new("AAPL".to_string(), 10.0, 150.0, Side::Buy);
    let order2 = LimitOrder::new("GOOGL".to_string(), 5.0, 2800.0, Side::Buy);
    
    state.add_open_order(order1.clone());
    
    // Try to remove order that was never added
    state.remove_from_open_orders(order2);
    
    // Should still have the first order
    let orders = state.get_open_orders();
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].get_symbol(), "AAPL");
}

#[test]
fn test_large_balance_operations() {
    let mut state = AppState::new();
    
    // Test with large numbers
    state.deposit(1_000_000_000.0);
    assert_eq!(state.check_balance(), 1_000_000_000.0);
    
    state.withdraw(500_000_000.0);
    assert_eq!(state.check_balance(), 500_000_000.0);
}

#[test]
fn test_fractional_share_trading() {
    let trade = Trade::buy("AAPL".to_string(), 0.5, 150.0);
    
    assert_eq!(trade.get_quantity(), 0.5);
    assert_eq!(trade.get_price_per(), 150.0);
}

#[test]
fn test_state_with_holdings_and_orders() {
    let mut state = AppState::new();
    
    // Add balance
    state.deposit(10000.0);
    
    // Add order
    let order = LimitOrder::new("AAPL".to_string(), 10.0, 150.0, Side::Buy);
    state.add_open_order(order);
    
    // Add trade
    let trade = Trade::buy("GOOGL".to_string(), 5.0, 2800.0);
    state.add_trade(trade);
    
    // Verify all components are present
    assert_eq!(state.check_balance(), 10000.0);
    assert_eq!(state.get_open_orders().len(), 1);
}
