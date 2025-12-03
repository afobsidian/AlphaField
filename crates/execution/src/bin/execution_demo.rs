//! Demo binary for testing Execution Engine

use alphafield_core::{ExecutionService, Order, OrderSide, OrderStatus, OrderType};
use alphafield_execution::{PaperTradingClient, RiskManager, MaxOrderValue};
use chrono::Utc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AlphaField Execution Engine Demo\n");

    // 1. Setup Execution Service (Paper Trading)
    let paper_client = PaperTradingClient::new();
    
    // 2. Setup Risk Manager
    let mut risk_manager = RiskManager::new(paper_client);
    
    // Add rule: Max order value $10,000
    risk_manager.add_check(MaxOrderValue { max_value: 10_000.0 });
    println!("✓ Risk Manager initialized (Max Order Value: $10,000)");

    println!("\n📊 TEST 1: Valid Order (Buy 1 BTC @ $50,000)");
    println!("{}", "-".repeat(70));
    
    // Note: In paper trading we assume price is provided or estimated.
    // Since MaxOrderValue check needs price, we must provide it for Limit orders.
    let valid_order = Order {
        id: Uuid::new_v4().to_string(),
        symbol: "BTCUSDT".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        quantity: 0.1, // $5,000 value
        price: Some(50_000.0),
        status: OrderStatus::New,
        timestamp: Utc::now(),
    };

    match risk_manager.submit_order(&valid_order).await {
        Ok(id) => println!("✓ Order Submitted Successfully! ID: {}", id),
        Err(e) => eprintln!("✗ Failed: {}", e),
    }

    println!("\n📊 TEST 2: Invalid Order (Buy 10 BTC @ $50,000 = $500,000)");
    println!("{}", "-".repeat(70));
    
    let invalid_order = Order {
        id: Uuid::new_v4().to_string(),
        symbol: "BTCUSDT".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        quantity: 10.0, // $500,000 value -> Should fail risk check
        price: Some(50_000.0),
        status: OrderStatus::New,
        timestamp: Utc::now(),
    };

    match risk_manager.submit_order(&invalid_order).await {
        Ok(id) => println!("✗ Order Submitted (Unexpected)! ID: {}", id),
        Err(e) => println!("✓ Order Rejected by Risk Manager: {}", e),
    }

    println!("\n{}", "=".repeat(70));
    println!("🎉 Demo Completed");
    println!("{}", "=".repeat(70));

    Ok(())
}
