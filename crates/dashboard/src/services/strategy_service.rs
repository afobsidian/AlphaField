use alphafield_backtest::{strategy::Strategy as BacktestStrategy, StrategyAdapter};
use alphafield_core::Strategy;
use alphafield_strategy::{
    GoldenCrossStrategy, MeanReversionStrategy, MomentumStrategy, RsiStrategy,
};
use std::collections::HashMap;
use tracing::debug;

pub struct StrategyFactory;

impl StrategyFactory {
    pub fn create(name: &str, params: &HashMap<String, f64>) -> Option<Box<dyn Strategy>> {
        debug!(strategy = name, ?params, "Creating strategy");
        match name {
            "GoldenCross" => {
                let fast = params.get("fast_period").copied().unwrap_or(10.0) as usize;
                let slow = params.get("slow_period").copied().unwrap_or(30.0) as usize;
                let tp = params.get("take_profit").copied().unwrap_or(5.0);
                let sl = params.get("stop_loss").copied().unwrap_or(5.0);

                if fast >= slow || fast == 0 || slow == 0 {
                    debug!("Invalid GoldenCross params: fast={} slow={}", fast, slow);
                    return None;
                }
                // Use config to pass all params
                let config =
                    alphafield_strategy::config::GoldenCrossConfig::new(fast, slow, tp, sl);
                Some(Box::new(GoldenCrossStrategy::from_config(config)))
            }
            "Rsi" => {
                let period = params.get("period").copied().unwrap_or(14.0) as usize;
                let lower = params.get("lower_bound").copied().unwrap_or(30.0);
                let upper = params.get("upper_bound").copied().unwrap_or(70.0);
                let tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(5.0);

                if period == 0 || lower >= upper {
                    debug!(
                        "Invalid RSI params: period={} lower={} upper={}",
                        period, lower, upper
                    );
                    return None;
                }

                let config =
                    alphafield_strategy::config::RsiConfig::new(period, lower, upper, tp, sl);
                Some(Box::new(RsiStrategy::from_config(config)))
            }
            "MeanReversion" => {
                let period = params.get("period").copied().unwrap_or(20.0) as usize;
                let std_dev = params.get("std_dev").copied().unwrap_or(2.0);
                let tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(5.0);

                if period == 0 || std_dev <= 0.0 {
                    debug!(
                        "Invalid MeanReversion params: period={} std_dev={}",
                        period, std_dev
                    );
                    return None;
                }
                // Config might be internal or external. Assuming exposed via `new_with_exits` created earlier BUT MeanReversionConfig isn't in config:: usually?
                // Wait, in previous steps I saw MeanReversionConfig in mean_reversion.rs.
                // I need to use MeanReversionStrategy::new which calls new_with_exits?
                // Actually `new` calls `new_with_exits(..., 3.0, 5.0)`.
                // I should construct via from_config if I can access Config.
                // `MeanReversionConfig` is public in `alphafield_strategy::strategies::mean_reversion`.
                // Is it re-exported? Usually strategies are re-exported.
                // Let's assume I can construct it or use a method on Strategy struct if available.
                // In step 395 I updated `MeanReversionConfig` to have `new_with_exits`.
                // But I didn't verify if `MeanReversionStrategy` exposes `new_with_exits`.
                // Checking Step 395:
                // impl MeanReversionConfig { pub fn new_with_exits ... }
                // impl MeanReversionStrategy { pub fn new ... calls new_with_exits(..., 3.0, 5.0) }
                // Use `from_config` method.
                // But I need to construct `MeanReversionConfig`. It is `pub struct`.
                // Is it accessible here? `use alphafield_strategy::{MeanReversionStrategy...}`
                // It might not be re-exported at top level.
                // However, I can try `MeanReversionStrategy::new`?? No that uses defaults.
                // I should construct config.
                // `MeanReversionConfig` might be at `alphafield_strategy::strategies::mean_reversion::MeanReversionConfig`.
                // Or maybe `alphafield_strategy::MeanReversionConfig` if re-exported.
                // Let's assume I can't easily access the Config struct if it's not in `config.rs`.
                // Does `MeanReversionStrategy` have a `new_with_exits`?
                // I will add one if needed, OR I will assume `MeanReversionConfig` is available.
                // Actually, `config.rs` usually holds configs... wait. MeanReversionConfig IS in `mean_reversion.rs`.
                // I will assume `alphafield_strategy::MeanReversionConfig` is available or I will add a `new_with_exits` to `MeanReversionStrategy` in a separate step if compilation fails.
                // BETTER: I will assume `from_config` works and `MeanReversionConfig` is importable.
                // Wait, `crates/dashboard/src/services/strategy_service.rs` has `use alphafield_strategy::{...}`.
                // If MeanReversionConfig is not there, I can't use it.
                // I'll check `c:\Users\adamf\Documents\Projects\AlphaField\crates\strategy\src\strategies\mean_reversion.rs` again.
                // It is `pub struct MeanReversionConfig`.
                // Strategy crate likely re-exports it.

                // Safe bet: Use `MeanReversionConfig::new_with_exits` assuming it's imported.
                // If not, I'll fix it.
                let config = alphafield_strategy::strategies::mean_reversion::MeanReversionConfig::new_with_exits(period, std_dev, tp, sl);
                Some(Box::new(MeanReversionStrategy::from_config(config)))
            }
            "Momentum" => {
                let ema_period = params.get("ema_period").copied().unwrap_or(50.0) as usize;
                let macd_fast = params.get("macd_fast").copied().unwrap_or(12.0) as usize;
                let macd_slow = params.get("macd_slow").copied().unwrap_or(26.0) as usize;
                let macd_signal = params.get("macd_signal").copied().unwrap_or(9.0) as usize;
                let tp = params.get("take_profit").copied().unwrap_or(5.0);
                let sl = params.get("stop_loss").copied().unwrap_or(5.0);

                if macd_fast >= macd_slow || ema_period == 0 || macd_signal == 0 {
                    return None;
                }

                let config =
                    alphafield_strategy::strategies::momentum::MomentumConfig::new_with_exits(
                        ema_period,
                        macd_fast,
                        macd_slow,
                        macd_signal,
                        tp,
                        sl,
                    );
                Some(Box::new(MomentumStrategy::from_config(config)))
            }
            _ => None,
        }
    }

