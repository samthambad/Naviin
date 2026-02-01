/// Command Handler Module
/// 
/// Processes user commands and executes the appropriate actions.
/// All command logic is centralized here for easy maintenance.

use std::sync::{Arc, Mutex};

use rust_decimal::Decimal;

use crate::AppState::AppState;
use crate::Finance;
use crate::FinanceProvider;
use crate::Orders;
use crate::Storage;

use sea_orm::DatabaseConnection;

/// SECTION: Command Processing

/// Main command processor - parses and executes commands
/// 
/// # Arguments
/// * `command` - The command string to process
/// * `state` - Application state (holdings, cash, etc.)
/// * `db` - Database connection for persistence
/// * `running` - Flag for background order monitoring
/// 
/// # Returns
/// Result message to display in output area
pub async fn process_command(
    command: &str,
    state: &Arc<Mutex<AppState>>,
    db: &DatabaseConnection,
    running: &Arc<std::sync::atomic::AtomicBool>,
) -> String {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    
    if parts.is_empty() {
        return "Empty command".to_string();
    }
    
    let cmd = parts[0].to_lowercase();
    let args = &parts[1..];
    
    match cmd.as_str() {
        // Account commands
        "fund" => handle_fund(state, db, args).await,
        "withdraw" => handle_withdraw(state, db, args).await,
        "summary" => handle_summary(state).await,
        
        // Price and watchlist commands
        "price" => handle_price(args).await,
        "addwatch" => handle_add_watch(state, db, args).await,
        "unwatch" => handle_remove_watch(state, db, args).await,
        
        // Trading commands
        "buy" => handle_buy(state, db, args).await,
        "sell" => handle_sell(state, db, args).await,
        "buylimit" => handle_buy_limit(state, db, args).await,
        "stoploss" => handle_stop_loss(state, db, args).await,
        "takeprofit" => handle_take_profit(state, db, args).await,
        
        // Background order commands
        "stopbg" => handle_stop_bg(running).await,
        "startbg" => handle_start_bg(running).await,
        
        // Trade history command
        "trades" => handle_trades(state).await,
        
        // System commands
        "reset" => handle_reset(state, db).await,
        "clear" => "__CLEAR__".to_string(),
        "help" => handle_help(),
        "exit" | "quit" => "Exiting...".to_string(),
        
        // Unknown command
        _ => format!("Unknown command: '{}'. Type 'help' for available commands.", cmd),
    }
}

/// SECTION: Account Commands

/// Adds funds to the account
/// Usage: fund <amount>
async fn handle_fund(
    state: &Arc<Mutex<AppState>>,
    db: &DatabaseConnection,
    args: &[&str],
) -> String {
    if args.is_empty() {
        return "Usage: fund <amount>".to_string();
    }
    
    let amount: Decimal = match args[0].parse() {
        Ok(v) => v,
        Err(_) => return "Invalid amount".to_string(),
    };
    
    if amount <= Decimal::ZERO {
        return "Amount must be positive".to_string();
    }
    
    Finance::fund(state, amount).await;
    Storage::save_state(state, db).await;
    
    format!("Added ${} to account", amount)
}

/// Withdraws funds from the account
/// Usage: withdraw <amount>
async fn handle_withdraw(
    state: &Arc<Mutex<AppState>>,
    db: &DatabaseConnection,
    args: &[&str],
) -> String {
    if args.is_empty() {
        return "Usage: withdraw <amount>".to_string();
    }
    
    let amount: Decimal = match args[0].parse() {
        Ok(v) => v,
        Err(_) => return "Invalid amount".to_string(),
    };
    
    let balance = {
        let state_guard = state.lock().unwrap();
        state_guard.check_balance()
    };
    
    if amount > balance {
        return format!("Insufficient balance. Current: ${}", balance);
    }
    
    Finance::withdraw(state, amount).await;
    Storage::save_state(state, db).await;
    
    format!("Withdrew ${} from account", amount)
}

