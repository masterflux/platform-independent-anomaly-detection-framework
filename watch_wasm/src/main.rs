
// use std::{collections::HashMap, error::Error, fs, path::Path};
// use serde::Deserialize;
// use csv::ReaderBuilder;
// use csv::Trim;

// use watch_wasm::utils::load_csv_multi;
// use watch_wasm::detectors::{BOCPD, CUSUM, MicroWatch, PELT, BOCPDMS};
// use watch_wasm::change_point_detector::ChangePointDetector;

// #[derive(Deserialize)]
// struct BocpdParams {
//     dataset: String,
//     alpha:   f64,
//     beta:    f64,
//     kappa:   f64,
//     mu:      f64,
// }

// #[derive(Deserialize)]
// struct BocpdmsParams {
//     dataset:   String,
//     intensity: f64,
//     prior_a:   f64,
//     prior_b:   f64,
// }

// #[derive(Deserialize)]
// struct CusumParams {
//     file:     String,
//     twarmup:  f64,
//     plimit:   f64,
// }

// #[derive(Deserialize)]
// struct PeltParams {
//     dataset:  String,
//     penalty:  f64,
//     jump:     usize,
//     min_size: usize,
// }

// /// Load any CSV or TSV of `D: Deserialize` into a HashMap keyed by `key_fn(&D)`.
// fn load_params<D>(
//     path: &str,
//     key_fn: impl Fn(&D) -> String,
// ) -> Result<HashMap<String, D>, Box<dyn Error>>
// where
//     for<'de> D: Deserialize<'de>,
// {
//     let text = fs::read_to_string(path)?;
//     let first_line = text.lines().next().unwrap_or("");
//     let delimiter = if first_line.contains('\t') { b'\t' } else { b',' };

//     let mut rdr = ReaderBuilder::new()
//         .delimiter(delimiter)
//         .trim(Trim::All)
//         .from_reader(text.as_bytes());

//     let mut map = HashMap::new();
//     for result in rdr.deserialize() {
//         let rec: D = result?;
//         map.insert(key_fn(&rec), rec);
//     }
//     Ok(map)
// }

// fn main() -> Result<(), Box<dyn Error>> {
//     // 1. Load best-params CSV/TSV from params/
//     let bocpd_map: HashMap<String, BocpdParams> =
//         load_params::<BocpdParams>("params/params_bocpd_best.csv", |p| p.dataset.clone())?;
//     let bms_map:   HashMap<String, BocpdmsParams> =
//         load_params::<BocpdmsParams>("params/params_bocpdms_best.csv", |p| p.dataset.clone())?;
//     let cusum_map: HashMap<String, CusumParams>   =
//         load_params::<CusumParams>("params/params_cusum_best.csv", |p| p.file.clone())?;
//     let pelt_map:  HashMap<String, PeltParams>    =
//         load_params::<PeltParams>("params/params_pelt_best.csv", |p| p.dataset.clone())?;

//     // Define the generic fallback for BOCPD
//     let default_bocpd = (0.001, 10.0, 0.1, 0.0);

//     // 2. Iterate each CSV dataset
//     let data_dir = Path::new("datasets/csv");
//     for entry in fs::read_dir(data_dir)? {
//         let path = entry?.path();
//         if path.extension().and_then(|s| s.to_str()) != Some("csv") {
//             continue;
//         }

//         let name = path.file_stem().unwrap().to_string_lossy().to_string();
//         let data = load_csv_multi(path.to_str().unwrap())?;
//         let rows = data.len();
//         let cols = data.get(0).map(|r| r.len()).unwrap_or(0);

//         println!("\n=== Dataset: {} ({} rows × {} cols) ===", name, rows, cols);

//         if cols > 1 {
//             // ─── Multivariate ─────────────────────────────────────
//             let mut mw = MicroWatch::new(0, 0.5, 3);

//             // PELT
//             let (penalty, jump, min_size) = pelt_map.get(&name)
//                 .map(|p| (p.penalty, p.jump, p.min_size))
//                 .unwrap_or((100.0, 5, 1));
//             println!("  PELT params:    penalty={}  jump={}  min_size={}", penalty, jump, min_size);
//             let mut pelt = PELT::new(penalty, jump, min_size);

//             // BOCPDMS
//             let (intensity, prior_a, prior_b) = bms_map.get(&name)
//                 .map(|p| (p.intensity, p.prior_a, p.prior_b))
//                 .unwrap_or((5.0, 5.0, 1.5));
//             println!("  BOCPDMS params: intensity={}  prior_a={}  prior_b={}", intensity, prior_a, prior_b);
//             let mut bms = BOCPDMS::new(intensity, prior_a, prior_b);

//             println!("MicroWatch (multivariate) → {:?}", mw.detect_multivariate(&data));
//             println!("PELT       (multivariate) → {:?}", pelt.detect_multivariate(&data));
//             println!("BOCPDMS    (multivariate) → {:?}", bms.detect_multivariate(&data));

//         } else if cols == 1 {
//             // ─── Univariate ──────────────────────────────────────
//             let univ: Vec<f64> = data.iter().map(|r| r[0]).collect();

//             // BOCPD: first with “best” params (if present), else fallback immediately
//             let bp_entry = bocpd_map.get(&name);
//             let (alpha, beta, kappa, mu) = bp_entry
//                 .map(|p| (p.alpha, p.beta, p.kappa, p.mu))
//                 .unwrap_or(default_bocpd);
//             println!("  BOCPD params:  alpha={}  beta={}  kappa={}  mu={}",
//                      alpha, beta, kappa, mu);

