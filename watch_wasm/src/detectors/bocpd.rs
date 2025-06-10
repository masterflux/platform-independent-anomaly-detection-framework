use crate::utils::gamma;
use crate::change_point_detector::ChangePointDetector;
use std::collections::HashMap;

pub struct BOCPD {
    alpha: f64,
    beta: f64,
    kappa: f64,
    mu: f64,
    lambda: f64,
}

impl BOCPD {
    pub fn new(alpha: f64, beta: f64, kappa: f64, mu: f64) -> Self {
        Self {
            alpha,
            beta,
            kappa,
            mu,
            lambda: 250.0,
        }
    }

    fn hazard_function(&self, _r: usize) -> f64{
        1.0 / self.lambda
    }

    fn student_t_pdf(&self, x: f64, alpha: f64, beta: f64, kappa: f64, mu: f64) -> f64 {
        let df = 2.0 * alpha;
        let scale = (beta * (kappa + 1.0) / (alpha * kappa)).sqrt();
        let z = (x - mu) / scale;
        
        let numerator = gamma((df + 1.0) / 2.0);
        let denominator = (df * std::f64::consts::PI).sqrt() * gamma(df / 2.0);
        let coefficient = numerator / denominator;
        
        coefficient * (1.0 + z.powi(2) / df).powf(-(df + 1.0) / 2.0) / scale
    }

    fn update_parameters(&self, x: f64, alpha: f64, beta: f64, kappa: f64, mu: f64) 
        -> (f64, f64, f64, f64) {
        let new_mu = (kappa * mu + x) / (kappa + 1.0);
        let new_kappa = kappa + 1.0;
        let new_alpha = alpha + 0.5;
        let new_beta = beta + (kappa * (x - mu).powi(2)) / (2.0 * (kappa + 1.0));
        
        (new_alpha, new_beta, new_kappa, new_mu)
    }
}

impl ChangePointDetector for BOCPD {
    fn detect(&mut self, data: &[f64]) -> Vec<usize> {
        let n = data.len();
        let mut r = vec![vec![0.0; n + 1]; n + 1];
        r[0][0] = 1.0;
        
        let mut change_points = Vec::new();
        let mut params = vec![(self.alpha, self.beta, self.kappa, self.mu)];
        
        for (t, &x) in data.iter().enumerate() {
            let mut new_params = Vec::new();
            new_params.push((self.alpha, self.beta, self.kappa, self.mu)); // reset params
            
            for (run_length, &(alpha, beta, kappa, mu)) in params.iter().enumerate() {
                let pred_prob = self.student_t_pdf(x, alpha, beta, kappa, mu);
                let hazard = self.hazard_function(run_length);
                
                // Update run length probabilities
                if run_length + 1 < r.len() && t + 1 < r[0].len() {
                    r[run_length + 1][t + 1] = r[run_length][t] * pred_prob * (1.0 - hazard);
                }
                
                // Change point probability
                if t + 1 < r[0].len() {
                    r[0][t + 1] += r[run_length][t] * pred_prob * hazard;
                }
                
                // Update parameters
                let updated = self.update_parameters(x, alpha, beta, kappa, mu);
                if run_length + 1 < new_params.len() {
                    new_params[run_length + 1] = updated;
                } else {
                    new_params.push(updated);
                }
            }
            
            // Normalize probabilities
            let sum: f64 = (0..=t+1).map(|i| r[i][t + 1]).sum();
            if sum > 0.0 {
                for i in 0..=t+1 {
                    r[i][t + 1] /= sum;
                }
            }
            
            params = new_params;
            
            // Detect change points (simple threshold on change point probability)
            if r[0][t + 1] > 0.3 {
                change_points.push(t);
            }
        }
        
        change_points
    }

    fn set_params(&mut self, params: HashMap<String, f64>) {
        if let Some(&alpha) = params.get("alpha") { self.alpha = alpha; }
        if let Some(&beta) = params.get("beta") { self.beta = beta; }
        if let Some(&kappa) = params.get("kappa") { self.kappa = kappa; }
        if let Some(&mu) = params.get("mu") { self.mu = mu; }
    }

    fn reinit(&mut self) {
        // Reset to initial state
        self.reinit();
    }
}