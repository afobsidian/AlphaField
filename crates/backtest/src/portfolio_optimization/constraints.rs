//! Portfolio Constraints
//!
//! Defines various constraints that can be applied during portfolio optimization,
//! such as minimum/maximum weights, sector limits, and sum-to-one constraints.

use serde::{Deserialize, Serialize};

/// A constraint on portfolio weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeightConstraint {
    /// Minimum weight for a strategy (0.0 to 1.0)
    MinWeight { strategy: String, min: f64 },
    /// Maximum weight for a strategy (0.0 to 1.0)
    MaxWeight { strategy: String, max: f64 },
    /// Exact weight for a strategy (for benchmarking)
    ExactWeight { strategy: String, weight: f64 },
    /// Minimum total weight for a group of strategies
    GroupMinWeight { strategies: Vec<String>, min: f64 },
    /// Maximum total weight for a group of strategies
    GroupMaxWeight { strategies: Vec<String>, max: f64 },
}

impl WeightConstraint {
    /// Check if a set of allocations satisfies this constraint
    pub fn check(&self, allocations: &std::collections::HashMap<String, f64>) -> bool {
        match self {
            WeightConstraint::MinWeight { strategy, min } => {
                allocations.get(strategy).copied().unwrap_or(0.0) >= *min
            }
            WeightConstraint::MaxWeight { strategy, max } => {
                allocations.get(strategy).copied().unwrap_or(0.0) <= *max
            }
            WeightConstraint::ExactWeight { strategy, weight } => {
                let actual = allocations.get(strategy).copied().unwrap_or(0.0);
                (actual - weight).abs() < 1e-6
            }
            WeightConstraint::GroupMinWeight { strategies, min } => {
                let total: f64 = strategies
                    .iter()
                    .map(|s| allocations.get(s).copied().unwrap_or(0.0))
                    .sum();
                total >= *min
            }
            WeightConstraint::GroupMaxWeight { strategies, max } => {
                let total: f64 = strategies
                    .iter()
                    .map(|s| allocations.get(s).copied().unwrap_or(0.0))
                    .sum();
                total <= *max
            }
        }
    }

    /// Get a human-readable description of this constraint
    pub fn description(&self) -> String {
        match self {
            WeightConstraint::MinWeight { strategy, min } => {
                format!("{} must have minimum weight {:.2}%", strategy, min * 100.0)
            }
            WeightConstraint::MaxWeight { strategy, max } => {
                format!("{} must have maximum weight {:.2}%", strategy, max * 100.0)
            }
            WeightConstraint::ExactWeight { strategy, weight } => {
                format!("{} must have exact weight {:.2}%", strategy, weight * 100.0)
            }
            WeightConstraint::GroupMinWeight { strategies, min } => {
                format!(
                    "Group [{}] must have minimum total weight {:.2}%",
                    strategies.join(", "),
                    min * 100.0
                )
            }
            WeightConstraint::GroupMaxWeight { strategies, max } => {
                format!(
                    "Group [{}] must have maximum total weight {:.2}%",
                    strategies.join(", "),
                    max * 100.0
                )
            }
        }
    }
}

/// A comprehensive portfolio constraint set
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PortfolioConstraint {
    /// Individual weight constraints
    pub weight_constraints: Vec<WeightConstraint>,
    /// Minimum number of strategies to include (non-zero weight)
    pub min_strategies: Option<usize>,
    /// Maximum number of strategies to include (non-zero weight)
    pub max_strategies: Option<usize>,
    /// Tolerance for sum-to-one constraint
    pub sum_tolerance: f64,
    /// Allow short positions (negative weights)
    pub allow_short: bool,
    /// Maximum leverage (sum of absolute weights)
    pub max_leverage: f64,
}

impl PortfolioConstraint {
    /// Create a new constraint set with default values
    pub fn new() -> Self {
        Self {
            weight_constraints: Vec::new(),
            min_strategies: None,
            max_strategies: None,
            sum_tolerance: 0.0001,
            allow_short: false,
            max_leverage: 1.0,
        }
    }

    /// Add a minimum weight constraint
    pub fn with_min_weight(mut self, strategy: impl Into<String>, min: f64) -> Self {
        self.weight_constraints.push(WeightConstraint::MinWeight {
            strategy: strategy.into(),
            min: min.clamp(0.0, 1.0),
        });
        self
    }

    /// Add a maximum weight constraint
    pub fn with_max_weight(mut self, strategy: impl Into<String>, max: f64) -> Self {
        self.weight_constraints.push(WeightConstraint::MaxWeight {
            strategy: strategy.into(),
            max: max.clamp(0.0, 1.0),
        });
        self
    }

