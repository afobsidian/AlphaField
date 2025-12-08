//! Drawdown Analysis Module
//!
//! Provides detailed drawdown analysis including underwater periods,
//! recovery times, and drawdown statistics.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Represents a single drawdown period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawdownPeriod {
    /// Start of drawdown (peak)
    pub start: DateTime<Utc>,
    /// Deepest point of drawdown (trough)
    pub trough: DateTime<Utc>,
    /// End of drawdown (recovery), None if still in drawdown
    pub end: Option<DateTime<Utc>>,
    /// Maximum depth of this drawdown (0.0 to 1.0)
    pub depth: f64,
    /// Duration from start to trough in seconds
    pub decline_duration_secs: i64,
    /// Duration from trough to recovery in seconds
    pub recovery_duration_secs: Option<i64>,
}

impl DrawdownPeriod {
    /// Total duration of the drawdown period
    pub fn total_duration(&self) -> Option<Duration> {
        self.end.map(|e| e - self.start)
    }
}

/// Complete drawdown analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawdownAnalysis {
    /// Current drawdown (0.0 if at peak)
    pub current_drawdown: f64,
    /// Maximum drawdown observed
    pub max_drawdown: f64,
    /// Maximum drawdown duration in seconds
    pub max_drawdown_duration_secs: i64,
    /// Average drawdown depth
    pub avg_drawdown: f64,
    /// Individual drawdown periods
    pub drawdown_periods: Vec<DrawdownPeriod>,
    /// Total time spent underwater (in drawdown) in seconds
    pub time_underwater_secs: i64,
    /// Percentage of total time spent in drawdown
    pub percent_time_underwater: f64,
    /// Ulcer Index (RMS of drawdowns)
    pub ulcer_index: f64,
    /// Recovery factor (total return / max drawdown)
    pub recovery_factor: f64,
}

impl Default for DrawdownAnalysis {
    fn default() -> Self {
        Self {
            current_drawdown: 0.0,
            max_drawdown: 0.0,
            max_drawdown_duration_secs: 0,
            avg_drawdown: 0.0,
            drawdown_periods: Vec::new(),
            time_underwater_secs: 0,
            percent_time_underwater: 0.0,
            ulcer_index: 0.0,
            recovery_factor: 0.0,
        }
    }
}

/// Drawdown curve point for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawdownPoint {
    pub timestamp: i64,
    pub drawdown: f64,
    pub peak: f64,
    pub current: f64,
}

