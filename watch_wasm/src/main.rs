use watch_wasm::change_point_detector::ChangePointDetector;

use watch_wasm::{
    utils::load_csv,
    detectors::{BOCPD, CUSUM, MicroWatch, PELT, BOCPDMS},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = load_csv("input.csv")?;
    println!("Loaded {} points", data.len());
    println!(" ");

    println!("\n=== Testing All Change Point Detection Algorithms ===\n");
    println!(" ");

    // 1. BOCPD
    println!("1. BOCPD (Bayesian Online Change Point Detection):");
    let mut bocpd = BOCPD::new(0.1, 0.01, 1.0, 0.0);
    println!("BOCPD: {:?}", bocpd.detect(&data));
    println!(" ");

    // 2. CUSUM
    println!("\n2. CUSUM (Cumulative Sum):");
    let mut cusum = CUSUM::new(30, 0.01);
    println!("CUSUM: {:?}", cusum.detect(&data));
    println!(" ");

    // 3. Micro-Watch
    println!("\n3. Micro-Watch:");
    for (i, name) in ["Euclidean", "Manhattan", "Chebyshev", "KL-Divergence", 
                      "Jensen-Shannon", "Bhattacharyya", "Hellinger"].iter().enumerate() {
        let mut mw = MicroWatch::new(i, 0.5, 5);
        println!("MW {}: {:?}", name, mw.detect(&data));
    }
    println!(" ");

    // 4. PELT
    println!("\n4. PELT (Pruned Exact Linear Time):");
    let mut pelt = PELT::new(10.0, 2, 1);
    println!("PELT: {:?}", pelt.detect(&data));
    println!(" ");

    // 5. BOCPDMS
    println!("\n5. BOCPDMS (Bayesian Online CP Detection - Multivariate):");
    let mut bocpdms = BOCPDMS::new(0.01, 0.01, 10.0);
    println!("BOCPDMS: {:?}", bocpdms.detect(&data));
    println!(" ");

    Ok(())
}
