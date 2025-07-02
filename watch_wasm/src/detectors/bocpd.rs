use crate::utils::gamma;
use crate::change_point_detector::ChangePointDetector;
use std::collections::HashMap;

/// Bayesian Online Change Point Detection (BOCPD)
pub struct BOCPD {
    alpha: f64,
    beta:  f64,
    kappa: f64,
    mu:    f64,
}

impl BOCPD {
   
    pub fn new(alpha: f64, beta: f64, kappa: f64, mu: f64) -> Self {
        Self { alpha, beta, kappa, mu }
    }

    #[inline]
    fn hazard_function(&self, _run_length: usize) -> f64 {
        // constant hazard = 1/250
        1.0 / 250.0
    }

    #[inline]
    fn student_t_pdf(&self, x: f64, alpha: f64, beta: f64, kappa: f64, mu: f64) -> f64 {
        let df    = 2.0 * alpha;
        let scale = (beta * (kappa + 1.0) / (alpha * kappa)).sqrt();
        let z     = (x - mu) / scale;

        let num   = gamma((df + 1.0) / 2.0);
        let den   = (df * std::f64::consts::PI).sqrt() * gamma(df / 2.0);
        let coeff = num / den;

        coeff * (1.0 + z.powi(2) / df).powf(-(df + 1.0) / 2.0) / scale
    }

    #[inline]
    fn update_parameters(
        &self,
        x: f64,
        alpha: f64,
        beta: f64,
        kappa: f64,
        mu: f64,
    ) -> (f64, f64, f64, f64) {
        let new_mu    = (kappa * mu + x) / (kappa + 1.0);
        let new_kappa = kappa + 1.0;
        let new_alpha = alpha + 0.5;
        let new_beta  = beta + (kappa * (x - mu).powi(2)) / (2.0 * (kappa + 1.0));
        (new_alpha, new_beta, new_kappa, new_mu)
    }
}

impl ChangePointDetector for BOCPD {
    fn detect(&mut self, data: &[f64]) -> Vec<usize> {
        let n = data.len();
        
        let mut r = vec![vec![0.0; n + 1]; n + 1];
        r[0][0] = 1.0;

        
        let mut params = vec![(self.alpha, self.beta, self.kappa, self.mu)];

        
        for (t, &x) in data.iter().enumerate() {
            let mut new_params = Vec::with_capacity(params.len() + 1);
            
            new_params.push((self.alpha, self.beta, self.kappa, self.mu));

            for (rl, &(a, b, k, m)) in params.iter().enumerate() {
                let pred = self.student_t_pdf(x, a, b, k, m);
                let haz  = self.hazard_function(rl);

                
                if rl + 1 <= n {
                    r[rl + 1][t + 1] = r[rl][t] * pred * (1.0 - haz);
                }
                
                r[0][t + 1] += r[rl][t] * pred * haz;

                
                new_params.push(self.update_parameters(x, a, b, k, m));
            }

            
            let col_sum: f64 = (0..=t + 1).map(|rl| r[rl][t + 1]).sum();
            if col_sum > 0.0 {
                for rl in 0..=t + 1 {
                    r[rl][t + 1] /= col_sum;
                }
            }

            params = new_params;
        }

        let nw = 5;
        let threshold = 0.1;
        let mut cps = Vec::new();
        
        for i in 1..(n - nw + 1) {
            let col = nw + i;
            if r[nw][col] > threshold {
                cps.push(i - nw);
            }
        }

        cps
    }

    fn set_params(&mut self, params: HashMap<String, f64>) {
        if let Some(&a) = params.get("alpha") { self.alpha = a; }
        if let Some(&b) = params.get("beta")  { self.beta  = b; }
        if let Some(&k) = params.get("kappa") { self.kappa = k; }
        if let Some(&m) = params.get("mu")    { self.mu    = m; }
    }

    fn reinit(&mut self) {
        // stateless
    }
}