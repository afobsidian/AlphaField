//! Monte Carlo Simulation Module
//!
//! Stress testing through trade order shuffling to analyze path dependency
//! and calculate confidence intervals.

use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Configuration for Monte Carlo simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonteCarloConfig {
    /// Number of simulations to run
    pub num_simulations: usize,
    /// Initial capital
    pub initial_capital: f64,
    /// Random seed for reproducibility (None = random)
    pub seed: Option<u64>,
}

impl Default for MonteCarloConfig {
    fn default() -> Self {
        Self {
            num_simulations: 1000,
            initial_capital: 10000.0,
            seed: None,
        }
    }
}

/// A single trade record from backtest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// Symbol traded
    pub symbol: String,
    /// Profit/loss from this trade
    pub pnl: f64,
    /// Return percentage
    pub return_pct: f64,
    /// Trade duration in bars/periods
    pub duration: usize,
}

/// Result from a single Monte Carlo simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    /// Final equity
    pub final_equity: f64,
    /// Total return
    pub total_return: f64,
    /// Maximum drawdown during simulation
    pub max_drawdown: f64,
    /// Sharpe ratio (if enough data points)
    pub sharpe_ratio: f64,
}

/// Aggregated Monte Carlo results with confidence intervals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonteCarloResult {
    /// Number of simulations run
    pub num_simulations: usize,
    /// Original (unshuffled) performance
    pub original_metrics: SimulationResult,

    /// Confidence intervals for final equity
    pub equity_5th: f64,
    pub equity_50th: f64,
    pub equity_95th: f64,

    /// Confidence intervals for returns
    pub return_5th: f64,
    pub return_50th: f64,
    pub return_95th: f64,

    /// Confidence intervals for max drawdown
    pub drawdown_5th: f64,
    pub drawdown_50th: f64,
    pub drawdown_95th: f64,

    /// Probability of profit (% of simulations with positive return)
    pub probability_of_profit: f64,

    /// All simulation results (for visualization)
    pub simulations: Vec<SimulationResult>,
}

/// Monte Carlo simulator
pub struct MonteCarloSimulator {
    config: MonteCarloConfig,
}

impl MonteCarloSimulator {
    pub fn new(config: MonteCarloConfig) -> Self {
        Self { config }
    }

