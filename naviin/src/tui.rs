/// TUI Module - Main terminal user interface
/// 
/// This module coordinates the display of UI areas:
/// 1. Top Row: Holdings | Open Orders | Watchlist (3 components, horizontal)
/// 2. Middle: Input component (command typing area)
/// 3. Bottom: Output component (command results display)
/// 
/// Auto-refreshes top components every 5 seconds for real-time price updates.

use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame, Terminal,
};
use sea_orm::DatabaseConnection;
use tokio::time::{interval, Instant};

use crate::AppState::AppState;
use crate::commands::process_command;
use crate::components::holdings::HoldingsComponent;
use crate::components::input::InputComponent;
use crate::components::open_orders::OpenOrdersComponent;
use crate::components::output::OutputComponent;
use crate::components::watchlist::WatchlistComponent;
use crate::Finance::Symbol;

/// Tracks which of the three top components is currently active for navigation
#[derive(Debug, Clone, Copy, PartialEq)]
enum TopSection {
    Holdings,
    OpenOrders,
    Watchlist,
}

/// Main TUI application state and coordinator
pub struct Tui {
    /// Flag to indicate if the application should exit
    exit: bool,
    /// Currently active top section for navigation
    active_top: TopSection,
    /// Top left: Holdings component
    holdings: HoldingsComponent,
    /// Top middle: Open orders component
    open_orders: OpenOrdersComponent,
    /// Top right: Watchlist display component
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
    /// Last time data was refreshed (for status/debugging)
    last_refresh: Instant,
}

impl Tui {
    /// SECTION: Constructor
    
    /// Creates a new TUI instance with all required dependencies
    pub fn new(
        symbols: Vec<Symbol>,
        state: Arc<Mutex<AppState>>,
        db: DatabaseConnection,
        running: Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        Self {
            exit: false,
            active_top: TopSection::Holdings,
            holdings: HoldingsComponent::new(),
            open_orders: OpenOrdersComponent::new(),
            watchlist: WatchlistComponent::new(symbols),
            input: InputComponent::new(),
            output: OutputComponent::new(),
            state,
            db,
            running,
            last_refresh: Instant::now(),
        }
    }

    /// SECTION: Main Loop
    
    /// Runs the application's main event loop until user quits
    /// Uses proper async event handling for responsive input
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()>
    where
        io::Error: From<<B as Backend>::Error>,
    {
        // Initial data refresh and draw
        self.refresh_all().await;
        terminal.draw(|frame| self.draw(frame))?;
        
        // Create a 5-second interval timer for auto-refresh
        let mut refresh_timer = interval(Duration::from_secs(5));
        refresh_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        
        // Track if we need to redraw
        let mut needs_redraw = false;
        
        while !self.exit {
            // Redraw if needed (after input processing or refresh)
            if needs_redraw {
                terminal.draw(|frame| self.draw(frame))?;
                needs_redraw = false;
            }
            
            // Wait for events with timeout (50ms for responsiveness)
            // This allows checking refresh timer while waiting for input
            tokio::select! {
                // Check for crossterm events
                event_result = Self::wait_for_event() => {
                    match event_result {
                        Ok(Some(Event::Key(key_event))) if key_event.kind == KeyEventKind::Press => {
                            self.handle_key_event(key_event).await;
                            needs_redraw = true; // Redraw after input
                        }
                        Ok(_) => {} // Other events (resize, etc)
                        Err(_) => {} // Error reading event
                    }
                }
                
                // Handle periodic refresh every 5 seconds
                _ = refresh_timer.tick() => {
                    self.refresh_prices_only().await;
                    self.last_refresh = Instant::now();
                    needs_redraw = true; // Redraw after price refresh
                }
            }
        }
        Ok(())
    }
    
    /// Async helper to wait for crossterm events
    /// Uses spawn_blocking to make crossterm's blocking call async-friendly
    async fn wait_for_event() -> io::Result<Option<Event>> {
        // Use a short timeout so we can also check for messages and timer
        tokio::task::spawn_blocking(|| {
            if event::poll(Duration::from_millis(50))? {
                Ok(Some(event::read()?))
            } else {
                Ok(None)
            }
        })
        .await
        .map_err(io::Error::other)?
    }

    /// SECTION: Rendering
    
    /// Draws all UI components in their assigned areas
    fn draw(&self, frame: &mut Frame) {
        let areas = self.calculate_layout(frame.area());
        
        // Render top row (3 components horizontally)
        frame.render_widget(&self.holdings, areas.holdings);
        frame.render_widget(&self.open_orders, areas.open_orders);
        frame.render_widget(&self.watchlist, areas.watchlist);
        
        // Render middle and bottom sections
        frame.render_widget(&self.input, areas.input);
        frame.render_widget(&self.output, areas.output);
    }

