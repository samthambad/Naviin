# Naviin

**A high-performance, Rust-powered CLI for portfolio tracking and simulated trading—starting as an accessible tool for technical beginners and evolving toward a broader audience as intuitive keybinds make complex operations as simple as typing.**

Naviin delivers real-time multi-asset monitoring (equities, ETFs, FX, commodities), virtual trade execution, and concurrency-safe state management. Built for speed and safety in Rust, it lays the foundation for future expansion into advanced strategy development and DeFi/RWA integration.

## Current Features

<img title="Screenshot" alt="Screenshot" src="/images/screenshot.png">

- **Real-time Market Data**: Multi-asset price fetching (Equities, ETFs, FX, and Commodities) via financial APIs.
- **Portfolio Tracking**: Multi-asset holdings with automatic average cost basis, real-time P&L, and performance metrics.
- **Advanced Order Execution**: Support for Limit, Stop-Loss, and Take-Profit orders.
- **Persistent Relational Storage**: SQL-based persistence (SQLite + SeaORM) for session continuity and trade history.
- **Concurrency-Safe Design**: Thread-safe state using `Arc<Mutex<>>`, `AtomicBool`, and Tokio for reliable background processing.

## Roadmap & Future Vision

Naviin is evolving from a paper trading tool into a robust platform for financial research and strategy development. The roadmap is structured into three progressive tiers:

### Phase 1:
- [x] Advanced Order Execution: Stop-Loss and Take-Profit orders.
- [x] Persistent Relational Storage: Migration to SQL for scalable trade history.
- [x] Better TUI

### Phase 2: Technical Analysis & Strategy Engine
- [x] Import
- [ ] Backtesting (WIP)

## Tech Stack

- **Core Language**: Rust (latest stable)
- **Async Runtime**: Tokio
- **Market Data**: `yfinance-rs` (current); extensible abstraction layer
- **Concurrency Primitives**: `std::sync::Arc`, `Mutex`, `AtomicBool`

## Architecture Overview

Modular, separation-of-concerns design:
- `AppState` – Centralized, thread-safe state
- `Finance` – Trade execution, P&L, portfolio logic
- `FinanceProvider` – Market data abstraction
- `Storage` – Relational persistence layer (SQLite via SeaORM)
- `Tui` – Ratatui/crossterm terminal interface
- `commands` – Command parser and command handlers

## Terminal UI

Naviin runs as a full-screen terminal UI rather than a line-by-line shell. On startup, `main.rs` enables crossterm raw mode, switches into the terminal alternate screen, loads persisted state from the database, starts background order monitoring, and launches the `Tui` event loop. On exit, it restores the normal terminal screen, stops background monitoring, saves state, and closes the database connection.

### Screen Layout

The UI is split vertically into three sections:

- **Top row, 40% height**: live account panels.
- **Middle, 20% height**: command input.
- **Bottom, 40% height**: command output.

The top row is split horizontally into:

- **Holdings**: current positions and cash balance, with refreshed market prices.
- **Open Orders**: pending buy limit, stop loss, and take profit orders.
- **Watchlist**: tracked symbols and refreshed market prices.

The input panel is where commands are typed. Results, help text, trade history, errors, and import prompts appear in the output panel.

### Event Loop and Refreshing

`Tui::run` performs an initial state refresh, draws the screen, then waits on two async events:

- Keyboard input from crossterm, read through `spawn_blocking` so terminal input does not block Tokio.
- A 5-second refresh timer.

After a command runs, the TUI refreshes all state-backed panels: holdings, open orders, watchlist, and cash. On each 5-second timer tick, it refreshes only market prices for holdings and the watchlist. Price refreshes run concurrently with `tokio::join!`.

### Keyboard Controls

| Key | Action |
| --- | --- |
| Type text | Insert command text at the cursor |
| Left / Right | Move the input cursor |
| Home / End | Move to start/end of the input |
| Backspace | Delete the previous character |
| Enter | Execute the current command |
| PageUp / PageDown | Scroll command output |
| Ctrl+Home / Ctrl+End | Jump to top/bottom of output |
| `Q` | Quit immediately |

The typed commands `exit` and `quit` also close the application. The `clear` command clears the output panel.

Commands are whitespace-delimited and case-insensitive for the command name. Symbols are normalized to uppercase by command handlers.

Common commands:

| Command | Purpose |
| --- | --- |
| `fund <amount>` | Add cash to the account |
| `withdraw <amount>` | Withdraw cash |
| `summary` | Show account summary |
| `price <symbol>` | Fetch a current market price |
| `addwatch <symbol>` | Add a symbol to the watchlist |
| `unwatch <symbol>` | Remove a symbol from the watchlist |
| `buy <symbol> <qty>` | Buy at current market price |
| `sell <symbol> <qty>` | Sell at current market price |
| `buylimit <symbol> <qty> <price>` | Create a buy limit order |
| `stoploss <symbol> <qty> <price>` | Create a stop loss order |
| `takeprofit <symbol> <qty> <price>` | Create a take profit order |
| `trades` | Show trade history |
| `import` | Start CSV import prompt |
| `stopbg` / `startbg` | Stop or start background order monitoring |
| `reset` | Reset account state |
| `help` | Show command help |

### Background Orders

Background order monitoring starts automatically. It checks open orders roughly every 10 seconds while enabled:

- `BuyLimit` executes when the current price is at or below the limit price.
- `StopLoss` executes when the current price is at or below the stop price.
- `TakeProfit` executes when the current price is at or above the target price.

Executed orders are converted into trades, holdings/cash are updated, and the order is removed from open orders. `stopbg` pauses this monitoring; `startbg` resumes it.

### Import Mode

The `import` command puts the app into a one-command prompt mode. The next input is treated as a CSV path instead of a normal command. Enter `cancel` or an empty path to leave import mode. Expected CSV columns are:

```
date,asset,asset_type,side,quantity,price,currency
```

## Getting Started

### Prerequisites
- Rust toolchain (latest stable)
- Cargo

### Installation & Running
```bash
git clone https://github.com/samthambad/Naviin.git
cd Naviin/naviin
cargo build --release
cargo run
```

## Development

Comprehensive test suite covering state transitions, calculations, concurrency, and storage.

```bash
cargo test          # Run tests
cargo fmt           # Format code
cargo clippy        # Lint
```

For TUI architecture, extension points, and manual testing notes, see
[docs/tui-developer-guide.md](docs/tui-developer-guide.md).

---

Built with ❤️ by Samarth Thambad. Stars, feedback, and PRs fuel the journey from beginner-friendly CLI to a democratised finance platform!
