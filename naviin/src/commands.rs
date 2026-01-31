/// Command Handler Module
/// 
/// Processes user commands and executes the appropriate actions.
/// All command logic is centralized here for easy maintenance.

use std::sync::{Arc, Mutex};

use rust_decimal::Decimal;

use crate::AppState::AppState;
use crate::Finance;
use crate::FinanceProvider;
use crate::Orders::{self, OrderType};
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
        "display" | "d" => handle_display(state).await,
        
        // Price and watchlist commands
        "price" => handle_price(args).await,
        "watch" => handle_watch_stream(state).await,
        "addwatch" => handle_add_watch(state, db, args).await,
        "unwatch" => handle_remove_watch(state, db, args).await,
        
        // Trading commands
        "buy" => handle_buy(state, db).await,
        "sell" => handle_sell(state, db).await,
        "buylimit" => handle_buy_limit(state, db).await,
        "stoploss" => handle_stop_loss(state, db).await,
        "takeprofit" => handle_take_profit(state, db).await,
        
        // Background order commands
        "stopbg" => handle_stop_bg(running).await,
        "startbg" => handle_start_bg(running).await,
        
        // System commands
        "reset" => handle_reset(state, db).await,
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
async fn handle_display(state: &Arc<Mutex<AppState>>) -> String {
    let state_guard = state.lock().unwrap();
    let balance = state_guard.check_balance();
    let watchlist = state_guard.get_watchlist();
    let holdings_count = state_guard.get_holdings_map().len();
    
    format!(
        "Balance: ${}\nWatchlist: {} symbols\nHoldings: {} positions",
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

/// Streams watchlist prices live
/// Usage: watch
async fn handle_watch_stream(state: &Arc<Mutex<AppState>>) -> String {
    let watchlist = {
        let state_guard = state.lock().unwrap();
        state_guard.get_watchlist()
    };
    
    if watchlist.is_empty() {
        return "Watchlist is empty. Use 'addwatch <symbol>' to add symbols.".to_string();
    }
    
    // Note: In full implementation, this would start a streaming task
    // For now, just return current prices
    let mut result = String::from("Watchlist:\n");
    for symbol in watchlist {
        let price = FinanceProvider::curr_price(&symbol, false).await;
        result.push_str(&format!("  {}: ${:.2}\n", symbol, price));
    }
    result
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

/// Executes a buy order
/// Usage: buy (interactive)
async fn handle_buy(_state: &Arc<Mutex<AppState>>, _db: &DatabaseConnection) -> String {
    // For interactive commands, we need to handle input differently in TUI
    // For now, return a message indicating this needs ticker input
    "Buy command: Enter 'buy <symbol> <quantity>' or use interactive mode".to_string()
}

/// Executes a sell order
/// Usage: sell (interactive)
async fn handle_sell(_state: &Arc<Mutex<AppState>>, _db: &DatabaseConnection) -> String {
    "Sell command: Enter 'sell <symbol> <quantity>' or use interactive mode".to_string()
}

/// Creates a buy limit order
/// Usage: buylimit (interactive)
async fn handle_buy_limit(state: &Arc<Mutex<AppState>>, db: &DatabaseConnection) -> String {
    if let Some(order) = Orders::create_order(OrderType::BuyLimit) {
        let symbol = order.get_symbol().clone();
        {
            let mut state_guard = state.lock().unwrap();
            state_guard.add_open_order(order);
        }
        Storage::save_state(state, db).await;
        format!("Buy limit order created for {}", symbol)
    } else {
        "Failed to create buy limit order".to_string()
    }
}

/// Creates a stop loss order
/// Usage: stoploss (interactive)
async fn handle_stop_loss(state: &Arc<Mutex<AppState>>, db: &DatabaseConnection) -> String {
    if let Some(order) = Orders::create_order(OrderType::StopLoss) {
        let symbol = order.get_symbol().clone();
        {
            let mut state_guard = state.lock().unwrap();
            state_guard.add_open_order(order);
        }
        Storage::save_state(state, db).await;
        format!("Stop loss order created for {}", symbol)
    } else {
        "Failed to create stop loss order".to_string()
    }
}

/// Creates a take profit order
/// Usage: takeprofit (interactive)
async fn handle_take_profit(state: &Arc<Mutex<AppState>>, db: &DatabaseConnection) -> String {
    if let Some(order) = Orders::create_order(OrderType::TakeProfit) {
        let symbol = order.get_symbol().clone();
        {
            let mut state_guard = state.lock().unwrap();
            state_guard.add_open_order(order);
        }
        Storage::save_state(state, db).await;
        format!("Take profit order created for {}", symbol)
    } else {
        "Failed to create take profit order".to_string()
    }
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
        fund <amount>      - Add funds to account\n\
        withdraw <amount>  - Withdraw funds from account\n\
        display, d         - Show account summary\n\n\
        PRICES & WATCHLIST:\n\
        price <symbol>     - Get current price for symbol\n\
        watch              - Show watchlist with prices\n\
        addwatch <symbol>  - Add symbol to watchlist\n\
        unwatch <symbol>   - Remove symbol from watchlist\n\n\
        TRADING:\n\
        buy                - Buy shares (interactive)\n\
        sell               - Sell shares (interactive)\n\
        buylimit           - Create buy limit order\n\
        stoploss           - Create stop loss order\n\
        takeprofit         - Create take profit order\n\n\
        SYSTEM:\n\
        stopbg             - Stop background orders\n\
        startbg            - Start background orders\n\
        reset              - Reset all data\n\
        help               - Show this help\n\
        exit, quit         - Exit application"
    )
}
