use rust_decimal::prelude::*;
use yfinance_rs::{Ticker, YfClient};
use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;
use std::io::Write;

pub async fn previous_price_close(symbol: &String, print: bool) -> Decimal {
    let client = YfClient::default();
    let ticker = Ticker::new(&client, symbol);

    match ticker.quote().await {
        Ok(quote) => match quote.previous_close {
            Some(price) => {
                if print {
                    println!("Previous close: {price}");
                }
                price.amount()
            }
            None => {
                eprintln!("{symbol} -> previous close unavailable");
                Decimal::ZERO
            }
        },
        Err(err) => {
            eprintln!("Failed to fetch {symbol} quote: {err}");
            Decimal::ZERO
        }
    }
}

pub async fn curr_price(symbol: &String, print: bool) -> Decimal {
    let client = YfClient::default();
    let ticker = Ticker::new(&client, symbol);

    match ticker.fast_info().await {
        Ok(fast) => match fast.last {
            Some(price) => {
                let amt = price.amount();
                if print {
                    println!("Current price: {amt}");
                }
                amt }
            None => {
                eprintln!("{symbol} -> current price unavailable");
                Decimal::ZERO }
        },
        Err(err) => {
            eprintln!("Failed to fetch {symbol} fast info: {err}");
            Decimal::ZERO }
    }
}

pub async fn stream_watchlist(symbols: Vec<String>) {
    if symbols.is_empty() {
        println!("Watchlist is empty. Add symbols first.");
        return;
    }

    use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
    use crossterm::cursor;
    use crossterm::execute;
    
    println!("\nStreaming live updates (Polling every 5s).");
    println!("Press 'x' to stop streaming.\n");
    println!("{:<10} {:<15} {:<20}", "Symbol", "Price", "Timestamp");
    println!("--------------------------------------------------");

    let client = YfClient::default();
    let num_symbols = symbols.len();
    let mut first_iteration = true;

    loop {
        // Move cursor up to overwrite previous prices (skip on first iteration)
        if !first_iteration {
            if let Ok(_) = execute!(
                std::io::stdout(),
                cursor::MoveUp(num_symbols as u16)
            ) {
                // Success - we're updating in place
            }
        }
        first_iteration = false;

        // Poll and print prices
        for sym in &symbols {
            let ticker = Ticker::new(&client, sym);
            let time = chrono::Local::now().format("%H:%M:%S");
            
            match ticker.fast_info().await {
                Ok(fast) => match fast.last {
                    Some(price) => {
                        // Clear the line and print updated price
                        print!("\r{:<10} {:<15.2} {:<20}\n", sym, price.amount(), time);
                    }
                    None => {
                        print!("\r{:<10} {:<15} {:<20}\n", sym, "N/A", time);
                    }
                },
                Err(_) => {
                    print!("\r{:<10} {:<15} {:<20}\n", sym, "Error", time);
                }
            }
            std::io::stdout().flush().unwrap();
        }

        // Wait 5 seconds, but check for 'x' keypress during that time
        let start_wait = std::time::Instant::now();
        while start_wait.elapsed() < Duration::from_secs(5) {
            // Enable raw mode to capture keypresses without Enter
            if let Ok(_) = enable_raw_mode() {
                if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                    if let Ok(Event::Key(key_event)) = event::read() {
                        if let KeyCode::Char('x') | KeyCode::Char('X') = key_event.code {
                            let _ = disable_raw_mode();
                            println!("\nExiting live prices...");
                            return;
                        }
                    }
                }
                let _ = disable_raw_mode();
            }
            tokio::task::yield_now().await;
        }
    }
}
