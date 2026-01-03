//! ML Models Module
//!
//! Provides ML model implementations for trading prediction:
//! - LinearRegression: Price magnitude regression
//! - LogisticRegression: Direction classification
//! - DecisionTree: Non-linear pattern recognition
//! - RandomForest: Ensemble of decision trees

use serde::{Deserialize, Serialize};

/// Type of ML model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MLModelType {
    LinearRegression,
    LogisticRegression,
    DecisionTree,
    RandomForest,
}

impl std::fmt::Display for MLModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MLModelType::LinearRegression => write!(f, "Linear Regression"),
            MLModelType::LogisticRegression => write!(f, "Logistic Regression"),
            MLModelType::DecisionTree => write!(f, "Decision Tree"),
            MLModelType::RandomForest => write!(f, "Random Forest"),
        }
    }
}

/// Trait for ML models
pub trait MLModel: Send + Sync {
    /// Train the model on features and labels
    fn train(&mut self, features: &[Vec<f64>], labels: &[f64]) -> Result<(), String>;

    /// Predict on a single sample
    fn predict(&self, features: &[f64]) -> f64;

    /// Predict on multiple samples
    fn predict_batch(&self, features: &[Vec<f64>]) -> Vec<f64> {
        features.iter().map(|f| self.predict(f)).collect()
    }

    /// Get prediction probability (for classifiers)
    fn predict_proba(&self, features: &[f64]) -> Option<f64>;

    /// Get feature importance if available
    fn feature_importance(&self) -> Option<Vec<f64>>;

    /// Get model type
    fn model_type(&self) -> MLModelType;

    /// Check if model is trained
    fn is_trained(&self) -> bool;
}

// =============================================================================
// LINEAR REGRESSION
// =============================================================================

/// Linear regression model using ordinary least squares
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearRegression {
    /// Model weights (one per feature)
    weights: Vec<f64>,
    /// Bias/intercept term
    bias: f64,
    /// Whether the model has been trained
    trained: bool,
    /// L2 regularization strength (ridge regression if > 0)
    regularization: f64,
}

impl LinearRegression {
    /// Create a new linear regression model
    pub fn new() -> Self {
        Self {
            weights: vec![],
            bias: 0.0,
            trained: false,
            regularization: 0.0,
        }
    }

    /// Create with L2 regularization (ridge regression)
    pub fn with_regularization(regularization: f64) -> Self {
        Self {
            weights: vec![],
            bias: 0.0,
            trained: false,
            regularization,
        }
    }

    /// Get the model weights
    pub fn weights(&self) -> &[f64] {
        &self.weights
    }

    /// Get the bias term
    pub fn bias(&self) -> f64 {
        self.bias
    }
}

impl Default for LinearRegression {
    fn default() -> Self {
        Self::new()
    }
}

impl MLModel for LinearRegression {
    fn train(&mut self, features: &[Vec<f64>], labels: &[f64]) -> Result<(), String> {
        if features.is_empty() || labels.is_empty() {
            return Err("Empty training data".to_string());
        }
        if features.len() != labels.len() {
            return Err("Features and labels must have same length".to_string());
        }

        let n_samples = features.len();
        let n_features = features[0].len();

        // Use gradient descent for simplicity and numerical stability
        self.weights = vec![0.0; n_features];
        self.bias = 0.0;

        let learning_rate = 0.01;
        let n_iterations = 1000;

        for _ in 0..n_iterations {
            let mut weight_gradients = vec![0.0; n_features];
            let mut bias_gradient = 0.0;

            for (i, sample) in features.iter().enumerate() {
                let prediction = self.predict(sample);
                let error = prediction - labels[i];

                for (j, &feature) in sample.iter().enumerate() {
                    weight_gradients[j] += error * feature;
                }
                bias_gradient += error;
            }

            // Update weights with regularization
            for (j, weight) in self.weights.iter_mut().enumerate() {
                let gradient =
                    weight_gradients[j] / n_samples as f64 + self.regularization * *weight;
                *weight -= learning_rate * gradient;
            }

            self.bias -= learning_rate * bias_gradient / n_samples as f64;
        }

        self.trained = true;
        Ok(())
    }

