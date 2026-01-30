use dotenvy::dotenv;
use std::io::Write;
use std::sync::{Arc, atomic::AtomicBool};
use std::{env, io};
// Import everything from the `naviin` library crate
use naviin::Orders::{self, OrderType};
use naviin::{AppState::monitor_order, Finance, FinanceProvider, Storage, UserInput};
use sea_orm::{Database, DatabaseConnection};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use crossterm::event::{self, Event, KeyCode};
use naviin::Tui::Tui;

#[tokio::main]
async fn main() {
    // let mut username = String::new();
    let _ = ratatui::run(|terminal| Tui::default().run(terminal));
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let db: DatabaseConnection = Database::connect(database_url)
        .await
        .expect("Failed to connect to database");
    let state = Storage::load_state().await;
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    monitor_order(state.clone(), running_clone).await;
    loop {
        print!("What would you like to do today? ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        
        // Manual input buffering in Raw Mode for instant 'd'
        if let Ok(_) = enable_raw_mode() {
            loop {
                if let Ok(Event::Key(key_event)) = event::read() {
                    match key_event.code {
                        // Instant 'D' trigger if it's the very first key
                        KeyCode::Char('D') if command.is_empty() => {
                            println!("D\r");
                            command = "display".to_string();
                            break;
                        }
                        // Echo other characters and add to buffer
                        KeyCode::Char(c) => {
                            command.push(c);
                            print!("{}", c);
                            io::stdout().flush().unwrap();
                        }
                        KeyCode::Backspace => {
                            if !command.is_empty() {
                                command.pop();
                                // Move back, overwrite with space, move back again
                                print!("\x08 \x08");
                                io::stdout().flush().unwrap();
                            }
                        }
                        KeyCode::Enter => {
                            println!("\r");
                            break;
                        }
                        KeyCode::Esc => {
                            command = "exit".to_string();
                            break;
                        }
                        _ => {}
                    }
                }
            }
            let _ = disable_raw_mode();
        } else {
            io::stdin().read_line(&mut command).expect("Failed to read line");
        }

        command = command.trim().to_string();
        match command.as_str() {
            "fund" => {
                print!("Amount: ");
                io::stdout().flush().unwrap();
                let mut fund_amount = String::new();
                io::stdin()
                    .read_line(&mut fund_amount)
                    .expect("Invalid amount entered");
                let fund_amount = fund_amount.trim().parse().unwrap();
                Finance::fund(&state, fund_amount).await;
                Storage::save_state(&state, &db).await;
            }
            "display" | "D" => state.lock().unwrap().display().await,
            "withdraw" => {
                print!("Amount: ");
                io::stdout().flush().unwrap();
                let mut withdraw_amount = String::new();
                io::stdin()
                    .read_line(&mut withdraw_amount)
                    .expect("Invalid amount entered");
                let withdraw_amount = withdraw_amount.trim().parse().unwrap();
                Finance::withdraw(&state, withdraw_amount).await;
                Storage::save_state(&state, &db).await;
            }
            "price" => {
                if let Some(ticker) = UserInput::ask_ticker() {
                    FinanceProvider::curr_price(&ticker, true).await;
                }
            }
            "watch" => {
                let watchlist = {
                    let state_guard = state.lock().unwrap();
                    state_guard.get_watchlist()
                };
                FinanceProvider::stream_watchlist(watchlist).await;
            }
            "addwatch" => {
                if let Some(ticker) = UserInput::ask_ticker() {
                    {
                        let mut state_guard = state.lock().unwrap();
                        state_guard.add_to_watchlist(ticker);
                    }
                    Storage::save_state(&state, &db).await;
                }
            }
            "unwatch" => {
                if let Some(ticker) = UserInput::ask_ticker() {
                    {
                        let mut state_guard = state.lock().unwrap();
                        state_guard.remove_from_watchlist(ticker);
                    }
                    Storage::save_state(&state, &db).await;
                }
            }
            "buy" => {
                Finance::create_buy(&state).await;
                Storage::save_state(&state, &db).await;
            }
            "sell" => {
                Finance::create_sell(&state).await;
                Storage::save_state(&state, &db).await;
            }
            "buylimit" => {
                if let Some(order) = Orders::create_order(OrderType::BuyLimit) {
                    {
                        let mut state_guard = state.lock().unwrap();
                        state_guard.add_open_order(order);
                    }
                    Storage::save_state(&state, &db).await;
                }
            }
            "stoploss" => {
                if let Some(order) = Orders::create_order(OrderType::StopLoss) {
                    {
                        let mut state_guard = state.lock().unwrap();
                        state_guard.add_open_order(order);
                    }
                    Storage::save_state(&state, &db).await;
                }
            }
            "takeprofit" => {
                if let Some(order) = Orders::create_order(OrderType::TakeProfit) {
                    {
                        let mut state_guard = state.lock().unwrap();
                        state_guard.add_open_order(order);
                    }
                    Storage::save_state(&state, &db).await;
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
            "reset" => Storage::default_state(&state, &db).await,
            "help" => UserInput::display_help(),
            "exit" => {
                running.store(false, std::sync::atomic::Ordering::Relaxed);
                Storage::save_state(&state, &db).await;
                break;
            }
            _ => println!("Wrong command entered"),
        }
    }
    db.close().await.expect("Failed to close database");
}
