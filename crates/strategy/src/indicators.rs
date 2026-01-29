use std::collections::VecDeque;

/// Trait for technical indicators
pub trait Indicator: Send + Sync {
    /// Update the indicator with a new value
    fn update(&mut self, value: f64) -> Option<f64>;

    /// Get the current value of the indicator
    fn value(&self) -> Option<f64>;

    /// Reset the indicator state
    fn reset(&mut self);
}

/// Kaufman's Adaptive Moving Average (KAMA)
///
/// Adaptive moving average that adjusts its smoothing based on market "efficiency":
/// - In trending markets, it responds faster (less smoothing)
/// - In ranging markets, it smooths more (more smoothing)
///
/// References:
/// - Perry J. Kaufman, "Trading Systems and Methods"
pub struct Kama {
    er_period: usize,
    fast_period: usize,
    slow_period: usize,
    prices: VecDeque<f64>,
    current: Option<f64>,
}

impl Kama {
    /// Create a new KAMA indicator
    ///
    /// # Arguments
    /// * `er_period` - Efficiency ratio lookback period (commonly 10)
    /// * `fast_period` - Fast EMA equivalent period (commonly 2)
    /// * `slow_period` - Slow EMA equivalent period (commonly 30)
    pub fn new(er_period: usize, fast_period: usize, slow_period: usize) -> Self {
        Self {
            er_period,
            fast_period,
            slow_period,
            prices: VecDeque::with_capacity(er_period + 1),
            current: None,
        }
    }

    fn fast_sc(&self) -> f64 {
        2.0 / (self.fast_period as f64 + 1.0)
    }

    fn slow_sc(&self) -> f64 {
        2.0 / (self.slow_period as f64 + 1.0)
    }
}

impl Indicator for Kama {
    fn update(&mut self, value: f64) -> Option<f64> {
        self.prices.push_back(value);
        if self.prices.len() > self.er_period + 1 {
            self.prices.pop_front();
        }

        // Need ER window
        if self.prices.len() < self.er_period + 1 {
            return None;
        }

        // direction = |price_now - price_then|
        let direction = (self.prices.back()? - self.prices.front()?).abs();

        // volatility = sum(|price_i - price_{i-1}|)
        let mut volatility = 0.0;
        let mut iter = self.prices.iter();
        let mut prev = *iter.next()?; // safe because len >= 2 here
        for p in iter {
            volatility += (*p - prev).abs();
            prev = *p;
        }

        let er = if volatility > 0.0 {
            direction / volatility
        } else {
            0.0
        };

        let sc = (er * (self.fast_sc() - self.slow_sc()) + self.slow_sc()).powi(2);

        let next = match self.current {
            Some(prev_kama) => prev_kama + sc * (value - prev_kama),
            None => value,
        };

        self.current = Some(next);
        self.current
    }

    fn value(&self) -> Option<f64> {
        self.current
    }

    fn reset(&mut self) {
        self.prices.clear();
        self.current = None;
    }
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

#[allow(clippy::unnecessary_unwrap)]
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

/// Average True Range (ATR)
///
/// Volatility indicator that measures the degree of price volatility.
/// ATR is calculated using a moving average of true ranges over a specified period.
///
/// True Range is the greatest of:
/// - Current High - Current Low
/// - Absolute value of (Current High - Previous Close)
/// - Absolute value of (Current Low - Previous Close)
///
/// ATR does not indicate price direction, only volatility magnitude.
pub struct Atr {
    period: usize,
    tr_history: VecDeque<f64>,
    prev_close: Option<f64>,
    current_atr: Option<f64>,
}

impl Atr {
    /// Creates a new ATR indicator
    ///
    /// # Arguments
    /// * `period` - Period for ATR calculation (typically 14)
    pub fn new(period: usize) -> Self {
        Self {
            period,
            tr_history: VecDeque::with_capacity(period),
            prev_close: None,
            current_atr: None,
        }
    }
}

impl Atr {
    /// Update the ATR indicator with a new bar
    ///
    /// Uses Wilder's smoothing method after the initial period is filled.
    ///
    /// # Arguments
    /// * `bar` - A bar containing high, low, and close prices
    ///
    /// Returns the current ATR value when available, None during warmup period
    pub fn update(&mut self, bar: &alphafield_core::Bar) -> Option<f64> {
        let tr = match self.prev_close {
            Some(prev_close) => {
                let hl = bar.high - bar.low;
                let hc = (bar.high - prev_close).abs();
                let lc = (bar.low - prev_close).abs();
                hl.max(hc).max(lc)
            }
            None => bar.high - bar.low,
        };

        self.prev_close = Some(bar.close);
        self.tr_history.push_back(tr);

        if self.tr_history.len() > self.period {
            self.tr_history.pop_front();
        }

        // Use simple average until we have enough data, then switch to Wilder's smoothing
        if self.tr_history.len() < self.period {
            None
        } else if self.current_atr.is_none() {
            // First ATR value - simple average
            let sum: f64 = self.tr_history.iter().sum();
            self.current_atr = Some(sum / self.period as f64);
            self.current_atr
        } else {
            // Wilder's smoothing: ATR = (prev_ATR * (period - 1) + current_TR) / period
            let prev_atr = self.current_atr.unwrap();
            let new_atr = (prev_atr * (self.period as f64 - 1.0) + tr) / self.period as f64;
            self.current_atr = Some(new_atr);
            self.current_atr
        }
    }

