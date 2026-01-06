use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use chrono;
use serde::{Deserialize, Serialize};

use crate::Finance::{Holding, Symbol};
use crate::FinanceProvider;
use crate::Orders::{OpenOrder, Side, Trade};

// Manages user account state including cash, holdings, trades, and pending orders
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

    // Withdraw funds with validation
    pub fn withdraw(&mut self, amount: f64) {
        if amount <= 0.0 {
            println!("Invalid amount");
            return;
        }
        if amount > self.cash_balance {
            println!("Insufficient balance");
            return;
        }
        self.cash_balance -= amount;
    }

    // Deduct purchase amount from balance without validation (used in buy functions)
    pub fn withdraw_purchase(&mut self, amount: f64) {
        if amount <= 0.0 {
            println!("Invalid amount");
            return;
        }
        self.cash_balance -= amount;
    }

    // Add sale proceeds to balance
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
                let curr_price = FinanceProvider::curr_price(symbol, false).await;
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

    // Get current cash balance
    pub fn check_balance(&self) -> f64 {
        self.cash_balance
    }

    // Get all holdings as copy of internal map
    pub fn get_holdings_map(&self) -> HashMap<Symbol, Holding> {
        self.holdings.clone()
    }

    // Update holdings and refresh display
    pub async fn set_holdings_map(&mut self, new_holdings_map: HashMap<Symbol, Holding>) {
        self.holdings = new_holdings_map;
        self.display().await;
    }

    // Add completed trade to history
    pub fn add_trade(&mut self, trade_to_add: Trade) {
        let mut new_trades = self.trades.clone();
        new_trades.push(trade_to_add);
        self.trades = new_trades;
    }

    // Get quantity of shares held for a specific ticker
    pub fn get_ticker_holdings_qty(&self, ticker: &String) -> f64 {
        match self.get_holdings_map().get(ticker) {
            Some(holding) => holding.get_qty(),
            None => 0.0,
        }
    }

    // Calculate available shares after accounting for pending sell orders
    pub fn get_available_holdings_qty(&self, ticker: &String) -> f64 {
        let mut qty = self.get_ticker_holdings_qty(ticker);
        for o in &self.open_orders {
            if o.get_side() == Side::Sell && o.get_symbol() == ticker {
                qty -= o.get_qty();
            }
        }
        return qty;
    }

    // Calculate available cash after accounting for pending buy orders
    pub fn get_available_cash(&self) -> f64 {
        let mut cash = self.cash_balance;
        for o in &self.open_orders {
            if o.get_side() == Side::Buy {
                cash -= o.get_price_per() * o.get_qty();
            }
        }
        return cash;
    }

    // Get all pending orders
    pub fn get_open_orders(&self) -> Vec<OpenOrder> {
        self.open_orders.clone()
    }

    // Add pending order to order book with validation
    pub fn add_open_order(&mut self, new_order: OpenOrder) {
        if new_order.get_side() == Side::Sell {
            // Check that you have enough to sell after accounting for the existing sell orders
            if self.get_available_holdings_qty(new_order.get_symbol()) - new_order.get_qty() < 0.0 {
                println!("You don't have enough of this to sell!");
                return;
            }
        } else {
            // Check for funds after accounting for other buys
            if self.get_available_cash() < new_order.get_qty() * new_order.get_price_per() {
                println!("You don't have enough of cash for this purchase!");
                return;
            }
        }
        self.open_orders.push(new_order);
        open_order_sorting(&mut self.open_orders);
        println!("Open order added");
    }

    // Remove pending order from order book
    pub fn remove_from_open_orders(&mut self, order_to_remove: OpenOrder) {
        self.open_orders.retain(|order| {
            !(order.get_symbol() == order_to_remove.get_symbol()
                && order.get_price_per() == order_to_remove.get_price_per()
                && order.get_qty() == order_to_remove.get_qty())
        });
        open_order_sorting(&mut self.open_orders);
    }
}

// Sort orders by timestamp then by price within same symbol/side
fn open_order_sorting(order_arr: &mut Vec<OpenOrder>) {
    order_arr.sort_by_key(|o| o.get_timestamp());

    order_arr.sort_by(|a, b| {
        if a.get_side() != b.get_side() || a.get_symbol() != b.get_symbol() {
            return std::cmp::Ordering::Equal;
        }
        return if a.get_side() == Side::Buy {
            a.get_price_per().partial_cmp(&b.get_price_per()).unwrap()
        } else {
            b.get_price_per().partial_cmp(&a.get_price_per()).unwrap()
        };
    });
}

// Background thread that monitors and executes pending orders when conditions are met
pub async fn monitor_order(state: Arc<Mutex<AppState>>, running: Arc<AtomicBool>) {
    // create a thread
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        while running.load(Ordering::Relaxed) {
            {
                let mut state_guard = state.lock().unwrap();
                let open_orders: Vec<OpenOrder> = state_guard.get_open_orders();
                let mut orders_executed: Vec<OpenOrder> = Vec::new();
                for o in open_orders {
                    rt.block_on(async {
                        match o {
                            OpenOrder::BuyLimit {
                                symbol: _,
                                quantity: _,
                                price: _,
                                timestamp: _,
                            } => {
                                if crate::Orders::buy_limit(&mut state_guard, &o).await {
                                    orders_executed.push(o);
                                }
                            }
                            OpenOrder::StopLoss {
                                symbol: _,
                                quantity: _,
                                price: _,
                                timestamp: _,
                            } => {
                                if crate::Orders::sell_stop_loss(&mut state_guard, &o).await {
                                    orders_executed.push(o);
                                }
                            }
                            OpenOrder::TakeProfit {
                                symbol: _,
                                quantity: _,
                                price: _,
                                timestamp: _,
                            } => {
                                if crate::Orders::sell_take_profit(&mut state_guard, &o).await {
                                    orders_executed.push(o);
                                }
                            }
                        }
                    });
                }
                for o in orders_executed {
                    println!("Open order executed: {}", o.get_symbol());
                    state_guard.remove_from_open_orders(o);
                }
            }
            thread::sleep(Duration::from_secs(10))
        }
        println!("Order shutting down")
    });
}
