//! Queue and Load-based dispatching rules
//!
//! Rules that consider system state (queues, utilization) for prioritization.

use crate::models::Task;
use crate::dispatching::{DispatchingRule, SchedulingContext, RuleScore};

/// FIFO - First In First Out
///
/// Prioritizes tasks by arrival order.
/// Simple and fair, ensures no starvation.
///
/// Score = arrival timestamp (earlier = lower = higher priority)
#[derive(Debug, Clone, Copy, Default)]
pub struct Fifo;

impl DispatchingRule for Fifo {
    fn name(&self) -> &'static str {
        "FIFO"
    }

    fn description(&self) -> &'static str {
        "First In First Out - process in arrival order"
    }

    fn evaluate(&self, task: &Task, context: &SchedulingContext) -> RuleScore {
        context.arrival_times
            .get(&task.id)
            .map(|t| t.timestamp_millis() as f64)
            .unwrap_or_else(|| {
                // If no arrival time, use release_time or 0
                task.release_time
                    .map(|t| t.timestamp_millis() as f64)
                    .unwrap_or(0.0)
            })
    }
}

/// WINQ - Work In Next Queue
///
/// Prioritizes tasks whose next operation has the shortest queue.
/// Helps balance workload across resources.
///
/// Score = queue length at next operation's resource
/// Shorter queue = lower score = higher priority
#[derive(Debug, Clone, Copy, Default)]
pub struct Winq;

impl DispatchingRule for Winq {
    fn name(&self) -> &'static str {
        "WINQ"
    }

    fn description(&self) -> &'static str {
        "Work In Next Queue - prefer shorter downstream queues"
    }

    fn evaluate(&self, task: &Task, context: &SchedulingContext) -> RuleScore {
        context.next_queue_length
            .get(&task.id)
            .copied()
            .unwrap_or(0) as f64
    }
}

/// LPUL - Least Planned Utilization Level
///
/// Prioritizes tasks that will use least-utilized resources.
/// Helps balance resource utilization across the system.
///
/// Score = utilization of the task's preferred resource
/// Lower utilization = lower score = higher priority
#[derive(Debug, Clone, Copy, Default)]
pub struct Lpul;

impl DispatchingRule for Lpul {
    fn name(&self) -> &'static str {
        "LPUL"
    }

    fn description(&self) -> &'static str {
        "Least Planned Utilization Level - use underutilized resources"
    }

    fn evaluate(&self, task: &Task, context: &SchedulingContext) -> RuleScore {
        // Find the minimum utilization among resources required by this task's first activity
        if let Some(activity) = task.activities.first() {
            let min_util = activity.resource_requirements
                .iter()
                .flat_map(|req| req.candidates.iter())
                .filter_map(|res_id| context.resource_utilization.get(res_id))
                .copied()
                .min_by(|a, b| a.partial_cmp(b).unwrap());

            min_util.unwrap_or(0.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Activity, ActivityDuration};
    use chrono::{TimeZone, Utc};

    fn make_simple_task(id: &str) -> Task {
        Task {
            id: id.to_string(),
            name: id.to_string(),
            category: String::new(),
            priority: 0,
            deadline: None,
            release_time: None,
            activities: vec![
                Activity::new(&format!("{}-A1", id), id, 1)
                    .with_duration(ActivityDuration::fixed(1000))
            ],
            attributes: Default::default(),
        }
    }

    fn make_task_with_resources(id: &str, resource_ids: Vec<&str>) -> Task {
        Task {
            id: id.to_string(),
            name: id.to_string(),
            category: String::new(),
            priority: 0,
            deadline: None,
            release_time: None,
            activities: vec![
                Activity::new(&format!("{}-A1", id), id, 1)
                    .with_duration(ActivityDuration::fixed(1000))
                    .with_resources("machine", resource_ids.into_iter().map(String::from).collect())
            ],
            attributes: Default::default(),
        }
    }

    #[test]
    fn test_fifo_prioritizes_earlier_arrival() {
        let first = make_simple_task("first");
        let second = make_simple_task("second");

        let ctx = SchedulingContext::at_epoch()
            .with_arrival_time("first", Utc.timestamp_millis_opt(1000).unwrap())
            .with_arrival_time("second", Utc.timestamp_millis_opt(2000).unwrap());

        let fifo = Fifo;

        assert!(fifo.evaluate(&first, &ctx) < fifo.evaluate(&second, &ctx));
    }

    #[test]
    fn test_fifo_uses_release_time_as_fallback() {
        let task = Task {
            id: "task".to_string(),
            name: "task".to_string(),
            category: String::new(),
            priority: 0,
            deadline: None,
            release_time: Some(Utc.timestamp_millis_opt(5000).unwrap()),
            activities: vec![],
            attributes: Default::default(),
        };

        let ctx = SchedulingContext::at_epoch(); // No arrival time set
        let fifo = Fifo;

        assert_eq!(fifo.evaluate(&task, &ctx), 5000.0);
    }

    #[test]
    fn test_winq_prioritizes_shorter_queues() {
        let short_queue = make_simple_task("short");
        let long_queue = make_simple_task("long");

        let ctx = SchedulingContext::at_epoch()
            .with_next_queue("short", 2)
            .with_next_queue("long", 10);

        let winq = Winq;

        assert!(winq.evaluate(&short_queue, &ctx) < winq.evaluate(&long_queue, &ctx));
    }

    #[test]
    fn test_lpul_prioritizes_underutilized_resources() {
        let uses_idle = make_task_with_resources("uses_idle", vec!["R1"]);
        let uses_busy = make_task_with_resources("uses_busy", vec!["R2"]);

        let ctx = SchedulingContext::at_epoch()
            .with_utilization("R1", 0.2)  // 20% utilized
            .with_utilization("R2", 0.9); // 90% utilized

        let lpul = Lpul;

        // Task using less utilized resource should have lower score
        assert!(lpul.evaluate(&uses_idle, &ctx) < lpul.evaluate(&uses_busy, &ctx));
    }

    #[test]
    fn test_lpul_picks_least_utilized_candidate() {
        // Task can use either R1 or R2
        let task = make_task_with_resources("task", vec!["R1", "R2"]);

        let ctx = SchedulingContext::at_epoch()
            .with_utilization("R1", 0.8)
            .with_utilization("R2", 0.3);

        let lpul = Lpul;

        // Should return the minimum utilization (0.3 from R2)
        assert_eq!(lpul.evaluate(&task, &ctx), 0.3);
    }
}
