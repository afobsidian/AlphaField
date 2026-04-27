//! Strategies API - Strategy Management Endpoints
//!
//! This module provides REST API endpoints for managing trading strategies,
//! including listing strategies, retrieving details, and filtering by category or regime.

use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Import strategy framework types
use alphafield_strategy::{
    MarketRegime, StrategyCategory, StrategyMetadata, StrategyRegistry, StrategyWithMetadata,
};

// Re-use AppState from api.rs
use crate::api::AppState;

// ============================================================================
// API Types
// ============================================================================

/// Summary view of a strategy for listing endpoints
#[derive(Debug, Clone, Serialize)]
pub struct StrategySummary {
    pub name: String,
    pub category: String,
    pub sub_type: Option<String>,
    pub description: String,
    pub expected_regimes: Vec<String>,
    pub risk_profile: RiskProfileSummary,
}

/// Summary of risk profile for API responses
#[derive(Debug, Clone, Serialize)]
pub struct RiskProfileSummary {
    pub max_drawdown_expected: f64,
    pub volatility_level: String,
    pub correlation_sensitivity: String,
    pub leverage_requirement: f64,
}

/// Query parameters for strategy listing
#[derive(Debug, Deserialize)]
pub struct StrategyQuery {
    pub category: Option<String>,
    pub regime: Option<String>,
}

/// Detailed strategy information
#[derive(Debug, Clone, Serialize)]
pub struct StrategyDetails {
    pub metadata: StrategyMetadata,
    pub registered: bool,
}

/// Strategy count summary
#[derive(Debug, Serialize)]
pub struct StrategyCountSummary {
    pub total: usize,
    pub by_category: std::collections::HashMap<String, usize>,
}

/// Error response for API errors
#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub code: String,
    pub details: Option<String>,
}

/// Application error type for strategy API
#[derive(Debug)]
pub struct AppError {
    pub error: String,
    pub code: String,
    pub details: Option<String>,
}

impl AppError {
    pub fn not_found(name: &str) -> Self {
        Self {
            error: format!("Strategy '{}' not found", name),
            code: "STRATEGY_NOT_FOUND".to_string(),
            details: Some(format!("No strategy with name '{}' is registered", name)),
        }
    }

    pub fn invalid_filter(filter: &str, value: &str) -> Self {
        Self {
            error: format!("Invalid {} value: {}", filter, value),
            code: "INVALID_FILTER".to_string(),
            details: Some(format!(
                "Valid values for {}: {}",
                filter,
                Self::get_valid_values(filter)
            )),
        }
    }

    pub fn internal_error(message: &str) -> Self {
        Self {
            error: "Internal server error".to_string(),
            code: "INTERNAL_ERROR".to_string(),
            details: Some(message.to_string()),
        }
    }

    fn get_valid_values(filter: &str) -> String {
        match filter {
            "category" => {
                "TrendFollowing, MeanReversion, Momentum, VolatilityBased, SentimentBased, MultiIndicator"
            }
            "regime" => {
                "Bull, Bear, Sideways, HighVolatility, LowVolatility, Trending, Ranging"
            }
            _ => "unknown filter",
        }
        .to_string()
    }
}

// Convert to axum response
impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match self.code.as_str() {
            "STRATEGY_NOT_FOUND" => axum::http::StatusCode::NOT_FOUND,
            "INVALID_FILTER" => axum::http::StatusCode::BAD_REQUEST,
            _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(ApiErrorResponse {
            error: self.error,
            code: self.code,
            details: self.details,
        });

        (status, body).into_response()
    }
}

// ============================================================================
// Registry Initialization
// ============================================================================

