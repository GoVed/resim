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

    // Parse command line arguments
    let reson_file = args.get(1).unwrap_or(&default_reson_file);
    let start_time = args.get(2).map_or(default_start_time, |s| DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc));
    let write_every = args.get(3).map_or(default_write_every, |s| s.parse().unwrap());
    let run_for = args.get(4).map_or(default_run_for, |s| s.parse().unwrap());

    // Parse the .reson file
    let (resources, processes, on_use_processes) = parse_simulation_file(reson_file).unwrap();
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
