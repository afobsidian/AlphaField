//! Parabolic SAR Strategy
//!
//! A trend-following strategy using Parabolic SAR as both a trend indicator and trailing stop.
//!
//! # Strategy Logic
//! - **Entry (Long)**: Price crosses above SAR (SAR is below price) and (optional) price > SMA trend filter.
//! - **Exit (Long)**: Price crosses below SAR (SAR is above price) or SAR trailing stop is hit.
//!
//! Spot-only: this strategy never opens shorts.
//!
//! References:
//! - Wilder, J. Welles. "New Concepts in Technical Trading Systems" (Parabolic SAR)

use crate::framework::{
    CorrelationSensitivity, MarketRegime, MetadataStrategy, RiskProfile, StrategyCategory,
    StrategyMetadata, VolatilityLevel,
};
use crate::indicators::{Indicator, Sma};
use alphafield_core::{Bar, Signal, SignalType, Strategy};

/// Configuration for the Parabolic SAR strategy.
///
/// Kept local to this module to avoid widening `config.rs` scope in this change.
/// If Phase 12.2 requires central config registration, this can be promoted.
#[derive(Debug, Clone)]
pub struct ParabolicSarConfig {
    /// Acceleration factor step (commonly 0.02).
    pub step: f64,
    /// Maximum acceleration factor (commonly 0.2).
    pub max_step: f64,
    /// Whether to require SMA trend confirmation for entries.
    pub trend_filter_enabled: bool,
    /// SMA period used for trend filter (commonly 50).
    pub trend_sma_period: usize,
}

impl ParabolicSarConfig {
    pub fn new(step: f64, max_step: f64) -> Self {
        Self {
            step,
            max_step,
            trend_filter_enabled: true,
            trend_sma_period: 50,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if !self.step.is_finite() || self.step <= 0.0 {
            return Err("step must be > 0".to_string());
        }
        if !self.max_step.is_finite() || self.max_step <= 0.0 {
            return Err("max_step must be > 0".to_string());
        }
        if self.max_step < self.step {
            return Err("max_step must be >= step".to_string());
        }
        if self.trend_filter_enabled && self.trend_sma_period == 0 {
            return Err("trend_sma_period must be > 0 when trend filter is enabled".to_string());
        }
        Ok(())
    }

    pub fn strategy_name(&self) -> &'static str {
        "Parabolic SAR"
    }

    pub fn default_config() -> Self {
        Self {
            step: 0.02,
            max_step: 0.2,
            trend_filter_enabled: true,
            trend_sma_period: 50,
        }
    }
}

/// Internal Parabolic SAR state machine.
///
/// This is a minimal, self-contained SAR implementation intended for strategy usage.
/// It uses classic Wilder logic:
/// - On each bar: `sar = prior_sar + af * (ep - prior_sar)`
/// - Update EP and AF when a new extreme is made in the direction of the trend
/// - Reverse when price crosses SAR; reset AF to step and set EP to opposite extreme
#[derive(Debug, Clone)]
struct ParabolicSar {
    step: f64,
    max_step: f64,

    // Trend direction: true = uptrend (SAR below price), false = downtrend (SAR above price)
    uptrend: Option<bool>,

    // Acceleration factor
    af: f64,

    // Extreme point (highest high in uptrend, lowest low in downtrend)
    ep: Option<f64>,

    // Current SAR value
    sar: Option<f64>,

    // Prior highs/lows for banding logic (use previous two bars)
    prev_high: Option<f64>,
    prev_low: Option<f64>,
    prev_prev_high: Option<f64>,
    prev_prev_low: Option<f64>,
}

impl ParabolicSar {
    fn new(step: f64, max_step: f64) -> Self {
        Self {
            step,
            max_step,
            uptrend: None,
            af: step,
            ep: None,
            sar: None,
            prev_high: None,
            prev_low: None,
            prev_prev_high: None,
            prev_prev_low: None,
        }
    }

