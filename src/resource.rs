use indexmap::IndexMap;

#[derive(Debug)]
pub struct Resource {
    pub unit: String,
    pub max: f64,
    pub amount: f64,
    pub life: u64,
    pub decay_at: Vec<u64>,
    pub decay_amount: Vec<f64>,
    pub amount_used_as_catalyst: f64,
}

impl Default for Resource {
    fn default() -> Self {
        Resource {
            unit: String::new(),
            max: f64::MAX,
            amount: 0.0,
            life: 0,
            decay_at: Vec::new(),
            decay_amount: Vec::new(),
            amount_used_as_catalyst: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Process {
    pub input: IndexMap<String, f64>,
    pub output: IndexMap<String, f64>,
    pub catalyst: IndexMap<String, f64>,
    pub max_catalyst: u64,
    pub period: u64,
    pub period_delta: u64,
    pub constraint: Vec<Vec<[u64;2]>>,
    pub constraint_modulo: Vec<u64>,
    pub on_use: f64,
    pub on_use_accumulate: f64,
    pub on_use_accumulate_for_writer: f64,
}

impl Default for Process {
    fn default() -> Self {
        Process {
            input: IndexMap::new(),
            output: IndexMap::new(),
            catalyst: IndexMap::new(),
            max_catalyst: 1,
            period: 0,
            period_delta: 0,
            constraint: Vec::new(),
            constraint_modulo: Vec::new(),
            on_use: 0.0,
            on_use_accumulate: 0.0,
            on_use_accumulate_for_writer: 0.0,
        }
    }
}