/// Displays account summary
/// Usage: display or d
async fn handle_summary(state: &Arc<Mutex<AppState>>) -> String {
    let state_guard = state.lock().unwrap();
    let balance = state_guard.check_balance();
    let watchlist = state_guard.get_watchlist();
    let holdings_count = state_guard.get_holdings_map().len();
    
    format!(
        "Cash balance: ${}\nWatchlist: {} symbols\nHoldings: {} positions",
        balance,
        watchlist.len(),
        holdings_count
    )
}

/// SECTION: Price and Watchlist Commands

/// Gets current price for a symbol
/// Usage: price <symbol>
async fn handle_price(args: &[&str]) -> String {
    if args.is_empty() {
        return "Usage: price <symbol>".to_string();
    }
    
    let symbol = args[0].to_uppercase();
    let price = FinanceProvider::curr_price(&symbol, false).await;
    
    if price == Decimal::ZERO {
        format!("Could not fetch price for {}", symbol)
    } else {
        format!("{}: ${:.2}", symbol, price)
    }
}

/// Adds a symbol to the watchlist
/// Usage: addwatch <symbol>
async fn handle_add_watch(
    state: &Arc<Mutex<AppState>>,
    db: &DatabaseConnection,
    args: &[&str],
) -> String {
    if args.is_empty() {
        return "Usage: addwatch <symbol>".to_string();
    }
    
    let symbol = args[0].to_uppercase();
    
    {
        let mut state_guard = state.lock().unwrap();
        state_guard.add_to_watchlist(symbol.clone());
    }
    
    Storage::save_state(state, db).await;
    format!("Added {} to watchlist", symbol)
}

/// Removes a symbol from the watchlist
/// Usage: unwatch <symbol>
async fn handle_remove_watch(
    state: &Arc<Mutex<AppState>>,
    db: &DatabaseConnection,
    args: &[&str],
) -> String {
    if args.is_empty() {
        return "Usage: unwatch <symbol>".to_string();
    }
    
    let symbol = args[0].to_uppercase();
    
    {
        let mut state_guard = state.lock().unwrap();
        state_guard.remove_from_watchlist(symbol.clone());
    }
    
    Storage::save_state(state, db).await;
    format!("Removed {} from watchlist", symbol)
}

/// SECTION: Trading Commands

/// Executes a market buy order
/// Usage: buy <symbol> <quantity>
async fn handle_buy(
    state: &Arc<Mutex<AppState>>,
    db: &DatabaseConnection,
    args: &[&str],
) -> String {
    if args.len() < 2 {
        return "Usage: buy <symbol> <quantity>".to_string();
    }
    
    let symbol = args[0].to_uppercase();
    let quantity: Decimal = match args[1].parse() {
        Ok(v) => v,
        Err(_) => return "Invalid quantity".to_string(),
    };
    
    if quantity <= Decimal::ZERO {
        return "Quantity must be positive".to_string();
    }
    
    // Get current price
    let price = FinanceProvider::curr_price(&symbol, false).await;
    if price == Decimal::ZERO {
        return format!("Could not get price for {}", symbol);
    }
    
    let total_cost = price * quantity;
    
    // Check balance
    let balance = {
        let state_guard = state.lock().unwrap();
        state_guard.check_balance()
    };
    
    if total_cost > balance {
        return format!("Insufficient funds. Need ${:.2}, have ${:.2}", total_cost, balance);
    }
    
    // Execute buy
    Finance::create_buy_with_params(state, symbol.clone(), quantity, price).await;
    Storage::save_state(state, db).await;
    
    format!("Bought {} shares of {} at ${:.2} (total: ${:.2})", quantity, symbol, price, total_cost)
}

