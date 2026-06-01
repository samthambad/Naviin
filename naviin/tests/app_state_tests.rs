use std::collections::HashMap;

use naviin::AppState::AppState;
use naviin::Finance::Holding;
use naviin::Orders::{OpenOrder, OrderType, Side, Trade};
use rust_decimal_macros::dec;

fn buy_limit(symbol: &str, qty: rust_decimal::Decimal, price: rust_decimal::Decimal) -> OpenOrder {
    OpenOrder::new(
        symbol.to_string(),
        qty,
        price,
        OrderType::BuyLimit,
        Side::Buy,
    )
}

fn stop_loss(symbol: &str, qty: rust_decimal::Decimal, price: rust_decimal::Decimal) -> OpenOrder {
    OpenOrder::new(
        symbol.to_string(),
        qty,
        price,
        OrderType::StopLoss,
        Side::Sell,
    )
}

#[test]
fn cash_operations_reject_invalid_withdrawals_without_changing_balance() {
    let mut state = AppState::new();
    state.deposit(dec!(1000));

    state.withdraw(dec!(250));
    assert_eq!(state.check_balance(), dec!(750));

    state.withdraw(dec!(1000));
    assert_eq!(state.check_balance(), dec!(750));

    state.withdraw(dec!(-1));
    assert_eq!(state.check_balance(), dec!(750));

    state.withdraw_purchase(dec!(-50));
    assert_eq!(state.check_balance(), dec!(750));
}

#[test]
fn buy_orders_reserve_cash_and_reject_over_commitment() {
    let mut state = AppState::new();
    state.deposit(dec!(1000));

    let first_order = buy_limit("AAPL", dec!(5), dec!(100));
    assert!(state.add_open_order(first_order).is_ok());
    assert_eq!(state.get_available_cash(), dec!(500));

    let too_large_second_order = buy_limit("MSFT", dec!(6), dec!(100));
    assert_eq!(
        state.add_open_order(too_large_second_order),
        Err("You don't have enough of cash for this purchase!".to_string())
    );
    assert_eq!(state.get_open_orders().len(), 1);
    assert_eq!(state.get_available_cash(), dec!(500));
}

#[tokio::test]
async fn sell_orders_reserve_shares_and_reject_over_commitment() {
    let mut state = AppState::new();
    let mut holdings = HashMap::new();
    holdings.insert(
        "AAPL".to_string(),
        Holding::new("AAPL".to_string(), dec!(10), dec!(150)),
    );
    state.set_holdings_map(holdings).await;

    let first_order = stop_loss("AAPL", dec!(6), dec!(140));
    assert!(state.add_open_order(first_order).is_ok());
    assert_eq!(
        state.get_available_holdings_qty(&"AAPL".to_string()),
        dec!(4)
    );

    let too_large_second_order = stop_loss("AAPL", dec!(5), dec!(130));
    assert_eq!(
        state.add_open_order(too_large_second_order),
        Err("You don't have enough of this to sell!".to_string())
    );
    assert_eq!(state.get_open_orders().len(), 1);
    assert_eq!(
        state.get_available_holdings_qty(&"AAPL".to_string()),
        dec!(4)
    );
}

#[test]
fn open_orders_are_sorted_by_price_within_same_symbol_and_side() {
    let mut state = AppState::new();
    state.deposit(dec!(10_000));

    state
        .add_open_order(buy_limit("AAPL", dec!(1), dec!(150)))
        .unwrap();
    state
        .add_open_order(buy_limit("AAPL", dec!(1), dec!(145)))
        .unwrap();
    state
        .add_open_order(buy_limit("AAPL", dec!(1), dec!(155)))
        .unwrap();

    let prices = state
        .get_open_orders()
        .iter()
        .map(|order| order.get_price_per())
        .collect::<Vec<_>>();

    assert_eq!(prices, vec![dec!(145), dec!(150), dec!(155)]);
}

#[test]
fn removing_an_open_order_restores_reserved_buying_power() {
    let mut state = AppState::new();
    state.deposit(dec!(1000));

    let order = buy_limit("AAPL", dec!(4), dec!(100));
    state.add_open_order(order.clone()).unwrap();
    assert_eq!(state.get_available_cash(), dec!(600));

    state.remove_from_open_orders(order);
    assert_eq!(state.get_open_orders().len(), 0);
    assert_eq!(state.get_available_cash(), dec!(1000));
}

#[test]
fn trade_history_preserves_trade_details_and_formats_recent_first() {
    let mut state = AppState::new();

    let mut older = Trade::buy("AAPL".to_string(), dec!(2), dec!(150));
    older.set_timestamp(1_704_067_200);
    let mut newer = Trade::sell("MSFT".to_string(), dec!(1.5), dec!(300));
    newer.set_timestamp(1_704_153_600);

    state.add_trade(older);
    state.add_trade(newer);

    let trades = state.get_trades();
    assert_eq!(trades.len(), 2);
    assert_eq!(trades[0].get_symbol(), "AAPL");
    assert_eq!(trades[0].get_side(), &Side::Buy);
    assert_eq!(trades[0].get_quantity(), dec!(2));
    assert_eq!(trades[0].get_price_per(), dec!(150));

    let display = state.display_trades();
    let msft_pos = display
        .find("MSFT")
        .expect("MSFT trade should be displayed");
    let aapl_pos = display
        .find("AAPL")
        .expect("AAPL trade should be displayed");
    assert!(msft_pos < aapl_pos, "newest trade should be shown first");
    assert!(display.contains("Market"));
}
