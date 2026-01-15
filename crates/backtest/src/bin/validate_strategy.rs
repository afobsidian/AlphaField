//! # Strategy Validation CLI Tool
//!
//! Command-line tool for validating trading strategies without requiring
//! full dashboard integration.

use alphafield_backtest::{
    Strategy, StrategyAdapter, StrategyValidator, ValidationConfig, ValidationReport,
    ValidationThresholds, WalkForwardConfig,
};
use alphafield_core::Bar;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use colored::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

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

        /// Data file path (CSV or JSON)
        #[arg(long)]
        data_file: String,

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

        /// Data directory (contains symbol-specific data files)
        #[arg(long)]
        data_dir: String,

        /// Output directory for reports
        #[arg(long)]
        output_dir: String,

        /// Output format (json, yaml, markdown)
        #[arg(long, default_value = "json")]
        format: OutputFormat,
    },
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
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate {
            strategy,
            symbol,
            interval,
            data_file,
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
            let bars = load_bars_from_file(&data_file)?;
            let config = ValidationConfig {
                data_source: data_file.clone(),
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

            // Create strategy based on name
            let strategy = create_strategy(&strategy)?;

            let validator = StrategyValidator::new(config);
            let report = validator
                .validate(strategy, &symbol, &bars)
                .context("Validation failed")?;

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
            data_dir,
            output_dir,
            format,
        } => {
            let strategies = fs::read_to_string(&batch_file)
                .context(format!("Failed to read batch file: {}", batch_file))?;

            let symbols_list: Vec<&str> = symbols.split(',').map(|s| s.trim()).collect();
            let strategy_names: Vec<&str> = strategies
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.trim())
                .collect();

            // Create output directory if it doesn't exist
            fs::create_dir_all(&output_dir)
                .context(format!("Failed to create output directory: {}", output_dir))?;

            let mut total_validations = 0;
            let mut passed_validations = 0;
            let mut failed_validations = 0;

            println!("{}", "Starting batch validation...".blue().bold());
            println!(
                "Strategies: {}, Symbols: {}\n",
                strategy_names.len(),
                symbols_list.len()
            );

            for strategy_name in strategy_names {
                for symbol in &symbols_list {
                    let data_file = format!("{}/{}.csv", data_dir, symbol);
                    if !Path::new(&data_file).exists() {
                        println!(
                            "{}",
                            format!(
                                "Skipping {}/{} - data file not found: {}",
                                strategy_name, symbol, data_file
                            )
                            .yellow()
                        );
                        continue;
                    }

                    total_validations += 1;
                    println!("Validating {}/{}...", strategy_name, symbol);

                    let bars = match load_bars_from_file(&data_file) {
                        Ok(b) => b,
                        Err(e) => {
                            println!(
                                "{}",
                                format!(
                                    "Failed to load data for {}/{}: {}",
                                    strategy_name, symbol, e
                                )
                                .red()
                            );
                            failed_validations += 1;
                            continue;
                        }
                    };

                    let config = ValidationConfig {
                        data_source: data_file.clone(),
                        symbol: symbol.to_string(),
                        interval: interval.clone(),
                        walk_forward: WalkForwardConfig::default(),
                        risk_free_rate: 0.02,
                        thresholds: ValidationThresholds::default(),
                        initial_capital: 10000.0,
                        fee_rate: 0.001,
                    };

                    let strategy = match create_strategy(strategy_name) {
                        Ok(s) => s,
                        Err(e) => {
                            println!(
                                "{}",
                                format!("Failed to create strategy {}: {}", strategy_name, e).red()
                            );
                            failed_validations += 1;
                            continue;
                        }
                    };

                    let validator = StrategyValidator::new(config);
                    let report = match validator.validate(strategy, symbol, &bars) {
                        Ok(r) => r,
                        Err(e) => {
                            println!(
                                "{}",
                                format!(
                                    "Validation failed for {}/{}: {}",
                                    strategy_name, symbol, e
                                )
                                .red()
                            );
                            failed_validations += 1;
                            continue;
                        }
                    };

                    // Save report
                    let filename =
                        format!("{}/{}_{}.{}", output_dir, strategy_name, symbol, format);
                    let report_content = match format {
                        OutputFormat::Json => serde_json::to_string_pretty(&report)?,
                        OutputFormat::Yaml => serde_yaml::to_string(&report)?,
                        OutputFormat::Markdown => generate_markdown_report(&report)?,
                        OutputFormat::Terminal => {
                            // For terminal format, use JSON for batch
                            serde_json::to_string_pretty(&report)?
                        }
                    };

                    fs::write(&filename, report_content)
                        .context(format!("Failed to write report: {}", filename))?;

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
                        alphafield_backtest::ValidationVerdict::Pass => passed_validations += 1,
                        _ => failed_validations += 1,
                    }
                }
            }

            // Print summary
            println!("\n{}", "Batch Validation Summary".blue().bold());
            println!("Total validations: {}", total_validations);
            println!("{}", format!("Passed: {}", passed_validations).green());
            println!("{}", format!("Failed: {}", failed_validations).red());

            std::process::exit(if failed_validations > 0 { 1 } else { 0 });
        }
    }
}

