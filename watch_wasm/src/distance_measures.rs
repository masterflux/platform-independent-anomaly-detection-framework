use std::cmp::min;

pub struct DistanceMeasures;

impl DistanceMeasures {
    pub fn euclidean(u: &[f64], v: &[f64]) -> f64 {
        u.iter()
         .zip(v.iter())
         .map(|(a, b)| (a - b).powi(2))
         .sum::<f64>()
         .sqrt()
    }

    pub fn manhattan(u: &[f64], v: &[f64]) -> f64 {
        u.iter()
         .zip(v.iter())
         .map(|(a, b)| (a - b).abs())
         .sum::<f64>()
    }

    pub fn chebyshev(u: &[f64], v: &[f64]) -> f64 {
        u.iter()
         .zip(v.iter())
         .map(|(a, b)| (a - b).abs())
         .fold(0.0, f64::max)
    }

    pub fn kl_divergence(u: &[f64], v: &[f64]) -> f64 {
        const EPSILON: f64 = 1e-10;
        u.iter()
         .zip(v.iter())
         .map(|(a, b)| {
             let a_safe = a.max(EPSILON);
             let b_safe = b.max(EPSILON);
             a_safe * (a_safe / b_safe).ln()
         })
         .sum::<f64>()
    }

    pub fn jensen_shannon_divergence(u: &[f64], v: &[f64]) -> f64 {
        const EPSILON: f64 = 1e-10;
        u.iter()
         .zip(v.iter())
         .map(|(a, b)| {
             let a_safe = a.max(EPSILON);
             let b_safe = b.max(EPSILON);
             let _m = (a_safe + b_safe) / 2.0;
             let dl = a_safe * (2.0 * a_safe / (a_safe + b_safe)).ln();
             let dr = b_safe * (2.0 * b_safe / (a_safe + b_safe)).ln();
             (dl + dr) / 2.0
         })
         .sum::<f64>()
    }

    pub fn bhattacharyya(u: &[f64], v: &[f64]) -> f64 {
        -u.iter()
          .zip(v.iter())
          .map(|(a, b)| (a * b).sqrt())
          .sum::<f64>()
          .ln()
    }

    pub fn hellinger(u: &[f64], v: &[f64]) -> f64 {
        (2.0 * u.iter()
              .zip(v.iter())
              .map(|(a, b)| (a.sqrt() - b.sqrt()).powi(2))
              .sum::<f64>())
        .sqrt()
    }

    pub fn wasserstein(mut u: Vec<f64>, mut v: Vec<f64>) -> f64 {
        u.sort_by(|x, y| x.partial_cmp(y).unwrap());
        v.sort_by(|x, y| x.partial_cmp(y).unwrap());
        let len = min(u.len(), v.len());
        u.iter()
         .take(len)
         .zip(v.iter().take(len))
         .map(|(x, y)| (x - y).abs())
         .sum::<f64>()
         / len as f64
    }

    pub fn get_distance_function(index: usize) -> fn(&[f64], &[f64]) -> f64 {
        match index {
            0 => Self::euclidean,
            1 => Self::manhattan,
            2 => Self::chebyshev,
            3 => Self::kl_divergence,
            4 => Self::jensen_shannon_divergence,
            5 => Self::bhattacharyya,
            6 => Self::hellinger,
            _ => Self::euclidean, // default
        }
    }
}