# AI Agent Strategy Implementation Template

## Overview
This template provides a standardized structure for implementing trading strategies in AlphaField. AI agents should follow this template exactly to ensure consistency, maintainability, and quality across all strategy implementations.

## Required Components

Every strategy implementation MUST include:

1. **Strategy Struct**: Holds strategy state and parameters
2. **Strategy Trait Implementation**: Implements `Strategy` and `MetadataStrategy`
3. **Constructor**: Creates strategy instances with validation
4. **Signal Generation**: Implements `on_bar` and `on_tick` methods
5. **Unit Tests**: Tests for all major functionality
6. **Hypothesis Document**: Complete hypothesis documentation following the template
7. **Module Export**: Added to category module exports

---

## Code Template

```rust
// File: crates/strategy/src/strategies/[category]/[strategy_name].rs

use crate::core::{Bar, Tick, Quote, Signal, SignalType, Strategy};
use crate::framework::{
    MetadataStrategy, StrategyMetadata, StrategyCategory, MarketRegime,
    RiskProfile, VolatilityLevel, CorrelationSensitivity
};
use crate::indicators::{Sma, Ema, Rsi, Indicator}; // Add indicators as needed
use std::collections::VecDeque;
use chrono::{DateTime, Utc};

// ============ Strategy Struct ============

/// [Brief description of what this strategy does]
/// 
/// # Strategy Logic
/// - **Entry**: [When buy signals are generated]
/// - **Exit**: [When sell signals are generated]
///
/// # Example
/// ```
/// use alphafield_strategy::strategies::[category]::[StrategyName];
/// use alphafield_strategy::config::[StrategyName]Config;
/// 
/// let config = [StrategyName]Config::new([param1], [param2]);
/// let strategy = [StrategyName]::from_config(config);
/// ```
pub struct [StrategyName] {
    // Required parameters
    [param1_type] [param1_name],  // [Description]
    [param2_type] [param2_name],  // [Description]
    
    // State variables
    [state1_type] [state1_name],  // [Description]
    [state2_type] [state2_name],  // [Description]
}

// ============ Constructor ============

impl [StrategyName] {
    /// Creates a new [StrategyName] strategy instance
    /// 
    /// # Arguments
    /// * `[param1_name]` - [Description]
    /// * `[param2_name]` - [Description]
    /// 
    /// # Returns
    /// * `Result<Self>` - New strategy instance or error
    /// 
    /// # Errors
    /// Returns error if parameters are invalid
    /// 
    /// # Example
    /// ```
    /// let strategy = [StrategyName]::new([param1_value], [param2_value])
    ///     .expect("Valid parameters");
    /// ```
    pub fn new([param1_name]: [param1_type], [param2_name]: [param2_type]) -> Result<Self, String> {
        // Validate parameters
        if [validation_condition] {
            return Err("[Error message]".to_string());
        }
        
        // Additional validations
        if [param1_name] < [min_value] || [param1_name] > [max_value] {
            return Err(format!("{} must be between {} and {}", 
                           "[param1_name]", [min_value], [max_value]));
        }

        Ok([StrategyName] {
            [param1_name],
            [param2_name],
            
            // Initialize state
            [state1_name]: [initial_value],
            [state2_name]: [initial_value],
        })
    }
    
    /// Creates strategy from configuration object
    /// 
    /// # Arguments
    /// * `config` - Strategy configuration
    /// 
    /// # Returns
    /// * `Self` - New strategy instance
    pub fn from_config(config: [StrategyName]Config) -> Self {
        Self::new(config.[param1_name], config.[param2_name])
            .expect("Config should be validated")
    }
}

// ============ Default Implementation ============

impl Default for [StrategyName] {
    fn default() -> Self {
        Self::new([default_param1], [default_param2])
            .expect("Default parameters should be valid")
    }
}

// ============ Strategy Trait Implementation ============

impl Strategy for [StrategyName] {
    fn name(&self) -> &str {
        "[StrategyName]"
    }

    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        // Update indicators/state
        [update_indicators_or_state];

        // Check entry conditions
        if [entry_condition_1] && [entry_condition_2] && [entry_condition_3] {
            return Some(vec![Signal {
                timestamp: bar.timestamp,
                symbol: "UNKNOWN".to_string(), // Will be set by backtest engine
                signal_type: SignalType::Buy,
                strength: [calculate_signal_strength],
                metadata: Some("[Reason for entry]".to_string()),
            }]);
        }