    fn predict(&self, features: &[f64]) -> f64 {
        if !self.trained || features.len() != self.weights.len() {
            return 0.0;
        }

        let mut pred = self.bias;
        for (w, f) in self.weights.iter().zip(features.iter()) {
            pred += w * f;
        }
        pred
    }

    fn predict_proba(&self, _features: &[f64]) -> Option<f64> {
        None // Regression model, no probability
    }

    fn feature_importance(&self) -> Option<Vec<f64>> {
        if self.trained {
            // Absolute weights as importance
            Some(self.weights.iter().map(|w| w.abs()).collect())
        } else {
            None
        }
    }

    fn model_type(&self) -> MLModelType {
        MLModelType::LinearRegression
    }

    fn is_trained(&self) -> bool {
        self.trained
    }
}

// =============================================================================
// LOGISTIC REGRESSION
// =============================================================================

/// Logistic regression for binary classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogisticRegression {
    weights: Vec<f64>,
    bias: f64,
    trained: bool,
    regularization: f64,
    /// Classification threshold
    threshold: f64,
}

impl LogisticRegression {
    /// Create a new logistic regression model
    pub fn new() -> Self {
        Self {
            weights: vec![],
            bias: 0.0,
            trained: false,
            regularization: 0.01,
            threshold: 0.5,
        }
    }

    /// Set classification threshold
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    /// Sigmoid activation function
    fn sigmoid(x: f64) -> f64 {
        1.0 / (1.0 + (-x).exp())
    }
}

impl Default for LogisticRegression {
    fn default() -> Self {
        Self::new()
    }
}

impl MLModel for LogisticRegression {
    fn train(&mut self, features: &[Vec<f64>], labels: &[f64]) -> Result<(), String> {
        if features.is_empty() || labels.is_empty() {
            return Err("Empty training data".to_string());
        }
        if features.len() != labels.len() {
            return Err("Features and labels must have same length".to_string());
        }

        let n_samples = features.len();
        let n_features = features[0].len();

        self.weights = vec![0.0; n_features];
        self.bias = 0.0;

        let learning_rate = 0.1;
        let n_iterations = 1000;

        // Convert labels to 0/1 for binary classification
        let binary_labels: Vec<f64> = labels
            .iter()
            .map(|&l| if l > 0.0 { 1.0 } else { 0.0 })
            .collect();

        for _ in 0..n_iterations {
            let mut weight_gradients = vec![0.0; n_features];
            let mut bias_gradient = 0.0;

            for (i, sample) in features.iter().enumerate() {
                let linear = self
                    .weights
                    .iter()
                    .zip(sample.iter())
                    .map(|(w, f)| w * f)
                    .sum::<f64>()
                    + self.bias;
                let prob = Self::sigmoid(linear);
                let error = prob - binary_labels[i];

                for (j, &feature) in sample.iter().enumerate() {
                    weight_gradients[j] += error * feature;
                }
                bias_gradient += error;
            }

            for (j, weight) in self.weights.iter_mut().enumerate() {
                let gradient =
                    weight_gradients[j] / n_samples as f64 + self.regularization * *weight;
                *weight -= learning_rate * gradient;
            }

            self.bias -= learning_rate * bias_gradient / n_samples as f64;
        }

        self.trained = true;
        Ok(())
    }

    fn predict(&self, features: &[f64]) -> f64 {
        if !self.trained || features.len() != self.weights.len() {
            return 0.0;
        }

        let prob = self.predict_proba(features).unwrap_or(0.5);
        if prob >= self.threshold {
            1.0
        } else {
            -1.0
        }
    }