/// Initialize the global strategy registry with all available strategies
/// This function should be called during application startup to populate
/// the registry with initial strategies.
pub fn initialize_registry() -> Arc<StrategyRegistry> {
    let registry = Arc::new(StrategyRegistry::new());

    // ------------------------------------------------------------------------
    // Trend Following Strategies (Phase 12.2)
    // ------------------------------------------------------------------------

    // Register Golden Cross strategy (trend following)
    let golden_cross = Arc::new(alphafield_strategy::strategies::GoldenCrossStrategy::new(
        10, 30,
    )) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(golden_cross) {
        eprintln!("Failed to register Golden Cross strategy: {}", e);
    }

    // Register Breakout strategy (trend following)
    let breakout =
        Arc::new(alphafield_strategy::strategies::trend_following::BreakoutStrategy::new(20))
            as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(breakout) {
        eprintln!("Failed to register Breakout strategy: {}", e);
    }

    // Register MA Crossover strategy (trend following)
    let ma_crossover = Arc::new(
        alphafield_strategy::strategies::trend_following::MACrossoverStrategy::new(10, 30),
    ) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(ma_crossover) {
        eprintln!("Failed to register MA Crossover strategy: {}", e);
    }

    // Register Adaptive MA KAMA strategy (trend following)
    let adaptive_ma = Arc::new(
        alphafield_strategy::strategies::trend_following::AdaptiveMAStrategy::new(10, 30, 10),
    ) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(adaptive_ma) {
        eprintln!("Failed to register Adaptive MA strategy: {}", e);
    }

    // Register Triple MA strategy (trend following)
    let triple_ma = Arc::new(
        alphafield_strategy::strategies::trend_following::TripleMAStrategy::new(5, 15, 30),
    ) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(triple_ma) {
        eprintln!("Failed to register Triple MA strategy: {}", e);
    }

    // Register MACD Trend strategy (trend following)
    let macd_trend = Arc::new(
        alphafield_strategy::strategies::trend_following::MacdTrendStrategy::new(12, 26, 9),
    ) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(macd_trend) {
        eprintln!("Failed to register MACD Trend strategy: {}", e);
    }

    // Register Parabolic SAR strategy (trend following)
    let parabolic_sar = Arc::new(
        alphafield_strategy::strategies::trend_following::ParabolicSARStrategy::new(0.02, 0.2),
    ) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(parabolic_sar) {
        eprintln!("Failed to register Parabolic SAR strategy: {}", e);
    }

    // ------------------------------------------------------------------------
    // Other Strategy Families
    // ------------------------------------------------------------------------

    // Register Bollinger Bands Mean Reversion strategy
    let mean_reversion =
        Arc::new(alphafield_strategy::strategies::BollingerBandsStrategy::new(20, 2.0))
            as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(mean_reversion) {
        eprintln!(
            "Failed to register Bollinger Bands Mean Reversion strategy: {}",
            e
        );
    }

    // ------------------------------------------------------------------------
    // Mean Reversion Strategies (Phase 12.3)
    // ------------------------------------------------------------------------

    // Register RSI Reversion strategy
    let rsi_reversion = Arc::new(
        alphafield_strategy::strategies::mean_reversion::RSIReversionStrategy::new(14, 30.0, 70.0),
    ) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(rsi_reversion) {
        eprintln!("Failed to register RSI Reversion strategy: {}", e);
    }

    // Register Stochastic Reversion strategy
    let stoch_reversion = Arc::new(
        alphafield_strategy::strategies::mean_reversion::StochReversionStrategy::new(14, 3),
    ) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(stoch_reversion) {
        eprintln!("Failed to register Stochastic Reversion strategy: {}", e);
    }

    // Register Z-Score Reversion strategy
    let zscore_reversion =
        Arc::new(alphafield_strategy::strategies::mean_reversion::ZScoreReversionStrategy::new(20))
            as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(zscore_reversion) {
        eprintln!("Failed to register Z-Score Reversion strategy: {}", e);
    }

    // Register Price Channel (Donchian) Reversion strategy
    let price_channel =
        Arc::new(alphafield_strategy::strategies::mean_reversion::PriceChannelStrategy::new(20))
            as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(price_channel) {
        eprintln!("Failed to register Price Channel Reversion strategy: {}", e);
    }

    // Register Keltner Channel Reversion strategy
    let keltner_reversion = Arc::new(
        alphafield_strategy::strategies::mean_reversion::KeltnerReversionStrategy::new(20, 10, 2.0),
    ) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(keltner_reversion) {
        eprintln!(
            "Failed to register Keltner Channel Reversion strategy: {}",
            e
        );
    }

    // Register Statistical Arbitrage strategy
    let stat_arb =
        Arc::new(alphafield_strategy::strategies::mean_reversion::StatArbStrategy::new(30))
            as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(stat_arb) {
        eprintln!("Failed to register Statistical Arbitrage strategy: {}", e);
    }

    // ------------------------------------------------------------------------
    // Momentum Strategies (Phase 12.4)
    // ------------------------------------------------------------------------

    // Register RSI Momentum strategy
    let rsi_momentum = Arc::new(
        alphafield_strategy::strategies::momentum::RsiMomentumStrategy::new(
            14, 50.0, 60.0, 5.0, 3.0,
        ),
    ) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(rsi_momentum) {
        eprintln!("Failed to register RSI Momentum strategy: {}", e);
    }

    // Register MACD Momentum strategy
    let macd_momentum =
        Arc::new(alphafield_strategy::strategies::momentum::MACDStrategy::new(50, 12, 26, 9))
            as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(macd_momentum) {
        eprintln!("Failed to register MACD Momentum strategy: {}", e);
    }

    // Register ROC (Rate of Change) strategy
    let roc = Arc::new(alphafield_strategy::strategies::momentum::RocStrategy::new(
        10, 2.0, -1.0, 5.0, 3.0,
    )) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(roc) {
        eprintln!("Failed to register ROC strategy: {}", e);
    }

    // Register ADX Trend strategy
    let adx_trend = Arc::new(
        alphafield_strategy::strategies::momentum::AdxTrendStrategy::new(14, 25.0, 20.0, 5.0, 3.0),
    ) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(adx_trend) {
        eprintln!("Failed to register ADX Trend strategy: {}", e);
    }

    // Register Momentum Factor strategy
    let momentum_factor = Arc::new(
        alphafield_strategy::strategies::momentum::MomentumFactorStrategy::new(20, 14, 2, 5.0, 3.0),
    ) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(momentum_factor) {
        eprintln!("Failed to register Momentum Factor strategy: {}", e);
    }

    // Register Volume Momentum strategy
    let volume_momentum = Arc::new(
        alphafield_strategy::strategies::momentum::VolumeMomentumStrategy::new(
            20, 20, 1.5, 5.0, 3.0,
        ),
    ) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(volume_momentum) {
        eprintln!("Failed to register Volume Momentum strategy: {}", e);
    }

    // Register Multi-Timeframe Momentum strategy
    let multi_tf_momentum = Arc::new(
        alphafield_strategy::strategies::momentum::MultiTfMomentumStrategy::new(20, 50, 5.0, 3.0),
    ) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(multi_tf_momentum) {
        eprintln!(
            "Failed to register Multi-Timeframe Momentum strategy: {}",
            e
        );
    }

    // ------------------------------------------------------------------------
    // Volatility-Based Strategies (Phase 12.5)
    // ------------------------------------------------------------------------

    // Register ATR Breakout strategy
    let atr_breakout = Arc::new(
        alphafield_strategy::strategies::volatility::ATRBreakoutStrategy::new(14, 1.5, 20),
    ) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(atr_breakout) {
        eprintln!("Failed to register ATR Breakout strategy: {}", e);
    }

    // Register Volatility Squeeze strategy
    let vol_squeeze = Arc::new(
        alphafield_strategy::strategies::volatility::VolSqueezeStrategy::new(20, 2.0, 20, 1.5, 0.1),
    ) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(vol_squeeze) {
        eprintln!("Failed to register Volatility Squeeze strategy: {}", e);
    }

    // Register Volatility Regime strategy
    let vol_regime =
        Arc::new(alphafield_strategy::strategies::volatility::VolRegimeStrategy::new(14, 100))
            as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(vol_regime) {
        eprintln!("Failed to register Volatility Regime strategy: {}", e);
    }

    // Register ATR Trailing Stop strategy
    let atr_trailing = Arc::new(
        alphafield_strategy::strategies::volatility::ATRTrailingStrategy::new(14, 2.0, 10, 30),
    ) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(atr_trailing) {
        eprintln!("Failed to register ATR Trailing Stop strategy: {}", e);
    }

    // Register Volatility-Adjusted Position Sizing strategy
    let vol_sizing = Arc::new(
        alphafield_strategy::strategies::volatility::VolSizingStrategy::new(14, 10.0, 100),
    ) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(vol_sizing) {
        eprintln!(
            "Failed to register Volatility-Adjusted Position Sizing strategy: {}",
            e
        );
    }

    // Register GARCH-Based strategy
    let garch = Arc::new(alphafield_strategy::strategies::volatility::GARCHStrategy::new(0.94, 20))
        as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(garch) {
        eprintln!("Failed to register GARCH-Based strategy: {}", e);
    }

    // Register VIX-Style strategy
    let vix_style =
        Arc::new(alphafield_strategy::strategies::volatility::VIXStyleStrategy::new(14, 100))
            as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(vix_style) {
        eprintln!("Failed to register VIX-Style strategy: {}", e);
    }

    // ------------------------------------------------------------------------
    // Sentiment-Based Strategies (Phase 12.6)
    // ------------------------------------------------------------------------

    // Register Sentiment Momentum strategy
    let sentiment_momentum =
        Arc::new(alphafield_strategy::strategies::sentiment::SentimentMomentumStrategy::new())
            as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(sentiment_momentum) {
        eprintln!("Failed to register Sentiment Momentum strategy: {}", e);
    }

    // Register Divergence strategy
    let divergence = Arc::new(alphafield_strategy::strategies::sentiment::DivergenceStrategy::new())
        as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(divergence) {
        eprintln!("Failed to register Divergence strategy: {}", e);
    }

    // Register Regime-Based Sentiment strategy
    let regime_sentiment =
        Arc::new(alphafield_strategy::strategies::sentiment::RegimeSentimentStrategy::new())
            as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(regime_sentiment) {
        eprintln!("Failed to register Regime-Based Sentiment strategy: {}", e);
    }

    // ------------------------------------------------------------------------
    // Multi-Indicator Strategies (Phase 12.8)
    // ------------------------------------------------------------------------

    // Register MACD + RSI Combo strategy
    let macd_rsi_combo = Arc::new(alphafield_strategy::strategies::MACDRSIComboStrategy::new(
        12, 26, 9, 14,
    )) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(macd_rsi_combo) {
        eprintln!("Failed to register MACD + RSI Combo strategy: {}", e);
    }

    // Register Trend + Mean Reversion Hybrid strategy
    let trend_mean_rev = Arc::new(alphafield_strategy::strategies::TrendMeanRevStrategy::new(
        20, 50, 14,
    )) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(trend_mean_rev) {
        eprintln!(
            "Failed to register Trend + Mean Reversion Hybrid strategy: {}",
            e
        );
    }

    // Register Confidence-Weighted strategy
    let confidence_weighted = Arc::new(
        alphafield_strategy::strategies::ConfidenceWeightedStrategy::new(20, 50, 12, 26, 9, 14),
    ) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(confidence_weighted) {
        eprintln!("Failed to register Confidence-Weighted strategy: {}", e);
    }

    // Register Adaptive Combination strategy
    let adaptive_combo = Arc::new(alphafield_strategy::strategies::AdaptiveComboStrategy::new(
        20, 50, 12, 26, 9, 14,
    )) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(adaptive_combo) {
        eprintln!("Failed to register Adaptive Combination strategy: {}", e);
    }

    // Register Ensemble Weighted strategy
    let ensemble_weighted =
        Arc::new(alphafield_strategy::strategies::EnsembleWeightedStrategy::new(10, 5))
            as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(ensemble_weighted) {
        eprintln!("Failed to register Ensemble Weighted strategy: {}", e);
    }

    // Register Regime-Switching strategy
    let regime_switching =
        Arc::new(alphafield_strategy::strategies::RegimeSwitchingStrategy::new(20, 50, 14, 14))
            as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(regime_switching) {
        eprintln!("Failed to register Regime-Switching strategy: {}", e);
    }

    // Register ML-Enhanced Multi-Indicator strategy
    let ml_enhanced = Arc::new(alphafield_strategy::strategies::MLEnhancedStrategy::new(
        20, 50, 12, 26, 9, 14, 14,
    )) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(ml_enhanced) {
        eprintln!(
            "Failed to register ML-Enhanced Multi-Indicator strategy: {}",
            e
        );
    }

    registry
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse strategy category from query string
fn parse_strategy_category(s: &str) -> Result<StrategyCategory, AppError> {
    match s.to_lowercase().replace('_', "").as_str() {
        "trendfollowing" | "trend" | "trend-following" => Ok(StrategyCategory::TrendFollowing),
        "meanreversion" | "mean" | "mean-reversion" => Ok(StrategyCategory::MeanReversion),
        "momentum" => Ok(StrategyCategory::Momentum),
        "volatilitybased" | "volatility" | "volatility-based" => {
            Ok(StrategyCategory::VolatilityBased)
        }
        "sentimentbased" | "sentiment" | "sentiment-based" => Ok(StrategyCategory::SentimentBased),
        "multiindicator" | "multi" | "multi-indicator" => Ok(StrategyCategory::MultiIndicator),
        _ => Err(AppError::invalid_filter("category", s)),
    }
}

/// Parse market regime from query string
fn parse_market_regime(s: &str) -> Result<MarketRegime, AppError> {
    match s.to_lowercase().replace('-', "").as_str() {
        "bull" | "bullish" => Ok(MarketRegime::Bull),
        "bear" | "bearish" => Ok(MarketRegime::Bear),
        "sideways" | "ranging" | "range" => Ok(MarketRegime::Sideways),
        "highvolatility" | "highvol" | "high" => Ok(MarketRegime::HighVolatility),
        "lowvolatility" | "lowvol" | "low" => Ok(MarketRegime::LowVolatility),
        "trending" => Ok(MarketRegime::Trending),
        _ => Err(AppError::invalid_filter("regime", s)),
    }
}

/// Convert strategy metadata to summary format
fn metadata_to_summary(metadata: &StrategyMetadata) -> StrategySummary {
    StrategySummary {
        name: metadata.name.clone(),
        category: format!("{:?}", metadata.category),
        sub_type: metadata.sub_type.clone(),
        description: metadata.description.clone(),
        expected_regimes: metadata
            .expected_regimes
            .iter()
            .map(|r| format!("{:?}", r))
            .collect(),
        risk_profile: RiskProfileSummary {
            max_drawdown_expected: metadata.risk_profile.max_drawdown_expected,
            volatility_level: format!("{:?}", metadata.risk_profile.volatility_level),
            correlation_sensitivity: format!("{:?}", metadata.risk_profile.correlation_sensitivity),
            leverage_requirement: metadata.risk_profile.leverage_requirement,
        },
    }
}

// ============================================================================
// API Handlers
// ============================================================================

/// List all registered strategies with optional filtering
///
/// # Query Parameters
/// - `category` (optional): Filter by strategy category (e.g., "TrendFollowing", "MeanReversion", "Momentum")
/// - `regime` (optional): Filter by suitable market regime (e.g., "Bull", "Bear", "Sideways")
///
/// # Returns
/// JSON array of strategy summaries
///
/// # Examples
/// ```bash
/// # Get all strategies
/// curl http://localhost:8080/api/strategies
///
/// # Get only trend-following strategies
/// curl http://localhost:8080/api/strategies?category=TrendFollowing
///
/// # Get strategies suitable for bull market
/// curl http://localhost:8080/api/strategies?regime=Bull
/// ```
pub async fn list_strategies(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StrategyQuery>,
) -> Result<Json<Vec<StrategySummary>>, AppError> {
    let registry = state.registry.clone();
    let strategy_names = if let Some(category_str) = query.category {
        let category = parse_strategy_category(&category_str)?;
        registry.list_by_category(category)
    } else if let Some(regime_str) = query.regime {
        let regime = parse_market_regime(&regime_str)?;
        registry.get_for_regime(regime)
    } else {
        registry.list_all()
    };

    let mut summaries = Vec::new();
    for name in strategy_names {
        if let Some(metadata) = registry.get_metadata(&name) {
            summaries.push(metadata_to_summary(&metadata));
        }
    }

    Ok(Json(summaries))
}

/// Get detailed information about a specific strategy
///
/// # Path Parameters
/// - `name`: Strategy name (e.g., "GoldenCross", "BollingerBands")
///
/// # Returns
/// Complete strategy metadata if found, or 404 if not found
///
/// # Examples
/// ```bash
/// curl http://localhost:8080/api/strategies/GoldenCross
/// curl http://localhost:8080/api/strategies/BollingerBands
/// ```
pub async fn get_strategy_details(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<StrategyDetails>, AppError> {
    let registry = state.registry.clone();
    let metadata = registry
        .get_metadata(&name)
        .ok_or_else(|| AppError::not_found(&name))?;

    Ok(Json(StrategyDetails {
        metadata,
        registered: true,
    }))
}

/// List all available strategy categories
///
/// # Returns
/// JSON array of available strategy categories
///
/// # Examples
/// ```bash
/// curl http://localhost:8080/api/strategies/categories
/// ```
pub async fn list_categories() -> Json<Vec<String>> {
    Json(vec![
        "TrendFollowing".to_string(),
        "MeanReversion".to_string(),
        "Momentum".to_string(),
        "VolatilityBased".to_string(),
        "SentimentBased".to_string(),
        "MultiIndicator".to_string(),
    ])
}

/// List all available market regimes
///
/// # Returns
/// JSON array of available market regimes
///
/// # Examples
/// ```bash
/// curl http://localhost:8080/api/strategies/regimes
/// ```
pub async fn list_regimes() -> Json<Vec<String>> {
    Json(vec![
        "Bull".to_string(),
        "Bear".to_string(),
        "Sideways".to_string(),
        "HighVolatility".to_string(),
        "LowVolatility".to_string(),
        "Trending".to_string(),
        "Ranging".to_string(),
    ])
}

/// Get strategy count summary
///
/// # Returns
/// Total count and breakdown by category
///
/// # Examples
/// ```bash
/// curl http://localhost:8080/api/strategies/summary
/// ```
pub async fn get_strategy_summary(
    State(state): State<Arc<AppState>>,
) -> Json<StrategyCountSummary> {
    let registry = state.registry.clone();
    let total = registry.count();
    let mut by_category = std::collections::HashMap::new();

    // Count strategies in each category
    for name in registry.list_all() {
        if let Some(metadata) = registry.get_metadata(&name) {
            let category = format!("{:?}", metadata.category);
            *by_category.entry(category).or_insert(0) += 1;
        }
    }

    Json(StrategyCountSummary { total, by_category })
}

// ============================================================================
// Router Builder
// ============================================================================

/// Create router for strategy API endpoints
pub fn create_strategy_router() -> Router<Arc<AppState>> {
    Router::new()
        // List all strategies (with optional filters)
        .route("/api/strategies", get(list_strategies))
        // Get specific strategy details
        .route("/api/strategies/:name", get(get_strategy_details))
        // List available categories
        .route("/api/strategies/categories", get(list_categories))
        // List available regimes
        .route("/api/strategies/regimes", get(list_regimes))
        // Get strategy summary/counts
        .route("/api/strategies/summary", get(get_strategy_summary))
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_strategy_category() {
        // Test valid categories
        assert!(matches!(
            parse_strategy_category("TrendFollowing").unwrap(),
            StrategyCategory::TrendFollowing
        ));
        assert!(matches!(
            parse_strategy_category("trend").unwrap(),
            StrategyCategory::TrendFollowing
        ));
        assert!(matches!(
            parse_strategy_category("meanreversion").unwrap(),
            StrategyCategory::MeanReversion
        ));
        assert!(matches!(
            parse_strategy_category("momentum").unwrap(),
            StrategyCategory::Momentum
        ));
        assert!(matches!(
            parse_strategy_category("volatility").unwrap(),
            StrategyCategory::VolatilityBased
        ));
        assert!(matches!(
            parse_strategy_category("sentiment").unwrap(),
            StrategyCategory::SentimentBased
        ));
        assert!(matches!(
            parse_strategy_category("multi").unwrap(),
            StrategyCategory::MultiIndicator
        ));
        // Baseline category removed - no longer available

        // Test case insensitivity / separator handling
        assert!(matches!(
            parse_strategy_category("TREND").unwrap(),
            StrategyCategory::TrendFollowing
        ));
        assert!(matches!(
            parse_strategy_category("Trend_Following").unwrap(),
            StrategyCategory::TrendFollowing
        ));
        assert!(matches!(
            parse_strategy_category("trend-following").unwrap(),
            StrategyCategory::TrendFollowing
        ));
        assert!(matches!(
            parse_strategy_category("mean_reversion").unwrap(),
            StrategyCategory::MeanReversion
        ));

        // Test invalid category includes proper error code/details
        let err = parse_strategy_category("invalid").unwrap_err();
        assert_eq!(err.code, "INVALID_FILTER");
        assert_eq!(err.error, "Invalid category value: invalid");
        assert!(err
            .details
            .unwrap_or_default()
            .contains("Valid values for category"));
    }

    #[test]
    fn test_parse_market_regime() {
        // Test valid regimes
        assert!(matches!(
            parse_market_regime("bull").unwrap(),
            MarketRegime::Bull
        ));
        assert!(matches!(
            parse_market_regime("bullish").unwrap(),
            MarketRegime::Bull
        ));
        assert!(matches!(
            parse_market_regime("bear").unwrap(),
            MarketRegime::Bear
        ));
        assert!(matches!(
            parse_market_regime("bearish").unwrap(),
            MarketRegime::Bear
        ));
        assert!(matches!(
            parse_market_regime("sideways").unwrap(),
            MarketRegime::Sideways
        ));
        assert!(matches!(
            parse_market_regime("ranging").unwrap(),
            MarketRegime::Sideways
        ));
        assert!(matches!(
            parse_market_regime("range").unwrap(),
            MarketRegime::Sideways
        ));
        assert!(matches!(
            parse_market_regime("high").unwrap(),
            MarketRegime::HighVolatility
        ));
        assert!(matches!(
            parse_market_regime("high-volatility").unwrap(),
            MarketRegime::HighVolatility
        ));
        assert!(matches!(
            parse_market_regime("low").unwrap(),
            MarketRegime::LowVolatility
        ));
        assert!(matches!(
            parse_market_regime("low-volatility").unwrap(),
            MarketRegime::LowVolatility
        ));
        assert!(matches!(
            parse_market_regime("trending").unwrap(),
            MarketRegime::Trending
        ));

        // Test case insensitivity
        assert!(matches!(
            parse_market_regime("BULL").unwrap(),
            MarketRegime::Bull
        ));
        assert!(matches!(
            parse_market_regime("BEAR").unwrap(),
            MarketRegime::Bear
        ));

        // Test invalid regime includes proper error code/details
        let err = parse_market_regime("invalid").unwrap_err();
        assert_eq!(err.code, "INVALID_FILTER");
        assert_eq!(err.error, "Invalid regime value: invalid");
        assert!(err
            .details
            .unwrap_or_default()
            .contains("Valid values for regime"));
    }
}
