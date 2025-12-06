use std::{collections::HashMap, hash::Hash};

use crate::Finance::{Holding, Symbol, Trade};

struct AppState {
    cash_balance: f64,
    holdings: HashMap<Symbol, Holding>,
    trades: Vec<Trade>,
}

impl AppState {
    fn new() -> Self {
        Self {
            cash_balance: 0.0,
            holdings: HashMap::new(),
            trades: Vec::new(),
        }
    }
}
