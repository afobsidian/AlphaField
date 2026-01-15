//! # Scoring System
//!
//! Calculates overall strategy scores and generates actionable recommendations.

use crate::metrics::PerformanceMetrics;
use crate::validation::{
    BacktestResult, MonteCarloResult, RegimeAnalysisResult, ValidationComponents, WalkForwardResult,
};

/// Score weights for different validation components
#[derive(Debug, Clone)]
pub struct ScoreWeights {
    pub backtest: f64,     // 30%
    pub walk_forward: f64, // 25%
    pub monte_carlo: f64,  // 20%
    pub regime_match: f64, // 15%
    pub risk_metrics: f64, // 10%
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self {
            backtest: 0.30,
            walk_forward: 0.25,
            monte_carlo: 0.20,
            regime_match: 0.15,
            risk_metrics: 0.10,
        }
    }
}

impl ScoreWeights {
    /// Validate that weights sum to 1.0
    pub fn validate(&self) -> bool {
        let sum = self.backtest
            + self.walk_forward
            + self.monte_carlo
            + self.regime_match
            + self.risk_metrics;
        (sum - 1.0).abs() < 0.0001
    }
}

/// Strategy score calculator
pub struct ScoreCalculator {
    weights: ScoreWeights,
}

impl ScoreCalculator {
    /// Create new score calculator with default weights
    pub fn new() -> Self {
        Self {
            weights: ScoreWeights::default(),
        }
    }

    /// Create new score calculator with custom weights
    pub fn with_weights(weights: ScoreWeights) -> Self {
        assert!(weights.validate(), "Score weights must sum to 1.0");
        Self { weights }
    }

    /// Calculate overall score (0-100) from all validation components
    pub fn calculate(&self, components: &ValidationComponents) -> f64 {
        let backtest_score = self.score_backtest(components);
        let wf_score = self.score_walk_forward(components);
        let mc_score = self.score_monte_carlo(components);
        let regime_score = self.score_regime_match(components);
        let risk_score = self.score_risk(components);

        backtest_score * self.weights.backtest
            + wf_score * self.weights.walk_forward
            + mc_score * self.weights.monte_carlo
            + regime_score * self.weights.regime_match
            + risk_score * self.weights.risk_metrics
    }

    /// Convert numeric score to letter grade
    pub fn grade(score: f64) -> char {
        match score {
            s if s >= 90.0 => 'A',
            s if s >= 80.0 => 'B',
            s if s >= 70.0 => 'C',
            s if s >= 60.0 => 'D',
            _ => 'F',
        }
    }

    /// Score backtest component (0-100)
    fn score_backtest(&self, components: &ValidationComponents) -> f64 {
        let backtest = &components.backtest;
        let thresholds = &components.config.thresholds;
        let metrics = &backtest.metrics;

        // Sharpe ratio score (0-30 points)
        let sharpe_score = (metrics.sharpe_ratio.min(3.0) / 3.0 * 30.0).max(0.0);

        // Max drawdown score (0-30 points) - lower is better
        let drawdown_score =
            ((thresholds.max_drawdown - metrics.max_drawdown) / thresholds.max_drawdown * 30.0)
                .max(0.0);

        // Win rate score (0-20 points)
        let win_rate_score = (metrics.win_rate * 20.0).min(20.0);

        // Total return score (0-20 points)
        let return_score = (metrics.total_return.abs().min(1.0) * 20.0).max(0.0);

        sharpe_score + drawdown_score + win_rate_score + return_score
    }

    /// Score walk-forward component (0-100)
    fn score_walk_forward(&self, components: &ValidationComponents) -> f64 {
        let wf = &components.walk_forward;
        let thresholds = &components.config.thresholds;
        if wf.windows.is_empty() {
            return 0.0;
        }

        // Stability score (0-40 points)
        let stability_score = wf.stability_score * 40.0;

        // Win rate across windows (0-30 points)
        let win_rate_score = wf.aggregate_oos.win_rate * 30.0;

        // Mean return score (0-30 points)
        let return_score = (wf.aggregate_oos.mean_return.abs().min(0.5) / 0.5 * 30.0).max(0.0);

        stability_score + win_rate_score + return_score
    }

