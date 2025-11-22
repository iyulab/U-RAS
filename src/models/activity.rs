//! Activity - Abstract Step within Task
//!
//! Domain-agnostic representation of work steps

use serde::{Deserialize, Serialize};

/// Activity - A step within a task requiring resources
///
/// Domain mappings:
/// - Manufacturing: Operation, Process Step
/// - Healthcare: Procedure Step, Treatment Phase
/// - Logistics: Transport Leg, Loading/Unloading
/// - Education: Lecture, Lab Session
/// - Cloud: Task Stage, Computation Step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    /// Unique identifier
    pub id: String,
    /// Parent task ID
    pub task_id: String,
    /// Sequence number within task
    pub sequence: i32,
    /// Duration specification
    pub duration: ActivityDuration,
    /// Resource requirements
    pub resource_requirements: Vec<ResourceRequirement>,
    /// Predecessor activity IDs (within same task)
    pub predecessors: Vec<String>,
    /// Can be split across time slots
    pub splittable: bool,
    /// Minimum split size if splittable (ms)
    pub min_split_ms: i64,
    /// Custom attributes
    pub attributes: std::collections::HashMap<String, String>,
}

/// Duration specification for activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDuration {
    /// Setup/preparation time (ms)
    pub setup_ms: i64,
    /// Processing/execution time (ms)
    pub process_ms: i64,
    /// Teardown/cleanup time (ms)
    pub teardown_ms: i64,
}

impl ActivityDuration {
    /// Create with all components
    pub fn new(setup_ms: i64, process_ms: i64, teardown_ms: i64) -> Self {
        Self {
            setup_ms,
            process_ms,
            teardown_ms,
        }
    }

    /// Create fixed duration (process only)
    pub fn fixed(process_ms: i64) -> Self {
        Self {
            setup_ms: 0,
            process_ms,
            teardown_ms: 0,
        }
    }

    /// Total duration
    pub fn total_ms(&self) -> i64 {
        self.setup_ms + self.process_ms + self.teardown_ms
    }
}

impl Default for ActivityDuration {
    fn default() -> Self {
        Self::fixed(0)
    }
}

/// Resource requirement for an activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirement {
    /// Resource type/category
    pub resource_type: String,
    /// Quantity needed
    pub quantity: i32,
    /// Alternative resources that can fulfill this requirement
    pub candidates: Vec<String>,
    /// Required skills/capabilities
    pub required_skills: Vec<String>,
}

impl ResourceRequirement {
    /// Create requirement for resource type
    pub fn new(resource_type: &str) -> Self {
        Self {
            resource_type: resource_type.to_string(),
            quantity: 1,
            candidates: Vec::new(),
            required_skills: Vec::new(),
        }
    }

    /// Set quantity
    pub fn with_quantity(mut self, quantity: i32) -> Self {
        self.quantity = quantity;
        self
    }

    /// Add candidate resource
    pub fn with_candidate(mut self, resource_id: &str) -> Self {
        self.candidates.push(resource_id.to_string());
        self
    }

    /// Add candidates
    pub fn with_candidates(mut self, candidates: Vec<String>) -> Self {
        self.candidates = candidates;
        self
    }

    /// Add required skill
    pub fn with_skill(mut self, skill: &str) -> Self {
        self.required_skills.push(skill.to_string());
        self
    }
}

impl Activity {
    /// Create new activity
    pub fn new(id: &str, task_id: &str, sequence: i32) -> Self {
        Self {
            id: id.to_string(),
            task_id: task_id.to_string(),
            sequence,
            duration: ActivityDuration::default(),
            resource_requirements: Vec::new(),
            predecessors: Vec::new(),
            splittable: false,
            min_split_ms: 0,
            attributes: std::collections::HashMap::new(),
        }
    }

    /// Set duration
    pub fn with_duration(mut self, duration: ActivityDuration) -> Self {
        self.duration = duration;
        self
    }

    /// Set duration with components
    pub fn with_time(mut self, setup_ms: i64, process_ms: i64, teardown_ms: i64) -> Self {
        self.duration = ActivityDuration::new(setup_ms, process_ms, teardown_ms);
        self
    }

    /// Add resource requirement
    pub fn with_requirement(mut self, requirement: ResourceRequirement) -> Self {
        self.resource_requirements.push(requirement);
        self
    }

    /// Add simple resource candidates (shorthand)
    pub fn with_resources(mut self, resource_type: &str, candidates: Vec<String>) -> Self {
        self.resource_requirements
            .push(ResourceRequirement::new(resource_type).with_candidates(candidates));
        self
    }

    /// Add predecessor
    pub fn with_predecessor(mut self, activity_id: &str) -> Self {
        self.predecessors.push(activity_id.to_string());
        self
    }

    /// Enable splitting
    pub fn with_splitting(mut self, min_split_ms: i64) -> Self {
        self.splittable = true;
        self.min_split_ms = min_split_ms;
        self
    }

    /// Add custom attribute
    pub fn with_attribute(mut self, key: &str, value: &str) -> Self {
        self.attributes.insert(key.to_string(), value.to_string());
        self
    }

    /// Get first candidate resource IDs
    pub fn candidate_resources(&self) -> Vec<String> {
        self.resource_requirements
            .iter()
            .flat_map(|req| req.candidates.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_creation() {
        let activity = Activity::new("A1", "T1", 1)
            .with_time(1000, 5000, 500)
            .with_resources("machine", vec!["M1".into(), "M2".into()]);

        assert_eq!(activity.id, "A1");
        assert_eq!(activity.duration.total_ms(), 6500);
        assert_eq!(activity.candidate_resources(), vec!["M1", "M2"]);
    }

    #[test]
    fn test_duration() {
        let duration = ActivityDuration::new(1000, 5000, 500);
        assert_eq!(duration.total_ms(), 6500);

        let fixed = ActivityDuration::fixed(3000);
        assert_eq!(fixed.total_ms(), 3000);
    }

    #[test]
    fn test_resource_requirement() {
        let req = ResourceRequirement::new("equipment")
            .with_quantity(2)
            .with_candidates(vec!["E1".into(), "E2".into()])
            .with_skill("welding");

        assert_eq!(req.quantity, 2);
        assert_eq!(req.candidates.len(), 2);
        assert_eq!(req.required_skills, vec!["welding"]);
    }
}
