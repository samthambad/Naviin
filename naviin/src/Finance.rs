use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::{AppState::AppState, FinanceProvider, UserInput};

pub async fn fund(state: &Arc<Mutex<AppState>>, amount: f64) {
    if amount <= 0.0 {
        println!("Invalid amount");
        return;
    }
    // validate payment first
    // separate thread not needed since it in run on user input
    let mut state_guard = state.lock().unwrap();
    state_guard.deposit(amount);
    state_guard.display().await;
}

pub async fn withdraw(state: &Arc<Mutex<AppState>>, amount: f64) {
    let mut state_guard = state.lock().unwrap();
    if amount <= 0.0 {
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

    pub async fn get_pnl(&self) -> f64 {
        // fetch current price
        let curr_price = FinanceProvider::curr_price(&self.name, false).await;
        // price delta per share
        let delta = curr_price - self.get_avg_price();
        // multiply by the shares owned
        delta * self.get_qty()
    }
}

pub async fn buy(state: &Arc<Mutex<AppState>>) {
    let symbol = match UserInput::ask_ticker() {
        Some(t) => t,
        None => return,
    };
    let purchase_qty: f64 = match UserInput::ask_quantity() {
        Some(q) => q,
        None => return,
    };
    let curr_price: f64 = FinanceProvider::curr_price(&symbol, false).await;
    let total_price: f64 = curr_price * purchase_qty;

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

pub async fn sell(state: &Arc<Mutex<AppState>>) {
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

pub(crate) async fn add_to_holdings(ticker: &String, quantity: f64, price_per: f64, state: &mut AppState) {
    let mut prev_holdings_map: HashMap<Symbol, Holding> = state.get_holdings_map();

    // Use HashMap's get method to check if holding exists
    if let Some(existing_holding) = prev_holdings_map.get(ticker) {
        // Update existing holding with new average cost
        let prev_avg_cost: f64 = existing_holding.get_avg_price();
        let prev_qty: f64 = existing_holding.quantity;
        let new_avg_cost: f64 =
            (prev_qty * prev_avg_cost + quantity * price_per) / (prev_qty + quantity);
        let new_qty: f64 = prev_qty + quantity;

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

pub(crate) async fn remove_from_holdings(ticker: &String, quantity: f64, state: &mut AppState) {
    let mut prev_holdings_map: HashMap<Symbol, Holding> = state.get_holdings_map();
    if let Some(existing_holding) = prev_holdings_map.get(ticker) {
        // Update existing holding with new average cost
        let prev_avg_cost: f64 = existing_holding.get_avg_price();
        let prev_qty: f64 = existing_holding.quantity;
        let new_qty: f64 = prev_qty - quantity;
        if new_qty == 0.0 {
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
