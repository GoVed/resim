mod parser;
mod resource;
mod simulation;

use simulation::Simulation;
use parser::parse_simulation_file;
use std::env;
use chrono::prelude::*;

fn main() {
    // Collect command line arguments
    let args: Vec<String> = env::args().collect();

    // Default values
    let default_reson_file = "example/simple_pencil.reson".to_string();
    let default_start_time = Utc::now().with_month(1).unwrap().with_day(1).unwrap().with_hour(0).unwrap().with_minute(0).unwrap().with_second(0).unwrap();
    let default_write_every = 3600;
    let default_run_for = 86400 * 31;

    // Parse command line arguments with keywords
    let mut reson_file = default_reson_file.clone();
    let mut start_time = default_start_time;
    let mut write_every = default_write_every;
    let mut run_for = default_run_for;

    for arg in &args[1..] {
        if let Some((key, value)) = arg.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            match key {
            "reson_file" => reson_file = value.to_string(),
            "start_time" => start_time = DateTime::parse_from_rfc3339(value).unwrap().with_timezone(&Utc),
            "write_every" => write_every = value.parse().unwrap(),
            "run_for" => run_for = value.parse().unwrap(),
            _ => eprintln!("Unknown argument: {}", key),
            }
        }
    }

    // Parse the .reson file
    let (resources, processes, on_use_processes) = parse_simulation_file(&reson_file).unwrap();
    println!("Resources: {:#?}", resources);
    println!("Processes: {:#?}", processes);

    // Initialize the simulation
    let mut sim = Simulation::new(resources, processes, on_use_processes);

    // Setting the simulation time
    sim.time = start_time;

    sim.write_every = write_every;
    sim.display_state();
    sim.run(run_for);
    sim.display_state();
}
