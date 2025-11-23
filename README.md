# U-RAS

**Resource Allocation and Scheduling** - High-performance optimization algorithms in Rust with C FFI support

[![Crates.io](https://img.shields.io/crates/v/u-ras.svg)](https://crates.io/crates/u-ras)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## Overview

U-RAS provides domain-agnostic optimization algorithms that can be applied to various scheduling and resource allocation problems:

- **Genetic Algorithm (GA)** - Evolutionary optimization with crossover and mutation
- **CP-SAT Solver** - Constraint Programming with SAT-based solving
- **Constraint Propagation** - Arc consistency and domain reduction

### Use Cases

- **Manufacturing** - Production scheduling, job-shop problems (via [U-APS](https://github.com/iyulab/U-APS))
- **Healthcare** - Operating room scheduling, staff allocation
- **Logistics** - Route optimization, fleet management
- **Education** - Timetabling, exam scheduling
- **Cloud Computing** - VM placement, job scheduling

## Features

- 🚀 **High Performance** - Written in Rust with parallel execution via Rayon
- 🔌 **C FFI Support** - Use from C#, Python, or any language with C bindings
- 📦 **Zero Dependencies on External Solvers** - Self-contained algorithms
- 🎯 **Domain Agnostic** - Abstract models adaptable to any scheduling domain

## Installation

### From crates.io

```toml
[dependencies]
u-ras = "0.1"
```

### From GitHub

```toml
[dependencies]
u-ras = { git = "https://github.com/iyulab/U-RAS" }
```

## Quick Start

```rust
use u_ras::models::{Task, Activity, Resource, ActivityDuration};
use u_ras::scheduler::SimpleScheduler;

// Create a task with activities
let task = Task::new("T1")
    .with_priority(5)
    .with_activity(
        Activity::new("A1", "T1", 1)
            .with_duration(ActivityDuration::fixed(5000))
            .with_resources("machine", vec!["M1".into(), "M2".into()])
    );

// Create resources
let resources = vec![
    Resource::primary("M1").with_efficiency(1.0),
    Resource::primary("M2").with_efficiency(0.9),
];

// Schedule
let scheduler = SimpleScheduler::new();
let schedule = scheduler.schedule(&[task], &resources, 0);

println!("Makespan: {} ms", schedule.makespan_ms);
```

## Core Concepts

| Concept | Description | Examples |
|---------|-------------|----------|
| **Task** | Unit of work to schedule | Job, Surgery, Delivery |
| **Activity** | Step within a task | Operation, Procedure, Leg |
| **Resource** | Allocatable entity | Machine, Doctor, Truck |
| **Constraint** | Rules for valid schedules | Precedence, Capacity, TimeWindow |
| **Schedule** | Output assignments | Activity → Resource → Time |

## Modules

### models
Core data structures for scheduling problems:
- `Task` - Work unit containing activities
- `Activity` - Atomic step requiring resources
- `Resource` - Allocatable entity with capabilities
- `Calendar` - Time availability windows
- `Constraint` - Scheduling rules and limits
- `Schedule` - Solution with assignments

### scheduler
Scheduling algorithms:
- `SimpleScheduler` - Priority-based greedy algorithm
- `ScheduleKpi` - Quality metrics (makespan, tardiness, utilization)

### ga
Genetic Algorithm implementations:
- `GaScheduler` - Single-objective GA
- `GaConfig` - Algorithm parameters (population, mutation rate, etc.)
- Dual-vector encoding for operation sequence and resource assignment

### cp
Constraint Programming:
- `CpSat` - CP-SAT solver for optimal solutions
- Constraint propagation with arc consistency

## Architecture

```
┌─────────────────────────────────────┐
│            U-RAS Core               │
├─────────────────────────────────────┤
│  Models: Task, Activity, Resource   │
│  Algorithms: GA, CP-SAT, Greedy     │
│  FFI: C-compatible interface        │
└─────────────────────────────────────┘
          ▲           ▲
          │           │
   ┌──────┴───┐ ┌─────┴────┐
   │  U-APS   │ │  Others  │
   │(Manufact)│ │(Medical) │
   └──────────┘ └──────────┘
```

## FFI Usage

U-RAS compiles to a C-compatible dynamic library:

```c
// C example
extern int uras_schedule(const char* request_json, char** result_ptr);
extern void uras_free_string(char* ptr);
```

```csharp
// C# example
[LibraryImport("u_ras")]
public static partial int uras_schedule(string request, out IntPtr result);
```

## Performance

Benchmarks on typical scheduling problems:

| Problem Size | GA (1000 gen) | CP-SAT | Greedy |
|-------------|---------------|--------|--------|
| 10 jobs, 5 resources | 50ms | 20ms | 1ms |
| 100 jobs, 20 resources | 500ms | 2s | 10ms |
| 500 jobs, 50 resources | 5s | timeout | 100ms |

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Related Projects

- [U-APS](https://github.com/iyulab/U-APS) - Manufacturing scheduling system built on U-RAS
