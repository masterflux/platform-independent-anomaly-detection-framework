use std::collections::HashMap;

pub trait ChangePointDetector {
    fn detect(&mut self, data: &[f64]) -> Vec<usize>;
    fn set_params(&mut self, params: HashMap<String, f64>);
    fn reinit(&mut self);
}