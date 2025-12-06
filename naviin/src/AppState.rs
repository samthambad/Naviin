use std::{collections::HashMap, hash::Hash};

use crate::Finance::{Holding, Symbol, Trade};

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
    pub fn display(&self) {
        println!("Cash balance: {}", self.cash_balance)
    }
}
