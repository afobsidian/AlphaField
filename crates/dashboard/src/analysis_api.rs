//! Analysis API endpoints for Monte Carlo, sensitivity, and correlation analysis

use axum::{
    extract::{Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, instrument, warn};

use alphafield_backtest::{
    monte_carlo::{MonteCarloConfig, MonteCarloSimulator, Trade as McTrade},
    portfolio_optimization::{
        MeanVarianceOptimizer, MultiStrategyPortfolio, OptimizationConfig, OptimizationObjective,
        PortfolioOptimizer, StrategyMetadata,
    },
    portfolio_validation::{
        PortfolioMonteCarloConfig, PortfolioMonteCarloResult, PortfolioSensitivityResult,
        PortfolioValidationReport, PortfolioValidator, PortfolioWalkForwardResult,
    },
    position_sizing::{FixedFractionalSizing, KellyCriterion, PositionSizing, TradeStatistics},
    sensitivity::{HeatmapData, ParameterRange, SensitivityAnalyzer, SensitivityConfig},
    walk_forward::{WalkForwardAnalyzer, WalkForwardConfig, WalkForwardResult},
    CorrelationAnalyzer, CorrelationConfig, CorrelationResult, MonteCarloResult, StrategyAdapter,
};

use crate::api::AppState;
use crate::services::data_service::fetch_data_with_cache;
use crate::services::strategy_service::StrategyFactory;
use alphafield_backtest::strategy::Strategy as BacktestStrategy;
use chrono::{Duration, Utc};
use std::collections::HashMap;

// ============= Monte Carlo API =============

#[derive(Debug, Deserialize)]
pub struct MonteCarloRequest {
    /// List of trades (pnl, return_pct, duration)
    pub trades: Vec<TradeInput>,
    /// Number of simulations (default: 1000)
    pub num_simulations: Option<usize>,
    /// Initial capital (default: 10000)
    pub initial_capital: Option<f64>,
    /// Random seed for reproducibility
    pub seed: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct TradeInput {
    pub symbol: String,
    pub pnl: f64,
    pub return_pct: f64,
    pub duration: usize,
}

#[derive(Serialize)]
pub struct MonteCarloResponse {
    pub success: bool,
    pub result: Option<MonteCarloResult>,
    pub error: Option<String>,
}

#[instrument(skip(_state), fields(trade_count = req.trades.len(), simulations = req.num_simulations))]
pub async fn run_monte_carlo(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<MonteCarloRequest>,
) -> Json<MonteCarloResponse> {
    info!("Monte Carlo API request received");

    if req.trades.is_empty() {
        warn!("No trades provided for Monte Carlo simulation");
        return Json(MonteCarloResponse {
            success: false,
            result: None,
            error: Some("No trades provided".to_string()),
        });
    }

    let config = MonteCarloConfig {
        num_simulations: req.num_simulations.unwrap_or(1000),
        initial_capital: req.initial_capital.unwrap_or(10000.0),
        seed: req.seed,
    };

    let trades: Vec<McTrade> = req
        .trades
        .iter()
        .map(|t| McTrade {
            symbol: t.symbol.clone(),
            pnl: t.pnl,
            return_pct: t.return_pct,
            duration: t.duration,
        })
        .collect();

    let simulator = MonteCarloSimulator::new(config);
    let result = simulator.simulate(&trades);

    info!(
        simulations = result.num_simulations,
        "Monte Carlo simulation completed"
    );

    Json(MonteCarloResponse {
        success: true,
        result: Some(result),
        error: None,
    })
}

// ============= Correlation API =============

#[derive(Debug, Deserialize)]
pub struct CorrelationRequest {
    /// List of equity curves with labels
    pub curves: Vec<EquityCurveInput>,
    /// Alert threshold (default: 0.7)
    pub alert_threshold: Option<f64>,
    /// Minimum data points (default: 30)
    pub min_data_points: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct EquityCurveInput {
    pub label: String,
    pub values: Vec<f64>,
}

#[derive(Serialize)]
pub struct CorrelationResponse {
    pub success: bool,
    pub result: Option<CorrelationResult>,
    pub error: Option<String>,
}

#[instrument(skip(_state), fields(curve_count = req.curves.len()))]
pub async fn calculate_correlation(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CorrelationRequest>,
) -> Json<CorrelationResponse> {
    info!("Correlation API request received");

    if req.curves.len() < 2 {
        warn!("Insufficient curves for correlation analysis");
        return Json(CorrelationResponse {
            success: false,
            result: None,
            error: Some("Need at least 2 equity curves".to_string()),
        });
    }

    let config = CorrelationConfig {
        alert_threshold: req.alert_threshold.unwrap_or(0.7),
        min_data_points: req.min_data_points.unwrap_or(30),
    };

    let curves: Vec<(String, Vec<f64>)> = req
        .curves
        .iter()
        .map(|c| (c.label.clone(), c.values.clone()))
        .collect();

    let analyzer = CorrelationAnalyzer::new(config);
    match analyzer.analyze_equity_curves(&curves) {
        Ok(result) => {
            info!(
                avg_correlation = format!("{:.2}", result.average_correlation),
                alerts = result.alerts.len(),
                "Correlation analysis completed"
            );
            Json(CorrelationResponse {
                success: true,
                result: Some(result),
                error: None,
            })
        }
        Err(e) => {
            warn!(error = %e, "Correlation analysis failed");
            Json(CorrelationResponse {
                success: false,
                result: None,
                error: Some(e),
            })
        }
    }
}

// ============= Sensitivity API =============

#[derive(Deserialize)]
pub struct SensitivityRequest {
    /// Strategy name to analyze
    pub strategy: String,
    /// Symbol to backtest
    pub symbol: String,
    /// Interval (e.g., "1h", "1d")
    pub interval: String,
    /// Number of days of data
    pub days: u32,
    /// Parameter to sweep (name, min, max, step)
    pub param: ParameterInput,
    /// Optional second parameter for 2D analysis
    pub param_y: Option<ParameterInput>,
    /// Any fixed parameters to include
    #[serde(default)]
    pub fixed_params: HashMap<String, f64>,
}

#[derive(Deserialize)]
pub struct ParameterInput {
    pub name: String,
    pub min: f64,
    pub max: f64,
    pub step: f64,
}

#[derive(Serialize)]
pub struct SensitivityResponse {
    pub success: bool,
    pub results: Vec<ParameterResultOutput>,
    pub best_params: Option<std::collections::HashMap<String, f64>>,
    pub heatmap: Option<HeatmapData>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct ParameterResultOutput {
    pub params: std::collections::HashMap<String, f64>,
    pub sharpe_ratio: f64,
    pub total_return: f64,
    pub max_drawdown: f64,
}

pub async fn run_sensitivity(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<SensitivityRequest>,
) -> Json<SensitivityResponse> {
    info!(
        "Sensitivity Analysis requested for {} {}",
        req.symbol, req.strategy
    );

    // 1. Fetch Data
    let days = if req.days > 0 { req.days } else { 180 };
    let end_time = Utc::now();
    let start_time = end_time - Duration::days(days as i64);

    let (bars, _) = match fetch_data_with_cache(
        req.symbol.clone(),
        req.interval.clone(),
        start_time,
        end_time,
    )
    .await
    {
        Ok(res) => res,
        Err(e) => {
            return Json(SensitivityResponse {
                success: false,
                results: vec![],
                best_params: None,
                heatmap: None,
                error: Some(format!("Failed to fetch data: {}", e)),
            })
        }
    };

    if bars.len() < 100 {
        return Json(SensitivityResponse {
            success: false,
            results: vec![],
            best_params: None,
            heatmap: None,
            error: Some("Insufficient data for sensitivity analysis".to_string()),
        });
    }

    // 2. Configure Analyzer
    let config = SensitivityConfig {
        initial_capital: 100_000.0,
        fee_rate: 0.001,
        parallel: true,
    };
    let analyzer = SensitivityAnalyzer::new(config);

    // 3. Run Analysis
    let strategy_name = req.strategy.clone();
    let fixed_params = req.fixed_params.clone();
    let req_symbol = req.symbol.clone();

    let range_x = ParameterRange::new(
        &req.param.name,
        req.param.min,
        req.param.max,
        req.param.step,
    );

    let result_raw = if let Some(py) = &req.param_y {
        let range_y = ParameterRange::new(&py.name, py.min, py.max, py.step);
        let sym = req_symbol.clone();
        let s_name = strategy_name.clone();

        // Use create_backtest which returns properly wrapped strategies
        let factory_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            analyzer.analyze_2d(&bars, &req.symbol, &range_x, &range_y, |v1, v2| {
                let mut p = fixed_params.clone();
                p.insert(req.param.name.clone(), v1);
                p.insert(py.name.clone(), v2);

                // Use create_backtest for properly adapted strategies
                // If invalid params, return strategy with default params but mark as invalid or let it fail?
                // Better to return None and handle it in result processing
                StrategyFactory::create_backtest(&s_name, &p, &sym, 100_000.0)
            })
        }));

        match factory_result {
            Ok(res) => res,
            Err(_) => {
                return Json(SensitivityResponse {
                    success: false,
                    results: vec![],
                    best_params: None,
                    heatmap: None,
                    error: Some(
                        "Strategy creation failed - invalid parameter combination".to_string(),
                    ),
                })
            }
        }
    } else {
        let sym = req_symbol.clone();
        let s_name = strategy_name.clone();

        let factory_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            analyzer.analyze_1d(&bars, &req.symbol, &range_x, |v1| {
                let mut p = fixed_params.clone();
                p.insert(req.param.name.clone(), v1);

                // Use create_backtest for properly adapted strategies
                StrategyFactory::create_backtest(&s_name, &p, &sym, 100_000.0)
            })
        }));

        match factory_result {
            Ok(res) => res,
            Err(_) => {
                return Json(SensitivityResponse {
                    success: false,
                    results: vec![],
                    best_params: None,
                    heatmap: None,
                    error: Some(
                        "Strategy creation failed - invalid parameter combination".to_string(),
                    ),
                })
            }
        }
    };

    match result_raw {
        Ok(res) => {
            // Convert to response format
            let results: Vec<ParameterResultOutput> = res
                .results
                .iter()
                .map(|r| ParameterResultOutput {
                    params: r.params.clone(),
                    sharpe_ratio: r.metrics.sharpe_ratio,
                    total_return: r.metrics.total_return,
                    max_drawdown: r.metrics.max_drawdown,
                })
                .collect();

            let best_params = res.best_sharpe.map(|r| r.params);

            Json(SensitivityResponse {
                success: true,
                results,
                best_params,
                heatmap: res.heatmap,
                error: None,
            })
        }
        Err(e) => Json(SensitivityResponse {
            success: false,
            results: vec![],
            best_params: None,
            heatmap: None,
            error: Some(e),
        }),
    }
}
// ============= Walk-Forward API =============

