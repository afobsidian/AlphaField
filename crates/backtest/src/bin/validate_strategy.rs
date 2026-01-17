//! # Strategy Validation CLI Tool
//!
//! Command-line tool for validating trading strategies without requiring
//! full dashboard integration.
use alphafield_backtest::validation::MarketRegime as BacktestRegime;
use alphafield_backtest::RegimeAnalyzer;
use alphafield_backtest::{
    Strategy, StrategyAdapter, StrategyValidator, ValidationConfig, ValidationReport,
    ValidationThresholds, WalkForwardConfig,
};
use alphafield_core::Bar;
use alphafield_strategy::{
    framework::{
        canonicalize_strategy_name, MetadataStrategy, StrategyCategory, StrategyMetadata,
        StrategyWithMetadata,
    },
    // Multi-indicator strategies
    AdaptiveComboStrategy,
    // Trend following additional
    AdaptiveMAStrategy,
    // Momentum strategies
    AdxTrendStrategy,
    BollingerBandsStrategy,
    BreakoutStrategy,
    ConfidenceWeightedStrategy,
    // Sentiment strategies
    DivergenceStrategy,
    EnsembleWeightedStrategy,
    // Trend following strategies
    GoldenCrossStrategy,
    // Mean reversion additional
    KeltnerReversionStrategy,
    MACDRSIComboStrategy,
    MACDStrategy,
    MACrossoverStrategy,
    MLEnhancedStrategy,
    MacdTrendStrategy,
    MomentumFactorStrategy,
    MultiTfMomentumStrategy,
    ParabolicSARStrategy,
    PriceChannelStrategy,
    RSIReversionStrategy,
    RegimeSentimentStrategy,
    RegimeSwitchingStrategy,
    RocStrategy,
    RsiMomentumStrategy,
    // Mean reversion strategies
    RsiStrategy,
    SentimentMomentumStrategy,
    StatArbStrategy,
    StochReversionStrategy,
    TrendMeanRevStrategy,
    TripleMAStrategy,
    VolumeMomentumStrategy,
    ZScoreReversionStrategy,
};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Parser)]
#[command(
    name = "validate_strategy",
    author = "AlphaField Team",
    version = "1.0.0",
    about = "Automated strategy validation framework for AlphaField",
    long_about = "Validate trading strategies through comprehensive testing including backtesting, walk-forward analysis, Monte Carlo simulation, and regime-based performance analysis."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Quiet mode (suppress all output except errors)
    #[arg(short, long, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a single strategy
    Validate {
        /// Strategy name (e.g., golden_cross, rsi_strategy)
        #[arg(long)]
        strategy: String,

        /// Symbol to test (e.g., BTC, ETH)
        #[arg(long)]
        symbol: String,

        /// Timeframe/interval (e.g., 1h, 4h, 1d)
        #[arg(long)]
        interval: String,

        /// Run walk-forward analysis
        #[arg(long, default_value = "false")]
        walk_forward: bool,

        /// Run Monte Carlo simulation
        #[arg(long, default_value = "false")]
        monte_carlo: bool,

        /// Run regime analysis
        #[arg(long, default_value = "false")]
        regime_analysis: bool,

        /// Output file path
        #[arg(long)]
        output: Option<String>,

        /// Output format (terminal, json, yaml, markdown)
        #[arg(long, default_value = "terminal")]
        format: OutputFormat,

        /// Minimum Sharpe ratio threshold
        #[arg(long, default_value = "1.0")]
        min_sharpe: f64,

        /// Maximum acceptable drawdown (e.g., 0.30 for 30%)
        #[arg(long, default_value = "0.30")]
        max_drawdown: f64,

        /// Minimum walk-forward win rate
        #[arg(long, default_value = "0.60")]
        min_win_rate: f64,

        /// Minimum Monte Carlo positive probability
        #[arg(long, default_value = "0.70")]
        min_positive_probability: f64,

        /// Initial capital
        #[arg(long, default_value = "10000.0")]
        initial_capital: f64,

        /// Fee rate
        #[arg(long, default_value = "0.001")]
        fee_rate: f64,

        /// Risk-free rate for Sharpe calculation
        #[arg(long, default_value = "0.02")]
        risk_free_rate: f64,
    },

    /// Batch validate multiple strategies
    Batch {
        /// Batch file path (one strategy per line)
        #[arg(long)]
        batch_file: String,

        /// Comma-separated symbols to test
        #[arg(long)]
        symbols: String,

        /// Timeframe/interval
        #[arg(long)]
        interval: String,

        /// Output directory for reports
        #[arg(long)]
        output_dir: String,

        /// Output format (json, yaml, markdown)
        #[arg(long, default_value = "json")]
        format: OutputFormat,
    },

    /// List all available strategies
    ListStrategies {
        /// Filter by category (trend_following, mean_reversion, momentum, volatility, sentiment, baseline)
        #[arg(long)]
        category: Option<String>,
    },
}

/// Strategy factory function type
type StrategyFactory = fn() -> Box<dyn StrategyWithMetadata>;

