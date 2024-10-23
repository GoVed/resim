# A simulator for resources

You can simulate whatever you want which consists of resources. 

# Usage

```
resim reson_file=your_file.reson start_time=timestamp_of_start_time write_every=duration_in_s run_for=duration_in_s
```
It runs the simulation for run_for seconds and writes to `output.csv` every write_every seconds.

# Output

A CSV file

```
timestamp, resource_1, resource_2, ...
0, units of resource_1 at 0th s, units of resource_2 at 0th s,
1, units of resource_1 at 1st s, units of resource_2 at 1st s,
```

# .reson format
A custom readable format to make inputs for the simulation. It is parsed by at the start of the simulation creating rust objects.

Also, yes it uses identations :)

It has two main types of objects resource & process.

- resource : something of value
- process : something that creates resource

## resource

It could be anything like money, human, work_hour, wood, etc.

```
resource_name
    resource // Identifier for resource type
    unit unit_for_the_resource 
    max max_amount_for_the_resource // optional
    amount starting_amount // optional
    life life_of_the_resource [s,h,m,d,w,y] // optional
```

Example

```
# A resource representing pencil machine, max 2 machines can be there with life of 5 years for each machine

pencil_machine
    resource
    unit count
    max 2
    life 5 y
```

## process

It could be something which produces/uses resources like manufacturing pencil from wood, purchasing wood, selling pencil, etc.

```
process_name
    process // identifier for process
    on_use max_on_use // identifier for on use process
    use
        resource_1 quantity_of_resource_1
        resource_2 quantity_of_resource_2
        .
        .
    produce
        resource_1 quantity_of_resource_1
        resource_1 quantity_of_resource_2
        .
        .
    catalyze // optional
        resource_1 quantity_of_resource_1
        resource_2 quantity_of_resource_2
        .
        .
    period repeated_time_at_which_process_is_executed [s,h,m,d,w,y]
    constraint // optional
        [s,h,m,d,w,y] at
        [s,h,m,d,w,y] start-end
        .
        .
```

Example

```
# A process which uses 0.7 electirc_intake, 10 wood, 2 graphite to generate 10 pencil every second with the help of 1 pencil_machine. Constraints are that it only works from mon-fri and from 10am-4pm

manufacture_pencil
    process
    use
        electirc_intake 0.7 
        wood 10
        graphite 2
    produce
        pencil 10
    catalyze
        pencil_machine 1
    period 1 s
    constraint 
        w 1-5
        h 10-16
```