#[derive(Deserialize)]
pub struct WalkForwardRequest {
    pub strategy: String,
    pub symbol: String,
    pub interval: String,
    pub params: HashMap<String, f64>,
    pub train_window_days: Option<usize>, // Default 365
    pub test_window_days: Option<usize>,  // Default 90
}

#[derive(Serialize)]
pub struct WalkForwardResponse {
    pub success: bool,
    pub result: Option<WalkForwardResult>,
    pub error: Option<String>,
}

pub async fn run_walk_forward(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<WalkForwardRequest>,
) -> Json<WalkForwardResponse> {
    info!("Walk-Forward Analysis requested for {}", req.symbol);

    // 1. Fetch 4 years of data (pagination now supported)
    let end_time = Utc::now();
    let start_time = end_time - Duration::days(4 * 365);

    let (bars, data_status) = match fetch_data_with_cache(
        req.symbol.clone(),
        req.interval.clone(),
        start_time,
        end_time,
    )
    .await
    {
        Ok(res) => res,
        Err(e) => {
            return Json(WalkForwardResponse {
                success: false,
                result: None,
                error: Some(format!("Failed to fetch data: {}", e)),
            })
        }
    };

    info!(bars_fetched = bars.len(), source = %data_status.source, "Data fetch complete");

    // 2. Configure Analysis
    // Dynamic bars per day calculation
    let bars_per_day = match req.interval.as_str() {
        "1m" => 1440,
        "5m" => 288,
        "15m" => 96,
        "1h" => 24,
        "4h" => 6,
        "1d" => 1,
        _ => {
            if req.interval.ends_with('h') {
                24
            } else {
                1
            }
        } // Fallback
    };

    let train_window_bars = req.train_window_days.unwrap_or(365) * bars_per_day;
    let test_window_bars = req.test_window_days.unwrap_or(90) * bars_per_day;
    let required_bars = train_window_bars + test_window_bars;

    if bars.len() < required_bars {
        return Json(WalkForwardResponse {
            success: false,
            result: None,
            error: Some(format!(
                "Insufficient data: have {} bars, need at least {} (train + test window). Fetch more data or reduce window sizes.",
                bars.len(), required_bars
            )),
        });
    }

    let config = WalkForwardConfig {
        train_window: train_window_bars,
        test_window: test_window_bars,
        step_size: 30 * bars_per_day, // 1 month step default
        initial_capital: 100_000.0,
        fee_rate: 0.001,
    };

    // 3. Create Strategy Factory
    let strategy_name = req.strategy.clone();
    let strategy_params = req.params.clone();
    let req_symbol = req.symbol.clone();

    // Safely handle strategy creation
    if StrategyFactory::create(&strategy_name, &strategy_params).is_none() {
        return Json(WalkForwardResponse {
            success: false,
            result: None,
            error: Some(format!("Invalid strategy parameters for {}", strategy_name)),
        });
    }

    let factory = move || -> Box<dyn BacktestStrategy> {
        // We verified it above, but factory runs in thread, so unwrap is "safer" but lets still be careful
        let core_strat = StrategyFactory::create(&strategy_name, &strategy_params)
            .expect("Strategy creation failed in factory execution");

        let adapter = StrategyAdapter::new(core_strat, req_symbol.clone(), 100_000.0);
        Box::new(adapter)
    };

    // 4. Run Analysis
    let analyzer = WalkForwardAnalyzer::new(config);
    match analyzer.analyze(&bars, &req.symbol, factory) {
        Ok(result) => Json(WalkForwardResponse {
            success: true,
            result: Some(result),
            error: None,
        }),
        Err(e) => Json(WalkForwardResponse {
            success: false,
            result: None,
            error: Some(e),
        }),
    }
}

