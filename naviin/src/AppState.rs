use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use chrono;
use serde::{Deserialize, Serialize};

use crate::Finance::{self, Holding, OpenOrder, Side, Symbol, Trade};
use crate::FinanceProvider;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppState {
    cash_balance: f64,
    holdings: HashMap<Symbol, Holding>,
    trades: Vec<Trade>,
    open_orders: Vec<OpenOrder>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            cash_balance: 0.0,
            holdings: HashMap::new(),
            trades: Vec::new(),
            open_orders: Vec::new(),
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

        // Open Orders Display
        if self.open_orders.is_empty() {
            println!("\nNO OPEN ORDERS");
        } else {
            println!("\nOPEN ORDERS:");
            println!(
                "{:<10} {:<6} {:<10} {:<12} {:<20}",
                "Symbol", "Side", "Qty", "Price/Share", "Timestamp"
            );
            println!("------------------------------------------------------------------");
            for order in &self.open_orders {
                let datetime =
                    chrono::DateTime::<chrono::Utc>::from_timestamp(order.get_timestamp(), 0)
                        .map(|dt| {
                            dt.with_timezone(&chrono::Local)
                                .format("%Y-%m-%d %H:%M:%S")
                                .to_string()
                        })
                        .unwrap_or_else(|| order.get_timestamp().to_string());
                println!(
                    "{:<10} {:<6} {:<10.2} {:<12.2} {:<20}",
                    order.get_symbol(),
                    match order.get_side() {
                        Side::Buy => "BUY",
                        Side::Sell => "SELL",
                    },
                    order.get_qty(),
                    order.get_price_per(),
                    datetime
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

    pub fn get_open_orders(&self) -> Vec<OpenOrder> {
        self.open_orders.clone()
    }

    // TODO
    // if you have a queue of open orders, does it execute FiFo,
    // or does it execute based on price, and then check for queue order as a tiebreaker
    pub fn add_open_order(&mut self, new_order: OpenOrder) {
        if matches!(new_order.get_side(), Side::Sell) {
            let qty_in_acc = self.get_ticker_holdings_qty(&new_order.get_symbol());
            for o in self.open_orders {

            }
            // 1. check previous open orders

            // 2. subtract the sell ones
            // 3. then compare the magnitude
            println!("You don't have enough of this to sell!");
            return;
        }
        self.open_orders.push(new_order);
        println!("Open order added");
    }

    pub fn remove_from_open_orders(&mut self, order_to_remove: OpenOrder) {
        self.open_orders.retain(|order| {
            !(order.get_symbol() == order_to_remove.get_symbol()
                && order.get_price_per() == order_to_remove.get_price_per()
                && order.get_qty() == order_to_remove.get_qty())
        });
    }
}

pub async fn monitor_order(state: Arc<Mutex<AppState>>, running: Arc<AtomicBool>) {
    // create a thread
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        while running.load(Ordering::Relaxed) {
            {
                let mut state_guard = state.lock().unwrap();
                let open_orders: Vec<OpenOrder> = state_guard.get_open_orders();
                // pull price
                let mut orders_executed: Vec<OpenOrder> = Vec::new();
                for o in open_orders {
                    rt.block_on(async {
                        if matches!(o.get_side(), Side::Buy)
                            && Finance::buy_limit(&mut state_guard, &o).await
                        {
                            // remove that one from the list
                            orders_executed.push(o);
                        } else if Finance::sell_stop_loss(&mut state_guard, &o).await {
                            orders_executed.push(o);
                        }
                    });
                }
                for o in orders_executed {
                    state_guard.remove_from_open_orders(o);
                }
            }
            thread::sleep(Duration::from_secs(10))
        }
        println!("Order shutting down")
    });
}
