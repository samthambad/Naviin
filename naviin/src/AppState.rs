use std::collections::HashMap;

use chrono;
use serde::{Deserialize, Serialize};

use crate::Finance::{Holding, Side, Symbol, Trade};
use crate::FinanceProvider;

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

    pub async fn display(&self) {
        println!("\n--- Naviin App State ---");
        println!("Cash Balance: {:.2}", self.cash_balance);

        // Holdings Display
        if self.holdings.is_empty() {
            println!("\nNO HOLDINGS");
        } else {
            println!("\nHOLDINGS:");
            println!(
                "{:<10} {:<10} {:<15} {:<15} {:<15} {:<15}",
                "Symbol", "Qty", "Avg Cost", "Curr Price", "Total Value", "PNL"
            );
            println!(
                "----------------------------------------------------------------------------------"
            );
            for (symbol, holding) in &self.holdings {
                let curr_price = FinanceProvider::previous_price_close(symbol, false).await;
                let total_value = holding.get_qty() * curr_price;
                let pnl = holding.get_pnl().await;
                println!(
                    "{:<10} {:<10.2} {:<15.2} {:<15.2} {:<15.2} {:<15.2}",
                    symbol,
                    holding.get_qty(),
                    holding.get_avg_price(),
                    curr_price,
                    total_value,
                    pnl
                );
            }
        }

        // Trades Display
        if self.trades.is_empty() {
            println!("\nNO TRADES");
        } else {
            println!("\nTRADES:");
            println!(
                "{:<10} {:<6} {:<10} {:<12} {:<20}",
                "Symbol", "Side", "Qty", "Price/Share", "Timestamp"
            );
            println!("------------------------------------------------------------------");
            for trade in &self.trades {
                // Convert timestamp to human-readable format if possible, otherwise print epoch
                let datetime =
                    chrono::DateTime::<chrono::Utc>::from_timestamp(trade.get_timestamp(), 0)
                        .map(|dt| {
                            dt.with_timezone(&chrono::Local)
                                .format("%Y-%m-%d %H:%M:%S")
                                .to_string()
                        })
                        .unwrap_or_else(|| trade.get_timestamp().to_string());
                println!(
                    "{:<10} {:<6} {:<10.2} {:<12.2} {:<20}",
                    trade.get_symbol(),
                    match trade.get_side() {
                        Side::Buy => "BUY",
                        Side::Sell => "SELL",
                    },
                    trade.get_quantity(),
                    trade.get_price_per(),
                    datetime
                );
            }
        }
        println!(
            "----------------------------------------------------------------------------------"
        );
    }

    pub fn check_balance(&self) -> f64 {
        self.cash_balance
    }

    pub fn get_holdings_map(&self) -> HashMap<Symbol, Holding> {
        self.holdings.clone()
    }

    pub async fn set_holdings_map(&mut self, new_holdings_map: HashMap<Symbol, Holding>) {
        self.holdings = new_holdings_map;
        self.display().await;
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
