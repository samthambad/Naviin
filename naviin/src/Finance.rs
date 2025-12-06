use crate::AppState::AppState;

pub fn fund(state: &mut AppState, amount: f64) {
    if amount <= 0.0 {
        println!("Invalid amount");
        return;
    }
    // validate payment first
    state.deposit(amount);
    state.display();
}

pub fn withdraw(state: &mut AppState, amount: f64) {
    if amount <= 0.0 {
        println!("Invalid amount");
        return;
    }
    if amount > state.check_balance() {
        println!("Insufficient balance");
        return;
    }
    state.withdraw(amount);
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
