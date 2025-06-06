use std::cmp::min;
use std::time::Instant;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::error::Error;

fn wasserstein_distance(mut a: Vec<f64>, mut b: Vec<f64>) -> f64 {
    a.sort_by(|x, y| x.partial_cmp(y).unwrap());
    b.sort_by(|x, y| x.partial_cmp(y).unwrap());
    let len = min(a.len(), b.len());
    a.iter()
     .take(len)
     .zip(b.iter().take(len))
     .map(|(x, y)| (x - y).abs())
     .sum::<f64>()
     / len as f64
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

fn load_csv(path: &str) -> Result<Vec<f64>, Box<dyn Error>> {
    let file = File::open(path)?;
    println!("File opened successfully!");

    let reader = BufReader::new(file);
    let mut data = Vec::new();
    for line in reader.lines() {
        let val: f64 = line?.trim().parse()?;
        data.push(val);
    }
    Ok(data)
}

fn main() {
    let start = Instant::now();

    println!("Attempting to open input.csv...");
    let data = load_csv("input.csv").expect("Unable to open CSV file");

    let duration = start.elapsed();
    println!("Execution time: {:?}", duration);

    let result = detect_watch(&data, 3.0, 5);
    println!("Change points: {:?}", result);
}