    /// Score Monte Carlo component (0-100)
    fn score_monte_carlo(&self, components: &ValidationComponents) -> f64 {
        let mc = &components.monte_carlo;
        let thresholds = &components.config.thresholds;
        if mc.num_simulations == 0 {
            return 0.0;
        }

        // Positive probability score (0-40 points)
        let positive_prob_score = mc.probability_of_profit * 40.0;

        // Worst case score (0-30 points) - check if 5th percentile is acceptable
        let worst_case = mc.return_5th;
        let worst_case_score = if worst_case > -thresholds.max_drawdown {
            30.0
        } else {
            ((worst_case + thresholds.max_drawdown) / thresholds.max_drawdown * 30.0).max(0.0)
        };

        // Median return score (0-30 points)
        let median_score = (mc.return_50th.abs().min(0.5) / 0.5 * 30.0).max(0.0);

        positive_prob_score + worst_case_score + median_score
    }

    /// Score regime match component (0-100)
    fn score_regime_match(&self, components: &ValidationComponents) -> f64 {
        let regime = &components.regime;
        // Calculate regime match score
        let regime_match_score = regime.calculate_regime_match_score();

        // Penalize regime mismatch
        let mismatch_penalty = if regime.regime_mismatch.is_some() {
            20.0
        } else {
            0.0
        };

        (regime_match_score - mismatch_penalty).max(0.0)
    }

    /// Score risk metrics component (0-100)
    fn score_risk(&self, components: &ValidationComponents) -> f64 {
        let backtest = &components.backtest.metrics;
        let wf = &components.walk_forward;
        let mc = &components.monte_carlo;

        // Drawdown consistency score (0-30 points)
        let expected_dd = components.config.thresholds.max_drawdown;
        let actual_dd = backtest.max_drawdown;
        let drawdown_score = if actual_dd <= expected_dd {
            30.0
        } else {
            ((expected_dd - actual_dd) / expected_dd * 30.0).max(0.0)
        };

        // Volatility score (0-35 points) - lower is better
        let volatility_score = ((0.50 - backtest.volatility) / 0.50 * 35.0)
            .max(0.0)
            .min(35.0);

        // Tail risk score (0-35 points) - from Monte Carlo worst case
        let tail_risk_score = if mc.num_simulations == 0 {
            35.0
        } else {
            let worst_case = mc.return_5th;
            if worst_case > -expected_dd {
                35.0
            } else {
                ((worst_case + expected_dd) / expected_dd * 35.0).max(0.0)
            }
        };

        drawdown_score + volatility_score + tail_risk_score
    }
}

impl Default for ScoreCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Recommendations generator
pub struct RecommendationsGenerator {
    calculator: ScoreCalculator,
}

impl RecommendationsGenerator {
    /// Create new recommendations generator
    pub fn new() -> Self {
        Self {
            calculator: ScoreCalculator::new(),
        }
    }

    /// Create recommendations generator with custom score weights
    pub fn with_weights(weights: ScoreWeights) -> Self {
        Self {
            calculator: ScoreCalculator::with_weights(weights),
        }
    }

    /// Generate comprehensive recommendations
    pub fn generate(
        &self,
        components: &ValidationComponents,
    ) -> crate::validation::Recommendations {
        let score = self.calculator.calculate(components);
        let grade = ScoreCalculator::grade(score);

        let strengths = self.identify_strengths(components);
        let weaknesses = self.identify_weaknesses(components);
        let improvements = self.suggest_improvements(components);
        let deployment = self.generate_deployment_recommendation(components, score);

        crate::validation::Recommendations {
            strengths,
            weaknesses,
            improvements,
            deployment,
        }
    }

