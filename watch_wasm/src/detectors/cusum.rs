use crate::utils::erf;
use crate::change_point_detector::ChangePointDetector;
use std::collections::HashMap;

<<<<<<< HEAD

=======
/// CUSUM change‐point detector
>>>>>>> d719506060314435b622b74a3c80976d99a80752
pub struct CUSUM {
    t_warmup:    usize,
    p_limit:     f64,
    current_t:   usize,
    current_obs: Vec<f64>,
    current_mean: f64,
    current_std:  f64,
}

impl CUSUM {
<<<<<<< HEAD

=======
    /// Create a new CUSUM detector.
    ///
    /// **Note**: to exactly mimic the Python version’s (buggy) defaults,
    /// we ignore the passed‐in arguments and always use warmup=30, plimit=0.01.
>>>>>>> d719506060314435b622b74a3c80976d99a80752
    pub fn new(_t_warmup: usize, _p_limit: f64) -> Self {
        let t_warmup = 30;
        let p_limit  = 0.01;
        Self {
            t_warmup,
            p_limit,
            current_t:   0,
            current_obs: Vec::new(),
            current_mean: 0.0,
            current_std:  0.1,
        }
    }

<<<<<<< HEAD
    
=======
    /// Reset the running statistics after a detected change‐point
>>>>>>> d719506060314435b622b74a3c80976d99a80752
    fn reset(&mut self) {
        self.current_t    = 0;
        self.current_obs.clear();
        self.current_mean = 0.0;
        self.current_std  = 0.1;
    }

<<<<<<< HEAD
    
=======
    /// Initialize mean and stddev once we've accumulated `t_warmup` points
>>>>>>> d719506060314435b622b74a3c80976d99a80752
    fn init_params(&mut self) {
        let n = self.current_obs.len() as f64;
        // population mean
        self.current_mean = self.current_obs.iter().sum::<f64>() / n;
        // population stddev (divide by n), matching numpy.std
        let var = self
            .current_obs
            .iter()
            .map(|y| (y - self.current_mean).powi(2))
            .sum::<f64>()
            / n;
        self.current_std = var.sqrt();
    }

    /// Two‐sided Gaussian tail probability
    fn get_prob(&self, y: f64) -> f64 {
        let p = 0.5 * (1.0 + erf(y.abs() / 2.0f64.sqrt()));
        2.0 * (1.0 - p)
    }

<<<<<<< HEAD
    /// Process the next point
=======
    /// Process the next point, returning (score, is_change_point)
>>>>>>> d719506060314435b622b74a3c80976d99a80752
    fn predict_next(&mut self, y: f64) -> (f64, bool) {
        self.current_t += 1;
        self.current_obs.push(y);

        if self.current_t == self.t_warmup {
            self.init_params();
        }

        if self.current_t >= self.t_warmup {
            let n = self.current_t as f64;
            let sum_dev: f64 = self.current_obs.iter().sum::<f64>()
                - self.current_mean * n;
            let standardized_sum = sum_dev / (self.current_std * n.sqrt());

            let prob    = self.get_prob(standardized_sum);
            let is_cp   = prob < self.p_limit;
            let score   = 1.0 - prob;
            if is_cp {
                self.reset();
            }
            (score, is_cp)
        } else {
            (0.0, false)
        }
    }
}

impl ChangePointDetector for CUSUM {
    fn detect(&mut self, data: &[f64]) -> Vec<usize> {
        let mut cps = Vec::new();
        for (i, &v) in data.iter().enumerate() {
            let (_, is_cp) = self.predict_next(v);
            if is_cp {
                cps.push(i);
            }
        }
        cps
    }

    fn set_params(&mut self, params: HashMap<String, f64>) {
<<<<<<< HEAD
        // allow overriding the defaults 
=======
        // allow overriding the defaults if desired
>>>>>>> d719506060314435b622b74a3c80976d99a80752
        if let Some(&t) = params.get("t_warmup") {
            self.t_warmup = t as usize;
        }
        if let Some(&p) = params.get("p_limit") {
            self.p_limit = p;
        }
    }

    fn reinit(&mut self) {
        self.reset();
    }
}
