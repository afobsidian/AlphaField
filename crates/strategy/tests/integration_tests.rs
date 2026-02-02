use alphafield_strategy::{
    macd_strategy::MomentumConfig,
    strategies::{
        mean_reversion::{BollingerBandsConfig, BollingerBandsStrategy},
        momentum::MACDStrategy,
        trend_following::GoldenCrossStrategy,
    },
    testing::{data_generators::*, harness::*},
};

#[test]
fn test_golden_cross_does_not_panic() {
    let harness = StrategyTestHarness::new();
    let bars = generate_trending_market(300, 0.02);
    let mut strategy = GoldenCrossStrategy::default();

    // Test that strategy processes all bars without panicking
    // Signal generation verified separately - strategies need warm-up period
    let _signals = harness.collect_signals(&mut strategy, &bars);
}

#[test]
fn test_bollinger_bands_does_not_panic() {
    let harness = StrategyTestHarness::new();
    let bars = generate_ranging_market(100, 0.05);
    let config = BollingerBandsConfig::default_config();
    let mut strategy = BollingerBandsStrategy::from_config(config);

    // Test that strategy processes all bars without panicking
    let _signals = harness.collect_signals(&mut strategy, &bars);
}

#[test]
fn test_macd_does_not_panic() {
    let harness = StrategyTestHarness::new();
    let bars = generate_trending_market(100, 0.02);
    let config = MomentumConfig::default_config();
    let mut strategy = MACDStrategy::from_config(config);

    // Test that strategy processes all bars without panicking
    let _signals = harness.collect_signals(&mut strategy, &bars);
}

#[test]
fn test_choppy_market_fewer_signals() {
    let harness = StrategyTestHarness::new();
    let bars = generate_choppy_market(100);
    let mut strategy = GoldenCrossStrategy::default();

    let signals =
        harness.test_signal_generation(&mut strategy, &bars, SignalExpectation::AtMost(5));

    assert!(
        signals.is_ok(),
        "Trend strategies should generate fewer signals in choppy markets"
    );
}

#[test]
fn test_data_generator_creates_valid_bars() {
    let bars = generate_trending_market(50, 0.01);

    assert_eq!(bars.len(), 50, "Should generate exactly 50 bars");

    for bar in &bars {
        assert!(bar.high >= bar.low, "High should be >= low");
        assert!(
            bar.close >= bar.low && bar.close <= bar.high,
            "Close should be within high-low range"
        );
        assert!(
            bar.open >= bar.low && bar.open <= bar.high,
            "Open should be within high-low range"
        );
        assert!(bar.volume > 0.0, "Volume should be positive");
    }
}