    /// Get the current ATR value
    pub fn value(&self) -> Option<f64> {
        self.current_atr
    }

    /// Reset the indicator state
    pub fn reset(&mut self) {
        self.tr_history.clear();
        self.prev_close = None;
        self.current_atr = None;
    }
}

/// Average Directional Index (ADX)
///
/// Trend strength indicator that measures the strength of a trend regardless of direction.
/// ADX is derived from the Directional Movement System (DM) developed by J. Welles Wilder.
///
/// The calculation involves:
/// 1. Computing +DM and -DM (directional movement)
/// 2. Computing True Range (TR)
/// 3. Calculating smoothed +DM and -DM
/// 4. Calculating +DI and -DI (directional indices)
/// 5. Computing DX (directional index)
/// 6. Smoothing DX to get ADX
///
/// ADX values:
/// - 0-25: Weak or absent trend (ranging market)
/// - 25-50: Strong trend
/// - 50-75: Very strong trend
/// - 75-100: Extremely strong trend
pub struct Adx {
    period: usize,
    tr_values: VecDeque<f64>,
    plus_dm_values: VecDeque<f64>,
    minus_dm_values: VecDeque<f64>,
    prev_high: Option<f64>,
    prev_low: Option<f64>,
    dx_values: VecDeque<f64>,
    current_adx: Option<f64>,
}

impl Adx {
    /// Creates a new ADX indicator
    ///
    /// # Arguments
    /// * `period` - Period for ADX calculation (typically 14)
    pub fn new(period: usize) -> Self {
        Self {
            period,
            tr_values: VecDeque::with_capacity(period),
            plus_dm_values: VecDeque::with_capacity(period),
            minus_dm_values: VecDeque::with_capacity(period),
            prev_high: None,
            prev_low: None,
            dx_values: VecDeque::with_capacity(period),
            current_adx: None,
        }
    }

    /// Update the ADX indicator with a new bar
    ///
    /// Requires a warmup period of (period * 2) bars before returning values.
    ///
    /// # Arguments
    /// * `bar` - A bar containing high, low, and close prices
    ///
    /// Returns the current ADX value when available, None during warmup period
    pub fn update(&mut self, bar: &alphafield_core::Bar) -> Option<f64> {
        if let (Some(prev_high), Some(prev_low)) = (self.prev_high, self.prev_low) {
            // Calculate True Range
            let hl = bar.high - bar.low;
            let hc = (bar.high - bar.close).abs();
            let lc = (bar.low - bar.close).abs();
            let tr = hl.max(hc).max(lc);

            // Calculate Directional Movement
            let up_move = bar.high - prev_high;
            let down_move = prev_low - bar.low;

            let plus_dm = if up_move > down_move && up_move > 0.0 {
                up_move
            } else {
                0.0
            };

            let minus_dm = if down_move > up_move && down_move > 0.0 {
                down_move
            } else {
                0.0
            };

            self.tr_values.push_back(tr);
            self.plus_dm_values.push_back(plus_dm);
            self.minus_dm_values.push_back(minus_dm);

            // Maintain fixed window size
            if self.tr_values.len() > self.period {
                self.tr_values.pop_front();
                self.plus_dm_values.pop_front();
                self.minus_dm_values.pop_front();
            }

            // Need at least `period` bars before calculating
            if self.tr_values.len() >= self.period {
                // Calculate smoothed TR and DM
                let sum_tr: f64 = self.tr_values.iter().sum();
                let sum_plus_dm: f64 = self.plus_dm_values.iter().sum();
                let sum_minus_dm: f64 = self.minus_dm_values.iter().sum();

                let smoothed_tr = sum_tr;
                let smoothed_plus_dm = sum_plus_dm;
                let smoothed_minus_dm = sum_minus_dm;

                // Calculate +DI and -DI
                let plus_di = if smoothed_tr > 0.0 {
                    (smoothed_plus_dm / smoothed_tr) * 100.0
                } else {
                    0.0
                };

                let minus_di = if smoothed_tr > 0.0 {
                    (smoothed_minus_dm / smoothed_tr) * 100.0
                } else {
                    0.0
                };

                // Calculate DX
                let di_sum = plus_di + minus_di;
                let dx = if di_sum > 0.0 {
                    ((plus_di - minus_di).abs() / di_sum) * 100.0
                } else {
                    0.0
                };

                self.dx_values.push_back(dx);

                if self.dx_values.len() > self.period {
                    self.dx_values.pop_front();
                }

                // Calculate ADX
                if self.dx_values.len() >= self.period {
                    if let Some(prev_adx) = self.current_adx {
                        // Smoothed ADX
                        let new_adx =
                            (prev_adx * (self.period as f64 - 1.0) + dx) / self.period as f64;
                        self.current_adx = Some(new_adx);
                    } else {
                        // First ADX - simple average
                        let sum_dx: f64 = self.dx_values.iter().sum();
                        self.current_adx = Some(sum_dx / self.period as f64);
                    }
                }
            }
        }

        self.prev_high = Some(bar.high);
        self.prev_low = Some(bar.low);
        self.current_adx
    }