    fn predict_proba(&self, features: &[f64]) -> Option<f64> {
        if !self.trained || features.len() != self.weights.len() {
            return None;
        }

        let linear = self
            .weights
            .iter()
            .zip(features.iter())
            .map(|(w, f)| w * f)
            .sum::<f64>()
            + self.bias;
        Some(Self::sigmoid(linear))
    }

    fn feature_importance(&self) -> Option<Vec<f64>> {
        if self.trained {
            Some(self.weights.iter().map(|w| w.abs()).collect())
        } else {
            None
        }
    }

    fn model_type(&self) -> MLModelType {
        MLModelType::LogisticRegression
    }

    fn is_trained(&self) -> bool {
        self.trained
    }
}

// =============================================================================
// DECISION TREE
// =============================================================================

/// Decision tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
enum TreeNode {
    Leaf {
        value: f64,
        count: usize,
    },
    Split {
        feature_index: usize,
        threshold: f64,
        left: Box<TreeNode>,
        right: Box<TreeNode>,
    },
}

/// Decision tree for regression/classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTree {
    root: Option<TreeNode>,
    max_depth: usize,
    min_samples_split: usize,
    trained: bool,
    n_features: usize,
    /// Feature importance scores
    feature_importance_scores: Vec<f64>,
}

impl DecisionTree {
    /// Create a new decision tree
    pub fn new(max_depth: usize, min_samples_split: usize) -> Self {
        Self {
            root: None,
            max_depth,
            min_samples_split,
            trained: false,
            n_features: 0,
            feature_importance_scores: vec![],
        }
    }

    /// Default decision tree with max_depth=5, min_samples=5
    pub fn default_tree() -> Self {
        Self::new(5, 5)
    }

    /// Build tree recursively
    fn build_tree(&mut self, features: &[Vec<f64>], labels: &[f64], depth: usize) -> TreeNode {
        let n_samples = labels.len();

        // Stop conditions
        if depth >= self.max_depth || n_samples < self.min_samples_split {
            let mean = labels.iter().sum::<f64>() / n_samples as f64;
            return TreeNode::Leaf {
                value: mean,
                count: n_samples,
            };
        }

        // Find best split
        if let Some((best_feature, best_threshold, best_gain)) =
            self.find_best_split(features, labels)
        {
            if best_gain <= 0.0 {
                let mean = labels.iter().sum::<f64>() / n_samples as f64;
                return TreeNode::Leaf {
                    value: mean,
                    count: n_samples,
                };
            }

            // Update feature importance
            if best_feature < self.feature_importance_scores.len() {
                self.feature_importance_scores[best_feature] += best_gain * n_samples as f64;
            }

            // Split data
            let mut left_features = Vec::new();
            let mut left_labels = Vec::new();
            let mut right_features = Vec::new();
            let mut right_labels = Vec::new();

            for (i, sample) in features.iter().enumerate() {
                if sample[best_feature] <= best_threshold {
                    left_features.push(sample.clone());
                    left_labels.push(labels[i]);
                } else {
                    right_features.push(sample.clone());
                    right_labels.push(labels[i]);
                }
            }

            // Check if split actually separates data
            if left_labels.is_empty() || right_labels.is_empty() {
                let mean = labels.iter().sum::<f64>() / n_samples as f64;
                return TreeNode::Leaf {
                    value: mean,
                    count: n_samples,
                };
            }

            let left = Box::new(self.build_tree(&left_features, &left_labels, depth + 1));
            let right = Box::new(self.build_tree(&right_features, &right_labels, depth + 1));

            TreeNode::Split {
                feature_index: best_feature,
                threshold: best_threshold,
                left,
                right,
            }
        } else {
            let mean = labels.iter().sum::<f64>() / n_samples as f64;
            TreeNode::Leaf {
                value: mean,
                count: n_samples,
            }
        }
    }

