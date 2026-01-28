//! # Strategy Validator
//!
//! Main orchestrator that runs all validation tests and generates
//! comprehensive validation reports.

#[allow(unused_imports)]
use crate::adapter::StrategyAdapter;
use crate::engine::BacktestEngine;
use crate::exchange::SlippageModel;
use crate::metrics::PerformanceMetrics;
use crate::monte_carlo::{MonteCarloConfig, MonteCarloSimulator, Trade as McTrade};
#[allow(unused_imports)]
use crate::portfolio::Portfolio;
use crate::strategy::Strategy;
#[allow(unused_imports)]
use crate::trade::Trade;
use crate::validation::scoring::RecommendationsGenerator;
use crate::validation::scoring::ScoreCalculator;
use crate::validation::{
    BacktestResult, RegimeAnalysisResult, RiskAssessment, RiskRating, TestPeriod,
    ValidationComponents, ValidationConfig, ValidationReport, ValidationVerdict,
};

use alphafield_core::{Bar, QuantError as CoreError};
use chrono::Utc;

/// Main strategy validator orchestrator
pub struct StrategyValidator {
    config: ValidationConfig,
}

impl StrategyValidator {
    /// Create new validator with configuration
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Validate a strategy against historical data
    ///
    /// This method provides comprehensive validation including:
    /// - Backtest performance analysis (30% of score)
    /// - Walk-forward robustness testing (25% of score) - **SIMPLIFIED version**
    /// - Monte Carlo simulation (20% of score)
    /// - Regime analysis (15% of score)
    /// - Risk assessment (10% of score)
    ///
    /// **IMPORTANT**: This method uses a SIMPLIFIED walk-forward implementation
    /// (buy-and-hold returns) because it receives a Box<dyn Strategy> without
    /// access to a factory function. For enhanced walk-forward with actual strategy
    /// execution, use `validate_with_factory()` instead.
    ///
    /// # Arguments
    /// * `strategy` - Box<dyn BacktestStrategy> to validate (ownership transferred)
    /// * `symbol` - Symbol name for data registration
    /// * `bars` - Historical bar data (must be sorted by time)
    ///
    /// # Returns
    /// Comprehensive validation report with all test results
    ///
    /// # When to Use
    /// - When you have a Box<dyn Strategy> instance
    /// - When you don't have access to strategy creation logic
    /// - When simplified walk-forward is acceptable
    ///
    /// # See Also
    /// - `validate_with_factory()` - For factory functions (enhanced walk-forward)
    pub fn validate(
        &self,
        strategy: Box<dyn Strategy>,
        symbol: &str,
        bars: &[Bar],
    ) -> Result<ValidationReport, CoreError> {
        if bars.is_empty() {
            return Err(CoreError::DataValidation(
                "No historical data provided for validation".to_string(),
            ));
        }

        // Create test period info
        let test_period = self.create_test_period(symbol, bars);

        // Run backtest (strategy ownership moved into run_backtest)
        let backtest_result = self.run_backtest(strategy, symbol, bars)?;

        // Run walk-forward analysis (no strategy reference here)
        let walk_forward_result = self.run_walk_forward(bars)?;

        // Run Monte Carlo simulation
        let monte_carlo_result = self.run_monte_carlo(&backtest_result)?;

        // Run regime analysis (no strategy reference required for regime analysis)
        let regime_result = self.run_regime_analysis(bars)?;

        // Calculate risk assessment
        let risk_assessment = self.assess_risk(&backtest_result, &monte_carlo_result);

        // Calculate overall score and generate verdict
        let components = ValidationComponents {
            backtest: backtest_result.clone(),
            walk_forward: walk_forward_result.clone(),
            monte_carlo: monte_carlo_result.clone(),
            regime: regime_result.clone(),
            config: self.config.clone(),
        };

        let calculator = ScoreCalculator::new();
        let overall_score = calculator.calculate(&components);
        let grade = ScoreCalculator::grade(overall_score);
        let verdict = self.generate_verdict(&components, overall_score);

        // Generate recommendations
        let rec_generator = RecommendationsGenerator::new();
        let recommendations = rec_generator.generate(&components);

        // Assemble report
        Ok(ValidationReport {
            strategy_name: symbol.to_string(),
            validated_at: Utc::now(),
            test_period,
            overall_score,
            grade,
            verdict,
            long_trades_count: backtest_result.long_trades_count,
            short_trades_count: backtest_result.short_trades_count,
            long_win_rate: backtest_result.long_win_rate,
            short_win_rate: backtest_result.short_win_rate,
            backtest: backtest_result,
            walk_forward: walk_forward_result,
            monte_carlo: monte_carlo_result,
            regime_analysis: regime_result,
            risk_assessment,
            recommendations,
        })
    }

