use std::io::{self, Write};

mod account;
mod storage;

fn main() {
    let mut username = String::new();

    println!("Enter login details");
    print!("Username:");
    io::stdout().flush().unwrap(); // to make sure it prints
    io::stdin()
        .read_line(&mut username)
        .expect("Invalid username entered");
    username = username.trim().to_string();
    if !storage::username_checker(&username) {
        println!("Account is not registered");
    }
}
