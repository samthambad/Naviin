use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rust_decimal::prelude::*;

use crate::{AppState::AppState, FinanceProvider, UserInput};

// Add funds to user account
pub async fn fund(state: &Arc<Mutex<AppState>>, amount: Decimal) {
    if amount < Decimal::ZERO {
        println!("Invalid amount");
        return;
    }
    // validate payment first
    // separate thread not needed since it in run on user input
    let mut state_guard = state.lock().unwrap();
    state_guard.deposit(amount);
    state_guard.display().await;
}

// Withdraw funds from user account if sufficient balance available
pub async fn withdraw(state: &Arc<Mutex<AppState>>, amount: Decimal) {
    let mut state_guard = state.lock().unwrap();
    if amount < Decimal::ZERO {
        println!("Invalid amount");
        return;
    }
    if amount > state_guard.check_balance() {
        println!("Insufficient balance");
        return;
    }
    state_guard.withdraw(amount);
    state_guard.display().await;
}

pub type Symbol = String;

// Represents owned stock position with quantity and average purchase cost
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Holding {
    name: String,
    quantity: Decimal,
    avg_cost: Decimal,
}

impl Holding {
    pub fn new(name: String, quantity: Decimal, avg_cost: Decimal) -> Self {
        Self {
            name,
            quantity,
            avg_cost,
        }
    }

    pub fn get_qty(&self) -> Decimal {
        self.quantity
    }

    pub fn get_avg_price(&self) -> Decimal {
        self.avg_cost
    }

    pub async fn get_pnl(&self) -> Decimal {
        let curr_price = FinanceProvider::curr_price(&self.name, false).await;
        let delta = curr_price - self.get_avg_price();
        delta * self.get_qty()
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

    pub async fn get_pnl(&self) -> f64 {
        // fetch current price
        let curr_price = FinanceProvider::curr_price(&self.name, false).await;
        // price delta per share
        let delta = curr_price - self.get_avg_price();
        // multiply by the shares owned
        delta * self.get_qty()
    }
}

// Asks user for input and calls Trade::buy
pub async fn create_buy(state: &Arc<Mutex<AppState>>) {
    let symbol = match UserInput::ask_ticker() {
        Some(t) => t,
        None => return,
    };
    let purchase_qty: f64 = match UserInput::ask_quantity() {
        Some(q) => q,
        None => return,
    };
    let curr_price = FinanceProvider::curr_price(&symbol, false).await;
    let total_price = curr_price * purchase_qty;

    let mut state_guard = state.lock().unwrap();
    println!("The total price is: {total_price}");
    if state_guard.check_balance() < total_price {
        println!("Insufficient balance");
    } else {
        state_guard.withdraw_purchase(total_price);
        add_to_holdings(&symbol, purchase_qty, curr_price, &mut state_guard).await;
        state_guard.add_trade(crate::Orders::Trade::buy(symbol, purchase_qty, curr_price));
    }
}

// Asks user for input and calls Trade::sell
pub async fn create_sell(state: &Arc<Mutex<AppState>>) {
    let ticker = match UserInput::ask_ticker() {
        Some(t) => t,
        None => return,
    };
    let quantity: f64 = match UserInput::ask_quantity() {
        Some(q) => q,
        None => return,
    };
    let curr_price: f64 = FinanceProvider::curr_price(&ticker, false).await;
    let total_price: f64 = curr_price * quantity;
    println!("The total price of sale is: {total_price}");

    let mut state_guard = state.lock().unwrap();
    // check holdings
    if state_guard.get_ticker_holdings_qty(&ticker) < quantity {
        println!("You dont have enough of that ticker");
    } else {
        // add funds
        state_guard.deposit_sell(total_price);
        remove_from_holdings(&ticker, quantity, &mut (*state_guard)).await;
        state_guard.add_trade(crate::Orders::Trade::sell(ticker, quantity, curr_price));
    }
}

// Update or create holding with new purchase, calculating average cost
pub(crate) async fn add_to_holdings(ticker: &String, quantity: Decimal, price_per: Decimal, state: &mut AppState) {
    let mut prev_holdings_map: HashMap<Symbol, Holding> = state.get_holdings_map();

    // Use HashMap's get method to check if holding exists
    if let Some(existing_holding) = prev_holdings_map.get(ticker) {
        let prev_avg_cost = existing_holding.get_avg_price();
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
    state.set_holdings_map(prev_holdings_map).await;
}

// Reduce or remove holding after sale, keeping average cost unchanged
pub(crate) async fn remove_from_holdings(ticker: &String, quantity: Decimal, state: &mut AppState) {
    let mut prev_holdings_map: HashMap<Symbol, Holding> = state.get_holdings_map();
    if let Some(existing_holding) = prev_holdings_map.get(ticker) {
        let prev_avg_cost = existing_holding.get_avg_price();
        let prev_qty = existing_holding.quantity;
        let new_qty = prev_qty - quantity;
        if new_qty == Decimal::ZERO {
            prev_holdings_map.remove(ticker);
        } else {
            prev_holdings_map.insert(
                ticker.clone(),
                Holding::new(ticker.clone(), new_qty, prev_avg_cost),
            );
            state.set_holdings_map(prev_holdings_map).await;
        }
    }
}
}
