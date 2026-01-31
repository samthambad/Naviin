/// TUI Module - Main terminal user interface
/// 
/// This module coordinates the display of three main areas:
/// 1. Top: Watchlist component (stock symbols and prices)
/// 2. Middle: Input component (command typing area)
/// 3. Bottom: Output component (command results display)
/// 
/// All component logic is delegated to their respective modules.

use std::io;
use std::sync::{Arc, Mutex};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame, Terminal,
};
use sea_orm::DatabaseConnection;

use crate::AppState::AppState;
use crate::commands::process_command;
use crate::components::input::InputComponent;
use crate::components::output::OutputComponent;
use crate::components::watchlist::WatchlistComponent;
use crate::Finance::Symbol;

/// Main TUI application state and coordinator
pub struct Tui {
    /// Flag to indicate if the application should exit
    exit: bool,
    /// Top section: Watchlist display component
    watchlist: WatchlistComponent,
    /// Middle section: Command input component
    input: InputComponent,
    /// Bottom section: Output display component
    output: OutputComponent,
    /// Application state (holdings, cash, orders)
    state: Arc<Mutex<AppState>>,
    /// Database connection for persistence
    db: DatabaseConnection,
    /// Background order monitoring flag
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl Tui {
    /// SECTION: Constructor
    
    /// Creates a new TUI instance with all required dependencies
    /// 
    /// # Arguments
    /// * `symbols` - Initial list of stock symbols to display in watchlist
    /// * `state` - Application state shared across components
    /// * `db` - Database connection for saving/loading state
    /// * `running` - Flag to control background order monitoring
    pub fn new(
        symbols: Vec<Symbol>,
        state: Arc<Mutex<AppState>>,
        db: DatabaseConnection,
        running: Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        Self {
            exit: false,
            watchlist: WatchlistComponent::new(symbols),
            input: InputComponent::new(),
            output: OutputComponent::new(),
            state,
            db,
            running,
        }
    }

    /// SECTION: Main Loop
    
    /// Runs the application's main event loop until user quits
    /// 
    /// # Arguments
    /// * `terminal` - The ratatui terminal to draw on
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()>
    where
        io::Error: From<<B as Backend>::Error>,
    {
        // Initial watchlist price fetch
        self.refresh_watchlist().await;
        
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().await?;
        }
        Ok(())
    }

    /// SECTION: Rendering
    
    /// Draws the three components in their respective screen areas
    /// Layout: Watchlist (top), Input (middle), Output (bottom)
    /// 
    /// # Arguments
    /// * `frame` - The frame to render on
    fn draw(&self, frame: &mut Frame) {
        let areas = self.calculate_layout(frame.area());
        
        // Render each component in its assigned area
        frame.render_widget(&self.watchlist, areas.watchlist);
        frame.render_widget(&self.input, areas.input);
        frame.render_widget(&self.output, areas.output);
    }

    /// Calculates the screen layout dividing space between three components
    /// 
    /// # Arguments
    /// * `area` - Total available screen area
    /// 
    /// # Returns
    /// Struct containing the three sub-areas
    fn calculate_layout(&self, area: Rect) -> LayoutAreas {
        // Split vertically into three sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40), // Watchlist gets 40%
                Constraint::Percentage(20), // Input gets 20%
                Constraint::Percentage(40), // Output gets 40%
            ])
            .split(area);

        LayoutAreas {
            watchlist: chunks[0],
            input: chunks[1],
            output: chunks[2],
        }
    }

    /// SECTION: Event Handling
    
    /// Processes keyboard and other input events
    async fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event).await;
            }
            _ => {}
        };
        Ok(())
    }

    /// Handles keyboard key press events
    /// Routes events to appropriate component or handles globally
    /// 
    /// # Arguments
    /// * `key_event` - The key event to process
    async fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            // Global quit command
            KeyCode::Char('q') if key_event.modifiers.is_empty() => {
                self.exit();
            }
            
            // Input navigation
            KeyCode::Left => self.input.move_cursor_left(),
            KeyCode::Right => self.input.move_cursor_right(),
            KeyCode::Home => self.input.move_cursor_start(),
            KeyCode::End => self.input.move_cursor_end(),
            
            // Text input
            KeyCode::Char(c) => self.input.enter_char(c),
            KeyCode::Backspace => self.input.backspace(),
            
            // Command execution (Enter key)
            KeyCode::Enter => self.execute_command().await,
            
            // Watchlist navigation
            KeyCode::Up => self.watchlist.previous(),
            KeyCode::Down => self.watchlist.next(),
            
            _ => {}
        }
    }

    /// SECTION: Command Execution
    
    /// Executes the current command from input and displays result
    async fn execute_command(&mut self) {
        let command = self.input.get_command().to_string();
        
        // Commit output to history
        self.output.commit_to_history();
        
        // Clear input
        self.input.clear();
        
        // Check for exit command
        if command.eq_ignore_ascii_case("exit") || command.eq_ignore_ascii_case("quit") {
            self.exit();
            return;
        }
        
        // Process command and get result
        let result = process_command(
            &command,
            &self.state,
            &self.db,
            &self.running,
        ).await;
        
        // Display result
        self.output.set_output(result);
        
        // Refresh watchlist if command might have changed it
        if command.starts_with("addwatch") || command.starts_with("unwatch") {
            self.refresh_watchlist().await;
        }
    }

    /// SECTION: Watchlist Management
    
    /// Refreshes watchlist symbols and prices from current state
    async fn refresh_watchlist(&mut self) {
        // Get current watchlist from state
        let symbols = {
            let state_guard = self.state.lock().unwrap();
            state_guard.get_watchlist()
        };
        
        // Update watchlist component with symbols
        self.watchlist.update_symbols(symbols);
        
        // Fetch current prices
        self.watchlist.refresh_prices().await;
    }

    /// SECTION: Application Control
    
    /// Signals the application to exit
    fn exit(&mut self) {
        self.exit = true;
    }
}

/// Layout areas for the three UI components
struct LayoutAreas {
    /// Area for watchlist component (top)
    watchlist: Rect,
    /// Area for input component (middle)
    input: Rect,
    /// Area for output component (bottom)
    output: Rect,
}
