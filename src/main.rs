mod parser;
mod resource;
mod simulation;

use simulation::Simulation;
use parser::parse_simulation_file;
use chrono::prelude::*;

fn main() {
    // Parse the .reson file
    let (resources, processes, on_use_processes) = parse_simulation_file("example/simple_pencil.reson").unwrap();
    
    // Initialize the simulation
    let mut sim = Simulation::new(resources, processes, on_use_processes);
    // Setting the simulation time to 1st January 2024
    sim.time = match Utc.with_ymd_and_hms(2024, 1, 1, 9, 0, 0) {
        chrono::LocalResult::Single(dt) => dt,
        _ => panic!("Invalid time"),
    };
    // sim.write_every = 86400;
    sim.display_state();
    sim.run(86400*7);
    sim.display_state();
}