        // Check exit conditions
        if [exit_condition_1] || [exit_condition_2] || [exit_condition_3] {
            return Some(vec![Signal {
                timestamp: bar.timestamp,
                symbol: "UNKNOWN".to_string(),
                signal_type: SignalType::Sell,
                strength: [calculate_signal_strength],
                metadata: Some("[Reason for exit]".to_string()),
            }]);
        }

        None
    }

    fn on_tick(&mut self, _tick: &Tick) -> Option<Signal> {
        // Most strategies don't use ticks, but can if needed
        // Return Some(Signal { ... }) if tick-based trading
        None
    }
    
    fn on_quote(&mut self, _quote: &Quote) -> Option<Signal> {
        // Most strategies don't use quotes, but can if needed
        // Return Some(Signal { ... }) if quote-based trading
        None
    }
}

// ============ MetadataStrategy Trait Implementation ============

impl MetadataStrategy for [StrategyName] {
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "[StrategyName]".to_string(),
            category: StrategyCategory::[StrategyCategory],
            sub_type: Some("[sub_type]".to_string()),
            description: "[Full description of the strategy, what it does, and how it works]".to_string(),
            hypothesis_path: "hypotheses/[category]/[strategy_name].md".to_string(),
            required_indicators: vec![
                "[indicator1]".to_string(),
                "[indicator2]".to_string(),
            ],
            expected_regimes: vec![
                MarketRegime::[Regime1],
                MarketRegime::[Regime2],
            ],
            risk_profile: RiskProfile {
                max_drawdown_expected: [value],
                volatility_level: VolatilityLevel::[Level],
                correlation_sensitivity: CorrelationSensitivity::[Sensitivity],
                leverage_requirement: [value],
            },
        }
    }

    fn category(&self) -> StrategyCategory {
        StrategyCategory::[StrategyCategory]
    }
}

