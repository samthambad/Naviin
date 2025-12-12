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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trade {
    symbol: Symbol,
    quantity: f64,
    price_per: f64,
    side: Side,
    timestamp: i64, // epoch seconds
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
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
        }
        else {
            // deduct funds
            state.withdraw_purchase(total_price);
            // add the purchase to holdings
            add_to_holdings(&ticker, quantity, price_per);
        }
}

fn add_to_holdings(ticker: &String, quantity: f64, price_per: f64, state: &mut AppState) {
    // calculate avg cost
        // check if there are any exising holdings with the same ticker
        state.holdings
        // if no then price_per is the avg cost
        // otherwise, (prev_qty*avg_cost + quantity*price_per) / (prev_qty + quantity)
}