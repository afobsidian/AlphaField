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
    HoldBaseline, MarketAverageBaseline, MarketRegime, StrategyCategory, StrategyMetadata,
    StrategyRegistry, StrategyWithMetadata,
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
                "TrendFollowing, MeanReversion, Momentum, VolatilityBased, SentimentBased, MultiIndicator, Baseline"
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

    // Register HODL baseline strategy
    let hodl = Arc::new(HoldBaseline::new()) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(hodl) {
        eprintln!("Failed to register HODL baseline: {}", e);
    }

    // Register Market Average baseline strategy
    let symbols = vec!["BTC".to_string(), "ETH".to_string(), "SOL".to_string()];
    let market_avg =
        Arc::new(MarketAverageBaseline::equal_weighted(symbols)) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(market_avg) {
        eprintln!("Failed to register Market Average baseline: {}", e);
    }

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

    // Register Momentum strategy
    let momentum = Arc::new(alphafield_strategy::strategies::MACDStrategy::new(
        50, 12, 26, 9,
    )) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(momentum) {
        eprintln!("Failed to register Momentum strategy: {}", e);
    }

    // Register RSI Mean Reversion strategy
    let rsi = Arc::new(alphafield_strategy::strategies::RsiStrategy::new(
        14, 30.0, 70.0,
    )) as Arc<dyn StrategyWithMetadata>;
    if let Err(e) = registry.register(rsi) {
        eprintln!("Failed to register RSI Mean Reversion strategy: {}", e);
    }

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
        "baseline" => Ok(StrategyCategory::Baseline),
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
/// - `category` (optional): Filter by strategy category (e.g., "TrendFollowing", "MeanReversion", "Baseline")
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
/// - `name`: Strategy name (e.g., "HODL_Baseline", "GoldenCross")
///
/// # Returns
/// Complete strategy metadata if found, or 404 if not found
///
/// # Examples
/// ```bash
/// curl http://localhost:8080/api/strategies/HODL_Baseline
/// curl http://localhost:8080/api/strategies/GoldenCross
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
        "Baseline".to_string(),
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
        assert!(matches!(
            parse_strategy_category("baseline").unwrap(),
            StrategyCategory::Baseline
        ));

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
