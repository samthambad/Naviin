/// Holdings Component - Displays owned stock positions
/// 
/// Shows current holdings with quantity, average cost, current price, and P&L.

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Widget},
};
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::Finance::{Holding, Symbol};
use crate::FinanceProvider;

/// Component that displays holdings with real-time prices and P&L
pub struct HoldingsComponent {
    /// Map of symbol to holding
    holdings: HashMap<Symbol, Holding>,
    /// Cached prices for each holding
    prices: HashMap<Symbol, Decimal>,
    /// Current selected row
    table_state: TableState,
    /// List of symbols for indexing (since HashMap is unordered)
    symbol_list: Vec<Symbol>,
    /// Cash balance
    cash: Decimal,
}

impl HoldingsComponent {
    /// SECTION: Constructor

    /// Creates a new holdings component
    pub fn new() -> Self {
        Self {
            holdings: HashMap::new(),
            prices: HashMap::new(),
            table_state: TableState::default(),
            symbol_list: Vec::new(),
            cash: Decimal::ZERO,
        }
    }

    /// SECTION: Data Management

    /// Updates holdings data and cash from state
    pub fn update_holdings(&mut self, holdings: HashMap<Symbol, Holding>, cash: Decimal) {
        self.holdings = holdings;
        self.cash = cash;
        self.symbol_list = self.holdings.keys().cloned().collect();
        if !self.symbol_list.is_empty() && self.table_state.selected().is_none() {
            self.table_state.select(Some(0));
        }
    }

    /// Fetches current prices for all holdings
    pub async fn refresh_prices(&mut self) {
        self.prices.clear();
        for symbol in self.holdings.keys() {
            let price = FinanceProvider::curr_price(symbol, false).await;
            self.prices.insert(symbol.clone(), price);
        }
    }

    /// SECTION: Navigation

    /// Moves to next holding
    pub fn next(&mut self) {
        if self.symbol_list.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.symbol_list.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    /// Moves to previous holding
    pub fn previous(&mut self) {
        if self.symbol_list.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.symbol_list.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    /// SECTION: Rendering

    fn render_table(&self, area: Rect, buf: &mut Buffer) {
        let header = Row::new(vec![
            Cell::from("Symbol").style(Style::default().fg(Color::Yellow).bold()),
            Cell::from("Qty").style(Style::default().fg(Color::Yellow).bold()),
            Cell::from("Avg").style(Style::default().fg(Color::Yellow).bold()),
            Cell::from("Price").style(Style::default().fg(Color::Yellow).bold()),
            Cell::from("P&L").style(Style::default().fg(Color::Yellow).bold()),
        ])
        .height(1);

        let rows: Vec<Row> = self
            .symbol_list
            .iter()
            .map(|symbol| {
                let holding = self.holdings.get(symbol).unwrap();
                let qty = holding.get_qty();
                let avg = holding.get_avg_price();
                let curr_price = self.prices.get(symbol).copied().unwrap_or(Decimal::ZERO);
                
                // Calculate P&L
                let pnl = (curr_price - avg) * qty;
                let pnl_str = if curr_price == Decimal::ZERO {
                    "N/A".to_string()
                } else {
                    format!("{:.2}", pnl)
                };
                let pnl_color = if pnl >= Decimal::ZERO { Color::Green } else { Color::Red };

                let cells = vec![
                    Cell::from(symbol.clone()),
                    Cell::from(format!("{:.2}", qty)),
                    Cell::from(format!("{:.2}", avg)),
                    Cell::from(format!("{:.2}", curr_price)).style(Style::default().fg(Color::Green)),
                    Cell::from(pnl_str).style(Style::default().fg(pnl_color)),
                ];

                Row::new(cells).height(1)
            })
            .collect();

        // Format title with cash balance
        let title = format!(" Holdings | Cash: ${:.2} ", self.cash);
        
        let table = Table::new(
            rows,
            &[
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .title(title.bold())
        )
        .row_highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .highlight_symbol("> ");

        table.render(area, buf);
    }
}

impl Widget for &HoldingsComponent {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.holdings.is_empty() {
            let empty_text = Text::from(vec![
                Line::from(""),
                Line::from("No holdings").centered(),
                Line::from(""),
                Line::from("Buy stocks to see them here").centered().dim(),
            ]);

            Paragraph::new(empty_text)
                .centered()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_set(border::ROUNDED)
                        .title(" Holdings ".bold()),
                )
                .render(area, buf);
        } else {
            self.render_table(area, buf);
        }
    }
}
