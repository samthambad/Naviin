use chrono::Utc;
use rust_decimal::prelude::*;

use crate::{AppState::AppState, FinanceProvider, UserInput};

#[derive(Clone, Debug, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Clone, Debug)]
pub struct Trade {
    symbol: String,
    quantity: Decimal,
    price_per: Decimal,
    side: Side,
    timestamp: i64,
}

// A completed transaction record for both market orders and executed conditional orders
impl Trade {
    // Create buy transaction record from immediate market order
    pub fn buy(symbol: String, quantity: Decimal, price_per: Decimal) -> Self {
        Self {
            symbol,
            quantity,
            price_per,
            side: Side::Buy,
            timestamp: Utc::now().timestamp(),
        }
    }

    // Create sell transaction record from immediate market order
    pub fn sell(symbol: String, quantity: Decimal, price_per: Decimal) -> Self {
        Self {
            symbol,
            quantity,
            price_per,
            side: Side::Sell,
            timestamp: Utc::now().timestamp(),
        }
    }

    pub fn get_symbol(&self) -> &String {
        &self.symbol
    }

    pub fn get_quantity(&self) -> Decimal {
        self.quantity
    }

    pub fn get_price_per(&self) -> Decimal {
        self.price_per
    }

    pub fn get_side(&self) -> &Side {
        &self.side
    }

    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn set_timestamp(&mut self, timestamp: i64) {
        self.timestamp = timestamp;
    }

    pub fn from_database(symbol: String, quantity: Decimal, price_per: Decimal, side: Side, timestamp: i64) -> Self {
        Self {
            symbol,
            quantity,
            price_per,
            side,
            timestamp,
        }
    }
}

// Category of conditional order to create
#[derive(Clone, Debug)]
pub enum OrderType {
    BuyLimit,
    StopLoss,
    TakeProfit,
}

// A pending order waiting for execution conditions to be met
#[derive(Clone, Debug)]
pub enum OpenOrder {
    BuyLimit {
        symbol: String,
        quantity: Decimal,
        price: Decimal,
        timestamp: i64,
    },
    StopLoss {
        symbol: String,
        quantity: Decimal,
        price: Decimal,
        timestamp: i64,
    },
    TakeProfit {
        symbol: String,
        quantity: Decimal,
        price: Decimal,
        timestamp: i64,
    },
}

impl OpenOrder {
    pub fn new(symbol: String, quantity: Decimal, price: Decimal, side: Side) -> Self {
        let timestamp = Utc::now().timestamp();
        match side {
            Side::Buy => OpenOrder::BuyLimit {
                symbol,
                quantity,
                price,
                timestamp,
            },
            Side::Sell => OpenOrder::StopLoss {
                symbol,
                quantity,
                price,
                timestamp,
            },
        }
    }

    pub fn get_symbol(&self) -> &String {
        match self {
            OpenOrder::BuyLimit { symbol, .. } => symbol,
            OpenOrder::StopLoss { symbol, .. } => symbol,
            OpenOrder::TakeProfit { symbol, .. } => symbol,
        }
    }

    pub fn get_qty(&self) -> Decimal {
        match self {
            OpenOrder::BuyLimit { quantity, .. } => *quantity,
            OpenOrder::StopLoss { quantity, .. } => *quantity,
            OpenOrder::TakeProfit { quantity, .. } => *quantity,
        }
    }

    pub fn get_price_per(&self) -> Decimal {
        match self {
            OpenOrder::BuyLimit { price, .. } => *price,
            OpenOrder::StopLoss { price, .. } => *price,
            OpenOrder::TakeProfit { price, .. } => *price,
        }
    }

    pub fn get_timestamp(&self) -> i64 {
        match self {
            OpenOrder::BuyLimit { timestamp, .. } => *timestamp,
            OpenOrder::StopLoss { timestamp, .. } => *timestamp,
            OpenOrder::TakeProfit { timestamp, .. } => *timestamp,
        }
    }

    pub fn get_side(&self) -> Side {
        match self {
            OpenOrder::BuyLimit { .. } => Side::Buy,
            OpenOrder::StopLoss { .. } => Side::Sell,
            OpenOrder::TakeProfit { .. } => Side::Sell,
        }
    }

    pub fn get_order_type(&self) -> &str {
        match self {
            OpenOrder::BuyLimit { .. } => "BuyLimit",
            OpenOrder::StopLoss { .. } => "StopLoss",
            OpenOrder::TakeProfit { .. } => "TakeProfit",
        }
    }
}

// Factory function to create pending orders based on user input and order type
pub fn create_order(order_type: OrderType) -> Option<OpenOrder> {
    let symbol = UserInput::ask_ticker()?;
    let quantity = UserInput::ask_quantity()?;
    let price = UserInput::ask_price()?;

    let order = match order_type {
        OrderType::BuyLimit => OpenOrder::BuyLimit {
            symbol,
            quantity,
            price,
            timestamp: Utc::now().timestamp(),
        },
        OrderType::StopLoss => OpenOrder::StopLoss {
            symbol,
            quantity,
            price,
            timestamp: Utc::now().timestamp(),
        },
        OrderType::TakeProfit => OpenOrder::TakeProfit {
            symbol,
            quantity,
            price,
            timestamp: Utc::now().timestamp(),
        },
    };
    Some(order)
}

// Execute buy limit order when current price is at or below limit price
pub async fn buy_limit(state: &mut AppState, order: &OpenOrder) -> bool {
    let symbol = order.get_symbol().clone();
    let limit_price = order.get_price_per();
    let purchase_qty = order.get_qty();
    let curr_cash = state.check_balance();
    let curr_price = FinanceProvider::curr_price(&symbol, false).await;
    let total_purchase_value = curr_price * purchase_qty;
    if curr_price <= limit_price {
        if total_purchase_value > curr_cash {
            return false;
        }
        state.withdraw_purchase(total_purchase_value);
        crate::Finance::add_to_holdings(&symbol, purchase_qty, curr_price, state).await;
        state.add_trade(Trade::buy(symbol, purchase_qty, curr_price));
        return true;
    }
    false
}

// Execute stop loss order when current price is at or below stop price to limit losses
pub async fn sell_stop_loss(state: &mut AppState, order: &OpenOrder) -> bool {
    let symbol = order.get_symbol().clone();
    let limit_price = order.get_price_per();
    let sale_qty = order.get_qty();
    let curr_price = FinanceProvider::curr_price(&symbol, false).await;
    let total_sale_value = curr_price * sale_qty;
    if curr_price <= limit_price {
        state.deposit_sell(total_sale_value);
        crate::Finance::remove_from_holdings(&symbol, sale_qty, state).await;
        state.add_trade(Trade::sell(symbol, sale_qty, curr_price));
        return true;
    }
    false
}

// Execute take profit order when current price is at or above target price to lock in gains
pub async fn sell_take_profit(state: &mut AppState, order: &OpenOrder) -> bool {
    let symbol = order.get_symbol().clone();
    let take_profit_price = order.get_price_per();
    let sale_qty = order.get_qty();
    let curr_price = FinanceProvider::curr_price(&symbol, false).await;
    let total_sale_value = take_profit_price * sale_qty;
    if curr_price >= take_profit_price {
        state.deposit_sell(total_sale_value);
        crate::Finance::remove_from_holdings(&symbol, sale_qty, state).await;
        state.add_trade(Trade::sell(symbol, sale_qty, take_profit_price));
        return true;
    }
    false
}
