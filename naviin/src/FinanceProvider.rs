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

    match ticker.quote().await {
        Ok(quote) => match quote.price{
            Some(price) => {
                if print {
                    println!("Current price: {price}");
                }
                price.amount()
            }
            None => {
                eprintln!("Current price unavailable for {symbol}");
                Decimal::ZERO
            }
        },
        Err(err) => {
            eprintln!("Failed to fetch current price for {symbol}: {err}");
            Decimal::ZERO
        }
    }
}