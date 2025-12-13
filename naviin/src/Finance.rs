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

    pub fn get_qty(&self) -> f64 {
        self.quantity
    }

    pub fn get_avg_price(&self) -> f64 {
        self.avg_cost
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
    let ticker = UserInput::ask_ticker();
    let quantity: f64 = UserInput::ask_quantity();
    let curr_price: f64 = FinanceProvider::previous_price_close(&ticker, false).await;
    let total_price: f64 = curr_price * quantity;

    println!("The total price is: {total_price}");
    if state.check_balance() < total_price {
        println!("Insufficient balance");
    } else {
        state.withdraw_purchase(total_price);
        add_to_holdings(&ticker, quantity, curr_price, state);
        state.add_trade(Trade::buy(ticker, quantity, curr_price));
    }
}

pub async fn sell(state: &mut AppState) {
    let ticker = UserInput::ask_ticker();
    let quantity: f64 = UserInput::ask_quantity();
    let curr_price: f64 = FinanceProvider::previous_price_close(&ticker, false).await;
    let total_price: f64 = curr_price * quantity;
    println!("The total price of sale is: {total_price}");

    // check holdings
    if state.get_ticker_holdings_qty(&ticker) < quantity {
        println!("You dont have enough of that ticker");
    } else {
        // add funds
        state.deposit_sell(total_price);
        remove_from_holdings(&ticker, quantity, state);
        state.add_trade(Trade::sell(ticker, quantity, curr_price));
    }
}

fn add_to_holdings(ticker: & String, quantity: f64, price_per: f64, state: &mut AppState) {
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

fn remove_from_holdings(ticker: &String, quantity: f64, state: &mut AppState) {
    let mut prev_holdings_map: HashMap<Symbol, Holding> = state.get_holdings_map();
    if let Some(existing_holding) = prev_holdings_map.get(ticker) {
        // Update existing holding with new average cost
        let prev_avg_cost = existing_holding.avg_cost;
        let prev_qty = existing_holding.quantity;
        let new_qty = prev_qty - quantity;
        if new_qty == 0.0 {
            prev_holdings_map.remove(ticker);
        } else {
            prev_holdings_map.insert(
                ticker.clone(),
                Holding::new(ticker.clone(), new_qty, prev_avg_cost),
            );
            state.set_holdings_map(prev_holdings_map);
        }
    }
}
