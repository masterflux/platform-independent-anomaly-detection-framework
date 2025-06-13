use crate::change_point_detector::ChangePointDetector;
use std::collections::HashMap;

pub struct BOCPDMS {
    prior_a: f64,
    prior_b: f64,
    intensity: f64,
}

impl BOCPDMS {
    pub fn new(prior_a: f64, prior_b: f64, intensity: f64) -> Self {
        Self { prior_a, prior_b, intensity }
    }

    /// univariate helper (your existing code)
    fn detect_variance_change(&self, data: &[f64], window_size: usize) -> Vec<usize> {
        let mut cps = Vec::new();
        if data.len() < 2 * window_size { return cps; }
        for i in window_size..data.len() - window_size {
            let l = &data[i - window_size..i];
            let r = &data[i..i + window_size];
            let ml = l.iter().sum::<f64>() / window_size as f64;
            let mr = r.iter().sum::<f64>() / window_size as f64;
            let vl = l.iter().map(|x| (x - ml).powi(2)).sum::<f64>() / window_size as f64;
            let vr = r.iter().map(|x| (x - mr).powi(2)).sum::<f64>() / window_size as f64;
            let ratio = if vl > 0.0 && vr > 0.0 { (vl/vr).max(vr/vl) } else { 1.0 };
            if ratio > self.intensity { cps.push(i); }
        }
        cps
    }

    /// NEW: collapse each row to its mean and run univariate detector
    pub fn detect_multivariate(&mut self, data: &[Vec<f64>]) -> Vec<usize> {
        let univ: Vec<f64> = data
            .iter()
            .map(|row| row.iter().sum::<f64>() / row.len() as f64)
            .collect();
        self.detect_variance_change(&univ, 20)
    }
}

impl ChangePointDetector for BOCPDMS {
    fn detect(&mut self, data: &[f64])       -> Vec<usize> { self.detect_variance_change(data, 20) }
    fn set_params(&mut self, _p: HashMap<String,f64>) { /*…*/ }
    fn reinit(&mut self) { /* stateless */ }
}
