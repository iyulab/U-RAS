//! Due date-based dispatching rules
//!
//! Rules that prioritize tasks based on deadline urgency.

use crate::models::Task;
use crate::dispatching::{DispatchingRule, SchedulingContext, RuleScore};

/// EDD - Earliest Due Date
///
/// Prioritizes tasks with the earliest deadline.
/// Minimizes maximum lateness.
///
/// Score = deadline timestamp (earlier = lower = higher priority)
/// Tasks without deadlines get maximum score (lowest priority)
#[derive(Debug, Clone, Copy, Default)]
pub struct Edd;

impl DispatchingRule for Edd {
    fn name(&self) -> &'static str {
        "EDD"
    }

    fn description(&self) -> &'static str {
        "Earliest Due Date - prioritize nearest deadlines"
    }

    fn evaluate(&self, task: &Task, _context: &SchedulingContext) -> RuleScore {
        task.deadline
            .map(|d| d.timestamp_millis() as f64)
            .unwrap_or(f64::MAX)
    }
}

/// MST - Minimum Slack Time
///
/// Prioritizes tasks with the least slack (time until deadline - remaining work).
/// Helps prevent deadline misses.
///
/// Score = slack_time = (deadline - current_time) - remaining_work
/// Lower slack = lower score = higher priority
#[derive(Debug, Clone, Copy, Default)]
pub struct Mst;

impl DispatchingRule for Mst {
    fn name(&self) -> &'static str {
        "MST"
    }

    fn description(&self) -> &'static str {
        "Minimum Slack Time - prioritize tasks with least buffer"
    }

    fn evaluate(&self, task: &Task, context: &SchedulingContext) -> RuleScore {
        let deadline = match task.deadline {
            Some(d) => d.timestamp_millis(),
            None => return f64::MAX, // No deadline = lowest priority
        };

        let current = context.current_time.timestamp_millis();
        let time_until_deadline = deadline - current;

        // Get remaining work
        let remaining_work = context.remaining_work
            .get(&task.id)
            .copied()
            .unwrap_or_else(|| {
                task.activities.iter().map(|a| a.duration.total_ms()).sum()
            });

        // Slack = time available - work needed
        (time_until_deadline - remaining_work) as f64
    }
}

/// CR - Critical Ratio
///
/// Ratio of time remaining until deadline to work remaining.
/// CR < 1 means the task will be late at current pace.
///
/// Score = (deadline - now) / remaining_work
/// Lower ratio = more critical = lower score = higher priority
#[derive(Debug, Clone, Copy, Default)]
pub struct Cr;

impl DispatchingRule for Cr {
    fn name(&self) -> &'static str {
        "CR"
    }

    fn description(&self) -> &'static str {
        "Critical Ratio - prioritize tasks falling behind schedule"
    }

    fn evaluate(&self, task: &Task, context: &SchedulingContext) -> RuleScore {
        let deadline = match task.deadline {
            Some(d) => d.timestamp_millis(),
            None => return f64::MAX,
        };

        let current = context.current_time.timestamp_millis();
        let time_until_deadline = (deadline - current) as f64;

        let remaining_work = context.remaining_work
            .get(&task.id)
            .copied()
            .unwrap_or_else(|| {
                task.activities.iter().map(|a| a.duration.total_ms()).sum()
            }) as f64;

        if remaining_work <= 0.0 {
            return f64::MAX; // Already complete
        }

        time_until_deadline / remaining_work
    }
}

/// S/RO - Slack per Remaining Operations
///
/// Distributes slack evenly across remaining operations.
/// Useful when operations have dependencies.
///
/// Score = slack / remaining_operation_count
/// Lower = more urgent = higher priority
#[derive(Debug, Clone, Copy, Default)]
pub struct Sro;

