use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn gamma(z: f64) -> f64 { 
      // Approximation of gamma function using Stirling's formula
    if z < 0.5 {
        std::f64::consts::PI / (std::f64::consts::PI * z).sin() / gamma(1.0 - z)
    } else {
        let z = z - 1.0;
        let x = 0.99999999999980993
            + 676.5203681218851 / (z + 1.0)
            - 1259.1392167224028 / (z + 2.0)
            + 771.32342877765313 / (z + 3.0)
            - 176.61502916214059 / (z + 4.0)
            + 12.507343278686905 / (z + 5.0)
            - 0.13857109526572012 / (z + 6.0)
            + 9.9843695780195716e-6 / (z + 7.0)
            + 1.5056327351493116e-7 / (z + 8.0);
        
        let t = z + 7.5;
        (2.0 * std::f64::consts::PI).sqrt() * t.powf(z + 0.5) * (-t).exp() * x
    }
 }
 
pub fn erf(x: f64) -> f64 { 

     // Approximation of error function
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;
    
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    
    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();
    
    sign * y
 }

pub fn load_csv(path: &str) -> Result<Vec<f64>, Box<dyn Error>> {
   let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut data = Vec::new();
    
    for line in reader.lines() {
        let val: f64 = line?.trim().parse()?;
        data.push(val);
    }
    
    Ok(data)
}