lazy_static! {
    /// Global strategy factory registry
    ///
    /// This registry maps canonical strategy names to factory functions and metadata.
    /// Using factory functions allows creating fresh strategy instances on demand,
    /// which is essential for backtesting where each test needs an independent strategy.
    ///
    /// # Key Features:
    /// - Lazy initialization: Registry is built once on first access
    /// - Factory pattern: Each entry returns a new strategy instance
    /// - Metadata included: Enables listing and categorization
    /// - Canonical names: Supports fuzzy matching (underscores, dashes, spaces)
    ///
    /// # Registration Process:
    /// 1. Create default instance of strategy
    /// 2. Extract metadata (name, category, description, expected regimes)
    /// 3. Canonicalize the strategy name for consistent lookup
    /// 4. Store factory function and metadata in HashMap
    static ref STRATEGY_FACTORY_REGISTRY: HashMap<String, (StrategyFactory, StrategyMetadata)> = {
        let mut registry = HashMap::new();

        // Helper macro to register strategies
        // This macro eliminates boilerplate code by:
        // - Creating a default instance to extract metadata
        // - Canonicalizing the strategy name for consistent lookups
        // - Creating a factory function for new instances
        macro_rules! register_strategy {
            ($strategy:ty) => {
                let instance = <$strategy>::default();
                let metadata = instance.metadata();
                let canonical_name = canonicalize_strategy_name(&metadata.name);
                let factory: StrategyFactory = || Box::new(<$strategy>::default());
                registry.insert(canonical_name, (factory, metadata));
            };
        }

        // Register trend following strategies
        // These strategies perform best in trending markets
        register_strategy!(GoldenCrossStrategy);
        register_strategy!(MacdTrendStrategy);
        register_strategy!(AdaptiveMAStrategy);
        register_strategy!(BreakoutStrategy);
        register_strategy!(MACrossoverStrategy);
        register_strategy!(ParabolicSARStrategy);
        register_strategy!(TripleMAStrategy);

        // Register mean reversion strategies
        // These strategies perform best in ranging/sideways markets
        register_strategy!(RsiStrategy);
        register_strategy!(BollingerBandsStrategy);
        register_strategy!(KeltnerReversionStrategy);
        register_strategy!(PriceChannelStrategy);
        register_strategy!(RSIReversionStrategy);
        register_strategy!(StatArbStrategy);
        register_strategy!(StochReversionStrategy);
        register_strategy!(ZScoreReversionStrategy);

        // Register momentum strategies
        // These strategies identify and follow strong price trends
        register_strategy!(AdxTrendStrategy);
        register_strategy!(MACDStrategy);
        register_strategy!(MomentumFactorStrategy);
        register_strategy!(MultiTfMomentumStrategy);
        register_strategy!(RocStrategy);
        register_strategy!(RsiMomentumStrategy);
        register_strategy!(VolumeMomentumStrategy);

        // Register multi-indicator strategies
        // These strategies combine multiple indicators in sophisticated ways
        register_strategy!(AdaptiveComboStrategy);
        register_strategy!(ConfidenceWeightedStrategy);
        register_strategy!(EnsembleWeightedStrategy);
        register_strategy!(MACDRSIComboStrategy);
        register_strategy!(MLEnhancedStrategy);
        register_strategy!(RegimeSwitchingStrategy);
        register_strategy!(TrendMeanRevStrategy);

        // Register sentiment strategies
        // These strategies combine technical indicators with market sentiment
        register_strategy!(DivergenceStrategy);
        register_strategy!(RegimeSentimentStrategy);
        register_strategy!(SentimentMomentumStrategy);

        registry
    };
}

