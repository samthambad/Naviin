use std::io::{self, Write};

mod AppState;
mod Finance;
mod FinanceProvider;
mod Storage;

#[tokio::main]
async fn main() {
    // let mut username = String::new();
    // TODO: load the state here
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
            Finance::fund(&mut state, fund_amount);
            Storage::save_state(&state);
        }
        if command == "display" {
            state.display();
        }
        if command == "withdraw" {
            print!("Amount: ");
            io::stdout().flush().unwrap();
            let mut withdraw_amount = String::new();
            io::stdin()
                .read_line(&mut withdraw_amount)
                .expect("Invalid amount entered");
            let withdraw_amount: f64 = withdraw_amount.trim().parse().unwrap();
            Finance::withdraw(&mut state, withdraw_amount);
            Storage::save_state(&state);
        }
        if command == "price" {
            print!("Enter ticker: ");
            io::stdout().flush().unwrap();
            let mut ticker = String::new();
            io::stdin()
                .read_line(&mut ticker)
                .expect("Invalid amount entered");
            let ticker = ticker.trim();
            FinanceProvider::print_previous_close(&ticker).await;
        }
        if command == "exit" {
            Storage::save_state(&state);
            break;
        }
    }
}
