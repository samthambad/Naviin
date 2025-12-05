use std::io::{self, Write};

mod account;

fn main() {
    let mut username = String::new();

    println!("Enter login details");
    print!("Username:");
    io::stdout().flush().unwrap(); // to make sure it prints
    io::stdin()
        .read_line(&mut username)
        .expect("Invalid username entered");
}

fn validate(&String username) -> bool {
    // check against storage
}
