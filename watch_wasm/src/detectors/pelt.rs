use crate::change_point_detector::ChangePointDetector;
use std::collections::HashMap;

fn pdist(rows: &[Vec<f64>]) -> Vec<f64> {
    let n = rows.len();
    let mut out = Vec::with_capacity(n * (n - 1) / 2);
    for i in 0..n {
        for j in i + 1..n {
            let mut d2 = 0.0;
            for (a, b) in rows[i].iter().zip(&rows[j]) {
                d2 += (a - b).powi(2);
            }
            out.push(d2);
        }
    }
    out
}

fn squareform(cond: &[f64]) -> Vec<Vec<f64>> {
    let s = cond.len();
    let d = ((1.0 + (1.0 + 8.0 * (s as f64)).sqrt()) / 2.0).round() as usize;
    let mut m = vec![vec![0.0; d]; d];
    let mut idx = 0;
    for i in 0..d {
        for j in (i + 1)..d {
            let v = cond[idx];
            m[i][j] = v;
            m[j][i] = v;
            idx += 1;
        }
    }
    m
}


struct CostRbf {
    min_size:    usize,
    gram:        Vec<Vec<f64>>,
    psum:        Vec<Vec<f64>>, 
    diag_prefix: Vec<f64>,      
    n:           usize,
}

impl CostRbf {
    fn new(min_size: usize) -> Self {
        Self { min_size, gram: Vec::new(), psum: Vec::new(), diag_prefix: Vec::new(), n: 0 }
    }

    
    fn fit(&mut self, rows: &[Vec<f64>]) {
        self.n = rows.len();
        
        let mut d2 = pdist(rows);
        d2.retain(|v| v.is_finite());
        d2.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        let gamma = d2.get(d2.len()/2).filter(|&&m| m>0.0).map(|&m|1.0/m).unwrap_or(1.0);

        
        let sq = squareform(&pdist(rows));
        self.gram = sq.into_iter()
            .map(|row| row.into_iter().map(|d2| (-(gamma*d2)).exp()).collect())
            .collect();

        
        let n = self.n;
        let mut ps = vec![vec![0.0; n+1]; n+1];
        for i in 0..n {
            let mut acc = 0.0;
            for j in 0..n {
                acc += self.gram[i][j];
                ps[i+1][j+1] = ps[i][j+1] + acc;
            }
        }
        self.psum = ps;

        let mut dp = Vec::with_capacity(n+1);
        dp.push(0.0);
        for i in 0..n { dp.push(dp[i] + self.gram[i][i]); }
        self.diag_prefix = dp;
    }

    
    fn error(&self, start: usize, end: usize) -> f64 {
        let len = end.saturating_sub(start);
        if len < self.min_size { return f64::INFINITY; }
        let trace = self.diag_prefix[end] - self.diag_prefix[start];
        let ps = &self.psum;
        let total = ps[end][end] - ps[start][end] - ps[end][start] + ps[start][start];
        trace - total / (len as f64)
    }
}

pub struct PELT {
    pen:       f64,
    min_size:  usize,
    jump:      usize,
    n:         usize,
    cost:      CostRbf,
}

impl PELT {
    pub fn new(pen: f64, min_size: usize, jump: usize) -> Self {
        Self { pen, min_size, jump, n:0, cost: CostRbf::new(min_size) }
    }

    fn fit(&mut self, data: &[Vec<f64>]) {
        self.n = data.len();
        self.cost.min_size = self.min_size;
        self.cost.fit(data);
    }

    fn segment(&self) -> HashMap<(usize, usize), f64> {
        let mut parts: HashMap<usize, HashMap<(usize,usize),f64>> = HashMap::new();
        parts.insert(0, HashMap::from([((0,0),0.0)]));
        let mut adm = Vec::new();
        let mut idxs: Vec<usize> = (self.min_size..self.n).step_by(self.jump).collect();
        idxs.push(self.n);

        for &bkp in &idxs {
            let new_pt = ((bkp.saturating_sub(self.min_size))/self.jump)*self.jump;
            adm.push(new_pt);
            let mut subs = Vec::new();
            for &t in &adm {
                if let Some(dp) = parts.get(&t) {
                    let mut m = dp.clone();
                    m.insert((t,bkp), self.cost.error(t,bkp) + self.pen);
                    subs.push(m);
                }
            }
            if let Some(best) = subs.into_iter().min_by(|a,b|
                a.values().sum::<f64>().total_cmp(&b.values().sum::<f64>())
            ) {
                let best_sum = best.values().sum::<f64>();
                parts.insert(bkp, best.clone());
                adm.retain(|&t| parts.get(&t)
                    .map(|p| p.values().sum::<f64>() <= best_sum + self.pen)
                    .unwrap_or(false));
            }
        }

        if let Some(final_map) = parts.get(&self.n) {
            let mut fm = final_map.clone();
            fm.remove(&(0,0));
            fm
        } else {
            HashMap::new()
        }
    }

    /// Univariate detection
    fn detect_univariate(&mut self, uni: &[f64]) -> Vec<usize> {
        let rows: Vec<Vec<f64>> = uni.iter().map(|&x| vec![x]).collect();
        self.fit(&rows);
        let part = self.segment();
        let mut cps: Vec<usize> = part.keys().map(|&(_,e)| e).collect();
        cps.sort_unstable();
        cps
    }

    /// Multivariate (row‐means)
    pub fn detect_multivariate(&mut self, data: &[Vec<f64>]) -> Vec<usize> {
        let uni: Vec<f64> = data.iter()
            .map(|r| r.iter().sum::<f64>()/(r.len() as f64))
            .collect();
        self.detect_univariate(&uni)
    }
}

impl ChangePointDetector for PELT {
    fn detect(&mut self, data: &[f64]) -> Vec<usize> {
        self.detect_univariate(data)
    }
    fn set_params(&mut self, params: HashMap<String, f64>) {
        if let Some(&p) = params.get("penalty")  { self.pen      = p; }
        if let Some(&m) = params.get("min_size") { self.min_size = m as usize; }
        if let Some(&j) = params.get("jump")     { self.jump     = j as usize; }
        self.cost.min_size = self.min_size;
    }
    fn reinit(&mut self) { }
}