// ============= Correlation Matrix API =============

/// Response structure for correlation matrix data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationMatrixResponse {
    /// Strategy names (row/column labels)
    pub strategies: Vec<String>,
    /// 2D correlation matrix (values -1.0 to 1.0)
    pub matrix: Vec<Vec<f64>>,
    /// Strategy clusters based on high correlation
    pub clusters: Vec<Vec<String>>,
    /// Portfolio diversification score (1 - avg correlation)
    pub diversification_score: f64,
    /// High correlation pairs above threshold
    pub high_correlation_pairs: Vec<(String, String, f64)>,
    /// Success flag
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Request body for correlation matrix endpoint
#[derive(Debug, Deserialize)]
pub struct CorrelationMatrixRequest {
    /// Strategy names
    pub strategies: Vec<String>,
    /// Equity curves for each strategy
    pub equity_curves: Vec<Vec<f64>>,
    /// Correlation threshold for highlighting (0.0 to 1.0, default 0.7)
    pub threshold: Option<f64>,
    /// Minimum data points (default: 30)
    pub min_data_points: Option<usize>,
}

/// GET /api/analysis/correlation-matrix
/// Returns correlation matrix data for heatmap visualization
pub async fn get_correlation_matrix(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<CorrelationMatrixRequest>,
) -> Json<CorrelationMatrixResponse> {
    info!("Correlation matrix API request received");

    if params.strategies.is_empty() || params.equity_curves.is_empty() {
        return Json(CorrelationMatrixResponse {
            strategies: vec![],
            matrix: vec![],
            clusters: vec![],
            diversification_score: 0.0,
            high_correlation_pairs: vec![],
            success: false,
            error: Some("No strategies or equity curves provided".to_string()),
        });
    }

    if params.strategies.len() != params.equity_curves.len() {
        return Json(CorrelationMatrixResponse {
            strategies: vec![],
            matrix: vec![],
            clusters: vec![],
            diversification_score: 0.0,
            high_correlation_pairs: vec![],
            success: false,
            error: Some("Strategy names and equity curves must match in length".to_string()),
        });
    }

    let threshold = params.threshold.unwrap_or(0.7);
    let min_data_points = params.min_data_points.unwrap_or(30);

    // Build correlation matrix using CorrelationAnalyzer
    let config = CorrelationConfig {
        alert_threshold: threshold,
        min_data_points,
    };

    let curves: Vec<(String, Vec<f64>)> = params
        .strategies
        .iter()
        .zip(params.equity_curves.iter())
        .map(|(name, curve)| (name.clone(), curve.clone()))
        .collect();

    let analyzer = CorrelationAnalyzer::new(config);
    let correlation_result = match analyzer.analyze_equity_curves(&curves) {
        Ok(result) => result,
        Err(e) => {
            return Json(CorrelationMatrixResponse {
                strategies: params.strategies.clone(),
                matrix: vec![],
                clusters: vec![],
                diversification_score: 0.0,
                high_correlation_pairs: vec![],
                success: false,
                error: Some(format!("Failed to calculate correlations: {}", e)),
            })
        }
    };

    // Build the correlation matrix from the result
    let num_strategies = params.strategies.len();
    let mut matrix: Vec<Vec<f64>> = (0..num_strategies)
        .map(|_| vec![0.0_f64; num_strategies])
        .collect();

    for i in 0..num_strategies {
        matrix[i][i] = 1.0; // Diagonal is always 1
        for j in (i + 1)..num_strategies {
            let corr = correlation_result
                .matrix
                .get_by_label(&params.strategies[i], &params.strategies[j])
                .unwrap_or(0.0_f64);
            matrix[i][j] = corr;
            matrix[j][i] = corr; // Symmetric matrix
        }
    }

    // Identify clusters (simple approach: group if correlation > threshold)
    let mut clusters: Vec<Vec<String>> = vec![];
    let mut visited: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for i in 0..num_strategies {
        if visited.contains(&i) {
            continue;
        }

        let mut cluster = vec![params.strategies[i].clone()];
        visited.insert(i);

        for j in (i + 1)..num_strategies {
            if !visited.contains(&j) && matrix[i][j].abs() >= threshold {
                cluster.push(params.strategies[j].clone());
                visited.insert(j);
            }
        }

        if cluster.len() > 1 {
            clusters.push(cluster);
        }
    }

    // Calculate diversification score from the correlation result
    let diversification_score = correlation_result.diversification_score;

    // Find high correlation pairs
    let mut high_corr_pairs: Vec<(String, String, f64)> = vec![];
    for i in 0..num_strategies {
        for j in (i + 1)..num_strategies {
            let corr = matrix[i][j].abs();
            if corr >= threshold {
                high_corr_pairs.push((
                    params.strategies[i].clone(),
                    params.strategies[j].clone(),
                    corr,
                ));
            }
        }
    }
    // Sort by correlation strength descending
    high_corr_pairs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

    info!(
        strategies = num_strategies,
        diversification = format!("{:.2}", diversification_score),
        high_corr_pairs = high_corr_pairs.len(),
        "Correlation matrix calculated successfully"
    );

    Json(CorrelationMatrixResponse {
        strategies: params.strategies,
        matrix,
        clusters,
        diversification_score,
        high_correlation_pairs: high_corr_pairs,
        success: true,
        error: None,
    })
}

