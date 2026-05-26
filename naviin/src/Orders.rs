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
    order_type: String, // "Market", "BuyLimit", "StopLoss", "TakeProfit"
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
            order_type: "Market".to_string(),
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
            order_type: "Market".to_string(),
        }
    }
    
    // Create buy transaction with specific order type
    fn buy_with_type(symbol: String, quantity: Decimal, price_per: Decimal, order_type: String) -> Self {
        Self {
            symbol,
            quantity,
            price_per,
            side: Side::Buy,
            timestamp: Utc::now().timestamp(),
            order_type,
        }
    }
    
    // Create sell transaction with specific order type
   fn sell_with_type(symbol: String, quantity: Decimal, price_per: Decimal, order_type: String) -> Self {
        Self {
            symbol,
            quantity,
            price_per,
            side: Side::Sell,
            timestamp: Utc::now().timestamp(),
            order_type,
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

    pub fn get_order_type(&self) -> &String {
        &self.order_type
    }

    pub fn from_database(symbol: String, quantity: Decimal, price_per: Decimal, side: Side, timestamp: i64, order_type: String) -> Self {
        Self {
            symbol,
            quantity,
            price_per,
            side,
            timestamp,
            order_type,
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
pub struct OpenOrder {
    symbol: String,
    quantity: Decimal,
    price: Decimal,
    timestamp: i64,
    order_type: OrderType,
    side: Side,
}

impl OpenOrder {
    pub fn new(symbol: String, quantity: Decimal, price: Decimal, order_type: OrderType, side: Side) -> Self {
        let timestamp = Utc::now().timestamp();
        Self {
            symbol,
            quantity,
            price,
            timestamp,
            order_type,
            side
        }
    }

    pub fn get_symbol(&self) -> &String {
        &self.symbol
    }

    pub fn get_qty(&self) -> Decimal {
        self.quantity
    }

    pub fn get_price_per(&self) -> Decimal {
        self.price
    }

    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn get_side(&self) -> Side {
        self.side.clone()
    }

    pub fn get_order_type(&self) -> OrderType {
        self.order_type.clone()
    }
}

// Factory function to create pending orders based on user input and order type
pub fn create_order(order_type: OrderType) -> Option<OpenOrder> {
    let symbol = UserInput::ask_ticker()?;
    let quantity = UserInput::ask_quantity()?;
    let price = UserInput::ask_price()?;

    let order = match order_type {
        OrderType::BuyLimit => OpenOrder {
            symbol,
            quantity,
            price,
            timestamp: Utc::now().timestamp(),
            order_type: OrderType::BuyLimit,
            side: Side::Buy,
        },
        OrderType::StopLoss => OpenOrder {
            symbol,
            quantity,
            price,
            timestamp: Utc::now().timestamp(),
            order_type: OrderType::StopLoss,
            side: Side::Sell
        },
        OrderType::TakeProfit => OpenOrder {
            symbol,
            quantity,
            price,
            timestamp: Utc::now().timestamp(),
            order_type: OrderType::TakeProfit,
            side: Side::Sell
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
        state.add_trade(Trade::buy_with_type(symbol, purchase_qty, curr_price, "BuyLimit".to_string()));
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
        state.add_trade(Trade::sell_with_type(symbol, sale_qty, curr_price, "StopLoss".to_string()));
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
        state.add_trade(Trade::sell_with_type(symbol, sale_qty, take_profit_price, "TakeProfit".to_string()));
        return true;
    }
    false
}
