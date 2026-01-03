//! ML Strategy Adapter
//!
//! Wraps trained ML models into a trading strategy that can be used
//! with the backtest engine.

use super::features::{FeatureConfig, FeatureExtractor};
use super::models::MLModel;
use super::normalization::{Scaler, StandardScaler};
use crate::strategy::{OrderRequest, OrderSide, OrderType, Strategy};
use alphafield_core::Bar;
use serde::{Deserialize, Serialize};

/// Configuration for ML strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLStrategyConfig {
    /// Symbol to trade
    pub symbol: String,
    /// Confidence threshold for generating signals (0-1)
    pub confidence_threshold: f64,
    /// Position sizing (fraction of capital)
    pub position_size: f64,
    /// Use direction (classification) or magnitude (regression)
    pub use_direction: bool,
    /// Minimum prediction magnitude for trades (for regression)
    pub min_magnitude: f64,
    /// Take profit percentage
    pub take_profit: f64,
    /// Stop loss percentage
    pub stop_loss: f64,
}

impl Default for MLStrategyConfig {
    fn default() -> Self {
        Self {
            symbol: "BTCUSDT".to_string(),
            confidence_threshold: 0.6,
            position_size: 1.0,
            use_direction: true,
            min_magnitude: 0.001,
            take_profit: 3.0,
            stop_loss: 2.0,
        }
    }
}

/// ML-based trading strategy
///
/// Uses a trained model to generate buy/sell signals based on
/// feature predictions.
pub struct MLStrategy {
    config: MLStrategyConfig,
    model: Box<dyn MLModel>,
    feature_extractor: FeatureExtractor,
    scaler: StandardScaler,
    /// Buffer of recent bars for feature extraction
    bar_buffer: Vec<Bar>,
    /// Warmup period needed before predictions
    warmup_period: usize,
    /// Current position state
    in_position: bool,
    /// Entry price for exit logic
    entry_price: Option<f64>,
}

impl MLStrategy {
    /// Create a new ML strategy
    pub fn new(
        model: Box<dyn MLModel>,
        scaler: StandardScaler,
        feature_config: FeatureConfig,
        strategy_config: MLStrategyConfig,
    ) -> Self {
        let feature_extractor = FeatureExtractor::new(feature_config);
        // Calculate warmup period based on feature config
        let warmup_period = 60; // Minimum buffer size for feature extraction

        Self {
            config: strategy_config,
            model,
            feature_extractor,
            scaler,
            bar_buffer: Vec::with_capacity(warmup_period + 10),
            warmup_period,
            in_position: false,
            entry_price: None,
        }
    }

    /// Check if strategy is ready to make predictions
    pub fn is_ready(&self) -> bool {
        self.bar_buffer.len() >= self.warmup_period && self.model.is_trained()
    }

    /// Extract current features from buffer
    fn extract_current_features(&self) -> Option<Vec<f64>> {
        if self.bar_buffer.len() < self.warmup_period {
            return None;
        }

        // Extract features from recent bars
        let features = self.feature_extractor.extract(&self.bar_buffer);
        if features.is_empty() {
            return None;
        }

        // Get the last (most recent) feature vector
        let last_features = features.features.last()?.clone();

        // Normalize using the fitted scaler
        if self.scaler.is_fitted() {
            let scaled = self.scaler.transform(&[last_features]);
            scaled.into_iter().next()
        } else {
            Some(last_features)
        }
    }

    /// Make prediction and convert to trading signal
    fn make_prediction(&self, features: &[f64]) -> Option<TradeSignal> {
        let prediction = self.model.predict(features);

        if self.config.use_direction {
            // Classification mode
            if let Some(proba) = self.model.predict_proba(features) {
                if proba >= self.config.confidence_threshold {
                    return Some(TradeSignal::Buy { confidence: proba });
                } else if proba <= 1.0 - self.config.confidence_threshold {
                    return Some(TradeSignal::Sell {
                        confidence: 1.0 - proba,
                    });
                }
            } else {
                // No probability available, use raw prediction
                if prediction > 0.0 {
                    return Some(TradeSignal::Buy { confidence: 0.7 });
                } else if prediction < 0.0 {
                    return Some(TradeSignal::Sell { confidence: 0.7 });
                }
            }
        } else {
            // Regression mode - predict magnitude
            if prediction.abs() >= self.config.min_magnitude {
                if prediction > 0.0 {
                    return Some(TradeSignal::Buy {
                        confidence: prediction.abs().min(1.0),
                    });
                } else {
                    return Some(TradeSignal::Sell {
                        confidence: prediction.abs().min(1.0),
                    });
                }
            }
        }

        None
    }
}

