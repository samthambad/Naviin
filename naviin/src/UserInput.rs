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

pub fn ask_quantity() -> Option<f64> {
    loop {
        print!("Enter the quantity (or 'cancel' to go back): ");
        if let Err(e) = io::stdout().flush() {
            eprintln!("Failed to flush stdout: {}", e);
            continue;
        }
        let mut quantity = String::new();
        match io::stdin().read_line(&mut quantity) {
            Ok(_) => {
                let trimmed = quantity.trim();
                if trimmed.eq_ignore_ascii_case("cancel") {
                    return None;
                }
                match trimmed.parse::<f64>() {
                    Ok(num) => {
                        if num <= 0.0 {
                            println!("Enter a positive quantity");
                            continue;
                        }
                        return Some(num);
                    }
                    Err(_) => println!("Invalid number entered. Please enter a valid quantity."),
                }
            },
            Err(error) => println!("Error reading input: {}. Please try again.", error),
        }
    }
}