//             let mut bocpd = BOCPD::new(alpha, beta, kappa, mu);
//             let mut bocpd_cps = bocpd.detect(&univ);

//             // If we **did** pull from CSV but got no cps → retry with generic defaults
//             if bp_entry.is_some() && bocpd_cps.is_empty() {
//                 println!("    ↳ no change-points found → retrying with default BOCPD params");
//                 let (da, db, dk, dm) = default_bocpd;
//                 println!("      fallback params: alpha={}  beta={}  kappa={}  mu={}",
//                          da, db, dk, dm);
//                 bocpd = BOCPD::new(da, db, dk, dm);
//                 bocpd_cps = bocpd.detect(&univ);
//             }

//             println!("BOCPD                     → {:?}", bocpd_cps);

//             // CUSUM
//             let (twarmup_f, plimit) = cusum_map.get(&name)
//                 .map(|p| (p.twarmup, p.plimit))
//                 .unwrap_or((5.0, 0.1));
//             println!("  CUSUM params:  twarmup={}  plimit={}", twarmup_f as usize, plimit);
//             let mut cusum = CUSUM::new(twarmup_f as usize, plimit);
//             println!("CUSUM                     → {:?}", cusum.detect(&univ));

//             // MicroWatch
//             println!("  MicroWatch params: warmup=0  threshold=0.5  window=3");
//             let mut mw = MicroWatch::new(0, 0.5, 3);
//             println!("MicroWatch                → {:?}", mw.detect(&univ));

//             // PELT
//             let (penalty, jump, min_size) = pelt_map.get(&name)
//                 .map(|p| (p.penalty, p.jump, p.min_size))
//                 .unwrap_or((100.0, 5, 1));
//             println!("  PELT params:    penalty={}  jump={}  min_size={}", penalty, jump, min_size);
//             let mut pelt = PELT::new(penalty, jump, min_size);
//             println!("PELT                      → {:?}", pelt.detect(&univ));

//             // BOCPDMS
//             let (intensity, prior_a, prior_b) = bms_map.get(&name)
//                 .map(|p| (p.intensity, p.prior_a, p.prior_b))
//                 .unwrap_or((5.0, 5.0, 1.5));
//             println!("  BOCPDMS params: intensity={}  prior_a={}  prior_b={}", intensity, prior_a, prior_b);
//             let mut bms = BOCPDMS::new(intensity, prior_a, prior_b);
//             println!("BOCPDMS                   → {:?}", bms.detect(&univ));

//         } else {
//             println!("  (no columns found, skipping {})", name);
//         }
//     }

//     Ok(())
// }









// //////////////////saving result



// File: watch_wasm/src/bin/dump_results.rs

// use std::{collections::HashMap, error::Error, fs, path::Path};
// use serde::Deserialize;
// use csv::{ReaderBuilder, Trim, Writer};
// use watch_wasm::change_point_detector::ChangePointDetector;
// use watch_wasm::utils::load_csv_multi;
// use watch_wasm::detectors::{BOCPD, CUSUM, MicroWatch, PELT, BOCPDMS};

// /// Load any CSV/TSV of `D: Deserialize` into a HashMap keyed by `key_fn(&D)`.
// fn load_params<D>(
//     path: &str,
//     key_fn: impl Fn(&D) -> String,
// ) -> Result<HashMap<String, D>, Box<dyn Error>>
// where
//     for<'de> D: Deserialize<'de>,
// {
//     let text = fs::read_to_string(path)?;
//     let first_line = text.lines().next().unwrap_or("");
//     let delimiter = if first_line.contains('\t') { b'\t' } else { b',' };

//     let mut rdr = ReaderBuilder::new()
//         .delimiter(delimiter)
//         .trim(Trim::All)
//         .from_reader(text.as_bytes());

//     let mut map = HashMap::new();
//     for result in rdr.deserialize() {
//         let rec: D = result?;
//         map.insert(key_fn(&rec), rec);
//     }
//     Ok(map)
// }

// #[derive(Deserialize)]
// struct BocpdParams {
//     dataset: String,
//     alpha:   f64,
//     beta:    f64,
//     kappa:   f64,
//     mu:      f64,
// }

// #[derive(Deserialize)]
// struct BocpdmsParams {
//     dataset:   String,
//     intensity: f64,
//     prior_a:   f64,
//     prior_b:   f64,
// }

// #[derive(Deserialize)]
// struct CusumParams {
//     file:     String,
//     twarmup:  f64,
//     plimit:   f64,
// }

// #[derive(Deserialize)]
// struct PeltParams {
//     dataset:  String,
//     penalty:  f64,
//     jump:     usize,
//     min_size: usize,
// }

// /// Join a list of indices with “;”
// fn vec_to_str(v: &[usize]) -> String {
//     v.iter()
//      .map(|i| i.to_string())
//      .collect::<Vec<_>>()
//      .join(";")
// }

// fn main() -> Result<(), Box<dyn Error>> {
//     // 1. Load best‐params from params/
//     let bocpd_map: HashMap<String, BocpdParams> =
//         load_params("params/params_bocpd_best.csv",   |p: &BocpdParams|  p.dataset.clone())?;
//     let bms_map:   HashMap<String, BocpdmsParams> =
//         load_params("params/params_bocpdms_best.csv",|p: &BocpdmsParams|p.dataset.clone())?;
//     let cusum_map: HashMap<String, CusumParams>   =
//         load_params("params/params_cusum_best.csv",  |p: &CusumParams|  p.file.clone())?;
//     let pelt_map:  HashMap<String, PeltParams>    =
//         load_params("params/params_pelt_best.csv",  |p: &PeltParams|  p.dataset.clone())?;

