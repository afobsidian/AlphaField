use alphafield_backtest::{strategy::Strategy as BacktestStrategy, StrategyAdapter};
use alphafield_core::Strategy;
use alphafield_strategy::{
    framework::canonicalize_strategy_name, BollingerBandsStrategy, GoldenCrossStrategy,
    MomentumStrategy, RsiStrategy,
};
use std::collections::HashMap;
use tracing::debug;

pub struct StrategyFactory;

impl StrategyFactory {
    pub fn create(name: &str, params: &HashMap<String, f64>) -> Option<Box<dyn Strategy>> {
        let name = canonicalize_strategy_name(name);
        debug!(strategy = name, ?params, "Creating strategy");
        match name.as_str() {
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
            "Breakout" => {
                // NOTE: Breakout strategy limitation
                // The BreakoutStrategy supports multi-level take profits (tp1_pct, tp2_pct, tp3_pct)
                // with configurable partial position exits. However, the dashboard API currently only
                // exposes three parameters: lookback, take_profit, and stop_loss.
                //
                // The BreakoutStrategy::new() constructor uses a default config with hardcoded
                // TP levels: 3%, 6%, and 10%, closing 30%, 40%, and 30% of position respectively.
                //
                // Future enhancement: Update dashboard UI and StrategyFactory to accept additional
                // TP parameters and use BreakoutStrategy::from_config() for full customization.
                let lookback = params.get("lookback").copied().unwrap_or(20.0) as usize;
                let _tp = params.get("take_profit").copied().unwrap_or(5.0);
                let _sl = params.get("stop_loss").copied().unwrap_or(5.0);

                if lookback == 0 {
                    return None;
                }

                // BreakoutStrategy::new only takes lookback; exits are handled by higher layers / defaults.
                Some(Box::new(
                    alphafield_strategy::strategies::trend_following::BreakoutStrategy::new(
                        lookback,
                    ),
                ))
            }
            "MACrossover" => {
                let fast = params.get("fast_period").copied().unwrap_or(10.0) as usize;
                let slow = params.get("slow_period").copied().unwrap_or(30.0) as usize;
                let _tp = params.get("take_profit").copied().unwrap_or(5.0);
                let _sl = params.get("stop_loss").copied().unwrap_or(5.0);

                if fast == 0 || slow == 0 || fast >= slow {
                    return None;
                }

                Some(Box::new(
                    alphafield_strategy::strategies::trend_following::MACrossoverStrategy::new(
                        fast, slow,
                    ),
                ))
            }
            "AdaptiveMA" => {
                let fast = params.get("fast_period").copied().unwrap_or(10.0) as usize;
                let slow = params.get("slow_period").copied().unwrap_or(30.0) as usize;
                let price_period = params.get("price_period").copied().unwrap_or(10.0) as usize;
                let _tp = params.get("take_profit").copied().unwrap_or(5.0);
                let _sl = params.get("stop_loss").copied().unwrap_or(5.0);

                if fast == 0 || slow == 0 || price_period == 0 {
                    return None;
                }

                Some(Box::new(
                    alphafield_strategy::strategies::trend_following::AdaptiveMAStrategy::new(
                        fast,
                        slow,
                        price_period,
                    ),
                ))
            }
            "TripleMA" => {
                let fast = params.get("fast_period").copied().unwrap_or(5.0) as usize;
                let medium = params.get("medium_period").copied().unwrap_or(15.0) as usize;
                let slow = params.get("slow_period").copied().unwrap_or(30.0) as usize;
                let _tp = params.get("take_profit").copied().unwrap_or(5.0);
                let _sl = params.get("stop_loss").copied().unwrap_or(5.0);

                if fast == 0 || medium == 0 || slow == 0 {
                    return None;
                }
                if !(fast < medium && medium < slow) {
                    return None;
                }

                Some(Box::new(
                    alphafield_strategy::strategies::trend_following::TripleMAStrategy::new(
                        fast, medium, slow,
                    ),
                ))
            }
            "MacdTrend" => {
                let fast = params.get("fast_period").copied().unwrap_or(12.0) as usize;
                let slow = params.get("slow_period").copied().unwrap_or(26.0) as usize;
                let signal = params.get("signal_period").copied().unwrap_or(9.0) as usize;
                let _tp = params.get("take_profit").copied().unwrap_or(5.0);
                let _sl = params.get("stop_loss").copied().unwrap_or(5.0);

                if fast == 0 || slow == 0 || signal == 0 || fast >= slow {
                    return None;
                }

                Some(Box::new(
                    alphafield_strategy::strategies::trend_following::MacdTrendStrategy::new(
                        fast, slow, signal,
                    ),
                ))
            }
            "ParabolicSAR" => {
                let af_step = params.get("af_step").copied().unwrap_or(0.02);
                let af_max = params.get("af_max").copied().unwrap_or(0.2);
                let _tp = params.get("take_profit").copied().unwrap_or(5.0);
                let _sl = params.get("stop_loss").copied().unwrap_or(5.0);

                if af_step <= 0.0 || af_max <= 0.0 || af_step > af_max {
                    return None;
                }

                Some(Box::new(
                    alphafield_strategy::strategies::trend_following::ParabolicSARStrategy::new(
                        af_step, af_max,
                    ),
                ))
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

                let config =
                    alphafield_strategy::strategies::mean_reversion::BollingerBandsConfig {
                        period,
                        num_std_dev: std_dev,
                        rsi_period: 14,
                        rsi_oversold: 30.0,
                        rsi_overbought: 70.0,
                        take_profit: tp,
                        stop_loss: sl,
                    };
                Some(Box::new(BollingerBandsStrategy::from_config(config)))
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
        let name = canonicalize_strategy_name(name);
        debug!(strategy = name, ?params, "Creating backtest strategy");
        match name.as_str() {
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
            "Breakout" => {
                // NOTE: Breakout strategy limitation
                // The BreakoutStrategy supports multi-level take profits (tp1_pct, tp2_pct, tp3_pct)
                // with configurable partial position exits. However, dashboard API currently only
                // exposes three parameters: lookback, take_profit, and stop_loss.
                //
                // The BreakoutStrategy::new() constructor uses a default config with hardcoded
                // TP levels: 3%, 6%, and 10%, closing 30%, 40%, and 30% of position respectively.
                //
                // Future enhancement: Update dashboard UI and StrategyFactory to accept additional
                // TP parameters and use BreakoutStrategy::from_config() for full customization.
                let lookback = params.get("lookback").copied().unwrap_or(20.0) as usize;

                if lookback == 0 {
                    return None;
                }

                let strat = alphafield_strategy::strategies::trend_following::BreakoutStrategy::new(
                    lookback,
                );
                Some(Box::new(StrategyAdapter::new(strat, symbol, capital)))
            }
            "MACrossover" => {
                let fast = params.get("fast_period").copied().unwrap_or(10.0) as usize;
                let slow = params.get("slow_period").copied().unwrap_or(30.0) as usize;

                if fast == 0 || slow == 0 || fast >= slow {
                    return None;
                }

                let strat =
                    alphafield_strategy::strategies::trend_following::MACrossoverStrategy::new(
                        fast, slow,
                    );
                Some(Box::new(StrategyAdapter::new(strat, symbol, capital)))
            }
            "AdaptiveMA" => {
                let fast = params.get("fast_period").copied().unwrap_or(10.0) as usize;
                let slow = params.get("slow_period").copied().unwrap_or(30.0) as usize;
                let price_period = params.get("price_period").copied().unwrap_or(10.0) as usize;

                if fast == 0 || slow == 0 || price_period == 0 {
                    return None;
                }

                let strat =
                    alphafield_strategy::strategies::trend_following::AdaptiveMAStrategy::new(
                        fast,
                        slow,
                        price_period,
                    );
                Some(Box::new(StrategyAdapter::new(strat, symbol, capital)))
            }
            "TripleMA" => {
                let fast = params.get("fast_period").copied().unwrap_or(5.0) as usize;
                let medium = params.get("medium_period").copied().unwrap_or(15.0) as usize;
                let slow = params.get("slow_period").copied().unwrap_or(30.0) as usize;

                if fast == 0 || medium == 0 || slow == 0 {
                    return None;
                }
                if !(fast < medium && medium < slow) {
                    return None;
                }

                let strat = alphafield_strategy::strategies::trend_following::TripleMAStrategy::new(
                    fast, medium, slow,
                );
                Some(Box::new(StrategyAdapter::new(strat, symbol, capital)))
            }
            "MacdTrend" => {
                let fast = params.get("fast_period").copied().unwrap_or(12.0) as usize;
                let slow = params.get("slow_period").copied().unwrap_or(26.0) as usize;
                let signal = params.get("signal_period").copied().unwrap_or(9.0) as usize;

                if fast == 0 || slow == 0 || signal == 0 || fast >= slow {
                    return None;
                }

                let strat =
                    alphafield_strategy::strategies::trend_following::MacdTrendStrategy::new(
                        fast, slow, signal,
                    );
                Some(Box::new(StrategyAdapter::new(strat, symbol, capital)))
            }
            "ParabolicSAR" => {
                let af_step = params.get("af_step").copied().unwrap_or(0.02);
                let af_max = params.get("af_max").copied().unwrap_or(0.2);

                if af_step <= 0.0 || af_max <= 0.0 || af_step > af_max {
                    return None;
                }

                let strat =
                    alphafield_strategy::strategies::trend_following::ParabolicSARStrategy::new(
                        af_step, af_max,
                    );
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
                let config =
                    alphafield_strategy::strategies::mean_reversion::BollingerBandsConfig {
                        period,
                        num_std_dev: std_dev,
                        rsi_period: 14,
                        rsi_oversold: 30.0,
                        rsi_overbought: 70.0,
                        take_profit: tp,
                        stop_loss: sl,
                    };
                let strat = BollingerBandsStrategy::from_config(config);
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
