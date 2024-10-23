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
    pub last_write_time: u64,
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
            last_write_time: 0,
        };

        // Write the headers to the CSV file
        let mut headers = vec!["time".to_string()];
        for resource_name in sim.resources.keys() {
            headers.push(resource_name.clone() + "_min");
            headers.push(resource_name.clone() + "_avg");
            headers.push(resource_name.clone() + "_max");
            headers.push(resource_name.clone());
        }
        for process_name in sim.on_use_processes.keys() {
            headers.push(process_name.clone());
        }
        sim.csv_writer.write_record(&headers).unwrap();

        sim
    }

    pub fn set_start_time(&mut self, time: DateTime<Utc>) {
        self.time = time;
        self.last_write_time = time.timestamp() as u64;
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

        // Update the min, max and avg values for resources
        self.update_resource_min_max_avg();
    }

    fn update_resource_min_max_avg(&mut self) {
        for resource in self.resources.values_mut() {
            if resource.amount < resource.resource_min_for_writer {
                resource.resource_min_for_writer = resource.amount;
            }
            if resource.amount > resource.resource_max_for_writer {
                resource.resource_max_for_writer = resource.amount;
            }
            resource.resource_avg_for_writer += resource.amount;
        }
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
        for (_, process) in &mut self.on_use_processes {            
            for (resource_name, amount) in &process.input {
                if let Some(resource) = self.resources.get_mut(resource_name) {
                    let addition = -1.0 * amount * process.on_use_accumulate / process.on_use;
                    resource.amount += addition;
                }
            }
            process.on_use_accumulate_for_writer += process.on_use_accumulate+process.on_use;
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
            let can_run = self.times_process_can_run(process);            
            if can_run > 0 {
                for (resource_name, amount) in &process.input {
                    if let Some(resource) = self.resources.get_mut(resource_name) {
                        resource.amount -= amount * can_run as f64;
                        // Deduct the decayed amount from the latest decay if exists
                        if resource.decay_at.len() > 0 {
                            let mut amount_to_deduct = amount * can_run as f64;
                            for i in 0..resource.decay_at.len() {
                                resource.decay_amount[i] -= amount_to_deduct;
                                if resource.decay_amount[i] < 0.0 {
                                    amount_to_deduct = -1.0 * resource.decay_amount[i];
                                    resource.decay_amount[i] = 0.0;
                                } else {
                                    break;
                                }
                            }
                        }
                    } else if let Some(resource) = self.on_use_processes.get_mut(resource_name) {
                        resource.on_use_accumulate += amount * can_run as f64;
                    }
                }
                for (resource_name, amount) in &process.catalyst {
                    if let Some(resource) = self.resources.get_mut(resource_name) {
                        resource.amount_used_as_catalyst += amount * can_run as f64;
                    }
                }
    
                // Add output resources
                for (resource_name, amount) in &process.output {
                    if let Some(resource) = self.resources.get_mut(resource_name) {
                        resource.amount += amount * can_run as f64;
                        if resource.life > 0 {
                            resource.decay_at.push(self.time.timestamp() as u64 + resource.life);
                            resource.decay_amount.push(*amount * can_run as f64);
                        }
                    }
                }
            }
        }
    }
    

    fn times_process_can_run(&self, process: &Process) -> u64 {
        // Check if the time is right for the process along with the constraints
        if !self.time_period_check(process.period, process.period_delta) {
            return 0;
        }
        if !self.constraint_check(&process.constraint, &process.constraint_modulo) {
            return 0;
        }

        // Check if the process has enough catalyst resources
        let mut can_run = process.max_catalyst;
        for (resource_name, amount) in &process.catalyst {
            if let Some(resource) = self.resources.get(resource_name) {
                let amount_can_use = ((resource.amount - resource.amount_used_as_catalyst) / *amount) as u64;
                if amount_can_use < can_run {
                    can_run = amount_can_use;
                }
                if can_run == 0 {
                    return 0;
                }
            } else {
                return 0;
            }
        }

        // Check if the process has enough input resources
        for (resource_name, amount) in &process.input {
            if let Some(resource) = self.resources.get(resource_name) {
                let amount_can_use = (resource.amount - resource.amount_used_as_catalyst) / *amount;
                if amount_can_use < can_run as f64 {
                    can_run = amount_can_use as u64;
                }
                if can_run == 0 {
                    return 0;
                }
            } else if let Some(on_use_process) = self.on_use_processes.get(resource_name) {                
                let amount_can_use = -1.0 * on_use_process.on_use_accumulate / *amount;
                if amount_can_use < can_run as f64 {
                    can_run = amount_can_use as u64;
                }
                if can_run == 0 {
                    return 0;
                }
            } else {
                return 0;                
            }
        }
        
        // Chech if output resources are not exceeding their maximum
        for (resource_name, amount) in &process.output {
            if let Some(resource) = self.resources.get(resource_name) {
                let amount_can_use = (resource.max - resource.amount) / *amount;
                if amount_can_use < can_run as f64 {
                    can_run = amount_can_use as u64;
                }
                if can_run == 0 {
                    return 0;
                }
            }
        }
        can_run
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
        for resource in self.resources.values_mut() {
            record.push(resource.resource_min_for_writer.to_string());
            record.push((resource.resource_avg_for_writer / (self.time.timestamp() as f64 - self.last_write_time as f64)).to_string());
            record.push(resource.resource_max_for_writer.to_string());
            record.push(resource.amount.to_string());
            resource.resource_min_for_writer = f64::MAX;
            resource.resource_max_for_writer = 0.0;
            resource.resource_avg_for_writer = 0.0;
        }
        for process in self.on_use_processes.values_mut() {
            record.push(process.on_use_accumulate_for_writer.to_string());
            process.on_use_accumulate_for_writer = 0.0;
        }
        self.csv_writer.write_record(&record).unwrap();
        self.last_write_time = self.time.timestamp() as u64;
    }
}
