use naviin::Storage;
use naviin::AppState::AppState;
use std::sync::{Arc, Mutex};
use std::fs;

// Helper function to clean up test files
fn cleanup_test_file() {
    let _ = fs::remove_file("state.json");
}

// Run storage tests serially to avoid file conflicts
// Note: These tests modify the same state.json file

#[test]
#[ignore] // Run with: cargo test --ignored --test-threads=1
fn test_save_and_load_state() {
    cleanup_test_file();
    
    // Create a state with some data
    let state = Arc::new(Mutex::new(AppState::new()));
    {
        let mut guard = state.lock().unwrap();
        guard.deposit(1000.0);
    }
    
    // Save state
    Storage::save_state(&state);
    
    // Load state
    let loaded_state = Storage::load_state();
    let loaded_balance = loaded_state.lock().unwrap().check_balance();
    
    assert_eq!(loaded_balance, 1000.0);
    
    cleanup_test_file();
}

#[test]
fn test_load_state_when_file_missing() {
    cleanup_test_file();
    
    // Load state should return a new empty state
    let state = Storage::load_state();
    let balance = state.lock().unwrap().check_balance();
    
    assert_eq!(balance, 0.0);
}

#[test]
fn test_load_state_with_corrupted_json() {
    cleanup_test_file();
    
    // Write invalid JSON to the state file
    fs::write("state.json", "{ invalid json content }").unwrap();
    
    // Load state should return a new empty state on parse error
    let state = Storage::load_state();
    let balance = state.lock().unwrap().check_balance();
    
    assert_eq!(balance, 0.0);
    
    cleanup_test_file();
}

#[test]
fn test_default_state() {
    cleanup_test_file();
    
    // Create a state with some balance
    let state = Arc::new(Mutex::new(AppState::new()));
    {
        let mut guard = state.lock().unwrap();
        guard.deposit(5000.0);
    }
    
    // Reset to default
    Storage::default_state(&state);
    
    // Check that state was reset
    let balance = state.lock().unwrap().check_balance();
    assert_eq!(balance, 0.0);
    
    cleanup_test_file();
}

#[test]
fn test_save_state_creates_file() {
    cleanup_test_file();
    
    let state = Arc::new(Mutex::new(AppState::new()));
    
    // Save state
    Storage::save_state(&state);
    
    // Verify file exists
    assert!(fs::metadata("state.json").is_ok());
    
    cleanup_test_file();
}

#[test]
#[ignore] // Run with: cargo test --ignored --test-threads=1
fn test_save_state_with_multiple_operations() {
    cleanup_test_file();
    
    let state = Arc::new(Mutex::new(AppState::new()));
    {
        let mut guard = state.lock().unwrap();
        guard.deposit(1000.0);
        guard.withdraw(200.0);
        guard.deposit_sell(500.0);
    }
    
    Storage::save_state(&state);
    
    let loaded_state = Storage::load_state();
    let loaded_balance = loaded_state.lock().unwrap().check_balance();
    
    // 1000 - 200 + 500 = 1300
    assert_eq!(loaded_balance, 1300.0);
    
    cleanup_test_file();
}

#[test]
fn test_username_checker() {
    let username = "test_user".to_string();
    let result = Storage::username_checker(&username);
    
    // Currently always returns true
    assert!(result);
}

#[test]
#[ignore] // Run with: cargo test --ignored --test-threads=1
fn test_multiple_save_and_load_cycles() {
    cleanup_test_file();
    
    let state = Arc::new(Mutex::new(AppState::new()));
    
    // First cycle
    {
        let mut guard = state.lock().unwrap();
        guard.deposit(100.0);
    }
    Storage::save_state(&state);
    
    // Second cycle
    {
        let mut guard = state.lock().unwrap();
        guard.deposit(200.0);
    }
    Storage::save_state(&state);
    
    // Load and verify
    let loaded_state = Storage::load_state();
    let balance = loaded_state.lock().unwrap().check_balance();
    
    assert_eq!(balance, 300.0);
    
    cleanup_test_file();
}

#[test]
fn test_default_state_creates_empty_state() {
    cleanup_test_file();
    
    let state = Arc::new(Mutex::new(AppState::new()));
    
    Storage::default_state(&state);
    
    // Verify state is empty
    let guard = state.lock().unwrap();
    assert_eq!(guard.check_balance(), 0.0);
    assert!(guard.get_holdings_map().is_empty());
    assert!(guard.get_open_orders().is_empty());
    
    cleanup_test_file();
}
