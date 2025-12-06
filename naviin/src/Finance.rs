use crate::AppState::AppState;

pub fn fund(state: &mut AppState, amount: f64) {
    // validate payment first
    state.deposit(amount);
    state.display();
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
