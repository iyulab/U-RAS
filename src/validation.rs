//! Validation - Input validation for scheduling
//!
//! Ensures data integrity before scheduling

use crate::models::{Resource, Task};

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub entity_id: Option<String>,
}

impl ValidationResult {
    pub fn ok() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn with_error(mut self, code: &str, message: &str) -> Self {
        self.is_valid = false;
        self.errors.push(ValidationError {
            code: code.to_string(),
            message: message.to_string(),
            entity_id: None,
        });
        self
    }
}

/// Validate scheduling input
pub fn validate_input(tasks: &[Task], resources: &[Resource]) -> ValidationResult {
    let mut result = ValidationResult::ok();

    // Check for duplicate task IDs
    let mut task_ids = std::collections::HashSet::new();
    for task in tasks {
        if !task_ids.insert(&task.id) {
            result =
                result.with_error("DUPLICATE_TASK", &format!("Duplicate task ID: {}", task.id));
        }
    }

    // Check for duplicate resource IDs
    let mut resource_ids = std::collections::HashSet::new();
    for resource in resources {
        if !resource_ids.insert(resource.id.clone()) {
            result = result.with_error(
                "DUPLICATE_RESOURCE",
                &format!("Duplicate resource ID: {}", resource.id),
            );
        }
    }

    // Check activity references
    for task in tasks {
        for activity in &task.activities {
            let candidates = activity.candidate_resources();
            for candidate in &candidates {
                if !resource_ids.contains(candidate) {
                    result = result.with_error(
                        "INVALID_RESOURCE_REF",
                        &format!(
                            "Activity {} references unknown resource {}",
                            activity.id, candidate
                        ),
                    );
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Activity;

    #[test]
    fn test_valid_input() {
        let tasks = vec![Task::new("T1").with_activity(
            Activity::new("A1", "T1", 1).with_resources("machine", vec!["M1".into()]),
        )];
        let resources = vec![Resource::primary("M1")];

        let result = validate_input(&tasks, &resources);
        assert!(result.is_valid);
    }

    #[test]
    fn test_duplicate_task_id() {
        let tasks = vec![Task::new("T1"), Task::new("T1")];
        let resources = vec![];

        let result = validate_input(&tasks, &resources);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_invalid_resource_reference() {
        let tasks = vec![Task::new("T1").with_activity(
            Activity::new("A1", "T1", 1).with_resources("machine", vec!["UNKNOWN".into()]),
        )];
        let resources = vec![Resource::primary("M1")];

        let result = validate_input(&tasks, &resources);
        assert!(!result.is_valid);
    }
}