// ============ Unit Tests ============

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    /// Helper to create test bars
    fn create_test_bar(timestamp: i64, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Bar {
        Bar {
            timestamp: Utc.timestamp(timestamp, 0),
            open,
            high,
            low,
            close,
            volume,
        }
    }

    #[test]
    fn test_strategy_creation() {
        let strategy = [StrategyName]::new([param1_value], [param2_value]);
        assert!(strategy.is_ok());
        
        let s = strategy.unwrap();
        assert_eq!(s.[param1_name], [param1_value]);
        assert_eq!(s.[param2_name], [param2_value]);
    }

    #[test]
    fn test_invalid_parameters() {
        let strategy = [StrategyName]::new([invalid_param1], [invalid_param2]);
        assert!(strategy.is_err());
    }
    
    #[test]
    fn test_default_creation() {
        let strategy = [StrategyName]::default();
        assert_eq!(strategy.name(), "[StrategyName]");
    }

    #[test]
    fn test_entry_signal_generation() {
        let mut strategy = [StrategyName]::default();
        
        // Create bars that should trigger entry
        let bar1 = create_test_bar(0, 100.0, 105.0, 95.0, 102.0, 1000.0);
        let signal1 = strategy.on_bar(&bar1);
        assert!(signal1.is_none()); // No signal yet

        let bar2 = create_test_bar(1, 102.0, 108.0, 100.0, 106.0, 1200.0);
        let signal2 = strategy.on_bar(&bar2);
        assert!(signal2.is_some()); // Should have entry signal
        
        if let Some(signals) = signal2 {
            assert_eq!(signals.len(), 1);
            let signal = &signals[0];
            assert_eq!(signal.signal_type, SignalType::Buy);
            assert!(signal.strength > 0.0);
            assert!(signal.metadata.is_some());
        }
    }

    #[test]
    fn test_exit_signal_generation() {
        let mut strategy = [StrategyName]::default();
        
        // Generate entry first
        let entry_bar = create_test_bar(0, 100.0, 105.0, 95.0, 102.0, 1000.0);
        strategy.on_bar(&entry_bar);
        
        // Create bars that should trigger exit
        let exit_bar = create_test_bar(1, 102.0, 103.0, 90.0, 91.0, 1000.0);
        let signals = strategy.on_bar(&exit_bar);
        assert!(signals.is_some());
        
        if let Some(sigs) = signals {
            assert_eq!(sigs.len(), 1);
            let signal = &sigs[0];
            assert_eq!(signal.signal_type, SignalType::Sell);
            assert!(signal.metadata.is_some());
        }
    }
    
    #[test]
    fn test_multiple_signals() {
        let mut strategy = [StrategyName]::default();
        
        // Test that strategy can generate multiple signals over time
        // and handles state correctly between signals
        let bars = vec![
            create_test_bar(0, 100.0, 101.0, 99.0, 100.5, 1000.0),
            create_test_bar(1, 100.5, 102.0, 100.0, 101.5, 1100.0),
            create_test_bar(2, 101.5, 103.0, 101.0, 102.5, 1200.0),
            create_test_bar(3, 102.5, 104.0, 102.0, 103.5, 1300.0),
            create_test_bar(4, 103.5, 105.0, 103.0, 104.5, 1400.0),
        ];
        
        let mut signal_count = 0;
        for bar in bars {
            if let Some(signals) = strategy.on_bar(&bar) {
                signal_count += signals.len();
            }
        }
        
        // Assert expected number of signals
        assert_eq!(signal_count, [expected_signal_count]);
    }

    #[test]
    fn test_strategy_state_persistence() {
        let mut strategy = [StrategyName]::default();
        
        // Feed multiple bars and verify state is maintained
        for i in 0..10 {
            let price = 100.0 + i as f64;
            let bar = create_test_bar(i, price, price + 1.0, price - 1.0, price + 0.5, 1000.0);
            strategy.on_bar(&bar);
        }
        
        // Verify internal state (implementation-specific)
        // assert_eq!(strategy.[state_variable], [expected_value]);
    }

    #[test]
    fn test_metadata() {
        let strategy = [StrategyName]::default();
        let metadata = strategy.metadata();
        
        assert_eq!(metadata.name, "[StrategyName]");
        assert_eq!(metadata.category, StrategyCategory::[StrategyCategory]);
        assert_eq!(metadata.sub_type, Some("[sub_type]".to_string()));
        assert!(metadata.description.len() > 0);
        assert!(metadata.required_indicators.len() > 0);
        assert!(metadata.expected_regimes.len() > 0);
    }
    
    #[test]
    fn test_edge_cases() {
        let mut strategy = [StrategyName]::default();
        
        // Test with equal OHLC
        let flat_bar = create_test_bar(0, 100.0, 100.0, 100.0, 100.0, 1000.0);
        let signal = strategy.on_bar(&flat_bar);
        // Assert expected behavior for flat bar
        
        // Test with extreme values
        let extreme_bar = create_test_bar(1, 1.0, 1000000.0, 0.0001, 500000.0, 1000000.0);
        let signal = strategy.on_bar(&extreme_bar);
        // Assert expected behavior for extreme values
    }

    // Add more tests as needed for your specific strategy
}
```

---

## Implementation Checklist

Use this checklist to ensure your strategy is complete:

### Code Implementation
- [ ] Strategy struct with parameters and state defined
- [ ] Constructor with parameter validation
- [ ] `Strategy` trait implementation (name, on_bar, on_tick, on_quote)
- [ ] `MetadataStrategy` trait implementation (metadata, category)
- [ ] `on_bar` method with entry/exit logic
- [ ] `on_tick` method (even if just returns None)
- [ ] `on_quote` method (even if just returns None)
- [ ] `Default` implementation
- [ ] Comprehensive unit tests

### Documentation
- [ ] Hypothesis document created in `doc/phase_12/hypotheses/[category]/[strategy_name].md`
- [ ] All sections of hypothesis template completed
- [ ] Code comments explain logic
- [ ] Documentation examples compile and run
- [ ] Metadata fields are accurate and complete

### Integration
- [ ] Strategy added to category module exports (`crates/strategy/src/strategies/[category]/mod.rs`)
- [ ] Strategy added to main module exports if needed
- [ ] Registered in strategy registry (in main or initialization code)
- [ ] API endpoint can retrieve strategy metadata
- [ ] Strategy can be selected in backtest UI

### Testing
- [ ] Unit tests cover entry logic
- [ ] Unit tests cover exit logic
- [ ] Unit tests cover edge cases
- [ ] Unit tests cover parameter validation
- [ ] Integration tests pass (backtest execution)
- [ ] Tests cover state management between bars

### Validation
- [ ] Backtest runs successfully on real data
- [ ] Walk-forward validation passes
- [ ] Monte Carlo simulation completed
- [ ] Performance metrics calculated and reasonable
- [ ] Failure modes documented in hypothesis
- [ ] Risk profile matches backtest results

---

## Common Patterns

### Indicator Window with VecDeque
```rust
use std::collections::VecDeque;