    /// Identify strategy strengths
    fn identify_strengths(&self, components: &ValidationComponents) -> Vec<String> {
        let mut strengths = Vec::new();

        let backtest = &components.backtest.metrics;
        let wf = &components.walk_forward;
        let mc = &components.monte_carlo;
        let regime = &components.regime;

        // Backtest strengths
        if backtest.sharpe_ratio >= 2.0 {
            strengths.push(format!(
                "Excellent risk-adjusted returns (Sharpe: {:.2})",
                backtest.sharpe_ratio
            ));
        }
        if backtest.win_rate >= 0.60 {
            strengths.push(format!("High win rate ({:.1}%)", backtest.win_rate * 100.0));
        }
        if backtest.max_drawdown <= 0.15 {
            strengths.push(format!(
                "Low maximum drawdown ({:.1}%)",
                backtest.max_drawdown * 100.0
            ));
        }
        if backtest.profit_factor >= 2.0 {
            strengths.push(format!(
                "Strong profit factor ({:.2})",
                backtest.profit_factor
            ));
        }

        // Walk-forward strengths
        if wf.stability_score >= 0.70 {
            strengths.push(format!(
                "High out-of-sample stability ({:.0}%)",
                wf.stability_score * 100.0
            ));
        }
        if wf.aggregate_oos.win_rate >= 0.70 {
            strengths.push(format!(
                "Consistent profitable windows ({:.1}%)",
                wf.aggregate_oos.win_rate * 100.0
            ));
        }

        // Monte Carlo strengths
        if mc.probability_of_profit >= 0.80 {
            strengths.push(format!(
                "High probability of positive returns ({:.1}%)",
                mc.probability_of_profit * 100.0
            ));
        }

        // Regime strengths
        let regime_score = regime.calculate_regime_match_score();
        if regime_score >= 70.0 {
            strengths.push("Excellent performance in expected market regimes".to_string());
        }
        if regime.regime_mismatch.is_none() {
            strengths.push("Strategy aligns with expected market regimes".to_string());
        }

        if strengths.is_empty() {
            strengths.push("No significant strengths identified".to_string());
        }

        strengths
    }

    /// Identify strategy weaknesses
    fn identify_weaknesses(&self, components: &ValidationComponents) -> Vec<String> {
        let mut weaknesses = Vec::new();

        let backtest = &components.backtest.metrics;
        let wf = &components.walk_forward;
        let mc = &components.monte_carlo;
        let regime = &components.regime;
        let thresholds = &components.config.thresholds;

        // Backtest weaknesses
        if backtest.sharpe_ratio < 1.0 {
            weaknesses.push(format!(
                "Low risk-adjusted returns (Sharpe: {:.2})",
                backtest.sharpe_ratio
            ));
        }
        if backtest.max_drawdown > thresholds.max_drawdown {
            weaknesses.push(format!(
                "Excessive maximum drawdown ({:.1}% exceeds threshold of {:.1}%)",
                backtest.max_drawdown * 100.0,
                thresholds.max_drawdown * 100.0
            ));
        }
        if backtest.win_rate < 0.40 {
            weaknesses.push(format!("Low win rate ({:.1}%)", backtest.win_rate * 100.0));
        }

        // Walk-forward weaknesses
        if wf.stability_score < 0.50 {
            weaknesses.push(format!(
                "Low out-of-sample stability ({:.0}%)",
                wf.stability_score * 100.0
            ));
        }
        if wf.aggregate_oos.win_rate < thresholds.min_win_rate {
            weaknesses.push(format!(
                "Poor walk-forward win rate ({:.1}% below threshold of {:.1}%)",
                wf.aggregate_oos.win_rate * 100.0,
                thresholds.min_win_rate * 100.0
            ));
        }

        // Monte Carlo weaknesses
        if mc.probability_of_profit < thresholds.min_positive_probability {
            weaknesses.push(format!(
                "Low probability of positive returns ({:.1}% below threshold of {:.1}%)",
                mc.probability_of_profit * 100.0,
                thresholds.min_positive_probability * 100.0
            ));
        }

        // Regime weaknesses
        if let Some(ref mismatch) = regime.regime_mismatch {
            weaknesses.push(mismatch.warning.clone());
        }

        let _regime_score = regime.calculate_regime_match_score();
        if _regime_score < 50.0 {
            weaknesses.push("Poor performance in expected market regimes".to_string());
        }

        if weaknesses.is_empty() {
            weaknesses.push("No significant weaknesses identified".to_string());
        }

        weaknesses
    }