//     // generic fallback for BOCPD
//     let default_bocpd = (0.001, 0.001, 0.1, 0.0);

//     // ensure results directory exists
//     fs::create_dir_all("results")?;

//     // open results/results.csv and write header
//     let mut wtr = Writer::from_path("results/results.csv")?;
//     wtr.write_record(&[
//         "dataset",
//         "BOCPD",
//         "CUSUM",
//         "MicroWatch",
//         "PELT",
//         "BOCPDMS_univ",
//         "BOCPDMS_multi",
//     ])?;

//     // iterate datasets
//     let data_dir = Path::new("datasets/csv");
//     for entry in fs::read_dir(data_dir)? {
//         let path = entry?.path();
//         if path.extension().and_then(|s| s.to_str()) != Some("csv") {
//             continue;
//         }

//         let name = path.file_stem().unwrap().to_string_lossy().into_owned();
//         let data = load_csv_multi(path.to_str().unwrap())?;
//         let cols = data.get(0).map(|r| r.len()).unwrap_or(0);

//         // placeholders
//         let mut bocpd_lst  = Vec::new();
//         let mut cusum_lst  = Vec::new();
//         let mut mw_lst     = Vec::new();
//         let mut pelt_lst   = Vec::new();
//         let mut bocpdms_u  = Vec::new();
//         let mut bocpdms_m  = Vec::new();

//         if cols > 1 {
//             // ─── multivariate ─────────────────────────────────────
//             mw_lst    = MicroWatch::new(0,   0.5, 3)
//                             .detect_multivariate(&data);
//             pelt_lst  = PELT::new(100.0, 5, 1)
//                             .detect_multivariate(&data);
//             bocpdms_m = BOCPDMS::new(5.0, 5.0, 1.5)
//                             .detect_multivariate(&data);
//         } else if cols == 1 {
//             // ─── univariate ──────────────────────────────────────
//             let univ: Vec<f64> = data.iter().map(|r| r[0]).collect();

//             // BOCPD: try “best” then fallback if empty
//             let bp_entry = bocpd_map.get(&name);
//             let (alpha, beta, kappa, mu) = bp_entry
//                 .map(|p| (p.alpha, p.beta, p.kappa, p.mu))
//                 .unwrap_or(default_bocpd);
//             let mut bocpd = BOCPD::new(alpha, beta, kappa, mu);
//             bocpd_lst = bocpd.detect(&univ);

//             if bp_entry.is_some() && bocpd_lst.is_empty() {
//                 let (da, db, dk, dm) = default_bocpd;
//                 bocpd = BOCPD::new(da, db, dk, dm);
//                 bocpd_lst = bocpd.detect(&univ);
//             }

//             // CUSUM
//             let (twarmup, plimit) = cusum_map.get(&name)
//                 .map(|p| (p.twarmup as usize, p.plimit))
//                 .unwrap_or((5, 0.1));
//             cusum_lst = CUSUM::new(twarmup, plimit).detect(&univ);

//             // MicroWatch
//             mw_lst = MicroWatch::new(0, 0.5, 3).detect(&univ);

//             // PELT
//             let (penalty, jump, min_size) = pelt_map.get(&name)
//                 .map(|p| (p.penalty, p.jump, p.min_size))
//                 .unwrap_or((100.0, 5, 1));
//             pelt_lst = PELT::new(penalty, jump, min_size).detect(&univ);

//             // BOCPDMS (univariate)
//             bocpdms_u = BOCPDMS::new(5.0, 5.0, 1.5).detect(&univ);
//         }

//         // stringify and write row
//         wtr.write_record(&[
//             name.as_str(),
//             &vec_to_str(&bocpd_lst),
//             &vec_to_str(&cusum_lst),
//             &vec_to_str(&mw_lst),
//             &vec_to_str(&pelt_lst),
//             &vec_to_str(&bocpdms_u),
//             &vec_to_str(&bocpdms_m),
//         ])?;
//     }

//     wtr.flush()?;
//     println!("→ results/results.csv written");
//     Ok(())
// }



/////////////////////////////////new for bocpdms/////////////////////////////current working

// src/main.rs

// use std::{collections::HashMap, error::Error, fs, path::Path, process::Command};
// use serde::Deserialize;
// use serde_json;
// use csv::{ReaderBuilder, Trim};

// use watch_wasm::utils::load_csv_multi;
// use watch_wasm::change_point_detector::ChangePointDetector;
// use watch_wasm::detectors::{BOCPD, CUSUM, MicroWatch, PELT};

// #[derive(Deserialize)]
// struct BocpdParams {
//     dataset: String,
//     alpha:   f64,
//     beta:    f64,
//     kappa:   f64,
//     mu:      f64,
// }

// #[derive(Deserialize)]
// struct BocpdmsParams {
//     dataset:   String,
//     intensity: f64,
//     prior_a:   f64,
//     prior_b:   f64,
// }

// #[derive(Deserialize)]
// struct CusumParams {
//     file:     String,
//     twarmup:  f64,
//     plimit:   f64,
// }

// #[derive(Deserialize)]
// struct PeltParams {
//     dataset:  String,
//     penalty:  f64,
//     jump:     usize,
//     min_size: usize,
// }

// /// MicroWatch tuning parameters
// #[derive(Deserialize)]
// struct WatchParams {
//     // rename the CSV’s “file_name” column
//     #[serde(rename = "file_name")]
//     dataset: String,

//     distance_index: usize,
//     batch_size:     usize,