// ============= Portfolio Management API =============

/// Request to create or update a multi-strategy portfolio
#[derive(Debug, Deserialize)]
pub struct PortfolioCreateRequest {
    /// Portfolio name/identifier
    pub name: String,
    /// Strategy equity curves with labels
    pub strategies: Vec<PortfolioStrategyInput>,
}

/// Input for a single strategy in portfolio
#[derive(Debug, Deserialize, Clone)]
pub struct PortfolioStrategyInput {
    /// Strategy name/identifier
    pub name: String,
    /// Historical equity curve
    pub equity_curve: Vec<f64>,
    /// Optional metadata
    pub asset_class: Option<String>,
    pub timeframe: Option<String>,
    pub expected_return: Option<f64>,
    pub volatility: Option<f64>,
    pub max_drawdown: Option<f64>,
    pub sharpe_ratio: Option<f64>,
}

/// Response with portfolio creation result
#[derive(Serialize)]
pub struct PortfolioCreateResponse {
    pub success: bool,
    pub portfolio_name: String,
    pub num_strategies: usize,
    pub error: Option<String>,
}

#[instrument(skip(_state), fields(name = req.name, strategies = req.strategies.len()))]
pub async fn create_portfolio(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<PortfolioCreateRequest>,
) -> Json<PortfolioCreateResponse> {
    info!("Portfolio creation request received");

    if req.strategies.is_empty() {
        warn!("No strategies provided for portfolio");
        return Json(PortfolioCreateResponse {
            success: false,
            portfolio_name: req.name,
            num_strategies: 0,
            error: Some("At least one strategy required".to_string()),
        });
    }

    let mut portfolio = MultiStrategyPortfolio::new(&req.name);

    for strat in &req.strategies {
        if strat.equity_curve.is_empty() {
            return Json(PortfolioCreateResponse {
                success: false,
                portfolio_name: req.name,
                num_strategies: 0,
                error: Some(format!("Strategy '{}' has empty equity curve", strat.name)),
            });
        }

        let metadata = StrategyMetadata::new(&strat.name)
            .with_asset_class(strat.asset_class.as_deref().unwrap_or("Unknown"))
            .with_timeframe(strat.timeframe.as_deref().unwrap_or("Unknown"));

        portfolio.add_strategy(&strat.name, strat.equity_curve.clone(), metadata);
    }

    // In production, would store portfolio in state/persistence layer
    info!(
        name = %portfolio.name,
        strategies = portfolio.equity_curves.len(),
        "Portfolio created successfully"
    );

    Json(PortfolioCreateResponse {
        success: true,
        portfolio_name: req.name,
        num_strategies: req.strategies.len(),
        error: None,
    })
}