    /// Validate strategy using a factory function for enhanced walk-forward analysis
    ///
    /// This method provides full walk-forward analysis with actual strategy execution
    /// in each test window, rather than simplified buy-and-hold returns.
    ///
    /// # Arguments
    /// * `strategy_factory` - Function that creates fresh strategy instances
    /// * `symbol` - Trading symbol
    /// * `bars` - Historical bar data
    ///
    /// # Returns
    /// Comprehensive validation report with enhanced walk-forward metrics
    ///
    /// # When to Use
    /// - Validating strategies from registry (which provide factories)
    /// - When you have access to strategy creation logic
    /// - When accurate walk-forward analysis is critical
    ///
    /// # See Also
    /// - `validate()` - For boxed strategies (simplified walk-forward)
    pub fn validate_with_factory<F>(
        &self,
        strategy_factory: F,
        symbol: &str,
        bars: &[Bar],
    ) -> Result<ValidationReport, CoreError>
    where
        F: Fn() -> Box<dyn Strategy> + Clone + 'static,
    {
        if bars.is_empty() {
            return Err(CoreError::DataValidation(
                "No historical data provided for validation".to_string(),
            ));
        }

        // Create test period info
        let test_period = self.create_test_period(symbol, bars);

        // Run backtest (call factory once for main backtest)
        let strategy = strategy_factory();
        let backtest_result = self.run_backtest(strategy, symbol, bars)?;

        // **ENHANCED**: Run walk-forward with actual strategy execution
        // Gracefully handle insufficient data by returning empty results
        let walk_forward_result =
            if let Ok(result) =
                crate::walk_forward::WalkForwardAnalyzer::new(self.config.walk_forward.clone())
                    .analyze(bars, symbol, strategy_factory.clone())
            {
                result
            } else {
                // Not enough data for full walk-forward, return empty result
                crate::walk_forward::WalkForwardResult {
                    windows: Vec::new(),
                    aggregate_oos: crate::walk_forward::AggregateMetrics {
                        mean_return: 0.0,
                        median_return: 0.0,
                        mean_sharpe: 0.0,
                        worst_drawdown: 0.0,
                        win_rate: 0.0,
                    },
                    stability_score: 0.0,
                }
            };

        // Run Monte Carlo simulation (uses backtest result)
        let monte_carlo_result = self.run_monte_carlo(&backtest_result)?;

        // Run regime analysis (no strategy reference required)
        // Gracefully handle insufficient data by returning empty results
        let regime_result = self.run_regime_analysis(bars).unwrap_or_default();

        // Calculate risk assessment
        let risk_assessment = self.assess_risk(&backtest_result, &monte_carlo_result);

        // Calculate overall score and generate verdict
        let components = ValidationComponents {
            backtest: backtest_result.clone(),
            walk_forward: walk_forward_result.clone(),
            monte_carlo: monte_carlo_result.clone(),
            regime: regime_result.clone(),
            config: self.config.clone(),
        };

        let calculator = ScoreCalculator::new();
        let overall_score = calculator.calculate(&components);
        let grade = ScoreCalculator::grade(overall_score);
        let verdict = self.generate_verdict(&components, overall_score);

        // Generate recommendations
        let rec_generator = RecommendationsGenerator::new();
        let recommendations = rec_generator.generate(&components);

        // Assemble report
        Ok(ValidationReport {
            strategy_name: symbol.to_string(),
            validated_at: Utc::now(),
            test_period,
            overall_score,
            grade,
            verdict,
            long_trades_count: backtest_result.long_trades_count,
            short_trades_count: backtest_result.short_trades_count,
            long_win_rate: backtest_result.long_win_rate,
            short_win_rate: backtest_result.short_win_rate,
            backtest: backtest_result,
            walk_forward: walk_forward_result,
            monte_carlo: monte_carlo_result,
            regime_analysis: regime_result,
            risk_assessment,
            recommendations,
        })
    }

