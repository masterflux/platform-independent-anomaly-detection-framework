use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs,
    path::Path,
    io::{stdin, stdout, Write},
    time::Instant,
};

use serde::Deserialize;
use serde_json;
use csv::{ReaderBuilder, Trim, Writer};
use watch_wasm::change_point_detector::ChangePointDetector;
use watch_wasm::utils::load_csv_multi;
use watch_wasm::detectors::{BOCPD, CUSUM, MicroWatch, PELT};

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

/// MicroWatch tuning 
fn load_params_list<D>(path: &str) -> Result<Vec<D>, Box<dyn Error>>
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
struct BocpdParams {
    dataset: String,
    alpha: f64,
    beta: f64,
    kappa: f64,
    mu: f64,
}
#[derive(Deserialize)]
struct CusumParams {
    file: String,
    twarmup: f64,
    plimit: f64,
}
#[derive(Deserialize)]
struct PeltParams {
    dataset: String,
    penalty: f64,
    jump: usize,
    min_size: usize,
}
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

fn print_result_block(method: &str, params: &str, cp_str: &str, t: f64) {
    println!("┌──────── {} ─────────", method);
    println!("│ Parameters : {}", params);
    println!("│ ChangePts  : {}", if cp_str.is_empty() { "(none)" } else { cp_str });
    println!("│ Time (ms)  : {:.3}", t);
    println!("└───────────────────────────────");
}

