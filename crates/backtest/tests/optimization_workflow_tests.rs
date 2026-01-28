//! Integration test for optimization workflow
//!
//! This test verifies that the complete optimization workflow runs successfully
//! with a simple strategy and mock data.

use alphafield_backtest::{OptimizationWorkflow, ParameterRange, StrategyAdapter, WorkflowConfig};
use alphafield_core::{Bar, TradingMode};
use alphafield_strategy::strategies::GoldenCrossStrategy;
use chrono::{Duration, Utc};
use std::collections::HashMap;

/// Generate mock historical data for testing
fn generate_mock_data(bars: usize) -> Vec<Bar> {
    let mut data = Vec::new();
    let start_time = Utc::now() - Duration::days(bars as i64);
    let mut price = 100.0;

    for i in 0..bars {
        // Create volatile price movements with clear trend reversals for Golden Cross signals
        // Simulate bull/bear market cycles every 50-100 bars
        let cycle_length = 80.0;
        let cycle_position = (i as f64 % cycle_length) / cycle_length;
        let trend_phase = cycle_position * 2.0 * std::f64::consts::PI;

        // Create strong up and down trends with reversals
        let trend_strength = 5.0;
        let trend_move = trend_phase.sin() * trend_strength;

        // Add significant random volatility
        let random_move = ((i * 13) % 40) as f64 - 20.0; // Random +/- 6.0

        // Combine components
        price += trend_move + random_move;
        price = price.max(20.0); // Minimum price floor

        // Create realistic OHLC with good volatility for MA crossovers
        let open = price;
        let volatility = 0.025; // 2.5% daily volatility for more signals
        let high = price * (1.0 + volatility * 1.5);
        let low = price * (1.0 - volatility * 1.5);
        let close = price + (open * 0.005 * ((i % 7) as f64 - 3.0)); // Close varies relative to open

        data.push(Bar {
            timestamp: start_time + Duration::days(i as i64),
            open,
            high,
            low,
            close,
            volume: 1500.0 + ((i % 10) * 300) as f64,
        });

        price = close;
    }

    data
}

#[test]
fn test_optimization_workflow_basic() {
    // Generate 500 days of mock data
    let data = generate_mock_data(500);
    assert!(!data.is_empty());

    // Create workflow with minimal config for faster testing
    let config = WorkflowConfig {
        initial_capital: 100_000.0,
        fee_rate: 0.001,
        slippage: alphafield_backtest::SlippageModel::FixedPercent(0.0005),
        walk_forward_config: alphafield_backtest::WalkForwardConfig {
            train_window: 100,
            test_window: 50,
            step_size: 30,
            initial_capital: 100_000.0,
            fee_rate: 0.001,
        },
        include_3d_sensitivity: false, // Disable for faster testing
        train_test_split_ratio: 0.70,
        monte_carlo_config: None, // Monte Carlo disabled for testing
        risk_free_rate: 0.02,
        trading_mode: TradingMode::Spot,
    };

    let workflow = OptimizationWorkflow::new(config);

    // Use simple parameter bounds for GoldenCross
    let bounds = vec![
        alphafield_backtest::ParamBounds::new("fast_period", 5.0, 15.0, 5.0),
        alphafield_backtest::ParamBounds::new("slow_period", 20.0, 40.0, 10.0),
    ];

    // Factory that creates GoldenCross strategies wrapped in adapter
    let symbol = "TEST";
    let factory = |params: &HashMap<String, f64>| {
        let fast_period = params.get("fast_period").copied().unwrap_or(10.0) as usize;
        let slow_period = params.get("slow_period").copied().unwrap_or(30.0) as usize;

        if fast_period >= slow_period {
            return None; // Invalid parameters
        }

        let strategy = GoldenCrossStrategy::new(fast_period, slow_period);
        let adapter = StrategyAdapter::new(strategy, symbol, 100_000.0);

        Some(Box::new(adapter) as Box<dyn alphafield_backtest::strategy::Strategy>)
    };

    // Run workflow
    let result = workflow.run(&data, symbol, &factory, &bounds, None);

    // Verify workflow completed successfully
    assert!(
        result.is_ok(),
        "Workflow should complete successfully: {:?}",
        result.err()
    );

    let workflow_result = result.unwrap();

    // Verify optimization was attempted (may not find good parameters with mock data)
    assert!(
        workflow_result.optimization.iterations_tested > 0,
        "Should test at least one iteration"
    );

    // Verify parameter dispersion was calculated
    assert!(
        workflow_result.parameter_dispersion.sharpe_std >= 0.0,
        "Sharpe std should be non-negative"
    );

    // Verify walk-forward validation ran (may have zero windows with insufficient data)
    assert!(
        workflow_result.walk_forward_validation.stability_score >= 0.0,
        "Stability score should be non-negative"
    );
    assert!(
        workflow_result.walk_forward_validation.stability_score <= 1.0,
        "Stability score should be <= 1.0"
    );

    // Verify robustness score is in valid range
    assert!(
        workflow_result.robustness_score >= 0.0,
        "Robustness score should be >= 0"
    );
    assert!(
        workflow_result.robustness_score <= 100.0,
        "Robustness score should be <= 100"
    );

    println!("✓ Workflow completed successfully");
    println!(
        "  - Iterations tested: {}",
        workflow_result.optimization.iterations_tested
    );
    println!(
        "  - Best Sharpe: {:.2}",
        workflow_result.optimization.best_sharpe
    );
    println!(
        "  - Robustness score: {:.2}",
        workflow_result.robustness_score
    );
    println!(
        "  - Walk-forward stability: {:.2}",
        workflow_result.walk_forward_validation.stability_score
    );
    println!(
        "  - Parameter dispersion CV: {:.2}",
        workflow_result.parameter_dispersion.sharpe_cv
    );
}

