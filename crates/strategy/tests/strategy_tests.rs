use alphafield_core::{Bar, SignalType, Strategy};
use alphafield_strategy::strategies::{
    BollingerBandsStrategy, GoldenCrossStrategy, MeanReversionStrategy, MomentumStrategy,
    RsiStrategy,
};
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

#[test]
fn test_golden_cross_signals() {
    let mut strategy = GoldenCrossStrategy::new(3, 5);

    // Setup: Fast < Slow initially
    // We need enough bars to establish the trend
    let prices = vec![
        100.0, 90.0, 80.0, 70.0, 60.0, // Now reverse trend sharply
        80.0, 100.0, 120.0, 140.0,
    ];

    let mut all_signals = Vec::new();
    for (i, price) in prices.iter().enumerate() {
        if let Some(signals) = strategy.on_bar(&create_bar(*price)) {
            for signal in &signals {
                println!(
                    "Bar {}: Price {}, Signal {:?}",
                    i, price, signal.signal_type
                );
            }
            all_signals.extend(signals);
        }
    }

    // Should eventually get a Buy signal as Fast crosses above Slow
    assert!(all_signals.iter().any(|s| s.signal_type == SignalType::Buy));
}

#[test]
fn test_mean_reversion_backward_compatibility() {
    // Test that MeanReversionStrategy works as a backward compatibility alias
    let mut strategy = MeanReversionStrategy::new(10, 2.0);

    // Feed enough bars to initialize indicators (BB needs 10, RSI needs 14)
    for i in 0..30 {
        let price = 100.0 + (i as f64);
        let _ = strategy.on_bar(&create_bar(price));
    }

    // Verify the alias points to BollingerBandsStrategy
    assert_eq!(strategy.name(), "Bollinger Bands Mean Reversion");
}

#[test]
fn test_rsi_signals() {
    let mut strategy = RsiStrategy::new(3, 30.0, 70.0);

    // Create a sequence that goes from neutral to overbought to oversold
    let prices = vec![
        50.0, 52.0, 54.0, // Neutral
        60.0, 70.0, 80.0, 90.0, // Overbought
        80.0, 70.0, 60.0, 50.0, 40.0, 30.0, 20.0, // Oversold
    ];

    let mut buy_signals = 0;
    let mut sell_signals = 0;

    for price in prices {
        if let Some(signals) = strategy.on_bar(&create_bar(price)) {
            for signal in signals {
                match signal.signal_type {
                    SignalType::Buy => buy_signals += 1,
                    SignalType::Sell => sell_signals += 1,
                    _ => {}
                }
            }
        }
    }

    assert!(sell_signals > 0, "Should have sell signals (overbought)");
    assert!(buy_signals > 0, "Should have buy signals (oversold)");
}

#[test]
fn test_bollinger_bands_signals() {
    // Basic sanity check that BollingerBandsStrategy can be created and used
    let mut strategy = BollingerBandsStrategy::new(10, 2.0);

    // Feed enough bars to initialize indicators (BB needs 10, RSI needs 14)
    for i in 0..30 {
        let price = 100.0 + (i as f64);
        let _ = strategy.on_bar(&create_bar(price));
    }

    // Strategy should be functional - we're not testing the specific logic here
    // (that's covered by unit tests in bollinger_bands.rs)
    assert_eq!(strategy.name(), "Bollinger Bands Mean Reversion");
}

#[test]
fn test_momentum_signals() {
    let mut strategy = MomentumStrategy::new(10, 3, 6, 3);

    // Strong uptrend
    let prices: Vec<f64> = (0..20).map(|i| 100.0 + (i as f64) * 2.0).collect();

    let mut all_signals = Vec::new();
    for price in prices {
        if let Some(signals) = strategy.on_bar(&create_bar(price)) {
            all_signals.extend(signals);
        }
    }

    // Should have buy signals in strong uptrend
    assert!(all_signals.iter().any(|s| s.signal_type == SignalType::Buy));
}