    /// Add an exact weight constraint
    pub fn with_exact_weight(mut self, strategy: impl Into<String>, weight: f64) -> Self {
        self.weight_constraints.push(WeightConstraint::ExactWeight {
            strategy: strategy.into(),
            weight: weight.clamp(0.0, 1.0),
        });
        self
    }

    /// Add a group minimum weight constraint
    pub fn with_group_min_weight(mut self, strategies: Vec<impl Into<String>>, min: f64) -> Self {
        self.weight_constraints
            .push(WeightConstraint::GroupMinWeight {
                strategies: strategies.into_iter().map(|s| s.into()).collect(),
                min: min.clamp(0.0, 1.0),
            });
        self
    }

    /// Add a group maximum weight constraint
    pub fn with_group_max_weight(mut self, strategies: Vec<impl Into<String>>, max: f64) -> Self {
        self.weight_constraints
            .push(WeightConstraint::GroupMaxWeight {
                strategies: strategies.into_iter().map(|s| s.into()).collect(),
                max: max.clamp(0.0, 1.0),
            });
        self
    }

    /// Set minimum number of strategies
    pub fn with_min_strategies(mut self, min: usize) -> Self {
        self.min_strategies = Some(min);
        self
    }

    /// Set maximum number of strategies
    pub fn with_max_strategies(mut self, max: usize) -> Self {
        self.max_strategies = Some(max);
        self
    }

    /// Allow short positions
    pub fn with_short_positions(mut self, allow: bool) -> Self {
        self.allow_short = allow;
        self
    }

    /// Set maximum leverage
    pub fn with_max_leverage(mut self, leverage: f64) -> Self {
        self.max_leverage = leverage.clamp(1.0, 10.0);
        self
    }

    /// Check if allocations satisfy all constraints
    pub fn check(
        &self,
        allocations: &std::collections::HashMap<String, f64>,
    ) -> Result<(), String> {
        // Check sum-to-one constraint
        let total: f64 = allocations.values().sum();
        if (total - 1.0).abs() > self.sum_tolerance {
            return Err(format!(
                "Weights must sum to 1.0 (got {:.6}, tolerance: {:.6})",
                total, self.sum_tolerance
            ));
        }

        // Check leverage constraint
        let leverage: f64 = allocations.values().map(|w| w.abs()).sum();
        if leverage > self.max_leverage + self.sum_tolerance {
            return Err(format!(
                "Leverage exceeds maximum: {:.4} > {:.4}",
                leverage, self.max_leverage
            ));
        }

        // Check short constraint
        if !self.allow_short {
            for (strategy, weight) in allocations {
                if *weight < 0.0 {
                    return Err(format!(
                        "Short positions not allowed: {} has weight {:.4}",
                        strategy, weight
                    ));
                }
            }
        }

        // Check min/max strategies
        let num_strategies = allocations.values().filter(|w| **w > 1e-6).count();
        if let Some(min) = self.min_strategies {
            if num_strategies < min {
                return Err(format!(
                    "Need at least {} strategies, got {}",
                    min, num_strategies
                ));
            }
        }
        if let Some(max) = self.max_strategies {
            if num_strategies > max {
                return Err(format!(
                    "Can have at most {} strategies, got {}",
                    max, num_strategies
                ));
            }
        }

        // Check individual weight constraints
        for constraint in &self.weight_constraints {
            if !constraint.check(allocations) {
                return Err(format!("Constraint violated: {}", constraint.description()));
            }
        }

        Ok(())
    }

