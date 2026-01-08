use alphafield_core::{Order, OrderSide, OrderStatus, OrderType};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub total_value: f64,
    pub cash: f64,
    pub positions_value: f64,
    pub pnl: f64,
    pub pnl_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub pnl: f64,
    pub pnl_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
}

pub fn generate_mock_portfolio() -> Portfolio {
    Portfolio {
        total_value: 105_432.50,
        cash: 45_000.00,
        positions_value: 60_432.50,
        pnl: 5_432.50,
        pnl_percent: 5.43,
    }
}

pub fn generate_mock_positions() -> Vec<Position> {
    vec![
        Position {
            symbol: "BTCUSDT".to_string(),
            quantity: 0.5,
            entry_price: 92_000.0,
            current_price: 93_100.0,
            pnl: 550.0,
            pnl_percent: 1.20,
        },
        Position {
            symbol: "ETHUSDT".to_string(),
            quantity: 10.0,
            entry_price: 3_050.0,
            current_price: 3_092.0,
            pnl: 420.0,
            pnl_percent: 1.38,
        },
        Position {
            symbol: "SOLUSDT".to_string(),
            quantity: 50.0,
            entry_price: 138.0,
            current_price: 141.5,
            pnl: 175.0,
            pnl_percent: 2.54,
        },
    ]
}

pub fn generate_mock_orders() -> Vec<Order> {
    vec![
        Order {
            id: Uuid::new_v4().to_string(),
            symbol: "BTCUSDT".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            quantity: 0.5,
            price: Some(92_000.0),
            status: OrderStatus::Filled,
            timestamp: Utc::now() - chrono::Duration::hours(2),
            oco_group_id: None,
            iceberg_hidden_qty: None,
            stop_loss: None,
            take_profit: None,
            parent_order_id: None,
            limit_chase_amount: None,
        },
        Order {
            id: Uuid::new_v4().to_string(),
            symbol: "ETHUSDT".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            quantity: 10.0,
            price: Some(3_050.0),
            status: OrderStatus::Filled,
            timestamp: Utc::now() - chrono::Duration::hours(1),
            oco_group_id: None,
            iceberg_hidden_qty: None,
            stop_loss: None,
            take_profit: None,
            parent_order_id: None,
            limit_chase_amount: None,
        },
        Order {
            id: Uuid::new_v4().to_string(),
            symbol: "SOLUSDT".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            quantity: 50.0,
            price: Some(138.0),
            status: OrderStatus::Filled,
            timestamp: Utc::now() - chrono::Duration::minutes(30),
            oco_group_id: None,
            iceberg_hidden_qty: None,
            stop_loss: None,
            take_profit: None,
            parent_order_id: None,
            limit_chase_amount: None,
        },
    ]
}

pub fn generate_mock_performance() -> PerformanceMetrics {
    PerformanceMetrics {
        sharpe_ratio: 2.34,
        max_drawdown: -8.5,
        win_rate: 68.2,
        total_trades: 22,
        winning_trades: 15,
        losing_trades: 7,
    }
}