impl DrawdownAnalysis {
    /// Calculate drawdown analysis from equity history
    pub fn calculate(equity_history: &[(i64, f64)], total_return: f64) -> Self {
        if equity_history.is_empty() {
            return Self::default();
        }

        let mut peak = equity_history[0].1;
        let mut max_drawdown = 0.0;
        let mut current_drawdown = 0.0;
        let mut drawdown_periods = Vec::new();
        let mut current_period: Option<DrawdownPeriod> = None;
        let mut all_drawdowns = Vec::new();

        for &(ts, equity) in equity_history {
            let timestamp = DateTime::from_timestamp_millis(ts).unwrap_or_else(|| Utc::now());

            if equity > peak {
                // New peak - close any open drawdown period
                if let Some(mut period) = current_period.take() {
                    period.end = Some(timestamp);
                    period.recovery_duration_secs = Some((timestamp - period.trough).num_seconds());
                    drawdown_periods.push(period);
                }
                peak = equity;
                current_drawdown = 0.0;
            } else {
                // In drawdown
                let dd = (peak - equity) / peak;
                all_drawdowns.push(dd);

                if dd > current_drawdown {
                    current_drawdown = dd;
                }

                if dd > max_drawdown {
                    max_drawdown = dd;
                }

                // Start new period if not in one
                if current_period.is_none() {
                    current_period = Some(DrawdownPeriod {
                        start: timestamp,
                        trough: timestamp,
                        end: None,
                        depth: dd,
                        decline_duration_secs: 0,
                        recovery_duration_secs: None,
                    });
                }

                // Update current period
                if let Some(ref mut period) = current_period {
                    if dd > period.depth {
                        period.depth = dd;
                        period.trough = timestamp;
                        period.decline_duration_secs = (timestamp - period.start).num_seconds();
                    }
                }
            }
        }

        // Handle unclosed period
        if let Some(period) = current_period {
            drawdown_periods.push(period);
        }

        // Calculate statistics
        let avg_drawdown = if all_drawdowns.is_empty() {
            0.0
        } else {
            all_drawdowns.iter().sum::<f64>() / all_drawdowns.len() as f64
        };

        // Calculate max drawdown duration
        let max_drawdown_duration_secs = drawdown_periods
            .iter()
            .filter_map(|p| p.total_duration())
            .map(|d| d.num_seconds())
            .max()
            .unwrap_or(0);

        // Calculate time underwater
        let time_underwater_secs: i64 = drawdown_periods
            .iter()
            .filter_map(|p| p.total_duration())
            .map(|d| d.num_seconds())
            .sum();

        let total_time_secs = if equity_history.len() > 1 {
            equity_history.last().unwrap().0 - equity_history.first().unwrap().0
        } else {
            1
        } / 1000; // Convert ms to seconds

        let percent_time_underwater = if total_time_secs > 0 {
            time_underwater_secs as f64 / total_time_secs as f64 * 100.0
        } else {
            0.0
        };

        // Ulcer Index: sqrt(sum(dd^2) / n)
        let ulcer_index = if all_drawdowns.is_empty() {
            0.0
        } else {
            let sum_sq: f64 = all_drawdowns.iter().map(|d| d.powi(2)).sum();
            (sum_sq / all_drawdowns.len() as f64).sqrt()
        };

        // Recovery factor: total return / max drawdown
        let recovery_factor = if max_drawdown > 0.0 {
            total_return / max_drawdown
        } else if total_return > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };

        Self {
            current_drawdown,
            max_drawdown,
            max_drawdown_duration_secs,
            avg_drawdown,
            drawdown_periods,
            time_underwater_secs,
            percent_time_underwater,
            ulcer_index,
            recovery_factor,
        }
    }

    /// Generate drawdown curve for visualization
    pub fn generate_curve(equity_history: &[(i64, f64)]) -> Vec<DrawdownPoint> {
        if equity_history.is_empty() {
            return Vec::new();
        }

        let mut peak = equity_history[0].1;
        let mut curve = Vec::with_capacity(equity_history.len());

        for &(ts, equity) in equity_history {
            if equity > peak {
                peak = equity;
            }
            let drawdown = (peak - equity) / peak;
            curve.push(DrawdownPoint {
                timestamp: ts,
                drawdown,
                peak,
                current: equity,
            });
        }

        curve
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drawdown_analysis() {
        // Equity: 100 -> 110 -> 95 -> 105 -> 90 -> 115
        let equity = vec![
            (0i64, 100.0),
            (86400000, 110.0),  // Peak
            (172800000, 95.0),  // Drawdown 13.6%
            (259200000, 105.0), // Partial recovery
            (345600000, 90.0),  // Deeper drawdown 18.2%
            (432000000, 115.0), // New peak
        ];

        let analysis = DrawdownAnalysis::calculate(&equity, 0.15);

        assert!((analysis.max_drawdown - 0.182).abs() < 0.01);
        assert!(analysis.drawdown_periods.len() >= 1);
    }

    #[test]
    fn test_drawdown_curve() {
        let equity = vec![
            (0i64, 100.0),
            (86400000, 110.0),
            (172800000, 95.0),
        ];

        let curve = DrawdownAnalysis::generate_curve(&equity);
        assert_eq!(curve.len(), 3);
        assert_eq!(curve[0].drawdown, 0.0); // At peak
        assert_eq!(curve[1].drawdown, 0.0); // New peak
        assert!((curve[2].drawdown - 0.136).abs() < 0.01); // In drawdown
    }
}