    /// Get a list of all strategy names mentioned in constraints
    pub fn constrained_strategies(&self) -> Vec<String> {
        let mut strategies = std::collections::HashSet::new();

        for constraint in &self.weight_constraints {
            match constraint {
                WeightConstraint::MinWeight { strategy, .. } => {
                    strategies.insert(strategy.clone());
                }
                WeightConstraint::MaxWeight { strategy, .. } => {
                    strategies.insert(strategy.clone());
                }
                WeightConstraint::ExactWeight { strategy, .. } => {
                    strategies.insert(strategy.clone());
                }
                WeightConstraint::GroupMinWeight { strategies: s, .. } => {
                    for strat in s {
                        strategies.insert(strat.clone());
                    }
                }
                WeightConstraint::GroupMaxWeight { strategies: s, .. } => {
                    for strat in s {
                        strategies.insert(strat.clone());
                    }
                }
            }
        }

        strategies.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_allocations() -> HashMap<String, f64> {
        let mut allocations = HashMap::new();
        allocations.insert("A".to_string(), 0.4);
        allocations.insert("B".to_string(), 0.4);
        allocations.insert("C".to_string(), 0.2);
        allocations
    }

    #[test]
    fn test_min_weight_constraint() {
        let constraint = WeightConstraint::MinWeight {
            strategy: "A".to_string(),
            min: 0.3,
        };

        let allocations = create_test_allocations();
        assert!(constraint.check(&allocations));

        let mut low_allocations = HashMap::new();
        low_allocations.insert("A".to_string(), 0.2);
        assert!(!constraint.check(&low_allocations));
    }

    #[test]
    fn test_max_weight_constraint() {
        let constraint = WeightConstraint::MaxWeight {
            strategy: "A".to_string(),
            max: 0.5,
        };

        let allocations = create_test_allocations();
        assert!(constraint.check(&allocations));

        let mut high_allocations = HashMap::new();
        high_allocations.insert("A".to_string(), 0.6);
        assert!(!constraint.check(&high_allocations));
    }

    #[test]
    fn test_group_constraints() {
        let constraint = WeightConstraint::GroupMaxWeight {
            strategies: vec!["A".to_string(), "B".to_string()],
            max: 0.9,
        };

        let allocations = create_test_allocations();
        assert!(!constraint.check(&allocations)); // A + B = 0.8, which is <= 0.9, should pass

        // Actually 0.4 + 0.4 = 0.8 <= 0.9, so it should pass
        assert!(constraint.check(&allocations));

        let constraint2 = WeightConstraint::GroupMinWeight {
            strategies: vec!["A".to_string(), "B".to_string()],
            min: 0.7,
        };

        assert!(constraint2.check(&allocations)); // A + B = 0.8 >= 0.7
    }

    #[test]
    fn test_portfolio_constraint_builder() {
        let constraint = PortfolioConstraint::new()
            .with_min_weight("A", 0.1)
            .with_max_weight("B", 0.5)
            .with_min_strategies(2)
            .with_max_strategies(5);

        assert_eq!(constraint.weight_constraints.len(), 2);
        assert_eq!(constraint.min_strategies, Some(2));
        assert_eq!(constraint.max_strategies, Some(5));
    }

    #[test]
    fn test_constraint_check_sum() {
        let constraint = PortfolioConstraint::new();

        let mut allocations = HashMap::new();
        allocations.insert("A".to_string(), 0.5);
        allocations.insert("B".to_string(), 0.4); // Sum = 0.9, not 1.0

        assert!(constraint.check(&allocations).is_err());

        allocations.insert("B".to_string(), 0.5); // Sum = 1.0
        assert!(constraint.check(&allocations).is_ok());
    }

    #[test]
    fn test_constraint_no_short() {
        let constraint = PortfolioConstraint::new().with_short_positions(false);

        let mut allocations = HashMap::new();
        allocations.insert("A".to_string(), 1.2);
        allocations.insert("B".to_string(), -0.2); // Short position

        assert!(constraint.check(&allocations).is_err());
    }

    #[test]
    fn test_constraint_allow_short() {
        let constraint = PortfolioConstraint::new().with_short_positions(true);

        let mut allocations = HashMap::new();
        allocations.insert("A".to_string(), 1.2);
        allocations.insert("B".to_string(), -0.2); // Short position

        // Should pass short check but fail sum check (1.2 + (-0.2) = 1.0, actually OK)
        assert!(constraint.check(&allocations).is_ok());
    }

    #[test]
    fn test_constraint_leverage() {
        let constraint = PortfolioConstraint::new()
            .with_short_positions(true)
            .with_max_leverage(1.5);

        let mut allocations = HashMap::new();
        allocations.insert("A".to_string(), 1.2);
        allocations.insert("B".to_string(), -0.2);
        // Sum = 1.0, but leverage = 1.4, which is <= 1.5, should pass

        assert!(constraint.check(&allocations).is_ok());

        allocations.insert("A".to_string(), 1.5);
        allocations.insert("B".to_string(), -0.5);
        // Sum = 1.0, but leverage = 2.0, which is > 1.5, should fail

        assert!(constraint.check(&allocations).is_err());
    }

    #[test]
    fn test_constrained_strategies() {
        let constraint = PortfolioConstraint::new()
            .with_min_weight("A", 0.1)
            .with_max_weight("B", 0.5)
            .with_group_min_weight(vec!["C", "D"], 0.2);

        let strategies = constraint.constrained_strategies();
        assert!(strategies.contains(&"A".to_string()));
        assert!(strategies.contains(&"B".to_string()));
        assert!(strategies.contains(&"C".to_string()));
        assert!(strategies.contains(&"D".to_string()));
    }
}
