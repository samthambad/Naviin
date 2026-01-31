/// Watchlist Component - Displays stock symbols with real-time prices
/// 
/// This component renders a table showing watched stock symbols and their
/// current market prices. It supports navigation and price refresh.

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Widget},
};
use rust_decimal::Decimal;

use crate::Finance::Symbol;
use crate::FinanceProvider;

/// Component that displays the watchlist with real-time prices
pub struct WatchlistComponent {
    /// List of stock symbols being watched
    symbols: Vec<Symbol>,
    /// Cached prices for each symbol (aligned by index)
    prices: Vec<Decimal>,
    /// Current selected row in the table
    table_state: TableState,
}

impl WatchlistComponent {
    /// SECTION: Constructor
    
    /// Creates a new watchlist component with the given symbols
    /// 
    /// # Arguments
    /// * `symbols` - Vector of stock symbols to display
    pub fn new(symbols: Vec<Symbol>) -> Self {
        let mut table_state = TableState::default();
        if !symbols.is_empty() {
            table_state.select(Some(0));
        }
        Self {
            symbols,
            prices: Vec::new(),
            table_state,
        }
    }

    /// SECTION: Data Management
    
    /// Updates the list of symbols and resets selection
    /// 
    /// # Arguments
    /// * `symbols` - New vector of stock symbols
    pub fn update_symbols(&mut self, symbols: Vec<Symbol>) {
        self.symbols = symbols;
        if !self.symbols.is_empty() && self.table_state.selected().is_none() {
            self.table_state.select(Some(0));
        }
    }

    /// Fetches current prices for all symbols from the finance provider
    /// This is an async operation that updates the cached prices
    pub async fn refresh_prices(&mut self) {
        self.prices = Vec::with_capacity(self.symbols.len());
        for symbol in &self.symbols {
            let price = FinanceProvider::curr_price(symbol, false).await;
            self.prices.push(price);
        }
    }

    /// SECTION: Navigation
    
    /// Moves selection to the next symbol in the list
    /// Wraps around to the first item if at the end
    pub fn next(&mut self) {
        if self.symbols.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.symbols.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    /// Moves selection to the previous symbol in the list
    /// Wraps around to the last item if at the beginning
    pub fn previous(&mut self) {
        if self.symbols.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.symbols.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    /// SECTION: Rendering
    
    /// Renders the watchlist table with headers and data rows
    fn render_table(&self, area: Rect, buf: &mut Buffer) {
        // Create header row with styled column titles
        let header = Row::new(vec![
            Cell::from("Symbol").style(Style::default().fg(Color::Yellow).bold()),
            Cell::from("Price").style(Style::default().fg(Color::Yellow).bold()),
        ])
        .height(1);

        // Generate data rows from symbols and prices
        let rows: Vec<Row> = self
            .symbols
            .iter()
            .enumerate()
            .map(|(i, symbol)| {
                let price = self.prices.get(i).copied().unwrap_or(Decimal::ZERO);
                let price_str = if price == Decimal::ZERO {
                    "N/A".to_string()
                } else {
                    format!("{:.2}", price)
                };

                let cells = vec![
                    Cell::from(symbol.clone()),
                    Cell::from(price_str).style(Style::default().fg(Color::Green)),
                ];

                Row::new(cells).height(1)
            })
            .collect();

        // Build the table with styling and borders
        let table = Table::new(
            rows,
            &[Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .title(" Watchlist ".bold())
                .title_bottom(
                    Line::from(vec![
                        " Navigate ".into(),
                        "<Up>/<Down>".blue().bold(),
                        " Refresh ".into(),
                        "<R>".blue().bold(),
                    ])
                    .centered(),
                ),
        )
        .row_highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .highlight_symbol("> ");

        table.render(area, buf);
    }
}

impl Widget for &WatchlistComponent {
    /// Renders the watchlist component
    /// Shows empty state message if no symbols, otherwise renders table
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.symbols.is_empty() {
            // Empty state - show helpful message
            let empty_text = Text::from(vec![
                Line::from(""),
                Line::from("No symbols in watchlist").centered(),
                Line::from(""),
                Line::from("Add symbols to see prices here").centered().dim(),
            ]);

            Paragraph::new(empty_text)
                .centered()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_set(border::ROUNDED)
                        .title(" Watchlist ".bold()),
                )
                .render(area, buf);
        } else {
            // Render the data table
            self.render_table(area, buf);
        }
    }
}
