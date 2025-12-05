//! SchedulingContext - Runtime state for rule evaluation

use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Context information available during rule evaluation
///
/// Provides the current scheduling state that rules may use
/// to make prioritization decisions.
#[derive(Debug, Clone)]
pub struct SchedulingContext {
    /// Current simulation/scheduling time
    pub current_time: DateTime<Utc>,

    /// Remaining work per task (task_id -> remaining_ms)
    /// Used by LWKR, MWKR rules
    pub remaining_work: HashMap<String, i64>,

    /// Queue length at next operation's resource
    /// Used by WINQ rule
    pub next_queue_length: HashMap<String, usize>,

    /// Resource utilization (resource_id -> load factor 0.0-1.0)
    /// Used by LPUL rule
    pub resource_utilization: HashMap<String, f64>,

    /// Task arrival times for FIFO ordering
    pub arrival_times: HashMap<String, DateTime<Utc>>,

    /// Average processing time across waiting tasks (in ms)
    /// Used by ATC rule for normalization
    pub average_processing_time: Option<f64>,

    /// Custom attributes for domain-specific rules
    pub attributes: HashMap<String, String>,
}

impl SchedulingContext {
    /// Create a new context at the specified time
    pub fn new(current_time: DateTime<Utc>) -> Self {
        Self {
            current_time,
            remaining_work: HashMap::new(),
            next_queue_length: HashMap::new(),
            resource_utilization: HashMap::new(),
            arrival_times: HashMap::new(),
            average_processing_time: None,
            attributes: HashMap::new(),
        }
    }

    /// Create context at epoch (time = 0)
    pub fn at_epoch() -> Self {
        Self::new(DateTime::from_timestamp_millis(0).unwrap())
    }

    /// Set remaining work for a task
    pub fn with_remaining_work(mut self, task_id: impl Into<String>, remaining_ms: i64) -> Self {
        self.remaining_work.insert(task_id.into(), remaining_ms);
        self
    }

    /// Set arrival time for a task
    pub fn with_arrival_time(mut self, task_id: impl Into<String>, time: DateTime<Utc>) -> Self {
        self.arrival_times.insert(task_id.into(), time);
        self
    }

    /// Set queue length for next operation
    pub fn with_next_queue(mut self, task_id: impl Into<String>, length: usize) -> Self {
        self.next_queue_length.insert(task_id.into(), length);
        self
    }

    /// Set resource utilization
    pub fn with_utilization(mut self, resource_id: impl Into<String>, load: f64) -> Self {
        self.resource_utilization.insert(resource_id.into(), load);
        self
    }

    /// Set average processing time (for ATC rule normalization)
    pub fn with_average_processing_time(mut self, avg_ms: f64) -> Self {
        self.average_processing_time = Some(avg_ms);
        self
    }

    /// Get remaining work for a task (defaults to 0)
    pub fn get_remaining_work(&self, task_id: &str) -> i64 {
        self.remaining_work.get(task_id).copied().unwrap_or(0)
    }

    /// Get arrival time for a task
    pub fn get_arrival_time(&self, task_id: &str) -> Option<DateTime<Utc>> {
        self.arrival_times.get(task_id).copied()
    }
}

impl Default for SchedulingContext {
    fn default() -> Self {
        Self::at_epoch()
    }
}
