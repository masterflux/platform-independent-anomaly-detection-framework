// File: watch_wasm/src/bin/tune_micro_watch.rs

use std::{collections::HashMap, error::Error, fs, path::Path};

use watch_wasm::change_point_detector::ChangePointDetector;
use watch_wasm::detectors::MicroWatch;
use watch_wasm::utils::load_csv_multi;
use watch_wasm::utils::metrics::{load_annotations, scores};

fn main() -> Result<(), Box<dyn Error>> {
    let data_dir = Path::new("datasets/csv");
    let annotations_file = "./annotations.json";

    // define your grid
    let thresh_vals   = vec![0.5, 1.0, 1.5, 2.0];
    let batch_vals    = vec![3_usize, 5, 10];
    let newbuf_vals   = vec![16_usize, 32, 64];
    let maxbuf_factor = 4; // max_dist_size = newbuf * factor

    for entry in fs::read_dir(data_dir)? {
        let entry = entry?;
        let path  = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("csv") {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy();

        // only univariate for now
        let data = load_csv_multi(path.to_str().unwrap())?;
        if data.is_empty() || data[0].len() != 1 {
            continue;
        }
        let univ: Vec<f64> = data.iter().map(|r| r[0]).collect();
        let n_obs = univ.len();

        // load GT
        let annotations = load_annotations(annotations_file, &name);

        let mut best_f1  = 0.0;
        let mut best_cov = 0.0;
        let mut best_cfg = HashMap::new();

        for &th in &thresh_vals {
            for &bs in &batch_vals {
                for &nb in &newbuf_vals {
                    // new detector
                    let mut mw = MicroWatch::new(0, th, bs);
                    // override its other two params
                    mw.set_params(HashMap::from([
                        ("new_dist_buffer_size".to_string(), nb as f64),
                        ("max_dist_size".to_string(),     (nb*maxbuf_factor) as f64),
                    ]));
                    mw.reinit();

                    let preds = mw.detect(&univ);
                    let (f1, cov) = scores(&preds, &name, n_obs, annotations_file);

                    if f1 > best_f1 {
                        best_f1  = f1;
                        best_cov = cov;
                        best_cfg = HashMap::from([
                            ("threshold_ratio".to_string(),       th),
                            ("batch_size".to_string(),           bs as f64),
                            ("new_dist_buffer_size".to_string(), nb as f64),
                            ("max_dist_size".to_string(),       (nb*maxbuf_factor) as f64),
                        ]);
                    }
                }
            }
        }

        println!("\n=== Tuning {} ({} samples) ===", name, n_obs);
        println!(" Best F1   = {:.4}", best_f1);
        println!(" Best Cover= {:.4}", best_cov);
        println!(" Best cfg  = {:#?}", best_cfg);
    }

    Ok(())
}