#[test]
fn test_optimization_workflow_trading_mode_margin() {
    // Generate 300 days of mock data
    let data = generate_mock_data(300);
    assert!(!data.is_empty());

    // Create workflow with Margin trading mode
    use alphafield_core::TradingMode;

    let config = WorkflowConfig {
        initial_capital: 100_000.0,
        fee_rate: 0.001,
        slippage: alphafield_backtest::SlippageModel::FixedPercent(0.0005),
        walk_forward_config: alphafield_backtest::WalkForwardConfig {
            train_window: 80,
            test_window: 40,
            step_size: 30,
            initial_capital: 100_000.0,
            fee_rate: 0.001,
        },
        include_3d_sensitivity: false, // Disable for faster testing
        train_test_split_ratio: 0.70,
        monte_carlo_config: None,
        risk_free_rate: 0.02,
        trading_mode: TradingMode::Margin, // Test Margin mode
    };

    let workflow = OptimizationWorkflow::new(config);

    // Use simple parameter bounds for GoldenCross
    let bounds = vec![
        alphafield_backtest::ParamBounds::new("fast_period", 5.0, 15.0, 5.0),
        alphafield_backtest::ParamBounds::new("slow_period", 20.0, 40.0, 10.0),
    ];

    // Factory that creates GoldenCross strategies wrapped in adapter
    let symbol = "TEST";
    let factory = |params: &HashMap<String, f64>| {
        let fast_period = params.get("fast_period").copied().unwrap_or(10.0) as usize;
        let slow_period = params.get("slow_period").copied().unwrap_or(30.0) as usize;

        if fast_period >= slow_period {
            return None;
        }

        let strategy = GoldenCrossStrategy::new(fast_period, slow_period);
        let adapter = StrategyAdapter::new(strategy, symbol, 100_000.0);

        Some(Box::new(adapter) as Box<dyn alphafield_backtest::strategy::Strategy>)
    };

    // Run workflow
    let result = workflow.run(&data, symbol, &factory, &bounds, None);

    // Verify workflow completed successfully
    assert!(
        result.is_ok(),
        "Workflow with Margin mode should complete successfully: {:?}",
        result.err()
    );

    let workflow_result = result.unwrap();

    // Verify optimization was attempted
    assert!(
        workflow_result.optimization.iterations_tested > 0,
        "Should test at least one iteration"
    );

    // Verify all metrics are in valid ranges
    assert!(
        workflow_result.robustness_score >= 0.0 && workflow_result.robustness_score <= 100.0,
        "Robustness score should be in valid range"
    );

    assert!(
        workflow_result.walk_forward_validation.stability_score >= 0.0
            && workflow_result.walk_forward_validation.stability_score <= 1.0,
        "Stability score should be in valid range"
    );

    println!("✓ Margin mode workflow completed successfully");
    println!("  - Trading mode: Margin",);
    println!(
        "  - Iterations tested: {}",
        workflow_result.optimization.iterations_tested
    );
    println!(
        "  - Robustness score: {:.2}",
        workflow_result.robustness_score
    );
}