#[derive(Debug, Clone, PartialEq)]
enum OutputFormat {
    Terminal,
    Json,
    Yaml,
    Markdown,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "terminal" => Ok(OutputFormat::Terminal),
            "json" => Ok(OutputFormat::Json),
            "yaml" => Ok(OutputFormat::Yaml),
            "markdown" => Ok(OutputFormat::Markdown),
            _ => Err(format!("Invalid output format: {}", s)),
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Terminal => write!(f, "terminal"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Yaml => write!(f, "yaml"),
            OutputFormat::Markdown => write!(f, "markdown"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Try to load .env, but fallback to manual parsing if it fails
    if dotenvy::dotenv().is_err() {
        // Manual fallback: read DATABASE_URL directly from .env
        if let Ok(contents) = fs::read_to_string(".env") {
            for line in contents.lines() {
                if line.starts_with("DATABASE_URL=") {
                    let value = line.trim_start_matches("DATABASE_URL=");
                    std::env::set_var("DATABASE_URL", value);
                    break;
                }
            }
        }
    }
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate {
            strategy,
            symbol,
            interval,
            walk_forward: _wf,
            monte_carlo: _mc,
            regime_analysis: _ra,
            output,
            format,
            min_sharpe,
            max_drawdown,
            min_win_rate,
            min_positive_probability,
            initial_capital,
            fee_rate,
            risk_free_rate,
        } => {
            let bars = load_bars(&symbol, &interval).await?;
            let config = ValidationConfig {
                data_source: format!("database:{}", symbol),
                symbol: symbol.clone(),
                interval: interval.clone(),
                walk_forward: WalkForwardConfig::default(),
                risk_free_rate,
                thresholds: ValidationThresholds {
                    min_sharpe,
                    max_drawdown,
                    min_win_rate,
                    min_positive_probability,
                },
                initial_capital,
                fee_rate,
            };

            // Look up strategy factory from registry for enhanced walk-forward
            let normalized_name = canonicalize_strategy_name(&strategy);
            let (core_factory, metadata) = STRATEGY_FACTORY_REGISTRY
                .get(&normalized_name)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Unknown strategy: '{}'. Available strategies: {}. Use --list-strategies to see all options.",
                        strategy,
                        STRATEGY_FACTORY_REGISTRY
                            .keys()
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })?;

            // Create wrapped factory that adds StrategyAdapter
            let symbol_clone = symbol.clone();
            let wrapped_factory = move || {
                let core_strategy = core_factory();
                Box::new(StrategyAdapter::new(
                    core_strategy,
                    &symbol_clone,
                    initial_capital,
                )) as Box<dyn Strategy>
            };

            // **ENHANCED**: Use validate_with_factory for proper walk-forward
            let validator = StrategyValidator::new(config);
            let report = validator
                .validate_with_factory(wrapped_factory, &symbol, &bars)
                .context("Validation failed")?;

            // Run regime analysis separately if strategy has metadata
            let report = {
                // metadata is now a reference from the registry tuple
                println!("📊 Strategy has metadata, running regime analysis...");

                // Convert core MarketRegime to validation MarketRegime
                use alphafield_backtest::validation::MarketRegime as BacktestRegime;
                let expected_regimes = metadata
                    .expected_regimes
                    .iter()
                    .map(|r| match r {
                        alphafield_strategy::MarketRegime::Bull => BacktestRegime::Bull,
                        alphafield_strategy::MarketRegime::Bear => BacktestRegime::Bear,
                        alphafield_strategy::MarketRegime::Sideways => BacktestRegime::Sideways,
                        alphafield_strategy::MarketRegime::HighVolatility => {
                            BacktestRegime::HighVolatility
                        }
                        alphafield_strategy::MarketRegime::LowVolatility => {
                            BacktestRegime::LowVolatility
                        }
                        alphafield_strategy::MarketRegime::Trending => BacktestRegime::Trending,
                        alphafield_strategy::MarketRegime::Ranging => BacktestRegime::Ranging,
                    })
                    .collect();

                // Run regime analysis
                let analyzer = RegimeAnalyzer::default();
                let regime_result = analyzer.analyze(&bars, expected_regimes);

                // Merge regime analysis into report
                ValidationReport {
                    regime_analysis: regime_result.unwrap_or_default(),
                    ..report
                }
            };

            // Output report
            match format {
                OutputFormat::Terminal => {
                    print_terminal_report(&report);
                }
                OutputFormat::Json => {
                    let json = serde_json::to_string_pretty(&report)?;
                    output_report(&output, &json)?;
                }
                OutputFormat::Yaml => {
                    let yaml = serde_yaml::to_string(&report)?;
                    output_report(&output, &yaml)?;
                }
                OutputFormat::Markdown => {
                    let markdown = generate_markdown_report(&report)?;
                    output_report(&output, &markdown)?;
                }
            }

            // Set exit code based on verdict
            std::process::exit(match report.verdict {
                alphafield_backtest::ValidationVerdict::Pass => 0,
                alphafield_backtest::ValidationVerdict::Fail => 1,
                alphafield_backtest::ValidationVerdict::NeedsOptimization => 1,
            });
        }
        Commands::Batch {
            batch_file,
            symbols,
            interval,
            output_dir,
            format,
        } => {
            let strategies = fs::read_to_string(&batch_file)
                .context(format!("Failed to read batch file: {}", batch_file))?;

            let symbols_list: Vec<String> =
                symbols.split(',').map(|s| s.trim().to_string()).collect();
            let strategy_names: Vec<String> = strategies
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_string())
                .collect();

            // Create output directory if it doesn't exist
            fs::create_dir_all(&output_dir)
                .context(format!("Failed to create output directory: {}", output_dir))?;

            // Use atomic counters for thread-safe parallel processing
            let total_validations = Arc::new(AtomicUsize::new(0));
            let passed_validations = Arc::new(AtomicUsize::new(0));
            let failed_validations = Arc::new(AtomicUsize::new(0));

            // Create semaphore to limit concurrent database connections to 100
            // (PostgreSQL default max_connections)
            let semaphore = Arc::new(Semaphore::new(100));

            println!(
                "{}",
                "Starting batch validation with parallel processing..."
                    .blue()
                    .bold()
            );
            println!(
                "Strategies: {}, Symbols: {}, Total validations: {} (max concurrent: 100)\n",
                strategy_names.len(),
                symbols_list.len(),
                strategy_names.len() * symbols_list.len()
            );

            // Collect all validation tasks
            let mut tasks = Vec::new();

