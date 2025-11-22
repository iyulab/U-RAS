//! KPI - Key Performance Indicators for schedules
//!
//! Metrics for evaluating schedule quality

use crate::models::{Schedule, Task};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Schedule quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleKpi {
    /// Makespan (total completion time)
    pub makespan_ms: i64,
    /// Total tardiness across all tasks
    pub total_tardiness_ms: i64,
    /// Maximum tardiness
    pub max_tardiness_ms: i64,
    /// On-time delivery rate (0.0 to 1.0)
    pub on_time_rate: f64,
    /// Average resource utilization (0.0 to 1.0)
    pub avg_utilization: f64,
    /// Resource utilization by resource
    pub utilization_by_resource: HashMap<String, f64>,
    /// Average flow time (time from release to completion)
    pub avg_flow_time_ms: f64,
}

impl ScheduleKpi {
    /// Calculate KPIs from schedule and tasks
    pub fn calculate(schedule: &Schedule, tasks: &[Task]) -> Self {
        let mut total_tardiness = 0i64;
        let mut max_tardiness = 0i64;
        let mut on_time_count = 0;
        let mut total_flow_time = 0i64;
        let mut task_count = 0;

        for task in tasks {
            if let Some(completion) = schedule.task_completion_time(&task.id) {
                task_count += 1;

                // Calculate tardiness
                if let Some(deadline) = &task.deadline {
                    let deadline_ms = deadline.timestamp_millis();
                    if completion > deadline_ms {
                        let tardiness = completion - deadline_ms;
                        total_tardiness += tardiness;
                        max_tardiness = max_tardiness.max(tardiness);
                    } else {
                        on_time_count += 1;
                    }
                } else {
                    on_time_count += 1; // No deadline = on time
                }

                // Calculate flow time
                let release = task.release_time.map(|t| t.timestamp_millis()).unwrap_or(0);
                total_flow_time += completion - release;
            }
        }

        // Calculate utilization
        let utilization_by_resource = schedule.all_utilizations();
        let avg_utilization = if utilization_by_resource.is_empty() {
            0.0
        } else {
            utilization_by_resource.values().sum::<f64>() / utilization_by_resource.len() as f64
        };

        let on_time_rate = if task_count > 0 {
            on_time_count as f64 / task_count as f64
        } else {
            1.0
        };

        let avg_flow_time = if task_count > 0 {
            total_flow_time as f64 / task_count as f64
        } else {
            0.0
        };

        Self {
            makespan_ms: schedule.makespan_ms,
            total_tardiness_ms: total_tardiness,
            max_tardiness_ms: max_tardiness,
            on_time_rate,
            avg_utilization,
            utilization_by_resource,
            avg_flow_time_ms: avg_flow_time,
        }
    }

    /// Check if schedule meets quality thresholds
    pub fn meets_thresholds(&self, max_tardiness: i64, min_utilization: f64) -> bool {
        self.max_tardiness_ms <= max_tardiness && self.avg_utilization >= min_utilization
    }
}

impl Default for ScheduleKpi {
    fn default() -> Self {
        Self {
            makespan_ms: 0,
            total_tardiness_ms: 0,
            max_tardiness_ms: 0,
            on_time_rate: 1.0,
            avg_utilization: 0.0,
            utilization_by_resource: HashMap::new(),
            avg_flow_time_ms: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Activity, ActivityDuration, Assignment, Resource};
    use chrono::Utc;

    #[test]
    fn test_kpi_calculation() {
        let mut schedule = Schedule::new();
        schedule.add_assignment(Assignment::new("A1", "T1", "R1", 0, 5000));
        schedule.add_assignment(Assignment::new("A2", "T2", "R1", 5000, 8000));

        let tasks = vec![
            Task::new("T1").with_activity(
                Activity::new("A1", "T1", 1).with_duration(ActivityDuration::fixed(5000)),
            ),
            Task::new("T2").with_activity(
                Activity::new("A2", "T2", 1).with_duration(ActivityDuration::fixed(3000)),
            ),
        ];

        let kpi = ScheduleKpi::calculate(&schedule, &tasks);

        assert_eq!(kpi.makespan_ms, 8000);
        assert_eq!(kpi.on_time_rate, 1.0);
    }

    #[test]
    fn test_tardiness_calculation() {
        let mut schedule = Schedule::new();
        schedule.add_assignment(Assignment::new("A1", "T1", "R1", 0, 10000));

        // Deadline at 5000ms, but task completes at 10000ms = 5000ms late
        let deadline = chrono::DateTime::from_timestamp_millis(5000).unwrap();
        let task = Task::new("T1")
            .with_deadline(deadline)
            .with_activity(Activity::new("A1", "T1", 1));

        let kpi = ScheduleKpi::calculate(&schedule, &[task]);

        assert_eq!(kpi.total_tardiness_ms, 5000);
        assert_eq!(kpi.on_time_rate, 0.0);
    }
}
