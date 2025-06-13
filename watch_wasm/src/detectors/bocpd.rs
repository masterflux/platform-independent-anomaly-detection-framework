use crate::utils::gamma;
use crate::change_point_detector::ChangePointDetector;
use std::collections::HashMap;

pub struct BOCPD {
    alpha:        f64,
    beta:         f64,
    kappa:        f64,
    mu:           f64,
    lambda:       f64,
    cp_threshold: f64,
}

impl BOCPD {
    pub fn new(alpha: f64, beta: f64, kappa: f64, mu: f64) -> Self {
        Self {
            alpha,
            beta,
            kappa,
            mu,
            lambda: 20.0,       // hazard = 1/20 = 0.05
            cp_threshold: 0.01, // lower threshold so we actually pick some peaks
        }
    }

    fn hazard_function(&self, _run_length: usize) -> f64 {
        1.0 / self.lambda
    }

    fn student_t_pdf(&self, x: f64, alpha: f64, beta: f64, kappa: f64, mu: f64) -> f64 {
        let df    = 2.0 * alpha;
        let scale = (beta * (kappa + 1.0) / (alpha * kappa)).sqrt();
        let z     = (x - mu) / scale;

        let num = gamma((df + 1.0) / 2.0);
        let den = (df * std::f64::consts::PI).sqrt() * gamma(df / 2.0);
        let coeff = num / den;

        coeff * (1.0 + z.powi(2) / df).powf(-(df + 1.0) / 2.0) / scale
    }

    fn update_parameters(
        &self,
        x: f64,
        alpha: f64,
        beta: f64,
        kappa: f64,
        mu: f64
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
        // r[r][t] = P(run‐length = r at time t)
        let mut r = vec![vec![0.0; n + 1]; n + 1];
        r[0][0] = 1.0;

        // track posterior params for each run‐length
        let mut params = vec![(self.alpha, self.beta, self.kappa, self.mu)];

        // record instantaneous CP probability at each t
        let mut cp_prob = Vec::with_capacity(n);

        for (t, &x) in data.iter().enumerate() {
            let mut new_params = Vec::with_capacity(t + 2);
            // the “reset” branch
            new_params.push((self.alpha, self.beta, self.kappa, self.mu));

            for (run_length, &(a, b, k, m)) in params.iter().enumerate() {
                let pred = self.student_t_pdf(x, a, b, k, m);
                let haz  = self.hazard_function(run_length);

                // continuation
                if run_length + 1 <= n && t + 1 <= n {
                    r[run_length + 1][t + 1] = r[run_length][t] * pred * (1.0 - haz);
                }
                // change‐point occurs
                if t + 1 <= n {
                    r[0][t + 1] += r[run_length][t] * pred * haz;
                }

                // update run‐length parameters
                let upd = self.update_parameters(x, a, b, k, m);
                new_params.push(upd);
            }

            // normalize the column t+1
            let sum: f64 = (0..=t+1).map(|i| r[i][t + 1]).sum();
            if sum > 0.0 {
                for i in 0..=t+1 {
                    r[i][t + 1] /= sum;
                }
            }

            params = new_params;
            cp_prob.push(r[0][t + 1]);
        }

        // pick local peaks above threshold
        let mut cps = Vec::new();
        for t in 1..cp_prob.len() - 1 {
            let p = cp_prob[t];
            if p > self.cp_threshold
               && p > cp_prob[t - 1]
               && p > cp_prob[t + 1]
            {
                cps.push(t);
            }
        }
        cps
    }

    fn set_params(&mut self, params: HashMap<String, f64>) {
        if let Some(&a)  = params.get("alpha")       { self.alpha        = a;  }
        if let Some(&b)  = params.get("beta")        { self.beta         = b;  }
        if let Some(&k)  = params.get("kappa")       { self.kappa        = k;  }
        if let Some(&m)  = params.get("mu")          { self.mu           = m;  }
        if let Some(&l)  = params.get("lambda")      { self.lambda       = l;  }
        if let Some(&th) = params.get("cp_threshold"){ self.cp_threshold= th; }
    }

    fn reinit(&mut self) {
        // nothing to do
    }
}
