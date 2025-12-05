# U-RAS

**Universal Resource Allocation and Scheduling** - Domain-agnostic optimization algorithms in Rust

[![Crates.io](https://img.shields.io/crates/v/u-ras.svg)](https://crates.io/crates/u-ras)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## Overview

U-RAS provides domain-agnostic optimization algorithms that can be applied to various scheduling and resource allocation problems:

- **Genetic Algorithm (GA)** - Evolutionary optimization with dual-vector encoding
- **CP-SAT Solver** - Constraint Programming with SAT-based solving
- **Dispatching Rules** - 14+ priority rules with multi-layer strategies
- **Time Constraints** - TimeWindow, PERT estimation, probabilistic scheduling

### Use Cases

- **Manufacturing** - Production scheduling, job-shop problems (via [U-APS](https://github.com/iyulab/U-APS-releases))
- **Healthcare** - Operating room scheduling, staff allocation
- **Logistics** - Route optimization, fleet management
- **Education** - Timetabling, exam scheduling
- **Cloud Computing** - VM placement, job scheduling

## Features

- **High Performance** - Written in Rust with parallel execution via Rayon
- **C FFI Support** - Use from C#, Python, or any language with C bindings
- **Zero Dependencies on External Solvers** - Self-contained algorithms
- **Domain Agnostic** - Abstract models adaptable to any scheduling domain
- **Extensible Rules** - Implement custom dispatching rules via trait

## Installation

### From crates.io

```toml
[dependencies]
u-ras = "0.2"
```

### From GitHub

```toml
[dependencies]
u-ras = { git = "https://github.com/iyulab/U-RAS" }
```

## Quick Start

### Basic Scheduling

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

### Dispatching Rules

```rust
use u_ras::dispatching::{RuleEngine, SchedulingContext, rules};

// Create multi-layer rule engine
let engine = RuleEngine::new()
    .with_rule(rules::CriticalRatio)      // Primary: CR (Critical Ratio)
    .with_tie_breaker(rules::Spt)          // Tie-breaker: SPT
    .with_tie_breaker(rules::Fifo);        // Final: FIFO

// Sort activities by priority
let context = SchedulingContext::default();
let sorted = engine.sort(&activities, &context);
```

### Time Constraints with PERT

```rust
use u_ras::models::time_constraints::{TimeWindow, PertEstimate};

// Create deadline constraint
let window = TimeWindow::deadline(86400000)  // 24 hours
    .soft(1.5);  // Soft constraint with penalty

// PERT 3-point estimation
let pert = PertEstimate::new(
    4000,   // Optimistic: 4 sec
    6000,   // Most Likely: 6 sec
    14000   // Pessimistic: 14 sec
);

println!("Expected duration: {} ms", pert.mean_ms());   // ~7000 ms
println!("95% confidence: {} ms", pert.p95());          // ~9800 ms
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
- `TimeWindow` - Time boundary constraints (hard/soft)
- `PertEstimate` - 3-point duration estimation
- `DurationDistribution` - Probabilistic duration models

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

### dispatching

Priority-based dispatching rules:

#### Time-Based Rules
| Rule | Description |
|------|-------------|
| `Spt` | Shortest Processing Time |
| `Lpt` | Longest Processing Time |
| `Lwkr` | Least Work Remaining |
| `Mwkr` | Most Work Remaining |

#### Due Date Rules
| Rule | Description |
|------|-------------|
| `Edd` | Earliest Due Date |
| `Mst` | Minimum Slack Time |
| `CriticalRatio` | Critical Ratio (CR) |
| `SlackPerOperation` | Slack per Remaining Operation (S/RO) |

#### Queue/Load Rules
| Rule | Description |
|------|-------------|
| `Fifo` | First In First Out |
| `Winq` | Work In Next Queue |
| `Lpul` | Least Pool Utilization Level |

#### Advanced Rules
| Rule | Description |
|------|-------------|
| `Atc` | Apparent Tardiness Cost |
| `Wspt` | Weighted Shortest Processing Time |

#### Multi-Layer Strategy

```rust
// Combine rules with tie-breakers
let engine = RuleEngine::new()
    .with_rule(rules::Atc::new(0.5))  // ATC with k-factor
    .with_tie_breaker(rules::Edd)
    .with_tie_breaker(rules::Fifo);
```

### validation

Input validation utilities:

- Resource availability validation
- Constraint consistency checks
- Calendar overlap detection

## Architecture

```
┌─────────────────────────────────────────────────┐
│                 U-RAS Core                      │
├─────────────────────────────────────────────────┤
│  Models: Task, Activity, Resource, TimeWindow   │
│  Algorithms: GA, CP-SAT, Greedy                 │
│  Dispatching: 14+ rules, Multi-layer engine     │
│  FFI: C-compatible interface                    │
└─────────────────────────────────────────────────┘
              ▲           ▲           ▲
              │           │           │
       ┌──────┴───┐ ┌─────┴────┐ ┌────┴─────┐
       │  U-APS   │ │ Medical  │ │ Logistics│
       │(Manufact)│ │ Scheduler│ │ Planner  │
       └──────────┘ └──────────┘ └──────────┘
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
|--------------|---------------|--------|--------|
| 10 jobs, 5 resources | 50ms | 20ms | 1ms |
| 100 jobs, 20 resources | 500ms | 2s | 10ms |
| 500 jobs, 50 resources | 5s | timeout | 100ms |

## Changelog

### v0.2.0 (2025-12)

- **Dispatching Rules Module**: 14+ priority rules with multi-layer strategy support
  - Time-based: SPT, LPT, LWKR, MWKR
  - Due date: EDD, MST, CR, S/RO
  - Queue/Load: FIFO, WINQ, LPUL
  - Advanced: ATC, WSPT
- **Time Constraints**: TimeWindow with hard/soft constraints and penalty system
- **PERT Estimation**: 3-point duration estimation with confidence intervals
- **Duration Distribution**: Fixed, PERT, Uniform, Triangular, LogNormal
- **Rule Engine**: Composable rules with tie-breaker chain

### v0.1.1

- Initial release with GA, CP-SAT, and basic scheduling

## License

Licensed under either of:

- MIT license ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Related Projects

- [U-APS](https://github.com/iyulab/U-APS-releases) - Manufacturing scheduling system built on U-RAS