pub struct ExampleStrategy {
    window_size: usize,
    prices: VecDeque<f64>,
}

impl ExampleStrategy {
    pub fn new(window_size: usize) -> Result<Self> {
        if window_size < 2 {
            return Err("Window size must be >= 2".to_string());
        }
        
        Ok(ExampleStrategy {
            window_size,
            prices: VecDeque::with_capacity(window_size),
        })
    }
    
    fn calculate_sma(&self) -> Option<f64> {
        if self.prices.len() < self.window_size {
            return None;
        }
        
        let sum: f64 = self.prices.iter().sum();
        Some(sum / self.window_size as f64)
    }
    
    fn calculate_stddev(&self) -> Option<f64> {
        if self.prices.len() < self.window_size {
            return None;
        }
        
        let sma = self.calculate_sma()?;
        let variance: f64 = self.prices.iter()
            .map(|&x| (x - sma).powi(2))
            .sum::<f64>() / self.window_size as f64;
        Some(variance.sqrt())
    }
}

impl Strategy for ExampleStrategy {
    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        // Add to window
        self.prices.push_back(bar.close);
        
        // Maintain window size
        if self.prices.len() > self.window_size {
            self.prices.pop_front();
        }
        
        // Use indicator
        if let Some(sma) = self.calculate_sma() {
            if let Some(stddev) = self.calculate_stddev() {
                // Generate signals based on SMA and stddev
                let z_score = (bar.close - sma) / stddev;
                
                if z_score > 2.0 {
                    // Overbought - sell signal
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Sell,
                        strength: (z_score / 5.0).min(1.0), // Normalize to 0-1
                        metadata: Some(format!("Overbought: Z-score {:.2}", z_score)),
                    }]);
                } else if z_score < -2.0 {
                    // Oversold - buy signal
                    return Some(vec![Signal {
                        timestamp: bar.timestamp,
                        symbol: "UNKNOWN".to_string(),
                        signal_type: SignalType::Buy,
                        strength: ((-z_score) / 5.0).min(1.0),
                        metadata: Some(format!("Oversold: Z-score {:.2}", z_score)),
                    }]);
                }
            }
        }
        
        None
    }
    
    fn on_tick(&mut self, _tick: &Tick) -> Option<Signal> {
        None
    }
    
    fn on_quote(&mut self, _quote: &Quote) -> Option<Signal> {
        None
    }
}
```

### Tracking Position State
```rust
pub struct ExampleStrategy {
    in_position: bool,
    entry_price: Option<f64>,
    entry_time: Option<DateTime<Utc>>,
    highest_price_since_entry: Option<f64>,
}