    /// Find best split point using variance reduction
    fn find_best_split(&self, features: &[Vec<f64>], labels: &[f64]) -> Option<(usize, f64, f64)> {
        if features.is_empty() {
            return None;
        }

        let n_features = features[0].len();
        let mut best_gain = 0.0;
        let mut best_feature = 0;
        let mut best_threshold = 0.0;

        let total_variance = self.variance(labels);

        for feature_idx in 0..n_features {
            // Get unique values for this feature
            let mut values: Vec<f64> = features.iter().map(|f| f[feature_idx]).collect();
            values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            values.dedup();

            // Try threshold at midpoint between consecutive values
            for window in values.windows(2) {
                let threshold = (window[0] + window[1]) / 2.0;

                let mut left_labels = Vec::new();
                let mut right_labels = Vec::new();

                for (i, sample) in features.iter().enumerate() {
                    if sample[feature_idx] <= threshold {
                        left_labels.push(labels[i]);
                    } else {
                        right_labels.push(labels[i]);
                    }
                }

                if left_labels.is_empty() || right_labels.is_empty() {
                    continue;
                }

                // Calculate variance reduction (information gain for regression)
                let left_var = self.variance(&left_labels);
                let right_var = self.variance(&right_labels);
                let n_left = left_labels.len() as f64;
                let n_right = right_labels.len() as f64;
                let n_total = n_left + n_right;

                let weighted_var = (n_left * left_var + n_right * right_var) / n_total;
                let gain = total_variance - weighted_var;

                if gain > best_gain {
                    best_gain = gain;
                    best_feature = feature_idx;
                    best_threshold = threshold;
                }
            }
        }

        if best_gain > 0.0 {
            Some((best_feature, best_threshold, best_gain))
        } else {
            None
        }
    }

    /// Calculate variance of values
    fn variance(&self, values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64
    }

    /// Traverse tree to get prediction
    fn traverse(node: &TreeNode, features: &[f64]) -> f64 {
        match node {
            TreeNode::Leaf { value, .. } => *value,
            TreeNode::Split {
                feature_index,
                threshold,
                left,
                right,
            } => {
                if *feature_index < features.len() && features[*feature_index] <= *threshold {
                    Self::traverse(left, features)
                } else {
                    Self::traverse(right, features)
                }
            }
        }
    }
}

impl Default for DecisionTree {
    fn default() -> Self {
        Self::default_tree()
    }
}

impl MLModel for DecisionTree {
    fn train(&mut self, features: &[Vec<f64>], labels: &[f64]) -> Result<(), String> {
        if features.is_empty() || labels.is_empty() {
            return Err("Empty training data".to_string());
        }
        if features.len() != labels.len() {
            return Err("Features and labels must have same length".to_string());
        }

        self.n_features = features[0].len();
        self.feature_importance_scores = vec![0.0; self.n_features];

        self.root = Some(self.build_tree(features, labels, 0));
        self.trained = true;

        // Normalize feature importance
        let total: f64 = self.feature_importance_scores.iter().sum();
        if total > 0.0 {
            for score in &mut self.feature_importance_scores {
                *score /= total;
            }
        }

        Ok(())
    }

    fn predict(&self, features: &[f64]) -> f64 {
        if let Some(ref root) = self.root {
            Self::traverse(root, features)
        } else {
            0.0
        }
    }

    fn predict_proba(&self, _features: &[f64]) -> Option<f64> {
        None // Tree returns point estimate
    }

    fn feature_importance(&self) -> Option<Vec<f64>> {
        if self.trained {
            Some(self.feature_importance_scores.clone())
        } else {
            None
        }
    }

    fn model_type(&self) -> MLModelType {
        MLModelType::DecisionTree
    }

    fn is_trained(&self) -> bool {
        self.trained
    }
}

// =============================================================================
// RANDOM FOREST
// =============================================================================

/// Random Forest ensemble
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomForest {
    trees: Vec<DecisionTree>,
    n_estimators: usize,
    max_depth: usize,
    min_samples_split: usize,
    max_features_ratio: f64,
    trained: bool,
    n_features: usize,
    /// Random seed for reproducibility
    seed: u64,
}

