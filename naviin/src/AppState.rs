use std::{collections::HashMap, hash::Hash};

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

    pub fn display(&self) {
        println!("Cash balance: {}", self.cash_balance)
    }

    pub fn check_balance(&self) -> f64 {
        self.cash_balance
    }
}