/// Executes a market sell order
/// Usage: sell <symbol> <quantity>
async fn handle_sell(
    state: &Arc<Mutex<AppState>>,
    db: &DatabaseConnection,
    args: &[&str],
) -> String {
    if args.len() < 2 {
        return "Usage: sell <symbol> <quantity>".to_string();
    }
    
    let symbol = args[0].to_uppercase();
    let quantity: Decimal = match args[1].parse() {
        Ok(v) => v,
        Err(_) => return "Invalid quantity".to_string(),
    };
    
    if quantity <= Decimal::ZERO {
        return "Quantity must be positive".to_string();
    }
    
    // Check holdings
    let available_qty = {
        let state_guard = state.lock().unwrap();
        state_guard.get_ticker_holdings_qty(&symbol)
    };
    
    if quantity > available_qty {
        return format!("Insufficient holdings. Have {:.2} shares of {}", available_qty, symbol);
    }
    
    // Get current price
    let price = FinanceProvider::curr_price(&symbol, false).await;
    if price == Decimal::ZERO {
        return format!("Could not get price for {}", symbol);
    }
    
    let total_value = price * quantity;
    
    // Execute sell
    Finance::create_sell_with_params(state, symbol.clone(), quantity, price).await;
    Storage::save_state(state, db).await;
    
    format!("Sold {} shares of {} at ${:.2} (total: ${:.2})", quantity, symbol, price, total_value)
}

/// Creates a buy limit order
/// Usage: buylimit <symbol> <quantity> <price>
async fn handle_buy_limit(
    state: &Arc<Mutex<AppState>>,
    db: &DatabaseConnection,
    args: &[&str],
) -> String {
    if args.len() < 3 {
        return "Usage: buylimit <symbol> <quantity> <price>".to_string();
    }
    
    let symbol = args[0].to_uppercase();
    let quantity: Decimal = match args[1].parse() {
        Ok(v) => v,
        Err(_) => return "Invalid quantity".to_string(),
    };
    let price: Decimal = match args[2].parse() {
        Ok(v) => v,
        Err(_) => return "Invalid price".to_string(),
    };
    
    if quantity <= Decimal::ZERO || price <= Decimal::ZERO {
        return "Quantity and price must be positive".to_string();
    }
    
    // Create order
    let order = Orders::OpenOrder::BuyLimit {
        symbol: symbol.clone(),
        quantity,
        price,
        timestamp: chrono::Utc::now().timestamp(),
    };
    
    {
        let mut state_guard = state.lock().unwrap();
        match state_guard.add_open_order(order) { Ok(msg) => msg, Err(e) => return e };
    }
    Storage::save_state(state, db).await;
    
    format!("Buy limit order created: {} shares of {} at ${:.2}", quantity, symbol, price)
}

/// Creates a stop loss order
/// Usage: stoploss <symbol> <quantity> <price>
async fn handle_stop_loss(
    state: &Arc<Mutex<AppState>>,
    db: &DatabaseConnection,
    args: &[&str],
) -> String {
    if args.len() < 3 {
        return "Usage: stoploss <symbol> <quantity> <price>".to_string();
    }
    
    let symbol = args[0].to_uppercase();
    let quantity: Decimal = match args[1].parse() {
        Ok(v) => v,
        Err(_) => return "Invalid quantity".to_string(),
    };
    let price: Decimal = match args[2].parse() {
        Ok(v) => v,
        Err(_) => return "Invalid price".to_string(),
    };
    
    if quantity <= Decimal::ZERO || price <= Decimal::ZERO {
        return "Quantity and price must be positive".to_string();
    }
    
    // Check holdings
    let available_qty = {
        let state_guard = state.lock().unwrap();
        state_guard.get_ticker_holdings_qty(&symbol)
    };
    
    if quantity > available_qty {
        return format!("Insufficient holdings. Have {:.2} shares of {}", available_qty, symbol);
    }
    
    // Create order
    let order = Orders::OpenOrder::StopLoss {
        symbol: symbol.clone(),
        quantity,
        price,
        timestamp: chrono::Utc::now().timestamp(),
    };
    
    {
        let mut state_guard = state.lock().unwrap();
        match state_guard.add_open_order(order) { Ok(msg) => msg, Err(e) => return e };
    }
    Storage::save_state(state, db).await;
    
    format!("Stop loss order created: {} shares of {} at ${:.2}", quantity, symbol, price)
}

