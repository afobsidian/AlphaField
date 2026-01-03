//! Ensemble Methods Module
//!
//! Provides ensemble model utilities including voting and averaging.

use super::models::MLModel;
use serde::{Deserialize, Serialize};

/// Ensemble combination method
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EnsembleMethod {
    /// Average predictions
    Average,
    /// Weighted average
    WeightedAverage,
    /// Majority voting (for classification)
    Voting,
}

/// Simple ensemble that combines multiple models
pub struct SimpleEnsemble {
    models: Vec<Box<dyn MLModel>>,
    weights: Vec<f64>,
    method: EnsembleMethod,
}

impl SimpleEnsemble {
    /// Create new ensemble with averaging
    pub fn new(models: Vec<Box<dyn MLModel>>) -> Self {
        let n = models.len();
        Self {
            models,
            weights: vec![1.0 / n as f64; n],
            method: EnsembleMethod::Average,
        }
    }

    /// Create ensemble with custom weights
    pub fn with_weights(models: Vec<Box<dyn MLModel>>, weights: Vec<f64>) -> Self {
        let normalized = normalize_weights(&weights);
        Self {
            models,
            weights: normalized,
            method: EnsembleMethod::WeightedAverage,
        }
    }

    /// Predict using ensemble
    pub fn predict(&self, features: &[f64]) -> f64 {
        match self.method {
            EnsembleMethod::Average => {
                let sum: f64 = self.models.iter().map(|m| m.predict(features)).sum();
                sum / self.models.len() as f64
            }
            EnsembleMethod::WeightedAverage => self
                .models
                .iter()
                .zip(self.weights.iter())
                .map(|(m, w)| m.predict(features) * w)
                .sum(),
            EnsembleMethod::Voting => {
                let preds: Vec<f64> = self.models.iter().map(|m| m.predict(features)).collect();
                majority_vote(&preds)
            }
        }
    }

    /// Check if all models are trained
    pub fn is_trained(&self) -> bool {
        self.models.iter().all(|m| m.is_trained())
    }

    /// Number of models in ensemble
    pub fn n_models(&self) -> usize {
        self.models.len()
    }
}

/// Normalize weights to sum to 1
fn normalize_weights(weights: &[f64]) -> Vec<f64> {
    let sum: f64 = weights.iter().sum();
    if sum > 0.0 {
        weights.iter().map(|w| w / sum).collect()
    } else {
        vec![1.0 / weights.len() as f64; weights.len()]
    }
}

/// Majority vote for classification
fn majority_vote(predictions: &[f64]) -> f64 {
    if predictions.is_empty() {
        return 0.0;
    }

    let positive = predictions.iter().filter(|&&p| p > 0.0).count();
    let negative = predictions.len() - positive;

    if positive > negative {
        1.0
    } else if negative > positive {
        -1.0
    } else {
        0.0 // Tie
    }
}

/// Metrics for model comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    /// Mean Absolute Error
    pub mae: f64,
    /// Mean Squared Error
    pub mse: f64,
    /// Root Mean Squared Error
    pub rmse: f64,
    /// R-squared (coefficient of determination)
    pub r_squared: f64,
    /// Accuracy (for classification)
    pub accuracy: Option<f64>,
}

impl ModelMetrics {
    /// Calculate regression metrics
    pub fn calculate_regression(predictions: &[f64], labels: &[f64]) -> Self {
        let n = predictions.len() as f64;
        if n == 0.0 {
            return Self::default();
        }

        let mae: f64 = predictions
            .iter()
            .zip(labels.iter())
            .map(|(p, l)| (p - l).abs())
            .sum::<f64>()
            / n;

        let mse: f64 = predictions
            .iter()
            .zip(labels.iter())
            .map(|(p, l)| (p - l).powi(2))
            .sum::<f64>()
            / n;

        let rmse = mse.sqrt();

        let mean_label = labels.iter().sum::<f64>() / n;
        let ss_tot: f64 = labels.iter().map(|l| (l - mean_label).powi(2)).sum();
        let ss_res: f64 = predictions
            .iter()
            .zip(labels.iter())
            .map(|(p, l)| (p - l).powi(2))
            .sum();

        let r_squared = if ss_tot > 0.0 {
            1.0 - ss_res / ss_tot
        } else {
            0.0
        };

        Self {
            mae,
            mse,
            rmse,
            r_squared,
            accuracy: None,
        }
    }

    /// Calculate classification metrics
    pub fn calculate_classification(predictions: &[f64], labels: &[f64]) -> Self {
        let n = predictions.len() as f64;
        if n == 0.0 {
            return Self::default();
        }

        let correct = predictions
            .iter()
            .zip(labels.iter())
            .filter(|(p, l)| {
                let pred_class: f64 = if **p > 0.0 { 1.0 } else { -1.0 };
                let true_class: f64 = if **l > 0.0 { 1.0 } else { -1.0 };
                (pred_class - true_class).abs() < 0.01
            })
            .count();

        let accuracy = correct as f64 / n;

        Self {
            mae: 0.0,
            mse: 0.0,
            rmse: 0.0,
            r_squared: 0.0,
            accuracy: Some(accuracy),
        }
    }
}

impl Default for ModelMetrics {
    fn default() -> Self {
        Self {
            mae: 0.0,
            mse: 0.0,
            rmse: 0.0,
            r_squared: 0.0,
            accuracy: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_weights() {
        let weights = vec![2.0, 3.0, 5.0];
        let normalized = normalize_weights(&weights);

        assert!((normalized.iter().sum::<f64>() - 1.0).abs() < 1e-10);
        assert!((normalized[0] - 0.2).abs() < 1e-10);
    }

    #[test]
    fn test_majority_vote() {
        assert_eq!(majority_vote(&[1.0, 1.0, -1.0]), 1.0);
        assert_eq!(majority_vote(&[-1.0, -1.0, 1.0]), -1.0);
        assert_eq!(majority_vote(&[1.0, -1.0]), 0.0); // Tie
    }

    #[test]
    fn test_regression_metrics() {
        let predictions = vec![1.0, 2.0, 3.0];
        let labels = vec![1.1, 2.1, 2.9];

        let metrics = ModelMetrics::calculate_regression(&predictions, &labels);

        assert!(metrics.mae < 0.2);
        assert!(metrics.rmse < 0.2);
        assert!(metrics.r_squared > 0.9);
    }

    #[test]
    fn test_classification_metrics() {
        let predictions = vec![1.0, 1.0, -1.0, -1.0];
        let labels = vec![1.0, -1.0, -1.0, -1.0]; // 3/4 correct

        let metrics = ModelMetrics::calculate_classification(&predictions, &labels);

        assert_eq!(metrics.accuracy, Some(0.75));
    }

    #[test]
    fn test_empty_metrics() {
        let metrics = ModelMetrics::calculate_regression(&[], &[]);
        assert_eq!(metrics.mae, 0.0);
    }
}
