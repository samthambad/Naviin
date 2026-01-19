CREATE TABLE appstate
(
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    cash_balance REAL    NOT NULL,
    updated_at   INTEGER NOT NULL
);

CREATE TABLE holdings
(
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol     TEXT    NOT NULL,
    quantity   REAL    NOT NULL,
    avg_cost   REAL    NOT NULL,
    account_id INTEGER NOT NULL,
    FOREIGN KEY (account_id) REFERENCES account (id)
);

CREATE TABLE trades
(
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol     TEXT    NOT NULL,
    quantity   REAL    NOT NULL,
    price_per  REAL    NOT NULL,
    side       TEXT    NOT NULL, -- "BUY" or "SELL"
    order_type TEXT NOT_NULL,
    timestamp  INTEGER NOT NULL,
    account_id INTEGER NOT NULL,
    FOREIGN KEY (account_id) REFERENCES account (id)
);

CREATE TABLE open_orders
(
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    order_type TEXT    NOT NULL, -- "BUY_LIMIT", "STOP_LOSS", "TAKE_PROFIT"
    symbol     TEXT    NOT NULL,
    quantity   REAL    NOT NULL,
    price      REAL    NOT NULL,
    timestamp  INTEGER NOT NULL,
    account_id INTEGER NOT NULL,
    FOREIGN KEY (account_id) REFERENCES account (id)
);