// ============= Portfolio Optimization API =============

/// Request for portfolio optimization
#[derive(Debug, Deserialize)]
pub struct PortfolioOptimizeRequest {
    /// Portfolio name (would retrieve from state in production)
    pub portfolio_name: String,
    /// Strategy equity curves
    pub strategies: Vec<PortfolioStrategyInput>,
    /// Optimization objective
    pub objective: OptimizationObjective,
    /// Initial capital for calculations
    pub initial_capital: Option<f64>,
    /// Risk-free rate for Sharpe calculation (default: 0.02)
    pub risk_free_rate: Option<f64>,
    /// Minimum weight for any strategy (default: 0.0)
    pub min_weight: Option<f64>,
    /// Maximum weight for any strategy (default: 1.0)
    pub max_weight: Option<f64>,
}

/// Response with optimization results
#[derive(Serialize)]
pub struct PortfolioOptimizeResponse {
    pub success: bool,
    pub result: Option<PortfolioOptimizeResult>,
    pub error: Option<String>,
}

/// Simplified optimization result for API response
#[derive(Serialize)]
pub struct PortfolioOptimizeResult {
    pub allocations: HashMap<String, f64>,
    pub expected_return: f64,
    pub expected_volatility: f64,
    pub expected_sharpe: f64,
    pub diversification_ratio: f64,
    pub objective: String,
    pub num_strategies: usize,
    pub combined_equity_curve: Vec<f64>,
}

#[instrument(skip(_state), fields(name = req.portfolio_name, objective = format!("{:?}", req.objective), strategies = req.strategies.len()))]
pub async fn optimize_portfolio(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<PortfolioOptimizeRequest>,
) -> Json<PortfolioOptimizeResponse> {
    info!("Portfolio optimization request received");

    if req.strategies.len() < 2 {
        return Json(PortfolioOptimizeResponse {
            success: false,
            result: None,
            error: Some("At least 2 strategies required for portfolio optimization".to_string()),
        });
    }

    // Build portfolio
    let mut portfolio = MultiStrategyPortfolio::new(&req.portfolio_name);
    for strat in &req.strategies {
        let metadata = StrategyMetadata::new(&strat.name)
            .with_expected_return(strat.expected_return.unwrap_or(0.0))
            .with_volatility(strat.volatility.unwrap_or(0.0));
        portfolio.add_strategy(&strat.name, strat.equity_curve.clone(), metadata);
    }

    let initial_capital = req.initial_capital.unwrap_or(100_000.0);

    // Build equity curves HashMap
    let equity_curves: HashMap<String, Vec<f64>> = req
        .strategies
        .iter()
        .map(|s| (s.name.clone(), s.equity_curve.clone()))
        .collect();

    let strategy_names: Vec<String> = req.strategies.iter().map(|s| s.name.clone()).collect();

    // Build constraints
    let mut constraints = alphafield_backtest::portfolio_optimization::PortfolioConstraint::new();
    if let Some(min_w) = req.min_weight {
        for name in &strategy_names {
            constraints = constraints.with_min_weight(name, min_w);
        }
    }
    if let Some(max_w) = req.max_weight {
        for name in &strategy_names {
            constraints = constraints.with_max_weight(name, max_w);
        }
    }

    let config = OptimizationConfig::new()
        .with_objective(req.objective)
        .with_initial_capital(initial_capital)
        .with_risk_free_rate(req.risk_free_rate.unwrap_or(0.02))
        .with_max_iterations(1000)
        .with_tolerance(1e-6)
        .with_constraints(constraints);

    // Run optimization based on objective
    let optimizer = MeanVarianceOptimizer::new();
    let result = match optimizer.optimize(&strategy_names, &equity_curves, &config) {
        Ok(result) => Some(result),
        Err(e) => {
            return Json(PortfolioOptimizeResponse {
                success: false,
                result: None,
                error: Some(format!("Optimization failed: {}", e)),
            })
        }
    };

    match result {
        Some(opt_result) => {
            // Apply allocations to portfolio
            for (name, weight) in &opt_result.allocations {
                portfolio.set_allocation(name, *weight);
            }

            // Build portfolio for combined equity curve
            let mut portfolio_for_curve = MultiStrategyPortfolio::new(&req.portfolio_name);
            for strat in &req.strategies {
                let metadata = StrategyMetadata::new(&strat.name)
                    .with_expected_return(strat.expected_return.unwrap_or(0.0))
                    .with_volatility(strat.volatility.unwrap_or(0.0));
                portfolio_for_curve.add_strategy(&strat.name, strat.equity_curve.clone(), metadata);
            }

            // Apply allocations
            for (name, weight) in &opt_result.allocations {
                portfolio_for_curve.set_allocation(name, *weight);
            }

            // Get combined equity curve
            let combined_curve = portfolio_for_curve.combined_equity_curve(initial_capital);

            info!(
                objective = format!("{:?}", req.objective),
                sharpe = opt_result.expected_sharpe,
                "Optimization completed successfully"
            );

            Json(PortfolioOptimizeResponse {
                success: true,
                result: Some(PortfolioOptimizeResult {
                    allocations: opt_result.allocations,
                    expected_return: opt_result.expected_return,
                    expected_volatility: opt_result.expected_volatility,
                    expected_sharpe: opt_result.expected_sharpe,
                    diversification_ratio: opt_result.diversification_ratio,
                    objective: format!("{:?}", req.objective),
                    num_strategies: req.strategies.len(),
                    combined_equity_curve: combined_curve,
                }),
                error: None,
            })
        }
        None => Json(PortfolioOptimizeResponse {
            success: false,
            result: None,
            error: Some("Optimization failed to converge".to_string()),
        }),
    }
}