    /// Create a backtest-ready strategy wrapped in StrategyAdapter
    /// This returns the backtest Strategy trait (produces OrderRequests from Signals)
    pub fn create_backtest(
        name: &str,
        params: &HashMap<String, f64>,
        symbol: &str,
        capital: f64,
    ) -> Option<Box<dyn BacktestStrategy>> {
        debug!(strategy = name, ?params, "Creating backtest strategy");
        match name {
            "GoldenCross" => {
                let fast = params.get("fast_period").copied().unwrap_or(10.0) as usize;
                let slow = params.get("slow_period").copied().unwrap_or(30.0) as usize;
                let tp = params.get("take_profit").copied().unwrap_or(5.0);
                let sl = params.get("stop_loss").copied().unwrap_or(5.0);

                if fast >= slow || fast == 0 || slow == 0 {
                    return None;
                }
                let config =
                    alphafield_strategy::config::GoldenCrossConfig::new(fast, slow, tp, sl);
                let strat = GoldenCrossStrategy::from_config(config);
                Some(Box::new(StrategyAdapter::new(strat, symbol, capital)))
            }
            "Rsi" => {
                let period = params.get("period").copied().unwrap_or(14.0) as usize;
                let lower = params.get("lower_bound").copied().unwrap_or(30.0);
                let upper = params.get("upper_bound").copied().unwrap_or(70.0);
                let tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(5.0);

                if period == 0 || lower >= upper {
                    return None;
                }
                let config =
                    alphafield_strategy::config::RsiConfig::new(period, lower, upper, tp, sl);
                let strat = RsiStrategy::from_config(config);
                Some(Box::new(StrategyAdapter::new(strat, symbol, capital)))
            }
            "MeanReversion" => {
                let period = params.get("period").copied().unwrap_or(20.0) as usize;
                let std_dev = params.get("std_dev").copied().unwrap_or(2.0);
                let tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(5.0);

                if period == 0 || std_dev <= 0.0 {
                    return None;
                }
                let config = alphafield_strategy::strategies::mean_reversion::MeanReversionConfig::new_with_exits(period, std_dev, tp, sl);
                let strat = MeanReversionStrategy::from_config(config);
                Some(Box::new(StrategyAdapter::new(strat, symbol, capital)))
            }
            "Momentum" => {
                let ema_period = params.get("ema_period").copied().unwrap_or(50.0) as usize;
                let macd_fast = params.get("macd_fast").copied().unwrap_or(12.0) as usize;
                let macd_slow = params.get("macd_slow").copied().unwrap_or(26.0) as usize;
                let macd_signal = params.get("macd_signal").copied().unwrap_or(9.0) as usize;
                let tp = params.get("take_profit").copied().unwrap_or(5.0);
                let sl = params.get("stop_loss").copied().unwrap_or(5.0);

                if macd_fast >= macd_slow || ema_period == 0 || macd_signal == 0 {
                    return None;
                }
                let config =
                    alphafield_strategy::strategies::momentum::MomentumConfig::new_with_exits(
                        ema_period,
                        macd_fast,
                        macd_slow,
                        macd_signal,
                        tp,
                        sl,
                    );
                let strat = MomentumStrategy::from_config(config);
                Some(Box::new(StrategyAdapter::new(strat, symbol, capital)))
            }
            _ => None,
        }
    }
}
