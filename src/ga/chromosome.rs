//! Chromosome - Dual-Vector Encoding for Scheduling
//!
//! Dual-vector representation:
//! - OSV (Operation Sequence Vector): Activity execution order
//! - MAV (Machine Assignment Vector): Resource assignment

use std::collections::HashMap;
use rand::prelude::*;
use crate::models::{Resource, ResourceType};

/// Chromosome with dual-vector encoding
#[derive(Debug, Clone, PartialEq)]
pub struct Chromosome {
    /// Operation Sequence Vector - activity execution order
    /// Represented as task ID permutation, k-th occurrence = k-th activity
    pub osv: Vec<String>,

    /// Machine Assignment Vector - resource for each activity
    /// Index: fixed order of all activities (Task1-A1, Task1-A2, ..., TaskN-AM)
    pub mav: Vec<String>,

    /// Activity index mapping: (task_id, sequence) -> mav_index
    pub activity_index: HashMap<(String, i32), usize>,

    /// Fitness (lower is better)
    pub fitness: f64,
}

/// Activity information for chromosome operations
#[derive(Debug, Clone)]
pub struct ActivityInfo {
    pub task_id: String,
    pub activity_id: String,
    pub sequence: i32,
    pub candidates: Vec<String>,
    pub process_time_ms: i64,
}

impl Chromosome {
    /// Create random chromosome
    pub fn random(activities: &[ActivityInfo], rng: &mut impl Rng) -> Self {
        let (osv, activity_index) = Self::create_random_osv(activities, rng);
        let mav = Self::create_random_mav(activities, rng);

        Self {
            osv,
            mav,
            activity_index,
            fitness: f64::INFINITY,
        }
    }

    /// Create chromosome with load balancing
    pub fn with_load_balancing(
        activities: &[ActivityInfo],
        resources: &[Resource],
        rng: &mut impl Rng,
    ) -> Self {
        let (osv, activity_index) = Self::create_random_osv(activities, rng);
        let mav = Self::create_load_balanced_mav(activities, resources);

        Self {
            osv,
            mav,
            activity_index,
            fitness: f64::INFINITY,
        }
    }

    /// Create chromosome with shortest processing time
    pub fn with_shortest_time(
        activities: &[ActivityInfo],
        process_times: &std::collections::HashMap<(String, String), i64>,
        rng: &mut impl Rng,
    ) -> Self {
        let (osv, activity_index) = Self::create_random_osv(activities, rng);
        let mav = Self::create_shortest_time_mav(activities, process_times);

        Self {
            osv,
            mav,
            activity_index,
            fitness: f64::INFINITY,
        }
    }

    /// Create MAV - shortest processing time
    fn create_shortest_time_mav(
        activities: &[ActivityInfo],
        process_times: &std::collections::HashMap<(String, String), i64>,
    ) -> Vec<String> {
        activities
            .iter()
            .map(|act| {
                if act.candidates.is_empty() {
                    return "NONE".to_string();
                }

                // Select resource with shortest processing time
                act.candidates
                    .iter()
                    .min_by_key(|c| {
                        process_times
                            .get(&(act.activity_id.clone(), (*c).clone()))
                            .unwrap_or(&i64::MAX)
                    })
                    .cloned()
                    .unwrap_or_else(|| act.candidates[0].clone())
            })
            .collect()
    }

    /// Create OSV - random order
    fn create_random_osv(
        activities: &[ActivityInfo],
        rng: &mut impl Rng,
    ) -> (Vec<String>, HashMap<(String, i32), usize>) {
        let mut task_activity_counts: HashMap<String, i32> = HashMap::new();
        let mut activity_index: HashMap<(String, i32), usize> = HashMap::new();

        for (idx, act) in activities.iter().enumerate() {
            *task_activity_counts.entry(act.task_id.clone()).or_insert(0) += 1;
            activity_index.insert((act.task_id.clone(), act.sequence), idx);
        }

        // Build OSV: each task ID appears once per activity
        let mut osv: Vec<String> = Vec::new();
        for (task_id, count) in &task_activity_counts {
            for _ in 0..*count {
                osv.push(task_id.clone());
            }
        }

        osv.shuffle(rng);
        (osv, activity_index)
    }

    /// Create MAV - random assignment
    fn create_random_mav(activities: &[ActivityInfo], rng: &mut impl Rng) -> Vec<String> {
        activities
            .iter()
            .map(|act| {
                if act.candidates.is_empty() {
                    "NONE".to_string()
                } else {
                    act.candidates.choose(rng).unwrap().clone()
                }
            })
            .collect()
    }

