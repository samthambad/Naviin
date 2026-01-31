/// Open Orders Component - Displays pending orders
///
/// Shows all open/pending orders (BuyLimit, StopLoss, TakeProfit) with details.
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Widget},
};

use crate::Orders::OpenOrder;

/// Component that displays open orders
pub struct OpenOrdersComponent {
    /// List of open orders
    orders: Vec<OpenOrder>,
    /// Current selected row
    table_state: TableState,
}

impl OpenOrdersComponent {
    /// SECTION: Constructor

    /// Creates a new open orders component
    pub fn new() -> Self {
        Self {
            orders: Vec::new(),
            table_state: TableState::default(),
        }
    }

    /// SECTION: Data Management

    /// Updates the orders list
    pub fn update_orders(&mut self, orders: Vec<OpenOrder>) {
        self.orders = orders;
        if !self.orders.is_empty() && self.table_state.selected().is_none() {
            self.table_state.select(Some(0));
        }
    }

    /// SECTION: Navigation

    /// Moves to next order
    pub fn next(&mut self) {
        if self.orders.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.orders.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    /// Moves to previous order
    pub fn previous(&mut self) {
        if self.orders.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.orders.len() - 1
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
            Cell::from("Type").style(Style::default().fg(Color::Yellow).bold()),
            Cell::from("Symbol").style(Style::default().fg(Color::Yellow).bold()),
            Cell::from("Qty").style(Style::default().fg(Color::Yellow).bold()),
            Cell::from("Price").style(Style::default().fg(Color::Yellow).bold()),
        ])
        .height(1);

        let rows: Vec<Row> = self
            .orders
            .iter()
            .map(|order| {
                let order_type = order.get_order_type();
                let symbol = order.get_symbol();
                let qty = order.get_qty();
                let price = order.get_price_per();

                // Color based on order type
                let type_color = match order_type {
                    "BuyLimit" => Color::Green,
                    "StopLoss" => Color::Red,
                    "TakeProfit" => Color::Blue,
                    _ => Color::White,
                };

                let cells = vec![
                    Cell::from(order_type.to_string()).style(Style::default().fg(type_color)),
                    Cell::from(symbol.clone()),
                    Cell::from(format!("{:.2}", qty)),
                    Cell::from(format!("{:.2}", price)),
                ];

                Row::new(cells).height(1)
            })
            .collect();

        let table = Table::new(
            rows,
            &[
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .title(" Open Orders ".bold())
        )
        .row_highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .highlight_symbol("> ");

        table.render(area, buf);
    }
}

impl Widget for &OpenOrdersComponent {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.orders.is_empty() {
            let empty_text = Text::from(vec![
                Line::from(""),
                Line::from("No open orders").centered(),
                Line::from(""),
                Line::from("Create orders with buylimit, stoploss, takeprofit")
                    .centered()
                    .dim(),
            ]);

            Paragraph::new(empty_text)
                .centered()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_set(border::ROUNDED)
                        .title(" Open Orders ".bold()),
                )
                .render(area, buf);
        } else {
            self.render_table(area, buf);
        }
    }
}
