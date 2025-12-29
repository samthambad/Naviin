// Import everything from the `naviin` library crate
use naviin::{AppState::AppState, Finance, FinanceProvider, Storage, UserInput};

#[tokio::main]
async fn main() {
    // let mut username = String::new();
    let state = Storage::load_state();
    loop {
        print!("What would you like to do today? ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin()
            .read_line(&mut command)
            .expect("Invalid command entered");
        command = command.trim().to_string();
        if command == "fund" {
            print!("Amount: ");
            io::stdout().flush().unwrap();
            let mut fund_amount = String::new();
            io::stdin()
                .read_line(&mut fund_amount)
                .expect("Invalid amount entered");
            let fund_amount: f64 = fund_amount.trim().parse().unwrap();
            Finance::fund(&state, fund_amount).await;
            Storage::save_state(&state);
        }
        if command == "display" {
            state.lock().unwrap().display().await;
        }
        if command == "withdraw" {
            print!("Amount: ");
            io::stdout().flush().unwrap();
            let mut withdraw_amount = String::new();
            io::stdin()
                .read_line(&mut withdraw_amount)
                .expect("Invalid amount entered");
            let withdraw_amount: f64 = withdraw_amount.trim().parse().unwrap();
            Finance::withdraw(&state, withdraw_amount).await;
            Storage::save_state(&state);
        }
        if command == "price"
            && let Some(ticker) = UserInput::ask_ticker()
        {
            FinanceProvider::previous_price_close(&ticker, true).await;
        }
        if command == "buy" {
            Finance::buy(&state).await;
            Storage::save_state(&state);
        }
        if command == "buylimit" {
            match new_limit_order = Finance::create_limit_order() {
                Some(t) => t,
                None => {
                    println!("Trouble creating limit order!");
                    return
                },
            }
            let mut state_guard = state.lock().unwrap();
            state_guard.add_open_order(new_limit_order);
            Storage::save_state(&state);
        }
        if command == "sell" {
            Finance::sell(&state).await;
            Storage::save_state(&state);
        }
        if command == "reset" {
            Storage::default_state();
        }
        if command == "help" {
            UserInput::display_help();
        }
        if command == "exit" {
            Storage::save_state(&state);
            break;
        }
    }
}