//     // rename the CSV’s “threshold” column
//     #[serde(rename = "threshold")]
//     threshold_ratio:      f64,

//     max_dist_size:        usize,
//     new_dist_buffer_size: usize,
// }

// /// Load any CSV/TSV of `D: Deserialize` into a HashMap keyed by `key_fn(&D)`.
// fn load_params<D>(
//     path: &str,
//     key_fn: impl Fn(&D) -> String,
// ) -> Result<HashMap<String, D>, Box<dyn Error>>
// where
//     for<'de> D: Deserialize<'de>,
// {
//     let text = fs::read_to_string(path)?;
//     let first = text.lines().next().unwrap_or("");
//     let delim = if first.contains('\t') { b'\t' } else { b',' };
//     let mut rdr = ReaderBuilder::new()
//         .delimiter(delim)
//         .trim(Trim::All)
//         .from_reader(text.as_bytes());
//     let mut map = HashMap::new();
//     for rec in rdr.deserialize() {
//         let rec: D = rec?;
//         map.insert(key_fn(&rec), rec);
//     }
//     Ok(map)
// }

// /// Invoke the Python BOCPDMS script (in `scripts/`) and parse its JSON output.
// fn detect_bocpdms_py(csv_path: &str, params_csv: &str) -> Vec<usize> {
//     if cfg!(target_arch = "wasm32") {
//         eprintln!("warning: skipping BOCPDMS under WASM");
//         return Vec::new();
//     }
//     let output = Command::new("python3")
//         .arg("scripts/detect_bocpdms.py")
//         .arg(csv_path)
//         .arg(params_csv)
//         .output()
//         .expect("failed to invoke detect_bocpdms.py");
//     if !output.status.success() {
//         panic!(
//             "bocpdms script failed:\n{}",
//             String::from_utf8_lossy(&output.stderr)
//         );
//     }
//     serde_json::from_slice(&output.stdout)
//         .expect("failed to parse JSON from bocpdms script")
// }

// fn main() -> Result<(), Box<dyn Error>> {
//     // 1) Load all the “best‐params” tables
//     let bocpd_map  = load_params("params/params_bocpd_best.csv",   |p: &BocpdParams|  p.dataset.clone())?;
//     let bms_map    = load_params("params/params_bocpdms_best.csv",|p: &BocpdmsParams|p.dataset.clone())?;
//     let cusum_map  = load_params("params/params_cusum_best.csv",  |p: &CusumParams|  p.file.clone())?;
//     let pelt_map   = load_params("params/params_pelt_best.csv",   |p: &PeltParams|  p.dataset.clone())?;
//     let watch_map  = load_params("params/params_watch_best.csv",  |p: &WatchParams|  p.dataset.clone())?;

//     // Fallback for BOCPD if “best” yields no CPs
//     let default_bocpd = (0.001, 10.0, 0.1, 0.0);

//     // 2) Iterate each CSV under datasets/csv
//     let data_dir = Path::new("datasets/csv");
//     for entry in fs::read_dir(data_dir)? {
//         let path = entry?.path();
//         if path.extension().and_then(|s| s.to_str()) != Some("csv") {
//             continue;
//         }
//         let name = path.file_stem().unwrap().to_string_lossy();
//         let data = load_csv_multi(path.to_str().unwrap())?;
//         let rows = data.len();
//         let cols = data.get(0).map(|r| r.len()).unwrap_or(0);

//         println!("\n=== Dataset: {} ({} rows × {} cols) ===", name, rows, cols);

//         // Prepare MicroWatch
//         // — lookup this dataset’s row in watch_map
//         let mut mw = if let Some(wp) = watch_map.get(&*name) {
//             let mut m = MicroWatch::new(wp.distance_index, 0.0, 1);
//             // build a small f64‐map for set_params
//             let mut pm = HashMap::new();
//             pm.insert("batch_size".to_string(),           wp.batch_size           as f64);
//             pm.insert("threshold_ratio".to_string(),      wp.threshold_ratio);
//             pm.insert("max_dist_size".to_string(),        wp.max_dist_size        as f64);
//             pm.insert("new_dist_buffer_size".to_string(), wp.new_dist_buffer_size as f64);
//             m.set_params(pm);
//             m
//         } else {
//             // fallback defaults
//             MicroWatch::new(0, 0.5, 3)
//         };

//         if cols > 1 {
//             // ── Multivariate ───────────────────────
//             let mut pelt = {
//                 let (pen, j, ms) = pelt_map.get(&*name)
//                     .map(|p| (p.penalty, p.jump, p.min_size))
//                     .unwrap_or((100.0, 5, 1));
//                 println!("  PELT params:    penalty={}  jump={}  min_size={}", pen, j, ms);
//                 PELT::new(pen, j, ms)
//             };

//             println!("MicroWatch (multi) → {:?}", mw.detect_multivariate(&data));
//             println!("PELT       (multi) → {:?}", pelt.detect_multivariate(&data));

//             // Python BOCPDMS
//             let cps = detect_bocpdms_py(path.to_str().unwrap(), "params/params_bocpdms_best.csv");
//             println!("BOCPDMS    (multi) → {:?}", cps);

//         } else {
//             // ── Univariate ─────────────────────────
//             let univ: Vec<f64> = data.iter().map(|r| r[0]).collect();

