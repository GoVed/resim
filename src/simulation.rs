use std::time;
use std::collections::HashMap;
use crate::resource::{Resource, Process};
use chrono::prelude::*;
pub struct Simulation {
    pub resources: HashMap<String, Resource>,
    pub processes: HashMap<String, Process>,
    pub on_use_processes: HashMap<String, Process>,
    pub time: DateTime<Utc>,
    pub csv_writer: csv::Writer<std::fs::File>,
    pub write_every: u64,
}

impl Simulation {
    pub fn new(resources:HashMap<String, Resource>, processes:HashMap<String, Process>, on_use_processes:HashMap<String,Process>) -> Self {
        let mut sim  = Simulation {
            resources: resources,
            processes: processes,
            on_use_processes: on_use_processes,
            time: Utc::now(),
            csv_writer: csv::Writer::from_path("output.csv").unwrap(),
            write_every: 1,
        };

        // Write the headers to the CSV file
        let mut headers = vec!["timestamp".to_string()];
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
    
        // Deduct resources for on_use_processes at the start
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
    
        for (_, resource) in &mut self.resources {
            // Decay resources
            let now = self.time.timestamp() as u64;
            while resource.decay_at.len() > 0 && resource.decay_at[0] <= now {
                resource.amount -= resource.decay_amount[0];
                resource.decay_at.remove(0);
                resource.decay_amount.remove(0);
            }
        }
    
        for (_, process) in &self.processes {
            if self.can_process_run(process) {
                for (resource_name, amount) in &process.input {
                    if let Some(resource) = self.resources.get_mut(resource_name) {
                        resource.amount -= amount;
                    } else if let Some(resource) = self.on_use_processes.get_mut(resource_name) {
                        resource.on_use_accumulate += amount;
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
    
        // Add back the remaining amount for on_use_processes at the end
        for (_, process) in &self.on_use_processes {            
            for (resource_name, amount) in &process.input {
                if let Some(resource) = self.resources.get_mut(resource_name) {
                    let addition = -1.0 * amount * process.on_use_accumulate / process.on_use;
                    resource.amount += addition;
                }
            }
            
        }
    }
    

    fn can_process_run(&self, process: &Process) -> bool {
        // Check if the time is right for the process along with the constraints
        if !self.time_period_check(process.period, process.period_delta) {
            return false;
        }
        // Check if the process has enough input resources
        for (resource_name, amount) in &process.input {
            if let Some(resource) = self.resources.get(resource_name) {
                if resource.amount < *amount {
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
                if resource.amount < *amount {
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
        let mut record = vec![self.time.timestamp().to_string()];
        for resource in self.resources.values() {
            record.push(resource.amount.to_string());
        }
        self.csv_writer.write_record(&record).unwrap();
    }
}
