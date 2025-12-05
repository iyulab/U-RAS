//! Built-in dispatching rules
//!
//! # Time-based Rules
//! - [`Spt`] - Shortest Processing Time
//! - [`Lpt`] - Longest Processing Time
//! - [`Lwkr`] - Least Work Remaining
//! - [`Mwkr`] - Most Work Remaining
//! - [`Wspt`] - Weighted Shortest Processing Time
//!
//! # Due Date Rules
//! - [`Edd`] - Earliest Due Date
//! - [`Mst`] - Minimum Slack Time
//! - [`Cr`] - Critical Ratio
//! - [`Sro`] - Slack per Remaining Operations
//! - [`Atc`] - Apparent Tardiness Cost
//!
//! # Queue/Load Rules
//! - [`Fifo`] - First In First Out
//! - [`Winq`] - Work In Next Queue
//! - [`Lpul`] - Least Planned Utilization Level

mod time_based;
mod due_date;
mod queue_load;

pub use time_based::*;
pub use due_date::*;
pub use queue_load::*;
