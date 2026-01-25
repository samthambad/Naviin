use rust_decimal::Decimal;
use std::io::{self, Write};

pub fn ask_ticker() -> Option<String> {
    loop {
        print!("Enter the ticker (or 'cancel' to go back): ");
        if let Err(e) = io::stdout().flush() {
            eprintln!("Failed to flush stdout: {}", e);
            continue;
        }
        let mut ticker = String::new();
        match io::stdin().read_line(&mut ticker) {
            Ok(_) => {
                ticker.retain(|c| !c.is_whitespace());
                if ticker.is_empty() {
                    println!("Ticker cannot be empty. Please try again.");
                    continue;
                }
                if ticker.eq_ignore_ascii_case("cancel") {
                    return None;
                }
                return Some(ticker.to_uppercase());
            }
            Err(error) => println!("Error reading input: {}. Please try again.", error),
        }
    }
}

fn get_user_input_f64(prompt: &str, error_label: &str) -> Option<Decimal> {
    loop {
        print!("{}: ", prompt);
        io::stdout().flush().ok(); // Simplified for brevity

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            continue;
        }

        let trimmed = input.trim();
        if trimmed.eq_ignore_ascii_case("cancel") {
            return None;
        }

        match trimmed.parse::<Decimal>() {
            Ok(num) if num > Decimal::ZERO => return Some(num),
            Ok(_) => println!("Please enter a positive {}.", error_label),
            Err(_) => println!("Invalid {}. Please try again.", error_label),
        }
    }
}

pub fn ask_quantity() -> Option<Decimal> {
    get_user_input_f64("Enter the quantity (or 'cancel' to go back)", "quantity")
}

pub fn ask_price() -> Option<Decimal> {
    get_user_input_f64("Enter the price (or 'cancel' to go back)", "price")
}

pub fn display_help() {
    println!("Available Commands:");
    println!("  fund              - Deposit funds into your account.");
    println!("  withdraw          - Withdraw funds from your account.");
    println!("  buy               - Purchase shares of a stock. You will be prompted for ticker and quantity.");
    println!("  buylimit          - Purchase shares of a stock <= your limit price, good till cancelled.");
    println!("  sell              - Sell shares of a stock. You will be prompted for ticker and quantity.");
    println!("  stoploss          - Sell shares of a stock when price <= your limit price, good till cancelled.");
    println!("  startbg           - Allow open orders to run execution in the background.");
    println!("  stopbg            - Stop open orders from running execution in the background.");
    println!(
        "  display           - Show your current cash balance, holdings, and their unrealized P&L."
    );
    println!("  price             - Get the current market price for a specified stock ticker.");
    println!("  watch             - Display live price updates for stocks in your watchlist.");
    println!("  addwatch          - Add a stock to your watchlist.");
    println!("  unwatch           - Remove a stock from your watchlist.");
    println!("  reset             - Clear all your financial data and start fresh.");
    println!("  exit              - Save your session and exit the application.");
    println!("  help              - Display this help message.");
}
