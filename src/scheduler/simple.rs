//! Simple Scheduler - Priority-based greedy scheduling
//!
//! Fast heuristic scheduler for baseline solutions

use std::collections::HashMap;
use crate::models::{Task, Resource, Schedule, Assignment, TransitionMatrixCollection};

/// Simple priority-based scheduler
pub struct SimpleScheduler {
    /// Transition matrices for setup times
    transition_matrices: TransitionMatrixCollection,
}

/// Request for scheduling
pub struct ScheduleRequest {
    pub tasks: Vec<Task>,
    pub resources: Vec<Resource>,
    pub start_time_ms: i64,
    pub transition_matrices: TransitionMatrixCollection,
}

impl ScheduleRequest {
    pub fn new(tasks: Vec<Task>, resources: Vec<Resource>) -> Self {
        Self {
            tasks,
            resources,
            start_time_ms: 0,
            transition_matrices: TransitionMatrixCollection::new(),
        }
    }

    pub fn with_start_time(mut self, start_time_ms: i64) -> Self {
        self.start_time_ms = start_time_ms;
        self
    }

    pub fn with_transition_matrices(mut self, matrices: TransitionMatrixCollection) -> Self {
        self.transition_matrices = matrices;
        self
    }
}

impl SimpleScheduler {
    /// Create new simple scheduler
    pub fn new() -> Self {
        Self {
            transition_matrices: TransitionMatrixCollection::new(),
        }
    }

    /// Set transition matrices
    pub fn with_transition_matrices(mut self, matrices: TransitionMatrixCollection) -> Self {
        self.transition_matrices = matrices;
        self
    }

    /// Schedule tasks on resources
    pub fn schedule(
        &self,
        tasks: &[Task],
        resources: &[Resource],
        start_time_ms: i64,
    ) -> Schedule {
        let mut schedule = Schedule::new();
        let mut resource_available: HashMap<String, i64> = HashMap::new();
        let mut last_category: HashMap<String, String> = HashMap::new();

        // Initialize resource availability
        for resource in resources {
            resource_available.insert(resource.id.clone(), start_time_ms);
        }

        // Sort tasks by priority (descending)
        let mut sorted_tasks: Vec<&Task> = tasks.iter().collect();
        sorted_tasks.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Schedule each task
        for task in sorted_tasks {
            let mut task_start = start_time_ms;

            for activity in &task.activities {
                // Find best resource
                let candidates = activity.candidate_resources();
                if candidates.is_empty() {
                    continue;
                }

                // Select resource with earliest availability
                let mut best_resource: Option<&str> = None;
                let mut best_start = i64::MAX;

                for candidate in &candidates {
                    if let Some(&available) = resource_available.get(candidate) {
                        let actual_start = available.max(task_start);
                        if actual_start < best_start {
                            best_start = actual_start;
                            best_resource = Some(candidate);
                        }
                    }
                }

                if let Some(resource_id) = best_resource {
                    // Calculate setup time
                    let setup_time = if let Some(prev_cat) = last_category.get(resource_id) {
                        self.transition_matrices.get_transition_time(
                            resource_id,
                            prev_cat,
                            &task.category,
                        )
                    } else {
                        0
                    };

                    let start = best_start;
                    let end = start + setup_time + activity.duration.process_ms;

                    // Create assignment
                    let assignment = Assignment {
                        activity_id: activity.id.clone(),
                        task_id: task.id.clone(),
                        resource_id: resource_id.to_string(),
                        start_ms: start,
                        end_ms: end,
                        setup_ms: setup_time,
                    };

                    schedule.add_assignment(assignment);

                    // Update state
                    resource_available.insert(resource_id.to_string(), end);
                    last_category.insert(resource_id.to_string(), task.category.clone());
                    task_start = end; // Next activity can't start before this one ends
                }
            }
        }

        schedule
    }

    /// Schedule from request
    pub fn schedule_request(&self, request: &ScheduleRequest) -> Schedule {
        let scheduler = self.clone()
            .with_transition_matrices(request.transition_matrices.clone());

        scheduler.schedule(&request.tasks, &request.resources, request.start_time_ms)
    }
}

impl Default for SimpleScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SimpleScheduler {
    fn clone(&self) -> Self {
        Self {
            transition_matrices: self.transition_matrices.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Activity, ActivityDuration};

    fn create_test_scenario() -> (Vec<Task>, Vec<Resource>) {
        let tasks = vec![
            Task::new("T1")
                .with_priority(5)
                .with_activity(
                    Activity::new("T1-A1", "T1", 1)
                        .with_duration(ActivityDuration::fixed(5000))
                        .with_resources("machine", vec!["M1".into(), "M2".into()])
                ),
            Task::new("T2")
                .with_priority(3)
                .with_activity(
                    Activity::new("T2-A1", "T2", 1)
                        .with_duration(ActivityDuration::fixed(3000))
                        .with_resources("machine", vec!["M1".into()])
                ),
        ];

        let resources = vec![
            Resource::primary("M1").with_efficiency(1.0),
            Resource::primary("M2").with_efficiency(0.9),
        ];

        (tasks, resources)
    }

    #[test]
    fn test_simple_scheduling() {
        let (tasks, resources) = create_test_scenario();
        let scheduler = SimpleScheduler::new();

        let schedule = scheduler.schedule(&tasks, &resources, 0);

        assert_eq!(schedule.assignment_count(), 2);
        assert!(schedule.makespan_ms > 0);
    }

    #[test]
    fn test_priority_ordering() {
        let (tasks, resources) = create_test_scenario();
        let scheduler = SimpleScheduler::new();

        let schedule = scheduler.schedule(&tasks, &resources, 0);

        // T1 (priority 5) should be scheduled before T2 (priority 3)
        let t1_end = schedule.task_completion_time("T1").unwrap();
        let t2_start = schedule.assignment_for_activity("T2-A1").unwrap().start_ms;

        // T1 should start at 0 or T2 should start after T1
        let t1_start = schedule.assignment_for_activity("T1-A1").unwrap().start_ms;
        assert!(t1_start <= t2_start || t1_end <= t2_start);
    }

    #[test]
    fn test_multiple_activities() {
        let task = Task::new("T1")
            .with_activity(
                Activity::new("T1-A1", "T1", 1)
                    .with_duration(ActivityDuration::fixed(3000))
                    .with_resources("machine", vec!["M1".into()])
            )
            .with_activity(
                Activity::new("T1-A2", "T1", 2)
                    .with_duration(ActivityDuration::fixed(2000))
                    .with_resources("machine", vec!["M1".into()])
            );

        let resources = vec![Resource::primary("M1")];
        let scheduler = SimpleScheduler::new();

        let schedule = scheduler.schedule(&[task], &resources, 0);

        assert_eq!(schedule.assignment_count(), 2);

        // A2 should start after A1
        let a1 = schedule.assignment_for_activity("T1-A1").unwrap();
        let a2 = schedule.assignment_for_activity("T1-A2").unwrap();

        assert!(a2.start_ms >= a1.end_ms);
    }

    #[test]
    fn test_empty_input() {
        let scheduler = SimpleScheduler::new();
        let schedule = scheduler.schedule(&[], &[], 0);

        assert_eq!(schedule.assignment_count(), 0);
        assert_eq!(schedule.makespan_ms, 0);
    }
}