    /// Run basic backtest on historical data
    fn run_backtest(
        &self,
        strategy: Box<dyn Strategy>,
        symbol: &str,
        bars: &[Bar],
    ) -> Result<BacktestResult, CoreError> {
        let mut engine = BacktestEngine::new(
            self.config.initial_capital,
            self.config.fee_rate,
            SlippageModel::FixedPercent(0.0005), // 0.05% slippage
        );

        // Use provided backtest strategy directly
        engine.set_strategy(strategy);
        engine.add_data(symbol, bars.to_vec());

        // Run backtest
        let _metrics = engine.run().map_err(|e| CoreError::Api(e.to_string()))?;

        // Extract results from engine fields
        let portfolio = &engine.portfolio;
        let equity_history = &portfolio.equity_history;
        let trades = &portfolio.trades;

        let metrics_with_trades = PerformanceMetrics::calculate_with_trades(
            equity_history,
            trades,
            self.config.risk_free_rate,
        );

        let win_rate = if trades.is_empty() {
            0.0
        } else {
            let winning_trades = trades.iter().filter(|t| t.pnl > 0.0).count() as f64;
            winning_trades / trades.len() as f64
        };

        let profit_factor = {
            let gross_profit: f64 = trades.iter().filter(|t| t.pnl > 0.0).map(|t| t.pnl).sum();
            let gross_loss: f64 = trades
                .iter()
                .filter(|t| t.pnl < 0.0)
                .map(|t| t.pnl.abs())
                .sum();

            if gross_loss > 0.0 {
                gross_profit / gross_loss
            } else {
                f64::INFINITY
            }
        };

        // Calculate long/short breakdown
        let long_trades: Vec<_> = trades
            .iter()
            .filter(|t| matches!(t.side, crate::TradeSide::Long))
            .collect();
        let short_trades: Vec<_> = trades
            .iter()
            .filter(|t| matches!(t.side, crate::TradeSide::Short))
            .collect();

        let long_trades_count = long_trades.len();
        let short_trades_count = short_trades.len();

        let long_winning: Vec<_> = long_trades.iter().filter(|t| t.pnl > 0.0).collect();
        let short_winning: Vec<_> = short_trades.iter().filter(|t| t.pnl > 0.0).collect();

        let long_win_rate = if long_trades_count > 0 {
            long_winning.len() as f64 / long_trades_count as f64
        } else {
            0.0
        };

        let short_win_rate = if short_trades_count > 0 {
            short_winning.len() as f64 / short_trades_count as f64
        } else {
            0.0
        };

        let total_long_profit: f64 = long_trades.iter().map(|t| t.pnl).sum();
        let total_short_profit: f64 = short_trades.iter().map(|t| t.pnl).sum();

        let avg_long_profit = if long_trades_count > 0 {
            total_long_profit / long_trades_count as f64
        } else {
            0.0
        };

        let avg_short_profit = if short_trades_count > 0 {
            total_short_profit / short_trades_count as f64
        } else {
            0.0
        };

        Ok(BacktestResult {
            metrics: metrics_with_trades,
            total_trades: trades.len(),
            win_rate,
            profit_factor,
            trades: trades.clone(),
            long_trades_count,
            short_trades_count,
            long_win_rate,
            short_win_rate,
            avg_long_profit,
            avg_short_profit,
            total_long_profit,
            total_short_profit,
        })
    }

