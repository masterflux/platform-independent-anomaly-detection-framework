use std::cmp::min;
use std::time::Instant;


fn wasserstein_distance(mut a: Vec<f64>, mut b: Vec<f64>) -> f64 {
    a.sort_by(|x, y| x.partial_cmp(y).unwrap());
    b.sort_by(|x, y| x.partial_cmp(y).unwrap());
    let len = min(a.len(), b.len());
    a.iter().take(len).zip(b.iter().take(len)).map(|(x, y)| (x - y).abs()).sum::<f64>() / len as f64
}

fn detect_watch(data: &[f64], threshold_ratio: f64, batch_size: usize) -> Vec<usize> {
    let mut change_points = Vec::new();
    let mut reference: Vec<f64> = data[..batch_size * 3].to_vec();

    let mut threshold: f64 = 0.0;
    for i in 0..3 {
        let batch = &reference[i * batch_size..(i + 1) * batch_size];
        threshold = threshold.max(wasserstein_distance(batch.to_vec(), reference.clone()));
    }
    threshold *= threshold_ratio;

    let mut ref_data = reference.clone();
    let mut i = batch_size * 3;
    while i + batch_size <= data.len() {
        let batch = &data[i..i + batch_size];
        let dist = wasserstein_distance(batch.to_vec(), ref_data.clone());
        if dist > threshold {
            change_points.push(i);
            ref_data.clear();
        }
        ref_data.extend_from_slice(batch);
        i += batch_size;
    }
    change_points
}

fn rand_normal(mean: f64, stddev: f64) -> f64 {
    use rand_distr::{Distribution, Normal};
    let normal = Normal::new(mean, stddev).unwrap();
    normal.sample(&mut rand::thread_rng())
}

fn main() {
    let start = Instant::now();
    let mut data = Vec::new();
   // Segment 1: mean 0
for _ in 0..30 {
    data.push(rand_normal(0.0, 1.0));
}
// Segment 2: mean 5
for _ in 0..30 {
    data.push(rand_normal(5.0, 1.0));
}
// Segment 3: mean -3
for _ in 0..30 {
    data.push(rand_normal(-3.0, 1.0));
}
// Segment 4: mean 2
for _ in 0..30 {
    data.push(rand_normal(2.0, 1.0));
}
    let duration = start.elapsed();
    println!("Execution time: {:?}", duration);

    let result = detect_watch(&data, 3.0, 5);
    println!("Change points: {:?}", result);

}
