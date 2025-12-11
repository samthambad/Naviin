use std::io;

pub fn askTicker() -> String {
    let mut ticker = String::new();
    io::stdin()
        .read_line(&mut ticker)
        .expect("Invalid amount entered");
    ticker
}
