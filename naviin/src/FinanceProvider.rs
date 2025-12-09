use yfinance_rs::{Ticker, YfClient};

pub async fn print_previous_close(symbol: &str) {
    let symbol = symbol.trim();
    let client = YfClient::default();
    let ticker = Ticker::new(&client, symbol);

    match ticker.quote().await {
        Ok(quote) => match quote.previous_close {
            Some(price) => println!("Previous close: {price}"),
            None => eprintln!("{symbol} -> previous close unavailable"),
        },
        Err(err) => eprintln!("Failed to fetch {symbol} quote: {err}"),
    }
}
