use rust_decimal::prelude::*;
use yfinance_rs::{Ticker, YfClient, StreamBuilder, StreamMethod};

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

    println!("\nStreaming live updates (Polling every 5s). Press Ctrl+C to exit.");
    println!("{:<10} {:<15} {:<20}", "Symbol", "Price", "Timestamp");
    println!("--------------------------------------------------");

    let client = YfClient::default();

    loop {
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
        
        // Sleep for 5 seconds before next poll
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        println!("---"); // Separator for readability
    }
}