    /// Run walk-forward analysis
    fn run_walk_forward(
        &self,
        bars: &[Bar],
    ) -> Result<crate::walk_forward::WalkForwardResult, CoreError> {
        // Simplified walk-forward: run on full dataset and split into windows
        let config = self.config.walk_forward.clone();

        // Note: Full walk-forward requires optimization workflow integration
        // For now, we'll create a simplified version with buy-and-hold returns
        let window_results = Vec::new();

        // Split data into training and test windows
        let total_bars = bars.len();
        let window_size = config.train_window + config.test_window;

        if total_bars < window_size {
            // Not enough data for full walk-forward, return empty result
            return Ok(crate::walk_forward::WalkForwardResult {
                windows: Vec::new(),
                aggregate_oos: crate::walk_forward::AggregateMetrics {
                    mean_return: 0.0,
                    median_return: 0.0,
                    mean_sharpe: 0.0,
                    worst_drawdown: 0.0,
                    win_rate: 0.0,
                },
                stability_score: 0.0,
            });
        }

        // Run on multiple windows
        let mut test_returns: Vec<f64> = Vec::new();
        let mut test_sharpes: Vec<f64> = Vec::new();
        let mut profitable_windows: usize = 0;

        for start in (0..total_bars.saturating_sub(window_size)).step_by(config.step_size) {
            let train_end = start + config.train_window;
            let test_end = (train_end + config.test_window).min(total_bars);

            let test_bars = &bars[train_end..test_end];

            if test_bars.is_empty() {
                continue;
            }

            // Run on test window (simplified - just buy and hold returns)
            if test_bars.len() > 1 {
                let initial_close = test_bars.first().unwrap().close;
                let final_close = test_bars.last().unwrap().close;
                let return_pct = (final_close - initial_close) / initial_close;
                test_returns.push(return_pct);

                if return_pct > 0.0 {
                    profitable_windows += 1;
                }

                // Calculate simple Sharpe estimate
                test_sharpes.push(return_pct / 0.01); // Assume 1% std dev
            }
        }

        // Calculate aggregate metrics
        let win_rate = if !test_returns.is_empty() {
            profitable_windows as f64 / test_returns.len() as f64
        } else {
            0.0
        };

        let mean_return = if !test_returns.is_empty() {
            test_returns.iter().sum::<f64>() / test_returns.len() as f64
        } else {
            0.0
        };

        let mut sorted_returns = test_returns.clone();
        sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let median_return = if !sorted_returns.is_empty() {
            let mid = sorted_returns.len() / 2;
            sorted_returns[mid]
        } else {
            0.0
        };

        let mean_sharpe = if !test_sharpes.is_empty() && test_sharpes.len() > 1 {
            let mean = test_sharpes.iter().sum::<f64>() / test_sharpes.len() as f64;
            let variance = test_sharpes.iter().map(|s| (s - mean).powi(2)).sum::<f64>()
                / test_sharpes.len() as f64;
            let std_dev = variance.sqrt();

            if mean.abs() > 0.0001 {
                1.0 - (std_dev / mean.abs()).min(1.0)
            } else {
                0.0
            }
        } else {
            0.0
        };

        Ok(crate::walk_forward::WalkForwardResult {
            windows: window_results,
            aggregate_oos: crate::walk_forward::AggregateMetrics {
                mean_return,
                median_return,
                mean_sharpe,
                worst_drawdown: 0.0,
                win_rate,
            },
            stability_score: mean_sharpe,
        })
    }