//             // BOCPD
//             let mut bocpd = {
//                 let (a,b,k,m) = bocpd_map.get(&*name)
//                     .map(|p| (p.alpha, p.beta, p.kappa, p.mu))
//                     .unwrap_or(default_bocpd);
//                 println!("  BOCPD params: alpha={}  beta={}  kappa={}  mu={}", a,b,k,m);
//                 BOCPD::new(a,b,k,m)
//             };
//             let mut cps_b = bocpd.detect(&univ);
//             // fallback if empty
//             if bocpd_map.contains_key(&*name) && cps_b.is_empty() {
//                 let (da,db,dk,dm) = default_bocpd;
//                 bocpd = BOCPD::new(da,db,dk,dm);
//                 cps_b = bocpd.detect(&univ);
//             }
//             println!("BOCPD   → {:?}", cps_b);

//             // CUSUM
//             let (tw, pl) = cusum_map.get(&*name)
//                 .map(|p| (p.twarmup as usize, p.plimit))
//                 .unwrap_or((5, 0.1));
//             println!("  CUSUM params: twarmup={}  plimit={}", tw, pl);
//             println!("CUSUM   → {:?}", CUSUM::new(tw, pl).detect(&univ));

//             // MicroWatch
//             println!("MicroW  → {:?}", mw.detect(&univ));

//             // PELT
//             let pelt_cps = {
//                 let (pen, j, ms) = pelt_map.get(&*name)
//                     .map(|p| (p.penalty, p.jump, p.min_size))
//                     .unwrap_or((100.0, 5, 1));
//                 println!("  PELT params: penalty={}  jump={}  min_size={}", pen, j, ms);
//                 PELT::new(pen, j, ms).detect(&univ)
//             };
//             println!("PELT    → {:?}", pelt_cps);

//             // Python BOCPDMS
//             let cps = detect_bocpdms_py(path.to_str().unwrap(), "params/params_bocpdms_best.csv");
//             println!("BOCPDMS → {:?}", cps);
//         }
//     }
//     Ok(())
// }



//--------------------current working one with ubuntu--------------------------

// use std::{collections::HashMap, error::Error, fs, path::Path, process::Command};
// use serde::Deserialize;
// use serde_json;
// use csv::{ReaderBuilder, Trim};

// use watch_wasm::utils::load_csv_multi;
// use watch_wasm::change_point_detector::ChangePointDetector;
// use watch_wasm::detectors::{BOCPD, CUSUM, MicroWatch, PELT};

// /// Load any CSV/TSV of `D: Deserialize` into a HashMap keyed by `key_fn(&D)`.
// fn load_params<D>(
//     path: &str,
//     key_fn: impl Fn(&D) -> String,
// ) -> Result<HashMap<String, D>, Box<dyn Error>>
// where
//     for<'de> D: Deserialize<'de>,
// {
//     let text = fs::read_to_string(path)?;
//     let first = text.lines().next().unwrap_or("");
//     let delim = if first.contains('\t') { b'\t' } else { b',' };
//     let mut rdr = ReaderBuilder::new()
//         .delimiter(delim)
//         .trim(Trim::All)
//         .from_reader(text.as_bytes());
//     let mut map = HashMap::new();
//     for rec in rdr.deserialize() {
//         let rec: D = rec?;
//         map.insert(key_fn(&rec), rec);
//     }
//     Ok(map)
// }

// /// For the seven MicroWatch rows per dataset
// fn load_params_list<D>(
//     path: &str,
// ) -> Result<Vec<D>, Box<dyn Error>>
// where
//     for<'de> D: Deserialize<'de>,
// {
//     let text = fs::read_to_string(path)?;
//     let first = text.lines().next().unwrap_or("");
//     let delim = if first.contains('\t') { b'\t' } else { b',' };
//     let mut rdr = ReaderBuilder::new()
//         .delimiter(delim)
//         .trim(Trim::All)
//         .from_reader(text.as_bytes());
//     let mut v = Vec::new();
//     for rec in rdr.deserialize() {
//         v.push(rec?);
//     }
//     Ok(v)
// }

// #[derive(Deserialize)]
// struct BocpdParams {
//     dataset: String,
//     alpha:   f64,
//     beta:    f64,
//     kappa:   f64,
//     mu:      f64,
// }

// #[derive(Deserialize)]
// struct BocpdmsParams {
//     dataset:   String,
//     intensity: f64,
//     prior_a:   f64,
//     prior_b:   f64,
// }

// #[derive(Deserialize)]
// struct CusumParams {
//     file:     String,
//     twarmup:  f64,
//     plimit:   f64,
// }

// #[derive(Deserialize)]
// struct PeltParams {
//     dataset:  String,
//     penalty:  f64,
//     jump:     usize,
//     min_size: usize,
// }

// /// MicroWatch tuning parameters (7 rows per dataset)
// #[derive(Deserialize)]
// struct WatchParams {
//     #[serde(rename = "file_name")]
//     dataset: String,

//     distance_index:      usize,
//     batch_size:          usize,
//     #[serde(rename = "threshold")]
//     threshold_ratio:     f64,
//     max_dist_size:       usize,
//     new_dist_buffer_size: usize,
// }

// /// Invoke the Python BOCPDMS script (in `scripts/`) and parse its JSON output.
// fn detect_bocpdms_py(csv_path: &str, params_csv: &str) -> Vec<usize> {
//     if cfg!(target_arch = "wasm32") {
//         eprintln!("warning: skipping BOCPDMS under WASM");
//         return Vec::new();
//     }
//     let output = Command::new("python3")
//         .arg("scripts/detect_bocpdms.py")
//         .arg(csv_path)
//         .arg(params_csv)
//         .output()
//         .expect("failed to invoke detect_bocpdms.py");
//     if !output.status.success() {
//         panic!(
//             "bocpdms script failed:\n{}",
//             String::from_utf8_lossy(&output.stderr)
//         );
//     }
//     serde_json::from_slice(&output.stdout)
//         .expect("failed to parse JSON from bocpdms script")
// }

