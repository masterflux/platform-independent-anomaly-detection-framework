

use std::{collections::HashMap, error::Error, fs, path::Path};

use watch_wasm::change_point_detector::ChangePointDetector;
use watch_wasm::detectors::{MicroWatch, BOCPD, CUSUM};
use watch_wasm::utils::load_csv_multi;

// Tune microWatch
fn tune_mw(
    series: &[f64],
    mw_thresh: &[f64],
    mw_batch: &[usize],
    mw_buf: &[usize],
    mw_factor: usize,
    totals: &mut HashMap<(usize, usize, usize, usize), usize>
) {
    for (ti, &th) in mw_thresh.iter().enumerate() {
        for &bs in mw_batch {
            for &nb in mw_buf {
                let mut det = MicroWatch::new(0, th, bs);
                det.set_params(HashMap::from([
                    ("new_dist_buffer_size".to_string(), nb as f64),
                    ("max_dist_size".to_string(), (nb * mw_factor) as f64),
                ]));
                det.reinit();
                let cnt = det.detect(series).len();
                let key = (ti, bs, nb, nb * mw_factor);
                *totals.entry(key).or_insert(0) += cnt;
            }
        }
    }
}

/// Tune BOCPD 
fn tune_bocpd(
    series: &[f64],
    bocpd_alpha: &[f64],
    bocpd_beta: &[f64],
    bocpd_kappa: &[f64],
    bocpd_mu: &[f64],
    bocpd_lambda: &[f64],
    bocpd_thresh: &[f64],
    totals: &mut HashMap<(usize, usize, usize, usize, usize, usize), usize>
) {
    for (ai, &a) in bocpd_alpha.iter().enumerate() {
        for (bi, &b) in bocpd_beta.iter().enumerate() {
            for (ki, &k) in bocpd_kappa.iter().enumerate() {
                for (mi, &m) in bocpd_mu.iter().enumerate() {
                    for (li, &l) in bocpd_lambda.iter().enumerate() {
                        for (ti, &ct) in bocpd_thresh.iter().enumerate() {
                            let mut det = BOCPD::new(a, b, k, m);
                            det.set_params(HashMap::from([
                                ("lambda".to_string(), l),
                                ("cp_threshold".to_string(), ct),
                            ]));
                            det.reinit();
                            let cnt = det.detect(series).len();
                            let key = (ai, bi, ki, mi, li, ti);
                            *totals.entry(key).or_insert(0) += cnt;
                        }
                    }
                }
            }
        }
    }
}

/// Tune CUSUM 
fn tune_cusum(
    series: &[f64],
    cusum_warmup: &[usize],
    cusum_plimit: &[f64],
    totals: &mut HashMap<(usize, usize), usize>
) {
    for (wi, &warm) in cusum_warmup.iter().enumerate() {
        for (pi, &plim) in cusum_plimit.iter().enumerate() {
            let mut det = CUSUM::new(warm, plim);
            det.reinit();
            let cnt = det.detect(series).len();
            let key = (wi, pi);
            *totals.entry(key).or_insert(0) += cnt;
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let data_dir = Path::new("datasets/csv");

    // === MicroWatch parameter grid ===
    let mw_thresh = vec![0.5, 1.0, 1.5, 2.0];
    let mw_batch  = vec![3_usize, 5, 10];
    let mw_buf    = vec![16_usize, 32, 64];
    let mw_factor = 4_usize;
    let mut mw_totals: HashMap<(usize, usize, usize, usize), usize> = HashMap::new();

    // === BOCPD parameter grid ===
    let bocpd_alpha  = vec![1.0, 2.0, 5.0, 10.0];
    let bocpd_beta   = vec![1.0, 2.0, 5.0];
    let bocpd_kappa  = vec![1.0, 5.0, 10.0];
    let bocpd_mu     = vec![0.0, 1.0];
    let bocpd_lambda = vec![10.0, 20.0, 50.0];
    let bocpd_thresh = vec![0.01, 0.05, 0.1];
    let mut bocpd_totals: HashMap<(usize, usize, usize, usize, usize, usize), usize> = HashMap::new();

    // === CUSUM parameter grid ===
    let cusum_warmup = vec![5_usize, 10, 20];
    let cusum_plimit = vec![0.01, 0.05, 0.1];
    let mut cusum_totals: HashMap<(usize, usize), usize> = HashMap::new();

    let mut series_count = 0;

    for entry in fs::read_dir(data_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|s| s.to_str()) != Some("csv") {
            continue;
        }
        let data = load_csv_multi(path.to_str().unwrap())?;
        if data.is_empty() || data[0].len() != 1 {
            continue;
        }
        let series: Vec<f64> = data.iter().map(|r| r[0]).collect();
        series_count += 1;

        tune_mw(&series, &mw_thresh, &mw_batch, &mw_buf, mw_factor, &mut mw_totals);
        tune_bocpd(&series, &bocpd_alpha, &bocpd_beta, &bocpd_kappa, &bocpd_mu, &bocpd_lambda, &bocpd_thresh, &mut bocpd_totals);
        tune_cusum(&series, &cusum_warmup, &cusum_plimit, &mut cusum_totals);
    }

    println!("Scanned {} series in total", series_count);

    // Report MicroWatch result
    if let Some((&(ti, bs, nb, maxb), &tot)) = mw_totals.iter().max_by_key(|&(_, &c)| c) {
        println!(
            "Best MicroWatch => threshold={:.2}, batch={}, buf={}, maxbuf={} (total cps={})",
            mw_thresh[ti], bs, nb, maxb, tot
        );
    }

    // Report BOCPD result
    if let Some((&(ai, bi, ki, mi, li, ti), &tot)) = bocpd_totals.iter().max_by_key(|&(_, &c)| c) {
        println!(
            "Best BOCPD      => alpha={:.1}, beta={:.1}, kappa={:.1}, mu={:.1}, lambda={:.1}, threshold={:.3} (total cps={})",
            bocpd_alpha[ai], bocpd_beta[bi], bocpd_kappa[ki], bocpd_mu[mi], bocpd_lambda[li], bocpd_thresh[ti], tot
        );
    }

    // Report CUSUM result
    if let Some((&(wi, pi), &tot)) = cusum_totals.iter().max_by_key(|&(_, &c)| c) {
        println!(
            "Best CUSUM      => t_warmup={}, p_limit={:.3} (total cps={})",
            cusum_warmup[wi], cusum_plimit[pi], tot
        );
    }

    Ok(())
}