            for strategy_name in strategy_names {
                for symbol in symbols_list.clone() {
                    let interval_clone = interval.clone();
                    let output_dir_clone = output_dir.clone();
                    let format_clone = format.clone();
                    let total = Arc::clone(&total_validations);
                    let passed = Arc::clone(&passed_validations);
                    let failed = Arc::clone(&failed_validations);
                    let strategy_name_clone = strategy_name.clone();
                    let semaphore = Arc::clone(&semaphore);

                    // Spawn validation task
                    let task = tokio::spawn(async move {
                        // Acquire semaphore permit to limit concurrent database connections
                        let _permit = semaphore.acquire().await.unwrap();

                        total.fetch_add(1, Ordering::SeqCst);
                        println!("Validating {}/{}...", strategy_name_clone, symbol);

                        let bars = match load_bars(&symbol, &interval_clone).await {
                            Ok(b) => b,
                            Err(e) => {
                                println!(
                                    "{}",
                                    format!(
                                        "Failed to load data for {}/{}: {}",
                                        strategy_name_clone, symbol, e
                                    )
                                    .red()
                                );
                                failed.fetch_add(1, Ordering::SeqCst);
                                return;
                            }
                        };

                        let config = ValidationConfig {
                            data_source: format!("database:{}", symbol),
                            symbol: symbol.clone(),
                            interval: interval_clone.clone(),
                            walk_forward: WalkForwardConfig::default(),
                            risk_free_rate: 0.02,
                            thresholds: ValidationThresholds::default(),
                            initial_capital: 10000.0,
                            fee_rate: 0.001,
                        };

                        // Look up strategy factory from registry for enhanced walk-forward
                        let normalized_name = canonicalize_strategy_name(&strategy_name_clone);
                        let (core_factory, metadata) =
                            match STRATEGY_FACTORY_REGISTRY.get(&normalized_name) {
                                Some(f) => f,
                                None => {
                                    println!(
                                        "{}",
                                        format!("Failed to find strategy: {}", strategy_name_clone)
                                            .red()
                                    );
                                    failed.fetch_add(1, Ordering::SeqCst);
                                    return;
                                }
                            };

                        // Create wrapped factory that adds StrategyAdapter
                        let symbol_for_factory = symbol.clone();
                        let wrapped_factory = move || {
                            let core_strategy = core_factory();
                            Box::new(StrategyAdapter::new(
                                core_strategy,
                                &symbol_for_factory,
                                10000.0,
                            )) as Box<dyn Strategy>
                        };

                        // **ENHANCED**: Use validate_with_factory for proper walk-forward
                        let validator = StrategyValidator::new(config);
                        let report = match validator.validate_with_factory(
                            wrapped_factory,
                            &symbol,
                            &bars,
                        ) {
                            Ok(r) => r,
                            Err(e) => {
                                println!(
                                    "{}",
                                    format!(
                                        "Validation failed for {}/{}: {}",
                                        strategy_name_clone, symbol, e
                                    )
                                    .red()
                                );
                                failed.fetch_add(1, Ordering::SeqCst);
                                return;
                            }
                        };

                        // Run regime analysis separately if strategy has metadata
                        let report = {
                            println!(
                                "📊 Strategy has metadata, running regime analysis for {}/{}...",
                                strategy_name_clone, symbol
                            );

                            // Convert core MarketRegime to validation MarketRegime
                            let expected_regimes = metadata
                                .expected_regimes
                                .iter()
                                .map(|r| match r {
                                    alphafield_strategy::MarketRegime::Bull => BacktestRegime::Bull,
                                    alphafield_strategy::MarketRegime::Bear => BacktestRegime::Bear,
                                    alphafield_strategy::MarketRegime::Sideways => {
                                        BacktestRegime::Sideways
                                    }
                                    alphafield_strategy::MarketRegime::HighVolatility => {
                                        BacktestRegime::HighVolatility
                                    }
                                    alphafield_strategy::MarketRegime::LowVolatility => {
                                        BacktestRegime::LowVolatility
                                    }
                                    alphafield_strategy::MarketRegime::Trending => {
                                        BacktestRegime::Trending
                                    }
                                    alphafield_strategy::MarketRegime::Ranging => {
                                        BacktestRegime::Ranging
                                    }
                                })
                                .collect();

                            // Run regime analysis
                            let analyzer = RegimeAnalyzer::default();
                            let regime_result = analyzer.analyze(&bars, expected_regimes);

                            // Merge regime analysis into report
                            ValidationReport {
                                regime_analysis: regime_result.unwrap_or_default(),
                                ..report
                            }
                        };

                        // Save report
                        let filename = format!(
                            "{}/{}_{}.{}",
                            output_dir_clone, strategy_name_clone, symbol, format_clone
                        );
                        let report_content = match format_clone {
                            OutputFormat::Json => serde_json::to_string_pretty(&report).unwrap(),
                            OutputFormat::Yaml => serde_yaml::to_string(&report).unwrap(),
                            OutputFormat::Markdown => generate_markdown_report(&report).unwrap(),
                            OutputFormat::Terminal => {
                                // For terminal format, use JSON for batch
                                serde_json::to_string_pretty(&report).unwrap()
                            }
                        };

                        if let Err(e) = fs::write(&filename, report_content) {
                            println!("{}", format!("Failed to write report: {}", e).red());
                            failed.fetch_add(1, Ordering::SeqCst);
                            return;
                        }

                        let verdict_color = match report.verdict {
                            alphafield_backtest::ValidationVerdict::Pass => "green",
                            alphafield_backtest::ValidationVerdict::NeedsOptimization => "yellow",
                            alphafield_backtest::ValidationVerdict::Fail => "red",
                        };

                        println!(
                            "{} (Score: {:.1}, Grade: {})\n",
                            format!(
                                "Verdict: {}",
                                format!("{:?}", report.verdict).to_uppercase()
                            )
                            .color(verdict_color)
                            .bold(),
                            report.overall_score,
                            report.grade
                        );

                        match report.verdict {
                            alphafield_backtest::ValidationVerdict::Pass => {
                                passed.fetch_add(1, Ordering::SeqCst);
                            }
                            _ => {
                                failed.fetch_add(1, Ordering::SeqCst);
                            }
                        }
                    });

                    tasks.push(task);
                }
            }

