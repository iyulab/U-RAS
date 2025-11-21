# U-RAS (Universal Resource Allocation and Scheduling)

A domain-agnostic scheduling engine for resource allocation optimization.

## Overview

U-RAS provides core scheduling algorithms and abstractions that can be applied to various domains:

- **Manufacturing** (via U-APS)
- **Healthcare** - Operating room scheduling, staff allocation
- **Logistics** - Route optimization, fleet management
- **Education** - Classroom allocation, exam scheduling
- **Cloud Computing** - VM placement, job scheduling
- **Energy** - Grid optimization, demand response

## Core Concepts

| Concept | Description | Domain Examples |
|---------|-------------|-----------------|
| **Task** | Unit of work to schedule | Job, Surgery, Delivery, Course |
| **Activity** | Step within a task | Operation, Procedure, Leg, Lecture |
| **Resource** | Allocatable entity | Machine, Doctor, Truck, Classroom |
| **Constraint** | Rules for valid schedules | Precedence, Capacity, TimeWindow |
| **Schedule** | Output assignments | Activity → Resource → Time |

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

## Modules

### models
- `Task` - Work unit with activities
- `Activity` - Step requiring resources
- `Resource` - Allocatable entity with skills
- `Calendar` - Time availability
- `Constraint` - Scheduling rules
- `Schedule` - Output assignments

### scheduler
- `SimpleScheduler` - Priority-based greedy
- `ScheduleKpi` - Quality metrics

### ga (planned)
- `GaScheduler` - Genetic Algorithm
- `Nsga2` - Multi-objective optimization
- `HybridGaSa` - GA + Simulated Annealing

### cp (planned)
- `CpSat` - Constraint Programming solver

## Architecture

```
┌─────────────────────────────────────┐
│            U-RAS Core               │
├─────────────────────────────────────┤
│  • Task, Activity, Resource         │
│  • Constraint, Schedule, KPI        │
│  • GA, NSGA-II, CP-SAT algorithms   │
└─────────────────────────────────────┘
          ▲           ▲
          │           │
   ┌──────┴───┐ ┌─────┴────┐
   │  U-APS   │ │ U-Health │
   │(Manufact)│ │(Medical) │
   └──────────┘ └──────────┘
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
u-ras = { git = "https://github.com/iyulab/U-RAS" }
```

## License

MIT
