# Objective

A CLI-native trading OS that tracks your portfolio, strategies,
and performance, eventually using crypto as the universal money layer.

This is aimed at technical beginners who want real-time prices and a safe way to learn trading fundamentals through paper trading, using a no-bloat command-line toolkit that teaches how portfolios and risk really work.

## Because currently, the alternatives are:

- overwhelming GUIs
- expensive quant platforms
- overly simplistic paper trading apps
- spreadsheets

## Sales pitch

- Paper trade with real market data

- Track positions, P&L, and risk precisely

- Build and test strategies incrementally

- Stay fully local and transparent

# MVP 2025 december week 2

1. As a user I want to fund my account, so that I can use the app - **Done**
2. As a user I want to buy a stock, so that I can participate in the market - **Done**
3. As a user I want to see my holdings so that I can track the net asset value - **Done**
4. As a user I want to sell a stock, so that I can participate in the market - **Done**
5. As a user I want to see the prices, so that I know what to buy/sell at - **Done**
6. As a user I want to see a list of executed buy/sell trades - **Done**

# Feature Roadmap

Here is the path to evolve the application from a basic paper trading tool into a serious platform for learning and strategy development.

### Tier 1: Foundational Features for Serious Trading

These features move beyond simple market orders and make the simulation realistic and robust.

1.  **Advanced Order Types:** Real trading isn't just "buy now." Users need control.
    *   **Limit Orders:** `buy AAPL --limit 150.00` - Execute a trade only at a specific price or better.
    *   **Stop-Loss / Take-Profit Orders:** `sell AAPL --stop-loss 140.00` - Fundamental risk management. Automatically exit a position to lock in profit or prevent further loss.

2.  **Robust, Queryable History:** A single JSON file for state is not scalable or analyzable.
    *   **Switch to SQLite:** Replace `Storage.rs`'s file I/O with a local SQLite database (`rusqlite` crate). It's still a single local file, fast, and allows for complex queries on trade history.
    *   **Detailed Trade Log:** Every single fill, fee, and order should be an immutable record in the database. This is the ground truth for all P&L calculations.

3.  **Realistic Cost Simulation:** "Fake" P&L is a trap for beginners.
    *   **Configurable Commissions/Fees:** Add a setting to model broker commissions (e.g., $0.01 per share). This forces users to account for transaction costs.

#### UX improvements

1. Single keypress keybind.
2. Recents dropdown for price lookup etc.

### Tier 2: Data Analysis & Basic Strategy

This is where the "data-first" promise comes to life, helping users build and test strategies.

1.  **Historical Data Access:** Users need to look back to plan forward.
    *   **`history` Command:** Create a command (`history AAPL --period 1y --interval 1d`) to fetch and display historical OHLCV (Open, High, Low, Close, Volume) data.

2.  **Basic Technical Analysis:** Give users the building blocks for analysis directly in the CLI.
    *   **`analyze` Command:** Implement a command to calculate and display basic technical indicators like Simple Moving Averages (`SMA:50`), Exponential Moving Averages (`EMA:20`), and Relative Strength Index (`RSI:14`).

3.  **Basic Scripting / Automation:** The first step towards backtesting.
    *   **Strategy Files:** Allow a user to define a simple sequence of commands in a text file and execute it with `naviin run-strategy <file>`.

### Tier 3: The Path to a Full Backtesting Engine

These features turn the tool from a trading simulator into a true research platform.

1.  **A Formal Backtesting Mode:**
    *   **`backtest` Command:** Create a top-level command that takes a strategy file and runs it against historical data, simulating the passage of time and executing trades.

2.  **Performance Metrics & Reporting:** A backtest is useless without a report.
    *   **End-of-Run Summary:** After a backtest, print a report with key metrics: Total P&L, Sharpe Ratio, Max Drawdown, and Win/Loss Ratio.

3.  **Configuration File:**
    *   **`config.toml`:** Create a configuration file (e.g., in `~/.config/naviin/`) for storing API keys, default parameters, and command aliases.

# Actual MVP

1. Money is stored as blockchain
2. Hook users via competitive trading/math
3. Sharing strats / education market can make more money
4. Democratise quant

# Dev Requirements

1. Use fastest and most supported language possible, Rust / Python
2. Able to handle large data
3. Fast and Simple GUI/TUI
