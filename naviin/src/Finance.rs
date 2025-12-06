pub fn fund(amount: f64) {
    println!("Fund amount: {amount}");
}

pub type Symbol = String;

pub struct Holding {
    name: String,
    quantity: f64,
    avg_cost: f64,
}

pub struct Trade {
    symbol: Symbol,
    quantity: f64,
    price_per: f64,
    side: Side,
    timestamp: i64, // epoch seconds
}

pub enum Side {
    Buy,
    Sell,
}
