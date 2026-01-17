use alphafield_core::{Bar, SignalType, Strategy};
use alphafield_strategy::strategies::RSIReversionStrategy;
use chrono::Utc;

fn create_bar(price: f64) -> Bar {
    Bar {
        timestamp: Utc::now(),
        open: price,
        high: price + 1.0,
        low: price - 1.0,
        close: price,
        volume: 1000.0,
    }
}

fn main() {
    let mut strategy = RSIReversionStrategy::new(3, 30.0, 70.0);

    let prices = vec![
        50.0, 52.0, 54.0, // Neutral
        60.0, 70.0, 80.0, 90.0, // Overbought
        80.0, 70.0, 60.0, 50.0, 40.0, 30.0, 20.0, // Oversold
    ];

    for (i, price) in prices.iter().enumerate() {
        if let Some(signals) = strategy.on_bar(&create_bar(*price)) {
            println!("Bar {}: Price = {:.1}, Signals: {}", i, price, signals.len());
            for signal in &signals {
                println!("  - {:?}: {}", signal.signal_type, signal.metadata.as_ref().unwrap_or(&"no metadata".to_string()));
            }
        } else {
            println!("Bar {}: Price = {:.1}, No signals", i, price);
        }
    }
}
