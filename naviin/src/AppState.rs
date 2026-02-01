use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rust_decimal::prelude::*;
use chrono;

use crate::Finance::{Holding, Symbol};
use crate::FinanceProvider;
use crate::Orders::{OpenOrder, Side, Trade};

// Manages user account state including cash, holdings, trades, and pending orders
#[derive(Debug)]
pub struct AppState {
    cash_balance: Decimal,
    holdings: HashMap<Symbol, Holding>,
    trades: Vec<Trade>,
    open_orders: Vec<OpenOrder>,
    watchlist: Vec<Symbol>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            cash_balance: Decimal::ZERO,
            holdings: HashMap::new(),
            trades: Vec::new(),
            open_orders: Vec::new(),
            watchlist: Vec::new(),
        }
    }

    pub fn deposit(&mut self, amount: Decimal) {
        self.cash_balance += amount;
    }

    // Withdraw funds with validation
    pub fn withdraw(&mut self, amount: Decimal) {
        if amount < Decimal::ZERO {
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
    pub fn withdraw_purchase(&mut self, amount: Decimal) {
        if amount < Decimal::ZERO {
            println!("Invalid amount");
            return;
        }
        self.cash_balance -= amount;
    }

    // Add sale proceeds to balance
    pub fn deposit_sell(&mut self, amount: Decimal) {
        self.cash_balance += amount;
    }

    // Get current cash balance
    pub fn check_balance(&self) -> Decimal {
        self.cash_balance
    }

    pub fn set_cash_balance(&mut self, new_balance: Decimal) {
        self.cash_balance = new_balance;
    }

    // Get all holdings as copy of internal map
    pub fn get_holdings_map(&self) -> HashMap<Symbol, Holding> {
        self.holdings.clone()
    }

    // Update holdings and refresh display
    pub async fn set_holdings_map(&mut self, new_holdings_map: HashMap<Symbol, Holding>) {
        self.holdings = new_holdings_map;
    }

    // Add completed trade to history
    pub fn add_trade(&mut self, trade_to_add: Trade) {
        let mut new_trades = self.trades.clone();
        new_trades.push(trade_to_add);
        self.trades = new_trades;
    }
    
    pub fn set_trades(&mut self, new_trades: Vec<Trade>) {
        self.trades = new_trades;
    }
    
    pub fn get_trades(&self) -> Vec<Trade> {
        self.trades.clone()
    }

    /// Formats trade history as a string for TUI display
    /// Returns formatted string or "No trades yet" if empty
    pub fn display_trades(&self) -> String {
        if self.trades.is_empty() {
            return "No trades yet".to_string();
        }
        
        let mut result = String::from("Trade History:\n");
        result.push_str("────────────────────────────────────────────────────────────\n");
        result.push_str(&format!("{:<10} {:<8} {:<6} {:<8} {:<12} {:<16}\n", "Type", "Symbol", "Side", "Qty", "Price", "Time"));
        result.push_str("────────────────────────────────────────────────────────────\n");
        
        for trade in self.trades.iter().rev().take(20) { // Show last 20, most recent first
            let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(trade.get_timestamp(), 0)
                .map(|dt| dt.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            
            let side = match trade.get_side() {
                Side::Buy => "BUY",
                Side::Sell => "SELL",
            };
            
            result.push_str(&format!(
                "{:<10} {:<8} {:<6} {:<8} ${:<11.2} {:<16}\n",
                trade.get_order_type(),
                trade.get_symbol(),
                side,
                trade.get_quantity(),
                trade.get_price_per(),
                datetime
            ));
        }
        
        if self.trades.len() > 20 {
            result.push_str(&format!("\n... and {} more trades", self.trades.len() - 20));
        }
        
        result
    }

    pub fn add_to_watchlist(&mut self, symbol: Symbol) {
        if !self.watchlist.contains(&symbol) {
            self.watchlist.push(symbol);
            println!("Added to watchlist");
        } else {
            println!("Already in watchlist");
        }
    }

    pub fn remove_from_watchlist(&mut self, symbol: Symbol) {
        if let Some(pos) = self.watchlist.iter().position(|x| *x == symbol) {
            self.watchlist.remove(pos);
            println!("Removed from watchlist");
        } else {
            println!("Not in watchlist");
        }
    }

    pub fn get_watchlist(&self) -> Vec<Symbol> {
        self.watchlist.clone()
    }

    pub fn set_watchlist(&mut self, watchlist: Vec<Symbol>) {
        self.watchlist = watchlist;
    }

    // Get quantity of shares held for a specific ticker
    pub fn get_ticker_holdings_qty(&self, ticker: &String) -> Decimal {
        match self.get_holdings_map().get(ticker) {
            Some(holding) => holding.get_qty(),
            None => Decimal::ZERO,
        }
    }

    // Calculate available shares after accounting for pending sell orders
    pub fn get_available_holdings_qty(&self, ticker: &String) -> Decimal {
        let mut qty = self.get_ticker_holdings_qty(ticker);
        for o in &self.open_orders {
            if o.get_side() == Side::Sell && o.get_symbol() == ticker {
                qty -= o.get_qty();
            }
        }
        qty
    }

    // Calculate available cash after accounting for pending buy orders
    pub fn get_available_cash(&self) -> Decimal {
        let mut cash = self.cash_balance;
        for o in &self.open_orders {
            if o.get_side() == Side::Buy {
                cash -= o.get_price_per() * o.get_qty();
            }
        }
        cash
    }

    // Get all pending orders
    pub fn get_open_orders(&self) -> Vec<OpenOrder> {
        self.open_orders.clone()
    }

    pub fn set_open_orders(&mut self, new_open_orders: Vec<OpenOrder>) {
        self.open_orders = new_open_orders;
        open_order_sorting(&mut self.open_orders);
    }

    // Add pending order to order book with validation
    // Returns Ok(message) on success, Err(message) on failure
    pub fn add_open_order(&mut self, new_order: OpenOrder) -> Result<String, String> {
        if new_order.get_side() == Side::Sell {
            // Check that you have enough to sell after accounting for existing sell orders
            if self.get_available_holdings_qty(new_order.get_symbol()) - new_order.get_qty() < Decimal::ZERO {
                return Err("You don't have enough of this to sell!".to_string());
            }
        } else {
            // Check for funds after accounting for other buys
            if self.get_available_cash() < new_order.get_qty() * new_order.get_price_per() {
                return Err("You don't have enough of cash for this purchase!".to_string());
            }
        }
        let symbol = new_order.get_symbol().clone();
        let order_type = new_order.get_order_type().to_string();
        self.open_orders.push(new_order);
        open_order_sorting(&mut self.open_orders);
        Ok(format!("{} order added for {}", order_type, symbol))
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
            a.get_price_per().cmp(&b.get_price_per())
        } else {
            b.get_price_per().cmp(&a.get_price_per())
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
                    crate::message_queue::send_order_executed(o.get_order_type(), o.get_symbol());
                    state_guard.remove_from_open_orders(o);
                }
            }
            thread::sleep(Duration::from_secs(10))
        }
        println!("Order shutting down")
    });
}
