//! U-RAS - Universal Resource Allocation and Scheduling
//!
//! A domain-agnostic scheduling engine for resource allocation optimization.
//!
//! # Overview
//!
//! U-RAS provides core scheduling algorithms and abstractions that can be
//! applied to various domains including:
//!
//! - Manufacturing (via U-APS)
//! - Healthcare scheduling
//! - Logistics and transportation
//! - Education (classroom allocation)
//! - Cloud computing resource management
//! - Energy grid optimization
//!
//! # Core Concepts
//!
//! - **Task**: A unit of work to be scheduled
//! - **Activity**: A step within a task requiring resources
//! - **Resource**: An entity that can be allocated (machine, person, room)
//! - **Constraint**: Rules governing valid schedules
//! - **Schedule**: The output - assignments of activities to resources over time
//!
//! # Example
//!
//! ```rust
//! use u_ras::models::{Task, Activity, Resource, ActivityDuration};
//! use u_ras::scheduler::SimpleScheduler;
//!
//! // Create tasks
//! let task = Task::new("T1")
//!     .with_priority(5)
//!     .with_activity(
//!         Activity::new("A1", "T1", 1)
//!             .with_duration(ActivityDuration::fixed(5000))
//!             .with_resources("machine", vec!["M1".into(), "M2".into()])
//!     );
//!
//! // Create resources
//! let resource = Resource::primary("M1")
//!     .with_efficiency(1.0);
//!
//! // Schedule
//! let scheduler = SimpleScheduler::new();
//! let schedule = scheduler.schedule(&[task], &[resource], 0);
//!
//! assert!(schedule.makespan_ms > 0);
//! ```

pub mod models;
pub mod scheduler;
pub mod ga;
pub mod cp;
pub mod validation;

pub use models::*;
pub use scheduler::*;
pub use ga::*;
pub use cp::*;
