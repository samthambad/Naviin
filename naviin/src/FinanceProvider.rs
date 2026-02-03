use rust_decimal::prelude::*;
use yfinance_rs::{Ticker, YfClient};

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
