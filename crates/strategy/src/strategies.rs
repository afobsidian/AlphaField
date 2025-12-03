use alphafield_core::{Bar, Signal, SignalType, Strategy};
use crate::indicators::{Indicator, Sma, Rsi};
use chrono::Utc;

/// Golden Cross Strategy
/// Buys when Fast SMA crosses above Slow SMA
/// Sells when Fast SMA crosses below Slow SMA
pub struct GoldenCrossStrategy {
    fast_sma: Sma,
    slow_sma: Sma,
    last_fast: Option<f64>,
    last_slow: Option<f64>,
}

impl GoldenCrossStrategy {
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        Self {
            fast_sma: Sma::new(fast_period),
            slow_sma: Sma::new(slow_period),
            last_fast: None,
            last_slow: None,
        }
    }
}

impl Strategy for GoldenCrossStrategy {
    fn name(&self) -> &str {
        "Golden Cross"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Signal> {
        let fast = self.fast_sma.update(bar.close)?;
        let slow = self.slow_sma.update(bar.close)?;

        let mut signal = None;

        if let (Some(prev_fast), Some(prev_slow)) = (self.last_fast, self.last_slow) {
            // Check for crossover
            if prev_fast <= prev_slow && fast > slow {
                // Golden Cross (Bullish)
                signal = Some(Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(), // Strategy doesn't know symbol yet, usually passed in context
                    signal_type: SignalType::Buy,
                    strength: 1.0,
                    metadata: Some(format!("Golden Cross: Fast {:.2} > Slow {:.2}", fast, slow)),
                });
            } else if prev_fast >= prev_slow && fast < slow {
                // Death Cross (Bearish)
                signal = Some(Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Sell,
                    strength: 1.0,
                    metadata: Some(format!("Death Cross: Fast {:.2} < Slow {:.2}", fast, slow)),
                });
            }
        }

        self.last_fast = Some(fast);
        self.last_slow = Some(slow);

        signal
    }
}

/// RSI Strategy
/// Buys when RSI < 30 (Oversold)
/// Sells when RSI > 70 (Overbought)
pub struct RsiStrategy {
    rsi: Rsi,
    lower_bound: f64,
    upper_bound: f64,
    position: SignalType, // Track current position to avoid spamming signals
}

impl RsiStrategy {
    pub fn new(period: usize, lower: f64, upper: f64) -> Self {
        Self {
            rsi: Rsi::new(period),
            lower_bound: lower,
            upper_bound: upper,
            position: SignalType::Hold,
        }
    }
}

impl Strategy for RsiStrategy {
    fn name(&self) -> &str {
        "RSI Mean Reversion"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Signal> {
        let rsi_val = self.rsi.update(bar.close)?;

        if rsi_val < self.lower_bound && self.position != SignalType::Buy {
            self.position = SignalType::Buy;
            return Some(Signal {
                timestamp: bar.timestamp,
                symbol: "UNKNOWN".to_string(),
                signal_type: SignalType::Buy,
                strength: (self.lower_bound - rsi_val) / self.lower_bound, // Higher strength if deeper oversold
                metadata: Some(format!("RSI Oversold: {:.2}", rsi_val)),
            });
        } else if rsi_val > self.upper_bound && self.position != SignalType::Sell {
            self.position = SignalType::Sell;
            return Some(Signal {
                timestamp: bar.timestamp,
                symbol: "UNKNOWN".to_string(),
                signal_type: SignalType::Sell,
                strength: (rsi_val - self.upper_bound) / (100.0 - self.upper_bound),
                metadata: Some(format!("RSI Overbought: {:.2}", rsi_val)),
            });
        }

        None
    }
}
