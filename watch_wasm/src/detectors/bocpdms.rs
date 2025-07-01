// src/detectors/bocpdms.rs

use crate::change_point_detector::ChangePointDetector;
use std::collections::HashMap;

/// A very simple BOCPDMS‐style detector: compares variances in a sliding window.
pub struct BOCPDMS {
    prior_a:   f64,
    prior_b:   f64,
    intensity: f64,
}

impl BOCPDMS {
    /// Create a new BOCPDMS detector with hyperparameters.
    pub fn new(prior_a: f64, prior_b: f64, intensity: f64) -> Self {
        Self { prior_a, prior_b, intensity }
    }

    /// Core univariate variance‐ratio change‐point logic.
    fn detect_variance_change(&self, data: &[f64], window_size: usize) -> Vec<usize> {
        let mut cps = Vec::new();
        if data.len() < 2 * window_size {
            return cps;
        }
        for i in window_size..(data.len() - window_size) {
            let left  = &data[i - window_size..i];
            let right = &data[i..i + window_size];

            let mean_l = left.iter().sum::<f64>() / window_size as f64;
            let mean_r = right.iter().sum::<f64>() / window_size as f64;

            let var_l = left.iter()
                            .map(|x| (x - mean_l).powi(2))
                            .sum::<f64>() / window_size as f64;
            let var_r = right.iter()
                             .map(|x| (x - mean_r).powi(2))
                             .sum::<f64>() / window_size as f64;

            let ratio = if var_l > 0.0 && var_r > 0.0 {
                (var_l / var_r).max(var_r / var_l)
            } else {
                1.0
            };

            if ratio > self.intensity {
                cps.push(i);
            }
        }
        cps
    }

    /// Collapse a multivariate series (by row‐means) then run the above univariate detector.
    pub fn detect_multivariate(&mut self, data: &[Vec<f64>]) -> Vec<usize> {
        let univ: Vec<f64> = data
            .iter()
            .map(|row| row.iter().sum::<f64>() / row.len() as f64)
            .collect();
        self.detect(&univ)
    }
}

impl ChangePointDetector for BOCPDMS {
    /// Univariate API (uses a fixed window size of 20).
    fn detect(&mut self, data: &[f64]) -> Vec<usize> {
        self.detect_variance_change(data, 20)
    }

    fn set_params(&mut self, _params: HashMap<String, f64>) {
        // no‐op: we pass hyperparams in via `new(...)` in main.rs
    }

    fn reinit(&mut self) {
        // stateless
    }
}