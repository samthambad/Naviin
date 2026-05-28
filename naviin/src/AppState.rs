use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono;
use rust_decimal::prelude::*;
use tokio::time;

use crate::Finance::{Holding, Symbol};
use crate::Orders::{OpenOrder, OrderType, Side, Trade};

// Manages user account state including cash, holdings, trades, and pending orders
#[derive(Debug)]
pub struct AppState {
    cash_balance: Decimal,
    holdings: HashMap<Symbol, Holding>,
    trades: Vec<Trade>,
    open_orders: Vec<OpenOrder>,
    watchlist: Vec<Symbol>,
    pending_import: bool,
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
            pending_import: false,
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
        result.push_str(&format!(
            "{:<10} {:<8} {:<6} {:<8} {:<12} {:<16}\n",
            "Type", "Symbol", "Side", "Qty", "Price", "Time"
        ));
        result.push_str("────────────────────────────────────────────────────────────\n");

        for trade in self.trades.iter().rev().take(20) {
            // Show last 20, most recent first
            let datetime =
                chrono::DateTime::<chrono::Utc>::from_timestamp(trade.get_timestamp(), 0)
                    .map(|dt| {
                        dt.with_timezone(&chrono::Local)
                            .format("%Y-%m-%d %H:%M")
                            .to_string()
                    })
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

    pub fn add_to_watchlist(&mut self, symbol: Symbol) -> bool {
        if !self.watchlist.contains(&symbol) {
            self.watchlist.push(symbol);
            return true;
        }
        false
    }

    pub fn remove_from_watchlist(&mut self, symbol: Symbol) -> bool {
        if let Some(pos) = self.watchlist.iter().position(|x| *x == symbol) {
            self.watchlist.remove(pos);
            return true;
        }
        false
    }

    pub fn get_watchlist(&self) -> Vec<Symbol> {
        self.watchlist.clone()
    }

    pub fn set_watchlist(&mut self, watchlist: Vec<Symbol>) {
        self.watchlist = watchlist;
    }

    pub fn set_pending_import(&mut self, pending: bool) {
        self.pending_import = pending;
    }

    pub fn is_pending_import(&self) -> bool {
        self.pending_import
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
            if self.get_available_holdings_qty(new_order.get_symbol()) - new_order.get_qty()
                < Decimal::ZERO
            {
                return Err("You don't have enough of this to sell!".to_string());
            }
        } else {
            // Check for funds after accounting for other buys
            if self.get_available_cash() < new_order.get_qty() * new_order.get_price_per() {
                return Err("You don't have enough of cash for this purchase!".to_string());
            }
        }
        let symbol = new_order.get_symbol().clone();
        let order_type = format!("{:?}", new_order.get_order_type());
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
        if a.get_side() == Side::Buy {
            a.get_price_per().cmp(&b.get_price_per())
        } else {
            b.get_price_per().cmp(&a.get_price_per())
        }
    });
}

// Background task that monitors and executes pending orders when conditions are met
pub fn monitor_order(state: Arc<Mutex<AppState>>, running: Arc<AtomicBool>) {
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(10));

        while running.load(Ordering::Relaxed) {
            interval.tick().await;

            let open_orders = {
                let state_guard = state.lock().unwrap();
                state_guard.get_open_orders()
            };

            let mut priced_orders = Vec::new();
            for order in open_orders {
                let symbol = order.get_symbol().clone();
                let current_price = crate::FinanceProvider::curr_price(&symbol, false).await;
                priced_orders.push((order, current_price));
            }

            let mut state_guard = state.lock().unwrap();
            for (order, current_price) in priced_orders {
                if execute_order_with_price(&mut state_guard, &order, current_price) {
                    state_guard.remove_from_open_orders(order);
                }
            }
        }
        println!("Order shutting down");
    });
}

fn execute_order_with_price(
    state: &mut AppState,
    order: &OpenOrder,
    current_price: Decimal,
) -> bool {
    match order.get_order_type() {
        OrderType::BuyLimit => execute_buy_limit_with_price(state, order, current_price),
        OrderType::StopLoss => execute_stop_loss_with_price(state, order, current_price),
        OrderType::TakeProfit => execute_take_profit_with_price(state, order, current_price),
    }
}

fn execute_buy_limit_with_price(
    state: &mut AppState,
    order: &OpenOrder,
    current_price: Decimal,
) -> bool {
    let symbol = order.get_symbol().clone();
    let limit_price = order.get_price_per();
    let purchase_qty = order.get_qty();
    let total_purchase_value = current_price * purchase_qty;

    if current_price > limit_price || total_purchase_value > state.check_balance() {
        return false;
    }

    state.withdraw_purchase(total_purchase_value);
    add_to_holdings(state, &symbol, purchase_qty, current_price);
    state.add_trade(Trade::buy_with_type(
        symbol,
        purchase_qty,
        current_price,
        "BuyLimit".to_string(),
    ));
    true
}

fn execute_stop_loss_with_price(
    state: &mut AppState,
    order: &OpenOrder,
    current_price: Decimal,
) -> bool {
    let symbol = order.get_symbol().clone();
    let stop_price = order.get_price_per();
    let sale_qty = order.get_qty();

    if current_price > stop_price {
        return false;
    }

    state.deposit_sell(current_price * sale_qty);
    remove_from_holdings(state, &symbol, sale_qty);
    state.add_trade(Trade::sell_with_type(
        symbol,
        sale_qty,
        current_price,
        "StopLoss".to_string(),
    ));
    true
}

fn execute_take_profit_with_price(
    state: &mut AppState,
    order: &OpenOrder,
    current_price: Decimal,
) -> bool {
    let symbol = order.get_symbol().clone();
    let take_profit_price = order.get_price_per();
    let sale_qty = order.get_qty();

    if current_price < take_profit_price {
        return false;
    }

    state.deposit_sell(take_profit_price * sale_qty);
    remove_from_holdings(state, &symbol, sale_qty);
    state.add_trade(Trade::sell_with_type(
        symbol,
        sale_qty,
        take_profit_price,
        "TakeProfit".to_string(),
    ));
    true
}

fn add_to_holdings(state: &mut AppState, ticker: &String, quantity: Decimal, price_per: Decimal) {
    if let Some(existing_holding) = state.holdings.get(ticker) {
        let prev_avg_cost = existing_holding.get_avg_price();
        let prev_qty = existing_holding.get_qty();
        let new_avg_cost =
            (prev_qty * prev_avg_cost + quantity * price_per) / (prev_qty + quantity);
        let new_qty = prev_qty + quantity;

        state.holdings.insert(
            ticker.clone(),
            Holding::new(ticker.clone(), new_qty, new_avg_cost),
        );
    } else {
        state.holdings.insert(
            ticker.clone(),
            Holding::new(ticker.clone(), quantity, price_per),
        );
    }
}

fn remove_from_holdings(state: &mut AppState, ticker: &String, quantity: Decimal) {
    if let Some(existing_holding) = state.holdings.get(ticker) {
        let prev_avg_cost = existing_holding.get_avg_price();
        let prev_qty = existing_holding.get_qty();
        let new_qty = prev_qty - quantity;

        if new_qty == Decimal::ZERO {
            state.holdings.remove(ticker);
        } else {
            state.holdings.insert(
                ticker.clone(),
                Holding::new(ticker.clone(), new_qty, prev_avg_cost),
            );
        }
    }
}
