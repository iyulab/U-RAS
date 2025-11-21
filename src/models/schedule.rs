//! Schedule - Output of scheduling algorithms
//!
//! Represents resource allocations and timing decisions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Schedule - The result of a scheduling operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    /// Activity assignments
    pub assignments: Vec<Assignment>,
    /// Total completion time (makespan)
    pub makespan_ms: i64,
    /// Constraint violations (if any)
    pub violations: Vec<Violation>,
}

/// Assignment - Allocation of an activity to a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    /// Activity ID
    pub activity_id: String,
    /// Parent task ID
    pub task_id: String,
    /// Assigned resource ID
    pub resource_id: String,
    /// Start time (epoch ms)
    pub start_ms: i64,
    /// End time (epoch ms)
    pub end_ms: i64,
    /// Setup/transition time (ms)
    pub setup_ms: i64,
}

/// Constraint violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// Violation type
    pub violation_type: ViolationType,
    /// Related entity ID
    pub entity_id: String,
    /// Description
    pub message: String,
    /// Severity (0-100)
    pub severity: i32,
}

/// Types of violations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ViolationType {
    /// Deadline missed
    DeadlineMiss,
    /// Resource overloaded
    CapacityExceeded,
    /// Precedence violated
    PrecedenceViolation,
    /// Resource unavailable
    ResourceUnavailable,
    /// Skill mismatch
    SkillMismatch,
    /// Custom violation
    Custom(String),
}

impl Assignment {
    /// Create new assignment
    pub fn new(
        activity_id: &str,
        task_id: &str,
        resource_id: &str,
        start_ms: i64,
        end_ms: i64,
    ) -> Self {
        Self {
            activity_id: activity_id.to_string(),
            task_id: task_id.to_string(),
            resource_id: resource_id.to_string(),
            start_ms,
            end_ms,
            setup_ms: 0,
        }
    }

    /// Set setup time
    pub fn with_setup(mut self, setup_ms: i64) -> Self {
        self.setup_ms = setup_ms;
        self
    }

    /// Duration in milliseconds
    pub fn duration_ms(&self) -> i64 {
        self.end_ms - self.start_ms
    }

    /// Processing time (excluding setup)
    pub fn process_ms(&self) -> i64 {
        self.end_ms - self.start_ms - self.setup_ms
    }
}

impl Schedule {
    /// Create empty schedule
    pub fn new() -> Self {
        Self {
            assignments: Vec::new(),
            makespan_ms: 0,
            violations: Vec::new(),
        }
    }

    /// Add assignment
    pub fn add_assignment(&mut self, assignment: Assignment) {
        if assignment.end_ms > self.makespan_ms {
            self.makespan_ms = assignment.end_ms;
        }
        self.assignments.push(assignment);
    }

    /// Add violation
    pub fn add_violation(&mut self, violation: Violation) {
        self.violations.push(violation);
    }

    /// Check if valid (no violations)
    pub fn is_valid(&self) -> bool {
        self.violations.is_empty()
    }

    /// Get assignment for activity
    pub fn assignment_for_activity(&self, activity_id: &str) -> Option<&Assignment> {
        self.assignments.iter().find(|a| a.activity_id == activity_id)
    }

    /// Get assignments for task
    pub fn assignments_for_task(&self, task_id: &str) -> Vec<&Assignment> {
        self.assignments.iter().filter(|a| a.task_id == task_id).collect()
    }

    /// Get assignments for resource
    pub fn assignments_for_resource(&self, resource_id: &str) -> Vec<&Assignment> {
        self.assignments.iter().filter(|a| a.resource_id == resource_id).collect()
    }

    /// Calculate resource utilization
    pub fn resource_utilization(&self, resource_id: &str, horizon_ms: i64) -> f64 {
        if horizon_ms == 0 {
            return 0.0;
        }

        let busy_time: i64 = self.assignments_for_resource(resource_id)
            .iter()
            .map(|a| a.duration_ms())
            .sum();

        busy_time as f64 / horizon_ms as f64
    }

    /// Calculate all resource utilizations
    pub fn all_utilizations(&self) -> HashMap<String, f64> {
        let mut utilizations = HashMap::new();

        if self.makespan_ms == 0 {
            return utilizations;
        }

        for assignment in &self.assignments {
            let entry = utilizations.entry(assignment.resource_id.clone()).or_insert(0.0);
            *entry += assignment.duration_ms() as f64 / self.makespan_ms as f64;
        }

        utilizations
    }

    /// Get completion time for task
    pub fn task_completion_time(&self, task_id: &str) -> Option<i64> {
        self.assignments_for_task(task_id)
            .iter()
            .map(|a| a.end_ms)
            .max()
    }

    /// Total number of assignments
    pub fn assignment_count(&self) -> usize {
        self.assignments.len()
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Self::new()
    }
}

impl Violation {
    /// Create deadline miss violation
    pub fn deadline_miss(task_id: &str, message: &str) -> Self {
        Self {
            violation_type: ViolationType::DeadlineMiss,
            entity_id: task_id.to_string(),
            message: message.to_string(),
            severity: 80,
        }
    }

    /// Create capacity exceeded violation
    pub fn capacity_exceeded(resource_id: &str, message: &str) -> Self {
        Self {
            violation_type: ViolationType::CapacityExceeded,
            entity_id: resource_id.to_string(),
            message: message.to_string(),
            severity: 90,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_creation() {
        let mut schedule = Schedule::new();

        schedule.add_assignment(Assignment::new("A1", "T1", "R1", 0, 5000));
        schedule.add_assignment(Assignment::new("A2", "T1", "R1", 5000, 8000));

        assert_eq!(schedule.makespan_ms, 8000);
        assert_eq!(schedule.assignment_count(), 2);
        assert!(schedule.is_valid());
    }

    #[test]
    fn test_utilization() {
        let mut schedule = Schedule::new();

        schedule.add_assignment(Assignment::new("A1", "T1", "R1", 0, 5000));
        schedule.add_assignment(Assignment::new("A2", "T2", "R1", 5000, 10000));

        assert_eq!(schedule.resource_utilization("R1", 10000), 1.0);
        assert_eq!(schedule.resource_utilization("R1", 20000), 0.5);
    }

    #[test]
    fn test_violations() {
        let mut schedule = Schedule::new();
        schedule.add_violation(Violation::deadline_miss("T1", "Late by 1 hour"));

        assert!(!schedule.is_valid());
        assert_eq!(schedule.violations.len(), 1);
    }
}
