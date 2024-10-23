use std::time;
use indexmap::IndexMap;
use crate::resource::{Process, Resource};
use chrono::prelude::*;
pub struct Simulation {
    pub resources: IndexMap<String, Resource>,
    pub processes: IndexMap<String, Process>,
    pub on_use_processes: IndexMap<String, Process>,
    pub time: DateTime<Utc>,
    pub csv_writer: csv::Writer<std::fs::File>,
    pub write_every: u64,
}

impl Simulation {
    pub fn new(resources:IndexMap<String, Resource>, processes:IndexMap<String, Process>, on_use_processes:IndexMap<String,Process>) -> Self {
        let mut sim  = Simulation {
            resources: resources,
            processes: processes,
            on_use_processes: on_use_processes,
            time: Utc::now(),
            csv_writer: csv::Writer::from_path("output.csv").unwrap(),
            write_every: 1,
        };

        // Write the headers to the CSV file
        let mut headers = vec!["time".to_string()];
        for resource_name in sim.resources.keys() {
            headers.push(resource_name.clone());
        }
        sim.csv_writer.write_record(&headers).unwrap();

        sim
    }

    pub fn run(&mut self, duration: u64) {
        let start_time_nanoseconds = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_nanos();
        for time_in_s in 0..duration {
            self.simulate_tick();
            //Adding one second to the time
            self.time = self.time + chrono::Duration::seconds(1);
            if time_in_s % self.write_every == 0 {
                self.write_current_state_to_csv();
            }
        }
        let end_time_nanoseconds = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_nanos();
        println!("Simulation took {} seconds", (end_time_nanoseconds - start_time_nanoseconds) as f64 / 1_000_000_000.0);
    }

    fn simulate_tick(&mut self) {
        // Decay resources
        self.decay_resources();
        
        // Deduct resources for on_use_processes at the start
        self.deduct_on_use_processes();

        // Set the amount_used_as_catalyst to 0
        self.reset_amount_used_as_catalyst();
    
        // Run processes
        self.run_processes();
    
        // Add back the remaining amount for on_use_processes at the end
        self.add_back_on_use_processes();
    }

    fn deduct_on_use_processes(&mut self) {
        for (_, process) in &mut self.on_use_processes {
            let mut feasible = true;
            for (resource_name, amount) in &process.input {
                if let Some(resource) = self.resources.get_mut(resource_name) {
                    if resource.amount < *amount {
                        feasible = false;
                        break;
                    }
                }
            }
            if feasible {
                for (resource_name, amount) in &process.input {
                    if let Some(resource) = self.resources.get_mut(resource_name) {
                        resource.amount -= amount;
                    }
                }
                process.on_use_accumulate = -1.0 * process.on_use;
            } else {
                process.on_use_accumulate = 0.0;
            }
        }
    }

    fn reset_amount_used_as_catalyst(&mut self) {
        for (_, resource) in &mut self.resources {
            resource.amount_used_as_catalyst = 0.0;
        }
    }

    fn add_back_on_use_processes(&mut self) {
        for (_, process) in &self.on_use_processes {            
            for (resource_name, amount) in &process.input {
                if let Some(resource) = self.resources.get_mut(resource_name) {
                    let addition = -1.0 * amount * process.on_use_accumulate / process.on_use;
                    resource.amount += addition;
                }
            }
            
        }
    }

    fn decay_resources(&mut self) {
        let now = self.time.timestamp() as u64;
        for (_, resource) in &mut self.resources {
            while resource.decay_at.len() > 0 && resource.decay_at[0] <= now {
                resource.amount -= resource.decay_amount[0];
                resource.decay_at.remove(0);
                resource.decay_amount.remove(0);
            }
        }
    }

    fn run_processes(&mut self) {
        for (_, process) in &self.processes {
            if self.can_process_run(process) {
                for (resource_name, amount) in &process.input {
                    if let Some(resource) = self.resources.get_mut(resource_name) {
                        resource.amount -= amount;
                        // Deduct the decayed amount from the latest decay if exists
                        if resource.decay_at.len() > 0 {
                            resource.decay_amount[0] -= amount;
                        }
                    } else if let Some(resource) = self.on_use_processes.get_mut(resource_name) {
                        resource.on_use_accumulate += amount;
                    }
                }
                for (resource_name, amount) in &process.catalyst {
                    if let Some(resource) = self.resources.get_mut(resource_name) {
                        resource.amount_used_as_catalyst += amount;
                    }
                }
    
                // Add output resources
                for (resource_name, amount) in &process.output {
                    if let Some(resource) = self.resources.get_mut(resource_name) {
                        resource.amount += amount;
                        if resource.life > 0 {
                            resource.decay_at.push(self.time.timestamp() as u64 + resource.life);
                            resource.decay_amount.push(*amount);
                        }
                    }
                }
            }
        }
    }
    

    fn can_process_run(&self, process: &Process) -> bool {
        // Check if the time is right for the process along with the constraints
        if !self.time_period_check(process.period, process.period_delta) {
            return false;
        }
        if !self.constraint_check(&process.constraint, &process.constraint_modulo) {
            return false;
        }
        // Check if the process has enough input resources
        for (resource_name, amount) in &process.input {
            if let Some(resource) = self.resources.get(resource_name) {
                if resource.amount - resource.amount_used_as_catalyst < *amount {
                    return false;
                }
            } else if let Some(on_use_process) = self.on_use_processes.get(resource_name) {
                if on_use_process.on_use_accumulate + *amount > 0.0 {
                    return false;
                }
            } else {
                return false;                
            }
        }
        // Check if the process has enough catalyst resources
        for (resource_name, amount) in &process.catalyst {
            if let Some(resource) = self.resources.get(resource_name) {
                if resource.amount - resource.amount_used_as_catalyst < *amount {
                    return false;
                }
            } else {
                return false;
            }
        }
        // Chech if output resources are not exceeding their maximum
        for (resource_name, amount) in &process.output {
            if let Some(resource) = self.resources.get(resource_name) {
                if resource.amount + amount > resource.max {
                    return false;
                }
            }
        }
        true
    }

    fn time_period_check(&self, period: u64, period_delta: u64) -> bool {
        let now = self.time.timestamp() as u64;

        if (now - period_delta) % period == 0 {
            return true;
        }
        false
    }

    fn constraint_check(&self, constraint: &Vec<Vec<[u64;2]>>, constraint_modulo: &Vec<u64>) -> bool {
        let now = self.time.timestamp() as u64;
        for (i, constraint_list) in constraint.iter().enumerate() {
            let modulo = constraint_modulo[i];
            let mut feasible_for_this_modulo = false;
            let time_to_check = now % modulo;
            for [start, end] in constraint_list {
                if (*start <= *end && time_to_check >= *start && time_to_check <= *end) ||
                   (*start > *end && (time_to_check >= *start || time_to_check <= *end)) {
                    feasible_for_this_modulo = true;
                    break;
                }
            }
            if !feasible_for_this_modulo {
                return false;
            }
        }
        true
    }
    
    pub fn display_state(&self) {
        println!("Current state of resources at time {}s:", self.time);
        for (resource_name, resource) in &self.resources {
            println!(
                "{}: {} {} (Max: {:e})",
                resource_name,
                resource.amount,
                resource.unit,
                resource.max,
            );
        }
    }

    fn write_current_state_to_csv(&mut self) {
        // timestamp, resource_0_amount, resource_1_amount, ...
        let mut record = vec![self.time.to_string()];
        for resource in self.resources.values() {
            record.push(resource.amount.to_string());
        }
        self.csv_writer.write_record(&record).unwrap();
    }
}
