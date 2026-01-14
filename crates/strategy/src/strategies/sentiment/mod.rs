//! Sentiment-Based Trading Strategies
//!
//! This module contains sentiment-focused strategies that use sentiment indicators
//! to generate trading signals. These strategies are designed to:
//! - Follow sentiment momentum trends
//! - Detect price-sentiment divergences for reversal signals
//! - Adapt sentiment interpretation based on market regimes
//! - Use technical sentiment indicators derived from price action (RSI, momentum, volume)
//!
//! Note: This module currently implements strategies using AssetSentiment (technical
//! sentiment calculated from price action). True sentiment-based strategies requiring
//! external data sources (Fear & Greed Index, news sentiment, social volume, etc.) are
//! deferred pending API infrastructure setup.

pub mod divergence_strategy;
pub mod regime_sentiment;
pub mod sentiment_momentum;

// Re-export strategies for convenience
pub use divergence_strategy::DivergenceStrategy;
pub use regime_sentiment::RegimeSentimentStrategy;
pub use sentiment_momentum::SentimentMomentumStrategy;
