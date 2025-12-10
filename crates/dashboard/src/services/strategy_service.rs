use std::collections::HashMap;
use alphafield_core::Strategy;
use alphafield_strategy::{GoldenCrossStrategy, MeanReversionStrategy, MomentumStrategy, RsiStrategy};
use alphafield_backtest::{StrategyAdapter, strategy::Strategy as BacktestStrategy};
use tracing::debug;

pub struct StrategyFactory;

impl StrategyFactory {
    pub fn create(name: &str, params: &HashMap<String, f64>) -> Option<Box<dyn Strategy>> {
        debug!(strategy = name, ?params, "Creating strategy");
        match name {
            "GoldenCross" => {
                let fast = params.get("fast_period").copied().unwrap_or(10.0) as usize;
                let slow = params.get("slow_period").copied().unwrap_or(30.0) as usize;
                // Validate params to prevent panic
                if fast >= slow || fast == 0 || slow == 0 {
                    debug!("Invalid GoldenCross params: fast={} slow={}", fast, slow);
                    return None;
                }
                Some(Box::new(GoldenCrossStrategy::new(fast, slow)))
            }
            "Rsi" => {
                let period = params.get("period").copied().unwrap_or(14.0) as usize;
                let lower = params.get("lower_bound").copied().unwrap_or(30.0);
                let upper = params.get("upper_bound").copied().unwrap_or(70.0);
                // Validate
                if period == 0 || lower >= upper {
                    debug!("Invalid RSI params: period={} lower={} upper={}", period, lower, upper);
                    return None;
                }
                Some(Box::new(RsiStrategy::new(period, lower, upper)))
            }
            "MeanReversion" => {
                let period = params.get("period").copied().unwrap_or(20.0) as usize;
                let std_dev = params.get("std_dev").copied().unwrap_or(2.0);
                // Validate
                if period == 0 || std_dev <= 0.0 {
                    debug!("Invalid MeanReversion params: period={} std_dev={}", period, std_dev);
                    return None;
                }
                Some(Box::new(MeanReversionStrategy::new(period, std_dev)))
            }
            "Momentum" => {
                let ema_period = params.get("ema_period").copied().unwrap_or(50.0) as usize;
                let macd_fast = params.get("macd_fast").copied().unwrap_or(12.0) as usize;
                let macd_slow = params.get("macd_slow").copied().unwrap_or(26.0) as usize;
                let macd_signal = params.get("macd_signal").copied().unwrap_or(9.0) as usize;
                // Validate MACD requires fast < slow
                if macd_fast >= macd_slow || ema_period == 0 || macd_signal == 0 {
                    debug!("Invalid Momentum params: ema={} fast={} slow={} signal={}", 
                           ema_period, macd_fast, macd_slow, macd_signal);
                    return None;
                }
                Some(Box::new(MomentumStrategy::new(ema_period, macd_fast, macd_slow, macd_signal)))
            }
            _ => None,
        }
    }

    /// Create a backtest-ready strategy wrapped in StrategyAdapter
    /// This returns the backtest Strategy trait (produces OrderRequests from Signals)
    pub fn create_backtest(name: &str, params: &HashMap<String, f64>, symbol: &str, capital: f64) -> Option<Box<dyn BacktestStrategy>> {
        debug!(strategy = name, ?params, "Creating backtest strategy");
        match name {
            "GoldenCross" => {
                let fast = params.get("fast_period").copied().unwrap_or(10.0) as usize;
                let slow = params.get("slow_period").copied().unwrap_or(30.0) as usize;
                if fast >= slow || fast == 0 || slow == 0 {
                    return None;
                }
                let strat = GoldenCrossStrategy::new(fast, slow);
                Some(Box::new(StrategyAdapter::new(strat, symbol, capital)))
            }
            "Rsi" => {
                let period = params.get("period").copied().unwrap_or(14.0) as usize;
                let lower = params.get("lower_bound").copied().unwrap_or(30.0);
                let upper = params.get("upper_bound").copied().unwrap_or(70.0);
                if period == 0 || lower >= upper {
                    return None;
                }
                let strat = RsiStrategy::new(period, lower, upper);
                Some(Box::new(StrategyAdapter::new(strat, symbol, capital)))
            }
            "MeanReversion" => {
                let period = params.get("period").copied().unwrap_or(20.0) as usize;
                let std_dev = params.get("std_dev").copied().unwrap_or(2.0);
                if period == 0 || std_dev <= 0.0 {
                    return None;
                }
                let strat = MeanReversionStrategy::new(period, std_dev);
                Some(Box::new(StrategyAdapter::new(strat, symbol, capital)))
            }
            "Momentum" => {
                let ema_period = params.get("ema_period").copied().unwrap_or(50.0) as usize;
                let macd_fast = params.get("macd_fast").copied().unwrap_or(12.0) as usize;
                let macd_slow = params.get("macd_slow").copied().unwrap_or(26.0) as usize;
                let macd_signal = params.get("macd_signal").copied().unwrap_or(9.0) as usize;
                if macd_fast >= macd_slow || ema_period == 0 || macd_signal == 0 {
                    return None;
                }
                let strat = MomentumStrategy::new(ema_period, macd_fast, macd_slow, macd_signal);
                Some(Box::new(StrategyAdapter::new(strat, symbol, capital)))
            }
            _ => None,
        }
    }
}
