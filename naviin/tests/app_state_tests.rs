// Import the AppState struct from our main naviin library.
// The name of the crate is `naviin`, as defined in Cargo.toml.
use naviin::AppState::AppState;
use naviin::Finance::{OpenOrder, Side, Trade};

#[test]
fn test_deposit_and_balance() {
    // Arrange: Create a new AppState
    let mut state = AppState::new();

    // Act: Deposit 100.0 into the account
    state.deposit(100.0);

    // Assert: Use the public `check_balance` method to verify the result
    assert_eq!(state.check_balance(), 100.0);
}

#[test]
fn test_withdraw_and_balance() {
    // Arrange: Create an AppState with an initial balance
    let mut state = AppState::new();
    state.deposit(100.0);

    // Act: Withdraw 50.0
    state.withdraw(50.0);

    // Assert: Check the final balance
    assert_eq!(state.check_balance(), 50.0);
}

#[test]
fn test_withdraw_with_invalid_amount() {
    // Arrange
    let mut state = AppState::new();
    state.deposit(100.0);

    // Act: Withdraw a negative amount
    state.withdraw(-50.0);

    // Assert: The balance should not have changed
    assert_eq!(state.check_balance(), 100.0);
}

#[test]
fn test_withdraw_with_zero_amount() {
    let mut state = AppState::new();
    state.deposit(100.0);

    // Withdraw zero - should be invalid
    state.withdraw(0.0);

    // Balance should remain unchanged
    assert_eq!(state.check_balance(), 100.0);
}

#[test]
fn test_withdraw_purchase() {
    let mut state = AppState::new();
    state.deposit(100.0);
    state.withdraw_purchase(50.0);
    assert_eq!(state.check_balance(), 50.0);
}

#[test]
fn test_withdraw_purchase_invalid_amount() {
    let mut state = AppState::new();
    state.deposit(100.0);

    // Invalid negative amount
    state.withdraw_purchase(-10.0);

    // Balance should not change
    assert_eq!(state.check_balance(), 100.0);
}

#[test]
fn test_deposit_sell() {
    let mut state = AppState::new();
    state.deposit_sell(50.0);
    assert_eq!(state.check_balance(), 50.0);
}

#[test]
fn test_multiple_deposits_and_withdrawals() {
    let mut state = AppState::new();
    state.deposit_sell(50.0);
    state.withdraw_purchase(30.0);
    state.deposit(50.0);
    state.withdraw(20.0);
    assert_eq!(state.check_balance(), 50.0);
}

#[test]
fn test_add_trade() {
    let mut state = AppState::new();
    let trade = Trade::buy("AAPL".to_string(), 10.0, 150.0);

    state.add_trade(trade.clone());

    // We can't directly inspect trades without a getter, but we can verify the state doesn't panic
    // This is a basic smoke test
    assert_eq!(state.check_balance(), 0.0);
}

#[test]
fn test_get_ticker_holdings_qty_empty() {
    let state = AppState::new();

    // Getting quantity for non-existent ticker should return 0
    let qty = state.get_ticker_holdings_qty(&"AAPL".to_string());
    assert_eq!(qty, 0.0);
}

#[test]
fn test_add_open_order() {
    let mut state = AppState::new();

    // Create a limit order manually
    let order = OpenOrder::new("AAPL".to_string(), 10.0, 150.0, Side::Buy);

    state.add_open_order(order);

    // Verify order was added
    let orders = state.get_open_orders();
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].get_symbol(), "AAPL");
    assert_eq!(orders[0].get_qty(), 10.0);
    assert_eq!(orders[0].get_price_per(), 150.0);
}

#[test]
fn test_remove_from_open_orders() {
    let mut state = AppState::new();

    let order1 = OpenOrder::new("AAPL".to_string(), 10.0, 150.0, Side::Buy);
    let order2 = OpenOrder::new("GOOGL".to_string(), 5.0, 2800.0, Side::Buy);

    state.add_open_order(order1.clone());
    state.add_open_order(order2);

    // Remove first order
    state.remove_from_open_orders(order1);

    // Verify only one order remains
    let orders = state.get_open_orders();
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].get_symbol(), "GOOGL");
}

#[test]
fn test_get_holdings_map_empty() {
    let state = AppState::new();

    let holdings = state.get_holdings_map();
    assert!(holdings.is_empty());
}

#[test]
fn test_new_state_has_zero_balance() {
    let state = AppState::new();
    assert_eq!(state.check_balance(), 0.0);
}

#[test]
fn test_default_trait() {
    let state = AppState::default();
    assert_eq!(state.check_balance(), 0.0);
}
#[test]
fn test_order_removal_works_with_cloned_order() {
    let mut state = AppState::new();
    let order = OpenOrder::new("AAPL".to_string(), 10.0, 150.0, Side::Buy);
    let order_copy = order.clone();
    state.add_open_order(order_copy);

    // Test that removal works with cloned order
    state.remove_from_open_orders(order.clone());
    assert_eq!(state.get_open_orders().len(), 0);
}
