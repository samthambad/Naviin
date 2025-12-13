use std::{collections::HashMap, thread::sleep};

use serde::{Deserialize, Serialize};

use crate::Finance::{Holding, Symbol, Trade};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppState {
    cash_balance: f64,
    holdings: HashMap<Symbol, Holding>,
    trades: Vec<Trade>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            cash_balance: 0.0,
            holdings: HashMap::new(),
            trades: Vec::new(),
        }
    }

    pub fn deposit(&mut self, amount: f64) {
        self.cash_balance += amount;
    }

    pub fn withdraw(&mut self, amount: f64) {
        if amount <= 0.0 {
            println!("Invalid amount");
            return;
        }
        self.cash_balance -= amount;
    }

    pub fn withdraw_purchase(&mut self, amount: f64) {
        if amount <= 0.0 {
            println!("Invalid amount");
            return;
        }
        self.cash_balance -= amount;
    }

    pub fn deposit_sell(&mut self, amount: f64) {
        self.cash_balance += amount;
    }

    pub fn display(&self) {
        println!("Cash balance: {}", self.cash_balance);
        if self.holdings.len() == 0 {
            println!("NO HOLDINGS");
        }
        else {
            println!("Holdings:");
            for (symbol, holding) in &self.holdings {
                println!("  {}: {:?}", symbol, holding);
            }
        }
    }

    pub fn check_balance(&self) -> f64 {
        self.cash_balance
    }

    pub fn get_holdings_map(&self) ->HashMap<Symbol, Holding> {
        self.holdings.clone()
    }

    pub fn set_holdings_map(&mut self, new_holdings_map: HashMap<Symbol, Holding>) {
        self.holdings = new_holdings_map;
        self.display();
    }

    pub fn add_trade(&mut self, trade_to_add: Trade) {
        let mut new_trades = self.trades.clone();
        new_trades.push(trade_to_add);
        self.trades = new_trades;
    }

    pub fn get_ticker_holdings_qty(&self, ticker: &String) -> f64 {
        match self.get_holdings_map().get(ticker) {
            Some(holding) => holding.get_qty(),
            None => 0.0,
        }
    }

}
