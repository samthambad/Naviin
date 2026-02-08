use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

use rust_decimal::Decimal;

use crate::AppState::AppState;
use crate::Finance;
use crate::Orders::{Side, Trade};

#[derive(Debug)]
struct CsvTradeRow {
    date: String,
    asset: String,
    asset_type: String,
    side: Side,
    quantity: Decimal,
    price: Decimal,
    currency: Option<String>,
}

pub async fn import_trades_from_csv(
    state: &std::sync::Arc<std::sync::Mutex<AppState>>,
    path: &str,
) -> Result<String, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file: {e}"))?;
    let reader = BufReader::new(file);

    let mut lines = reader.lines();
    let header_line = match lines.next() {
        Some(Ok(line)) => line,
        Some(Err(e)) => return Err(format!("Failed to read file: {e}")),
        None => return Err("CSV is empty".to_string()),
    };

    let headers = parse_csv_row(&header_line);
    let header_map = build_header_map(&headers);

    for required in ["date", "asset", "asset_type", "side", "quantity", "price"] {
        if !header_map.contains_key(required) {
            return Err(format!("Missing required column: {required}"));
        }
    }





fn build_header_map(headers: &[String]) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    for (idx, header) in headers.iter().enumerate() {
        let key = header.trim().to_lowercase();
        if !key.is_empty() {
            map.insert(key, idx);
        }
    }
    map
}

fn parse_csv_row(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                if in_quotes && chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = !in_quotes;
                }
            }
            ',' if !in_quotes => {
                out.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    out.push(current.trim().to_string());
    out
}