    /// Run Monte Carlo simulation
    fn run_monte_carlo(
        &self,
        backtest_result: &BacktestResult,
    ) -> Result<crate::monte_carlo::MonteCarloResult, CoreError> {
        let config = MonteCarloConfig {
            num_simulations: 1000,
            initial_capital: self.config.initial_capital,
            seed: Some(42),
        };

        let simulator = MonteCarloSimulator::new(config);

        // Extract actual trade returns from backtest
        let mc_trades: Vec<McTrade> = backtest_result
            .trades
            .iter()
            .map(|trade| McTrade {
                symbol: trade.symbol.clone(),
                pnl: trade.pnl,
                return_pct: trade.return_pct(),
                duration: (trade.duration_secs / 3600) as usize, // Convert seconds to hours/bars
            })
            .collect();

        if mc_trades.is_empty() {
            return Ok(crate::monte_carlo::MonteCarloResult {
                num_simulations: 0,
                original_metrics: crate::monte_carlo::SimulationResult {
                    final_equity: self.config.initial_capital,
                    total_return: 0.0,
                    max_drawdown: 0.0,
                    sharpe_ratio: 0.0,
                },
                equity_5th: self.config.initial_capital,
                equity_50th: self.config.initial_capital,
                equity_95th: self.config.initial_capital,
                return_5th: 0.0,
                return_50th: 0.0,
                return_95th: 0.0,
                percentile_5: 0.0,
                percentile_50: 0.0,
                percentile_95: 0.0,
                drawdown_5th: 0.0,
                drawdown_50th: 0.0,
                drawdown_95th: 0.0,
                probability_of_profit: 0.0,
                positive_probability: 0.0,
                simulations: Vec::new(),
            });
        }

        // Run Monte Carlo simulation
        let result = simulator.simulate(&mc_trades);

        Ok(result)
    }

    /// Run regime analysis
    fn run_regime_analysis(&self, bars: &[Bar]) -> Result<RegimeAnalysisResult, CoreError> {
        let analyzer = crate::validation::RegimeAnalyzer::default();

        // For now, use empty expected regimes
        let expected_regimes: Vec<crate::validation::MarketRegime> = Vec::new();

        analyzer.analyze(bars, expected_regimes)
    }

    /// Assess strategy risk profile
    fn assess_risk(
        &self,
        backtest_result: &BacktestResult,
        monte_carlo_result: &crate::monte_carlo::MonteCarloResult,
    ) -> RiskAssessment {
        let metrics = &backtest_result.metrics;

        // Expected vs actual drawdown
        let expected_max_drawdown = self.config.thresholds.max_drawdown;
        let actual_max_drawdown = metrics.max_drawdown;

        // Tail risk from Monte Carlo
        let tail_risk = monte_carlo_result.return_5th;

        // Determine risk rating
        let risk_rating = if actual_max_drawdown <= 0.10 && tail_risk > -0.10 {
            RiskRating::Low
        } else if actual_max_drawdown <= 0.20 && tail_risk > -0.20 {
            RiskRating::Medium
        } else if actual_max_drawdown <= 0.30 && tail_risk > -0.30 {
            RiskRating::High
        } else {
            RiskRating::Extreme
        };

        RiskAssessment {
            expected_max_drawdown,
            actual_max_drawdown,
            tail_risk,
            avg_exposure: 0.5, // Simplified: assume 50% average exposure
            leverage: 1.0,     // Spot-only trading
            risk_rating,
        }
    }

    /// Generate validation verdict
    fn generate_verdict(
        &self,
        components: &ValidationComponents,
        overall_score: f64,
    ) -> ValidationVerdict {
        let backtest = &components.backtest.metrics;
        let wf = &components.walk_forward;
        let mc = &components.monte_carlo;
        let thresholds = &components.config.thresholds;
        let regime = &components.regime;

        // Check for critical failures
        let critical_failures = [
            backtest.sharpe_ratio < thresholds.min_sharpe,
            backtest.max_drawdown > thresholds.max_drawdown * 1.5, // Exceeds threshold by 50%
            wf.aggregate_oos.win_rate < thresholds.min_win_rate * 0.8,
            mc.probability_of_profit < thresholds.min_positive_probability * 0.8,
        ];

        if critical_failures.iter().any(|&f| f) {
            return ValidationVerdict::Fail;
        }

        // Check if optimization is needed
        let needs_optimization = [
            backtest.sharpe_ratio < thresholds.min_sharpe * 1.2,
            wf.stability_score < 0.60,
            regime.regime_mismatch.is_some(),
            overall_score < 70.0,
        ];

        if needs_optimization.iter().any(|&f| f) {
            return ValidationVerdict::NeedsOptimization;
        }

        ValidationVerdict::Pass
    }

