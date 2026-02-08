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

    // column headings need not follow order
    for required in ["date", "asset", "asset_type", "side", "quantity", "price"] {
        if !header_map.contains_key(required) {
            return Err(format!("Missing required column: {required}"));
        }
    }

    let mut imported = 0usize;
    let mut skipped = 0usize;
    let mut errors = 0usize;
    let mut last_errors: Vec<String> = Vec::new();

    for (idx, line) in lines.enumerate() {
        let line_number = idx + 2; // header is line 1
        let raw = match line {
            Ok(l) => l,
            Err(e) => {
                errors += 1;
                skipped += 1;
                push_error(&mut last_errors, format!("Line {line_number}: {e}"));
                continue;
            }
        };

        if raw.trim().is_empty() {
            continue;
        }

        let cols = parse_csv_row(&raw);
        let row = match parse_trade_row(&cols, &header_map) {
            Ok(row) => row,
            Err(msg) => {
                errors += 1;
                skipped += 1;
                push_error(&mut last_errors, format!("Line {line_number}: {msg}"));
                continue;
            }
        };

        match row.side {
            Side::Buy => {
                let mut trade = Trade::buy(row.asset.clone(), row.quantity, row.price);
                trade.set_timestamp(parse_date_to_timestamp(&row.date));
                {
                    let mut guard = state.lock().unwrap();
                    guard.add_trade(trade);
                }
                Finance::add_to_holdings(&row.asset, row.quantity, row.price, &mut state.lock().unwrap()).await;
            }
            Side::Sell => {
                let available_qty = { state.lock().unwrap().get_ticker_holdings_qty(&row.asset) };
                if available_qty < row.quantity {
                    errors += 1;
                    skipped += 1;
                    push_error(
                        &mut last_errors,
                        format!(
                            "Line {line_number}: Insufficient holdings for {} (have {}, need {})",
                            row.asset, available_qty, row.quantity
                        ),
                    );
                    continue;
                }
                let mut trade = Trade::sell(row.asset.clone(), row.quantity, row.price);
                trade.set_timestamp(parse_date_to_timestamp(&row.date));
                {
                    let mut guard = state.lock().unwrap();
                    guard.add_trade(trade);
                }
                Finance::remove_from_holdings(&row.asset, row.quantity, &mut state.lock().unwrap()).await;
            }
        }
        imported += 1;
    }

    if imported == 0 && errors > 0 {
        return Err(format!(
            "No trades imported. Errors: {errors}. Example: {}",
            last_errors.join(" | ")
        ));
    }

    if errors > 0 {
        Ok(format!(
            "Imported {imported} trades ({skipped} skipped). {errors} errors. Example: {}",
            last_errors.join(" | ")
        ))
    } else {
        Ok(format!("Imported {imported} trades ({skipped} skipped)."))
    }
}

fn push_error(errors: &mut Vec<String>, msg: String) {
    if errors.len() < 3 {
        errors.push(msg);
    }
}

fn parse_trade_row(
    cols: &[String],
    header_map: &HashMap<String, usize>,
) -> Result<CsvTradeRow, String> {
    let date = get_value(cols, header_map, "date")?;
    let asset = get_value(cols, header_map, "asset")?;
    let asset_type = get_value(cols, header_map, "asset_type")?;
    let side_raw = get_value(cols, header_map, "side")?;
    let quantity_raw = get_value(cols, header_map, "quantity")?;
    let price_raw = get_value(cols, header_map, "price")?;
    let currency = get_optional(cols, header_map, "currency");

    if asset.is_empty() {
        return Err("Asset is empty".to_string());
    }

    let side = parse_side(&side_raw)?;
    let quantity = parse_decimal(&quantity_raw, "quantity")?;
    let price = parse_decimal(&price_raw, "price")?;

    if quantity <= Decimal::ZERO {
        return Err("Quantity must be positive".to_string());
    }
    if price <= Decimal::ZERO {
        return Err("Price must be positive".to_string());
    }

    let asset_type_norm = asset_type.to_uppercase();
    if asset_type_norm != "STOCK" && asset_type_norm != "CRYPTO" {
        return Err("asset_type must be STOCK or CRYPTO".to_string());
    }

    Ok(CsvTradeRow {
        date,
        asset: asset.to_uppercase(),
        asset_type: asset_type_norm,
        side,
        quantity,
        price,
        currency,
    })
}

fn parse_side(side: &str) -> Result<Side, String> {
    match side.trim().to_uppercase().as_str() {
        "BUY" => Ok(Side::Buy),
        "SELL" => Ok(Side::Sell),
        _ => Err("side must be BUY or SELL".to_string()),
    }
}

fn parse_decimal(value: &str, field: &str) -> Result<Decimal, String> {
    value
        .trim()
        .parse::<Decimal>()
        .map_err(|_| format!("Invalid {field}"))
}

fn get_value(
    cols: &[String],
    header_map: &HashMap<String, usize>,
    key: &str,
) -> Result<String, String> {
    match header_map.get(key) {
        Some(&idx) => Ok(cols.get(idx).map(|v| v.trim().to_string()).unwrap_or_default()),
        None => Err(format!("Missing {key} column")),
    }
}

fn get_optional(cols: &[String], header_map: &HashMap<String, usize>, key: &str) -> Option<String> {
    header_map
        .get(key)
        .and_then(|&idx| cols.get(idx))
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
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

fn parse_date_to_timestamp(date: &str) -> i64 {
    let trimmed = date.trim();
    if trimmed.is_empty() {
        return chrono::Utc::now().timestamp();
    }

    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(trimmed) {
        return dt.timestamp();
    }

    if let Ok(date_only) = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return chrono::DateTime::<chrono::Utc>::from_utc(date_only.and_hms_opt(0, 0, 0).unwrap(), chrono::Utc)
            .timestamp();
    }

    chrono::Utc::now().timestamp()
}