// fn main() -> Result<(), Box<dyn Error>> {
//     // ─ load your existing best‐params tables ─────
//     let bocpd_map  = load_params("params/params_bocpd_best.csv",    |p: &BocpdParams|   p.dataset.clone())?;
//     let bms_map    = load_params("params/params_bocpdms_best.csv", |p: &BocpdmsParams| p.dataset.clone())?;
//     let cusum_map  = load_params("params/params_cusum_best.csv",   |p: &CusumParams|   p.file.clone())?;
//     let pelt_map   = load_params("params/params_pelt_best.csv",    |p: &PeltParams|    p.dataset.clone())?;

//     // ─ load all 7 MicroWatch rows, then group by dataset ─────────
//     let mut watch_map: HashMap<String, Vec<WatchParams>> = HashMap::new();
//     for wp in load_params_list::<WatchParams>("params/params_watch_best.csv")? {
//         watch_map.entry(wp.dataset.clone())
//                  .or_default()
//                  .push(wp);
//     }

//     // fallback BOCPD
//     let default_bocpd = (0.001, 10.0, 0.1, 0.0);

//     // ─ iterate every CSV under datasets/csv ───────────────────────
//     let data_dir = Path::new("datasets/csv");
//     for entry in fs::read_dir(data_dir)? {
//         let path = entry?.path();
//         if path.extension().and_then(|s| s.to_str()) != Some("csv") {
//             continue;
//         }
//         let name = path.file_stem().unwrap().to_string_lossy();
//         let data = load_csv_multi(path.to_str().unwrap())?;
//         let rows = data.len();
//         let cols = data.get(0).map(|r| r.len()).unwrap_or(0);

//         println!("\n=== Dataset: {} ({} rows × {} cols) ===", name, rows, cols);

//         // ─── MicroWatch: run all distance_index = 0..6 ────────────
//         if let Some(params_list) = watch_map.get(&*name) {
//             if cols > 1 {
//                 // multivariate
//                 for wp in params_list {
//                     let mut mw = MicroWatch::new(wp.distance_index, 0.0, 1);
//                     let mut pm = HashMap::new();
//                     pm.insert("batch_size".to_string(),           wp.batch_size           as f64);
//                     pm.insert("threshold_ratio".to_string(),      wp.threshold_ratio);
//                     pm.insert("max_dist_size".to_string(),        wp.max_dist_size        as f64);
//                     pm.insert("new_dist_buffer_size".to_string(), wp.new_dist_buffer_size as f64);
//                     mw.set_params(pm);

//                     let cps = mw.detect_multivariate(&data);
//                     println!(
//                         "MicroWatch idx={}  (batch_size={}, threshold_ratio={:.4}, max_dist_size={}, new_dist_buffer_size={}) → {:?}",
//                         wp.distance_index,
//                         wp.batch_size,
//                         wp.threshold_ratio,
//                         wp.max_dist_size,
//                         wp.new_dist_buffer_size,
//                         cps
//                     );
//                 }

//                 let mut pelt = {
//                     let (pen, j, ms) = pelt_map.get(&*name)
//                         .map(|p| (p.penalty, p.jump, p.min_size))
//                         .unwrap_or((100.0, 5, 1));
//                     println!("  PELT params: penalty={}  jump={}  min_size={}", pen, j, ms);
//                     PELT::new(pen, j, ms)
//                 };
//                 println!("PELT    (multi) → {:?}", pelt.detect_multivariate(&data));

//                 // Python BOCPDMS
//                 let cps = detect_bocpdms_py(path.to_str().unwrap(), "params/params_bocpdms_best.csv");
//                 println!("BOCPDMS (multi) → {:?}", cps);

//             } else {
//                 // univariate
//                 let univ: Vec<f64> = data.iter().map(|r| r[0]).collect();
//                 for wp in params_list {
//                     let mut mw = MicroWatch::new(wp.distance_index, 0.0, 1);
//                     let mut pm = HashMap::new();
//                     pm.insert("batch_size".to_string(),           wp.batch_size           as f64);
//                     pm.insert("threshold_ratio".to_string(),      wp.threshold_ratio);
//                     pm.insert("max_dist_size".to_string(),        wp.max_dist_size        as f64);
//                     pm.insert("new_dist_buffer_size".to_string(), wp.new_dist_buffer_size as f64);
//                     mw.set_params(pm);

//                     let cps = mw.detect(&univ);
//                     println!(
//                         "MicroWatch idx={}  (batch_size={}, threshold_ratio={:.4}, max_dist_size={}, new_dist_buffer_size={}) → {:?}",
//                         wp.distance_index,
//                         wp.batch_size,
//                         wp.threshold_ratio,
//                         wp.max_dist_size,
//                         wp.new_dist_buffer_size,
//                         cps
//                     );
//                 }
//             }
//         } else {
//             println!("  (no MicroWatch params for `{}`)", name);
//         }

//         // ─── then all your existing detectors ───────────────────────
//         if cols == 1 {
//             let univ: Vec<f64> = data.iter().map(|r| r[0]).collect();

