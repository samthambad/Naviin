use yahoo_finance_api as yahoo;

/// Returns the most recent closing price for the provided ticker, if available.
pub fn fetch_price_ticker(ticker: &str) -> Option<f64> {
    let provider = yahoo::YahooConnector::new();

    match provider.get_latest_quotes(ticker, "1d") {
        Ok(response) => match response.quotes() {
            // equivalent to quote => quote.close
            Ok(quotes) => quotes.first().map(|quote| quote.close),
            Err(err) => {
                eprintln!("Failed to parse quote data for {ticker}: {err:?}");
                None
            }
        },
        Err(err) => {
            eprintln!("Failed to fetch ticker {ticker}: {err:?}");
            None
        }
    }
}
