use std::collections::VecDeque;

/// Trait for technical indicators
pub trait Indicator {
    /// Update the indicator with a new value
    fn update(&mut self, value: f64) -> Option<f64>;

    /// Get the current value of the indicator
    fn value(&self) -> Option<f64>;

    /// Reset the indicator state
    fn reset(&mut self);
}

/// Simple Moving Average (SMA)
pub struct Sma {
    period: usize,
    window: VecDeque<f64>,
    sum: f64,
}

impl Sma {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            window: VecDeque::with_capacity(period),
            sum: 0.0,
        }
    }
}

impl Indicator for Sma {
    fn update(&mut self, value: f64) -> Option<f64> {
        self.window.push_back(value);
        self.sum += value;

        if self.window.len() > self.period {
            if let Some(removed) = self.window.pop_front() {
                self.sum -= removed;
            }
        }

        self.value()
    }

    fn value(&self) -> Option<f64> {
        if self.window.len() < self.period {
            None
        } else {
            Some(self.sum / self.period as f64)
        }
    }

    fn reset(&mut self) {
        self.window.clear();
        self.sum = 0.0;
    }
}

/// Exponential Moving Average (EMA)
pub struct Ema {
    /// Period for EMA calculation (kept for introspection/debugging)
    #[allow(dead_code)]
    period: usize,
    k: f64,
    current: Option<f64>,
}

impl Ema {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            k: 2.0 / (period as f64 + 1.0),
            current: None,
        }
    }
}

impl Indicator for Ema {
    fn update(&mut self, value: f64) -> Option<f64> {
        match self.current {
            Some(prev) => {
                let next = (value - prev) * self.k + prev;
                self.current = Some(next);
            }
            None => {
                self.current = Some(value);
            }
        }
        self.current
    }

    fn value(&self) -> Option<f64> {
        self.current
    }

    fn reset(&mut self) {
        self.current = None;
    }
}

/// Relative Strength Index (RSI)
pub struct Rsi {
    period: usize,
    gains: VecDeque<f64>,
    losses: VecDeque<f64>,
    prev_value: Option<f64>,
    avg_gain: Option<f64>,
    avg_loss: Option<f64>,
}

impl Rsi {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            gains: VecDeque::new(),
            losses: VecDeque::new(),
            prev_value: None,
            avg_gain: None,
            avg_loss: None,
        }
    }
}

impl Indicator for Rsi {
    fn update(&mut self, value: f64) -> Option<f64> {
        let change = match self.prev_value {
            Some(prev) => value - prev,
            None => {
                self.prev_value = Some(value);
                return None;
            }
        };

        self.prev_value = Some(value);

        let gain = if change > 0.0 { change } else { 0.0 };
        let loss = if change < 0.0 { -change } else { 0.0 };

        // Initial calculation (Wilder's Smoothing)
        if self.avg_gain.is_none() {
            self.gains.push_back(gain);
            self.losses.push_back(loss);

            if self.gains.len() == self.period {
                let sum_gain: f64 = self.gains.iter().sum();
                let sum_loss: f64 = self.losses.iter().sum();

                self.avg_gain = Some(sum_gain / self.period as f64);
                self.avg_loss = Some(sum_loss / self.period as f64);

                // Clear history as we switch to smoothing
                self.gains.clear();
                self.losses.clear();
            } else {
                return None;
            }
        } else {
            // Smoothed update
            self.avg_gain = Some(
                (self.avg_gain.unwrap() * (self.period as f64 - 1.0) + gain) / self.period as f64,
            );
            self.avg_loss = Some(
                (self.avg_loss.unwrap() * (self.period as f64 - 1.0) + loss) / self.period as f64,
            );
        }

        self.value()
    }

    fn value(&self) -> Option<f64> {
        match (self.avg_gain, self.avg_loss) {
            (Some(avg_gain), Some(avg_loss)) => {
                if avg_loss == 0.0 {
                    if avg_gain == 0.0 {
                        Some(50.0)
                    } else {
                        Some(100.0)
                    }
                } else {
                    let rs = avg_gain / avg_loss;
                    Some(100.0 - (100.0 / (1.0 + rs)))
                }
            }
            _ => None,
        }
    }