//             // BOCPD
//             let mut bocpd = {
//                 let (a,b,k,m) = bocpd_map.get(&*name)
//                     .map(|p| (p.alpha, p.beta, p.kappa, p.mu))
//                     .unwrap_or(default_bocpd);
//                 BOCPD::new(a,b,k,m)
//             };
//             let mut cps_b = bocpd.detect(&univ);
//             if bocpd_map.contains_key(&*name) && cps_b.is_empty() {
//                 bocpd = BOCPD::new(default_bocpd.0,
//                                    default_bocpd.1,
//                                    default_bocpd.2,
//                                    default_bocpd.3);
//                 cps_b = bocpd.detect(&univ);
//             }
//             println!("BOCPD   → {:?}", cps_b);

//             // CUSUM
//             let (tw,pl) = cusum_map.get(&*name)
//                              .map(|p| (p.twarmup as usize, p.plimit))
//                              .unwrap_or((5,0.1));
//             println!("CUSUM   → {:?}", CUSUM::new(tw,pl).detect(&univ));

//             // PELT
//             let pelt_cps = {
//                 let (pen,j,ms) = pelt_map.get(&*name)
//                     .map(|p| (p.penalty,p.jump,p.min_size))
//                     .unwrap_or((100.0,5,1));
//                 PELT::new(pen,j,ms).detect(&univ)
//             };
//             println!("PELT    → {:?}", pelt_cps);

//             // Python BOCPDMS
//             let bms_cps = detect_bocpdms_py(path.to_str().unwrap(),
//                                             "params/params_bocpdms_best.csv");
//             println!("BOCPDMS → {:?}", bms_cps);
//         }
//     }

//     Ok(())
// }




//saving ubuntu-----------------------------


use std::{collections::HashMap, error::Error, fs, path::Path, process::Command};
use serde::Deserialize;
use serde_json;
use csv::{ReaderBuilder, Trim, WriterBuilder};

use watch_wasm::utils::load_csv_multi;
use watch_wasm::change_point_detector::ChangePointDetector;
use watch_wasm::detectors::{BOCPD, CUSUM, MicroWatch, PELT};

/// Load any CSV/TSV of `D: Deserialize` into a HashMap keyed by `key_fn(&D)`.
fn load_params<D>(
    path: &str,
    key_fn: impl Fn(&D) -> String,
) -> Result<HashMap<String, D>, Box<dyn Error>>
where
    for<'de> D: Deserialize<'de>,
{
    let text = fs::read_to_string(path)?;
    let first = text.lines().next().unwrap_or("");
    let delim = if first.contains('\t') { b'\t' } else { b',' };
    let mut rdr = ReaderBuilder::new()
        .delimiter(delim)
        .trim(Trim::All)
        .from_reader(text.as_bytes());
    let mut map = HashMap::new();
    for rec in rdr.deserialize() {
        let rec: D = rec?;
        map.insert(key_fn(&rec), rec);
    }
    Ok(map)
}

/// Load any CSV/TSV into a Vec for MicroWatch (7 rows per dataset)
fn load_params_list<D>(
    path: &str,
) -> Result<Vec<D>, Box<dyn Error>>
where
    for<'de> D: Deserialize<'de>,
{
    let text = fs::read_to_string(path)?;
    let first = text.lines().next().unwrap_or("");
    let delim = if first.contains('\t') { b'\t' } else { b',' };
    let mut rdr = ReaderBuilder::new()
        .delimiter(delim)
        .trim(Trim::All)
        .from_reader(text.as_bytes());
    let mut v = Vec::new();
    for rec in rdr.deserialize() {
        v.push(rec?);
    }
    Ok(v)
}

#[derive(Deserialize)]
struct BocpdParams { dataset: String, alpha: f64, beta: f64, kappa: f64, mu: f64 }

#[derive(Deserialize)]
struct BocpdmsParams { dataset: String, intensity: f64, prior_a: f64, prior_b: f64 }

#[derive(Deserialize)]
struct CusumParams { file: String, twarmup: f64, plimit: f64 }

#[derive(Deserialize)]
struct PeltParams { dataset: String, penalty: f64, jump: usize, min_size: usize }

#[derive(Deserialize)]
struct WatchParams {
    #[serde(rename = "file_name")]
    dataset: String,
    distance_index: usize,
    batch_size: usize,
    #[serde(rename = "threshold")]
    threshold_ratio: f64,
    max_dist_size: usize,
    new_dist_buffer_size: usize,
}

/// Invoke Python BOCPDMS and parse JSON output
fn detect_bocpdms_py(csv_path: &str, params_csv: &str) -> Vec<usize> {
    if cfg!(target_arch = "wasm32") {
        eprintln!("warning: skipping BOCPDMS under WASM");
        return Vec::new();
    }
    let output = Command::new("python3")
        .arg("scripts/detect_bocpdms.py")
        .arg(csv_path)
        .arg(params_csv)
        .output()
        .expect("failed to invoke detect_bocpdms.py");
    if !output.status.success() {
        panic!("bocpdms script failed:\n{}", String::from_utf8_lossy(&output.stderr));
    }
    serde_json::from_slice(&output.stdout)
        .expect("failed to parse JSON from bocpdms script")
}

