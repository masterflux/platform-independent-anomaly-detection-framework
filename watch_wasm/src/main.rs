// use std::{error::Error, fs, path::Path};
// use watch_wasm::change_point_detector::ChangePointDetector;
// use watch_wasm::utils::load_csv_multi;
// use watch_wasm::detectors::{BOCPD, CUSUM, MicroWatch, PELT, BOCPDMS};

// fn main() -> Result<(), Box<dyn Error>> {
//     let data_dir = Path::new("datasets/csv");
//     for entry in fs::read_dir(data_dir)? {
//         let entry = entry?;
//         let path = entry.path();
//         if path.extension().and_then(|s| s.to_str()) != Some("csv") {
//             continue;
//         }
//         let name = path.file_stem().unwrap().to_string_lossy();
//         let data = load_csv_multi(path.to_str().unwrap())?;
//         let rows = data.len();
//         let cols = data.get(0).map(|r| r.len()).unwrap_or(0);

//         println!("\n=== Dataset: {} ({} rows × {} cols) ===", name, rows, cols);

//         if cols > 1 {
//             // Multivariate
//             let mut bocpdms = BOCPDMS::new(5.0, 5.0, 1.5);
//             let cps = bocpdms.detect_multivariate(&data);
//             println!("BOCPDMS (multivariate) → {:?}", cps);
//         } else if cols == 1 {
//             // Univariate
//             let univ: Vec<f64> = data.iter().map(|r| r[0]).collect();

//             // 1) BOCPD
//             let mut bocpd = BOCPD::new(1.0,1.0, 1.0, 0.0);
//             println!("BOCPD   → {:?}", bocpd.detect(&univ));

//             // 2) CUSUM
//             let mut cusum = CUSUM::new(5, 0.100);
//             println!("CUSUM   → {:?}", cusum.detect(&univ));

//             // 3) MicroWatch 
//             let mut mw = MicroWatch::new(0, 0.5, 3);
//             println!("Micro-E → {:?}", mw.detect(&univ));

//             // 4) PELT
//             let mut pelt = PELT::new(100.0, 5, 1);
//             println!("PELT    → {:?}", pelt.detect(&univ));

//             // 5) BOCPDMS (univariate)
//             let mut bocpdms = BOCPDMS::new(5.0, 5.0, 1.5);
//             println!("BOCPDMS → {:?}", bocpdms.detect(&univ));
//         } else {
//             println!("  (no columns found, skipping)");
//         }
//     }

//     Ok(())
// }


// File: watch_wasm/src/bin/dump_results.rs

use std::{error::Error, fs, path::Path};
use csv::Writer;
use watch_wasm::change_point_detector::ChangePointDetector;
use watch_wasm::utils::load_csv_multi;
use watch_wasm::detectors::{BOCPD, CUSUM, MicroWatch, PELT, BOCPDMS};

fn vec_to_str(v: &[usize]) -> String {
    v.iter()
     .map(|i| i.to_string())
     .collect::<Vec<_>>()
     .join(";")
}

fn main() -> Result<(), Box<dyn Error>> {
    
    let mut wtr = Writer::from_path("results.csv")?;
    // header
    wtr.write_record(&[
        "dataset", "BOCPD", "CUSUM", "MicroWatch", "PELT", "BOCPDMS_univ", "BOCPDMS_multi"
    ])?;

    let data_dir = Path::new("datasets/csv");
    for entry in fs::read_dir(data_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|s| s.to_str()) != Some("csv") {
            continue;
        }
        // dataset name 
        let name = path.file_stem().unwrap().to_string_lossy();
        let data = load_csv_multi(path.to_str().unwrap())?;
        let cols = data.get(0).map(|r| r.len()).unwrap_or(0);

        // placeholders for each algorithm's change-points
        let mut bocpd_lst = Vec::new();
        let mut cusum_lst = Vec::new();
        let mut mw_lst    = Vec::new();
        let mut pelt_lst  = Vec::new();
        let mut bocpdms_u = Vec::new();
        let mut bocpdms_m = Vec::new();

        if cols > 1 {
            // multivariate only BOCPDMS
            bocpdms_m = BOCPDMS::new(5.0, 5.0, 1.5)
                .detect_multivariate(&data);
        } else if cols == 1 {
            // univariate: run all 5 detectors
            let univ: Vec<f64> = data.iter().map(|r| r[0]).collect();

            bocpd_lst  = BOCPD::new(1.0, 1.0, 1.0, 0.0).detect(&univ);
            cusum_lst  = CUSUM::new(5,  0.1).detect(&univ);
            mw_lst     = MicroWatch::new(0,   0.5, 3).detect(&univ);
            pelt_lst   = PELT::new(100.0, 5, 1).detect(&univ);
            bocpdms_u  = BOCPDMS::new(5.0, 5.0, 1.5).detect(&univ);
        }

        // stringify each vector of change-points
        let bocpd_s     = vec_to_str(&bocpd_lst);
        let cusum_s     = vec_to_str(&cusum_lst);
        let mw_s        = vec_to_str(&mw_lst);
        let pelt_s      = vec_to_str(&pelt_lst);
        let bocpdms_u_s = vec_to_str(&bocpdms_u);
        let bocpdms_m_s = vec_to_str(&bocpdms_m);

        wtr.write_record(&[
            name.as_ref(),
            bocpd_s.as_str(),
            cusum_s.as_str(),
            mw_s.as_str(),
            pelt_s.as_str(),
            bocpdms_u_s.as_str(),
            bocpdms_m_s.as_str(),
        ])?;
    }

    wtr.flush()?;
    println!("→ results.csv written");
    Ok(())
}