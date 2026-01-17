//! Integration test for optimization workflow
//!
//! This test verifies that the complete optimization workflow runs successfully
//! with a simple strategy and mock data.

use alphafield_backtest::{OptimizationWorkflow, ParameterRange, StrategyAdapter, WorkflowConfig};
use alphafield_core::Bar;
use alphafield_strategy::strategies::GoldenCrossStrategy;
use chrono::{Duration, Utc};
use std::collections::HashMap;

/// Generate mock historical data for testing
fn generate_mock_data(bars: usize) -> Vec<Bar> {
    let mut data = Vec::new();
    let start_time = Utc::now() - Duration::days(bars as i64);
    let mut price = 100.0;

    for i in 0..bars {
        // Simulate some price movement (random walk with trend)
        price += (i as f64 * 0.01) - 0.5 + ((i % 10) as f64 - 5.0) * 0.1;

        data.push(Bar {
            timestamp: start_time + Duration::days(i as i64),
            open: price,
            high: price * 1.01,
            low: price * 0.99,
            close: price,
            volume: 1000.0,
        });
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