    /// Create test period information
    fn create_test_period(&self, _symbol: &str, bars: &[Bar]) -> TestPeriod {
        if bars.is_empty() {
            return TestPeriod {
                start: Utc::now(),
                end: Utc::now(),
                total_bars: 0,
            };
        }

        let start = bars.first().unwrap().timestamp;
        let end = bars.last().unwrap().timestamp;
        let total_bars = bars.len();

        TestPeriod {
            start,
            end,
            total_bars,
        }
    }
}

#[cfg(test)]
mod tests {
    /// Test validate_with_factory() method with successful validation
    #[test]
    fn test_validate_with_factory_success() {
        let config = create_test_config();
        let validator = StrategyValidator::new(config);
        let bars = create_test_bars(400); // 400 bars, need at least 125 (100 train + 25 test)

        let factory = || -> Box<dyn Strategy> { Box::new(TestStrategy::new()) };

        let result = validator.validate_with_factory(factory, "TEST", &bars);
        assert!(result.is_ok());

        let report = result.unwrap();
        assert_eq!(report.strategy_name, "TEST");
        assert!(report.walk_forward.stability_score >= 0.0);
        assert!(report.walk_forward.stability_score <= 1.0);
        assert!(report.overall_score >= 0.0);
        assert!(report.overall_score <= 100.0);
    }

    /// Test validate_with_factory() with insufficient data for walk-forward
    #[test]
    fn test_validate_with_factory_insufficient_data() {
        let config = create_test_config();
        let validator = StrategyValidator::new(config);
        let bars = create_test_bars(50); // Only 50 bars, but need 125 (100 train + 25 test)

        let factory = || -> Box<dyn Strategy> { Box::new(TestStrategy::new()) };

        let result = validator.validate_with_factory(factory, "TEST", &bars);

        // Debug: Print error if any
        if let Err(ref e) = result {
            eprintln!("validate_with_factory failed with: {:?}", e);
        }

        // Should succeed overall but with minimal walk-forward results
        assert!(
            result.is_ok(),
            "Validation should succeed even with insufficient data"
        );
        let report = result.unwrap();
        assert!(report.walk_forward.windows.is_empty());
    }

    /// Test that validate_with_factory() produces enhanced walk-forward results
    /// compared to the simplified validate() method
    #[test]
    fn test_validate_vs_validate_with_factory() {
        let config = create_test_config();
        let validator = StrategyValidator::new(config);
        let bars = create_test_bars(400);

        // Test 1: Original method with Box<dyn Strategy> (simplified walk-forward)
        let strategy = Box::new(TestStrategy::new());
        let result_boxed = validator.validate(strategy, "TEST", &bars).unwrap();

        // Test 2: New method with factory (enhanced walk-forward)
        let factory = || -> Box<dyn Strategy> { Box::new(TestStrategy::new()) };
        let result_factory = validator
            .validate_with_factory(factory, "TEST", &bars)
            .unwrap();

        // Main backtest results should be similar (same data, same strategy type)
        // May differ slightly due to random factors in some components
        assert!(
            (result_boxed.backtest.metrics.total_return
                - result_factory.backtest.metrics.total_return)
                .abs()
                < 0.01
        );

        // Walk-forward results should differ significantly:
        // - Simplified: empty windows or minimal results
        // - Enhanced: actual strategy execution in windows
        assert!(
            result_factory.walk_forward.windows.len() >= result_boxed.walk_forward.windows.len()
        );

        // Enhanced version should have non-empty windows (actual strategy execution)
        if !result_factory.walk_forward.windows.is_empty() {
            let first_window = &result_factory.walk_forward.windows[0];
            assert!(first_window.train_start < first_window.train_end);
            assert!(first_window.test_start < first_window.test_end);
        }
    }

