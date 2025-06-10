use crate::utils::erf;
use crate::change_point_detector::ChangePointDetector;
use std::collections::HashMap;

pub struct CUSUM {
    t_warmup: usize,
    p_limit: f64,
    current_t: usize,
    current_obs: Vec<f64>,
    current_mean: f64,
    current_std: f64,
}

impl CUSUM {
    pub fn new(t_warmup: usize, p_limit: f64) -> Self {
        Self {
            t_warmup,
            p_limit,
            current_t: 0,
            current_obs: Vec::new(),
            current_mean: 0.0,
            current_std: 0.1,
        }
    }

    fn reset(&mut self) {
        self.current_t = 0;
        self.current_obs.clear();
        self.current_mean = 0.0;
        self.current_std = 0.1;
    }

    fn init_params(&mut self) {
        self.current_mean = self.current_obs.iter().sum::<f64>() / self.current_obs.len() as f64;
        let variance = self.current_obs.iter()
            .map(|x| (x - self.current_mean).powi(2))
            .sum::<f64>() / self.current_obs.len() as f64;
        self.current_std = variance.sqrt().max(0.1);
    }

    fn get_prob(&self, y: f64) -> f64 {
        let p = 0.5 * (1.0 + erf(y.abs() / 2.0_f64.sqrt()));
        2.0 * (1.0 - p)
    }

    fn predict_next(&mut self, y: f64) -> (f64, bool) {
        self.current_t += 1;
        self.current_obs.push(y);

        if self.current_t == self.t_warmup {
            self.init_params();
        }

        if self.current_t >= self.t_warmup {
            let standardized_sum = (self.current_obs.iter().sum::<f64>() - 
                                  self.current_mean * self.current_obs.len() as f64) /
                                 (self.current_std * (self.current_t as f64).sqrt());
            let prob = self.get_prob(standardized_sum);
            let is_changepoint = prob < self.p_limit;
            
            if is_changepoint {
                self.reset();
            }
            
            (1.0 - prob, is_changepoint)
        } else {
            (0.0, false)
        }
    }
}

impl ChangePointDetector for CUSUM {
    fn detect(&mut self, data: &[f64]) -> Vec<usize> {
        let mut change_points = Vec::new();
        
        for (i, &value) in data.iter().enumerate() {
            let (_, is_changepoint) = self.predict_next(value);
            if is_changepoint {
                change_points.push(i);
            }
        }
        
        change_points
    }

    fn set_params(&mut self, params: HashMap<String, f64>) {
        if let Some(&t_warmup) = params.get("t_warmup") { 
            self.t_warmup = t_warmup as usize; 
        }
        if let Some(&p_limit) = params.get("p_limit") { 
            self.p_limit = p_limit; 
        }
    }

    fn reinit(&mut self) {
        self.reset();
    }
}