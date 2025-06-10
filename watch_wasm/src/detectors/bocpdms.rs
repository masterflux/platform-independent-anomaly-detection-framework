use crate::change_point_detector::ChangePointDetector;
use std::collections::HashMap;

pub struct BOCPDMS {
    prior_a: f64,
    prior_b: f64,
    intensity: f64,
}

impl BOCPDMS {
    pub fn new(prior_a: f64, prior_b: f64, intensity: f64) -> Self {
        Self {
            prior_a,
            prior_b,
            intensity,
        }
    }

    // Simplified multivariate detection using variance change
    fn detect_variance_change(&self, data: &[f64], window_size: usize) -> Vec<usize> {
        let mut change_points = Vec::new();
        
        if data.len() < 2 * window_size {
            return change_points;
        }
        
        for i in window_size..(data.len() - window_size) {
            let left_window = &data[i-window_size..i];
            let right_window = &data[i..i+window_size];
            
            let left_mean = left_window.iter().sum::<f64>() / window_size as f64;
            let right_mean = right_window.iter().sum::<f64>() / window_size as f64;
            
            let left_var = left_window.iter()
                .map(|x| (x - left_mean).powi(2))
                .sum::<f64>() / window_size as f64;
            let right_var = right_window.iter()
                .map(|x| (x - right_mean).powi(2))
                .sum::<f64>() / window_size as f64;
            
            // Simple ratio test for variance change
            let ratio = if left_var > 0.0 && right_var > 0.0 {
                (left_var / right_var).max(right_var / left_var)
            } else {
                1.0
            };
            
            if ratio > self.intensity {
                change_points.push(i);
            }
        }
        
        change_points
    }
}

impl ChangePointDetector for BOCPDMS {
    fn detect(&mut self, data: &[f64]) -> Vec<usize> {
        self.detect_variance_change(data, 20) // Fixed window size
    }

    fn set_params(&mut self, params: HashMap<String, f64>) {
        if let Some(&prior_a) = params.get("prior_a") {
            self.prior_a = prior_a;
        }
        if let Some(&prior_b) = params.get("prior_b") {
            self.prior_b = prior_b;
        }
        if let Some(&intensity) = params.get("intensity") {
            self.intensity = intensity;
        }
    }

    fn reinit(&mut self) {
        // Stateless, no need to reinit
    }
}