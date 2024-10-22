use crate::resource::{Resource, Process};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::iter::Peekable;

pub fn parse_simulation_file(filename: &str) -> io::Result<(HashMap<String, Resource>, HashMap<String, Process>, HashMap<String, Process>)> {
    let path = Path::new(filename);
    let file = File::open(&path)?;
    let reader = io::BufReader::new(file);

    let mut resources = HashMap::new();
    let mut processes = HashMap::new();
    let mut on_use_processes = HashMap::new();
    
    // Collect all lines from the file into a vector so we can process them multiple times
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    let mut iter = lines.into_iter().peekable();

    let mut current_indentation = 0;
    let mut name = String::new();
    while let Some(line) = iter.next() {
        let line = line.trim().to_string();
        if line.is_empty() || line.starts_with('#') || line.chars().all(char::is_whitespace) {
            continue; // Skip empty lines, comments, lines with all whitespaces
        }
        current_indentation = line.chars().take_while(|&c| c == ' ').count();
        // Check if it's a resource or process declaration
        if line.ends_with("resource") {
            let resource = parse_resource(&mut iter, current_indentation);
            resources.insert(name.clone(), resource);
        } else if line.ends_with("process") {
            let process = parse_process(&mut iter, current_indentation);
            if process.on_use > 0.0 {
                on_use_processes.insert(name.clone(), process);
            } else {
                processes.insert(name.clone(), process);
            }
        }
        else {
            name = line;
        }
    }

    Ok((resources, processes, on_use_processes))
}

// Updated resource parsing function
fn parse_resource<I>(iter: &mut Peekable<I>, start_indentation: usize) -> Resource
where
    I: Iterator<Item = String>,
{
    let mut resource = Resource::default(); // Default values for resource

    while let Some(line) = iter.peek() {
        if line.is_empty() || line.starts_with('#') || line.chars().all(char::is_whitespace) {
            iter.next(); // Skip empty lines, comments, lines with all whitespaces
            continue;
        }
        let line_indentation = line.chars().take_while(|&c| c == ' ').count();
        if line_indentation <= start_indentation {
            return resource;
        }
        let line = iter.next().unwrap().trim().to_string();
    
        let tokens: Vec<&str> = line.split_whitespace().collect();

        match tokens[0] {
            "unit" => {
                resource.unit = tokens[1].to_string();
            }
            "max" => {
                resource.max = tokens[1].parse().unwrap();
            }
            "life" => {
                resource.life = parse_time_string(tokens[1], tokens[2]);
            }
            "amount" => {
                resource.amount = tokens[1].parse().unwrap(); // Default or initial amount, if specified
            }
            _ => {
                println!("Unknown token: {}", tokens[0]);
            }
        }
    }

    resource
}

// Updated process parsing function
fn parse_process<I>(iter: &mut Peekable<I>, start_indentation: usize) -> Process
where
    I: Iterator<Item = String>,
{
    let mut process = Process::default();

    while let Some(line) = iter.peek() {
        if line.is_empty() || line.starts_with('#') || line.chars().all(char::is_whitespace) {
            iter.next(); // Skip empty lines, comments, lines with all whitespaces
            continue;
        }
        let line_indentation = line.chars().take_while(|&c| c == ' ').count();
        if line_indentation <= start_indentation {
            return process;
        }
        let line = iter.next().unwrap().trim().to_string();
        
        let tokens: Vec<&str> = line.split_whitespace().collect();

        match tokens[0] {
            "produce" => {
                parse_resource_list(&mut *iter, line_indentation, &mut process.output);
            }
            "use" => {
                parse_resource_list(&mut *iter, line_indentation, &mut process.input);
            }
            "catalyze" => {
                parse_resource_list(&mut *iter, line_indentation, &mut process.catalyst);
            }
            "period" => {
                process.period = parse_time_string(tokens[1], tokens[2]);
            }
            "period_delta" => {
                process.period_delta = parse_time_string(tokens[1], tokens[2]);
                if process.period == 604800 {
                    process.period_delta += 4 * 86400;
                    process.period_delta %= 604800;
                }
            }
            "constraint" => {
                let (constraint, constraint_modulo) = parse_constraint(&mut *iter, line_indentation);
                process.constraint = constraint;
                process.constraint_modulo = constraint_modulo;
            }
            "on_use" => {
                process.on_use = tokens[1].parse().unwrap();
            }
            _ => {
                println!("Unknown token: {}", tokens[0]);
            }
        }
    }

    process
}

