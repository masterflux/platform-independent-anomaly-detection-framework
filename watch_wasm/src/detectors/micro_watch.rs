use crate::distance_measures::DistanceMeasures;
use crate::change_point_detector::ChangePointDetector;
use std::collections::HashMap;

pub struct MicroWatch {
    threshold_ratio:      f64,
    max_dist_size:        usize,
    new_dist_buffer_size: usize,
    batch_size:           usize,
    distance_metric:      fn(&[f64], &[f64]) -> f64,
    is_creating_new_dist: bool,
    dist_buffer:          Vec<Vec<f64>>, 
    sum:                  Vec<f64>,      
    dist_len:             usize,
    threshold:            f64,
}

impl MicroWatch {
    pub fn new(distance_index: usize, threshold_ratio: f64, batch_size: usize) -> Self {
        Self {
            threshold_ratio,
            max_dist_size: 72,
            new_dist_buffer_size: 32,
            batch_size,
            distance_metric: DistanceMeasures::get_distance_function(distance_index),
            is_creating_new_dist: true,
            dist_buffer: Vec::new(),
            sum: Vec::new(),
            dist_len: 0,
            threshold: 0.0,
        }
    }

    fn distance_to_mean(&self, batch: &[Vec<f64>], mean: &[f64]) -> f64 {
        let flat_batch: Vec<f64> = batch.iter().flatten().cloned().collect();
        let mean_repeated: Vec<f64> = mean
            .iter()
            .cloned()
            .cycle()
            .take(batch.len() * mean.len())
            .collect();
        (self.distance_metric)(&flat_batch, &mean_repeated)
    }

    fn compute_mean(&self) -> Vec<f64> {
        if self.dist_len == 0 {
            return vec![0.0; self.sum.len()];
        }
        self.sum.iter().map(|s| s / self.dist_len as f64).collect()
    }

    
    fn detect_internal(&mut self, data: &[Vec<f64>]) -> Vec<usize> {
        let mut change_points = Vec::new();
        let mut i = 0;

        if self.sum.is_empty() && !data.is_empty() {
            self.sum = vec![0.0; data[0].len()];
        }

        while i < data.len() {
            let end = (i + self.batch_size).min(data.len());
            let batch = &data[i..end];

            if self.is_creating_new_dist {
                self.dist_buffer.extend_from_slice(batch);
                self.dist_len += batch.len();
                for row in batch {
                    for (k, val) in row.iter().enumerate() {
                        self.sum[k] += *val;
                    }
                }

                if self.dist_len >= self.new_dist_buffer_size {
                    self.is_creating_new_dist = false;
                    let mean = self.compute_mean();
                    let mut max_dist = 0.0;
                    for chunk in self.dist_buffer.chunks(self.batch_size) {
                        if chunk.len() == self.batch_size {
                            let d = self.distance_to_mean(chunk, &mean);
                            if d > max_dist {
                                max_dist = d;
                            }
                        }
                    }
                    self.threshold = max_dist * self.threshold_ratio;
                }
            } else {
                let mean = self.compute_mean();
                let d = self.distance_to_mean(batch, &mean);
                if d > self.threshold {
                    change_points.push(i);

                    
                    self.is_creating_new_dist = true;
                    self.dist_buffer.clear();
                    self.sum.fill(0.0);
                    self.dist_len = 0;
                    self.threshold = 0.0;
                }

                if self.dist_len < self.max_dist_size {
                    self.dist_buffer.extend_from_slice(batch);
                    self.dist_len += batch.len();
                    for row in batch {
                        for (k, val) in row.iter().enumerate() {
                            self.sum[k] += *val;
                        }
                    }
                    let mean = self.compute_mean();
                    let mut max_dist = 0.0;
                    for chunk in self.dist_buffer.chunks(self.batch_size) {
                        if chunk.len() == self.batch_size {
                            let d = self.distance_to_mean(chunk, &mean);
                            if d > max_dist {
                                max_dist = d;
                            }
                        }
                    }
                    self.threshold = max_dist * self.threshold_ratio;
                }
            }

            i += self.batch_size;
        }

        change_points
    }

   
    pub fn detect_multivariate(&mut self, data: &[Vec<f64>]) -> Vec<usize> {
        self.detect_internal(data)
    }
}

impl ChangePointDetector for MicroWatch {
    fn detect(&mut self, data: &[f64]) -> Vec<usize> {
    
        let wrapped: Vec<Vec<f64>> = data.iter().map(|v| vec![*v]).collect();
        self.detect_internal(&wrapped)
    }

    fn set_params(&mut self, params: HashMap<String, f64>) {
        if let Some(&tr) = params.get("threshold_ratio") {
            self.threshold_ratio = tr;
        }
        if let Some(&mds) = params.get("max_dist_size") {
            self.max_dist_size = mds as usize;
        }
        if let Some(&ndb) = params.get("new_dist_buffer_size") {
            self.new_dist_buffer_size = ndb as usize;
        }
        if let Some(&bs) = params.get("batch_size") {
            self.batch_size = bs as usize;
        }
    }

    fn reinit(&mut self) {
        self.is_creating_new_dist = true;
        self.dist_buffer.clear();
        self.sum.fill(0.0);
        self.dist_len = 0;
        self.threshold = 0.0;
    }
}