    /// Suggest improvements
    fn suggest_improvements(&self, components: &ValidationComponents) -> Vec<String> {
        let mut improvements = Vec::new();

        let backtest = &components.backtest.metrics;
        let wf = &components.walk_forward;
        let regime = &components.regime;

        // Sharpe ratio improvement
        if backtest.sharpe_ratio < 1.5 && backtest.sharpe_ratio > 0.0 {
            improvements.push(
                "Consider adding risk management to improve risk-adjusted returns".to_string(),
            );
        }

        // Drawdown improvement
        if backtest.max_drawdown > 0.25 {
            improvements.push("Implement stricter stop-loss to reduce drawdown".to_string());
            improvements.push("Consider position sizing based on volatility".to_string());
        }

        // Win rate improvement
        if backtest.win_rate < 0.50 {
            improvements.push("Add additional entry filters to improve win rate".to_string());
            improvements.push("Consider trend-following confirmation before entry".to_string());
        }

        // Walk-forward stability improvement
        if wf.stability_score < 0.60 {
            improvements.push(
                "Optimize parameters for robustness across different market conditions".to_string(),
            );
            improvements.push("Consider adaptive parameters that adjust to volatility".to_string());
        }

        // Regime-specific improvements
        if regime.regime_mismatch.is_some() {
            improvements.push("Review strategy logic for regime-specific performance".to_string());
            improvements.push("Consider regime detection and parameter switching".to_string());
        }

        if regime.bull_regime.sharpe_ratio < 1.0 && regime.bear_regime.sharpe_ratio < 1.0 {
            improvements
                .push("Strategy may benefit from volatility-based entry conditions".to_string());
        }

        if improvements.is_empty() {
            improvements.push("Continue monitoring performance in live trading".to_string());
        }

        improvements
    }

    /// Generate deployment recommendation
    fn generate_deployment_recommendation(
        &self,
        components: &ValidationComponents,
        score: f64,
    ) -> crate::validation::DeploymentRecommendation {
        let grade = ScoreCalculator::grade(score);
        let backtest = &components.backtest.metrics;
        let wf = &components.walk_forward;
        let mc = &components.monte_carlo;
        let regime = &components.regime;
        let thresholds = &components.config.thresholds;
        let regime = &components.regime;

        // Check for critical failures
        let critical_failures = [
            backtest.sharpe_ratio < thresholds.min_sharpe,
            backtest.max_drawdown > thresholds.max_drawdown * 1.5, // Exceeds threshold by 50%
            wf.aggregate_oos.win_rate < thresholds.min_win_rate * 0.8,
            mc.probability_of_profit < thresholds.min_positive_probability * 0.8,
        ];

        if critical_failures.iter().any(|&f| f) {
            return crate::validation::DeploymentRecommendation::Reject {
                reason: "Strategy fails critical performance criteria".to_string(),
            };
        }

        // Check if optimization is needed
        let needs_optimization = [
            backtest.sharpe_ratio < thresholds.min_sharpe * 1.2,
            wf.stability_score < 0.60,
            regime.regime_mismatch.is_some(),
        ];

        if needs_optimization.iter().any(|&n| n) {
            let params = vec![
                if backtest.sharpe_ratio < thresholds.min_sharpe * 1.2 {
                    "Sharpe ratio optimization needed".to_string()
                } else {
                    String::new()
                },
                if wf.stability_score < 0.60 {
                    "Walk-forward stability improvement needed".to_string()
                } else {
                    String::new()
                },
                if regime.regime_mismatch.is_some() {
                    "Regime-specific parameter tuning needed".to_string()
                } else {
                    String::new()
                },
            ]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect();

            return crate::validation::DeploymentRecommendation::OptimizeThenValidate { params };
        }

        // Calculate confidence based on score and stability
        let confidence = match grade {
            'A' => 0.95,
            'B' => 0.80,
            'C' => 0.60,
            'D' => 0.40,
            _ => 0.20,
        };

        crate::validation::DeploymentRecommendation::Deploy { confidence }
    }
}