impl Strategy for ExampleStrategy {
    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        if !self.in_position {
            // Check entry conditions
            if [entry_condition] {
                self.in_position = true;
                self.entry_price = Some(bar.close);
                self.entry_time = Some(bar.timestamp);
                self.highest_price_since_entry = Some(bar.close);
                
                return Some(vec![Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Buy,
                    strength: 1.0,
                    metadata: Some("Entry signal".to_string()),
                }]);
            }
        } else {
            // Track highest price for trailing stops
            if let Some(ref mut highest) = self.highest_price_since_entry {
                if bar.close > *highest {
                    *highest = bar.close;
                }
            }
            
            // Check exit conditions
            if [exit_condition_1] {
                self.in_position = false;
                let entry_price = self.entry_price.unwrap();
                let profit_pct = (bar.close - entry_price) / entry_price * 100.0;
                
                self.entry_price = None;
                self.entry_time = None;
                self.highest_price_since_entry = None;
                
                return Some(vec![Signal {
                    timestamp: bar.timestamp,
                    symbol: "UNKNOWN".to_string(),
                    signal_type: SignalType::Sell,
                    strength: 1.0,
                    metadata: Some(format!("Exit: {:.1}% profit", profit_pct)),
                }]);
            }
            
            // Trailing stop example
            if let Some(entry_price) = self.entry_price {
                if let Some(highest) = self.highest_price_since_entry {
                    let trailing_stop_pct = 5.0; // 5% trailing stop
                    let stop_price = highest * (1.0 - trailing_stop_pct / 100.0);
                    
                    if bar.close < stop_price {
                        self.in_position = false;
                        self.entry_price = None;
                        self.entry_time = None;
                        self.highest_price_since_entry = None;
                        
                        return Some(vec![Signal {
                            timestamp: bar.timestamp,
                            symbol: "UNKNOWN".to_string(),
                            signal_type: SignalType::Sell,
                            strength: 1.0,
                            metadata: Some(format!("Trailing stop hit at {:.2}", bar.close)),
                        }]);
                    }
                }
            }
        }
        
        None
    }
    
    fn on_tick(&mut self, _tick: &Tick) -> Option<Signal> {
        None
    }
    
    fn on_quote(&mut self, _quote: &Quote) -> Option<Signal> {
        None
    }
}
```

### Multiple Indicator Filters
```rust
pub struct ExampleStrategy {
    // RSI parameters
    rsi_period: usize,
    rsi_overbought: f64,
    rsi_oversold: f64,
    rsi: Rsi,
    
    // Moving average parameters
    sma_short_period: usize,
    sma_long_period: usize,
    sma_short: Sma,
    sma_long: Sma,
    
    // Volume parameters
    avg_volume_period: usize,
    avg_volume: Sma,
    
    // State
    last_short_sma: Option<f64>,
    last_long_sma: Option<f64>,
}

impl ExampleStrategy {
    pub fn new(
        rsi_period: usize,
        sma_short: usize,
        sma_long: usize,
        avg_volume_period: usize,
    ) -> Result<Self> {
        if sma_short >= sma_long {
            return Err("Short SMA must be less than long SMA".to_string());
        }
        
        Ok(ExampleStrategy {
            rsi_period,
            rsi_overbought: 70.0,
            rsi_oversold: 30.0,
            rsi: Rsi::new(rsi_period),
            
            sma_short_period,
            sma_long_period,
            sma_short: Sma::new(sma_short),
            sma_long: Sma::new(sma_long),
            
            avg_volume_period,
            avg_volume: Sma::new(avg_volume_period),
            
            last_short_sma: None,
            last_long_sma: None,
        })
    }
}

impl Strategy for ExampleStrategy {
    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        // Update all indicators
        let rsi_value = self.rsi.update(bar.close);
        let short_sma = self.sma_short.update(bar.close);
        let long_sma = self.sma_long.update(bar.close);
        let avg_vol = self.avg_volume.update(bar.volume);
        
        // Check if indicators are ready
        let rsi = match rsi_value {
            Some(r) if r.is_finite() => r,
            _ => return None,
        };
        
        let short = match short_sma {
            Some(s) if s.is_finite() => s,
            _ => return None,
        };
        
        let long = match long_sma {
            Some(l) if l.is_finite() => l,
            _ => return None,
        };
        
        let volume_ok = match avg_vol {
            Some(v) if v > 0.0 => bar.volume > v * 1.5, // Volume > 1.5x average
            _ => false,
        };
        
        // ENTRY CONDITIONS (all must be true)
        let rsi_ok = rsi < self.rsi_oversold;
        let golden_cross = self.last_short_sma <= self.last_long_sma && short > long;
        
        if rsi_ok && golden_cross && volume_ok {
            // Save for next comparison
            self.last_short_sma = Some(short);
            self.last_long_sma = Some(long);
            
            return Some(vec![Signal {
                timestamp: bar.timestamp,
                symbol: "UNKNOWN".to_string(),
                signal_type: SignalType::Buy,
                strength: 0.8,
                metadata: Some(format!(
                    "Entry: RSI {:.1} (oversold), Golden Cross, Vol {:.0}x avg",
                    rsi,
                    bar.volume / avg_vol.unwrap()
                )),
            }]);
        }
        
