use std::io::{self, Write};

pub fn ask_ticker() -> String {
    print!("Enter the ticker: ");
    io::stdout().flush().unwrap();
    let mut ticker = String::new();
    io::stdin()
        .read_line(&mut ticker)
        .expect("Invalid amount entered");
    ticker
}

pub fn ask_quantity() -> f64 {
    print!("Enter the quantity: ");
    io::stdout().flush().unwrap();
    let mut quantity = String::new();
    io::stdin()
        .read_line(&mut quantity)
        .expect("Invalid amount entered");
    let quantity: f64 = quantity.trim().parse().unwrap();
    quantity

}