/// Creates a take profit order
/// Usage: takeprofit <symbol> <quantity> <price>
async fn handle_take_profit(
    state: &Arc<Mutex<AppState>>,
    db: &DatabaseConnection,
    args: &[&str],
) -> String {
    if args.len() < 3 {
        return "Usage: takeprofit <symbol> <quantity> <price>".to_string();
    }
    
    let symbol = args[0].to_uppercase();
    let quantity: Decimal = match args[1].parse() {
        Ok(v) => v,
        Err(_) => return "Invalid quantity".to_string(),
    };
    let price: Decimal = match args[2].parse() {
        Ok(v) => v,
        Err(_) => return "Invalid price".to_string(),
    };
    
    if quantity <= Decimal::ZERO || price <= Decimal::ZERO {
        return "Quantity and price must be positive".to_string();
    }
    
    // Check holdings
    let available_qty = {
        let state_guard = state.lock().unwrap();
        state_guard.get_ticker_holdings_qty(&symbol)
    };
    
    if quantity > available_qty {
        return format!("Insufficient holdings. Have {:.2} shares of {}", available_qty, symbol);
    }
    
    // Create order
    let order = Orders::OpenOrder::TakeProfit {
        symbol: symbol.clone(),
        quantity,
        price,
        timestamp: chrono::Utc::now().timestamp(),
    };
    
    {
        let mut state_guard = state.lock().unwrap();
        match state_guard.add_open_order(order) { Ok(msg) => msg, Err(e) => return e };
    }
    Storage::save_state(state, db).await;
    
    format!("Take profit order created: {} shares of {} at ${:.2}", quantity, symbol, price)
}

/// SECTION: Background Order Commands

/// Stops background order monitoring
/// Usage: stopbg
async fn handle_stop_bg(running: &Arc<std::sync::atomic::AtomicBool>) -> String {
    running.store(false, std::sync::atomic::Ordering::Relaxed);
    "Background order monitoring stopped".to_string()
}

/// Starts background order monitoring
/// Usage: startbg
async fn handle_start_bg(running: &Arc<std::sync::atomic::AtomicBool>) -> String {
    running.store(true, std::sync::atomic::Ordering::Relaxed);
    "Background order monitoring started".to_string()
}

/// SECTION: Trade History

/// Displays trade history
/// Usage: trades
async fn handle_trades(state: &Arc<Mutex<AppState>>) -> String {
    let state_guard = state.lock().unwrap();
    state_guard.display_trades()
}

/// SECTION: System Commands

/// Resets all data to default state
/// Usage: reset
async fn handle_reset(state: &Arc<Mutex<AppState>>, db: &DatabaseConnection) -> String {
    Storage::default_state(state, db).await;
    "Account reset to default state".to_string()
}

/// Displays help information
/// Usage: help
fn handle_help() -> String {
    String::from(
        "Available Commands:\n\n\
        ACCOUNT:\n\
        fund <amount>              - Add funds to account\n\
        withdraw <amount>          - Withdraw funds from account\n\
        summary                    - Show summary of finances\n\
        PRICES & WATCHLIST:\n\
        price <symbol>             - Get current price for symbol\n\
        addwatch <symbol>          - Add symbol to watchlist\n\
        unwatch <symbol>           - Remove symbol from watchlist\n\n\
        TRADING:\n\
        buy <symbol> <qty>         - Buy shares at market price\n\
        sell <symbol> <qty>        - Sell shares at market price\n\
        buylimit <sym> <qty> <pr>  - Create buy limit order\n\
        stoploss <sym> <qty> <pr>  - Create stop loss order\n\
        takeprofit <sym> <qty> <pr> - Create take profit order\n\
        trades                     - Show trade history\n\n\
        SYSTEM:\n\
        stopbg                     - Stop background orders\n\
        startbg                    - Start background orders\n\
        reset                      - Reset all data\n\
        clear                      - Clear screen\n\
        help                       - Show this help\n\
        exit, quit                 - Exit application\n\n\
        NAVIGATION:\n\
        PgUp/PgDn                  - Scroll output\n\
        Ctrl+Home/Ctrl+End         - Output top/bottom"
    )
}