            // Wait for all validation tasks to complete
            for task in tasks {
                let _ = task.await;
            }

            // Print summary
            println!("\n{}", "Batch Validation Summary".blue().bold());
            println!(
                "Total validations: {}",
                total_validations.load(Ordering::SeqCst)
            );
            println!(
                "{}",
                format!("Passed: {}", passed_validations.load(Ordering::SeqCst)).green()
            );
            println!(
                "{}",
                format!("Failed: {}", failed_validations.load(Ordering::SeqCst)).red()
            );

            std::process::exit(if failed_validations.load(Ordering::SeqCst) > 0 {
                1
            } else {
                0
            });
        }
        Commands::ListStrategies { category } => {
            println!("{}", "Available Strategies".cyan().bold());
            println!("{}", "=".repeat(50));

            if let Some(cat_filter) = category {
                // Parse category string to enum
                // Supports flexible input: "trend_following", "Trend Following", etc.
                let target_category = match cat_filter.to_lowercase().as_str() {
                    "trend_following" => StrategyCategory::TrendFollowing,
                    "mean_reversion" => StrategyCategory::MeanReversion,
                    "momentum" => StrategyCategory::Momentum,
                    "volatility" => StrategyCategory::VolatilityBased,
                    "sentiment" => StrategyCategory::SentimentBased,
                    "baseline" => StrategyCategory::Baseline,
                    _ => {
                        eprintln!("Unknown category: {}. Valid options: trend_following, mean_reversion, momentum, volatility, sentiment, baseline", cat_filter);
                        std::process::exit(1);
                    }
                };

                // Filter strategies by category and display
                let filtered: Vec<_> = STRATEGY_FACTORY_REGISTRY
                    .iter()
                    .filter(|(_, (_, metadata))| metadata.category == target_category)
                    .collect();

                println!("\n{} ({})\n", "Category".yellow().bold(), cat_filter);
                for (name, (_, metadata)) in &filtered {
                    print_strategy_info(name, metadata);
                }
            } else {
                // List all strategies grouped by category
                // This provides a better user experience than a flat list
                let categories = vec![
                    (StrategyCategory::TrendFollowing, "Trend Following"),
                    (StrategyCategory::MeanReversion, "Mean Reversion"),
                    (StrategyCategory::Momentum, "Momentum"),
                    (StrategyCategory::VolatilityBased, "Volatility"),
                    (StrategyCategory::SentimentBased, "Sentiment"),
                ];

                for (category, category_name) in categories {
                    let filtered: Vec<_> = STRATEGY_FACTORY_REGISTRY
                        .iter()
                        .filter(|(_, (_, metadata))| metadata.category == category)
                        .collect();

                    if !filtered.is_empty() {
                        println!(
                            "\n{} ({})\n",
                            category_name.cyan().bold(),
                            format!("{:?}", category).to_lowercase().replace("_", " ")
                        );
                        for (name, (_, metadata)) in &filtered {
                            print_strategy_info(name, metadata);
                        }
                    }
                }
            }

            println!("\n{}", "=".repeat(50));
            println!(
                "Total strategies available: {}",
                STRATEGY_FACTORY_REGISTRY.len()
            );
        }
    }
    Ok(())
}

/// Print strategy information to stdout
///
/// This helper function formats and displays strategy metadata in a user-friendly way.
/// It handles long descriptions by truncating and formats enums for readability.
///
/// # Display Format
/// ```text
///   • StrategyName
///     Category: category name
///     Type: sub_type (if present)
///     Description: Brief description (truncated to 80 chars)
///     Expected regimes: Regime1, Regime2, ...
/// ```
///
/// # Arguments
/// * `name` - Canonical strategy name (key from registry)
/// * `metadata` - Strategy metadata containing all details
fn print_strategy_info(name: &str, metadata: &StrategyMetadata) {
    println!("  • {}", name.green().bold());
    println!(
        "    Category: {}",
        format!("{:?}", metadata.category)
            .to_lowercase()
            .replace("_", " ")
    );

    if let Some(sub_type) = &metadata.sub_type {
        println!("    Type: {}", sub_type);
    }

    if !metadata.description.is_empty() {
        // Truncate long descriptions for better terminal display
        let desc = if metadata.description.len() > 80 {
            format!("{}...", &metadata.description[..77])
        } else {
            metadata.description.clone()
        };
        println!("    {}", desc.dimmed());
    }

    if !metadata.expected_regimes.is_empty() {
        // Convert regime enums to readable strings
        let regimes: Vec<String> = metadata
            .expected_regimes
            .iter()
            .map(|r| format!("{:?}", r))
            .collect();
        println!("    Expected regimes: {}", regimes.join(", "));
    }

    println!();
}