        // EXIT CONDITIONS (any can be true)
        let rsi_overbought = rsi > self.rsi_overbought;
        let death_cross = self.last_short_sma >= self.last_long_sma && short < long;
        
        if rsi_overbought || death_cross {
            self.last_short_sma = Some(short);
            self.last_long_sma = Some(long);
            
            let reason = if rsi_overbought {
                "RSI overbought".to_string()
            } else {
                "Death cross".to_string()
            };
            
            return Some(vec![Signal {
                timestamp: bar.timestamp,
                symbol: "UNKNOWN".to_string(),
                signal_type: SignalType::Sell,
                strength: 0.8,
                metadata: Some(reason),
            }]);
        }
        
        // Save current values for next iteration
        self.last_short_sma = Some(short);
        self.last_long_sma = Some(long);
        
        None
    }
    
    fn on_tick(&mut self, _tick: &Tick) -> Option<Signal> {
        None
    }
    
    fn on_quote(&mut self, _quote: &Quote) -> Option<Signal> {
        None
    }
}
```

### Signal Strength Calculation
```rust
impl Strategy for ExampleStrategy {
    fn on_bar(&mut self, bar: &Bar) -> Option<Vec<Signal>> {
        // Calculate multiple factors for signal strength
        let trend_strength = self.calculate_trend_strength();
        let volume_strength = self.calculate_volume_strength(bar);
        let momentum_strength = self.calculate_momentum_strength();
        
        // Combine factors with weights
        let combined_strength = 
            trend_strength * 0.4 +
            volume_strength * 0.3 +
            momentum_strength * 0.3;
        
        // Clamp to [0, 1]
        let signal_strength = combined_strength.max(0.0).min(1.0);
        
        if signal_strength > 0.5 {
            return Some(vec![Signal {
                timestamp: bar.timestamp,
                symbol: "UNKNOWN".to_string(),
                signal_type: SignalType::Buy,
                strength: signal_strength,
                metadata: Some(format!(
                    "Strength: {:.2} (Trend: {:.2}, Vol: {:.2}, Mom: {:.2})",
                    signal_strength, trend_strength, volume_strength, momentum_strength
                )),
            }]);
        }
        
        None
    }
    
    fn calculate_trend_strength(&self) -> f64 {
        // Return 0-1 based on how strong the trend is
        // Example: based on slope of SMA
        0.8 // Placeholder
    }
    
    fn calculate_volume_strength(&self, bar: &Bar) -> f64 {
        // Return 0-1 based on volume above average
        let avg_vol = self.avg_volume.value().unwrap_or(1.0);
        let vol_ratio = bar.volume / avg_vol;
        
        // Clamp: 1x avg = 0.5, 2x avg = 0.75, 3x avg = 0.875
        (vol_ratio / (vol_ratio + 1.0)).min(1.0)
    }
    