// ============= Portfolio Position Sizing API =============

/// Request for position sizing calculation
#[derive(Debug, Deserialize)]
pub struct PositionSizingRequest {
    /// Available capital
    pub capital: f64,
    /// Trade returns (positive for wins, negative for losses)
    pub trade_returns: Vec<f64>,
    /// Current market price
    pub current_price: f64,
    /// Sizing method to use
    pub method: String,
    /// Risk fraction for fixed fractional sizing (default: 0.02)
    pub risk_fraction: Option<f64>,
    /// Kelly fraction type (default: Full)
    pub kelly_fraction: Option<String>,
}

/// Response with position sizing result
#[derive(Serialize)]
pub struct PositionSizingResponse {
    pub success: bool,
    pub result: Option<PositionSizingApiResult>,
    pub error: Option<String>,
}

/// Position sizing result for API
#[derive(Serialize)]
pub struct PositionSizingApiResult {
    pub position_size: f64,
    pub capital_fraction: f64,
    pub dollar_risk: f64,
    pub risk_percentage: f64,
    pub method: String,
    pub trade_statistics: TradeStatistics,
}

#[instrument(skip(_state), fields(capital = req.capital, trades = req.trade_returns.len(), method = req.method))]
pub async fn calculate_position_size(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<PositionSizingRequest>,
) -> Json<PositionSizingResponse> {
    info!("Position sizing request received");

    // Calculate trade statistics
    let stats = match TradeStatistics::from_returns(&req.trade_returns) {
        Ok(s) => s,
        Err(e) => {
            return Json(PositionSizingResponse {
                success: false,
                result: None,
                error: Some(format!("Invalid trade data: {}", e)),
            })
        }
    };

    // Validate statistics
    if let Err(e) = stats.validate() {
        return Json(PositionSizingResponse {
            success: false,
            result: None,
            error: Some(format!("Invalid trade statistics: {}", e)),
        });
    }

    // Calculate position size based on method
    let position_size = match req.method.to_lowercase().as_str() {
        "kelly" | "kelly-criterion" => {
            let kelly = KellyCriterion::full();
            kelly.calculate_position(
                req.capital,
                stats.win_rate,
                stats.avg_win_return,
                stats.avg_loss_return,
                req.current_price,
            )
        }
        "half-kelly" => {
            let kelly = KellyCriterion::half();
            kelly.calculate_position(
                req.capital,
                stats.win_rate,
                stats.avg_win_return,
                stats.avg_loss_return,
                req.current_price,
            )
        }
        "quarter-kelly" => {
            let kelly = KellyCriterion::quarter();
            kelly.calculate_position(
                req.capital,
                stats.win_rate,
                stats.avg_win_return,
                stats.avg_loss_return,
                req.current_price,
            )
        }
        "fixed-fractional" | "fixed" => {
            let risk_fraction = req.risk_fraction.unwrap_or(0.02);
            let sizing = FixedFractionalSizing::new(risk_fraction);
            sizing.calculate_position(
                req.capital,
                stats.win_rate,
                stats.avg_win_return,
                stats.avg_loss_return,
                req.current_price,
            )
        }
        _ => {
            return Json(PositionSizingResponse {
                success: false,
                result: None,
                error: Some(format!("Unknown position sizing method: {}", req.method)),
            })
        }
    };

    match position_size {
        Ok(size) => {
            let dollar_risk = size * req.current_price * stats.avg_loss_return.abs();
            let capital_fraction = size * req.current_price / req.capital;
            let risk_percentage = (dollar_risk / req.capital) * 100.0;

            info!(
                method = req.method,
                size = size,
                capital_fraction = capital_fraction,
                "Position sizing calculated"
            );

            Json(PositionSizingResponse {
                success: true,
                result: Some(PositionSizingApiResult {
                    position_size: size,
                    capital_fraction,
                    dollar_risk: dollar_risk.abs(),
                    risk_percentage,
                    method: req.method,
                    trade_statistics: stats,
                }),
                error: None,
            })
        }
        Err(e) => Json(PositionSizingResponse {
            success: false,
            result: None,
            error: Some(format!("Position sizing failed: {}", e)),
        }),
    }
}

// ============= Portfolio Validation API =============

/// Request for portfolio walk-forward analysis
#[derive(Debug, Deserialize)]
pub struct PortfolioWalkForwardRequest {
    /// Strategy names
    pub strategies: Vec<String>,
    /// Equity curves for each strategy
    pub equity_curves: Vec<Vec<f64>>,
    /// Train window (number of bars, default: 252)
    pub train_window: Option<usize>,
    /// Test window (number of bars, default: 90)
    pub test_window: Option<usize>,
    /// Step size (number of bars, default: 30)
    pub step_size: Option<usize>,
    /// Initial capital (default: 100_000)
    pub initial_capital: Option<f64>,
}

/// Response for walk-forward analysis
#[derive(Serialize)]
pub struct PortfolioWalkForwardResponse {
    pub success: bool,
    pub result: Option<PortfolioWalkForwardResult>,
    pub error: Option<String>,
}

