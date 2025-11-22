//! Task - Abstract Work Unit
//!
//! Domain-agnostic representation of schedulable work

use super::activity::Activity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Task - Abstract schedulable work unit
///
/// Domain mappings:
/// - Manufacturing: Job, Work Order
/// - Healthcare: Patient Case, Procedure
/// - Logistics: Shipment, Delivery Order
/// - Education: Course, Exam Session
/// - Cloud: Job, Workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Task category/type
    pub category: String,
    /// Priority (higher = more important)
    pub priority: i32,
    /// Deadline for completion
    pub deadline: Option<DateTime<Utc>>,
    /// Earliest start time
    pub release_time: Option<DateTime<Utc>>,
    /// Activities that comprise this task
    pub activities: Vec<Activity>,
    /// Custom attributes for domain-specific data
    pub attributes: std::collections::HashMap<String, String>,
}

impl Task {
    /// Create new task with ID
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            name: id.to_string(),
            category: String::new(),
            priority: 1,
            deadline: None,
            release_time: None,
            activities: Vec::new(),
            attributes: std::collections::HashMap::new(),
        }
    }

    /// Set task name
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// Set category
    pub fn with_category(mut self, category: &str) -> Self {
        self.category = category.to_string();
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Set deadline
    pub fn with_deadline(mut self, deadline: DateTime<Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Set release time
    pub fn with_release_time(mut self, release_time: DateTime<Utc>) -> Self {
        self.release_time = Some(release_time);
        self
    }

    /// Add activity
    pub fn with_activity(mut self, activity: Activity) -> Self {
        self.activities.push(activity);
        self
    }

    /// Add custom attribute
    pub fn with_attribute(mut self, key: &str, value: &str) -> Self {
        self.attributes.insert(key.to_string(), value.to_string());
        self
    }

    /// Get total estimated duration
    pub fn total_duration_ms(&self) -> i64 {
        self.activities.iter().map(|a| a.duration.process_ms).sum()
    }

    /// Check if task has activities
    pub fn has_activities(&self) -> bool {
        !self.activities.is_empty()
    }
}

impl Default for Task {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::activity::ActivityDuration;

    #[test]
    fn test_task_creation() {
        let task = Task::new("T1")
            .with_name("Test Task")
            .with_priority(5)
            .with_category("urgent");

        assert_eq!(task.id, "T1");
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.priority, 5);
    }

    #[test]
    fn test_task_with_activities() {
        let task = Task::new("T1")
            .with_activity(
                Activity::new("A1", "T1", 1).with_duration(ActivityDuration::fixed(5000)),
            )
            .with_activity(
                Activity::new("A2", "T1", 2).with_duration(ActivityDuration::fixed(3000)),
            );

        assert_eq!(task.activities.len(), 2);
        assert_eq!(task.total_duration_ms(), 8000);
    }
}
