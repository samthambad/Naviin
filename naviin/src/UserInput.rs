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

pub async fn check_input_now() -> io::Result<String> {
    use tokio::io::AsyncBufReadExt;
    let mut reader = tokio::io::BufReader::new(tokio::io::stdin());
    let mut line = String::new();
    
    // We use a select with a small timeout to check if stdin has data
    // without blocking the caller indefinitely
    tokio::select! {
        result = reader.read_line(&mut line) => {
            result.map(|_| line)
        }
        _ = tokio::time::sleep(tokio::time::Duration::from_millis(10)) => {
            Err(io::Error::new(io::ErrorKind::WouldBlock, "No input available"))
        }
    }
}

