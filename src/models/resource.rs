//! Resource - Abstract Allocatable Entity
//!
//! Domain-agnostic representation of resources

use super::calendar::Calendar;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Resource - An entity that can be allocated to activities
///
/// Domain mappings:
/// - Manufacturing: Equipment, Worker, Tool
/// - Healthcare: Doctor, Nurse, Operating Room, Equipment
/// - Logistics: Truck, Driver, Warehouse Bay
/// - Education: Classroom, Instructor, Lab Equipment
/// - Cloud: VM, CPU Core, Memory, Storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Resource type/category
    pub resource_type: ResourceType,
    /// Capacity (units available simultaneously)
    pub capacity: i32,
    /// Efficiency factor (1.0 = normal)
    pub efficiency: f64,
    /// Availability calendar
    pub calendar: Option<Calendar>,
    /// Skills/capabilities
    pub skills: Vec<Skill>,
    /// Cost per time unit (optional)
    pub cost_per_hour: Option<f64>,
    /// Custom attributes
    pub attributes: HashMap<String, String>,
}

/// Resource type classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResourceType {
    /// Primary processing resource (machine, room, vehicle)
    Primary,
    /// Secondary/support resource (tool, fixture)
    Secondary,
    /// Human resource (worker, operator)
    Human,
    /// Consumable (material, energy) - quantity decreases
    Consumable,
    /// Custom type
    Custom(String),
}

/// Skill/capability of a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Skill name
    pub name: String,
    /// Proficiency level (0.0 to 1.0)
    pub level: f64,
}

impl Skill {
    pub fn new(name: &str, level: f64) -> Self {
        Self {
            name: name.to_string(),
            level: level.clamp(0.0, 1.0),
        }
    }
}

impl Resource {
    /// Create new resource
    pub fn new(id: &str, resource_type: ResourceType) -> Self {
        Self {
            id: id.to_string(),
            name: id.to_string(),
            resource_type,
            capacity: 1,
            efficiency: 1.0,
            calendar: None,
            skills: Vec::new(),
            cost_per_hour: None,
            attributes: HashMap::new(),
        }
    }

    /// Create primary resource (machine, room)
    pub fn primary(id: &str) -> Self {
        Self::new(id, ResourceType::Primary)
    }

    /// Create human resource
    pub fn human(id: &str) -> Self {
        Self::new(id, ResourceType::Human)
    }

    /// Create secondary resource (tool)
    pub fn secondary(id: &str) -> Self {
        Self::new(id, ResourceType::Secondary)
    }

    /// Set name
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// Set capacity
    pub fn with_capacity(mut self, capacity: i32) -> Self {
        self.capacity = capacity;
        self
    }

    /// Set efficiency
    pub fn with_efficiency(mut self, efficiency: f64) -> Self {
        self.efficiency = efficiency;
        self
    }

    /// Set calendar
    pub fn with_calendar(mut self, calendar: Calendar) -> Self {
        self.calendar = Some(calendar);
        self
    }

    /// Add skill
    pub fn with_skill(mut self, name: &str, level: f64) -> Self {
        self.skills.push(Skill::new(name, level));
        self
    }

    /// Set cost
    pub fn with_cost(mut self, cost_per_hour: f64) -> Self {
        self.cost_per_hour = Some(cost_per_hour);
        self
    }

    /// Add attribute
    pub fn with_attribute(mut self, key: &str, value: &str) -> Self {
        self.attributes.insert(key.to_string(), value.to_string());
        self
    }

    /// Check if resource has skill
    pub fn has_skill(&self, skill_name: &str) -> bool {
        self.skills.iter().any(|s| s.name == skill_name)
    }

    /// Get skill level (0.0 if not present)
    pub fn skill_level(&self, skill_name: &str) -> f64 {
        self.skills
            .iter()
            .find(|s| s.name == skill_name)
            .map(|s| s.level)
            .unwrap_or(0.0)
    }

    /// Check if available at time
    pub fn is_available_at(&self, timestamp_ms: i64) -> bool {
        match &self.calendar {
            Some(cal) => cal.is_working_time(timestamp_ms),
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_creation() {
        let resource = Resource::primary("M1")
            .with_name("Machine 1")
            .with_efficiency(0.9)
            .with_capacity(2);

        assert_eq!(resource.id, "M1");
        assert_eq!(resource.efficiency, 0.9);
        assert_eq!(resource.capacity, 2);
    }

    #[test]
    fn test_resource_skills() {
        let resource = Resource::human("W1")
            .with_skill("welding", 0.8)
            .with_skill("assembly", 0.6);

        assert!(resource.has_skill("welding"));
        assert!(!resource.has_skill("painting"));
        assert_eq!(resource.skill_level("welding"), 0.8);
        assert_eq!(resource.skill_level("unknown"), 0.0);
    }

    #[test]
    fn test_resource_types() {
        let primary = Resource::primary("P1");
        let human = Resource::human("H1");
        let secondary = Resource::secondary("S1");

        assert_eq!(primary.resource_type, ResourceType::Primary);
        assert_eq!(human.resource_type, ResourceType::Human);
        assert_eq!(secondary.resource_type, ResourceType::Secondary);
    }
}
