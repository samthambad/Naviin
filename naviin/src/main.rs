use std::io::{self, Write};

mod AppState;
mod Finance;
mod Storage;

fn main() {
    let mut username = String::new();
    let mut state = AppState::AppState::new();

    println!("Enter login details");
    print!("Username:");
    io::stdout().flush().unwrap(); // to make sure it prints
    io::stdin()
        .read_line(&mut username)
        .expect("Invalid username entered");
    username = username.trim().to_string();
    if !Storage::username_checker(&username) {
        println!("Account is not registered");
    }
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
        if command == "exit" {
            Storage::save_state(&state);
            break;
        }
    }
}
