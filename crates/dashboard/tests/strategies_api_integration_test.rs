// Integration test for strategies API
use alphafield_dashboard::api::AppState;
use alphafield_dashboard::strategies_api::{initialize_registry, list_strategies, StrategyQuery};
use axum::extract::State;
use std::sync::Arc;

#[tokio::test]
async fn test_strategies_api_integration() {
    // Initialize the registry
    let registry = initialize_registry();

    // Create AppState with the registry
    let state = Arc::new(AppState {
        db: None,
        hub: Arc::new(alphafield_dashboard::websocket::DashboardHub::new(100)),
        registry: registry.clone(),
    });

    // Test that we can list all strategies
    let query = StrategyQuery {
        category: None,
        regime: None,
    };

    let result = list_strategies(State(state.clone()), axum::extract::Query(query)).await;

    assert!(result.is_ok(), "Should be able to list strategies");
    let strategies = result.unwrap();
    let strategies_vec = strategies.0;

    println!("Found {} strategies:", strategies_vec.len());
    for strategy in &strategies_vec {
        println!("- {} ({})", strategy.name, strategy.category);
    }

    // Verify we have the expected strategies
    let strategy_names: Vec<String> = strategies_vec.iter().map(|s| s.name.clone()).collect();

    // Baselines
    assert!(
        strategy_names.contains(&"HODL_Baseline".to_string()),
        "Should have HODL baseline"
    );
    assert!(
        strategy_names.contains(&"Market_Average_Baseline".to_string()),
        "Should have Market Average baseline"
    );

    // Trend Following (Phase 12.2)
    assert!(
        strategy_names.contains(&"Golden Cross".to_string()),
        "Should have Golden Cross strategy"
    );
    assert!(
        strategy_names.contains(&"Breakout".to_string()),
        "Should have Breakout strategy"
    );
    assert!(
        strategy_names.contains(&"MA Crossover".to_string()),
        "Should have MA Crossover strategy"
    );
    assert!(
        strategy_names.contains(&"Adaptive MA".to_string()),
        "Should have Adaptive MA strategy"
    );
    assert!(
        strategy_names.contains(&"Triple MA".to_string()),
        "Should have Triple MA strategy"
    );
    assert!(
        strategy_names.contains(&"MACD Trend".to_string()),
        "Should have MACD Trend strategy"
    );
    assert!(
        strategy_names.contains(&"Parabolic SAR".to_string()),
        "Should have Parabolic SAR strategy"
    );

    // Other strategy families
    assert!(
        strategy_names.contains(&"EMA-MACD Momentum".to_string()),
        "Should have Momentum strategy"
    );
    assert!(
        strategy_names.contains(&"RSI Mean Reversion".to_string()),
        "Should have RSI strategy"
    );
    assert!(
        strategy_names.contains(&"Bollinger Bands Mean Reversion".to_string()),
        "Should have Bollinger Bands strategy"
    );

    // Test filtering by category
    let query = StrategyQuery {
        category: Some("TrendFollowing".to_string()),
        regime: None,
    };

    let result = list_strategies(State(state.clone()), axum::extract::Query(query)).await;

    assert!(result.is_ok(), "Should be able to filter by category");
    let trend_following = result.unwrap().0;

    let trend_names: Vec<String> = trend_following.iter().map(|s| s.name.clone()).collect();
    assert_eq!(
        trend_following.len(),
        7,
        "Should have 7 trend following strategies"
    );
    assert!(
        trend_names.contains(&"Golden Cross".to_string()),
        "Should include Golden Cross"
    );
    assert!(
        trend_names.contains(&"Breakout".to_string()),
        "Should include Breakout"
    );
    assert!(
        trend_names.contains(&"MA Crossover".to_string()),
        "Should include MA Crossover"
    );
    assert!(
        trend_names.contains(&"Adaptive MA".to_string()),
        "Should include Adaptive MA"
    );
    assert!(
        trend_names.contains(&"Triple MA".to_string()),
        "Should include Triple MA"
    );
    assert!(
        trend_names.contains(&"MACD Trend".to_string()),
        "Should include MACD Trend"
    );
    assert!(
        trend_names.contains(&"Parabolic SAR".to_string()),
        "Should include Parabolic SAR"
    );

    // Test filtering by regime
    let query = StrategyQuery {
        category: None,
        regime: Some("Bull".to_string()),
    };

    let result = list_strategies(State(state.clone()), axum::extract::Query(query)).await;

    assert!(result.is_ok(), "Should be able to filter by regime");
    let bull_strategies = result.unwrap().0;
    println!("Found {} strategies for Bull regime", bull_strategies.len());

    // Test getting strategy details
    use alphafield_dashboard::strategies_api::get_strategy_details;
    use axum::extract::Path;

    let result = get_strategy_details(State(state.clone()), Path("Golden Cross".to_string())).await;

    assert!(result.is_ok(), "Should be able to get strategy details");
    let details = result.unwrap();
    assert_eq!(details.metadata.name, "Golden Cross");

    println!("✅ Strategies API integration test passed!");
}
