use std::collections::HashMap;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{AppState::AppState, FinanceProvider, UserInput};

pub fn fund(state: &mut AppState, amount: f64) {
    if amount <= 0.0 {
        println!("Invalid amount");
        return;
    }
    // validate payment first
    state.deposit(amount);
    state.display();
}

pub fn withdraw(state: &mut AppState, amount: f64) {
    if amount <= 0.0 {
        println!("Invalid amount");
        return;
    }
    if amount > state.check_balance() {
        println!("Insufficient balance");
        return;
    }
    state.withdraw(amount);
    state.display();
}

pub type Symbol = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Holding {
    name: String,
    quantity: f64,
    avg_cost: f64,
}

impl Holding {
    pub fn new(name: String, quantity: f64, avg_cost: f64) -> Self {
        Self {
            name,
            quantity,
            avg_cost,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trade {
    symbol: Symbol,
    quantity: f64,
    price_per: f64,
    side: Side,
    timestamp: i64, // epoch seconds
}

impl Trade {
    pub fn buy(symbol: Symbol, quantity: f64, price_per: f64) -> Self {
        Self {
            symbol,
            quantity,
            price_per,
            side: Side::Buy,
            timestamp: Utc::now().timestamp(),
        }
    }

    pub fn sell(symbol: Symbol, quantity: f64, price_per: f64) -> Self {
        Self {
            symbol,
            quantity,
            price_per,
            side: Side::Sell,
            timestamp: Utc::now().timestamp(),
        }
    }
}

pub async fn buy(state: &mut AppState) {
    // ask for ticker
    let ticker = UserInput::ask_ticker();
    // ask for quantity (fractions are allowed)
    let quantity: f64 = UserInput::ask_quantity();
    // get the price per stock
    let price_per: f64 = FinanceProvider::previous_price_close(&ticker, false).await;
    // show the total price of purchase
    let total_price: f64 = price_per * quantity;
    println!("The total price is: {total_price}");
    // check account funds
    if state.check_balance() < total_price {
        println!("Insufficient balance");
    } else {
        // deduct funds
        state.withdraw_purchase(total_price);
        // add the purchase to holdings
        add_to_holdings(&ticker, quantity, price_per, state);
    }
}

fn add_to_holdings(ticker: &String, quantity: f64, price_per: f64, state: &mut AppState) {
    let mut prev_holdings_map: HashMap<Symbol, Holding> = state.get_holdings_map();

    // Use HashMap's get method to check if holding exists
    if let Some(existing_holding) = prev_holdings_map.get(ticker) {
        // Update existing holding with new average cost
        let prev_avg_cost = existing_holding.avg_cost;
        let prev_qty = existing_holding.quantity;
        let new_avg_cost =
            (prev_qty * prev_avg_cost + quantity * price_per) / (prev_qty + quantity);
        let new_qty = prev_qty + quantity;

        prev_holdings_map.insert(
            ticker.clone(),
            Holding::new(ticker.clone(), new_qty, new_avg_cost),
        );
    } else {
        // Insert new holding
        prev_holdings_map.insert(
            ticker.clone(),
            Holding::new(ticker.clone(), quantity, price_per),
        );
    }
    state.set_holdings_map(prev_holdings_map);
}