    /// Update SAR state with the next bar and return the updated SAR (if available).
    ///
    /// Note: we require enough history to define an initial trend and SAR.
    fn update(&mut self, bar: &Bar) -> Option<f64> {
        // Shift previous bar history
        self.prev_prev_high = self.prev_high;
        self.prev_prev_low = self.prev_low;
        self.prev_high = Some(bar.high);
        self.prev_low = Some(bar.low);

        // Need at least two completed bars of history to initialize properly.
        if self.prev_prev_high.is_none() || self.prev_prev_low.is_none() {
            return None;
        }

        // Initialize on first opportunity:
        // - Determine initial trend by comparing close to prior close midpoint heuristic.
        // - Set EP to current high/low depending on trend.
        // - Set SAR to prior low (uptrend) or prior high (downtrend).
        if self.uptrend.is_none() {
            // Choose trend direction based on whether the current close is above the prior close.
            // We only have current bar + previous bar; use previous bar close proxy:
            // Use (prev_high+prev_low)/2 as a rough prior close proxy to avoid extra stored close.
            let prev_mid = (self.prev_prev_high.unwrap() + self.prev_prev_low.unwrap()) / 2.0;
            let up = bar.close >= prev_mid;

            self.uptrend = Some(up);
            self.af = self.step;

            if up {
                self.ep = Some(bar.high);
                // Start SAR at prior low
                self.sar = Some(self.prev_prev_low.unwrap());
            } else {
                self.ep = Some(bar.low);
                // Start SAR at prior high
                self.sar = Some(self.prev_prev_high.unwrap());
            }

            return self.sar;
        }

        let uptrend = self.uptrend.unwrap();
        let mut sar = self.sar?;
        let mut ep = self.ep?;

        // Core Wilder update
        sar = sar + self.af * (ep - sar);

        // Band SAR not to penetrate the last two lows/highs depending on trend
        if uptrend {
            let low1 = self.prev_low.unwrap();
            let low2 = self.prev_prev_low.unwrap();
            sar = sar.min(low1).min(low2);
        } else {
            let high1 = self.prev_high.unwrap();
            let high2 = self.prev_prev_high.unwrap();
            sar = sar.max(high1).max(high2);
        }

        // Check reversal first (using bar extremes)
        let reversal = if uptrend {
            // In uptrend, reversal if low <= SAR
            bar.low <= sar
        } else {
            // In downtrend, reversal if high >= SAR
            bar.high >= sar
        };

        if reversal {
            // Flip trend
            let new_uptrend = !uptrend;
            self.uptrend = Some(new_uptrend);

            // On reversal:
            // - SAR becomes prior EP
            // - EP becomes current extreme in new trend
            // - AF resets
            sar = ep;
            self.af = self.step;

            if new_uptrend {
                ep = bar.high;
                // Ensure SAR is not above last two lows
                let low1 = self.prev_low.unwrap();
                let low2 = self.prev_prev_low.unwrap();
                sar = sar.min(low1).min(low2);
            } else {
                ep = bar.low;
                // Ensure SAR is not below last two highs
                let high1 = self.prev_high.unwrap();
                let high2 = self.prev_prev_high.unwrap();
                sar = sar.max(high1).max(high2);
            }

            self.ep = Some(ep);
            self.sar = Some(sar);
            return self.sar;
        }

        // No reversal: update EP and AF when new extremes are made
        if uptrend {
            if bar.high > ep {
                ep = bar.high;
                self.af = (self.af + self.step).min(self.max_step);
            }
        } else if bar.low < ep {
            ep = bar.low;
            self.af = (self.af + self.step).min(self.max_step);
        }

        self.ep = Some(ep);
        self.sar = Some(sar);
        self.sar
    }
}

/// Parabolic SAR Strategy
pub struct ParabolicSARStrategy {
    config: ParabolicSarConfig,
    sar: ParabolicSar,
    trend_sma: Option<Sma>,

    in_position: bool,
    entry_price: Option<f64>,
}

impl Default for ParabolicSARStrategy {
    fn default() -> Self {
        // Default: 0.02 step, 0.2 max_step, 50-period SMA trend filter
        Self::from_config(ParabolicSarConfig::default_config())
    }
}

impl ParabolicSARStrategy {
    pub fn new(step: f64, max_step: f64) -> Self {
        let cfg = ParabolicSarConfig::new(step, max_step);
        Self::from_config(cfg)
    }

    pub fn from_config(config: ParabolicSarConfig) -> Self {
        config.validate().expect("Invalid ParabolicSarConfig");

        let trend_sma = if config.trend_filter_enabled {
            Some(Sma::new(config.trend_sma_period))
        } else {
            None
        };

        Self {
            sar: ParabolicSar::new(config.step, config.max_step),
            config,
            trend_sma,
            in_position: false,
            entry_price: None,
        }
    }

