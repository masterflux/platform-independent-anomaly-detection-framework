use crate::change_point_detector::ChangePointDetector;
use std::collections::HashMap;

pub struct PELT {
    penalty: f64,
    min_size: usize,
    jump: usize,
}

impl PELT {
    pub fn new(penalty: f64, min_size: usize, jump: usize) -> Self {
        Self {
            penalty,
            min_size,
            jump,
        }
    }

    fn cost(&self, data: &[f64], start: usize, end: usize) -> f64 {
        if end <= start + self.min_size {
            return f64::INFINITY;
        }
        
        let segment = &data[start..end];
        let mean = segment.iter().sum::<f64>() / segment.len() as f64;
        segment.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
    }

    fn segment(&self, data: &[f64]) -> HashMap<(usize, usize), f64> {
        let n = data.len();
        let mut partitions: HashMap<usize, HashMap<(usize, usize), f64>> = HashMap::new();
        let mut init_map = HashMap::new();
        init_map.insert((0, 0), 0.0);
        partitions.insert(0, init_map);
        partitions.get_mut(&0).unwrap().insert((0, 0), 0.0);
        
        let mut admissible = Vec::new();
        
        let mut indices = Vec::new();
        let mut k = self.min_size;
        while k < n {
            indices.push(k);
            k += self.jump;
        }
        indices.push(n);
        
        for &bkp in &indices {
            let new_adm_pt = ((bkp.saturating_sub(self.min_size)) / self.jump) * self.jump;
            admissible.push(new_adm_pt);
            
            let mut subproblems = Vec::new();
            
            for &t in &admissible {
                if let Some(partition) = partitions.get(&t) {
                    let mut tmp_partition = partition.clone();
                    tmp_partition.insert((t, bkp), self.cost(data, t, bkp) + self.penalty);
                    subproblems.push(tmp_partition);
                }
            }
            
            if !subproblems.is_empty() {
                let best_partition = subproblems.into_iter()
                    .min_by(|a, b| {
                        let sum_a: f64 = a.values().sum();
                        let sum_b: f64 = b.values().sum();
                        sum_a.partial_cmp(&sum_b).unwrap()
                    })
                    .unwrap();
                
                partitions.insert(bkp, best_partition.clone());
                
                // Trim admissible set
                let best_sum: f64 = best_partition.values().sum();
                admissible.retain(|&t| {
                    if let Some(partition) = partitions.get(&t) {
                        let sum: f64 = partition.values().sum();
                        sum <= best_sum + self.penalty
                    } else {
                        false
                    }
                });
            }
        }
        
        if let Some(best_partition) = partitions.get(&n) {
            let mut result = best_partition.clone();
            result.remove(&(0, 0));
            result
        } else {
            HashMap::new()
        }
    }
}

impl ChangePointDetector for PELT {
    fn detect(&mut self, data: &[f64]) -> Vec<usize> {
        let partition = self.segment(data);
        let mut bkps: Vec<usize> = partition.keys().map(|(_, end)| *end).collect();
        bkps.sort();
        bkps.into_iter().filter(|&x| x < data.len()).collect()
    }

    fn set_params(&mut self, params: HashMap<String, f64>) {
        if let Some(&penalty) = params.get("penalty") {
            self.penalty = penalty;
        }
        if let Some(&min_size) = params.get("min_size") {
            self.min_size = min_size as usize;
        }
        if let Some(&jump) = params.get("jump") {
            self.jump = jump as usize;
        }
    }

    fn reinit(&mut self) {
        // PELT is stateless, no need to reinit
    }
}