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
- [ ] Better User Experience

### Phase 2: Technical Analysis & Strategy Engine
- Dividend tracking
- Transaction Modeling: Configurable commissions and fees for realistic performance simulation.
- ...

### Phase 3: Quantitative Research Platform
- ...

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
- `UserInput` – Interactive CLI loop

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

Comprehensive test suite covering state transitions, calculations, concurrency, and storage.

```bash
cargo test          # Run tests
cargo fmt           # Format code
cargo clippy        # Lint
```

---

Built with ❤️ by Samarth Thambad. Stars, feedback, and PRs fuel the journey from beginner-friendly CLI to a democratised finance platform!
