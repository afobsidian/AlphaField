use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_return: f64,
    pub cagr: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub volatility: f64,
}

impl PerformanceMetrics {
    pub fn calculate(equity_history: &[(i64, f64)], risk_free_rate: f64) -> Self {
        if equity_history.is_empty() {
            return Self {
                total_return: 0.0,
                cagr: 0.0,
                sharpe_ratio: 0.0,
                max_drawdown: 0.0,
                volatility: 0.0,
            };
        }

        let initial_equity = equity_history.first().unwrap().1;
        let final_equity = equity_history.last().unwrap().1;

        // Total Return
        let total_return = (final_equity - initial_equity) / initial_equity;

        // CAGR (assuming daily data points for simplicity, can be adjusted)
        // Duration in years
        let start_time = equity_history.first().unwrap().0;
        let end_time = equity_history.last().unwrap().0;
        let duration_years = (end_time - start_time) as f64 / (1000.0 * 3600.0 * 24.0 * 365.0);
        
        let cagr = if duration_years > 0.0 {
            (final_equity / initial_equity).powf(1.0 / duration_years) - 1.0
        } else {
            0.0
        };

        // Max Drawdown
        let mut max_drawdown = 0.0;
        let mut peak = initial_equity;
        
        for &(_, equity) in equity_history {
            if equity > peak {
                peak = equity;
            }
            let drawdown = (peak - equity) / peak;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        // Sharpe Ratio & Volatility
        // Calculate daily returns
        let mut returns = Vec::new();
        for i in 1..equity_history.len() {
            let prev = equity_history[i-1].1;
            let curr = equity_history[i].1;
            returns.push((curr - prev) / prev);
        }

        let avg_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter().map(|r| (r - avg_return).powi(2)).sum::<f64>() / returns.len() as f64;
        let volatility = variance.sqrt() * (252.0f64).sqrt(); // Annualized volatility

        let annualized_return = avg_return * 252.0; // Simple annualization
        let sharpe_ratio = if volatility > 0.0 {
            (annualized_return - risk_free_rate) / volatility
        } else {
            0.0
        };

        Self {
            total_return,
            cagr,
            sharpe_ratio,
            max_drawdown,
            volatility,
        }
    }
}