    fn reset(&mut self) {
        self.gains.clear();
        self.losses.clear();
        self.prev_value = None;
        self.avg_gain = None;
        self.avg_loss = None;
    }
}

/// Bollinger Bands
///
/// Technical indicator that consists of three bands:
/// - Middle band: Simple Moving Average
/// - Upper band: Middle + (standard deviation * multiplier)
/// - Lower band: Middle - (standard deviation * multiplier)
pub struct BollingerBands {
    period: usize,
    num_std_dev: f64,
    window: Vec<f64>,
    pos: usize,
    filled: bool,
}

impl BollingerBands {
    pub fn new(period: usize, num_std_dev: f64) -> Self {
        Self {
            period,
            num_std_dev,
            window: vec![0.0; period],
            pos: 0,
            filled: false,
        }
    }

    /// Updates the indicator and returns (upper_band, middle_band, lower_band)
    pub fn update(&mut self, value: f64) -> Option<(f64, f64, f64)> {
        self.window[self.pos] = value;
        self.pos += 1;

        if self.pos >= self.period {
            self.pos = 0;
            self.filled = true;
        }

        if !self.filled {
            return None;
        }

        // Calculate middle band (SMA)
        let middle: f64 = self.window.iter().sum::<f64>() / self.period as f64;

        // Calculate standard deviation
        let variance: f64 = self
            .window
            .iter()
            .map(|x| {
                let diff = x - middle;
                diff * diff
            })
            .sum::<f64>()
            / self.period as f64;

        let std_dev = variance.sqrt();

        let upper = middle + (std_dev * self.num_std_dev);
        let lower = middle - (std_dev * self.num_std_dev);

        Some((upper, middle, lower))
    }

    pub fn reset(&mut self) {
        self.window.fill(0.0);
        self.pos = 0;
        self.filled = false;
    }
}

/// MACD (Moving Average Convergence Divergence)
///
/// Trend-following momentum indicator that shows the relationship
/// between two exponential moving averages.
pub struct Macd {
    fast_ema: Ema,
    slow_ema: Ema,
    signal_ema: Ema,
}

impl Macd {
    /// Creates a new MACD indicator
    ///
    /// # Arguments
    /// * `fast_period` - Fast EMA period (typically 12)
    /// * `slow_period` - Slow EMA period (typically 26)
    /// * `signal_period` - Signal line EMA period (typically 9)
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Self {
        Self {
            fast_ema: Ema::new(fast_period),
            slow_ema: Ema::new(slow_period),
            signal_ema: Ema::new(signal_period),
        }
    }

    /// Updates the indicator and returns (macd_line, signal_line, histogram)
    pub fn update(&mut self, value: f64) -> Option<(f64, f64, f64)> {
        let fast = self.fast_ema.update(value)?;
        let slow = self.slow_ema.update(value)?;

        let macd_line = fast - slow;
        let signal_line = self.signal_ema.update(macd_line)?;
        let histogram = macd_line - signal_line;

        Some((macd_line, signal_line, histogram))
    }

    pub fn reset(&mut self) {
        self.fast_ema.reset();
        self.slow_ema.reset();
        self.signal_ema.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma() {
        let mut sma = Sma::new(3);
        assert_eq!(sma.update(10.0), None);
        assert_eq!(sma.update(20.0), None);
        assert_eq!(sma.update(30.0), Some(20.0)); // (10+20+30)/3 = 20
        assert_eq!(sma.update(40.0), Some(30.0)); // (20+30+40)/3 = 30
    }

    #[test]
    fn test_ema() {
        let mut ema = Ema::new(3); // k = 2/(3+1) = 0.5
        assert_eq!(ema.update(10.0), Some(10.0));
        assert_eq!(ema.update(20.0), Some(15.0)); // (20-10)*0.5 + 10 = 15
        assert_eq!(ema.update(30.0), Some(22.5)); // (30-15)*0.5 + 15 = 22.5
    }
}
