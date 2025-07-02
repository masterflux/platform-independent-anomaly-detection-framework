// src/detectors/micro_watch.rs

use crate::distance_measures::DistanceMeasures;
use crate::change_point_detector::ChangePointDetector;
use std::collections::HashMap;

/// MicroWatch: batch‐based distance‐to‐mean change point detector
pub struct MicroWatch {
    threshold_ratio:      f64,
    max_dist_size:        usize,
    new_dist_buffer_size: usize,
    batch_size:           usize,
    distance_metric:      fn(&[f64], &[f64]) -> f64,
    is_creating_new_dist: bool,
    dist_buffer:          Vec<f64>,
    sum:                  f64,
    dist_len:             usize,
    threshold:            f64,
}

impl MicroWatch {
    /// distance_index: which metric to use (0..6)
    /// threshold_ratio: multiplier for max distance in initial buffer
    /// batch_size: how many points per “window”
    pub fn new(distance_index: usize, threshold_ratio: f64, batch_size: usize) -> Self {
        Self {
            threshold_ratio,
            max_dist_size:        72,
            new_dist_buffer_size: 32,
            batch_size,
            distance_metric:      DistanceMeasures::get_distance_function(distance_index),
            is_creating_new_dist: true,
            dist_buffer:          Vec::new(),
            sum:                  0.0,
            dist_len:             0,
<<<<<<< HEAD
            threshold:            0.0, // ← start at zero
=======
            threshold:            0.51,
>>>>>>> d719506060314435b622b74a3c80976d99a80752
        }
    }

    fn distance_to_mean(&self, batch: &[f64], mean: f64) -> f64 {
        let mean_vec = vec![mean; batch.len()];
        (self.distance_metric)(batch, &mean_vec)
    }

<<<<<<< HEAD
    /// Collapse each row to a mean and run univariate detector
=======
    /// NEW: collapse multivariate series into a univariate by row‐means and reuse `detect`
>>>>>>> d719506060314435b622b74a3c80976d99a80752
    pub fn detect_multivariate(&mut self, data: &[Vec<f64>]) -> Vec<usize> {
        let univ: Vec<f64> = data
            .iter()
            .map(|row| row.iter().sum::<f64>() / (row.len() as f64))
            .collect();
        self.detect(&univ)
    }
}

impl ChangePointDetector for MicroWatch {
    fn detect(&mut self, data: &[f64]) -> Vec<usize> {
        let mut change_points = Vec::new();
        let mut i = 0;

<<<<<<< HEAD
        // ← iterate until we've covered the entire series, including the last partial batch
        while i < data.len() {
            let end = (i + self.batch_size).min(data.len());
            let batch = &data[i..end];

            if self.is_creating_new_dist {
                // Build up the “reference” distribution
                self.dist_buffer.extend_from_slice(batch);
                self.dist_len += batch.len();
                self.sum += batch.iter().copied().sum::<f64>();
=======
        while i + self.batch_size <= data.len() {
            let batch = &data[i..i + self.batch_size];

            if self.is_creating_new_dist {
                // building initial distribution
                self.dist_buffer.extend_from_slice(batch);
                self.dist_len += batch.len();
                self.sum += batch.iter().sum::<f64>();
>>>>>>> d719506060314435b622b74a3c80976d99a80752

                if self.dist_len >= self.new_dist_buffer_size {
                    self.is_creating_new_dist = false;
                    let dist_mean = self.sum / (self.dist_len as f64);

<<<<<<< HEAD
                    // compute max distance of all full‐size chunks in buffer
                    let mut max_dist = 0.0;
                    for chunk in self.dist_buffer.chunks(self.batch_size) {
                        if chunk.len() == self.batch_size {
                            let d = self.distance_to_mean(chunk, dist_mean);
                            if d > max_dist { max_dist = d; }
=======
                    // compute initial threshold
                    let mut max_dist: f64 = 0.0;
                    for chunk in self.dist_buffer.chunks(self.batch_size) {
                        if chunk.len() == self.batch_size {
                            let d = self.distance_to_mean(chunk, dist_mean);
                            max_dist = max_dist.max(d);
>>>>>>> d719506060314435b622b74a3c80976d99a80752
                        }
                    }
                    self.threshold = max_dist * self.threshold_ratio;
                }
            } else {
<<<<<<< HEAD
                // compare each incoming batch to “reference” mean
=======
                // test for change
>>>>>>> d719506060314435b622b74a3c80976d99a80752
                let dist_mean = self.sum / (self.dist_len as f64);
                let d = self.distance_to_mean(batch, dist_mean);
                if d > self.threshold {
                    change_points.push(i);
<<<<<<< HEAD

                    // reset to rebuild reference
=======
                    // reset distribution creation
>>>>>>> d719506060314435b622b74a3c80976d99a80752
                    self.is_creating_new_dist = true;
                    self.dist_buffer.clear();
                    self.dist_len = 0;
                    self.sum = 0.0;
                    self.threshold = 0.0;
                }

<<<<<<< HEAD
                // also keep growing the reference buffer (up to max_dist_size)
                if self.dist_len < self.max_dist_size {
                    self.dist_buffer.extend_from_slice(batch);
                    self.dist_len += batch.len();
                    self.sum += batch.iter().copied().sum::<f64>();

                    let dist_mean = self.sum / (self.dist_len as f64);
                    let mut max_dist = 0.0;
                    for chunk in self.dist_buffer.chunks(self.batch_size) {
                        if chunk.len() == self.batch_size {
                            let d = self.distance_to_mean(chunk, dist_mean);
                            if d > max_dist { max_dist = d; }
=======
                // update rolling distribution
                if self.dist_len < self.max_dist_size {
                    self.dist_buffer.extend_from_slice(batch);
                    self.dist_len += batch.len();
                    self.sum += batch.iter().sum::<f64>();

                    let dist_mean = self.sum / (self.dist_len as f64);
                    let mut max_dist: f64 = 0.0;
                    for chunk in self.dist_buffer.chunks(self.batch_size) {
                        if chunk.len() == self.batch_size {
                            let d = self.distance_to_mean(chunk, dist_mean);
                            max_dist = max_dist.max(d);
>>>>>>> d719506060314435b622b74a3c80976d99a80752
                        }
                    }
                    self.threshold = max_dist * self.threshold_ratio;
                }
            }

            i += self.batch_size;
        }

        change_points
    }

    /// Call this with a HashMap<String,f64> built from your tuned‐params CSV:
    /// e.g. { "batch_size": 8.0, "threshold_ratio": 0.45, ... }
    fn set_params(&mut self, params: HashMap<String, f64>) {
<<<<<<< HEAD
        if let Some(&tr)  = params.get("threshold_ratio")      { self.threshold_ratio      = tr;  }
        if let Some(&mds) = params.get("max_dist_size")        { self.max_dist_size        = mds as usize; }
        if let Some(&ndb) = params.get("new_dist_buffer_size") { self.new_dist_buffer_size = ndb as usize; }
        if let Some(&bs)  = params.get("batch_size")           { self.batch_size           = bs  as usize; }
=======
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
>>>>>>> d719506060314435b622b74a3c80976d99a80752
    }

    fn reinit(&mut self) {
        self.is_creating_new_dist = true;
        self.dist_buffer.clear();
        self.dist_len = 0;
        self.sum = 0.0;
        self.threshold = 0.0;
    }
}