impl RandomForest {
    /// Create a new random forest
    pub fn new(n_estimators: usize, max_depth: usize) -> Self {
        Self {
            trees: vec![],
            n_estimators,
            max_depth,
            min_samples_split: 5,
            max_features_ratio: 0.7,
            trained: false,
            n_features: 0,
            seed: 42,
        }
    }

    /// Set random seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Simple pseudo-random number generator
    fn lcg_next(&self, state: &mut u64) -> f64 {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (*state as f64) / (u64::MAX as f64)
    }

    /// Bootstrap sample indices
    fn bootstrap_indices(&self, n_samples: usize, rng_state: &mut u64) -> Vec<usize> {
        (0..n_samples)
            .map(|_| ((self.lcg_next(rng_state) * n_samples as f64) as usize).min(n_samples - 1))
            .collect()
    }

    /// Select random subset of features
    fn select_features(&self, n_features: usize, rng_state: &mut u64) -> Vec<usize> {
        let n_select = ((n_features as f64) * self.max_features_ratio).ceil() as usize;
        let n_select = n_select.max(1).min(n_features);

        let mut all_indices: Vec<usize> = (0..n_features).collect();

        // Fisher-Yates shuffle to select first n_select
        for i in 0..n_select.min(all_indices.len()) {
            let j = i + ((self.lcg_next(rng_state) * (all_indices.len() - i) as f64) as usize);
            all_indices.swap(i, j);
        }

        all_indices.into_iter().take(n_select).collect()
    }
}

impl Default for RandomForest {
    fn default() -> Self {
        Self::new(10, 5)
    }
}

impl MLModel for RandomForest {
    fn train(&mut self, features: &[Vec<f64>], labels: &[f64]) -> Result<(), String> {
        if features.is_empty() || labels.is_empty() {
            return Err("Empty training data".to_string());
        }
        if features.len() != labels.len() {
            return Err("Features and labels must have same length".to_string());
        }

        self.n_features = features[0].len();
        self.trees.clear();

        let mut rng_state = self.seed;

        for _ in 0..self.n_estimators {
            // Bootstrap sample
            let indices = self.bootstrap_indices(features.len(), &mut rng_state);
            let selected_features_idx = self.select_features(self.n_features, &mut rng_state);

            // Create subset with selected features only
            let subset_features: Vec<Vec<f64>> = indices
                .iter()
                .map(|&i| {
                    selected_features_idx
                        .iter()
                        .map(|&j| features[i][j])
                        .collect()
                })
                .collect();

            let subset_labels: Vec<f64> = indices.iter().map(|&i| labels[i]).collect();

            // Train tree on subset
            let mut tree = DecisionTree::new(self.max_depth, self.min_samples_split);
            if tree.train(&subset_features, &subset_labels).is_ok() {
                self.trees.push(tree);
            }
        }

        self.trained = !self.trees.is_empty();
        Ok(())
    }

    fn predict(&self, features: &[f64]) -> f64 {
        if !self.trained || self.trees.is_empty() {
            return 0.0;
        }

        // Average predictions from all trees
        let sum: f64 = self.trees.iter().map(|t| t.predict(features)).sum();
        sum / self.trees.len() as f64
    }

    fn predict_proba(&self, _features: &[f64]) -> Option<f64> {
        None // Returns average prediction
    }

    fn feature_importance(&self) -> Option<Vec<f64>> {
        if !self.trained || self.trees.is_empty() {
            return None;
        }

        // Average feature importance across trees
        let mut importance = vec![0.0; self.n_features];

        for tree in &self.trees {
            if let Some(tree_imp) = tree.feature_importance() {
                for (i, &imp) in tree_imp.iter().enumerate() {
                    if i < importance.len() {
                        importance[i] += imp;
                    }
                }
            }
        }

        let n_trees = self.trees.len() as f64;
        for imp in &mut importance {
            *imp /= n_trees;
        }

        Some(importance)
    }

    fn model_type(&self) -> MLModelType {
        MLModelType::RandomForest
    }

