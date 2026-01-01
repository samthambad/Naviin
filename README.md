# Naviin

Naviin is a high-performance, asynchronous portfolio management and market monitoring CLI tool built with Rust. It provides a robust interface for tracking digital assets, managing virtual balances, and executing simulated trades based on real-time market data.

## Features

- **Real-time Market Data:** Integration with live financial data providers to fetch up-to-the-minute asset prices.
- **Portfolio Tracking:** Manage multiple holdings with automatic average cost calculation and real-time P&L tracking.
- **Asynchronous Execution:** Built on top of `tokio`, featuring a non-blocking background order monitoring system for limit orders.
- **Transaction History:** Comprehensive logging of all trades (buys/sells) with precise timestamps.
- **State Persistence:** Robust serialization of application state to ensure data continuity across sessions.
- **Concurrency-Safe State Management:** Utilizes thread-safe primitives (`Arc`, `Mutex`, `AtomicBool`) to maintain state integrity during background processing.

## Tech Stack

- **Language:** Rust (Edition 2024)
- **Runtime:** Tokio (Asynchronous I/O)
- **Data Serialization:** Serde, Serde JSON (future migration to sql)
- **Market Data:** Integration with financial APIs via `yfinance-rs` (for now)

## Architecture

The project is structured with a modular design emphasizing separation of concerns:

- `AppState`: Centralized, thread-safe state management.
- `Finance`: Core business logic for trade execution and portfolio balancing.
- `FinanceProvider`: Abstraction layer for external market data interfaces.
- `Storage`: Data persistence layer handling JSON-based state serialization.
- `UserInput`: Command-line interface and interactive user feedback loop.

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/Naviin.git
   cd Naviin/naviin
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Run the application:
   ```bash
   cargo run
   ```

## Development

The project includes a comprehensive suite of tests covering state transitions, financial calculations, and storage operations.

To run the tests:
```bash
cargo test
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
