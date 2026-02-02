/// Main Entry Point - Naviin Trading Application
/// 
/// Initializes the application and starts the TUI interface.
/// All command processing is now handled through the TUI.

use dotenvy::dotenv;
use std::env;
use std::io;
use std::sync::{Arc, atomic::AtomicBool};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use sea_orm::{Database, DatabaseConnection};

use naviin::AppState::monitor_order;
use naviin::Storage;
use naviin::Tui::Tui;

/// SECTION: Terminal Setup

/// Initializes the terminal for TUI mode
/// Sets up raw mode and alternate screen
fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    // Enable raw mode for immediate key capture
    enable_raw_mode()?;
    
    // Get stdout handle
    let mut stdout = io::stdout();
    
    // Enter alternate screen and enable mouse capture
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture
    )?;
    
    // Create ratatui terminal with crossterm backend
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    
    Ok(terminal)
}

/// Restores terminal to normal state
/// Disables raw mode and returns to main screen
fn restore_terminal() -> io::Result<()> {
    // Disable raw mode
    disable_raw_mode()?;
    
    // Leave alternate screen and disable mouse capture
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    
    Ok(())
}

/// SECTION: Application Entry Point

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv().ok();
    
    // SECTION: Database Setup
    
    // Connect to database
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");
    let db: DatabaseConnection = Database::connect(&database_url)
        .await
        .expect("Failed to connect to database");
    
    // SECTION: State Initialization
    
    // Load application state from database
    let state = Storage::load_state().await;
    
    // Get initial watchlist for TUI
    let initial_watchlist = {
        let state_guard = state.lock().unwrap();
        state_guard.get_watchlist()
    };
    
    // SECTION: Background Services
    
    // Create flag for background order monitoring
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    
    // Start background order monitoring task
    monitor_order(state.clone(), running_clone).await;
    
    // SECTION: TUI Launch
    
    // Setup terminal
    let mut terminal = match setup_terminal() {
        Ok(term) => term,
        Err(e) => {
            eprintln!("Failed to setup terminal: {}", e);
            return;
        }
    };
    
    // Create TUI instance
    let mut tui = Tui::new(
        initial_watchlist,
        state.clone(),
        db.clone(),
        running.clone(),
    );
    
    // Run the TUI event loop
    let tui_result = tui.run(&mut terminal).await;
    
    // Handle TUI errors
    if let Err(e) = tui_result {
        eprintln!("TUI error: {}", e);
    }
    
    // SECTION: Cleanup
    
    // Restore terminal
    if let Err(e) = restore_terminal() {
        eprintln!("Failed to restore terminal: {}", e);
    }
    
    // Stop background monitoring
    running.store(false, std::sync::atomic::Ordering::Relaxed);
    
    // Save final state to database
    Storage::save_state(&state, &db).await;
    
    // Close database connection
    db.close().await.expect("Failed to close database");
    
    println!("Naviin closed successfully");
}
