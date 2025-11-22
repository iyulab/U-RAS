//! CP Variables - Interval Variables and Constraints

use serde::{Deserialize, Serialize};

/// Interval Variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntervalVar {
    /// Variable name
    pub name: String,
    /// Start time (ms)
    pub start: TimeVar,
    /// End time (ms)
    pub end: TimeVar,
    /// Duration (ms)
    pub duration: DurationVar,
    /// Optional execution
    pub is_optional: bool,
    /// Presence literal
    pub presence: Option<BoolVar>,
}

/// Time variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeVar {
    /// Minimum value
    pub min: i64,
    /// Maximum value
    pub max: i64,
    /// Fixed value (if any)
    pub fixed: Option<i64>,
}

impl TimeVar {
    pub fn new(min: i64, max: i64) -> Self {
        Self {
            min,
            max,
            fixed: None,
        }
    }

    pub fn fixed(value: i64) -> Self {
        Self {
            min: value,
            max: value,
            fixed: Some(value),
        }
    }

    pub fn is_fixed(&self) -> bool {
        self.fixed.is_some()
    }
}

/// Duration variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurationVar {
    /// Minimum duration
    pub min: i64,
    /// Maximum duration
    pub max: i64,
    /// Fixed duration
    pub fixed: Option<i64>,
}

impl DurationVar {
    pub fn new(min: i64, max: i64) -> Self {
        Self {
            min,
            max,
            fixed: None,
        }
    }

    pub fn fixed(value: i64) -> Self {
        Self {
            min: value,
            max: value,
            fixed: Some(value),
        }
    }
}

/// Boolean variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoolVar {
    pub name: String,
    pub fixed: Option<bool>,
}

impl BoolVar {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fixed: None,
        }
    }
}

/// Integer variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntVar {
    pub name: String,
    pub min: i64,
    pub max: i64,
    pub fixed: Option<i64>,
}

impl IntVar {
    pub fn new(name: impl Into<String>, min: i64, max: i64) -> Self {
        Self {
            name: name.into(),
            min,
            max,
            fixed: None,
        }
    }
}

impl IntervalVar {
    /// Create new interval variable
    pub fn new(
        name: impl Into<String>,
        start_min: i64,
        start_max: i64,
        duration: i64,
        end_max: i64,
    ) -> Self {
        Self {
            name: name.into(),
            start: TimeVar::new(start_min, start_max),
            end: TimeVar::new(start_min + duration, end_max),
            duration: DurationVar::fixed(duration),
            is_optional: false,
            presence: None,
        }
    }

    /// Convert to optional interval variable
    pub fn as_optional(mut self, presence_name: impl Into<String>) -> Self {
        self.is_optional = true;
        self.presence = Some(BoolVar::new(presence_name));
        self
    }

    /// Set variable duration
    pub fn with_variable_duration(mut self, min: i64, max: i64) -> Self {
        self.duration = DurationVar::new(min, max);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_var_creation() {
        let var = IntervalVar::new("op1", 0, 100, 50, 200);
        assert_eq!(var.name, "op1");
        assert_eq!(var.start.min, 0);
        assert_eq!(var.start.max, 100);
        assert_eq!(var.duration.fixed, Some(50));
    }

    #[test]
    fn test_optional_interval() {
        let var = IntervalVar::new("op1", 0, 100, 50, 200).as_optional("op1_present");
        assert!(var.is_optional);
        assert!(var.presence.is_some());
    }
}
