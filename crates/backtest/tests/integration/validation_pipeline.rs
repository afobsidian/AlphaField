//! Integration tests for strategy validation pipeline
//!
//! These tests validate the entire validation workflow using the database
//! instead of CSV files.

use alphafield_backtest::{
    Strategy, StrategyAdapter, StrategyValidator, ValidationConfig, ValidationReport,
    ValidationThresholds, ValidationVerdict, WalkForwardConfig,
};
use alphafield_core::Bar;
use alphafield_data::database::DatabaseClient;
use alphafield_strategy::{
    BollingerBandsStrategy, DivergenceStrategy, GoldenCrossStrategy, MacdTrendStrategy,
    RegimeSentimentStrategy, RsiStrategy, SentimentMomentumStrategy,
};
use chrono::{Duration, TimeZone, Utc};
use std::env;
use std::path::PathBuf;

/// Helper function to generate test bars for a symbol
fn generate_test_bars(symbol: &str, count: usize, trend: &str) -> Vec<Bar> {
    let mut bars = Vec::new();
    let base_price = if symbol == "BTC" { 50000.0 } else { 3000.0 };
    let base_date = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();

    for i in 0..count {
        let price = match trend {
            "uptrend" => base_price + (i as f64) * 100.0,
            "downtrend" => base_price - (i as f64) * 50.0,
            "sideways" => base_price + (i as f64).sin() * 500.0,
            "volatile" => base_price + (i as f64).sin() * 2000.0,
            _ => base_price,
        };

        let timestamp = base_date + Duration::hours(i as i64);

        bars.push(Bar {
            timestamp,
            open: price * 0.999,
            high: price * 1.001,
            low: price * 0.998,
            close: price,
            volume: 1000.0 + (i as f64) * 10.0,
        });
    }

    bars
}

/// Helper function to setup test database with sample data
async fn setup_test_database() -> Result<DatabaseClient, Box<dyn std::error::Error>> {
    // Use test database URL if available, otherwise skip database tests
    let database_url = env::var("TEST_DATABASE_URL")
        .or_else(|_| env::var("DATABASE_URL"))
        .unwrap_or_else(|_| {
            "postgresql://postgres:password@localhost:5432/alphafield_test".to_string()
        });

    env::set_var("DATABASE_URL", &database_url);

    let db = DatabaseClient::new_from_env().await?;

    // Clear existing test data
    let pool = &db.pool;

    // Delete test data if exists
    let _ = sqlx::query("DELETE FROM candles WHERE symbol IN ('BTC', 'ETH', 'SOL')")
        .execute(pool)
        .await;

    // Insert test data for BTC (uptrend)
    let btc_bars = generate_test_bars("BTC", 500, "uptrend");
    db.save_bars("BTC", "1h", &btc_bars).await?;

    // Insert test data for ETH (downtrend)
    let eth_bars = generate_test_bars("ETH", 500, "downtrend");
    db.save_bars("ETH", "1h", &eth_bars).await?;

    // Insert test data for SOL (volatile/sideways)
    let sol_bars = generate_test_bars("SOL", 500, "volatile");
    db.save_bars("SOL", "1h", &sol_bars).await?;

    Ok(db)
}

/// Helper function to create a backtest strategy wrapper
fn create_backtest_strategy(name: &str, symbol: &str, capital: f64) -> Box<dyn Strategy> {
    match name {
        "golden_cross" => Box::new(StrategyAdapter::new(GoldenCrossStrategy::default(), symbol)),
        "rsi" => Box::new(StrategyAdapter::new(RsiStrategy::default(), symbol)),
        "macd_trend" => Box::new(StrategyAdapter::new(MacdTrendStrategy::default(), symbol)),
        "bollinger_bands" => Box::new(StrategyAdapter::new(
            BollingerBandsStrategy::default(),
            symbol,
        )),
        "regime_sentiment" => Box::new(StrategyAdapter::new(
            RegimeSentimentStrategy::default(),
            symbol,
        )),
        "divergence" => Box::new(StrategyAdapter::new(DivergenceStrategy::default(), symbol)),
        "sentiment_momentum" => Box::new(StrategyAdapter::new(
            SentimentMomentumStrategy::default(),
            symbol,
        )),
        _ => panic!("Unknown strategy: {}", name),
    }
}

