use std::{collections::HashMap, sync::Arc, sync::Mutex};
use tokio::sync::Mutex as TokioMutex;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{AppState::AppState, FinanceProvider, UserInput};

pub async fn fund(state: &Arc<Mutex<AppState>>, amount: f64) {
    if amount <= 0.0 {
        println!("Invalid amount");
        return;
    }
    // validate payment first
    // seperate thread not needed since it in run on user input
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
        let curr_price = FinanceProvider::previous_price_close(&self.name, false).await;
        // price delta per share
        let delta = curr_price - self.get_avg_price();
        // multiply by the shares owned
        delta * self.get_qty()
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

    pub fn get_symbol(&self) -> &Symbol {
        &self.symbol
    }

    pub fn get_quantity(&self) -> f64 {
        self.quantity
    }

    pub fn get_price_per(&self) -> f64 {
        self.price_per
    }

    pub fn get_side(&self) -> &Side {
        &self.side
    }

    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
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
    let curr_price: f64 = FinanceProvider::previous_price_close(&symbol, false).await;
    let total_price: f64 = curr_price * purchase_qty;

    let mut state_guard = state.lock().unwrap();
    println!("The total price is: {total_price}");
    if state_guard.check_balance() < total_price {
        println!("Insufficient balance");
    } else {
        state_guard.withdraw_purchase(total_price);
        add_to_holdings(&symbol, purchase_qty, curr_price, &mut state_guard).await;
        state_guard.add_trade(Trade::buy(symbol, purchase_qty, curr_price));
    }
}

pub async fn create_limit_order() -> Option<LimitOrder> {
    let ticker = match UserInput::ask_ticker() {
        Some(t) => t,
        None => return None,
    };
    let quantity: f64 = match UserInput::ask_quantity() {
        Some(q) => q,
        None => return None,
    };
    let limit_price: f64 = match UserInput::ask_price() {
        Some(q) => q,
        None => return None,
    };
    // create new 
    Some(LimitOrder {
        symbol: ticker.clone(),
        quantity,
        price_per: limit_price,
    })
}

//  is good till cancelled
pub async fn buy_limit(state: &mut AppState, order: &LimitOrder) -> bool {

    let symbol = order.get_symbol().clone();
    let limit_price = order.get_price_per();
    let purchase_qty = order.get_qty();
    let curr_cash = state.check_balance();
    let total_purchase_value = limit_price * purchase_qty;
    let curr_price: f64 = FinanceProvider::previous_price_close(&symbol, false).await;
    if curr_price <= limit_price {
        if total_purchase_value > curr_cash {
            println!("Insufficient balance");
            return false;
        }
        state.withdraw_purchase(total_purchase_value);
        add_to_holdings(&symbol, purchase_qty, limit_price, state).await;
        state.add_trade(Trade::buy(symbol, purchase_qty, limit_price));
        return true;
    }
    false
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
    let curr_price: f64 = FinanceProvider::previous_price_close(&ticker, false).await;
    let total_price: f64 = curr_price * quantity;
    println!("The total price of sale is: {total_price}");

    let mut state_guard = state.lock().unwrap();
    // check holdings
    if state_guard.get_ticker_holdings_qty(&ticker) < quantity {
        println!("You dont have enough of that ticker");
    } else {
        // add funds
        state_guard.deposit_sell(total_price);
        remove_from_holdings(&ticker, quantity, state).await;
        state_guard.add_trade(Trade::sell(ticker, quantity, curr_price));
    }
}

async fn add_to_holdings(
    ticker: &String,
    quantity: f64,
    price_per: f64,
    state: &mut AppState,
) {
    let mut prev_holdings_map: HashMap<Symbol, Holding> = state.get_holdings_map();

    // Use HashMap's get method to check if holding exists
    if let Some(existing_holding) = prev_holdings_map.get(ticker) {
        // Update existing holding with new average cost
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

async fn remove_from_holdings(ticker: &String, quantity: f64, state: &Arc<Mutex<AppState>>) {
    let mut state_guard = state.lock().unwrap();
    let mut prev_holdings_map: HashMap<Symbol, Holding> = state_guard.get_holdings_map();
    if let Some(existing_holding) = prev_holdings_map.get(ticker) {
        // Update existing holding with new average cost
        let prev_avg_cost = existing_holding.get_avg_price();
        let prev_qty = existing_holding.quantity;
        let new_qty = prev_qty - quantity;
        if new_qty == 0.0 {
            prev_holdings_map.remove(ticker);
        } else {
            prev_holdings_map.insert(
                ticker.clone(),
                Holding::new(ticker.clone(), new_qty, prev_avg_cost),
            );
            state_guard.set_holdings_map(prev_holdings_map).await;
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LimitOrder {
    symbol: Symbol,
    quantity: f64,
    price_per: f64,
}

impl LimitOrder {
    pub fn get_symbol(&self) -> &Symbol {
        &self.symbol
    }
    pub fn get_price_per(&self) -> f64 {
        self.price_per
    }
    pub fn get_qty(&self) -> f64{
        self.quantity
    }
}
