use std::collections::HashMap;
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
    Frame, Terminal,
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
};
use rust_decimal::Decimal;
use sea_orm::DatabaseConnection;
use tokio::time::{Instant, interval};
use tokio::sync::mpsc;

use crate::AppState::AppState;
use crate::Finance::Symbol;
use crate::commands::process_command;
use crate::components::holdings::HoldingsComponent;
use crate::components::input::InputComponent;
use crate::components::open_orders::OpenOrdersComponent;
use crate::components::output::OutputComponent;
use crate::components::watchlist::WatchlistComponent;
use crate::FinanceProvider;

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

/// Main TUI application state and coordinator
pub struct Tui {
    /// Flag to indicate if the application should exit
    exit: bool,
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

    message_tx: mpsc::UnboundedSender<TuiMessage>,
    message_rx: mpsc::UnboundedReceiver<TuiMessage>,
    price_refresh_running: bool,
}

/// Used for message passing via channel
enum TuiMessage {
    PricesUpdated {
        holdings: HashMap<Symbol, Decimal>,
        watchlist: HashMap<Symbol, Decimal>,
    },
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
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        Self {
            exit: false,
            holdings: HoldingsComponent::new(),
            open_orders: OpenOrdersComponent::new(),
            watchlist: WatchlistComponent::new(symbols),
            input: InputComponent::new(),
            output: OutputComponent::new(),
            state,
            db,
            running,
            last_refresh: Instant::now(),
            message_tx,
            message_rx,
            price_refresh_running: false,
        }
    }

    /// SECTION: Main Loop

    /// Runs the application's main event loop until user quits, async event handling for responsive input
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

            // concurrent: checking for input while refreshing,
            // do whatever comes first skip the other one as both update the ui needed as either could overrun
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

                // TODO: refresh after executing orders
                // Handle periodic refresh every 5 seconds
                _ = refresh_timer.tick() => {
                    self.refresh_all().await;
                    self.last_refresh = Instant::now();
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
        let areas = self.calculate_layout(frame.area()); // in case user resizes terminal window

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
            KeyCode::Char('Q') => {
                self.exit();
            }

            // Input navigation
            KeyCode::Left => self.input.move_cursor_left(),
            KeyCode::Right => self.input.move_cursor_right(),
            KeyCode::Home
                if !key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.input.move_cursor_start()
            }
            KeyCode::End
                if !key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.input.move_cursor_end()
            }

            // Text input
            KeyCode::Char(c) => self.input.enter_char(c),
            KeyCode::Backspace => self.input.backspace(),

            // Command execution
            KeyCode::Enter => self.execute_command().await,

            // Output scrolling
            KeyCode::PageUp => self.output.scroll_up(5),
            KeyCode::PageDown => self.output.scroll_down(5),
            KeyCode::Home
                if key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.output.scroll_to_top()
            }
            KeyCode::End
                if key_event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.output.scroll_to_bottom()
            }

            _ => {}
        }
    }

    /// SECTION: Command Execution

    /// Executes the current command from input and displays result
    async fn execute_command(&mut self) {
        let command = self.input.get_command().to_string();

        // Commit output to history
        self.output.commit_to_history();

        self.input.clear();

        if command.eq_ignore_ascii_case("exit") || command.eq_ignore_ascii_case("quit") {
            self.exit();
            return;
        }

        if command.eq_ignore_ascii_case("clear") {
            self.output.clear();
            self.output.set_output("Screen cleared".to_string());
            return;
        }

        let result = process_command(&command, &self.state, &self.db, &self.running).await;

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
        Self::start_refresh_price(self);
    }

    /// Spawns a background task to fetch holdings/watchlist prices without blocking the UI loop.
    /// Sends a `TuiMessage::PricesUpdated` through `message_tx` when the refresh completes.
    async fn start_refresh_price(&mut self) {
        // Fetch prices for holdings and watchlist in parallel
        let tx = self.message_tx.clone();
        let holdings_symbols = self.holdings.get_holdings();
        let watchlist_symbols = self.watchlist.get_symbols();
        tokio::spawn(async move {
            let message = refresh_prices(holdings_symbols, watchlist_symbols).await;
            let _ = tx.send(message);
        });
    }
    async fn refresh_prices(holding_symbols: Vec<Symbol>, watchlist_symbols: Vec<Symbol>) -> TuiMessage {

        let mut holdings_map: HashMap<Symbol, Decimal> = HashMap::new();
        for symbol in holding_symbols {
            let price = FinanceProvider::curr_price(&symbol, false).await;
            holdings_map.insert(symbol, price);
        }
        let mut watchlist_map: HashMap<Symbol, Decimal> = HashMap::new();
        for symbol in watchlist_symbols {
            let price = FinanceProvider::curr_price(&symbol, false).await;
            watchlist_map.insert(symbol, price);
        }
        TuiMessage::PricesUpdated {
            holdings: holdings_map,
            watchlist: watchlist_map,
        }
    }
    /// SECTION: Application Control

    /// Signals the application to exit
    fn exit(&mut self) {
        self.exit = true;
    }
}