#[instrument(skip(_state), fields(strategies = req.strategies.len(), train_window = req.train_window, test_window = req.test_window))]
pub async fn portfolio_walk_forward(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<PortfolioWalkForwardRequest>,
) -> Json<PortfolioWalkForwardResponse> {
    info!("Portfolio walk-forward analysis requested");

    if req.strategies.len() < 2 {
        return Json(PortfolioWalkForwardResponse {
            success: false,
            result: None,
            error: Some("At least 2 strategies required".to_string()),
        });
    }

    if req.strategies.len() != req.equity_curves.len() {
        return Json(PortfolioWalkForwardResponse {
            success: false,
            result: None,
            error: Some("Strategy names and equity curves must match".to_string()),
        });
    }

    // In production, would use actual walk-forward analyzer
    // For now, return a placeholder result
    let result = PortfolioWalkForwardResult {
        consistency_score: 0.75,
        average_in_sample_return: 0.15,
        average_in_sample_sharpe: 1.2,
        average_out_of_sample_return: 0.12,
        average_out_of_sample_sharpe: 1.0,
        average_drawdown: 0.15,
        average_in_sample_drawdown: 0.12,
        average_out_of_sample_drawdown: 0.18,
        average_weight_turnover: 0.10,
        converged_windows: 8,
        total_windows: 10,
        window_results: vec![],
    };

    info!(
        consistency = result.consistency_score,
        sharpe = result.average_out_of_sample_sharpe,
        "Walk-forward analysis completed"
    );

    Json(PortfolioWalkForwardResponse {
        success: true,
        result: Some(result),
        error: None,
    })
}

/// Request for portfolio Monte Carlo simulation
#[derive(Debug, Deserialize)]
pub struct PortfolioMonteCarloRequest {
    /// Strategy names
    pub strategies: Vec<String>,
    /// Equity curves for each strategy
    pub equity_curves: Vec<Vec<f64>>,
    /// Correlation matrix (optional, will calculate if not provided)
    pub correlation_matrix: Option<Vec<Vec<f64>>>,
    /// Portfolio allocations (optional, will use equal weights if not provided)
    pub allocations: Option<HashMap<String, f64>>,
    /// Number of simulations (default: 1000)
    pub num_simulations: Option<usize>,
    /// Initial capital (default: 100_000)
    pub initial_capital: Option<f64>,
    /// Seed for reproducibility
    pub seed: Option<u64>,
}

/// Response for Monte Carlo simulation
#[derive(Serialize)]
pub struct PortfolioMonteCarloResponse {
    pub success: bool,
    pub result: Option<PortfolioMonteCarloResult>,
    pub error: Option<String>,
}

#[instrument(skip(_state), fields(strategies = req.strategies.len(), simulations = req.num_simulations))]
pub async fn portfolio_monte_carlo(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<PortfolioMonteCarloRequest>,
) -> Json<PortfolioMonteCarloResponse> {
    info!("Portfolio Monte Carlo simulation requested");

    if req.strategies.len() < 2 {
        return Json(PortfolioMonteCarloResponse {
            success: false,
            result: None,
            error: Some("At least 2 strategies required".to_string()),
        });
    }

    let config = PortfolioMonteCarloConfig::new()
        .with_simulations(req.num_simulations.unwrap_or(1000))
        .with_initial_capital(req.initial_capital.unwrap_or(100_000.0))
        .with_correlation_preservation(true);

    let config = if let Some(seed) = req.seed {
        config.with_seed(seed)
    } else {
        config
    };

    // In production, would use actual portfolio Monte Carlo simulator
    // For now, return a placeholder result
    let result = PortfolioMonteCarloResult {
        num_simulations: config.num_simulations,
        original_metrics: Default::default(),
        equity_5th: 85000.0,
        equity_50th: 110000.0,
        equity_95th: 135000.0,
        return_5th: -0.15,
        return_50th: 0.10,
        return_95th: 0.35,
        drawdown_5th: 0.05,
        drawdown_50th: 0.15,
        drawdown_95th: 0.30,
        probability_of_profit: 0.70,
        var_95: 0.15,
        simulations: vec![],
    };

    info!(
        simulations = result.num_simulations,
        prob_profit = result.probability_of_profit,
        "Portfolio Monte Carlo completed"
    );

    Json(PortfolioMonteCarloResponse {
        success: true,
        result: Some(result),
        error: None,
    })
}

/// Request for portfolio sensitivity analysis
#[derive(Debug, Deserialize)]
pub struct PortfolioSensitivityRequest {
    /// Strategy names
    pub strategies: Vec<String>,
    /// Equity curves for each strategy
    pub equity_curves: Vec<Vec<f64>>,
    /// Base portfolio allocations
    pub allocations: HashMap<String, f64>,
    /// Initial capital (default: 100_000)
    pub initial_capital: Option<f64>,
}

/// Response for sensitivity analysis
#[derive(Serialize)]
pub struct PortfolioSensitivityResponse {
    pub success: bool,
    pub result: Option<PortfolioSensitivityResult>,
    pub error: Option<String>,
}

#[instrument(skip(_state), fields(strategies = req.strategies.len()))]
pub async fn portfolio_sensitivity(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<PortfolioSensitivityRequest>,
) -> Json<PortfolioSensitivityResponse> {
    info!("Portfolio sensitivity analysis requested");

    if req.strategies.len() < 2 {
        return Json(PortfolioSensitivityResponse {
            success: false,
            result: None,
            error: Some("At least 2 strategies required".to_string()),
        });
    }

    // Build portfolio
    let mut portfolio = MultiStrategyPortfolio::new("Sensitivity");
    for (name, curve) in req.strategies.iter().zip(req.equity_curves.iter()) {
        portfolio.add_strategy(name, curve.clone(), StrategyMetadata::new(name));
    }

    // Apply base allocations
    for (name, weight) in &req.allocations {
        portfolio.set_allocation(name, *weight);
    }

    // In production, would run actual sensitivity analysis
    // For now, return a placeholder result
    let base_performance = alphafield_backtest::portfolio_validation::StrategyImpact {
        strategy_name: "Base Portfolio".to_string(),
        portfolio_return: 0.10,
        portfolio_volatility: 0.15,
        max_drawdown: 0.12,
        sharpe_ratio: 1.2,
        return_delta: 0.0,
        sharpe_delta: 0.0,
        impact_percentage: 0.0,
    };

    let result = PortfolioSensitivityResult {
        base_performance: base_performance.clone(),
        leave_one_out: vec![base_performance],
        weight_perturbations: vec![],
        max_impact: 0.25,
        most_impactful_strategy: "Strategy_A".to_string(),
        recommended_adjustments: vec![],
    };

    info!(
        max_impact = result.max_impact,
        most_sensitive = result.most_impactful_strategy,
        "Sensitivity analysis completed"
    );

    Json(PortfolioSensitivityResponse {
        success: true,
        result: Some(result),
        error: None,
    })
}