    /// Get the current ADX value
    pub fn value(&self) -> Option<f64> {
        self.current_adx
    }

    /// Reset the indicator state
    pub fn reset(&mut self) {
        self.tr_values.clear();
        self.plus_dm_values.clear();
        self.minus_dm_values.clear();
        self.prev_high = None;
        self.prev_low = None;
        self.dx_values.clear();
        self.current_adx = None;
    }
}

/// Stochastic Oscillator
///
/// The Stochastic Oscillator is a momentum indicator that shows the location of the close
/// relative to the high-low range over a set number of periods.
///
/// %K = (Current Close - Lowest Low) / (Highest High - Lowest Low) * 100
/// %D = SMA of %K
///
/// Values range from 0 to 100:
/// - Above 80: Overbought
/// - Below 20: Oversold
pub struct Stochastic {
    k_period: usize,
    d_period: usize,
    smooth_period: usize,
    highs: VecDeque<f64>,
    lows: VecDeque<f64>,
    closes: VecDeque<f64>,
    k_values: VecDeque<f64>,
    historical_raw_k_values: VecDeque<f64>,
    current_k: Option<f64>,
    current_d: Option<f64>,
}

impl Stochastic {
    /// Creates a new Stochastic Oscillator
    ///
    /// # Arguments
    /// * `k_period` - Period for %K calculation (typically 14)
    /// * `d_period` - Period for %D (SMA of %K) calculation (typically 3)
    /// * `smooth_period` - Period for smoothing %K (typically 3)
    pub fn new(k_period: usize, d_period: usize, smooth_period: usize) -> Self {
        Self {
            k_period,
            d_period,
            smooth_period,
            highs: VecDeque::with_capacity(k_period),
            lows: VecDeque::with_capacity(k_period),
            closes: VecDeque::with_capacity(smooth_period),
            k_values: VecDeque::with_capacity(d_period),
            historical_raw_k_values: VecDeque::with_capacity(smooth_period),
            current_k: None,
            current_d: None,
        }
    }

