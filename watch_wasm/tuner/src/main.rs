// File: watch_wasm/src/bin/tune_all.rs

use std::{collections::HashMap, error::Error, fs, path::Path};

// Bring all detectors into scope
use watch_wasm::change_point_detector::ChangePointDetector;
use watch_wasm::detectors::{MicroWatch, BOCPD, CUSUM, PELT, BOCPDMS};
use watch_wasm::utils::load_csv_multi;

//— helper to tune MicroWatch (unchanged) —
fn tune_mw(
    series: &[f64],
    thresh: &[f64],
    batch: &[usize],
    buf: &[usize],
    factor: usize,
    totals: &mut HashMap<(usize, usize, usize, usize), usize>,
) {
    for (ti, &t) in thresh.iter().enumerate() {
        for &b in batch {
            for &n in buf {
                let mut det = MicroWatch::new(0, t, b);
                det.set_params(HashMap::from([
                    ("new_dist_buffer_size".to_string(), n as f64),
                    ("max_dist_size".to_string(),        (n * factor) as f64),
                ]));
                det.reinit();
                let cnt = det.detect(series).len();
                let key = (ti, b, n, n * factor);
                *totals.entry(key).or_insert(0) += cnt;
            }
        }
    }
}

//— helper to tune BOCPD (unchanged) —
fn tune_bocpd(
    series: &[f64],
    alpha: &[f64],
    beta: &[f64],
    kappa: &[f64],
    mu: &[f64],
    lambda: &[f64],
    thresh: &[f64],
    totals: &mut HashMap<(usize,usize,usize,usize,usize,usize), usize>,
) {
    for (ai, &a) in alpha.iter().enumerate() {
        for (bi, &b) in beta.iter().enumerate() {
            for (ki, &k) in kappa.iter().enumerate() {
                for (mi, &m) in mu.iter().enumerate() {
                    for (li, &l) in lambda.iter().enumerate() {
                        for (ti, &ct) in thresh.iter().enumerate() {
                            let mut det = BOCPD::new(a, b, k, m);
                            det.set_params(HashMap::from([
                                ("lambda".to_string(),      l),
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

//— helper to tune CUSUM (unchanged) —
fn tune_cusum(
    series: &[f64],
    warmup: &[usize],
    plimit: &[f64],
    totals: &mut HashMap<(usize,usize), usize>,
) {
    for (wi, &w) in warmup.iter().enumerate() {
        for (pi, &p) in plimit.iter().enumerate() {
            let mut det = CUSUM::new(w, p);
            det.reinit();
            let cnt = det.detect(series).len();
            let key = (wi, pi);
            *totals.entry(key).or_insert(0) += cnt;
        }
    }
}

//— NEW: helper to tune PELT —
fn tune_pelt(
    series: &[f64],
    penalties: &[f64],
    min_sizes: &[usize],
    jumps: &[usize],
    totals: &mut HashMap<(usize,usize,usize), usize>,
) {
    for (pi, &pen) in penalties.iter().enumerate() {
        for (mi, &ms) in min_sizes.iter().enumerate() {
            for (ji, &jp) in jumps.iter().enumerate() {
                let mut det = PELT::new(pen, ms, jp);
                det.reinit();
                let cnt = det.detect(series).len();
                let key = (pi, mi, ji);
                *totals.entry(key).or_insert(0) += cnt;
            }
        }
    }
}

//— NEW: helper to tune BOCPDMS —
fn tune_bocpdms(
    series: &[f64],
    prior_as: &[f64],
    prior_bs: &[f64],
    intensities: &[f64],
    totals: &mut HashMap<(usize,usize,usize), usize>,
) {
    for (ai, &a) in prior_as.iter().enumerate() {
        for (bi, &b) in prior_bs.iter().enumerate() {
            for (ii, &intens) in intensities.iter().enumerate() {
                let mut det = BOCPDMS::new(a, b, intens);
                det.reinit();
                let cnt = det.detect(series).len();
                let key = (ai, bi, ii);
                *totals.entry(key).or_insert(0) += cnt;
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let data_dir = Path::new("datasets/csv");

    //— parameter grids —
    let mw_thresh    = vec![0.5, 1.0, 1.5, 2.0];
    let mw_batch     = vec![3, 5, 10];
    let mw_buf       = vec![16, 32, 64];
    let mw_factor    = 4;

    let bocpd_alpha  = vec![1.0, 2.0, 5.0, 10.0];
    let bocpd_beta   = vec![1.0, 2.0, 5.0];
    let bocpd_kappa  = vec![1.0, 5.0, 10.0];
    let bocpd_mu     = vec![0.0, 1.0];
    let bocpd_lambda = vec![10.0, 20.0, 50.0];
    let bocpd_thresh = vec![0.01, 0.05, 0.1];

    let cusum_warmup = vec![5, 10, 20];
    let cusum_plimit = vec![0.01, 0.05, 0.1];

    let pelt_pens    = vec![100.0, 200.0, 500.0]; // example values
    let pelt_min     = vec![5, 10, 20];
    let pelt_jump    = vec![1, 5, 10];

    let bocpdms_a    = vec![1.0, 2.0, 5.0];
    let bocpdms_b    = vec![1.0, 2.0, 5.0];
    let bocpdms_int  = vec![1.5, 2.0, 3.0];

    //— accumulators —
    let mut mw_totals     = HashMap::new();
    let mut bocpd_totals  = HashMap::new();
    let mut cusum_totals  = HashMap::new();
    let mut pelt_totals   = HashMap::new();
    let mut bocpdms_totals= HashMap::new();

    let mut series_count = 0;

    //— main loop —
    for entry in fs::read_dir(data_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|s| s.to_str()) != Some("csv") { continue; }
        let data = load_csv_multi(path.to_str().unwrap())?;
        if data.is_empty() || data[0].len() != 1   { continue; }

        series_count += 1;
        let series: Vec<f64> = data.into_iter().map(|r| r[0]).collect();

        tune_mw(&series, &mw_thresh, &mw_batch, &mw_buf, mw_factor, &mut mw_totals);
        tune_bocpd(&series, &bocpd_alpha, &bocpd_beta, &bocpd_kappa,
                  &bocpd_mu, &bocpd_lambda, &bocpd_thresh, &mut bocpd_totals);
        tune_cusum(&series, &cusum_warmup, &cusum_plimit, &mut cusum_totals);
        tune_pelt(&series, &pelt_pens, &pelt_min, &pelt_jump, &mut pelt_totals);
        tune_bocpdms(&series, &bocpdms_a, &bocpdms_b, &bocpdms_int, &mut bocpdms_totals);
    }

    println!("Scanned {} series total\n", series_count);

    //— report each best config —
    if let Some((&(ti,bs,nb,maxb), &tot)) = mw_totals.iter().max_by_key(|&(_, &c)| c) {
       println!(
            "MicroWatch => threshold_ratio={:.2}, batch_size={}, new_dist_buffer_size={}, max_dist_size={} (total cps={})",
            mw_thresh[ti], bs, nb, maxb, tot
        );
    }
    if let Some((&(ai,bi,ki,mi,li,ti), &tot)) = bocpd_totals.iter().max_by_key(|&(_, &c)| c) {
         println!(
            "BOCPD      => alpha={:.1}, beta={:.1}, kappa={:.1}, mu={:.1}, lambda={:.1}, cp_threshold={:.3} (total cps={})",
            bocpd_alpha[ai],
            bocpd_beta[bi],
            bocpd_kappa[ki],
            bocpd_mu[mi],
            bocpd_lambda[li],
            bocpd_thresh[ti],
            tot
        );
    }
    if let Some((&(wi,pi), &tot)) = cusum_totals.iter().max_by_key(|&(_, &c)| c) {
          println!("CUSUM      => warmup_period={}, p_limit={:.3} (total cps={})",
            cusum_warmup[wi], cusum_plimit[pi], tot);
    }
    if let Some((&(pi,mi,ji), &tot)) = pelt_totals.iter().max_by_key(|&(_, &c)| c) {
        println!("PELT       => penalty={}, min_segment_length={}, jump={}, (total cps={})",
            pelt_pens[pi], pelt_min[mi], pelt_jump[ji], tot);
    }
    if let Some((&(ai,bi,ii), &tot)) = bocpdms_totals.iter().max_by_key(|&(_, &c)| c) {
        println!("BOCPDMS    => prior_alpha={:.1}, prior_beta={:.1}, intensity_threshold={:.1} (total cps={})",
            bocpdms_a[ai], bocpdms_b[bi], bocpdms_int[ii], tot);
    }

    Ok(())
}