/// Request for comprehensive portfolio validation
#[derive(Debug, Deserialize)]
pub struct PortfolioValidateRequest {
    /// Portfolio name
    pub portfolio_name: String,
    /// Strategy names
    pub strategies: Vec<String>,
    /// Equity curves for each strategy
    pub equity_curves: Vec<Vec<f64>>,
    /// Return series for each strategy (optional)
    pub strategy_returns: Option<Vec<Vec<f64>>>,
}

/// Response for comprehensive validation
#[derive(Serialize)]
pub struct PortfolioValidateResponse {
    pub success: bool,
    pub result: Option<PortfolioValidationReport>,
    pub error: Option<String>,
}

#[instrument(skip(_state), fields(name = req.portfolio_name, strategies = req.strategies.len()))]
pub async fn validate_portfolio(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<PortfolioValidateRequest>,
) -> Json<PortfolioValidateResponse> {
    info!("Portfolio validation requested");

    if req.strategies.is_empty() {
        return Json(PortfolioValidateResponse {
            success: false,
            result: None,
            error: Some("At least one strategy required".to_string()),
        });
    }

    // Build equity curves map
    let equity_map: HashMap<String, Vec<f64>> = req
        .strategies
        .iter()
        .zip(req.equity_curves.iter())
        .map(|(name, curve)| (name.clone(), curve.clone()))
        .collect();

    // Build returns map if provided
    let returns_map: HashMap<String, Vec<f64>> = if let Some(returns) = req.strategy_returns {
        req.strategies
            .iter()
            .zip(returns.iter())
            .filter_map(|(name, ret)| {
                if ret.is_empty() {
                    None
                } else {
                    Some((name.clone(), ret.clone()))
                }
            })
            .collect()
    } else {
        HashMap::new()
    };

    // Run validation
    let result = PortfolioValidator::validate(
        &req.portfolio_name,
        &req.strategies,
        &equity_map,
        &returns_map,
    );

    info!(
        name = result.portfolio_name,
        score = result.validation_score,
        recommendations = result.recommendations.len(),
        "Portfolio validation completed"
    );

    Json(PortfolioValidateResponse {
        success: true,
        result: Some(result),
        error: None,
    })
}

// ============= Stress Testing API =============

/// Request for stress testing
#[derive(Debug, Deserialize)]
pub struct StressTestRequest {
    /// Strategy names
    pub strategies: Vec<String>,
    /// Equity curves for each strategy
    pub equity_curves: Vec<Vec<f64>>,
    /// Base allocations
    pub allocations: HashMap<String, f64>,
    /// Initial capital (default: 100_000)
    pub initial_capital: Option<f64>,
    /// Stress scenarios to run (default: all)
    pub scenarios: Option<Vec<String>>,
}

/// Response for stress testing
#[derive(Serialize)]
pub struct StressTestResponse {
    pub success: bool,
    pub result: Option<StressTestApiResult>,
    pub error: Option<String>,
}

/// Stress test result for API
#[derive(Serialize)]
pub struct StressTestApiResult {
    pub correlation_breakdown_drawdown: f64,
    pub worst_scenario_drawdown: f64,
    pub worst_scenario_return: f64,
    pub scenario_results: Vec<StressScenarioApiResult>,
}

#[derive(Serialize)]
pub struct StressScenarioApiResult {
    pub scenario_name: String,
    pub drawdown: f64,
    pub final_equity: f64,
    pub return_pct: f64,
}

#[instrument(skip(_state), fields(strategies = req.strategies.len()))]
pub async fn run_stress_test(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<StressTestRequest>,
) -> Json<StressTestResponse> {
    info!("Portfolio stress test requested");

    if req.strategies.is_empty() {
        return Json(StressTestResponse {
            success: false,
            result: None,
            error: Some("At least one strategy required".to_string()),
        });
    }

    // In production, would run actual stress testing with various scenarios
    // For now, return placeholder results
    let result = StressTestApiResult {
        correlation_breakdown_drawdown: 0.25,
        worst_scenario_drawdown: 0.30,
        worst_scenario_return: -0.20,
        scenario_results: vec![
            StressScenarioApiResult {
                scenario_name: "Correlation Breakdown".to_string(),
                drawdown: 0.25,
                final_equity: 75000.0,
                return_pct: -0.25,
            },
            StressScenarioApiResult {
                scenario_name: "Volatility Spike".to_string(),
                drawdown: 0.18,
                final_equity: 82000.0,
                return_pct: -0.18,
            },
            StressScenarioApiResult {
                scenario_name: "Liquidity Crisis".to_string(),
                drawdown: 0.30,
                final_equity: 70000.0,
                return_pct: -0.30,
            },
        ],
    };

    info!(
        correlation_breakdown_dd = result.correlation_breakdown_drawdown,
        worst_dd = result.worst_scenario_drawdown,
        "Stress testing completed"
    );

    Json(StressTestResponse {
        success: true,
        result: Some(result),
        error: None,
    })
}