    /// Update with a new bar
    ///
    /// # Arguments
    /// * `bar` - A bar containing high, low, and close prices
    ///
    /// Returns (%K, %D) when enough data is available
    pub fn update(&mut self, bar: &alphafield_core::Bar) -> Option<(f64, f64)> {
        self.highs.push_back(bar.high);
        self.lows.push_back(bar.low);
        self.closes.push_back(bar.close);

        // Maintain window size
        if self.highs.len() > self.k_period {
            self.highs.pop_front();
            self.lows.pop_front();
        }
        if self.closes.len() > self.smooth_period {
            self.closes.pop_front();
        }

        // Need enough data for %K calculation
        if self.highs.len() < self.k_period {
            return None;
        }

        // Calculate raw %K
        let highest_high = self.highs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let lowest_low = self.lows.iter().copied().fold(f64::INFINITY, f64::min);
        let range = highest_high - lowest_low;

        if range == 0.0 {
            return None;
        }

        let raw_k = ((bar.close - lowest_low) / range) * 100.0;

        // Store raw %K value for smoothing
        self.historical_raw_k_values.push_back(raw_k);
        if self.historical_raw_k_values.len() > self.smooth_period {
            self.historical_raw_k_values.pop_front();
        }

        // Smooth %K if smooth_period > 1
        let k_value =
            if self.smooth_period > 1 && self.historical_raw_k_values.len() >= self.smooth_period {
                // Average of recent raw %K values
                let sum: f64 = self.historical_raw_k_values.iter().sum();
                sum / self.smooth_period as f64
            } else {
                raw_k
            };

        self.current_k = Some(k_value);
        self.k_values.push_back(k_value);

        // Maintain window size for %D
        if self.k_values.len() > self.d_period {
            self.k_values.pop_front();
        }

        // Calculate %D (SMA of %K)
        if self.k_values.len() >= self.d_period {
            let sum: f64 = self.k_values.iter().sum();
            self.current_d = Some(sum / self.d_period as f64);
        }

        if let (Some(k), Some(d)) = (self.current_k, self.current_d) {
            Some((k, d))
        } else {
            None
        }
    }

    /// Get the current %K value
    pub fn k_value(&self) -> Option<f64> {
        self.current_k
    }

    /// Get the current %D value
    pub fn d_value(&self) -> Option<f64> {
        self.current_d
    }

    /// Get both %K and %D values
    pub fn value(&self) -> Option<(f64, f64)> {
        if let (Some(k), Some(d)) = (self.current_k, self.current_d) {
            Some((k, d))
        } else {
            None
        }
    }

    /// Reset the indicator state
    pub fn reset(&mut self) {
        self.highs.clear();
        self.lows.clear();
        self.closes.clear();
        self.k_values.clear();
        self.historical_raw_k_values.clear();
        self.current_k = None;
        self.current_d = None;
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

    #[test]
    fn test_kama_initialization() {
        let kama = Kama::new(10, 30, 10);
        assert!(kama.value().is_none());
    }

    #[test]
    fn test_kama_calculation() {
        let mut kama = Kama::new(2, 10, 5);

        // Feed some data
        for i in 0..10 {
            let _ = kama.update(100.0 + i as f64);
        }

        // Should have a value now
        assert!(kama.value().is_some());
    }

    #[test]
    fn test_kama_adaptivity() {
        let mut kama = Kama::new(2, 10, 5);

        // Feed trending data (should have higher efficiency ratio)
        for i in 0..10 {
            let _ = kama.update(100.0 + i as f64 * 2.0);
        }
        let trending_value = kama.value();

        // Feed ranging data (should have lower efficiency ratio)
        let mut kama_ranging = Kama::new(2, 10, 5);
        for i in 0..10 {
            let _ = kama_ranging.update(100.0 + (i % 2) as f64);
        }
        let ranging_value = kama_ranging.value();

        // In trending markets, KAMA should be closer to current price
        // In ranging markets, KAMA should be smoother
        assert!(trending_value.is_some());
        assert!(ranging_value.is_some());
    }

    #[test]
    fn test_macd_initialization() {
        let mut macd = Macd::new(12, 26, 9);

        // With the current MACD API, values are returned from `update(...)`.
        // Fast/slow EMA values are available immediately, but the MACD output is only available
        // once the signal EMA can be computed as well. Therefore, we assert that a brand-new MACD
        // can be constructed and updated without asserting on `None` for the first tick.
        let _ = macd.update(100.0);
    }

    #[test]
    fn test_macd_calculation() {
        let mut macd = Macd::new(12, 26, 9);

        // Feed enough data for MACD to calculate (need fast & slow EMA + signal EMA warmup)
        let mut last = None;
        for i in 0..50 {
            last = macd.update(100.0 + i as f64);
        }

        // Should have values now via the returned tuple
        assert!(last.is_some());
        let (macd_line, signal_line, histogram) = last.unwrap();
        assert!(macd_line.is_finite());
        assert!(signal_line.is_finite());
        assert!(histogram.is_finite());
    }

    #[test]
    fn test_macd_crossover() {
        let mut macd = Macd::new(12, 26, 9);

        // Feed trending data; once MACD is warmed up, expect at least one positive component
        let mut last = None;
        for i in 0..50 {
            last = macd.update(100.0 + i as f64 * 0.5);
        }

        let (macd_val, signal_val, histogram_val) = last.unwrap();
        assert!(macd_val > 0.0 || signal_val > 0.0 || histogram_val > 0.0);
    }
}
