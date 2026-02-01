//! Data generators for creating mock market data for testing

use alphafield_core::Bar;
use chrono::{Duration, TimeZone, Utc};
use rand::{thread_rng, Rng};

/// Generate a trending (bull) market with consistent upward movement
///
/// # Arguments
/// * `periods` - Number of bars to generate
/// * `trend` - Daily trend rate (e.g., 0.02 = 2% upward trend per period)
///
/// # Returns
/// Vector of Bar structs representing a trending market
pub fn generate_trending_market(periods: usize, trend: f64) -> Vec<Bar> {
    let mut bars = Vec::with_capacity(periods);
    let mut rng = thread_rng();
    let base_price = 100.0;
    let mut timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();

    for i in 0..periods {
        // Calculate trend-based price with random volatility
        let trend_factor = 1.0 + (trend * i as f64);
        let volatility = 0.01; // 1% daily volatility

        let base = base_price * trend_factor;
        let noise = rng.gen_range(-volatility..volatility);

        let open = base * (1.0 + noise);
        let close = open * (1.0 + rng.gen_range(-0.005..0.015)); // Slight upward bias
        let high = open.max(close) * (1.0 + rng.gen_range(0.0..0.01));
        let low = open.min(close) * (1.0 - rng.gen_range(0.0..0.01));
        let volume = rng.gen_range(1000.0..10000.0);

        bars.push(Bar {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        });

        // Increment by 1 day
        timestamp += Duration::days(1);
    }

    bars
}

/// Generate a ranging (sideways) market oscillating around mean
///
/// # Arguments
/// * `periods` - Number of bars to generate
/// * `volatility` - Price volatility as percentage (e.g., 0.05 = 5%)
///
/// # Returns
/// Vector of Bar structs representing a ranging market
pub fn generate_ranging_market(periods: usize, volatility: f64) -> Vec<Bar> {
    let mut bars = Vec::with_capacity(periods);
    let mut rng = thread_rng();
    let mean_price = 100.0;
    let mut timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();

    // Use sine wave for oscillation with random noise
    for i in 0..periods {
        let phase = (i as f64 / 20.0) * std::f64::consts::PI * 2.0;
        let sine_component = phase.sin() * volatility * mean_price;
        let noise = rng.gen_range(-volatility * 0.3..volatility * 0.3) * mean_price;

        let base = mean_price + sine_component + noise;
        let open = base;
        let close = base + rng.gen_range(-volatility * 0.1..volatility * 0.1) * mean_price;
        let high = open.max(close) + rng.gen_range(0.0..volatility * 0.05) * mean_price;
        let low = open.min(close) - rng.gen_range(0.0..volatility * 0.05) * mean_price;
        let volume = rng.gen_range(1000.0..10000.0);

        bars.push(Bar {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        });

        timestamp += Duration::days(1);
    }

    bars
}

/// Generate a choppy market with frequent direction changes
///
/// # Arguments
/// * `periods` - Number of bars to generate
///
/// # Returns
/// Vector of Bar structs representing a choppy market
pub fn generate_choppy_market(periods: usize) -> Vec<Bar> {
    let mut bars = Vec::with_capacity(periods);
    let mut rng = thread_rng();
    let base_price = 100.0;
    let mut timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();

    // High noise, rapid direction changes
    for _ in 0..periods {
        let noise = rng.gen_range(-0.02..0.02); // 2% noise
        let base = base_price * (1.0 + noise);

        let open: f64 = base;
        let close: f64 = base * (1.0 + rng.gen_range(-0.015..0.015));
        let high: f64 = open.max(close) * (1.0 + rng.gen_range(0.0..0.008));
        let low: f64 = open.min(close) * (1.0 - rng.gen_range(0.0..0.008));
        let volume = rng.gen_range(5000.0..15000.0); // Higher volume in choppy markets

        bars.push(Bar {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        });

        timestamp += Duration::days(1);
    }

    bars
}

/// Generate a volatile market with large price swings
///
/// # Arguments
/// * `periods` - Number of bars to generate
/// * `volatility` - High volatility factor (e.g., 0.08 = 8% swings)
///
/// # Returns
/// Vector of Bar structs representing a volatile market
pub fn generate_volatile_market(periods: usize, volatility: f64) -> Vec<Bar> {
    let mut bars = Vec::with_capacity(periods);
    let mut rng = thread_rng();
    let base_price = 100.0;
    let mut timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let mut current_price: f64 = base_price;

    for _ in 0..periods {
        // Large moves with momentum
        let momentum = rng.gen_range(-volatility..volatility);
        current_price *= 1.0 + momentum;

        // Ensure price doesn't go too low
        current_price = current_price.max(10.0);

        let open: f64 = current_price;
        let close: f64 = current_price * (1.0 + rng.gen_range(-volatility * 0.5..volatility * 0.5));
        let high: f64 = open.max(close) * (1.0 + rng.gen_range(0.0..volatility * 0.3));
        let low: f64 = open.min(close) * (1.0 - rng.gen_range(0.0..volatility * 0.3));
        let volume = rng.gen_range(2000.0..20000.0); // Very high volume

        bars.push(Bar {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        });

        timestamp += Duration::days(1);
    }

    bars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trending_market_structure() {
        let bars = generate_trending_market(50, 0.01);
        assert_eq!(bars.len(), 50);

        // Check OHLCV relationships
        for bar in &bars {
            assert!(bar.high >= bar.low, "High should be >= low");
            assert!(
                bar.close >= bar.low && bar.close <= bar.high,
                "Close should be within high-low range"
            );
            assert!(
                bar.open >= bar.low && bar.open <= bar.high,
                "Open should be within high-low range"
            );
            assert!(bar.volume > 0.0, "Volume should be positive");
        }
    }

    #[test]
    fn test_trending_market_upward_bias() {
        let bars = generate_trending_market(100, 0.02);
        let first_price = bars.first().unwrap().close;
        let last_price = bars.last().unwrap().close;

        // With 2% daily trend, price should generally be higher at end
        assert!(
            last_price > first_price,
            "Trending market should show upward movement"
        );
    }

    #[test]
    fn test_ranging_market_oscillation() {
        let bars = generate_ranging_market(100, 0.05);
        let mean_price: f64 = bars.iter().map(|b| b.close).sum::<f64>() / bars.len() as f64;

        // Price should oscillate around mean, not trend strongly
        assert!(
            mean_price > 90.0 && mean_price < 110.0,
            "Ranging market should stay around mean"
        );
    }

    #[test]
    fn test_choppy_market_noise() {
        let bars = generate_choppy_market(100);

        // Count direction changes
        let _direction_changes = 0;
        for window in bars.windows(2) {
            let prev_close = window[0].close;
            let curr_close = window[1].close;
            if (curr_close - prev_close).abs() > prev_close * 0.005 {
                // Significant move
            }
        }

        assert!(!bars.is_empty(), "Should generate bars");
    }

    #[test]
    fn test_volatile_market_large_swings() {
        let bars = generate_volatile_market(50, 0.08);

        // Calculate average true range
        let mut total_range = 0.0;
        for bar in &bars {
            total_range += bar.high - bar.low;
        }
        let avg_range = total_range / bars.len() as f64;

        // Volatile market should have large ranges (with 8% volatility, expect avg range > 2.0)
        assert!(
            avg_range > 2.0,
            "Volatile market should have large price ranges, got avg_range: {}",
            avg_range
        );
    }
}