/// Load bars from database
async fn load_bars(symbol: &str, interval: &str) -> Result<Vec<Bar>> {
    println!("📡 Connecting to database...");
    let db = alphafield_data::DatabaseClient::new_from_env()
        .await
        .context(
            "Failed to connect to database. Make sure DATABASE_URL is set in your .env file",
        )?;

    println!("🔍 Checking if data exists in database...");
    if db.exists(symbol, interval).await? {
        println!("✅ Loading historical data from database...");
        let bars = db.load_bars(symbol, interval).await?;
        println!("Loaded {} bars from database", bars.len());
        Ok(bars)
    } else {
        println!("🌐 Data not in database, fetching from API...");
        let client = alphafield_data::UnifiedDataClient::new_from_env();
        let bars = client
            .get_bars(symbol, interval, None, None, Some(1000))
            .await
            .context("Failed to fetch data from API. Check your API keys.")?;

        println!("💾 Saving {} bars to database...", bars.len());
        db.save_bars(symbol, interval, &bars)
            .await
            .context("Failed to save data to database")?;
        println!("✅ Saved {} bars to database", bars.len());
        Ok(bars)
    }
}

/// Create strategy instance from name using the global factory registry
///
/// # Arguments
/// * `name` - Strategy name (supports multiple formats: "golden_cross", "Golden Cross", etc.)
/// * `symbol` - Trading symbol for the strategy (e.g., "BTCUSDT")
/// * `capital` - Initial capital for position sizing
///
/// # Returns
/// * `Ok(Box<dyn Strategy>)` - Wrapped strategy ready for backtesting
/// * `Err(QuantError)` - If strategy name is not found
///
/// # Name Normalization
/// This function supports flexible strategy name input:
/// - Underscored: "golden_cross", "macd_trend"
/// - Spaced: "Golden Cross", "MACD Trend"
/// - Dashed: "golden-cross", "macd-trend"
/// - Already canonical: "GoldenCross", "MacdTrend"
///
/// # Architecture
/// 1. Normalize input (underscores/dashes → spaces)
/// 2. Canonicalize using shared function (display name → key)
/// 3. Lookup factory in registry
/// 4. Create fresh instance via factory
/// 5. Wrap with StrategyAdapter for backtest compatibility
///
/// # Why Factories?
/// Each validation/backtest needs a fresh strategy instance with clean state.
/// The factory pattern ensures this without cloning complex strategy objects.
#[allow(dead_code)]
fn create_strategy(name: &str, symbol: &str, capital: f64) -> Result<Box<dyn Strategy>> {
    // Normalize strategy name before canonicalization
    // Handle common variations: underscores, dashes, spaces, etc.
    let normalized_name = name
        .to_string()
        .replace("_", " ") // "golden_cross" -> "golden cross"
        .replace("-", " ") // "golden-cross" -> "golden cross"
        .trim()
        .to_string();

    // Canonicalize the strategy name using the shared function
    // This converts display names to internal keys
    let canonical_name = canonicalize_strategy_name(&normalized_name);

    // Look up strategy factory in the registry
    let (factory, _metadata) = STRATEGY_FACTORY_REGISTRY.get(&canonical_name).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown strategy: '{}'. Available strategies: {}. Use --list-strategies to see all options.",
            name,
            STRATEGY_FACTORY_REGISTRY
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;

    // Create a new strategy instance using the factory
    // This ensures clean state for each backtest
    let core_strategy = factory();

    // Wrap core strategy with StrategyAdapter for backtest use
    // Core strategies use Signals, backtest needs Orders
    Ok(Box::new(StrategyAdapter::new(
        core_strategy,
        symbol,
        capital,
    )))
}

/// Output report to file or stdout
fn output_report(output: &Option<String>, content: &str) -> Result<()> {
    match output {
        Some(path) => {
            let mut file =
                File::create(path).context(format!("Failed to create output file: {}", path))?;
            file.write_all(content.as_bytes())
                .context(format!("Failed to write to output file: {}", path))?;
            println!("Report saved to: {}", path);
        }
        None => {
            println!("{}", content);
        }
    }
    Ok(())
}

