use std::io::{self, Write};

// Import everything from the `naviin` library crate
use naviin::{Finance, FinanceProvider, Storage, UserInput};

#[tokio::main]
async fn main() {
    // let mut username = String::new();
    let mut state = Storage::load_state();
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
            Finance::fund(&mut state, fund_amount).await;
            Storage::save_state(&state);
        }
        if command == "display" {
            state.display().await;
        }
        if command == "withdraw" {
            print!("Amount: ");
            io::stdout().flush().unwrap();
            let mut withdraw_amount = String::new();
            io::stdin()
                .read_line(&mut withdraw_amount)
                .expect("Invalid amount entered");
            let withdraw_amount: f64 = withdraw_amount.trim().parse().unwrap();
            Finance::withdraw(&mut state, withdraw_amount).await;
            Storage::save_state(&state);
        }
        if command == "price"
            && let Some(ticker) = UserInput::ask_ticker() {
                FinanceProvider::previous_price_close(&ticker, true).await;
            }
        if command == "buy" {
            Finance::buy(&mut state).await;
            Storage::save_state(&state);
        }
        if command == "sell" {
            Finance::sell(&mut state).await;
            Storage::save_state(&state);
        }
        if command == "reset" {
            Storage::default_state(&mut state);
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