#[test]
fn test_optimization_workflow_with_sensitivity() {
    // Generate 300 days of mock data (less data for faster 3D sensitivity)
    let data = generate_mock_data(300);

    // Create workflow with 3D sensitivity enabled
    let config = WorkflowConfig {
        initial_capital: 100_000.0,
        fee_rate: 0.001,
        slippage: alphafield_backtest::SlippageModel::FixedPercent(0.0005),
        walk_forward_config: alphafield_backtest::WalkForwardConfig {
            train_window: 80,
            test_window: 40,
            step_size: 30,
            initial_capital: 100_000.0,
            fee_rate: 0.001,
        },
        include_3d_sensitivity: true,
        train_test_split_ratio: 0.70,
        monte_carlo_config: None, // Monte Carlo disabled for testing
        risk_free_rate: 0.02,
        trading_mode: TradingMode::Spot,
    };

    let workflow = OptimizationWorkflow::new(config);

    // Use smaller parameter bounds for faster 3D analysis
    let bounds = vec![
        alphafield_backtest::ParamBounds::new("fast_period", 5.0, 10.0, 5.0),
        alphafield_backtest::ParamBounds::new("slow_period", 20.0, 30.0, 10.0),
    ];

    // Create sensitivity parameters
    let param_x = ParameterRange::new("fast_period", 5.0, 10.0, 5.0);
    let param_y = ParameterRange::new("slow_period", 20.0, 30.0, 10.0);

    // Factory that creates GoldenCross strategies wrapped in adapter
    let symbol = "TEST";
    let factory = |params: &HashMap<String, f64>| {
        let fast_period = params.get("fast_period").copied().unwrap_or(10.0) as usize;
        let slow_period = params.get("slow_period").copied().unwrap_or(30.0) as usize;

        if fast_period >= slow_period {
            return None;
        }

        let strategy = GoldenCrossStrategy::new(fast_period, slow_period);
        let adapter = StrategyAdapter::new(strategy, symbol, 100_000.0);

        Some(Box::new(adapter) as Box<dyn alphafield_backtest::strategy::Strategy>)
    };

    // Run workflow with 3D sensitivity
    let result = workflow.run(&data, symbol, &factory, &bounds, Some((param_x, param_y)));

    assert!(
        result.is_ok(),
        "Workflow with 3D sensitivity should succeed"
    );

    let workflow_result = result.unwrap();

    // Verify 3D sensitivity was calculated
    assert!(
        workflow_result.sensitivity_3d.is_some(),
        "3D sensitivity should be present"
    );

    let sensitivity = workflow_result.sensitivity_3d.unwrap();
    assert!(sensitivity.heatmap.is_some(), "Heatmap should be generated");

    let heatmap = sensitivity.heatmap.unwrap();
    assert_eq!(
        heatmap.x_param, "fast_period",
        "X parameter should be fast_period"
    );
    assert_eq!(
        heatmap.y_param, "slow_period",
        "Y parameter should be slow_period"
    );
    assert!(
        !heatmap.sharpe_matrix.is_empty(),
        "Sharpe matrix should not be empty"
    );

    println!("✓ Workflow with 3D sensitivity completed successfully");
    println!(
        "  - Heatmap dimensions: {}x{}",
        heatmap.x_values.len(),
        heatmap.y_values.len()
    );
}

#[test]
fn test_workflow_config_defaults() {
    let config = WorkflowConfig::default();

    assert_eq!(config.initial_capital, 100_000.0);
    assert_eq!(config.fee_rate, 0.001);
    assert!(config.include_3d_sensitivity);
    assert_eq!(config.walk_forward_config.train_window, 252);
    assert_eq!(config.walk_forward_config.test_window, 63);
}