    /// Create MAV - load balanced
    fn create_load_balanced_mav(
        activities: &[ActivityInfo],
        resources: &[Resource],
    ) -> Vec<String> {
        let mut resource_load: HashMap<String, i64> = HashMap::new();

        // Initialize primary resources
        for res in resources {
            if res.resource_type == ResourceType::Primary {
                resource_load.insert(res.id.clone(), 0);
            }
        }

        activities
            .iter()
            .map(|act| {
                if act.candidates.is_empty() {
                    return "NONE".to_string();
                }

                // Select resource with lowest load
                let best = act
                    .candidates
                    .iter()
                    .filter(|c| resource_load.contains_key(*c))
                    .min_by_key(|c| resource_load.get(*c).unwrap_or(&i64::MAX))
                    .cloned()
                    .unwrap_or_else(|| act.candidates[0].clone());

                if let Some(load) = resource_load.get_mut(&best) {
                    *load += act.process_time_ms;
                }

                best
            })
            .collect()
    }

    /// Decode OSV to (task_id, sequence) pairs
    pub fn decode_osv(&self) -> Vec<(String, i32)> {
        let mut task_counters: HashMap<String, i32> = HashMap::new();

        self.osv
            .iter()
            .map(|task_id| {
                let seq = task_counters.entry(task_id.clone()).or_insert(0);
                *seq += 1;
                (task_id.clone(), *seq)
            })
            .collect()
    }

    /// Get assigned resource for activity
    pub fn get_assigned_resource(&self, task_id: &str, sequence: i32) -> Option<&String> {
        self.activity_index
            .get(&(task_id.to_string(), sequence))
            .and_then(|idx| self.mav.get(*idx))
    }

    /// Set resource for activity
    pub fn set_resource(&mut self, task_id: &str, sequence: i32, resource_id: String) {
        if let Some(idx) = self.activity_index.get(&(task_id.to_string(), sequence)) {
            if *idx < self.mav.len() {
                self.mav[*idx] = resource_id;
            }
        }
    }

    /// Validate chromosome
    pub fn is_valid(&self, activities: &[ActivityInfo]) -> bool {
        if self.osv.len() != activities.len() || self.mav.len() != activities.len() {
            return false;
        }

        // Check task counts
        let mut task_counts: HashMap<String, i32> = HashMap::new();
        for task_id in &self.osv {
            *task_counts.entry(task_id.clone()).or_insert(0) += 1;
        }

        let mut expected: HashMap<String, i32> = HashMap::new();
        for act in activities {
            *expected.entry(act.task_id.clone()).or_insert(0) += 1;
        }

        if task_counts != expected {
            return false;
        }

        // Check resource candidates
        for (idx, act) in activities.iter().enumerate() {
            if !act.candidates.is_empty() && !act.candidates.contains(&self.mav[idx]) {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_activities() -> Vec<ActivityInfo> {
        vec![
            ActivityInfo {
                task_id: "T1".to_string(),
                activity_id: "T1-A1".to_string(),
                sequence: 1,
                candidates: vec!["R1".to_string(), "R2".to_string()],
                process_time_ms: 30000,
            },
            ActivityInfo {
                task_id: "T1".to_string(),
                activity_id: "T1-A2".to_string(),
                sequence: 2,
                candidates: vec!["R2".to_string(), "R3".to_string()],
                process_time_ms: 45000,
            },
            ActivityInfo {
                task_id: "T2".to_string(),
                activity_id: "T2-A1".to_string(),
                sequence: 1,
                candidates: vec!["R1".to_string(), "R3".to_string()],
                process_time_ms: 20000,
            },
        ]
    }

    #[test]
    fn test_random_chromosome() {
        let activities = create_test_activities();
        let mut rng = rand::thread_rng();

        let chromosome = Chromosome::random(&activities, &mut rng);

        assert_eq!(chromosome.osv.len(), 3);
        assert_eq!(chromosome.mav.len(), 3);
        assert!(chromosome.is_valid(&activities));
    }

    #[test]
    fn test_decode_osv() {
        let activities = create_test_activities();
        let mut rng = rand::thread_rng();

        let chromosome = Chromosome::random(&activities, &mut rng);
        let decoded = chromosome.decode_osv();

        assert_eq!(decoded.len(), 3);

        let t1_count = decoded.iter().filter(|(id, _)| id == "T1").count();
        let t2_count = decoded.iter().filter(|(id, _)| id == "T2").count();

        assert_eq!(t1_count, 2);
        assert_eq!(t2_count, 1);
    }

    #[test]
    fn test_validity() {
        let activities = create_test_activities();
        let mut rng = rand::thread_rng();

        let mut chromosome = Chromosome::random(&activities, &mut rng);
        assert!(chromosome.is_valid(&activities));

        // Invalid resource
        chromosome.mav[0] = "INVALID".to_string();
        assert!(!chromosome.is_valid(&activities));
    }
}
