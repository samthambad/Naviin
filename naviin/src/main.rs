use std::io;
use std::io::Write;
use std::sync::{Arc, atomic::AtomicBool};
// Import everything from the `naviin` library crate
use naviin::{AppState::monitor_order, Finance, FinanceProvider, Storage, UserInput};
use naviin::Finance::OrderType;

#[tokio::main]
async fn main() {
    // let mut username = String::new();
    let state = Storage::load_state();
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    monitor_order(state.clone(), running_clone).await;
    loop {
        print!("What would you like to do today? ");
        io::stdout().flush().unwrap();
        let mut command = String::new();
        io::stdin()
            .read_line(&mut command)
            .expect("Invalid command entered");
        command = command.trim().to_string();
        match command.as_str() {
            "fund" => {
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
            "display" => state.lock().unwrap().display().await,
            "withdraw" => {
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
            "price" => {
                if let Some(ticker) = UserInput::ask_ticker() {
                    FinanceProvider::curr_price(&ticker, true).await;
                }
            }
            "buy" => {
                Finance::buy(&state).await;
                Storage::save_state(&state);
            }
            "sell" => {
                Finance::sell(&state).await;
                Storage::save_state(&state);
            }
            "buylimit" => {
                let new_limit_order = Finance::create_order(OrderType::BuyLimit);
                if let Some(order) = new_limit_order {
                    {
                        let mut state_guard = state.lock().unwrap();
                        state_guard.add_open_order(order);
                    }
                    Storage::save_state(&state);
                }
            }
            "stoploss" => {
                let new_limit_order = Finance::create_order(OrderType::StopLoss);
                if let Some(order) = new_limit_order {
                    {
                        let mut state_guard = state.lock().unwrap();
                        state_guard.add_open_order(order);
                    }
                    Storage::save_state(&state);
                }
            }
            "takeprofit" => {
                let new_limit_order = Finance::create_order(OrderType::TakeProfit);
                if let Some(order) = new_limit_order {
                    {
                        let mut state_guard = state.lock().unwrap();
                        state_guard.add_open_order(order);
                    }
                    Storage::save_state(&state);
                }
            }
            "stopbg" => {
                running.store(false, std::sync::atomic::Ordering::Relaxed);
                println!(
                    "All background orders will be paused till relaunch of app or `startbg` command"
                );
            }
            "startbg" => {
                running.store(true, std::sync::atomic::Ordering::Relaxed);
                println!("Background orders resumed execution");
            }
            "reset" => Storage::default_state(&state),
            "help" => UserInput::display_help(),
            "exit" => {
                running.store(false, std::sync::atomic::Ordering::Relaxed);
                Storage::save_state(&state);
                break;
            }
            _ => println!("Wrong command entered"),
        }
    }
}