    /// Calculates the screen layout
    /// Top row: 3 horizontal components (Holdings | Open Orders | Watchlist)
    /// Middle: Input
    /// Bottom: Output
    fn calculate_layout(&self, area: Rect) -> LayoutAreas {
        // First split vertically: top row (40%), input (20%), output (40%)
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(40),
            ])
            .split(area);

        // Split top row horizontally into 3 equal parts
        let top_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(vertical_chunks[0]);

        LayoutAreas {
            holdings: top_row[0],
            open_orders: top_row[1],
            watchlist: top_row[2],
            input: vertical_chunks[1],
            output: vertical_chunks[2],
        }
    }

    /// SECTION: Event Handling
    
    /// Handles keyboard key press events
    async fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            // Global quit
            KeyCode::Char('q') if key_event.modifiers.is_empty() => {
                self.exit();
            }
            
            // Input navigation
            KeyCode::Left => self.input.move_cursor_left(),
            KeyCode::Right => self.input.move_cursor_right(),
            KeyCode::Home if !key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.input.move_cursor_start()
            }
            KeyCode::End if !key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.input.move_cursor_end()
            }
            
            // Text input
            KeyCode::Char(c) => self.input.enter_char(c),
            KeyCode::Backspace => self.input.backspace(),
            
            // Command execution
            KeyCode::Enter => self.execute_command().await,
            
            // Top section navigation (Tab cycles through Holdings -> OpenOrders -> Watchlist)
            KeyCode::Tab => self.cycle_top_section(),
            
            // Navigation within active top section (Up/Down)
            KeyCode::Up => self.navigate_top_previous(),
            KeyCode::Down => self.navigate_top_next(),
            
            // Output scrolling
            KeyCode::PageUp => self.output.scroll_up(5),
            KeyCode::PageDown => self.output.scroll_down(5),
            KeyCode::Home if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.output.scroll_to_top()
            }
            KeyCode::End if key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                self.output.scroll_to_bottom()
            }
            
            _ => {}
        }
    }

    /// SECTION: Top Section Navigation

    /// Cycles to the next top section (Holdings -> OpenOrders -> Watchlist -> Holdings)
    fn cycle_top_section(&mut self) {
        self.active_top = match self.active_top {
            TopSection::Holdings => TopSection::OpenOrders,
            TopSection::OpenOrders => TopSection::Watchlist,
            TopSection::Watchlist => TopSection::Holdings,
        };
    }

    /// Navigates to previous item in the active top section
    fn navigate_top_previous(&mut self) {
        match self.active_top {
            TopSection::Holdings => self.holdings.previous(),
            TopSection::OpenOrders => self.open_orders.previous(),
            TopSection::Watchlist => self.watchlist.previous(),
        }
    }

    /// Navigates to next item in the active top section
    fn navigate_top_next(&mut self) {
        match self.active_top {
            TopSection::Holdings => self.holdings.next(),
            TopSection::OpenOrders => self.open_orders.next(),
            TopSection::Watchlist => self.watchlist.next(),
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
        
        // Check for clear command
        if command.eq_ignore_ascii_case("clear") {
            self.output.clear();
            self.output.set_output("Screen cleared".to_string());
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
        
        // Refresh all data if command might have changed state
        self.refresh_all().await;
    }

    /// SECTION: Data Refresh

    /// Refreshes all top section components with current data
    /// Used after commands that modify state
    async fn refresh_all(&mut self) {
        let state_guard = self.state.lock().unwrap();
        
        // Get all data from state
        let holdings = state_guard.get_holdings_map();
        let orders = state_guard.get_open_orders();
        let watchlist = state_guard.get_watchlist();
        let cash = state_guard.check_balance();
        
        // Update components
        self.holdings.update_holdings(holdings, cash);
        self.open_orders.update_orders(orders);
        self.watchlist.update_symbols(watchlist);
        
        // Release lock before async operations
        drop(state_guard);
        
        // Fetch prices for holdings and watchlist in parallel
        self.refresh_prices_only().await;
    }

    /// Refreshes only prices (not the full state)
    /// Optimized for the 5-second auto-refresh timer
    /// Fetches prices concurrently for maximum performance
    async fn refresh_prices_only(&mut self) {
        // Fetch prices concurrently using tokio::join for maximum performance
        // This runs both refresh operations in parallel
        tokio::join!(
            self.holdings.refresh_prices(),
            self.watchlist.refresh_prices()
        );
    }

    /// SECTION: Application Control
    
    /// Signals the application to exit
    fn exit(&mut self) {
        self.exit = true;
    }
}

/// Layout areas for all UI components
struct LayoutAreas {
    /// Area for holdings component (top left)
    holdings: Rect,
    /// Area for open orders component (top middle)
    open_orders: Rect,
    /// Area for watchlist component (top right)
    watchlist: Rect,
    /// Area for input component (middle)
    input: Rect,
    /// Area for output component (bottom)
    output: Rect,
}
