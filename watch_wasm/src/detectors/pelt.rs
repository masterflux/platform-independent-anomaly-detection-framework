use crate::change_point_detector::ChangePointDetector;
use std::collections::HashMap;

pub struct PELT {
    penalty: f64,
    min_size: usize,
    jump: usize,
}

impl PELT {
    pub fn new(penalty: f64, min_size: usize, jump: usize) -> Self {
        Self { penalty, min_size, jump }
    }

    fn cost(&self, data: &[f64], start: usize, end: usize) -> f64 {
        if end <= start + self.min_size {
            return f64::INFINITY;
        }
        let segment = &data[start..end];
        let mean = segment.iter().copied().sum::<f64>() / segment.len() as f64;
        segment.iter().map(|x| (x - mean).powi(2)).sum()
    }

    fn segment(&self, data: &[f64]) -> HashMap<(usize, usize), f64> {
        let n = data.len();

        // Initialize with segment (0,0) cost=0
        let mut partitions: HashMap<usize, HashMap<(usize, usize), f64>> = HashMap::new();
        let mut init_map = HashMap::new();
        init_map.insert((0, 0), 0.0);
        partitions.insert(0, init_map);

        let mut admissible = Vec::new();

        // Candidate breakpoints
        let mut indices = Vec::new();
        let mut k = self.min_size;
        while k < n {
            indices.push(k);
            k += self.jump;
        }
        indices.push(n);

        for &bkp in &indices {
            let new_pt = ((bkp.saturating_sub(self.min_size)) / self.jump) * self.jump;
            admissible.push(new_pt);

            // Build subproblems
            let mut subs = Vec::new();
            for &t in &admissible {
                if let Some(part) = partitions.get(&t) {
                    let mut tmp = part.clone();
                    tmp.insert((t, bkp), self.cost(data, t, bkp) + self.penalty);
                    subs.push(tmp);
                }
            }

            // Pick best via total_cmp (no panic)
            if let Some(best) = subs
                .into_iter()
                .min_by(|a, b| {
                    let sa: f64 = a.values().sum();
                    let sb: f64 = b.values().sum();
                    sa.total_cmp(&sb)
                })
            {
                partitions.insert(bkp, best.clone());
                let best_sum: f64 = best.values().sum();
                admissible.retain(|&t| {
                    partitions
                        .get(&t)
                        .map(|p| p.values().sum::<f64>() <= best_sum + self.penalty)
                        .unwrap_or(false)
                });
            }
        }

        // Return the map for final index n, dropping (0,0)
        if let Some(best) = partitions.get(&n) {
            let mut result = best.clone();
            result.remove(&(0, 0));
            result
        } else {
            HashMap::new()
        }
    }
}

impl ChangePointDetector for PELT {
    fn detect(&mut self, data: &[f64]) -> Vec<usize> {
        let part = self.segment(data);
        let mut bkps: Vec<usize> = part.keys().map(|&(_, end)| end).collect();
        bkps.sort_unstable();
        bkps.into_iter().filter(|&x| x < data.len()).collect()
    }

    fn set_params(&mut self, params: HashMap<String, f64>) {
        if let Some(&p) = params.get("penalty") {
            self.penalty = p;
        }
        if let Some(&m) = params.get("min_size") {
            self.min_size = m as usize;
        }
        if let Some(&j) = params.get("jump") {
            self.jump = j as usize;
        }
    }

    fn reinit(&mut self) {
        // PELT is stateless
    }
}