impl DispatchingRule for Sro {
    fn name(&self) -> &'static str {
        "S/RO"
    }

    fn description(&self) -> &'static str {
        "Slack per Remaining Operations - slack distributed across operations"
    }

    fn evaluate(&self, task: &Task, context: &SchedulingContext) -> RuleScore {
        let deadline = match task.deadline {
            Some(d) => d.timestamp_millis(),
            None => return f64::MAX,
        };

        let current = context.current_time.timestamp_millis();
        let time_until_deadline = deadline - current;

        let remaining_work = context.remaining_work
            .get(&task.id)
            .copied()
            .unwrap_or_else(|| {
                task.activities.iter().map(|a| a.duration.total_ms()).sum()
            });

        let slack = time_until_deadline - remaining_work;

        // Count remaining operations (simplified: use activity count)
        // In practice, this would track completed vs remaining activities
        let remaining_ops = task.activities.len().max(1) as f64;

        slack as f64 / remaining_ops
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Activity, ActivityDuration};
    use chrono::{TimeZone, Utc};

    fn make_task_with_deadline(id: &str, duration_ms: i64, deadline_ms: i64) -> Task {
        Task {
            id: id.to_string(),
            name: id.to_string(),
            category: String::new(),
            priority: 0,
            deadline: Some(Utc.timestamp_millis_opt(deadline_ms).unwrap()),
            release_time: None,
            activities: vec![
                Activity::new(&format!("{}-A1", id), id, 1)
                    .with_duration(ActivityDuration::fixed(duration_ms))
            ],
            attributes: Default::default(),
        }
    }

    #[test]
    fn test_edd_prioritizes_earlier_deadline() {
        let urgent = make_task_with_deadline("urgent", 1000, 5000);
        let relaxed = make_task_with_deadline("relaxed", 1000, 10000);

        let ctx = SchedulingContext::at_epoch();
        let edd = Edd;

        assert!(edd.evaluate(&urgent, &ctx) < edd.evaluate(&relaxed, &ctx));
    }

    #[test]
    fn test_edd_no_deadline_lowest_priority() {
        let with_deadline = make_task_with_deadline("with", 1000, 10000);
        let without = Task {
            id: "without".to_string(),
            name: "without".to_string(),
            category: String::new(),
            priority: 0,
            deadline: None,
            release_time: None,
            activities: vec![],
            attributes: Default::default(),
        };

        let ctx = SchedulingContext::at_epoch();
        let edd = Edd;

        assert!(edd.evaluate(&with_deadline, &ctx) < edd.evaluate(&without, &ctx));
    }

    #[test]
    fn test_mst_prioritizes_least_slack() {
        // Both due at 10000ms from epoch
        let tight = make_task_with_deadline("tight", 9000, 10000);  // Slack = 1000
        let loose = make_task_with_deadline("loose", 5000, 10000);  // Slack = 5000

        let ctx = SchedulingContext::at_epoch();
        let mst = Mst;

        // Tight has less slack, should have lower score
        assert!(mst.evaluate(&tight, &ctx) < mst.evaluate(&loose, &ctx));
    }

    #[test]
    fn test_cr_prioritizes_behind_schedule() {
        // Both need 5000ms work
        // Behind: deadline in 4000ms (CR = 0.8 < 1, will be late)
        // Ahead: deadline in 10000ms (CR = 2.0 > 1, on track)
        let behind = make_task_with_deadline("behind", 5000, 4000);
        let ahead = make_task_with_deadline("ahead", 5000, 10000);

        let ctx = SchedulingContext::at_epoch();
        let cr = Cr;

        // Behind schedule should have lower CR = lower score = higher priority
        assert!(cr.evaluate(&behind, &ctx) < cr.evaluate(&ahead, &ctx));
    }

    #[test]
    fn test_sro_accounts_for_operation_count() {
        // Same slack but different number of operations
        let single_op = make_task_with_deadline("single", 5000, 10000); // 1 op, slack=5000

        // Multi-op task with same total work and deadline
        let multi_op = Task {
            id: "multi".to_string(),
            name: "multi".to_string(),
            category: String::new(),
            priority: 0,
            deadline: Some(Utc.timestamp_millis_opt(10000).unwrap()),
            release_time: None,
            activities: vec![
                Activity::new("multi-A1", "multi", 1)
                    .with_duration(ActivityDuration::fixed(2500)),
                Activity::new("multi-A2", "multi", 2)
                    .with_duration(ActivityDuration::fixed(2500)),
            ],
            attributes: Default::default(),
        };

        let ctx = SchedulingContext::at_epoch();
        let sro = Sro;

        // single_op: slack/1 = 5000
        // multi_op: slack/2 = 2500
        // multi_op has less slack per op, should be more urgent
        assert!(sro.evaluate(&multi_op, &ctx) < sro.evaluate(&single_op, &ctx));
    }
}
