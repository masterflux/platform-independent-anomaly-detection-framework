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

   pub fn chebyshev_min(u: &[f64], v: &[f64]) -> f64 {
    u.iter()
     .zip(v.iter())
     .map(|(a, b)| (a - b).abs())
     .fold(f64::INFINITY, f64::min)
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
    -((u.iter()
        .zip(v.iter())
        .map(|(a, b)| (a * b).sqrt())
        .sum::<f64>())
        .ln())
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
            0 => Self::bhattacharyya, //2 //3 on paper
            1 => Self::chebyshev_min, // 6 //7 on paper
            2 => Self::euclidean, // 10 //11 on paper
            3 => Self::hellinger, // 13
            4 => Self::jensen_shannon_divergence, // 16
            5 => Self::kl_divergence, // 18
            6 => Self::manhattan, // 21
            _ => Self::euclidean, // default
        }
    }
}