# TUI Developer Guide

This guide is for contributors changing Naviin's terminal UI. The TUI is built with
Ratatui for rendering, crossterm for terminal input, and Tokio for async refresh
work.

## Files To Know

- `naviin/src/main.rs`: terminal setup, raw mode, alternate screen, database load,
  background order monitoring startup, TUI launch, and cleanup.
- `naviin/src/tui.rs`: top-level TUI coordinator. Owns layout, event handling,
  command execution, redraw timing, and component refreshes.
- `naviin/src/components/`: render-only UI components.
- `naviin/src/commands.rs`: command parser and command handlers. The TUI calls this
  when the user presses Enter.
- `naviin/src/AppState.rs`: shared portfolio state used by commands and refreshes.
- `naviin/src/FinanceProvider.rs`: market price fetching used by holdings and
  watchlist refreshes.

The crate currently exposes modules with capitalized names such as `AppState`,
`Finance`, `Orders`, and `Tui`. Match the existing module names when importing.

## Runtime Flow

Startup begins in `main.rs`:

1. Load `.env` and connect to `DATABASE_URL`.
2. Load `AppState` from storage.
3. Start background order monitoring with `monitor_order`.
4. Enter raw terminal mode and alternate screen.
5. Construct `Tui::new(...)` with the initial watchlist, shared state, database
   connection, and background-monitor flag.
6. Run `Tui::run`.
7. On exit, restore the terminal, stop background monitoring, save state, and close
   the database connection.

`Tui::run` does an initial full refresh and draw, then waits on two event sources:

- keyboard input from crossterm through `wait_for_event`, wrapped in
  `tokio::task::spawn_blocking`;
- a 5-second timer used for price-only refreshes.

The loop redraws after input handling or a timer refresh.

## Layout

`Tui::calculate_layout` owns the screen geometry:

- top 40%: holdings, open orders, watchlist;
- middle 20%: command input;
- bottom 40%: command output.

The top row is split horizontally into 33% / 34% / 33%. Components should render
inside the `Rect` they are given and should not know about the global layout.

## Component Pattern

Each component keeps only display state and implements `Widget for &Component`.
Current components:

- `HoldingsComponent`: holdings map, cached prices, cash, table selection.
- `OpenOrdersComponent`: pending orders and table selection.
- `WatchlistComponent`: watched symbols, cached prices, table selection.
- `InputComponent`: current command text and cursor position.
- `OutputComponent`: current output text, previous output history, scroll offset.

Use this pattern for new panels:

1. Store UI-local data in the component struct.
2. Add update methods that accept owned snapshots from app state.
3. Keep rendering in `impl Widget for &YourComponent`.
4. Avoid database access, shared-state locking, and command parsing inside
   components.

Components should be cheap to draw. Expensive work, especially async work, belongs
in `Tui` refresh methods or command handlers.

## State And Refresh Rules

There are two refresh paths in `tui.rs`:

- `refresh_all`: reads holdings, open orders, watchlist, and cash from `AppState`;
  updates components; drops the state lock; then refreshes prices.
- `refresh_prices_only`: refreshes holdings and watchlist market prices with
  `tokio::join!`.

Keep this separation intact:

- use `refresh_all` after commands that may change state;
- use `refresh_prices_only` for timer-based market data updates;
- do not hold `state.lock().unwrap()` across `.await`.

If a new component needs app-state data, add it to `refresh_all`. If it also needs
external async data, copy the state data first, drop the lock, then fetch.

## Keyboard Handling

`Tui::handle_key_event` owns key bindings:

- `Q`: quit immediately;
- Left / Right: move input cursor;
- Home / End: move input cursor to start/end;
- character keys: insert into input;
- Backspace: delete previous character;
- Enter: execute command;
- PageUp / PageDown: scroll output;
- Ctrl+Home / Ctrl+End: jump output to top/bottom.

When adding bindings, keep global bindings in `Tui`. Component-specific navigation
can be delegated to component methods, but crossterm event matching should stay in
one place unless the app grows a focus system.

## Command Flow

When Enter is pressed, `Tui::execute_command`:

1. reads the input command;
2. commits the current output to output history;
3. clears the input;
4. handles TUI-local commands: `exit`, `quit`, and `clear`;
5. calls `commands::process_command`;
6. writes the returned string into `OutputComponent`;
7. calls `refresh_all`.

Add new business commands in `commands.rs`, not in `tui.rs`, unless the command is
purely a UI concern. Command handlers should return user-facing text for the output
panel and should persist state changes through `Storage::save_state` when needed.

Import mode is implemented in `commands.rs` with `AppState::is_pending_import`.
When import is pending, the next entered line is treated as a path rather than a
normal command.

## Adding A New Panel

1. Create `naviin/src/components/<name>.rs`.
2. Add `pub mod <name>;` to `naviin/src/components/mod.rs`.
3. Add the component field to `Tui`.
4. Initialize it in `Tui::new`.
5. Allocate space in `calculate_layout`.
6. Render it in `draw`.
7. Feed it data from `refresh_all` or another explicit refresh method.

Keep the component's public API narrow. A typical component only needs `new`,
`update_*`, optional navigation methods, optional async `refresh_*`, and `Widget`
rendering.

## Adding A New Command

1. Add a match arm in `process_command`.
2. Implement a focused `handle_*` function.
3. Validate argument count and parse errors before mutating state.
4. Normalize symbols with `to_uppercase`.
5. Avoid keeping the app-state mutex locked during price fetches or database saves.
6. Save state after successful mutations.
7. Add the command to `handle_help`.
8. Add tests for parser behavior or the underlying state transition when practical.

For market orders, follow the existing `buy` and `sell` handlers: validate input,
fetch price, inspect state, execute through `Finance`, save, and return a concise
confirmation.

## Testing And Manual Checks

Run these from `naviin/`:

```bash
cargo fmt
cargo test
cargo clippy
```

For manual TUI testing, make sure `.env` has a valid `DATABASE_URL`, then run:

```bash
cargo run
```

Check at least:

- the terminal restores after `Q`, `quit`, and Ctrl+C/error paths where applicable;
- typing, cursor movement, and Backspace work;
- output scrolling works on long help/trade output;
- state-changing commands update panels immediately;
- prices refresh without freezing input;
- small terminal sizes do not panic.

## Common Pitfalls

- Raw mode hides normal terminal behavior. If a panic leaves the terminal broken,
  run `reset` in the shell.
- `InputComponent` tracks cursor position as a byte index today. That is fine for
  ASCII commands, but needs work before supporting arbitrary Unicode input.
- `TableState` is stored but tables are rendered as stateless widgets today. If
  selection becomes interactive, switch to the stateful render path.
- `OutputComponent::scroll_to_bottom` sets `usize::MAX`; rendering clamps it to
  the real maximum.
- Price refreshes call the finance provider for each symbol. Be careful about API
  rate limits when adding more frequent refreshes or larger watchlists.
- The background order monitor mutates the same shared `AppState` used by the TUI.
  Keep lock scopes short and avoid long synchronous work while holding the mutex.
 