fn main() -> Result<(), Box<dyn Error>> {
    // Ensure output directory exists
    fs::create_dir_all("results")?;
    // Prepare CSV writer
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_path("results/results.csv")?;
    // Write header row
    wtr.write_record(&[
        "dataset",
        "method",
        "parameters",
        "change_points"
    ])?;

    // Load parameter tables
    let bocpd_map  = load_params("params/params_bocpd_best.csv", |p: &BocpdParams| p.dataset.clone())?;
    let bms_map    = load_params("params/params_bocpdms_best.csv",|p: &BocpdmsParams| p.dataset.clone())?;
    let cusum_map  = load_params("params/params_cusum_best.csv",  |p: &CusumParams| p.file.clone())?;
    let pelt_map   = load_params("params/params_pelt_best.csv",   |p: &PeltParams| p.dataset.clone())?;
    let mut watch_map: HashMap<String, Vec<WatchParams>> = HashMap::new();
    for wp in load_params_list::<WatchParams>("params/params_watch_best.csv")? {
        watch_map.entry(wp.dataset.clone()).or_default().push(wp);
    }

    let default_bocpd = (0.001, 10.0, 0.1, 0.0);
    let data_dir = Path::new("datasets/csv");

    for entry in fs::read_dir(data_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|s| s.to_str()) != Some("csv") { continue; }
        let name = path.file_stem().unwrap().to_string_lossy().into_owned();
        let data = load_csv_multi(path.to_str().unwrap())?;
        let rows = data.len();
        let cols = data.get(0).map(|r| r.len()).unwrap_or(0);

        // Helper to join change points
        let join_cps = |vec: Vec<usize>| vec.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(";");

        // MicroWatch (multivariate vs univariate)
        if let Some(params_list) = watch_map.get(&name) {
            if cols > 1 {
                for wp in params_list {
                    let mut mw = MicroWatch::new(wp.distance_index, 0.0, 1);
                    let mut pm = HashMap::new();
                    pm.insert("batch_size".into(), wp.batch_size as f64);
                    pm.insert("threshold_ratio".into(), wp.threshold_ratio);
                    pm.insert("max_dist_size".into(), wp.max_dist_size as f64);
                    pm.insert("new_dist_buffer_size".into(), wp.new_dist_buffer_size as f64);
                    mw.set_params(pm.clone());
                    let cps = mw.detect_multivariate(&data);
                    wtr.write_record(&[
                        &name,
                        &format!("MicroWatch(idx={})", wp.distance_index),
                        &format!("{:?}", pm),
                        &join_cps(cps)
                    ])?;
                }
            } else {
                let univ: Vec<f64> = data.iter().map(|r| r[0]).collect();
                for wp in params_list {
                    let mut mw = MicroWatch::new(wp.distance_index, 0.0, 1);
                    let mut pm = HashMap::new();
                    pm.insert("batch_size".into(), wp.batch_size as f64);
                    pm.insert("threshold_ratio".into(), wp.threshold_ratio);
                    pm.insert("max_dist_size".into(), wp.max_dist_size as f64);
                    pm.insert("new_dist_buffer_size".into(), wp.new_dist_buffer_size as f64);
                    mw.set_params(pm.clone());
                    let cps = mw.detect(&univ);
                    wtr.write_record(&[
                        &name,
                        &format!("MicroWatch(idx={})", wp.distance_index),
                        &format!("{:?}", pm),
                        &join_cps(cps)
                    ])?;
                }
            }
        }

        // Other detectors for univariate or multivariate
        if cols > 1 {
            // PELT multivariate
            let (pen,j,ms) = pelt_map.get(&name).map(|p| (p.penalty,p.jump,p.min_size)).unwrap_or((100.0,5,1));
            let mut pelt = PELT::new(pen, j, ms);
            let cps = pelt.detect_multivariate(&data);
            wtr.write_record(&[&name, "PELT(multi)", &format!("penalty={},jump={},min_size={}",pen,j,ms), &join_cps(cps)])?;

            // BOCPDMS multivariate
            let cps = detect_bocpdms_py(path.to_str().unwrap(), "params/params_bocpdms_best.csv");
            wtr.write_record(&[&name, "BOCPDMS(multi)", "python-script", &join_cps(cps)])?;
        } else {
            let univ: Vec<f64> = data.iter().map(|r| r[0]).collect();

            // BOCPD
            let (a,b,k,m) = bocpd_map.get(&name).map(|p| (p.alpha,p.beta,p.kappa,p.mu)).unwrap_or(default_bocpd);
            let mut bocpd = BOCPD::new(a,b,k,m);
            let mut cps_b = bocpd.detect(&univ);
            if bocpd_map.contains_key(&name) && cps_b.is_empty() {
                bocpd = BOCPD::new(default_bocpd.0,default_bocpd.1,default_bocpd.2,default_bocpd.3);
                cps_b = bocpd.detect(&univ);
            }
            wtr.write_record(&[&name, "BOCPD", &format!("alpha={},beta={},kappa={},mu={}",a,b,k,m), &join_cps(cps_b)])?;

            // CUSUM
            let (tw,pl) = cusum_map.get(&name).map(|p| (p.twarmup as usize, p.plimit)).unwrap_or((5,0.1));
            let cps_c = CUSUM::new(tw,pl).detect(&univ);
            wtr.write_record(&[&name, "CUSUM", &format!("twarmup={},plimit={}",tw,pl), &join_cps(cps_c)])?;

            // PELT
            let (pen,j,ms) = pelt_map.get(&name).map(|p| (p.penalty,p.jump,p.min_size)).unwrap_or((100.0,5,1));
            let cps_p = PELT::new(pen,j,ms).detect(&univ);
            wtr.write_record(&[&name, "PELT", &format!("penalty={},jump={},min_size={}",pen,j,ms), &join_cps(cps_p)])?;

            // BOCPDMS
            let bms_cps = detect_bocpdms_py(path.to_str().unwrap(), "params/params_bocpdms_best.csv");
            wtr.write_record(&[&name, "BOCPDMS", "python-script", &join_cps(bms_cps)])?;
        }
    }

    // Flush writer
    wtr.flush()?;
    Ok(())
}
