use std::io::{self, Write};

pub fn ask_ticker() -> String {
    loop {
        print!("Enter the ticker: ");
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
                return ticker.to_uppercase();
            }
            Err(error) => println!("Error reading input: {}. Please try again.", error),
        }
    }
}

pub fn ask_quantity() -> f64 {
    loop {
        print!("Enter the quantity: ");
        if let Err(e) = io::stdout().flush() {
            eprintln!("Failed to flush stdout: {}", e);
            continue;
        }
        let mut quantity = String::new();
        match io::stdin().read_line(&mut quantity) {
            Ok(_) => match quantity.trim().parse::<f64>() {
                Ok(num) => {
                    if (num <= 0.0) {
                        println!("Enter a positive quantity");
                        continue;
                    }
                    return num;
                    
                }
                Err(_) => println!("Invalid number entered. Please enter a valid quantity."),
            },
            Err(error) => println!("Error reading input: {}. Please try again.", error),
        }
    }
}