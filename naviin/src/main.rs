use std::io::{self, Write};

mod Finance;
mod Storage;

fn main() {
    let mut username = String::new();

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
        fund_amount = fund_amount.parse().unwrap();
        println!("Fund amount: {fund_amount}");
    }
}