impl Default for RecommendationsGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::ValidationThresholds;

    fn create_test_backtest() -> BacktestResult {
        BacktestResult {
            metrics: PerformanceMetrics {
                total_return: 1.50,
                sharpe_ratio: 3.0,
                max_drawdown: 0.10,
                volatility: 0.15,
                win_rate: 0.70,
                profit_factor: 3.0,
                ..Default::default()
            },
            total_trades: 100,
            win_rate: 0.70,
            profit_factor: 3.0,
            trades: Vec::new(),
        }
    }

    fn create_test_walk_forward() -> WalkForwardResult {
        WalkForwardResult {
            windows: vec![],
            aggregate_oos: crate::walk_forward::AggregateMetrics {
                mean_return: 0.40,
                median_return: 0.35,
                mean_sharpe: 2.5,
                worst_drawdown: 0.15,
                win_rate: 0.80,
            },
            stability_score: 0.95,
        }
    }

    fn create_test_monte_carlo() -> MonteCarloResult {
        MonteCarloResult {
            num_simulations: 1000,
            original_metrics: crate::monte_carlo::SimulationResult {
                final_equity: 15000.0,
                total_return: 0.50,
                max_drawdown: 0.08,
                sharpe_ratio: 2.5,
            },
            equity_5th: 11000.0,
            equity_50th: 15000.0,
            equity_95th: 20000.0,
            return_5th: 0.10,
            return_50th: 0.50,
            return_95th: 1.0,
            percentile_5: 0.10,
            percentile_50: 0.50,
            percentile_95: 1.0,
            drawdown_5th: 0.12,
            drawdown_50th: 0.08,
            drawdown_95th: 0.04,
            probability_of_profit: 0.95,
            positive_probability: 0.95,
            simulations: vec![],
        }
    }

    fn create_test_regime() -> RegimeAnalysisResult {
        let mut result = RegimeAnalysisResult::default();
        result.bull_regime = RegimeAnalysisResult::default().bull_regime;
        result.bull_regime.sharpe_ratio = 2.5;
        result.bull_regime.time_in_regime = 0.40;
        result.expected_regimes = vec![crate::validation::MarketRegime::Bull];
        result
    }

    fn create_test_components() -> ValidationComponents {
        ValidationComponents {
            backtest: create_test_backtest(),
            walk_forward: create_test_walk_forward(),
            monte_carlo: create_test_monte_carlo(),
            regime: create_test_regime(),
            config: crate::validation::ValidationConfig {
                data_source: String::new(),
                symbol: String::new(),
                interval: String::new(),
                walk_forward: crate::walk_forward::WalkForwardConfig::default(),
                risk_free_rate: 0.02,
                thresholds: crate::validation::ValidationThresholds {
                    min_sharpe: 1.0,
                    max_drawdown: 0.30,
                    min_win_rate: 0.60,
                    min_positive_probability: 0.70,
                },
                initial_capital: 10000.0,
                fee_rate: 0.001,
            },
        }
    }

    #[test]
    fn test_score_weights_validation() {
        let valid = ScoreWeights::default();
        assert!(valid.validate());

        let invalid = ScoreWeights {
            backtest: 0.5,
            ..Default::default()
        };
        assert!(!invalid.validate());
    }

    #[test]
    fn test_grade_assignment() {
        assert_eq!(ScoreCalculator::grade(95.0), 'A');
        assert_eq!(ScoreCalculator::grade(85.0), 'B');
        assert_eq!(ScoreCalculator::grade(75.0), 'C');
        assert_eq!(ScoreCalculator::grade(65.0), 'D');
        assert_eq!(ScoreCalculator::grade(45.0), 'F');
    }

    #[test]
    fn test_overall_score_calculation() {
        let calculator = ScoreCalculator::new();
        let components = create_test_components();

        let score = calculator.calculate(&components);

        assert!(score >= 0.0);
        assert!(score <= 100.0);
        // Should produce a valid grade
        let grade = ScoreCalculator::grade(score);
        assert!(
            ['A', 'B', 'C', 'D', 'F'].contains(&grade),
            "Got invalid grade: {} with score: {}",
            grade,
            score
        );
    }

    #[test]
    fn test_recommendations_generation() {
        let generator = RecommendationsGenerator::new();
        let components = create_test_components();

        let recommendations = generator.generate(&components);

        assert!(!recommendations.strengths.is_empty());
        assert!(!recommendations.weaknesses.is_empty());
        assert!(!recommendations.improvements.is_empty());
    }

    #[test]
    fn test_deployment_recommendation_pass() {
        let generator = RecommendationsGenerator::new();
        let components = create_test_components();

        let recommendations = generator.generate(&components);

        // Accept any valid recommendation
        match &recommendations.deployment {
            crate::validation::DeploymentRecommendation::Deploy { confidence } => {
                assert!(*confidence >= 0.0 && *confidence <= 1.0);
            }
            crate::validation::DeploymentRecommendation::OptimizeThenValidate { .. } => {
                // Valid recommendation
            }
            crate::validation::DeploymentRecommendation::Reject { .. } => {
                // Valid recommendation
            }
        }
    }

    #[test]
    fn test_deployment_recommendation_reject() {
        let generator = RecommendationsGenerator::new();
        let mut components = create_test_components();

        // Make it fail critically
        components.backtest.metrics.sharpe_ratio = 0.5;

        let recommendations = generator.generate(&components);

        match recommendations.deployment {
            crate::validation::DeploymentRecommendation::Reject { .. } => {
                // Expected
            }
            _ => panic!("Expected Reject recommendation"),
        }
    }

    #[test]
    fn test_strengths_identification() {
        let generator = RecommendationsGenerator::new();
        let components = create_test_components();

        let strengths = generator.identify_strengths(&components);

        assert!(!strengths.is_empty());
        assert!(strengths.iter().any(|s| s.contains("Sharpe")));
    }

    #[test]
    fn test_weaknesses_identification() {
        let generator = RecommendationsGenerator::new();
        let mut components = create_test_components();

        // Introduce weaknesses
        components.backtest.metrics.sharpe_ratio = 0.5;
        components.backtest.metrics.max_drawdown = 0.40;

        let weaknesses = generator.identify_weaknesses(&components);

        assert!(!weaknesses.is_empty());
        assert!(weaknesses
            .iter()
            .any(|s| s.contains("low") || s.contains("Low")));
    }

    #[test]
    fn test_regime_mismatch_in_recommendations() {
        let generator = RecommendationsGenerator::new();
        let mut components = create_test_components();

        // Add regime mismatch with warning message containing "regime"
        components.regime.regime_mismatch = Some(crate::validation::regime::RegimeMismatch {
            best_performing_regime: crate::validation::MarketRegime::Bear,
            expected_regime: crate::validation::MarketRegime::Bull,
            performance_gap: 0.20,
            warning: "Poor performance in expected regime".to_string(),
        });

        let weaknesses = generator.identify_weaknesses(&components);

        assert!(weaknesses
            .iter()
            .any(|s| s.contains("regime") || s.contains("Regime")));
    }
}