    /// Run Monte Carlo simulation by shuffling trade order
    ///
    /// # Arguments
    /// * `trades` - List of trades from backtest (will be shuffled N times)
    ///
    /// # Returns
    /// Monte Carlo results with confidence intervals
    pub fn simulate(&self, trades: &[Trade]) -> MonteCarloResult {
        if trades.is_empty() {
            return MonteCarloResult {
                num_simulations: 0,
                original_metrics: SimulationResult {
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
                drawdown_5th: 0.0,
                drawdown_50th: 0.0,
                drawdown_95th: 0.0,
                probability_of_profit: 0.0,
                simulations: vec![],
            };
        }

        // Calculate original (unshuffled) result
        let original_metrics = self.run_simulation(trades);

        // Create RNG with optional seed
        let mut rng = match self.config.seed {
            Some(seed) => ChaCha8Rng::seed_from_u64(seed),
            None => ChaCha8Rng::from_entropy(),
        };

        // Run shuffled simulations
        let mut simulations = Vec::with_capacity(self.config.num_simulations);
        let mut shuffled_trades = trades.to_vec();

        for _ in 0..self.config.num_simulations {
            shuffled_trades.shuffle(&mut rng);
            let result = self.run_simulation(&shuffled_trades);
            simulations.push(result);
        }

        // Calculate percentiles
        let mut equities: Vec<f64> = simulations.iter().map(|s| s.final_equity).collect();
        let mut returns: Vec<f64> = simulations.iter().map(|s| s.total_return).collect();
        let mut drawdowns: Vec<f64> = simulations.iter().map(|s| s.max_drawdown).collect();

        equities.sort_by(|a, b| a.partial_cmp(b).unwrap());
        returns.sort_by(|a, b| a.partial_cmp(b).unwrap());
        drawdowns.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = simulations.len();
        let idx_5 = (n as f64 * 0.05) as usize;
        let idx_50 = n / 2;
        let idx_95 = (n as f64 * 0.95) as usize;

        let profitable = simulations.iter().filter(|s| s.total_return > 0.0).count();

        MonteCarloResult {
            num_simulations: self.config.num_simulations,
            original_metrics,
            equity_5th: equities[idx_5],
            equity_50th: equities[idx_50],
            equity_95th: equities[idx_95.min(n - 1)],
            return_5th: returns[idx_5],
            return_50th: returns[idx_50],
            return_95th: returns[idx_95.min(n - 1)],
            drawdown_5th: drawdowns[idx_5],
            drawdown_50th: drawdowns[idx_50],
            drawdown_95th: drawdowns[idx_95.min(n - 1)],
            probability_of_profit: profitable as f64 / n as f64,
            simulations,
        }
    }

    /// Run a single simulation with given trade order
    fn run_simulation(&self, trades: &[Trade]) -> SimulationResult {
        let mut equity = self.config.initial_capital;
        let mut peak = equity;
        let mut max_drawdown = 0.0;
        let mut equity_curve = vec![equity];

        for trade in trades {
            equity += trade.pnl;
            equity_curve.push(equity);

            if equity > peak {
                peak = equity;
            }

            let drawdown = (peak - equity) / peak;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        let total_return = (equity - self.config.initial_capital) / self.config.initial_capital;

        // Calculate Sharpe ratio from equity curve
        let sharpe_ratio = if equity_curve.len() > 1 {
            let returns: Vec<f64> = equity_curve
                .windows(2)
                .map(|w| (w[1] - w[0]) / w[0])
                .collect();

            let mean = returns.iter().sum::<f64>() / returns.len() as f64;
            let variance =
                returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;
            let std_dev = variance.sqrt();

            if std_dev > 0.0 {
                (mean * 252.0_f64.sqrt()) / (std_dev * 252.0_f64.sqrt())
            } else {
                0.0
            }
        } else {
            0.0
        };

        SimulationResult {
            final_equity: equity,
            total_return,
            max_drawdown,
            sharpe_ratio,
        }
    }
}

/// Helper to extract trades from a backtest run
/// This would typically be called after running a backtest
pub fn extract_trades_from_fills(
    fills: &[(String, f64, f64, f64)], // (symbol, quantity, price, pnl)
) -> Vec<Trade> {
    fills
        .iter()
        .filter(|(_, _, _, pnl)| pnl.abs() > 0.0) // Only completed round-trips
        .map(|(symbol, quantity, price, pnl)| {
            let entry_value = quantity.abs() * price;
            Trade {
                symbol: symbol.clone(),
                pnl: *pnl,
                return_pct: if entry_value > 0.0 {
                    pnl / entry_value
                } else {
                    0.0
                },
                duration: 1, // Simplified - would need trade timestamps for real duration
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monte_carlo_basic() {
        let trades = vec![
            Trade {
                symbol: "BTC".to_string(),
                pnl: 100.0,
                return_pct: 0.01,
                duration: 5,
            },
            Trade {
                symbol: "BTC".to_string(),
                pnl: -50.0,
                return_pct: -0.005,
                duration: 3,
            },
            Trade {
                symbol: "BTC".to_string(),
                pnl: 75.0,
                return_pct: 0.0075,
                duration: 4,
            },
            Trade {
                symbol: "BTC".to_string(),
                pnl: -25.0,
                return_pct: -0.0025,
                duration: 2,
            },
            Trade {
                symbol: "BTC".to_string(),
                pnl: 150.0,
                return_pct: 0.015,
                duration: 6,
            },
        ];

        let config = MonteCarloConfig {
            num_simulations: 100,
            initial_capital: 10000.0,
            seed: Some(42), // For reproducibility
        };

        let simulator = MonteCarloSimulator::new(config);
        let result = simulator.simulate(&trades);

        assert_eq!(result.num_simulations, 100);
        assert!(result.probability_of_profit > 0.0);

        // Original should have consistent result
        assert!((result.original_metrics.total_return - 0.025).abs() < 0.001);
    }

    #[test]
    fn test_empty_trades() {
        let config = MonteCarloConfig::default();
        let simulator = MonteCarloSimulator::new(config);
        let result = simulator.simulate(&[]);

        assert_eq!(result.num_simulations, 0);
        assert_eq!(result.original_metrics.total_return, 0.0);
    }
}