#[tokio::test]
#[ignore] // Requires database, run with: cargo test --package alphafield-backtest --test validation_pipeline -- --ignored
async fn test_single_strategy_validation_from_database() {
    let db = setup_test_database()
        .await
        .expect("Failed to setup test database");

    // Load bars from database
    let bars = db
        .load_bars("BTC", "1h")
        .await
        .expect("Failed to load bars from database");

    assert!(!bars.is_empty(), "Should have loaded bars from database");

    // Validate golden cross strategy
    let strategy = create_backtest_strategy("golden_cross", "BTC", 10000.0);

    let config = ValidationConfig {
        data_source: "database".to_string(),
        symbol: "BTC".to_string(),
        interval: "1h".to_string(),
        walk_forward: WalkForwardConfig::default(),
        risk_free_rate: 0.02,
        thresholds: ValidationThresholds::default(),
        initial_capital: 10000.0,
        fee_rate: 0.001,
    };

    let validator = StrategyValidator::new(config);
    let report = validator
        .validate(strategy, "BTC", &bars)
        .expect("Validation should succeed");

    // Verify report structure
    assert!(
        !report.summary.trades.is_empty(),
        "Should have generated trades"
    );
    assert!(
        report.overall_score >= 0.0,
        "Overall score should be non-negative"
    );
    assert!(
        report.initial_capital == 10000.0,
        "Initial capital should match"
    );

    // Verify verdict is set
    assert!(
        !matches!(report.verdict, ValidationVerdict::Pass),
        "With short test data, may not pass without optimization"
    );
}

#[tokio::test]
#[ignore]
async fn test_batch_validation_from_database() {
    let db = setup_test_database()
        .await
        .expect("Failed to setup test database");

    let strategies = vec!["golden_cross", "rsi", "macd_trend"];
    let symbols = vec!["BTC", "ETH", "SOL"];

    let mut validation_count = 0;
    let mut successful_count = 0;

    for strategy_name in &strategies {
        for symbol in &symbols {
            validation_count += 1;

            // Load bars from database
            let bars = db
                .load_bars(symbol, "1h")
                .await
                .expect(&format!("Failed to load bars for {}", symbol));

            // Create strategy
            let strategy = create_backtest_strategy(strategy_name, symbol, 10000.0);

            // Validate
            let config = ValidationConfig {
                data_source: "database".to_string(),
                symbol: symbol.to_string(),
                interval: "1h".to_string(),
                walk_forward: WalkForwardConfig::default(),
                risk_free_rate: 0.02,
                thresholds: ValidationThresholds::default(),
                initial_capital: 10000.0,
                fee_rate: 0.001,
            };

            let validator = StrategyValidator::new(config);
            let result = validator.validate(strategy, symbol, &bars);

            match result {
                Ok(report) => {
                    successful_count += 1;
                    assert!(
                        !report.summary.trades.is_empty(),
                        "Should have trades for {}/{}",
                        strategy_name,
                        symbol
                    );
                }
                Err(e) => {
                    eprintln!("Validation failed for {}/{}: {}", strategy_name, symbol, e);
                }
            }
        }
    }

    assert_eq!(
        validation_count, 9,
        "Should validate all strategy/symbol combinations"
    );
    assert!(
        successful_count > 0,
        "At least some validations should succeed"
    );
}

#[tokio::test]
#[ignore]
async fn test_validation_with_walk_forward() {
    let db = setup_test_database()
        .await
        .expect("Failed to setup test database");

    let bars = db
        .load_bars("BTC", "1h")
        .await
        .expect("Failed to load bars from database");

    let strategy = create_backtest_strategy("rsi", "BTC", 10000.0);

    let config = ValidationConfig {
        data_source: "database".to_string(),
        symbol: "BTC".to_string(),
        interval: "1h".to_string(),
        walk_forward: WalkForwardConfig {
            enabled: true,
            in_sample_pct: 0.7,
            num_windows: 3,
            min_window_size: 200,
            ..Default::default()
        },
        risk_free_rate: 0.02,
        thresholds: ValidationThresholds::default(),
        initial_capital: 10000.0,
        fee_rate: 0.001,
    };

    let validator = StrategyValidator::new(config);
    let report = validator
        .validate(strategy, "BTC", &bars)
        .expect("Walk-forward validation should succeed");

    // Verify walk-forward results
    assert!(
        report.walk_forward.in_sample_trades > 0,
        "Should have in-sample trades"
    );
    assert!(
        report.walk_forward.out_of_sample_trades > 0,
        "Should have out-of-sample trades"
    );
    assert!(
        report.walk_forward.stability_score >= 0.0,
        "Stability score should be valid"
    );
    assert!(
        report.walk_forward.stability_score <= 1.0,
        "Stability score should be <= 1.0"
    );
}

