use rust_decimal::prelude::*;
use yfinance_rs::{Ticker, YfClient};
use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;

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
    
    println!("\nStreaming live updates (Polling every 5s).");
    println!("Press 'x' to stop streaming.");
    println!("{:<10} {:<15} {:<20}", "Symbol", "Price", "Timestamp");
    println!("--------------------------------------------------");

    let client = YfClient::default();

    loop {
        // Poll prices
        for sym in &symbols {
            let ticker = Ticker::new(&client, sym);
            let time = chrono::Local::now().format("%H:%M:%S");
            
            match ticker.fast_info().await {
                Ok(fast) => match fast.last {
                    Some(price) => {
                        println!("{:<10} {:<15.2} {:<20}", sym, price.amount(), time);
                    }
                    None => {
                        println!("{:<10} {:<15} {:<20}", sym, "N/A", time);
                    }
                },
                Err(_) => {
                    println!("{:<10} {:<15} {:<20}", sym, "Error", time);
                }
            }
        }
        
        println!("---");

        // Wait 5 seconds, but check for 'x' keypress during that time
        let start_wait = std::time::Instant::now();
        while start_wait.elapsed() < Duration::from_secs(5) {
            // Enable raw mode to capture keypresses without Enter
            if let Ok(_) = enable_raw_mode() {
                if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                    if let Ok(Event::Key(key_event)) = event::read() {
                        if let KeyCode::Char('x') | KeyCode::Char('X') = key_event.code {
                            let _ = disable_raw_mode();
                            println!("Exiting stream...");
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
