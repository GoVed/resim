use std::collections::HashMap;

#[derive(Debug)]
pub struct Resource {
    pub unit: String,
    pub max: f64,
    pub amount: f64,
    pub life: u64,
    pub decay_at: Vec<u64>,
    pub decay_amount: Vec<f64>
}

impl Default for Resource {
    fn default() -> Self {
        Resource {
            unit: String::new(),
            max: f64::MAX,
            amount: 0.0,
            life: 0,
            decay_at: Vec::new(),
            decay_amount: Vec::new()
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Process {
    pub input: HashMap<String, f64>,
    pub output: HashMap<String, f64>,
    pub catalyst: HashMap<String, f64>,
    pub period: u64,
    pub period_delta: u64,
    pub constraint: String,
    pub on_use: f64,
    pub on_use_accumulate: f64
}