    /// Test validate_with_factory() error handling for empty data
    #[test]
    fn test_validate_with_factory_empty_data() {
        let config = create_test_config();
        let validator = StrategyValidator::new(config);
        let bars: Vec<Bar> = Vec::new();

        let factory = || -> Box<dyn Strategy> { Box::new(TestStrategy::new()) };

        let result = validator.validate_with_factory(factory, "TEST", &bars);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No historical data"));
    }

    /// Test validate_with_factory() produces valid report structure
    #[test]
    fn test_validate_with_factory_report_structure() {
        let config = create_test_config();
        let validator = StrategyValidator::new(config);
        let bars = create_test_bars(400);

        let factory = || -> Box<dyn Strategy> { Box::new(TestStrategy::new()) };

        let result = validator
            .validate_with_factory(factory, "TEST", &bars)
            .unwrap();

        // Verify all required fields are present and valid
        assert!(!result.strategy_name.is_empty());
        assert!(result.overall_score >= 0.0 && result.overall_score <= 100.0);

        // Verify component results
        assert_eq!(
            result.backtest.metrics.total_trades,
            result.backtest.total_trades
        );
        assert!(
            result.walk_forward.stability_score >= 0.0
                && result.walk_forward.stability_score <= 1.0
        );
        assert!(result.monte_carlo.num_simulations > 0);

        // Verify risk assessment
        assert!(result.risk_assessment.actual_max_drawdown >= 0.0);
        assert_eq!(result.risk_assessment.leverage, 1.0); // Spot-only

        // Verify verdict is one of the valid options
        matches!(
            result.verdict,
            ValidationVerdict::Pass
                | ValidationVerdict::Fail
                | ValidationVerdict::NeedsOptimization
        );
    }
    use super::*;
    use crate::validation::ValidationThresholds;
    use alphafield_core::Bar;
    use chrono::Utc;

    #[derive(Debug, Clone)]
    struct TestStrategy;

    impl TestStrategy {
        pub fn new() -> Self {
            Self
        }
    }

    impl crate::strategy::Strategy for TestStrategy {
        fn on_bar(
            &mut self,
            bar: &Bar,
        ) -> crate::error::Result<Vec<crate::strategy::OrderRequest>> {
            // Simple strategy: buy when price increases (produce a market buy order)
            if bar.close > bar.open {
                Ok(vec![crate::strategy::OrderRequest {
                    symbol: "TEST".to_string(),
                    side: crate::strategy::OrderSide::Buy,
                    quantity: 1.0,
                    order_type: crate::strategy::OrderType::Market,
                }])
            } else {
                Ok(Vec::new())
            }
        }

        fn on_tick(
            &mut self,
            _tick: &alphafield_core::Tick,
        ) -> crate::error::Result<Vec<crate::strategy::OrderRequest>> {
            Ok(Vec::new())
        }
    }

    fn create_test_bars(count: usize) -> Vec<Bar> {
        (0..count)
            .map(|i| {
                let base_price = 100.0 + (i as f64 * 0.1);
                Bar {
                    timestamp: Utc::now() + chrono::Duration::seconds(i as i64 * 3600),
                    open: base_price,
                    high: base_price + 1.0,
                    low: base_price - 1.0,
                    close: base_price + (i as f64 % 3.0 - 1.0) * 0.5,
                    volume: 1000.0,
                }
            })
            .collect()
    }

