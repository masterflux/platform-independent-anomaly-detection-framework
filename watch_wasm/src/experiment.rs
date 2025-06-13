// use crate::change_point_detector::ChangePointDetector;
// //use crate::utils::load_csv;
// use crate::detectors::{BOCPD, CUSUM, MicroWatch, PELT, BOCPDMS};

// use serde::Serialize;
// use std::collections::HashSet;
// use std::error::Error;
// use std::fs::{self, File};
// use std::io::{BufRead, BufReader, Write};
// use std::path::{Path, PathBuf};
// use std::time::Instant;

// #[derive(Debug, Serialize)]
// pub struct ExperimentResult {
//     pub dataset:     String,
//     pub algorithm:   String,
//     pub found:       usize,        // ← number of change–points detected
//     pub precision:   Option<f64>,  // ← None if no .truth
//     pub recall:      Option<f64>,
//     pub f1:          Option<f64>,
//     pub duration_ms: u128,
// }

// /// Try to load `dataset.truth`. If missing, return empty Vec.
// fn load_ground_truth(base: &Path) -> Result<Vec<usize>, Box<dyn Error>> {
//     let truth_path = base.with_extension("truth");
//     let file = match File::open(&truth_path) {
//         Ok(f) => f,
//         Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
//         Err(e) => return Err(Box::new(e)),
//     };
//     let reader = BufReader::new(file);
//     let mut truth = Vec::new();
//     for (i, line) in reader.lines().enumerate() {
//         let s = line?.trim().to_string();
//         if s.is_empty() { continue; }
//         if let Ok(idx) = s.parse() {
//             truth.push(idx);
//         } else {
//             eprintln!("Skipping invalid truth line {} in {:?}: {:?}", i+1, base, s);
//         }
//     }
//     Ok(truth)
// }

// /// Given the detector’s guesses and the true labels, compute TP/FP/FN and return (p,r,f1).
// fn compute_metrics(guesses: &[usize], truth: &[usize]) -> (f64, f64, f64) {
//     let tg: HashSet<_> = truth.iter().copied().collect();
//     let gg: HashSet<_> = guesses.iter().copied().collect();
//     let tp = gg.intersection(&tg).count() as f64;
//     let fp = gg.difference(&tg).count() as f64;
//     let fn_ = tg.difference(&gg).count() as f64;

//     let precision = if tp + fp > 0.0 { tp / (tp + fp) } else { 1.0 };
//     let recall    = if tp + fn_ > 0.0 { tp / (tp + fn_) } else { 1.0 };
//     let f1 = if precision + recall > 0.0 {
//         2.0 * precision * recall / (precision + recall)
//     } else { 0.0 };

//     (precision, recall, f1)
// }

// pub fn run_all_experiments(csv_dir: &Path) -> Result<Vec<ExperimentResult>, Box<dyn Error>> {
//     let mut results = Vec::new();

//     for entry in fs::read_dir(csv_dir)? {
//         let path = entry?.path();
//         if path.extension().and_then(|s| s.to_str()) != Some("csv") { continue; }

//         let dataset = path.file_stem().unwrap().to_string_lossy().into_owned();
//         let data = load_csv(path.to_str().unwrap())?;
//         let truth = load_ground_truth(&path)?;

//         // list of detectors to run
//         let mut detectors: Vec<(&str, Box<dyn ChangePointDetector>)> = vec![
//             ("BOCPD",   Box::new(BOCPD::new(0.1, 0.01, 1.0, 0.0))),
//             ("CUSUM",   Box::new(CUSUM::new(30, 0.01))),
//             ("Micro-E", Box::new(MicroWatch::new(0, 0.5, 5))), // Euclid = index 0
//             ("PELT",    Box::new(PELT::new(10.0, 2, 1))),
//             ("BOCPDMS", Box::new(BOCPDMS::new(0.01, 0.01, 10.0))),
//         ];

//         for (name, mut det) in detectors.drain(..) {
//             let start = Instant::now();
//             let cps = det.detect(&data);
//             let duration = start.elapsed().as_millis();

//             let (p, r, f1) = if truth.is_empty() {
//                 (None, None, None)
//             } else {
//                 let (pp, rr, ff) = compute_metrics(&cps, &truth);
//                 (Some(pp), Some(rr), Some(ff))
//             };

//             results.push(ExperimentResult {
//                 dataset:     dataset.clone(),
//                 algorithm:   name.into(),
//                 found:       cps.len(),
//                 precision:   p,
//                 recall:      r,
//                 f1:          f1,
//                 duration_ms: duration,
//             });
//         }
//     }

//     Ok(results)
// }

// fn main() -> Result<(), Box<dyn Error>> {
//     let csv_dir = PathBuf::from("datasets/csv");
//     let results = run_all_experiments(&csv_dir)?;

//     // Write out results.csv
//     let mut wtr = csv::Writer::from_path("results.csv")?;
//     wtr.serialize(("dataset","algorithm","found","precision","recall","f1","duration_ms"))?;
//     for rec in &results {
//         wtr.serialize(rec)?;
//     }
//     wtr.flush()?;

//     println!("✅ Wrote results.csv with {} rows", results.len());
//     Ok(())
// }