/// Trade signal from ML prediction
#[derive(Debug, Clone)]
enum TradeSignal {
    Buy { confidence: f64 },
    Sell { confidence: f64 },
}

impl Strategy for MLStrategy {
    fn on_bar(&mut self, bar: &Bar) -> crate::error::Result<Vec<OrderRequest>> {
        // Add bar to buffer
        self.bar_buffer.push(*bar);

        // Keep buffer size manageable
        while self.bar_buffer.len() > self.warmup_period + 20 {
            self.bar_buffer.remove(0);
        }

        let mut orders = Vec::new();
        let price = bar.close;

        // Exit logic first (if in position)
        if self.in_position {
            if let Some(entry) = self.entry_price {
                let profit_pct = (price - entry) / entry * 100.0;

                // Take profit
                if profit_pct >= self.config.take_profit {
                    self.in_position = false;
                    self.entry_price = None;
                    orders.push(OrderRequest {
                        symbol: self.config.symbol.clone(),
                        side: OrderSide::Sell,
                        quantity: self.config.position_size,
                        order_type: OrderType::Market,
                    });
                    return Ok(orders);
                }

                // Stop loss
                if profit_pct <= -self.config.stop_loss {
                    self.in_position = false;
                    self.entry_price = None;
                    orders.push(OrderRequest {
                        symbol: self.config.symbol.clone(),
                        side: OrderSide::Sell,
                        quantity: self.config.position_size,
                        order_type: OrderType::Market,
                    });
                    return Ok(orders);
                }
            }
        }

        // Check if ready for predictions
        if !self.is_ready() {
            return Ok(orders);
        }

        // Extract features and make prediction
        if let Some(features) = self.extract_current_features() {
            if let Some(signal) = self.make_prediction(&features) {
                match signal {
                    TradeSignal::Buy { confidence } if !self.in_position => {
                        // Use confidence for position sizing (higher confidence = larger position)
                        let sized_qty = self.config.position_size * confidence.min(1.0);
                        self.in_position = true;
                        self.entry_price = Some(price);
                        orders.push(OrderRequest {
                            symbol: self.config.symbol.clone(),
                            side: OrderSide::Buy,
                            quantity: sized_qty,
                            order_type: OrderType::Market,
                        });
                    }
                    TradeSignal::Sell { confidence } if self.in_position => {
                        // Use confidence for position sizing
                        let sized_qty = self.config.position_size * confidence.min(1.0);
                        self.in_position = false;
                        self.entry_price = None;
                        orders.push(OrderRequest {
                            symbol: self.config.symbol.clone(),
                            side: OrderSide::Sell,
                            quantity: sized_qty,
                            order_type: OrderType::Market,
                        });
                    }
                    _ => {}
                }
            }
        }

        Ok(orders)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ml::models::LinearRegression;
    use chrono::{Duration, Utc};

    fn make_test_bars(n: usize) -> Vec<Bar> {
        let start = Utc::now();
        (0..n)
            .map(|i| {
                let price = 100.0 + (i as f64) * 0.5;
                Bar {
                    timestamp: start + Duration::hours(i as i64),
                    open: price - 0.5,
                    high: price + 1.0,
                    low: price - 1.0,
                    close: price,
                    volume: 1000.0,
                }
            })
            .collect()
    }

    #[test]
    fn test_ml_strategy_creation() {
        let model = Box::new(LinearRegression::new());
        let scaler = StandardScaler::new();
        let feature_config = FeatureConfig::default();
        let strategy_config = MLStrategyConfig::default();

        let strategy = MLStrategy::new(model, scaler, feature_config, strategy_config);
        assert!(!strategy.is_ready()); // Not ready until warmup
    }

    #[test]
    fn test_ml_strategy_warmup() {
        let model = Box::new(LinearRegression::new());
        let scaler = StandardScaler::new();
        let feature_config = FeatureConfig::default();
        let strategy_config = MLStrategyConfig::default();

        let mut strategy = MLStrategy::new(model, scaler, feature_config, strategy_config);

        // Add bars during warmup
        let bars = make_test_bars(100);
        for bar in &bars[..50] {
            let _ = strategy.on_bar(bar);
        }
        assert!(!strategy.is_ready()); // Model not trained

        for bar in &bars[50..] {
            let _ = strategy.on_bar(bar);
        }
        // Still not ready because model isn't trained
        assert!(!strategy.is_ready());
    }

    #[test]
    fn test_strategy_config_defaults() {
        let config = MLStrategyConfig::default();
        assert_eq!(config.confidence_threshold, 0.6);
        assert!(config.use_direction);
    }
}