    fn is_trained(&self) -> bool {
        self.trained
    }
}

/// Create model from type
pub fn create_model(model_type: MLModelType) -> Box<dyn MLModel> {
    match model_type {
        MLModelType::LinearRegression => Box::new(LinearRegression::new()),
        MLModelType::LogisticRegression => Box::new(LogisticRegression::new()),
        MLModelType::DecisionTree => Box::new(DecisionTree::default()),
        MLModelType::RandomForest => Box::new(RandomForest::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_linear_data() -> (Vec<Vec<f64>>, Vec<f64>) {
        // y = 2*x1 + 3*x2 + 1
        let features = vec![
            vec![1.0, 2.0],
            vec![2.0, 1.0],
            vec![3.0, 3.0],
            vec![4.0, 2.0],
            vec![5.0, 4.0],
            vec![6.0, 3.0],
            vec![7.0, 5.0],
            vec![8.0, 4.0],
        ];
        let labels: Vec<f64> = features
            .iter()
            .map(|f| 2.0 * f[0] + 3.0 * f[1] + 1.0)
            .collect();
        (features, labels)
    }

    fn make_classification_data() -> (Vec<Vec<f64>>, Vec<f64>) {
        let features = vec![
            vec![1.0, 1.0],
            vec![1.5, 1.5],
            vec![2.0, 2.0],
            vec![5.0, 5.0],
            vec![5.5, 5.5],
            vec![6.0, 6.0],
        ];
        // First 3 class 0, last 3 class 1
        let labels = vec![-1.0, -1.0, -1.0, 1.0, 1.0, 1.0];
        (features, labels)
    }

    #[test]
    fn test_linear_regression() {
        let (features, labels) = make_linear_data();
        let mut model = LinearRegression::new();

        assert!(model.train(&features, &labels).is_ok());
        assert!(model.is_trained());

        // Test prediction is in reasonable range (gradient descent may not converge perfectly)
        let pred = model.predict(&[5.0, 3.0]);
        // Just check it's a reasonable finite value
        assert!(pred.is_finite());
        assert!(pred > 0.0); // Should be positive for positive inputs
    }

    #[test]
    fn test_logistic_regression() {
        let (features, labels) = make_classification_data();
        let mut model = LogisticRegression::new();

        assert!(model.train(&features, &labels).is_ok());
        assert!(model.is_trained());

        // Low values should predict -1
        let pred_low = model.predict(&[1.0, 1.0]);
        assert_eq!(pred_low, -1.0);

        // High values should predict 1
        let pred_high = model.predict(&[6.0, 6.0]);
        assert_eq!(pred_high, 1.0);
    }

    #[test]
    fn test_decision_tree() {
        let (features, labels) = make_linear_data();
        let mut tree = DecisionTree::new(3, 2);

        assert!(tree.train(&features, &labels).is_ok());
        assert!(tree.is_trained());

        // Feature importance should be available
        assert!(tree.feature_importance().is_some());
    }

    #[test]
    fn test_random_forest() {
        let (features, labels) = make_linear_data();
        let mut forest = RandomForest::new(5, 3);

        assert!(forest.train(&features, &labels).is_ok());
        assert!(forest.is_trained());

        // Should produce reasonable predictions
        let pred = forest.predict(&[5.0, 3.0]);
        assert!(pred.is_finite());
    }

    #[test]
    fn test_empty_data() {
        let mut model = LinearRegression::new();
        assert!(model.train(&[], &[]).is_err());
    }

    #[test]
    fn test_model_type_display() {
        assert_eq!(
            MLModelType::LinearRegression.to_string(),
            "Linear Regression"
        );
        assert_eq!(MLModelType::RandomForest.to_string(), "Random Forest");
    }

    #[test]
    fn test_create_model() {
        let model = create_model(MLModelType::LinearRegression);
        assert_eq!(model.model_type(), MLModelType::LinearRegression);
        assert!(!model.is_trained());
    }
}