/// Load bars from CSV or JSON file
fn load_bars_from_file(path: &str) -> Result<Vec<Bar>> {
    let content =
        fs::read_to_string(path).context(format!("Failed to read data file: {}", path))?;

    if path.ends_with(".json") {
        let bars: Vec<Bar> =
            serde_json::from_str(&content).context("Failed to parse JSON data file")?;
        Ok(bars)
    } else {
        // Assume CSV format
        parse_csv_bars(&content)
    }
}

/// Parse bars from CSV format
fn parse_csv_bars(content: &str) -> Result<Vec<Bar>> {
    let mut bars = Vec::new();

    for line in content.lines().skip(1) {
        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 6 {
            continue;
        }

        let timestamp = parts[0]
            .parse::<i64>()
            .ok()
            .and_then(|ts| DateTime::from_timestamp(ts, 0))
            .unwrap_or_else(|| Utc::now());

        let bar = Bar {
            timestamp,
            open: parts[1]
                .parse::<f64>()
                .context(format!("Failed to parse open price: {}", parts[1]))?,
            high: parts[2]
                .parse::<f64>()
                .context(format!("Failed to parse high price: {}", parts[2]))?,
            low: parts[3]
                .parse::<f64>()
                .context(format!("Failed to parse low price: {}", parts[3]))?,
            close: parts[4]
                .parse::<f64>()
                .context(format!("Failed to parse close price: {}", parts[4]))?,
            volume: parts[5]
                .parse::<f64>()
                .context(format!("Failed to parse volume: {}", parts[5]))?,
        };

        bars.push(bar);
    }

    Ok(bars)
}

/// Create strategy from name
fn create_strategy(name: &str) -> Result<Box<dyn Strategy>> {
    use alphafield_strategy::{
        BollingerBandsStrategy, GoldenCrossStrategy, MacdTrendStrategy, RsiStrategy,
    };

    const DEFAULT_CAPITAL: f64 = 10000.0;

    match name.to_lowercase().as_str() {
        "golden_cross" | "goldencross" => Ok(Box::new(StrategyAdapter::new(
            GoldenCrossStrategy::default(),
            "BTCUSDT",
            DEFAULT_CAPITAL,
        ))),
        "rsi" | "rsi_strategy" => Ok(Box::new(StrategyAdapter::new(
            RsiStrategy::default(),
            "BTCUSDT",
            DEFAULT_CAPITAL,
        ))),
        "macd" | "macd_strategy" => Ok(Box::new(StrategyAdapter::new(
                    MacdTrendStrategy::default(),
                    "BTCUSDT",
                    DEFAULT_CAPITAL,
                ))),
        "bollinger_bands" | "bollingerbands" | "bb" => Ok(Box::new(
            StrategyAdapter::new(
                BollingerBandsStrategy::default(),
                "BTCUSDT",
                DEFAULT_CAPITAL,
            ),
        )),
        _ => Err(anyhow::anyhow!(
            "Unknown strategy: {}. Available strategies: golden_cross, rsi_strategy, macd_strategy, bollinger_bands",
            name
        )),
    }
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
    if let Some(ref mismatch) = report.regime_analysis.regime_mismatch {
        md.push_str(&format!("- **Warning:** {}\n\n", mismatch.warning));
    } else {
        md.push_str("\n");
    }

    md.push_str("### Risk Assessment (10%)\n\n");
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
    md.push_str("\n");

    md.push_str("### Weaknesses\n\n");
    for weakness in &report.recommendations.weaknesses {
        md.push_str(&format!("- {}\n", weakness));
    }
    md.push_str("\n");

    md.push_str("### Improvements\n\n");
    for improvement in &report.recommendations.improvements {
        md.push_str(&format!("- {}\n", improvement));
    }
    md.push_str("\n");

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
            md.push_str("\n");
        }
        alphafield_backtest::DeploymentRecommendation::Reject { reason } => {
            md.push_str(&format!("**Status:** REJECT\n\n**Reason:** {}\n\n", reason));
        }
    }

    Ok(md)
}
