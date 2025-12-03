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
            self.avg_gain = Some((self.avg_gain.unwrap() * (self.period as f64 - 1.0) + gain) / self.period as f64);
            self.avg_loss = Some((self.avg_loss.unwrap() * (self.period as f64 - 1.0) + loss) / self.period as f64);
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