#[tokio::test]
#[ignore]
async fn test_validation_with_monte_carlo() {
    let db = setup_test_database()
        .await
        .expect("Failed to setup test database");

    let bars = db
        .load_bars("BTC", "1h")
        .await
        .expect("Failed to load bars from database");

    let strategy = create_backtest_strategy("golden_cross", "BTC", 10000.0);

    let config = ValidationConfig {
        data_source: "database".to_string(),
        symbol: "BTC".to_string(),
        interval: "1h".to_string(),
        walk_forward: WalkForwardConfig::default(),
        risk_free_rate: 0.02,
        thresholds: ValidationThresholds::default(),
        initial_capital: 10000.0,
        fee_rate: 0.001,
    };

    let validator = StrategyValidator::new(config);
    let report = validator
        .validate(strategy, "BTC", &bars)
        .expect("Monte Carlo validation should succeed");

    // Verify Monte Carlo results
    assert!(
        report.monte_carlo.num_simulations > 0,
        "Should run Monte Carlo simulations"
    );
    assert!(
        report.monte_carlo.mean_final_capital > 0.0,
        "Mean final capital should be positive"
    );
    assert!(
        report.monte_carlo.positive_probability >= 0.0,
        "Positive probability should be valid"
    );
    assert!(
        report.monte_carlo.positive_probability <= 1.0,
        "Positive probability should be <= 1.0"
    );
}

#[tokio::test]
#[ignore]
async fn test_validation_with_regime_analysis() {
    let db = setup_test_database()
        .await
        .expect("Failed to setup test database");

    // Test with uptrend data (BTC)
    let bars = db
        .load_bars("BTC", "1h")
        .await
        .expect("Failed to load bars from database");

    let strategy = create_backtest_strategy("macd_trend", "BTC", 10000.0);

    let config = ValidationConfig {
        data_source: "database".to_string(),
        symbol: "BTC".to_string(),
        interval: "1h".to_string(),
        walk_forward: WalkForwardConfig::default(),
        risk_free_rate: 0.02,
        thresholds: ValidationThresholds::default(),
        initial_capital: 10000.0,
        fee_rate: 0.001,
    };

    let validator = StrategyValidator::new(config);
    let report = validator
        .validate(strategy, "BTC", &bars)
        .expect("Regime analysis validation should succeed");

    // Verify regime analysis results
    assert!(
        !report.regime_analysis.regime_performance.is_empty(),
        "Should have regime performance data"
    );

    // Should have identified at least one regime (uptrend)
    assert!(
        report.regime_analysis.regime_performance.len() >= 1,
        "Should identify regimes from the data"
    );

    // Regime match score should be valid
    let regime_match = report.regime_analysis.calculate_regime_match_score();
    assert!(regime_match >= 0.0, "Regime match score should be valid");
    assert!(regime_match <= 1.0, "Regime match score should be <= 1.0");
}

#[tokio::test]
#[ignore]
async fn test_validation_report_serialization() {
    let db = setup_test_database()
        .await
        .expect("Failed to setup test database");

    let bars = db
        .load_bars("ETH", "1h")
        .await
        .expect("Failed to load bars from database");

    let strategy = create_backtest_strategy("rsi", "ETH", 10000.0);

    let config = ValidationConfig {
        data_source: "database".to_string(),
        symbol: "ETH".to_string(),
        interval: "1h".to_string(),
        walk_forward: WalkForwardConfig::default(),
        risk_free_rate: 0.02,
        thresholds: ValidationThresholds::default(),
        initial_capital: 10000.0,
        fee_rate: 0.001,
    };

    let validator = StrategyValidator::new(config);
    let report = validator
        .validate(strategy, "ETH", &bars)
        .expect("Validation should succeed");

    // Test JSON serialization
    let json = serde_json::to_string_pretty(&report).expect("Report should serialize to JSON");
    assert!(json.contains("verdict"), "JSON should contain verdict");
    assert!(
        json.contains("overall_score"),
        "JSON should contain overall_score"
    );

    // Test YAML serialization
    let yaml = serde_yaml::to_string(&report).expect("Report should serialize to YAML");
    assert!(yaml.contains("verdict:"), "YAML should contain verdict");

    // Test that we can deserialize back
    let deserialized: ValidationReport =
        serde_json::from_str(&json).expect("Should be able to deserialize JSON");

    assert_eq!(
        deserialized.overall_score, report.overall_score,
        "Deserialized report should match original"
    );
    assert_eq!(
        deserialized.verdict, report.verdict,
        "Deserialized verdict should match original"
    );
}

