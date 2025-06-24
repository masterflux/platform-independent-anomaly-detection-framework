// File: watch_wasm/src/bin/tune_micro_watch.rs

use std::{collections::HashMap, error::Error, fs, path::Path};

// Import the ChangePointDetector trait to bring detector methods into scope
use watch_wasm::change_point_detector::ChangePointDetector;
use watch_wasm::detectors::MicroWatch;
use watch_wasm::utils::load_csv_multi;

fn main() -> Result<(), Box<dyn Error>> {
    let data_dir = Path::new("datasets/csv");

    // define your grid
    let thresh_vals   = vec![0.5, 1.0, 1.5, 2.0];
    let batch_vals    = vec![3_usize, 5, 10];
    let newbuf_vals   = vec![16_usize, 32, 64];
    let maxbuf_factor = 4; // max_dist_size = newbuf * factor

    // aggregate counts for each param combo across all datasets
    // key: (threshold_index, batch, buf, maxbuf)
    let mut totals: HashMap<(usize, usize, usize, usize), usize> = HashMap::new();
    let mut series_count = 0;

    for entry in fs::read_dir(data_dir)? {
        let entry = entry?;
        let path  = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("csv") {
            continue;
        }

        // only univariate
        let data = load_csv_multi(path.to_str().unwrap())?;
        if data.is_empty() || data[0].len() != 1 {
            continue;
        }
        let univ: Vec<f64> = data.iter().map(|r| r[0]).collect();
        series_count += 1;

        // grid search for this series
        for (ti, &th) in thresh_vals.iter().enumerate() {
            for &bs in &batch_vals {
                for &nb in &newbuf_vals {
                    let mut mw = MicroWatch::new(0, th, bs);
                    mw.set_params(HashMap::from([
                        ("new_dist_buffer_size".to_string(), nb as f64),
                        ("max_dist_size".to_string(),        (nb * maxbuf_factor) as f64),
                    ]));
                    mw.reinit();

                    let count = mw.detect(&univ).len();
                    let key = (ti, bs, nb, nb * maxbuf_factor);
                    *totals.entry(key).or_insert(0) += count;
                }
            }
        }
    }

    // find the global best config
    if let Some((&best_cfg, &best_total)) = totals.iter().max_by_key(|&(_, &tot)| tot) {
        let (ti, bs, nb, maxbuf) = best_cfg;
        let th = thresh_vals[ti];
        println!("Scanned {} series", series_count);
        println!(
            "Best overall config: threshold={:.2}, batch={}, buf={}, maxbuf={} => total detected {} cps",
            th, bs, nb, maxbuf, best_total
        );
    }

    Ok(())
}