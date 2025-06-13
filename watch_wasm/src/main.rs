use std::{error::Error, fs, path::Path};
use watch_wasm::change_point_detector::ChangePointDetector;
use watch_wasm::utils::load_csv_multi;
use watch_wasm::detectors::{BOCPD, CUSUM, MicroWatch, PELT, BOCPDMS};

fn main() -> Result<(), Box<dyn Error>> {
    let data_dir = Path::new("datasets/csv");
    for entry in fs::read_dir(data_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("csv") {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy();
        let data = load_csv_multi(path.to_str().unwrap())?;
        let rows = data.len();
        let cols = data.get(0).map(|r| r.len()).unwrap_or(0);

        println!("\n=== Dataset: {} ({} rows × {} cols) ===", name, rows, cols);

        if cols > 1 {
            // Multivariate: only BOCPDMS.multivariate
            let mut bocpdms = BOCPDMS::new(0.01, 0.01, 3.0);
            let cps = bocpdms.detect_multivariate(&data);
            println!("BOCPDMS (multivariate) → {:?}", cps);
        } else if cols == 1 {
            // Univariate: extract column and run all detectors
            let univ: Vec<f64> = data.iter().map(|r| r[0]).collect();

            // 1) BOCPD
            let mut bocpd = BOCPD::new(0.1, 0.01, 1.0, 0.0);
            println!("BOCPD   → {:?}", bocpd.detect(&univ));

            // 2) CUSUM
            let mut cusum = CUSUM::new(30, 0.01);
            println!("CUSUM   → {:?}", cusum.detect(&univ));

            // 3) MicroWatch (Euclidean)
            let mut mw = MicroWatch::new(0, 0.5, 5);
            println!("Micro-E → {:?}", mw.detect(&univ));

            // 4) PELT
            let mut pelt = PELT::new(10.0, 2, 1);
            println!("PELT    → {:?}", pelt.detect(&univ));

            // 5) BOCPDMS (univariate)
            let mut bocpdms = BOCPDMS::new(0.01, 0.01, 3.0);
            println!("BOCPDMS → {:?}", bocpdms.detect(&univ));
        } else {
            println!("  (no columns found, skipping)");
        }
    }

    Ok(())
}