#[tokio::test]
#[ignore]
async fn test_validation_with_custom_thresholds() {
    let db = setup_test_database()
        .await
        .expect("Failed to setup test database");

    let bars = db
        .load_bars("SOL", "1h")
        .await
        .expect("Failed to load bars from database");

    let strategy = create_backtest_strategy("bollinger_bands", "SOL", 10000.0);

    // Set custom thresholds
    let custom_thresholds = ValidationThresholds {
        min_sharpe_ratio: 0.5,          // Lower than default (1.0)
        max_drawdown: 0.40,             // Higher than default (0.30)
        min_win_rate: 0.55,             // Lower than default (0.60)
        min_profit_factor: 1.2,         // Lower than default (1.5)
        max_consecutive_losses: 10,     // Higher than default (5)
        min_positive_probability: 0.60, // Lower than default (0.70)
    };

    let config = ValidationConfig {
        data_source: "database".to_string(),
        symbol: "SOL".to_string(),
        interval: "1h".to_string(),
        walk_forward: WalkForwardConfig::default(),
        risk_free_rate: 0.02,
        thresholds: custom_thresholds,
        initial_capital: 10000.0,
        fee_rate: 0.001,
    };

    let validator = StrategyValidator::new(config);
    let report = validator
        .validate(strategy, "SOL", &bars)
        .expect("Validation with custom thresholds should succeed");

    // Verify thresholds were applied (though exact verdict depends on performance)
    assert!(report.overall_score >= 0.0, "Overall score should be valid");
}

#[tokio::test]
#[ignore]
async fn test_multiple_timeframes_from_database() {
    let db = setup_test_database()
        .await
        .expect("Failed to setup test database");

    // Insert data for multiple timeframes
    let bars_1h = generate_test_bars("BTC", 500, "uptrend");
    let bars_4h: Vec<Bar> = bars_1h
        .chunks(4)
        .map(|chunk| {
            let first = chunk.first().unwrap();
            Bar {
                timestamp: first.timestamp,
                open: first.open,
                high: chunk.iter().map(|b| b.high).fold(f64::NAN, f64::max),
                low: chunk.iter().map(|b| b.low).fold(f64::NAN, f64::min),
                close: chunk.last().unwrap().close,
                volume: chunk.iter().map(|b| b.volume).sum(),
            }
        })
        .collect();

    // Save 4h data
    db.save_bars("BTC", "4h", &bars_4h)
        .await
        .expect("Failed to save 4h bars");

    // Load and validate both timeframes
    for interval in ["1h", "4h"] {
        let bars = db
            .load_bars("BTC", interval)
            .await
            .expect(&format!("Failed to load {} bars", interval));

        let strategy = create_backtest_strategy("golden_cross", "BTC", 10000.0);

        let config = ValidationConfig {
            data_source: "database".to_string(),
            symbol: "BTC".to_string(),
            interval: interval.to_string(),
            walk_forward: WalkForwardConfig::default(),
            risk_free_rate: 0.02,
            thresholds: ValidationThresholds::default(),
            initial_capital: 10000.0,
            fee_rate: 0.001,
        };

        let validator = StrategyValidator::new(config);
        let report = validator.validate(strategy, "BTC", &bars).expect(&format!(
            "Validation should succeed for {} timeframe",
            interval
        ));

        assert!(
            !report.summary.trades.is_empty(),
            "Should have trades for {} timeframe",
            interval
        );
    }
}

#[tokio::test]
#[ignore]
async fn test_validation_with_large_dataset() {
    let db = setup_test_database()
        .await
        .expect("Failed to setup test database");

    // Load a larger dataset
    let bars = db
        .load_bars("BTC", "1h")
        .await
        .expect("Failed to load bars from database");

    assert!(bars.len() >= 500, "Should have sufficient test data");

    let strategy = create_backtest_strategy("sentiment_momentum", "BTC", 10000.0);

    let config = ValidationConfig {
        data_source: "database".to_string(),
        symbol: "BTC".to_string(),
        interval: "1h".to_string(),
        walk_forward: WalkForwardConfig::default(),
        risk_free_rate: 0.02,
        thresholds: ValidationThresholds::default(),
        initial_capital: 10000.0,
        fee_rate: 0.001,
    };

    let validator = StrategyValidator::new(config);
    let report = validator
        .validate(strategy, "BTC", &bars)
        .expect("Validation with large dataset should succeed");

    // With larger dataset, should have meaningful statistics
    assert!(report.summary.total_bars >= 500, "Should process all bars");
    assert!(
        report.summary.trade_count >= 10,
        "Should generate trades from large dataset"
    );
}

#[tokio::test]
#[ignore]
async fn test_database_connection_error_handling() {
    // Test with invalid database URL
    env::set_var(
        "DATABASE_URL",
        "postgresql://invalid:invalid@localhost:9999/invalid",
    );

    let result = DatabaseClient::new_from_env().await;

    assert!(result.is_err(), "Should fail with invalid database URL");
}