fn parse_resource_list<I>(iter: &mut Peekable<I>, start_indentation: usize, hashmap_to_add: &mut HashMap<String, f64>)
where
    I: Iterator<Item = String>,
{
    while let Some(line) = iter.peek() {
        if line.is_empty() || line.starts_with('#') || line.chars().all(char::is_whitespace) {
            iter.next(); // Skip empty lines, comments, lines with all whitespaces
            continue;
        }
        let line_indentation = line.chars().take_while(|&c| c == ' ').count();
        if line_indentation <= start_indentation {
            return;
        }
        let line = iter.next().unwrap().trim().to_string();
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.len() == 2 {
            let key = tokens[0].to_string();
            let value: f64 = tokens[1].parse().unwrap();
            hashmap_to_add.insert(key, value);
        } else {
            println!("Invalid resource list entry: {}", line);
        }
    }
}

fn parse_time_string(num:&str, period:&str) -> u64{
    let num: u64 = num.parse().unwrap();
    match period {
        "s" => num,
        "m" => num * 60,
        "h" => num * 3600,
        "d" => num * 86400,
        "w" => num * 604800,
        "y" => num * 31557600,
        _ => panic!("Invalid time string"),
    }
}

fn parse_constraint<I>(iter: &mut Peekable<I>, start_indentation: usize) -> (Vec<Vec<[u64;2]>>, Vec<u64>)
where
    I: Iterator<Item = String>,
{
    let mut constraint: Vec<Vec<[u64;2]>> = Vec::new();
    let mut constraint_modulo: Vec<u64> = Vec::new();
    while let Some(line) = iter.peek() {
        if line.is_empty() || line.starts_with('#') || line.chars().all(char::is_whitespace) {
            iter.next(); // Skip empty lines, comments, lines with all whitespaces
            continue;
        }
        let line_indentation = line.chars().take_while(|&c| c == ' ').count();
        if line_indentation <= start_indentation {
            return (constraint, constraint_modulo);
        }
        let line = iter.next().unwrap().trim().to_string();
        let tokens: Vec<&str> = line.split_whitespace().collect();
        let mut ranges: Vec<[u64;2]> = Vec::new();
        let mut multiplier = 0;
        let mut delta = 0;
        match tokens[0] {
            "s" => {
                constraint_modulo.push(60);
                multiplier = 1;
            }
            "m" => {
                constraint_modulo.push(3600);
                multiplier = 60;
            }
            "h" => {
                constraint_modulo.push(86400);
                multiplier = 3600;
            }
            "w" => {
                constraint_modulo.push(604800);
                multiplier = 86400;
                // Jan 1 1970 was a Thursday
                delta = 4;
            }
            _ => {
                println!("Unknown token: {}", tokens[0]);
            }
            
        }
        for i in 1..tokens.len() {
            let mut range: [u64; 2] = [0, 0];
            let range_tokens: Vec<&str> = tokens[i].split('-').collect();
            range[0] = (range_tokens[0].parse::<u64>().unwrap() + delta) * multiplier;
            if range_tokens.len() == 1 {
            range[1] = range[0] + multiplier - 1;
            } else {
            range[1] = ((range_tokens[1].parse::<u64>().unwrap() + 1) + delta) * multiplier - 1;
            }
            range[0] %= constraint_modulo.last().unwrap();
            range[1] %= constraint_modulo.last().unwrap();
            ranges.push(range);
        }
        constraint.push(ranges);
    }
    (constraint, constraint_modulo)
}