/// Print report in terminal format
fn print_terminal_report(report: &ValidationReport) {
    println!("\n{}", "=".repeat(80).blue().bold());
    println!(
        "{}",
        format!("Strategy Validation Report: {}", report.strategy_name)
            .blue()
            .bold()
    );
    println!("{}", "=".repeat(80).blue().bold());

    // Test period
    println!("\n{}", "Test Period".cyan().bold());
    println!(
        "  Start: {}",
        report.test_period.start.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "  End:   {}",
        report.test_period.end.format("%Y-%m-%d %H:%M:%S")
    );
    println!("  Bars:  {}", report.test_period.total_bars);

    // Overall score
    let score_color = if report.overall_score >= 80.0 {
        "green"
    } else if report.overall_score >= 60.0 {
        "yellow"
    } else {
        "red"
    };
    println!("\n{}", "Overall Score".cyan().bold());
    println!(
        "  Score: {} ({})",
        format!("{:.1}", report.overall_score)
            .color(score_color)
            .bold(),
        report.grade
    );

    let verdict_color = match report.verdict {
        alphafield_backtest::ValidationVerdict::Pass => "green",
        alphafield_backtest::ValidationVerdict::NeedsOptimization => "yellow",
        alphafield_backtest::ValidationVerdict::Fail => "red",
    };
    println!(
        "  Verdict: {}",
        format!("{:?}", report.verdict)
            .to_uppercase()
            .color(verdict_color)
            .bold()
    );

    // Backtest results
    println!("\n{}", "Backtest Results (30%)".cyan().bold());
    println!(
        "  Total Return:    {:.2}%",
        report.backtest.metrics.total_return * 100.0
    );
    println!(
        "  Sharpe Ratio:    {:.2}",
        report.backtest.metrics.sharpe_ratio
    );
    println!(
        "  Max Drawdown:    {:.2}%",
        report.backtest.metrics.max_drawdown * 100.0
    );
    println!(
        "  Win Rate:        {:.1}%",
        report.backtest.metrics.win_rate * 100.0
    );
    println!("  Profit Factor:   {:.2}", report.backtest.profit_factor);
    println!("  Total Trades:    {}", report.backtest.total_trades);

    // Walk-forward results
    println!("\n{}", "Walk-Forward Analysis (25%)".cyan().bold());
    println!(
        "  Stability Score: {:.1}%",
        report.walk_forward.stability_score * 100.0
    );
    println!(
        "  Mean Return:     {:.2}%",
        report.walk_forward.aggregate_oos.mean_return * 100.0
    );
    println!(
        "  Win Rate:        {:.1}%",
        report.walk_forward.aggregate_oos.win_rate * 100.0
    );

    // Monte Carlo results
    println!("\n{}", "Monte Carlo Simulation (20%)".cyan().bold());
    println!(
        "  Positive Prob:   {:.1}%",
        report.monte_carlo.positive_probability * 100.0
    );
    println!(
        "  5th Percentile: {:.2}%",
        report.monte_carlo.percentile_5 * 100.0
    );
    println!(
        "  50th Percentile:{:.2}%",
        report.monte_carlo.percentile_50 * 100.0
    );
    println!(
        "  95th Percentile:{:.2}%",
        report.monte_carlo.percentile_95 * 100.0
    );

    // Regime analysis
    println!("\n{}", "Regime Analysis (15%)".cyan().bold());
    println!(
        "  Regime Match:   {:.1}%",
        report.regime_analysis.calculate_regime_match_score()
    );
    if let Some(ref mismatch) = report.regime_analysis.regime_mismatch {
        println!("  {}", format!("Warning: {}", mismatch.warning).yellow());
    }

    // Risk assessment
    println!("\n{}", "Risk Assessment (10%)".cyan().bold());
    println!(
        "  Expected DD:    {:.2}%",
        report.risk_assessment.expected_max_drawdown * 100.0
    );
    println!(
        "  Actual DD:      {:.2}%",
        report.risk_assessment.actual_max_drawdown * 100.0
    );
    println!(
        "  Tail Risk:      {:.2}%",
        report.risk_assessment.tail_risk * 100.0
    );
    println!("  Rating:         {:?}", report.risk_assessment.risk_rating);

    // Recommendations
    println!("\n{}", "Recommendations".cyan().bold());
    println!("\n  Strengths:");
    for strength in &report.recommendations.strengths {
        println!("    • {}", strength.green());
    }
    println!("\n  Weaknesses:");
    for weakness in &report.recommendations.weaknesses {
        println!("    • {}", weakness.red());
    }
    println!("\n  Improvements:");
    for improvement in &report.recommendations.improvements {
        println!("    • {}", improvement.yellow());
    }

    // Deployment recommendation
    println!("\n{}", "Deployment Recommendation".cyan().bold());
    match &report.recommendations.deployment {
        alphafield_backtest::DeploymentRecommendation::Deploy { confidence } => {
            println!(
                "  Status: {} ({:.0}% confidence)",
                "DEPLOY".green().bold(),
                confidence * 100.0
            );
        }
        alphafield_backtest::DeploymentRecommendation::OptimizeThenValidate { params } => {
            println!("  Status: {}", "OPTIMIZE THEN VALIDATE".yellow().bold());
            println!("  Parameters to optimize:");
            for param in params {
                println!("    • {}", param);
            }
        }
        alphafield_backtest::DeploymentRecommendation::Reject { reason } => {
            println!("  Status: {}", "REJECT".red().bold());
            println!("  Reason: {}", reason);
        }
    }

    println!("\n{}", "=".repeat(80).blue().bold());
    println!(
        "Validated at: {}",
        report.validated_at.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("{}", "=".repeat(80).blue().bold());
    println!();
}

/// Generate Markdown report
fn generate_markdown_report(report: &ValidationReport) -> Result<String> {
    let mut md = String::new();

    md.push_str(&format!(
        "# Strategy Validation Report: {}\n\n",
        report.strategy_name
    ));
    md.push_str(&format!(
        "**Validated at:** {}\n\n",
        report.validated_at.format("%Y-%m-%d %H:%M:%S UTC")
    ));

    // Test Period
    md.push_str("## Test Period\n\n");
    md.push_str(&format!(
        "- **Start:** {}\n",
        report.test_period.start.format("%Y-%m-%d %H:%M:%S")
    ));
    md.push_str(&format!(
        "- **End:** {}\n",
        report.test_period.end.format("%Y-%m-%d %H:%M:%S")
    ));
    md.push_str(&format!(
        "- **Total Bars:** {}\n\n",
        report.test_period.total_bars
    ));

    // Overall Score
    let verdict = format!("{:?}", report.verdict).to_uppercase();
    md.push_str("## Overall Score\n\n");
    md.push_str(&format!(
        "- **Score:** {:.1} ({})\n",
        report.overall_score, report.grade
    ));
    md.push_str(&format!("- **Verdict:** {}\n\n", verdict));

    // Component Results
    md.push_str("## Component Results\n\n");

    md.push_str("### Backtest (30%)\n\n");
    md.push_str(&format!(
        "- **Total Return:** {:.2}%\n",
        report.backtest.metrics.total_return * 100.0
    ));
    md.push_str(&format!(
        "- **Sharpe Ratio:** {:.2}\n",
        report.backtest.metrics.sharpe_ratio
    ));
    md.push_str(&format!(
        "- **Max Drawdown:** {:.2}%\n",
        report.backtest.metrics.max_drawdown * 100.0
    ));
    md.push_str(&format!(
        "- **Win Rate:** {:.1}%\n",
        report.backtest.metrics.win_rate * 100.0
    ));
    md.push_str(&format!(
        "- **Profit Factor:** {:.2}\n",
        report.backtest.profit_factor
    ));
    md.push_str(&format!(
        "- **Total Trades:** {}\n\n",
        report.backtest.total_trades
    ));

    md.push_str("### Walk-Forward Analysis (25%)\n\n");
    md.push_str(&format!(
        "- **Stability Score:** {:.1}%\n",
        report.walk_forward.stability_score * 100.0
    ));
    md.push_str(&format!(
        "- **Mean Return:** {:.2}%\n",
        report.walk_forward.aggregate_oos.mean_return * 100.0
    ));
    md.push_str(&format!(
        "- **Win Rate:** {:.1}%\n\n",
        report.walk_forward.aggregate_oos.win_rate * 100.0
    ));

    md.push_str("### Monte Carlo Simulation (20%)\n\n");
    md.push_str(&format!(
        "- **Positive Probability:** {:.1}%\n",
        report.monte_carlo.positive_probability * 100.0
    ));
    md.push_str(&format!(
        "- **5th Percentile:** {:.2}%\n",
        report.monte_carlo.percentile_5 * 100.0
    ));
    md.push_str(&format!(
        "- **50th Percentile:** {:.2}%\n",
        report.monte_carlo.percentile_50 * 100.0
    ));
    md.push_str(&format!(
        "- **95th Percentile:** {:.2}%\n\n",
        report.monte_carlo.percentile_95 * 100.0
    ));

    md.push_str("### Regime Analysis (15%)\n\n");
    md.push_str(&format!(
        "- **Regime Match Score:** {:.1}%\n",
        report.regime_analysis.calculate_regime_match_score()
    ));

    md.push_str(&format!(
        "- **Expected Max Drawdown:** {:.2}%\n",
        report.risk_assessment.expected_max_drawdown * 100.0
    ));
    md.push_str(&format!(
        "- **Actual Max Drawdown:** {:.2}%\n",
        report.risk_assessment.actual_max_drawdown * 100.0
    ));
    md.push_str(&format!(
        "- **Tail Risk:** {:.2}%\n",
        report.risk_assessment.tail_risk * 100.0
    ));
    md.push_str(&format!(
        "- **Risk Rating:** {:?}\n\n",
        report.risk_assessment.risk_rating
    ));

    // Recommendations
    md.push_str("## Recommendations\n\n");

    md.push_str("### Strengths\n\n");
    for strength in &report.recommendations.strengths {
        md.push_str(&format!("- {}\n", strength));
    }
    md.push('\n');

    md.push_str("### Weaknesses\n\n");
    for weakness in &report.recommendations.weaknesses {
        md.push_str(&format!("- {}\n", weakness));
    }
    md.push('\n');

    md.push_str("### Improvements\n\n");
    for improvement in &report.recommendations.improvements {
        md.push_str(&format!("- {}\n", improvement));
    }
    md.push('\n');

    // Deployment
    md.push_str("## Deployment Recommendation\n\n");
    match &report.recommendations.deployment {
        alphafield_backtest::DeploymentRecommendation::Deploy { confidence } => {
            md.push_str(&format!(
                "**Status:** DEPLOY ({:.0}% confidence)\n\n",
                confidence * 100.0
            ));
        }
        alphafield_backtest::DeploymentRecommendation::OptimizeThenValidate { params } => {
            md.push_str("**Status:** OPTIMIZE THEN VALIDATE\n\n");
            md.push_str("**Parameters to optimize:**\n\n");
            for param in params {
                md.push_str(&format!("- {}\n", param));
            }
            md.push('\n');
        }
        alphafield_backtest::DeploymentRecommendation::Reject { reason } => {
            md.push_str(&format!("**Status:** REJECT\n\n**Reason:** {}\n\n", reason));
        }
    }

    Ok(md)
}
