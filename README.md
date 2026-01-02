# Naviin

**A high-performance, Rust-powered CLI for portfolio tracking and simulated trading—starting as an accessible tool for technical beginners and evolving toward a broader audience as intuitive keybinds make complex operations as simple as typing.**

Naviin delivers real-time multi-asset monitoring (equities, ETFs, FX, commodities), virtual trade execution, and concurrency-safe state management. Built for speed and safety in Rust, it lays the foundation for future expansion into advanced strategy development and DeFi/RWA integration.

## Current Features

- **Real-time Market Data**: Multi-asset price fetching (Equities, ETFs, FX, and Commodities) via financial APIs.
- **Portfolio Tracking**: Multi-asset holdings with automatic average cost basis, real-time P&L, and performance metrics.
- **Simulated Trading**: Virtual buy/sell execution with limit order monitoring in a non-blocking background task.
- **Transaction History**: Detailed, timestamped logging of all trades.
- **State Persistence**: JSON-based serialization for seamless session continuity (planned migration to SQL).
- **Concurrency-Safe Design**: Thread-safe state using `Arc<Mutex<>>`, `AtomicBool`, and Tokio for reliable background processing.

## Roadmap & Future Vision

Naviin is evolving from a paper trading tool into a robust platform for financial research and strategy development. The roadmap is structured into three progressive tiers:

### Phase 1: Institutional-Grade Foundations
- **Advanced Order Execution**: Implementation of Stop-Loss and Take-Profit orders to complement existing limit orders.
- **Persistent Relational Storage**: Migration from JSON-based storage to **SQLite** (`rusqlite`) for scalable, queryable trade history and audit logs.
- **Transaction Modeling**: Integration of configurable commission and fee structures to simulate realistic net-of-fee performance.

### Phase 2: Technical Analysis & Strategy Engine
- **Multi-Asset Class Support**: Leveraging broad market data coverage to support global equities, ETFs, currencies (FX), and commodities.
- **Historical Data Analysis**: Advanced CLI tools for fetching and visualizing historical OHLCV data across multiple timeframes.
- **On-the-Fly Technical Indicators**: Built-in support for SMA, EMA, RSI, and MACD calculations directly within the terminal.
- **Strategy Automation**: A lightweight scripting engine to define and execute automated trading rules via localized strategy files.

### Phase 3: Quantitative Research Platform
- **Backtesting Suite**: A formal engine to simulate strategy performance against historical data with millisecond precision.
- **Advanced Performance Analytics**: Automated reporting of risk-adjusted metrics including Sharpe Ratio, Maximum Drawdown, and Win/Loss distribution.
- **DeFi & RWA Integration**: Exploring on-chain settlement layers and Real-World Asset (RWA) tokenization to bridge the gap between traditional finance and DeFi.

---

## Tech Stack

- **Core Language**: Rust (latest stable)
- **Async Runtime**: Tokio
- **Serialization**: Serde + serde_json (SQL migration planned)
- **Market Data**: `yfinance-rs` (current); extensible abstraction for future providers
- **Concurrency Primitives**: `std::sync::Arc`, `Mutex`, `AtomicBool`

## Architecture Overview

Modular design with clear separation of concerns:
- `AppState` – Centralized, thread-safe application state.
- `Finance` – Core logic for trades, P&L calculations, and portfolio management.
- `FinanceProvider` – Abstracted market data interface.
- `Storage` – Persistence layer (JSON → future SQL).
- `UserInput` – Interactive CLI loop and feedback.

## Getting Started

### Prerequisites
- Rust toolchain (latest stable)
- Cargo

### Installation & Running
```bash
git clone https://github.com/your-username/Naviin.git
cd Naviin/naviin
cargo build --release
cargo run
```

## Development

The project includes comprehensive tests for state transitions, financial calculations, concurrency safety, and storage.

Run tests:
```bash
cargo test
```

Code style enforcement:
```bash
cargo fmt
cargo clippy
```

## Contributing

Contributions are welcome! This is a part-time student project, so external input helps accelerate progress.

- Open issues for bugs, features, or DeFi roadmap discussions.
- Submit PRs with clear descriptions.
- Focus areas: performance optimizations, safer concurrency, better CLI UX, or early DeFi prototypes.

Please follow Rust coding conventions and add tests for new logic.

## License

Dual-licensed under **MIT OR Apache-2.0** at your option (standard for Rust ecosystem compatibility). See `LICENSE-MIT` and `LICENSE-APACHE` files for details.

---

Built with ❤️ and Rust's borrow checker. Feedback and stars appreciated as this evolves from CLI tool to DeFi platform!
