// Import the AppState struct from our main naviin library.
// The name of the crate is `naviin`, as defined in Cargo.toml.
use naviin::AppState::AppState;

#[test]
fn test_deposit_and_balance() {
    // Arrange: Create a new AppState
    let mut state = AppState::new();
    
    // Act: Deposit 100.0 into the account
    state.deposit(100.0);

    // Assert: Use the public `check_balance` method to verify the result
    assert_eq!(state.check_balance(), 100.0);
}

#[test]
fn test_withdraw_and_balance() {
    // Arrange: Create an AppState with an initial balance
    let mut state = AppState::new();
    state.deposit(100.0);

    // Act: Withdraw 50.0
    state.withdraw(50.0);

    // Assert: Check the final balance
    assert_eq!(state.check_balance(), 50.0);
}

#[test]
fn test_withdraw_with_invalid_amount() {
    // Arrange
    let mut state = AppState::new();
    state.deposit(100.0);

    // Act: Withdraw a negative amount
    state.withdraw(-50.0);

    // Assert: The balance should not have changed
    assert_eq!(state.check_balance(), 100.0);
}