    pub fn config(&self) -> &ParabolicSarConfig {
        &self.config
    }

    fn trend_filter_allows_entry(&self, price: f64) -> bool {
        match &self.trend_sma {
            Some(sma) => sma.value().map(|v| price > v).unwrap_or(false),
            None => true,
        }
    }

    fn reset_state(&mut self) {
        self.in_position = false;
        self.entry_price = None;
    }
}

impl Strategy for ParabolicSARStrategy {
    fn name(&self) -> &str {
        self.config.strategy_name()
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        // Update optional trend filter
        if let Some(sma) = &mut self.trend_sma {
            sma.update(bar.close);
        }

        // Update SAR
        let sar = self.sar.update(bar)?;
        let price = bar.close;

        // Entry/Exit logic (spot-only)
        if self.in_position {
            // Exit if price crosses below SAR (SAR above price) or touches SAR trailing stop
            if price <= sar {
                self.reset_state();
                return Some(vec![Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Sell,
                    strength: 1.0,
                    metadata: Some(format!(
                        "Parabolic SAR Exit: price {:.4} <= sar {:.4}",
                        price, sar
                    )),
                }]);
            }
            return None;
        }

        // Not in position: Entry if price crosses above SAR and filter passes
        // We use close vs SAR as the crossover proxy.
        if price > sar && self.trend_filter_allows_entry(price) {
            self.in_position = true;
            self.entry_price = Some(price);
            return Some(vec![Signal {
                timestamp: bar.timestamp,
                symbol: "UNKNOWN".to_string(),
                signal_type: SignalType::Buy,
                strength: 1.0,
                metadata: Some(format!(
                    "Parabolic SAR Entry: price {:.4} > sar {:.4}",
                    price, sar
                )),
            }]);
        }

        None
    }

    fn on_tick(&mut self, _tick: &alphafield_core::Tick) -> Option<Signal> {
        None
    }
}

