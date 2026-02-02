use alphafield_backtest::{strategy::Strategy as BacktestStrategy, StrategyAdapter};
use alphafield_core::Strategy;
use alphafield_strategy::{
    framework::canonicalize_strategy_name, macd_strategy::MomentumConfig,
    momentum_factor::MomentumFactorConfig, BollingerBandsStrategy, GoldenCrossStrategy,
    MACDStrategy,
};
// Volatility strategy configs
use alphafield_strategy::strategies::volatility::atr_breakout::ATRBreakoutConfig;
use alphafield_strategy::strategies::volatility::atr_trailing::ATRTrailingConfig;
use alphafield_strategy::strategies::volatility::garch_strategy::GARCHConfig;
use alphafield_strategy::strategies::volatility::vix_style::VIXStyleConfig;
use alphafield_strategy::strategies::volatility::vol_sizing::VolSizingConfig;
use alphafield_strategy::strategies::volatility::vol_squeeze::VolSqueezeConfig;
// Multi-indicator strategy configs
use alphafield_strategy::strategies::multi_indicator::adaptive_combo::AdaptiveComboConfig;
use alphafield_strategy::strategies::multi_indicator::confidence_weighted::ConfidenceWeightedConfig;
use alphafield_strategy::strategies::multi_indicator::ensemble_weighted::EnsembleWeightedConfig;
use alphafield_strategy::strategies::multi_indicator::macd_rsi_combo::MACDRSIConfig;
use alphafield_strategy::strategies::multi_indicator::ml_enhanced::{
    FeatureWeights, MLEnhancedConfig,
};
use alphafield_strategy::strategies::multi_indicator::regime_switching::RegimeSwitchingConfig;
use alphafield_strategy::strategies::multi_indicator::trend_mean_rev::TrendMeanRevConfig;
// Sentiment strategy configs
use alphafield_strategy::strategies::sentiment::divergence_strategy::DivergenceConfig;
use alphafield_strategy::strategies::sentiment::regime_sentiment::RegimeSentimentConfig;
use alphafield_strategy::strategies::sentiment::sentiment_momentum::SentimentMomentumConfig;
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
                let _tp = params.get("take_profit").copied().unwrap_or(3.0);
                let _sl = params.get("stop_loss").copied().unwrap_or(5.0);

                if period == 0 || lower >= upper {
                    debug!(
                        "Invalid RSI params: period={} lower={} upper={}",
                        period, lower, upper
                    );
                    return None;
                }

                let config = alphafield_strategy::strategies::mean_reversion::RSIReversionConfig {
                    rsi_period: period,
                    oversold_threshold: lower,
                    overbought_threshold: upper,
                    exit_threshold: 50.0,
                    trend_filter: true,
                    trend_period: 200,
                    stop_loss: _sl,
                };
                config
                    .validate()
                    .map_err(|e| {
                        debug!("Invalid RSI config: {}", e);
                        e
                    })
                    .ok()?;
                Some(Box::new(
                    alphafield_strategy::strategies::mean_reversion::RSIReversionStrategy::from_config(config),
                ))
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
            "BollingerBands" => {
                let period = params.get("period").copied().unwrap_or(20.0) as usize;
                let std_dev = params.get("std_dev").copied().unwrap_or(2.0);
                let tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(5.0);

                if period == 0 || std_dev <= 0.0 {
                    debug!(
                        "Invalid BollingerBands params: period={} std_dev={}",
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
            "RSIReversion" => {
                let period = params.get("period").copied().unwrap_or(14.0) as usize;
                let lower = params.get("lower_bound").copied().unwrap_or(30.0);
                let upper = params.get("upper_bound").copied().unwrap_or(70.0);
                let _tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(5.0);

                if period == 0 || lower >= upper {
                    debug!(
                        "Invalid RSIReversion params: period={} lower={} upper={}",
                        period, lower, upper
                    );
                    return None;
                }

                let config = alphafield_strategy::strategies::mean_reversion::RSIReversionConfig {
                    rsi_period: period,
                    oversold_threshold: lower,
                    overbought_threshold: upper,
                    exit_threshold: 50.0,
                    trend_filter: true,
                    trend_period: 200,
                    stop_loss: sl,
                };
                Some(Box::new(
                    alphafield_strategy::strategies::mean_reversion::RSIReversionStrategy::from_config(config),
                ))
            }
            "StochReversion" => {
                let k_period = params.get("k_period").copied().unwrap_or(14.0) as usize;
                let d_period = params.get("d_period").copied().unwrap_or(3.0) as usize;
                let lower = params.get("lower_bound").copied().unwrap_or(20.0);
                let upper = params.get("upper_bound").copied().unwrap_or(80.0);
                let sl = params.get("stop_loss").copied().unwrap_or(5.0);

                if k_period == 0 || d_period == 0 || lower >= upper {
                    debug!(
                        "Invalid StochReversion params: k_period={} d_period={} lower={} upper={}",
                        k_period, d_period, lower, upper
                    );
                    return None;
                }

                let config =
                    alphafield_strategy::strategies::mean_reversion::StochReversionConfig {
                        k_period,
                        d_period,
                        smooth_period: 3,
                        oversold: lower,
                        overbought: upper,
                        stop_loss: sl,
                    };
                Some(Box::new(
                    alphafield_strategy::strategies::mean_reversion::StochReversionStrategy::from_config(config),
                ))
            }
            "ZScoreReversion" => {
                let lookback = params.get("period").copied().unwrap_or(20.0) as usize;
                let entry_zscore = params.get("entry_threshold").copied().unwrap_or(-2.0);
                let exit_zscore = params.get("exit_threshold").copied().unwrap_or(0.0);
                let sl = params.get("stop_loss").copied().unwrap_or(5.0);

                if lookback == 0 || entry_zscore >= 0.0 {
                    debug!(
                        "Invalid ZScoreReversion params: lookback={} entry_zscore={}",
                        lookback, entry_zscore
                    );
                    return None;
                }

                let config =
                    alphafield_strategy::strategies::mean_reversion::ZScoreReversionConfig {
                        lookback_period: lookback,
                        entry_zscore,
                        exit_zscore,
                        min_price_change: 1.0,
                        stop_loss: sl,
                    };
                Some(Box::new(
                    alphafield_strategy::strategies::mean_reversion::ZScoreReversionStrategy::from_config(config),
                ))
            }
            "PriceChannel" => {
                let lookback = params.get("period").copied().unwrap_or(20.0) as usize;
                let tp = params.get("take_profit").copied().unwrap_or(50.0);
                let sl = params.get("stop_loss").copied().unwrap_or(5.0);

                if lookback == 0 {
                    debug!("Invalid PriceChannel params: lookback={}", lookback);
                    return None;
                }

                let config = alphafield_strategy::strategies::mean_reversion::PriceChannelConfig {
                    lookback_period: lookback,
                    exit_percent: tp,
                    stop_loss: sl,
                };
                Some(Box::new(
                    alphafield_strategy::strategies::mean_reversion::PriceChannelStrategy::from_config(config),
                ))
            }
            "KeltnerReversion" => {
                let ema_period = params.get("period").copied().unwrap_or(20.0) as usize;
                let atr_multiplier = params.get("multiplier").copied().unwrap_or(2.0);
                let _tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(5.0);

                if ema_period == 0 || atr_multiplier <= 0.0 {
                    debug!(
                        "Invalid KeltnerReversion params: ema_period={} atr_multiplier={}",
                        ema_period, atr_multiplier
                    );
                    return None;
                }

                let config =
                    alphafield_strategy::strategies::mean_reversion::KeltnerReversionConfig {
                        ema_period,
                        atr_period: 10,
                        atr_multiplier,
                        volume_multiplier: 1.5,
                        stop_loss: sl,
                    };
                Some(Box::new(
                    alphafield_strategy::strategies::mean_reversion::KeltnerReversionStrategy::from_config(config),
                ))
            }
            "StatArb" => {
                let lookback = params.get("lookback").copied().unwrap_or(60.0) as usize;
                let entry_zscore = params.get("entry_threshold").copied().unwrap_or(2.0);
                let exit_zscore = params.get("exit_threshold").copied().unwrap_or(0.0);
                let sl = params.get("stop_loss").copied().unwrap_or(5.0);

                if lookback == 0 || entry_zscore <= 0.0 {
                    debug!(
                        "Invalid StatArb params: lookback={} entry_zscore={}",
                        lookback, entry_zscore
                    );
                    return None;
                }

                let config = alphafield_strategy::strategies::mean_reversion::StatArbConfig {
                    lookback_period: lookback,
                    entry_zscore,
                    exit_zscore,
                    min_correlation: 0.7,
                    stop_loss: sl,
                };
                Some(Box::new(
                    alphafield_strategy::strategies::mean_reversion::StatArbStrategy::from_config(
                        config,
                    ),
                ))
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

                let config = MomentumConfig::new_with_exits(
                    ema_period,
                    macd_fast,
                    macd_slow,
                    macd_signal,
                    tp,
                    sl,
                );
                Some(Box::new(MACDStrategy::from_config(config)))
            }
            "MACDStrategy" => {
                let ema_period = params.get("ema_period").copied().unwrap_or(50.0) as usize;
                let macd_fast = params.get("fast_period").copied().unwrap_or(12.0) as usize;
                let macd_slow = params.get("slow_period").copied().unwrap_or(26.0) as usize;
                let macd_signal = params.get("signal_period").copied().unwrap_or(9.0) as usize;
                let tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(3.0);

                if macd_fast >= macd_slow
                    || macd_fast == 0
                    || macd_slow == 0
                    || macd_signal == 0
                    || ema_period == 0
                {
                    debug!(
                        "Invalid MACDStrategy params: ema={} fast={} slow={} signal={}",
                        ema_period, macd_fast, macd_slow, macd_signal
                    );
                    return None;
                }

                let config = MomentumConfig::new_with_exits(
                    ema_period,
                    macd_fast,
                    macd_slow,
                    macd_signal,
                    tp,
                    sl,
                );
                Some(Box::new(
                    alphafield_strategy::strategies::momentum::MACDStrategy::from_config(config),
                ))
            }
            "RsiMomentumStrategy" => {
                let rsi_period = params.get("period").copied().unwrap_or(14.0) as usize;
                let threshold = params.get("threshold").copied().unwrap_or(55.0);
                let tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(3.0);

                if rsi_period == 0 || threshold <= 0.0 || threshold >= 100.0 {
                    debug!(
                        "Invalid RsiMomentumStrategy params: period={} threshold={}",
                        rsi_period, threshold
                    );
                    return None;
                }

                let strategy = alphafield_strategy::strategies::momentum::RsiMomentumStrategy::new(
                    rsi_period, threshold, 1.0, tp, sl,
                );
                Some(Box::new(strategy))
            }
            "RocStrategy" => {
                let roc_period = params.get("period").copied().unwrap_or(20.0) as usize;
                let threshold = params.get("threshold").copied().unwrap_or(1.0);
                let tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(3.0);

                if roc_period == 0 || threshold <= 0.0 {
                    debug!(
                        "Invalid RocStrategy params: period={} threshold={}",
                        roc_period, threshold
                    );
                    return None;
                }

                let strategy = alphafield_strategy::strategies::momentum::RocStrategy::new(
                    roc_period, threshold, 0.0, tp, sl,
                );
                Some(Box::new(strategy))
            }
            "AdxTrendStrategy" => {
                let adx_period = params.get("period").copied().unwrap_or(14.0) as usize;
                let threshold = params.get("threshold").copied().unwrap_or(25.0);
                let tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(3.0);

                if adx_period == 0 || threshold <= 0.0 {
                    debug!(
                        "Invalid AdxTrendStrategy params: period={} threshold={}",
                        adx_period, threshold
                    );
                    return None;
                }

                let strategy = alphafield_strategy::strategies::momentum::AdxTrendStrategy::new(
                    adx_period, threshold, 10.0, tp, sl,
                );
                Some(Box::new(strategy))
            }
            "MomentumFactorStrategy" => {
                let lookback = params.get("lookback").copied().unwrap_or(40.0) as usize;
                let _formation = params.get("formation_period").copied().unwrap_or(90.0) as usize;
                let _skip = params.get("skip_period").copied().unwrap_or(30.0) as usize;
                let tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(3.0);

                if lookback == 0 {
                    debug!(
                        "Invalid MomentumFactorStrategy params: lookback={}",
                        lookback
                    );
                    return None;
                }

                let config = MomentumFactorConfig::new(lookback, 14, 2, tp, sl);
                Some(Box::new(
                    alphafield_strategy::strategies::momentum::MomentumFactorStrategy::from_config(
                        config,
                    ),
                ))
            }
            "VolumeMomentumStrategy" => {
                let price_period = params.get("price_period").copied().unwrap_or(20.0) as usize;
                let volume_period = params.get("volume_period").copied().unwrap_or(20.0) as usize;
                let volume_threshold = params.get("volume_threshold").copied().unwrap_or(1.6);
                let tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(3.0);

                if price_period == 0 || volume_period == 0 || volume_threshold <= 0.0 {
                    debug!(
                        "Invalid VolumeMomentumStrategy params: price_period={} volume_period={} threshold={}",
                        price_period, volume_period, volume_threshold
                    );
                    return None;
                }

                let strategy =
                    alphafield_strategy::strategies::momentum::VolumeMomentumStrategy::new(
                        price_period,
                        volume_period,
                        volume_threshold,
                        tp,
                        sl,
                    );
                Some(Box::new(strategy))
            }
            "MultiTfMomentumStrategy" => {
                let fast_ema = params.get("fast_ema").copied().unwrap_or(10.0) as usize;
                let slow_ema = params.get("slow_ema").copied().unwrap_or(75.0) as usize;
                let tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(3.0);

                if fast_ema == 0 || slow_ema == 0 {
                    debug!(
                        "Invalid MultiTfMomentumStrategy params: fast={} slow={}",
                        fast_ema, slow_ema
                    );
                    return None;
                }
                if fast_ema >= slow_ema {
                    debug!(
                        "Invalid MultiTfMomentumStrategy ordering: fast={} must be < slow={}",
                        fast_ema, slow_ema
                    );
                    return None;
                }

                let strategy =
                    alphafield_strategy::strategies::momentum::MultiTfMomentumStrategy::new(
                        fast_ema, slow_ema, tp, sl,
                    );
                Some(Box::new(strategy))
            }

            "ATRBreakout" => {
                let atr_period = params.get("atr_period").copied().unwrap_or(14.0) as usize;
                let atr_multiplier = params.get("atr_multiplier").copied().unwrap_or(2.0);
                let lookback = params.get("lookback_period").copied().unwrap_or(20.0) as usize;
                let sma_period = params.get("sma_period").copied().unwrap_or(50.0) as usize;
                let tp = params.get("take_profit").copied().unwrap_or(5.0);
                let sl = params.get("stop_loss").copied().unwrap_or(3.0);

                if atr_period == 0 || lookback == 0 || sma_period == 0 || atr_multiplier <= 0.0 {
                    debug!(
                        "Invalid ATRBreakout params: atr_period={} lookback={} sma={} mult={}",
                        atr_period, lookback, sma_period, atr_multiplier
                    );
                    return None;
                }

                let config = ATRBreakoutConfig {
                    atr_period,
                    atr_multiplier,
                    lookback_period: lookback,
                    trend_ma_period: sma_period,
                    volume_multiplier: 1.5,
                    take_profit: tp,
                    stop_loss: sl,
                };
                Some(Box::new(
                    alphafield_strategy::strategies::volatility::ATRBreakoutStrategy::from_config(
                        config,
                    ),
                ))
            }

            "ATRTrailingStop" => {
                let atr_period = params.get("atr_period").copied().unwrap_or(14.0) as usize;
                let atr_multiplier = params.get("atr_multiplier").copied().unwrap_or(2.0);
                let fast = params.get("fast_period").copied().unwrap_or(12.0) as usize;
                let slow = params.get("slow_period").copied().unwrap_or(26.0) as usize;
                let tp = params.get("take_profit").copied().unwrap_or(5.0);

                if atr_period == 0
                    || fast == 0
                    || slow == 0
                    || atr_multiplier <= 0.0
                    || fast >= slow
                {
                    debug!(
                        "Invalid ATRTrailingStop params: atr={} fast={} slow={} mult={}",
                        atr_period, fast, slow, atr_multiplier
                    );
                    return None;
                }

                let config = ATRTrailingConfig {
                    atr_period,
                    atr_multiplier,
                    fast_period: fast,
                    slow_period: slow,
                    min_trailing_pct: 1.0,
                    take_profit: tp,
                };
                Some(Box::new(
                    alphafield_strategy::strategies::volatility::ATRTrailingStrategy::from_config(
                        config,
                    ),
                ))
            }

            "VolatilitySqueeze" => {
                let bb_period = params.get("bb_period").copied().unwrap_or(20.0) as usize;
                let bb_std = params.get("bb_std").copied().unwrap_or(2.0);
                let kc_period = params.get("kc_period").copied().unwrap_or(20.0) as usize;
                let kc_mult = params.get("kc_mult").copied().unwrap_or(1.5);
                let tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(3.0);

                if bb_period == 0 || kc_period == 0 || bb_std <= 0.0 || kc_mult <= 0.0 {
                    debug!(
                        "Invalid VolatilitySqueeze params: bb={} bb_std={} kc={} kc_mult={}",
                        bb_period, bb_std, kc_period, kc_mult
                    );
                    return None;
                }

                let config = VolSqueezeConfig {
                    bb_period,
                    bb_std_dev: bb_std,
                    kk_period: kc_period,
                    kk_mult: kc_mult,
                    squeeze_threshold: 0.5,
                    volume_multiplier: 1.5,
                    take_profit: tp,
                    stop_loss: sl,
                };
                Some(Box::new(
                    alphafield_strategy::strategies::volatility::VolSqueezeStrategy::from_config(
                        config,
                    ),
                ))
            }
            "VolatilitySizing" => {
                let atr_period = params.get("atr_period").copied().unwrap_or(14.0) as usize;
                let baseline_period =
                    params.get("baseline_period").copied().unwrap_or(100.0) as usize;
                let risk_per_trade = params.get("risk_per_trade").copied().unwrap_or(1.5);
                let tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(3.0);

                if atr_period == 0 || baseline_period == 0 || risk_per_trade <= 0.0 {
                    debug!(
                        "Invalid VolSizingStrategy params: atr={} baseline={} risk={}",
                        atr_period, baseline_period, risk_per_trade
                    );
                    return None;
                }

                let config = VolSizingConfig {
                    atr_period,
                    base_size_pct: risk_per_trade,
                    min_size_pct: 1.0,
                    max_size_pct: 25.0,
                    vol_scaling_factor: 2.0,
                    baseline_period,
                    fast_period: 12,
                    slow_period: 26,
                    take_profit: tp,
                    stop_loss: sl,
                };
                Some(Box::new(
                    alphafield_strategy::strategies::volatility::VolSizingStrategy::from_config(
                        config,
                    ),
                ))
            }
            "GarchStrategy" => {
                let lambda = params.get("lambda").copied().unwrap_or(0.94);
                let return_window = params.get("return_window").copied().unwrap_or(20.0) as usize;
                let vol_threshold = params.get("volatility_threshold").copied().unwrap_or(2.0);
                let tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(3.0);

                if !(0.0..1.0).contains(&lambda) || return_window == 0 || vol_threshold <= 0.0 {
                    debug!(
                        "Invalid GarchStrategy params: lambda={} window={} threshold={}",
                        lambda, return_window, vol_threshold
                    );
                    return None;
                }

                let config = GARCHConfig {
                    lambda,
                    return_window,
                    fast_period: 12,
                    slow_period: 26,
                    vol_entry_threshold: vol_threshold,
                    vol_exit_multiplier: 0.5,
                    take_profit: tp,
                    stop_loss: sl,
                };
                Some(Box::new(
                    alphafield_strategy::strategies::volatility::GARCHStrategy::from_config(config),
                ))
            }

            "VIXStyleStrategy" => {
                let atr_period = params.get("atr_period").copied().unwrap_or(14.0) as usize;
                let lookback = params.get("lookback").copied().unwrap_or(20.0) as usize;
                let high_threshold = params.get("high_threshold").copied().unwrap_or(30.0);
                let low_threshold = params.get("low_threshold").copied().unwrap_or(20.0);
                let tp = params.get("take_profit").copied().unwrap_or(5.0);
                let sl = params.get("stop_loss").copied().unwrap_or(3.0);

                if atr_period == 0 || lookback == 0 || high_threshold <= low_threshold {
                    debug!(
                        "Invalid VIXStyleStrategy params: atr={} lookback={} high={} low={}",
                        atr_period, lookback, high_threshold, low_threshold
                    );
                    return None;
                }

                let config = VIXStyleConfig {
                    atr_period,
                    lookback_period: lookback,
                    extreme_fear_threshold: high_threshold,
                    extreme_greed_threshold: low_threshold,
                    volume_multiplier: 1.5,
                    take_profit: tp,
                    stop_loss: sl,
                };
                Some(Box::new(
                    alphafield_strategy::strategies::volatility::VIXStyleStrategy::from_config(
                        config,
                    ),
                ))
            }

            "TrendMeanRev" => {
                let ema_fast = params.get("ema_fast").copied().unwrap_or(10.0) as usize;
                let ema_slow = params.get("ema_slow").copied().unwrap_or(30.0) as usize;
                let rsi_period = params.get("rsi_period").copied().unwrap_or(14.0) as usize;
                let rsi_oversold = params.get("rsi_oversold").copied().unwrap_or(30.0);
                let rsi_overbought = params.get("rsi_overbought").copied().unwrap_or(70.0);
                let tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(3.0);

                if ema_fast == 0 || ema_slow == 0 || rsi_period == 0 || ema_fast >= ema_slow {
                    debug!(
                        "Invalid TrendMeanRev params: ema_fast={} ema_slow={} rsi={}",
                        ema_fast, ema_slow, rsi_period
                    );
                    return None;
                }

                let config = TrendMeanRevConfig {
                    ema_fast,
                    ema_slow,
                    rsi_period,
                    rsi_oversold,
                    rsi_overbought,
                    take_profit: tp,
                    stop_loss: sl,
                };
                Some(Box::new(
                    alphafield_strategy::strategies::multi_indicator::TrendMeanRevStrategy::from_config(config),
                ))
            }
            "MACDRSICombo" => {
                let macd_fast = params.get("macd_fast").copied().unwrap_or(12.0) as usize;
                let macd_slow = params.get("macd_slow").copied().unwrap_or(26.0) as usize;
                let macd_signal = params.get("macd_signal").copied().unwrap_or(9.0) as usize;
                let rsi_period = params.get("rsi_period").copied().unwrap_or(14.0) as usize;
                let rsi_overbought = params.get("rsi_overbought").copied().unwrap_or(70.0);
                let rsi_oversold = params.get("rsi_oversold").copied().unwrap_or(30.0);
                let tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(3.0);

                if macd_fast == 0
                    || macd_slow == 0
                    || macd_signal == 0
                    || rsi_period == 0
                    || macd_fast >= macd_slow
                {
                    debug!(
                        "Invalid MACDRSICombo params: macd_fast={} macd_slow={} rsi={}",
                        macd_fast, macd_slow, rsi_period
                    );
                    return None;
                }

                let config = MACDRSIConfig {
                    macd_fast,
                    macd_slow,
                    macd_signal,
                    rsi_period,
                    rsi_overbought,
                    rsi_oversold,
                    take_profit: tp,
                    stop_loss: sl,
                };
                Some(Box::new(
                    alphafield_strategy::strategies::multi_indicator::MACDRSIComboStrategy::from_config(config),
                ))
            }
            "AdaptiveCombo" => {
                let ema_fast = params.get("ema_fast").copied().unwrap_or(10.0) as usize;
                let ema_slow = params.get("ema_slow").copied().unwrap_or(30.0) as usize;
                let macd_fast = params.get("macd_fast").copied().unwrap_or(12.0) as usize;
                let macd_slow = params.get("macd_slow").copied().unwrap_or(26.0) as usize;
                let macd_signal = params.get("macd_signal").copied().unwrap_or(9.0) as usize;
                let rsi_period = params.get("rsi_period").copied().unwrap_or(14.0) as usize;
                let rsi_overbought = params.get("rsi_overbought").copied().unwrap_or(70.0);
                let rsi_oversold = params.get("rsi_oversold").copied().unwrap_or(30.0);
                let performance_lookback =
                    params.get("performance_lookback").copied().unwrap_or(10.0) as usize;
                let min_weight = params.get("min_weight").copied().unwrap_or(0.1);
                let tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(3.0);

                if ema_fast == 0
                    || ema_slow == 0
                    || macd_fast == 0
                    || macd_slow == 0
                    || rsi_period == 0
                {
                    debug!("Invalid AdaptiveCombo params: zero periods");
                    return None;
                }

                let config = AdaptiveComboConfig {
                    ema_fast,
                    ema_slow,
                    macd_fast,
                    macd_slow,
                    macd_signal,
                    rsi_period,
                    rsi_overbought,
                    rsi_oversold,
                    performance_lookback,
                    min_weight,
                    take_profit: tp,
                    stop_loss: sl,
                };
                Some(Box::new(
                    alphafield_strategy::strategies::multi_indicator::AdaptiveComboStrategy::from_config(config),
                ))
            }
            "ConfidenceWeighted" => {
                let ema_fast = params.get("ema_fast").copied().unwrap_or(10.0) as usize;
                let ema_slow = params.get("ema_slow").copied().unwrap_or(30.0) as usize;
                let macd_fast = params.get("macd_fast").copied().unwrap_or(12.0) as usize;
                let macd_slow = params.get("macd_slow").copied().unwrap_or(26.0) as usize;
                let macd_signal = params.get("macd_signal").copied().unwrap_or(9.0) as usize;
                let rsi_period = params.get("rsi_period").copied().unwrap_or(14.0) as usize;
                let rsi_overbought = params.get("rsi_overbought").copied().unwrap_or(70.0);
                let rsi_oversold = params.get("rsi_oversold").copied().unwrap_or(30.0);
                let min_confidence = params.get("min_confidence").copied().unwrap_or(0.5);
                let tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(3.0);

                if ema_fast == 0
                    || ema_slow == 0
                    || macd_fast == 0
                    || macd_slow == 0
                    || rsi_period == 0
                {
                    debug!("Invalid ConfidenceWeighted params: zero periods");
                    return None;
                }

                let config = ConfidenceWeightedConfig {
                    ema_fast,
                    ema_slow,
                    macd_fast,
                    macd_slow,
                    macd_signal,
                    rsi_period,
                    rsi_overbought,
                    rsi_oversold,
                    min_confidence,
                    take_profit: tp,
                    stop_loss: sl,
                };
                Some(Box::new(
                    alphafield_strategy::strategies::multi_indicator::ConfidenceWeightedStrategy::from_config(config),
                ))
            }
            "EnsembleWeighted" => {
                let performance_lookback =
                    params.get("performance_lookback").copied().unwrap_or(10.0) as usize;
                let min_weight = params.get("min_weight").copied().unwrap_or(0.1);
                let weight_smoothing = params.get("weight_smoothing").copied().unwrap_or(0.3);
                let tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(3.0);

                if performance_lookback == 0 || min_weight <= 0.0 || weight_smoothing <= 0.0 {
                    debug!(
                        "Invalid EnsembleWeighted params: lookback={} min_weight={} smoothing={}",
                        performance_lookback, min_weight, weight_smoothing
                    );
                    return None;
                }

                let config = EnsembleWeightedConfig {
                    performance_lookback,
                    min_weight,
                    weight_smoothing,
                    take_profit: tp,
                    stop_loss: sl,
                    num_strategies: 3,
                };
                Some(Box::new(
                    alphafield_strategy::strategies::multi_indicator::EnsembleWeightedStrategy::from_config(config),
                ))
            }
            "MLEnhanced" => {
                let ema_fast = params.get("ema_fast").copied().unwrap_or(10.0) as usize;
                let ema_slow = params.get("ema_slow").copied().unwrap_or(30.0) as usize;
                let macd_fast = params.get("macd_fast").copied().unwrap_or(12.0) as usize;
                let macd_slow = params.get("macd_slow").copied().unwrap_or(26.0) as usize;
                let macd_signal = params.get("macd_signal").copied().unwrap_or(9.0) as usize;
                let rsi_period = params.get("rsi_period").copied().unwrap_or(14.0) as usize;
                let atr_period = params.get("atr_period").copied().unwrap_or(14.0) as usize;
                let trend_weight = params.get("trend_weight").copied().unwrap_or(0.3);
                let momentum_weight = params.get("momentum_weight").copied().unwrap_or(0.35);
                let meanrev_weight = params.get("meanrev_weight").copied().unwrap_or(0.2);
                let volatility_weight = params.get("volatility_weight").copied().unwrap_or(0.15);
                let min_prediction_score =
                    params.get("min_prediction_score").copied().unwrap_or(0.6);
                let tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(3.0);

                if ema_fast == 0
                    || ema_slow == 0
                    || macd_fast == 0
                    || macd_slow == 0
                    || rsi_period == 0
                    || atr_period == 0
                {
                    debug!("Invalid MLEnhanced params: zero periods");
                    return None;
                }

                let config = MLEnhancedConfig {
                    ema_fast,
                    ema_slow,
                    macd_fast,
                    macd_slow,
                    macd_signal,
                    rsi_period,
                    atr_period,
                    feature_weights: FeatureWeights {
                        trend_weight,
                        momentum_weight,
                        meanrev_weight,
                        volatility_weight,
                    },
                    min_prediction_score,
                    take_profit: tp,
                    stop_loss: sl,
                };
                Some(Box::new(
                    alphafield_strategy::strategies::multi_indicator::MLEnhancedStrategy::from_config(config),
                ))
            }
            "RegimeSwitching" => {
                let ema_fast = params.get("ema_fast").copied().unwrap_or(10.0) as usize;
                let ema_slow = params.get("ema_slow").copied().unwrap_or(30.0) as usize;
                let atr_period = params.get("atr_period").copied().unwrap_or(14.0) as usize;
                let trend_threshold = params.get("trend_threshold").copied().unwrap_or(0.02);
                let volatility_threshold =
                    params.get("volatility_threshold").copied().unwrap_or(0.03);
                let rsi_period = params.get("rsi_period").copied().unwrap_or(14.0) as usize;
                let rsi_oversold = params.get("rsi_oversold").copied().unwrap_or(30.0);
                let rsi_overbought = params.get("rsi_overbought").copied().unwrap_or(70.0);
                let tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(3.0);

                if ema_fast == 0 || ema_slow == 0 || atr_period == 0 || rsi_period == 0 {
                    debug!("Invalid RegimeSwitching params: zero periods");
                    return None;
                }

                let config = RegimeSwitchingConfig {
                    ema_fast,
                    ema_slow,
                    atr_period,
                    trend_threshold,
                    volatility_threshold,
                    rsi_period,
                    rsi_oversold,
                    rsi_overbought,
                    take_profit: tp,
                    stop_loss: sl,
                };
                Some(Box::new(
                    alphafield_strategy::strategies::multi_indicator::RegimeSwitchingStrategy::from_config(config),
                ))
            }
            "Divergence" => {
                let price_lookback = params.get("price_lookback").copied().unwrap_or(20.0) as usize;
                let sentiment_lookback =
                    params.get("sentiment_lookback").copied().unwrap_or(15.0) as usize;
                let price_trend_threshold =
                    params.get("price_trend_threshold").copied().unwrap_or(3.5);
                let sentiment_trend_threshold = params
                    .get("sentiment_trend_threshold")
                    .copied()
                    .unwrap_or(5.0);
                let min_divergence_bars =
                    params.get("min_divergence_bars").copied().unwrap_or(3.0) as usize;
                let tp = params.get("take_profit").copied().unwrap_or(3.0);
                let sl = params.get("stop_loss").copied().unwrap_or(3.0);

                if price_lookback == 0 || sentiment_lookback == 0 || min_divergence_bars == 0 {
                    debug!(
                        "Invalid Divergence params: price_lookback={} sentiment_lookback={} min_bars={}",
                        price_lookback, sentiment_lookback, min_divergence_bars
                    );
                    return None;
                }

                let config = DivergenceConfig {
                    price_lookback,
                    sentiment_lookback,
                    price_trend_threshold,
                    sentiment_trend_threshold,
                    min_divergence_bars,
                    take_profit: tp,
                    stop_loss: sl,
                };
                Some(Box::new(
                    alphafield_strategy::strategies::sentiment::DivergenceStrategy::from_config(
                        config,
                    ),
                ))
            }
            "RegimeSentiment" => {
                let regime_lookback =
                    params.get("regime_lookback").copied().unwrap_or(20.0) as usize;
                let sentiment_lookback =
                    params.get("sentiment_lookback").copied().unwrap_or(15.0) as usize;
                let trend_threshold = params.get("trend_threshold").copied().unwrap_or(50.0);
                let volatility_threshold =
                    params.get("volatility_threshold").copied().unwrap_or(2.0);
                let bull_bullish_threshold = params
                    .get("bull_bullish_threshold")
                    .copied()
                    .unwrap_or(60.0);
                let bear_bullish_threshold = params
                    .get("bear_bullish_threshold")
                    .copied()
                    .unwrap_or(40.0);
                let sideways_bullish_threshold = params
                    .get("sideways_bullish_threshold")
                    .copied()
                    .unwrap_or(50.0);
                let momentum_threshold = params.get("momentum_threshold").copied().unwrap_or(5.0);

                if regime_lookback == 0 || sentiment_lookback == 0 {
                    debug!(
                        "Invalid RegimeSentiment params: regime={} sentiment={}",
                        regime_lookback, sentiment_lookback
                    );
                    return None;
                }

                let config = RegimeSentimentConfig {
                    regime_lookback,
                    sentiment_lookback,
                    trend_threshold,
                    volatility_threshold,
                    bull_bullish_threshold,
                    bear_bullish_threshold,
                    sideways_bullish_threshold,
                    momentum_threshold,
                };
                Some(Box::new(
                    alphafield_strategy::strategies::sentiment::RegimeSentimentStrategy::from_config(config),
                ))
            }
            "SentimentMomentum" => {
                let lookback_period =
                    params.get("lookback_period").copied().unwrap_or(15.0) as usize;
                let bullish_threshold = params.get("bullish_threshold").copied().unwrap_or(30.0);
                let bearish_threshold = params.get("bearish_threshold").copied().unwrap_or(70.0);
                let momentum_threshold = params.get("momentum_threshold").copied().unwrap_or(5.0);
                let volume_confirmation = params.get("volume_confirmation").copied().unwrap_or(1.5);

                if lookback_period == 0 || bullish_threshold >= bearish_threshold {
                    debug!(
                        "Invalid SentimentMomentum params: lookback={} bullish={} bearish={}",
                        lookback_period, bullish_threshold, bearish_threshold
                    );
                    return None;
                }

                let config = SentimentMomentumConfig {
                    lookback_period,
                    bullish_threshold,
                    bearish_threshold,
                    momentum_threshold,
                    volume_confirmation,
                };
                Some(Box::new(
                    alphafield_strategy::strategies::sentiment::SentimentMomentumStrategy::from_config(config),
                ))
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
        trading_mode: alphafield_core::TradingMode,
    ) -> Option<Box<dyn BacktestStrategy>> {
        debug!(
            strategy = name,
            ?params,
            ?trading_mode,
            "Creating backtest strategy"
        );
        // Use create() to instantiate the strategy, then wrap in StrategyAdapter
        Self::create(name, params).map(|strat| {
            let adapter =
                StrategyAdapter::new(strat, symbol, capital).with_trading_mode(trading_mode);
            Box::new(adapter) as Box<dyn BacktestStrategy>
        })
    }
}
