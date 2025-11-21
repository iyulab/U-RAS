//! Constraint - Rules and Limitations
//!
//! Domain-agnostic constraints for scheduling

use serde::{Deserialize, Serialize};

/// Constraint types for scheduling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constraint {
    /// Activity A must finish before Activity B starts
    Precedence {
        before: String,
        after: String,
        min_delay_ms: i64,
    },
    /// Resource cannot exceed capacity
    Capacity {
        resource_id: String,
        max_capacity: i32,
    },
    /// Activity must occur within time window
    TimeWindow {
        activity_id: String,
        start_ms: i64,
        end_ms: i64,
    },
    /// Activities cannot overlap on resource
    NoOverlap {
        resource_id: String,
        activity_ids: Vec<String>,
    },
    /// Transition cost between activities
    TransitionCost {
        from_category: String,
        to_category: String,
        cost_ms: i64,
    },
    /// Synchronization - activities must start together
    Synchronize {
        activity_ids: Vec<String>,
    },
}

impl Constraint {
    /// Create precedence constraint
    pub fn precedence(before: &str, after: &str) -> Self {
        Constraint::Precedence {
            before: before.to_string(),
            after: after.to_string(),
            min_delay_ms: 0,
        }
    }

    /// Create precedence with delay
    pub fn precedence_with_delay(before: &str, after: &str, delay_ms: i64) -> Self {
        Constraint::Precedence {
            before: before.to_string(),
            after: after.to_string(),
            min_delay_ms: delay_ms,
        }
    }

    /// Create capacity constraint
    pub fn capacity(resource_id: &str, max: i32) -> Self {
        Constraint::Capacity {
            resource_id: resource_id.to_string(),
            max_capacity: max,
        }
    }

    /// Create time window constraint
    pub fn time_window(activity_id: &str, start_ms: i64, end_ms: i64) -> Self {
        Constraint::TimeWindow {
            activity_id: activity_id.to_string(),
            start_ms,
            end_ms,
        }
    }

    /// Create no-overlap constraint
    pub fn no_overlap(resource_id: &str, activity_ids: Vec<String>) -> Self {
        Constraint::NoOverlap {
            resource_id: resource_id.to_string(),
            activity_ids,
        }
    }

    /// Create transition cost
    pub fn transition_cost(from: &str, to: &str, cost_ms: i64) -> Self {
        Constraint::TransitionCost {
            from_category: from.to_string(),
            to_category: to.to_string(),
            cost_ms,
        }
    }
}

/// Transition matrix for sequence-dependent setup times
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionMatrix {
    /// Matrix name
    pub name: String,
    /// Resource this applies to
    pub resource_id: String,
    /// Transition times: (from_category, to_category) -> time_ms
    pub transitions: std::collections::HashMap<(String, String), i64>,
    /// Default transition time
    pub default_ms: i64,
}

impl TransitionMatrix {
    /// Create new transition matrix
    pub fn new(name: &str, resource_id: &str) -> Self {
        Self {
            name: name.to_string(),
            resource_id: resource_id.to_string(),
            transitions: std::collections::HashMap::new(),
            default_ms: 0,
        }
    }

    /// Set transition time
    pub fn set_transition(&mut self, from: &str, to: &str, time_ms: i64) {
        self.transitions.insert(
            (from.to_string(), to.to_string()),
            time_ms,
        );
    }

    /// Get transition time
    pub fn get_transition(&self, from: &str, to: &str) -> i64 {
        self.transitions
            .get(&(from.to_string(), to.to_string()))
            .copied()
            .unwrap_or(self.default_ms)
    }

    /// Set default transition time
    pub fn with_default(mut self, default_ms: i64) -> Self {
        self.default_ms = default_ms;
        self
    }
}

/// Collection of transition matrices
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransitionMatrixCollection {
    pub matrices: Vec<TransitionMatrix>,
}

impl TransitionMatrixCollection {
    pub fn new() -> Self {
        Self { matrices: Vec::new() }
    }

    pub fn add(&mut self, matrix: TransitionMatrix) {
        self.matrices.push(matrix);
    }

    pub fn get_for_resource(&self, resource_id: &str) -> Option<&TransitionMatrix> {
        self.matrices.iter().find(|m| m.resource_id == resource_id)
    }

    pub fn get_transition_time(&self, resource_id: &str, from: &str, to: &str) -> i64 {
        self.get_for_resource(resource_id)
            .map(|m| m.get_transition(from, to))
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraints() {
        let prec = Constraint::precedence("A1", "A2");
        let cap = Constraint::capacity("R1", 5);

        match prec {
            Constraint::Precedence { before, after, .. } => {
                assert_eq!(before, "A1");
                assert_eq!(after, "A2");
            }
            _ => panic!("Wrong constraint type"),
        }

        match cap {
            Constraint::Capacity { max_capacity, .. } => {
                assert_eq!(max_capacity, 5);
            }
            _ => panic!("Wrong constraint type"),
        }
    }

    #[test]
    fn test_transition_matrix() {
        let mut matrix = TransitionMatrix::new("setup", "M1").with_default(1000);
        matrix.set_transition("A", "B", 5000);
        matrix.set_transition("B", "A", 3000);

        assert_eq!(matrix.get_transition("A", "B"), 5000);
        assert_eq!(matrix.get_transition("B", "A"), 3000);
        assert_eq!(matrix.get_transition("A", "C"), 1000); // default
    }
}