fn main() -> Result<(), Box<dyn Error>> {
    // Prompt user
    println!("Available algorithms: MicroWatch, BOCPD, CUSUM, PELT, all");
    print!("Enter comma-separated list or 'all': ");
    stdout().flush()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let choices: HashSet<String> = input
        .trim()
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .collect();
    let run_all = choices.contains("all");

    // Load parameter 
    let bocpd_map =
        load_params("params/params_bocpd_best.csv", |p: &BocpdParams| p.dataset.clone())?;
    let cusum_map = load_params("params/params_cusum_best.csv", |p: &CusumParams| p.file.clone())?;
    let pelt_map =
        load_params("params/params_pelt_best.csv", |p: &PeltParams| p.dataset.clone())?;
    let mut watch_map: HashMap<String, Vec<WatchParams>> = HashMap::new();
    for wp in load_params_list::<WatchParams>("params/params_watch_best.csv")? {
        watch_map.entry(wp.dataset.clone()).or_default().push(wp);
    }
    let default_bocpd = (0.001, 10.0, 0.1, 0.0);

    
    fs::create_dir_all("results")?;
    let mut wtr = Writer::from_path("results/results.csv")?;
    wtr.write_record(&[
        "dataset",
        "method",
        "parameters",
        "change_points",
        "time_ms",
    ])?;

    
    let mut durations_ms = Vec::new();

    for entry in fs::read_dir(Path::new("datasets/csv"))? {
        let path = entry?.path();
        if path.extension().and_then(|s| s.to_str()) != Some("csv") {
            continue;
        }
        let dataset = path.file_stem().unwrap().to_string_lossy();
        let data = load_csv_multi(path.to_str().unwrap())?;
        let cols = data.get(0).map(|r| r.len()).unwrap_or(0);

        println!("\n=== Dataset: {} ({} cols detected) ===", dataset, cols);

        
        let ds_start = Instant::now();

        // MicroWatch 0 - 6
        if run_all || choices.contains("microwatch") {
            if let Some(params_list) = watch_map.get(&*dataset) {
                for wp in params_list {
                    let mut mw = MicroWatch::new(wp.distance_index, 0.0, 1);
                    let mut pm = HashMap::new();
                    pm.insert("batch_size".into(), wp.batch_size as f64);
                    pm.insert("threshold_ratio".into(), wp.threshold_ratio);
                    pm.insert("max_dist_size".into(), wp.max_dist_size as f64);
                    pm.insert("new_dist_buffer_size".into(), wp.new_dist_buffer_size as f64);
                    mw.set_params(pm.clone());
                    let start = Instant::now();
                    let cps = if cols > 1 {
                        mw.detect_multivariate(&data)
                    } else {
                        mw.detect(&data.iter().map(|r| r[0]).collect::<Vec<_>>())
                    };
                    let t = start.elapsed().as_secs_f64() * 1000.0;
                    let method = format!("MicroWatch(idx={})", wp.distance_index);
                    let params_json = serde_json::to_string(&pm)?;
                    let cp_str = cps.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(";");
                    wtr.write_record(&[
                        dataset.as_ref(),
                        &method,
                        &params_json,
                        &cp_str,
                        &format!("{:.3}", t),
                    ])?;
                    print_result_block(&method, &params_json, &cp_str, t);
                }
            }
        }

        //BOCPD 
        if (run_all || choices.contains("bocpd")) && cols == 1 {
            let univ: Vec<f64> = data.iter().map(|r| r[0]).collect();
            let (a, b, k, m) = bocpd_map
                .get(&*dataset)
                .map(|p| (p.alpha, p.beta, p.kappa, p.mu))
                .unwrap_or(default_bocpd);
            let mut boc = BOCPD::new(a, b, k, m);
            let start = Instant::now();
            let cps = boc.detect(&univ);
            let t = start.elapsed().as_secs_f64() * 1000.0;
            let method = "BOCPD";
            let params = format!("alpha={}, beta={}, kappa={}, mu={}", a, b, k, m);
            let cp_str = cps.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(";");
            wtr.write_record(&[
                dataset.as_ref(),
                method,
                &params,
                &cp_str,
                &format!("{:.3}", t),
            ])?;
            print_result_block(method, &params, &cp_str, t);
        }

        //CUSUM 
        if (run_all || choices.contains("cusum")) && cols == 1 {
            let univ: Vec<f64> = data.iter().map(|r| r[0]).collect();
            let (tw, pl) = cusum_map
                .get(&*dataset)
                .map(|p| (p.twarmup as usize, p.plimit))
                .unwrap_or((5, 0.1));
            let start = Instant::now();
            let cps = CUSUM::new(tw, pl).detect(&univ);
            let t = start.elapsed().as_secs_f64() * 1000.0;
            let method = "CUSUM";
            let params = format!("twarmup={}, plimit={}", tw, pl);
            let cp_str = cps.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(";");
            wtr.write_record(&[
                dataset.as_ref(),
                method,
                &params,
                &cp_str,
                &format!("{:.3}", t),
            ])?;
            print_result_block(method, &params, &cp_str, t);
        }

        //PELT 
        if run_all || choices.contains("pelt"){
            let univ: Vec<f64> = data.iter().map(|r| r[0]).collect();
            let (pen, j, ms) = pelt_map
                .get(&*dataset)
                .map(|p| (p.penalty, p.jump, p.min_size))
                .unwrap_or((100.0, 5, 1));
            let start = Instant::now();
            let cps = PELT::new(pen, j, ms).detect(&univ);
            let t = start.elapsed().as_secs_f64() * 1000.0;
            let method = "PELT";
            let params = format!("penalty={}, jump={}, min_size={}", pen, j, ms);
            let cp_str = cps.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(";");
            wtr.write_record(&[
                dataset.as_ref(),
                method,
                &params,
                &cp_str,
                &format!("{:.3}", t),
            ])?;
            print_result_block(method, &params, &cp_str, t);
        }

        
        let total = ds_start.elapsed().as_secs_f64() * 1000.0;
        durations_ms.push(total);
        println!("→ Total time for {}: {:.3} ms", dataset, total);
    }

    wtr.flush()?;
    let avg_total: f64 = durations_ms.iter().sum::<f64>() / durations_ms.len() as f64;
    println!("\nAverage total time across datasets: {:.3} ms", avg_total);
    println!("Results saved to results/results.csv");
    Ok(())
}