#[test]
fn test_optimization_workflow_with_monte_carlo() {
    // Generate 400 days of mock data
    let data = generate_mock_data(400);

    // Create workflow with Monte Carlo enabled
    let config = WorkflowConfig {
        initial_capital: 100_000.0,
        fee_rate: 0.001,
        slippage: alphafield_backtest::SlippageModel::FixedPercent(0.0005),
        walk_forward_config: alphafield_backtest::WalkForwardConfig {
            train_window: 100,
            test_window: 50,
            step_size: 30,
            initial_capital: 100_000.0,
            fee_rate: 0.001,
        },
        include_3d_sensitivity: false, // Disable for faster testing
        train_test_split_ratio: 0.70,
        monte_carlo_config: Some(alphafield_backtest::MonteCarloConfig {
            num_simulations: 100, // Low number for faster testing
            initial_capital: 100_000.0,
            seed: Some(42), // Reproducible results
        }),
        risk_free_rate: 0.02,
        trading_mode: TradingMode::Spot,
    };

    let workflow = OptimizationWorkflow::new(config);

    // Use simple parameter bounds for GoldenCross
    let bounds = vec![
        alphafield_backtest::ParamBounds::new("fast_period", 5.0, 15.0, 5.0),
        alphafield_backtest::ParamBounds::new("slow_period", 20.0, 40.0, 10.0),
    ];

    // Factory that creates GoldenCross strategies wrapped in adapter
    let symbol = "TEST";
    let factory = |params: &HashMap<String, f64>| {
        let fast_period = params.get("fast_period").copied().unwrap_or(10.0) as usize;
        let slow_period = params.get("slow_period").copied().unwrap_or(30.0) as usize;

        if fast_period >= slow_period {
            return None;
        }

        let strategy = GoldenCrossStrategy::new(fast_period, slow_period);
        let adapter = StrategyAdapter::new(strategy, symbol, 100_000.0);

        Some(Box::new(adapter) as Box<dyn alphafield_backtest::strategy::Strategy>)
    };

    // Run workflow with Monte Carlo
    let result = workflow.run(&data, symbol, &factory, &bounds, None);

    assert!(
        result.is_ok(),
        "Workflow with Monte Carlo should succeed: {:?}",
        result.err()
    );

    let workflow_result = result.unwrap();

    // Debug: Check if trades were generated
    println!(
        "Optimization found {} trades in backtest",
        workflow_result.in_sample_metrics.total_trades
    );
    println!(
        "In-sample total return: {:.2}%",
        workflow_result.in_sample_metrics.total_return * 100.0
    );

    // Monte Carlo runs if trades were generated
    if workflow_result.in_sample_metrics.total_trades > 0 {
        // Verify Monte Carlo results are present when trades exist
        assert!(
            workflow_result.monte_carlo.is_some(),
            "Monte Carlo results should be present when trades exist (trades count: {})",
            workflow_result.in_sample_metrics.total_trades
        );

        let mc_result = workflow_result.monte_carlo.unwrap();

        // Verify Monte Carlo simulation ran
        assert_eq!(
            mc_result.num_simulations, 100,
            "Should run 100 simulations as configured"
        );

        // Verify confidence intervals are calculated
        assert!(
            mc_result.equity_5th > 0.0,
            "5th percentile equity should be positive"
        );
        assert!(
            mc_result.equity_95th > 0.0,
            "95th percentile equity should be positive"
        );
        assert!(
            mc_result.equity_95th >= mc_result.equity_5th,
            "95th percentile should be >= 5th percentile"
        );

        // Verify return percentiles
        assert!(
            mc_result.return_95th >= mc_result.return_5th,
            "95th return percentile should be >= 5th percentile"
        );

        // Verify drawdown percentiles (all should be non-negative)
        assert!(
            mc_result.drawdown_5th >= 0.0,
            "Drawdown percentiles should be non-negative"
        );
        assert!(
            mc_result.drawdown_95th >= mc_result.drawdown_5th,
            "95th drawdown percentile should be >= 5th percentile"
        );

        // Verify probability of profit is in valid range
        assert!(
            mc_result.probability_of_profit >= 0.0 && mc_result.probability_of_profit <= 1.0,
            "Probability of profit should be between 0 and 1"
        );

        // Verify simulations vector has correct length
        assert_eq!(
            mc_result.simulations.len(),
            100,
            "Should have 100 simulation results"
        );

        // Verify original metrics are calculated
        assert!(
            mc_result.original_metrics.total_return.is_finite(),
            "Original return should be finite"
        );
        assert!(
            mc_result.original_metrics.max_drawdown >= 0.0,
            "Original max drawdown should be non-negative"
        );

        println!("✓ Monte Carlo simulation validated successfully");
        println!("✓ Workflow with Monte Carlo completed successfully");
        println!("  - Simulations run: {}", mc_result.num_simulations);
        println!(
            "  - Probability of profit: {:.1}%",
            mc_result.probability_of_profit * 100.0
        );
        println!(
            "  - Equity 5th/50th/95th: {:.2} / {:.2} / {:.2}",
            mc_result.equity_5th, mc_result.equity_50th, mc_result.equity_95th
        );
        println!(
            "  - Drawdown 95th percentile: {:.1}%",
            mc_result.drawdown_95th * 100.0
        );
        println!(
            "  - Original return: {:.1}%",
            mc_result.original_metrics.total_return * 100.0
        );
    } else {
        // No trades generated - verify Monte Carlo is None and workflow still completed
        println!("⚠ No trades generated in backtest - Monte Carlo skipped (expected behavior)");
        assert!(
            workflow_result.monte_carlo.is_none(),
            "Monte Carlo should be None when no trades exist"
        );
    }

    // Verify robustness score incorporates Monte Carlo
    assert!(
        workflow_result.robustness_score >= 0.0,
        "Robustness score should be >= 0"
    );
    assert!(
        workflow_result.robustness_score <= 100.0,
        "Robustness score should be <= 100"
    );

    println!(
        "  - Robustness score: {:.2}",
        workflow_result.robustness_score
    );
}