    fn calculate_momentum_strength(&self) -> f64 {
        // Return 0-1 based on momentum
        0.6 // Placeholder
    }
}
```

---

## Validation Checklist

Before marking a strategy as complete, verify:

### Code Quality
- [ ] Code compiles without warnings (`cargo clippy -- -D warnings`)
- [ ] All tests pass (`cargo test`)
- [ ] Code follows Rust best practices
- [ ] Error handling is comprehensive and idiomatic
- [ ] State management is correct and thread-safe (if applicable)
- [ ] No unwrap() or expect() in production code without justification

### Documentation
- [ ] Hypothesis document is complete and follows template
- [ ] All parameters are documented in hypothesis
- [ ] Entry/exit rules are clearly defined
- [ ] Failure modes are identified and mitigated
- [ ] Risk profile is accurate and matches backtest results
- [ ] Code comments explain non-obvious logic
- [ ] Examples in documentation compile and run

### Testing
- [ ] Unit tests cover all scenarios
- [ ] Edge cases are tested (empty data, single bar, extreme values)
- [ ] Parameter validation works correctly
- [ ] Reset functionality works (if applicable)
- [ ] Integration tests cover backtest execution
- [ ] Tests are fast (no unnecessary work in tests)

### Performance
- [ ] Backtest runs successfully on historical data
- [ ] Performance metrics are reasonable (not suspiciously good)
- [ ] Walk-forward validation passes or is documented
- [ ] No obvious overfitting indicators
- [ ] Strategy completes backtests in reasonable time

### Integration
- [ ] API endpoint returns correct strategy data
- [ ] Database integration works (if applicable)
- [ ] Dashboard can display strategy correctly
- [ ] Strategy can be selected for backtesting
- [ ] Strategy registered in registry correctly

---

## Troubleshooting

### Common Issues and Solutions

**Issue**: "Strategy generates too many signals (whipsaws)"
- **Solution**: 
  - Add additional filters (volume, time since last signal)
  - Increase parameter thresholds
  - Add minimum hold period between signals
  - Add confirmation requirements

**Issue**: "Strategy never generates signals"
- **Solution**:
  - Lower thresholds for signal generation
  - Remove or relax some filters
  - Check indicator calculations for bugs
  - Verify data is being fed correctly
  - Add debug logging to see indicator values

**Issue**: "Strategy performs poorly in backtest"
- **Solution**:
  - Optimize parameters using walk-forward analysis
  - Add regime filters (avoid sideways markets)
  - Adjust entry/exit rules
  - Add stop-loss or take-profit
  - Consider if hypothesis is flawed

**Issue**: "Strategy fails walk-forward validation"
- **Solution**:
  - Parameters are likely overfitted to training period
  - Reduce number of parameters
  - Increase parameter ranges
  - Use simpler logic
  - Add adaptive mechanisms

**Issue**: "Tests pass but backtest fails"
- **Solution**:
  - Check state management between bars
  - Verify indicator calculations
  - Ensure signals are correctly formatted
  - Check for off-by-one errors in arrays/windows
  - Verify timestamp handling

**Issue**: "Memory usage is high"
- **Solution**:
  - Limit window sizes for indicators
  - Use fixed-size arrays instead of VecDeque where possible
  - Clear old data periodically
  - Check for memory leaks in state management

**Issue**: "Strategy is too slow"
- **Solution**:
  - Cache repeated calculations
  - Use efficient data structures
  - Avoid unnecessary allocations in on_bar
  - Profile code to find bottlenecks

**Issue**: "Strategy behaves differently on different timeframes"
- **Solution**:
  - Adjust parameters for each timeframe
  - Document expected behavior per timeframe
  - Consider making parameters timeframe-dependent
  - Test on multiple timeframes during development

---

## Example: Complete Strategy Implementation

For a complete, working example of a strategy implementation, including:
- Full Rust code with all components
- Complete hypothesis document
- Comprehensive tests
- Backtest results
- Validation outcomes

See `doc/phase_12/examples/golden_cross_complete.md` (to be created in future phases).

### Key Takeaways from Example:
1. **Structure**: Follow the exact structure from this template
2. **Documentation**: Hypothesis document is as important as code
3. **Testing**: Tests cover all scenarios, including edge cases
4. **Validation**: Strategy is validated before deployment
5. **Integration**: Strategy integrates with registry and API

---

## Best Practices

1. **Keep it Simple**: Start with simple logic, add complexity only if justified
2. **Test Early**: Write tests alongside code, not as an afterthought
3. **Document Decisions**: Explain WHY you chose specific parameters or logic
4. **Validate Rigorously**: Don't skip validation steps
5. **Learn from Failures**: Document what doesn't work and why
6. **Iterate Continuously**: Improve strategy based on feedback and results

---

## Resources

### Internal Documentation
- [AlphaField Architecture](../../architecture.md)
- [AlphaField API](../../api.md)
- [Phase 12 Plan](plan.md)
- [Core Types Documentation](../../crates/core/src/lib.rs)

### External Resources
- [Rust Book](https://doc.rust-lang.org/book/)
- [Chrono Documentation](https://docs.rs/chrono/)
- [Serde Documentation](https://docs.rs/serde/)
- [QuantConnect Documentation](https://www.quantconnect.com/docs/) - For strategy ideas

### Strategy Research
- [Quantopian Papers](https://www.quantopian.com/posts#t-research)
- [SSRN Finance Papers](https://ssrn.com/index.cfm/en/fen/)
- [Arxiv Quantitative Finance](https://arxiv.org/list/q-fin.RE)
- [TradingView Scripts](https://www.tradingview.com/scripts/) - For implementation ideas