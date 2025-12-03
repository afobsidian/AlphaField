use alphafield_backtest::{
    BacktestEngine, OrderRequest, OrderSide, OrderType, SlippageModel, Strategy,
};
use alphafield_core::Bar;
use chrono::{TimeZone, Utc};

type Result<T> = alphafield_backtest::error::Result<T>;

struct BuyAndHoldStrategy {
    symbol: String,
    invested: bool,
}

impl BuyAndHoldStrategy {
    fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            invested: false,
        }
    }
}

impl Strategy for BuyAndHoldStrategy {
    fn on_bar(&mut self, _bar: &Bar) -> Result<Vec<OrderRequest>> {
        if !self.invested {
            self.invested = true;
            // Calculate quantity based on assumed cash (simplified for example)
            // In a real strategy, we'd need access to Portfolio state via Context
            // For now, hardcoding a quantity that fits
            let quantity = 980.0;

            Ok(vec![OrderRequest {
                symbol: self.symbol.clone(),
                side: OrderSide::Buy,
                quantity,
                order_type: OrderType::Market,
            }])
        } else {
            Ok(Vec::new())
        }
    }
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // 1. Setup Engine
    let mut engine = BacktestEngine::new(
        100_000.0,                           // Initial Cash
        0.001,                               // 0.1% Fee
        SlippageModel::FixedPercent(0.0005), // 0.05% Slippage
    );

    // 2. Generate Mock Data (Linear uptrend)
    let mut bars = Vec::new();
    let mut price = 100.0;
    let start_time = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();

    for i in 0..100 {
        let timestamp = start_time + chrono::Duration::days(i);
        bars.push(Bar {
            timestamp,
            open: price,
            high: price + 2.0,
            low: price - 1.0,
            close: price + 1.0,
            volume: 1000.0,
        });
        price += 1.0;
    }

    let symbol = "BTC/USD";
    engine.add_data(symbol, bars.clone());

    // 3. Set Strategy
    let strategy = BuyAndHoldStrategy::new(symbol);
    engine.set_strategy(Box::new(strategy));

    // 4. Run Backtest
    let metrics = engine.run()?;

    // 5. Report
    println!("--- Backtest Results ---");
    println!("Total Return: {:.2}%", metrics.total_return * 100.0);
    println!("CAGR: {:.2}%", metrics.cagr * 100.0);
    println!("Sharpe Ratio: {:.2}", metrics.sharpe_ratio);
    println!("Max Drawdown: {:.2}%", metrics.max_drawdown * 100.0);
    println!("Volatility: {:.2}%", metrics.volatility * 100.0);
    println!(
        "Final Equity: {:.2}",
        engine
            .portfolio
            .total_equity(&std::collections::HashMap::from([(
                symbol.to_string(),
                bars.last().unwrap().close
            )]))
    );

    Ok(())
}