impl MetadataStrategy for ParabolicSARStrategy {
    fn metadata(&self) -> StrategyMetadata {
        let mut required_indicators = vec!["ParabolicSAR".to_string(), "Price".to_string()];
        if self.trend_sma.is_some() {
            required_indicators.push(format!("SMA({})", self.config.trend_sma_period));
        }

        StrategyMetadata {
            name: self.config.strategy_name().to_string(),
            category: StrategyCategory::TrendFollowing,
            sub_type: Some("parabolic_sar".to_string()),
            description: format!(
                "Parabolic SAR trend-following strategy with step {:.3}, max_step {:.3}. {}trend filter via SMA({}). \
                Enters when price crosses above SAR; exits on SAR trailing stop.",
                self.config.step,
                self.config.max_step,
                if self.config.trend_filter_enabled { "" } else { "no " },
                self.config.trend_sma_period
            ),
            hypothesis_path: "hypotheses/trend_following/parabolic_sar.md".to_string(),
            required_indicators,
            expected_regimes: vec![MarketRegime::Bull, MarketRegime::Trending],
            risk_profile: RiskProfile {
                max_drawdown_expected: 22.0,
                volatility_level: VolatilityLevel::Medium,
                correlation_sensitivity: CorrelationSensitivity::High,
                leverage_requirement: 1.0,
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::TrendFollowing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        assert!(ParabolicSarConfig::new(0.02, 0.2).validate().is_ok());
        assert!(ParabolicSarConfig::new(0.0, 0.2).validate().is_err());
        assert!(ParabolicSarConfig::new(0.02, 0.0).validate().is_err());
        assert!(ParabolicSarConfig::new(0.2, 0.02).validate().is_err());
    }

    #[test]
    fn test_parabolic_sar_initializes_after_two_bars() {
        let mut sar = ParabolicSar::new(0.02, 0.2);

        let t0 = chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap();
        let t1 = chrono::DateTime::from_timestamp(1_700_086_400, 0).unwrap();

        let b0 = Bar {
            timestamp: t0,
            open: 100.0,
            high: 101.0,
            low: 99.0,
            close: 100.5,
            volume: 1.0,
        };
        let b1 = Bar {
            timestamp: t1,
            open: 100.5,
            high: 102.0,
            low: 100.0,
            close: 101.5,
            volume: 1.0,
        };

        assert!(sar.update(&b0).is_none());
        assert!(sar.update(&b1).is_some());
    }

    #[test]
    fn test_strategy_metadata() {
        let strategy = ParabolicSARStrategy::new(0.02, 0.2);
        let meta = strategy.metadata();
        assert_eq!(meta.name, "Parabolic SAR");
        assert_eq!(meta.category, StrategyCategory::TrendFollowing);
        assert_eq!(meta.sub_type, Some("parabolic_sar".to_string()));
        assert_eq!(
            meta.hypothesis_path,
            "hypotheses/trend_following/parabolic_sar.md"
        );
    }

    #[test]
    fn test_strategy_generates_entry_and_exit_without_trend_filter() {
        let mut cfg = ParabolicSarConfig::new(0.02, 0.2);
        cfg.trend_filter_enabled = false;

        let mut strategy = ParabolicSARStrategy::from_config(cfg);

        let base = 1_700_000_000i64;

        // Seed bars to initialize SAR (small up move)
        let bars = [
            Bar {
                timestamp: chrono::DateTime::from_timestamp(base, 0).unwrap(),
                open: 100.0,
                high: 101.0,
                low: 99.5,
                close: 100.8,
                volume: 1.0,
            },
            Bar {
                timestamp: chrono::DateTime::from_timestamp(base + 86_400, 0).unwrap(),
                open: 100.8,
                high: 102.0,
                low: 100.0,
                close: 101.7,
                volume: 1.0,
            },
        ];

        // First bar: no signal (SAR not ready yet)
        assert!(strategy.on_bar(&bars[0]).is_none());

        // Second bar: SAR becomes available; likely generates entry if close > sar
        let sigs = strategy.on_bar(&bars[1]);
        assert!(sigs.is_some());
        let sigs = sigs.unwrap();
        assert!(sigs
            .iter()
            .any(|s| matches!(s.signal_type, SignalType::Buy)));

        // Now feed a sharp down bar that should cross below SAR to exit
        let down_bar = Bar {
            timestamp: chrono::DateTime::from_timestamp(base + 2 * 86_400, 0).unwrap(),
            open: 101.7,
            high: 101.9,
            low: 98.0,
            close: 98.5,
            volume: 1.0,
        };

        let exit = strategy.on_bar(&down_bar);
        assert!(exit.is_some());
        let exit = exit.unwrap();
        assert!(exit
            .iter()
            .any(|s| matches!(s.signal_type, SignalType::Sell)));
    }

    #[test]
    fn test_strategy_trend_filter_blocks_entry_until_sma_ready() {
        let mut cfg = ParabolicSarConfig::new(0.02, 0.2);
        cfg.trend_filter_enabled = true;
        cfg.trend_sma_period = 5;

        let mut strategy = ParabolicSARStrategy::from_config(cfg);

        let base = 1_700_000_000i64;

        // Provide 2 bars to initialize SAR, but fewer than SMA period -> filter should block entry.
        let b0 = Bar {
            timestamp: chrono::DateTime::from_timestamp(base, 0).unwrap(),
            open: 100.0,
            high: 101.0,
            low: 99.5,
            close: 100.8,
            volume: 1.0,
        };
        let b1 = Bar {
            timestamp: chrono::DateTime::from_timestamp(base + 86_400, 0).unwrap(),
            open: 100.8,
            high: 102.0,
            low: 100.0,
            close: 101.7,
            volume: 1.0,
        };

        assert!(strategy.on_bar(&b0).is_none());

        // SAR is ready here, but SMA is not; thus no entry expected.
        let sigs = strategy.on_bar(&b1);
        assert!(sigs.is_none());

        // Feed additional rising bars to warm up SMA; eventually strategy can enter.
        for i in 2..10 {
            let bar = Bar {
                timestamp: chrono::DateTime::from_timestamp(base + i * 86_400, 0).unwrap(),
                open: 100.0 + i as f64,
                high: 101.0 + i as f64,
                low: 99.0 + i as f64,
                close: 100.5 + i as f64,
                volume: 1.0,
            };

            if let Some(sigs) = strategy.on_bar(&bar) {
                if sigs
                    .iter()
                    .any(|s| matches!(s.signal_type, SignalType::Buy))
                {
                    return; // success
                }
            }
        }

        panic!("Expected entry after SMA warmup in rising market");
    }
}