    fn create_test_config() -> ValidationConfig {
        ValidationConfig {
            data_source: "test".to_string(),
            symbol: "BTC".to_string(),
            interval: "1h".to_string(),
            walk_forward: crate::walk_forward::WalkForwardConfig {
                train_window: 100,
                test_window: 25,
                step_size: 21,
                initial_capital: 10000.0,
                fee_rate: 0.001,
            },
            risk_free_rate: 0.02,
            thresholds: ValidationThresholds {
                min_sharpe: 1.0,
                max_drawdown: 0.30,
                min_win_rate: 0.60,
                min_positive_probability: 0.70,
            },
            initial_capital: 10000.0,
            fee_rate: 0.001,
        }
    }

    #[test]
    fn test_validator_creation() {
        let config = create_test_config();
        let validator = StrategyValidator::new(config);

        assert_eq!(validator.config.symbol, "BTC");
    }

    #[test]
    fn test_empty_bars_error() {
        let config = create_test_config();
        let validator = StrategyValidator::new(config);
        let strategy = TestStrategy::new();
        let bars = Vec::new();

        let result = validator.validate(Box::new(strategy), "BTC", &bars);

        assert!(result.is_err());
        match result {
            Err(CoreError::DataValidation(msg)) => {
                assert!(msg.contains("No historical data"));
            }
            _ => panic!("Expected DataValidation error"),
        }
    }

    #[test]
    fn test_validation_report_structure() {
        let config = create_test_config();
        let validator = StrategyValidator::new(config);
        let strategy = TestStrategy::new();
        let bars = create_test_bars(500);

        let report = validator
            .validate(Box::new(strategy), "BTC", &bars)
            .unwrap();

        assert_eq!(report.strategy_name, "BTC");
        assert!(report.overall_score >= 0.0);
        assert!(report.overall_score <= 100.0);
        assert!(['A', 'B', 'C', 'D', 'F'].contains(&report.grade));
    }

    #[test]
    fn test_verdict_generation() {
        let config = create_test_config();
        let validator = StrategyValidator::new(config);

        // Test Pass verdict
        let mut components = create_test_components();
        components.backtest.metrics.sharpe_ratio = 1.5;
        components.walk_forward.aggregate_oos.win_rate = 0.70;
        components.monte_carlo.probability_of_profit = 0.80;

        let verdict = validator.generate_verdict(&components, 85.0);
        assert_eq!(verdict, ValidationVerdict::Pass);

        // Test Fail verdict
        components.backtest.metrics.sharpe_ratio = 0.5;
        let verdict = validator.generate_verdict(&components, 40.0);
        assert_eq!(verdict, ValidationVerdict::Fail);

        // Test NeedsOptimization verdict
        components.backtest.metrics.sharpe_ratio = 1.1;
        components.walk_forward.stability_score = 0.5;
        let verdict = validator.generate_verdict(&components, 65.0);
        assert_eq!(verdict, ValidationVerdict::NeedsOptimization);
    }

    fn create_test_components() -> ValidationComponents {
        ValidationComponents {
            backtest: crate::validation::BacktestResult {
                metrics: Default::default(),
                total_trades: 0,
                win_rate: 0.0,
                profit_factor: 0.0,
                trades: Vec::new(),
                long_trades_count: 0,
                short_trades_count: 0,
                long_win_rate: 0.0,
                short_win_rate: 0.0,
                avg_long_profit: 0.0,
                avg_short_profit: 0.0,
                total_long_profit: 0.0,
                total_short_profit: 0.0,
            },
            walk_forward: crate::walk_forward::WalkForwardResult {
                windows: Vec::new(),
                aggregate_oos: crate::walk_forward::AggregateMetrics {
                    mean_return: 0.0,
                    median_return: 0.0,
                    mean_sharpe: 0.0,
                    worst_drawdown: 0.0,
                    win_rate: 0.70,
                },
                stability_score: 0.75,
            },
            monte_carlo: crate::monte_carlo::MonteCarloResult {
                probability_of_profit: 0.80,
                ..Default::default()
            },
            regime: crate::validation::RegimeAnalysisResult::default(),
            config: create_test_config(),
        }
    }
}
