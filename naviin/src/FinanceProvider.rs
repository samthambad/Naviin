use yfinance_rs::{Ticker, YfClient};
use rust_decimal::prelude::*;

pub async fn previous_price_close(symbol: &String, print: bool) -> f64 {
    let client = YfClient::default();
    let ticker = Ticker::new(&client, symbol);

    match ticker.quote().await {
        Ok(quote) => match quote.previous_close {
            Some(price) => {
                if print {
                    println!("Previous close: {price}");
                }
                price.amount().to_f64().unwrap()
            },
            None =>{
                eprintln!("{symbol} -> previous close unavailable");
                0.0
            },
        },
        Err(err) => {
            eprintln!("Failed to fetch {symbol} quote: {err}");
            0.0
        },
    }
}
