//! Time-based dispatching rules
//!
//! Rules that prioritize tasks based on processing time characteristics.

use crate::models::Task;
use crate::dispatching::{DispatchingRule, SchedulingContext, RuleScore};

/// SPT - Shortest Processing Time
///
/// Prioritizes tasks with the shortest total processing time.
/// Minimizes average flow time and work-in-progress.
///
/// Score = sum of all activity durations
#[derive(Debug, Clone, Copy, Default)]
pub struct Spt;

impl DispatchingRule for Spt {
    fn name(&self) -> &'static str {
        "SPT"
    }

    fn description(&self) -> &'static str {
        "Shortest Processing Time - prioritize shorter tasks"
    }

    fn evaluate(&self, task: &Task, _context: &SchedulingContext) -> RuleScore {
        task.activities
            .iter()
            .map(|a| a.duration.total_ms() as f64)
            .sum()
    }
}

/// LPT - Longest Processing Time
///
/// Prioritizes tasks with the longest total processing time.
/// Useful for load balancing in parallel machine environments.
///
/// Score = negative sum of durations (so longer = lower score = higher priority)
#[derive(Debug, Clone, Copy, Default)]
pub struct Lpt;

impl DispatchingRule for Lpt {
    fn name(&self) -> &'static str {
        "LPT"
    }

    fn description(&self) -> &'static str {
        "Longest Processing Time - prioritize longer tasks"
    }

    fn evaluate(&self, task: &Task, _context: &SchedulingContext) -> RuleScore {
        let total: f64 = task.activities
            .iter()
            .map(|a| a.duration.total_ms() as f64)
            .sum();
        -total // Negate so longer tasks get lower (better) scores
    }
}

/// LWKR - Least Work Remaining
///
/// Prioritizes tasks with the least remaining work.
/// Requires context to know which activities are already completed.
///
/// Score = remaining work from context (or total if not set)
#[derive(Debug, Clone, Copy, Default)]
pub struct Lwkr;

impl DispatchingRule for Lwkr {
    fn name(&self) -> &'static str {
        "LWKR"
    }

    fn description(&self) -> &'static str {
        "Least Work Remaining - prioritize tasks near completion"
    }

    fn evaluate(&self, task: &Task, context: &SchedulingContext) -> RuleScore {
        // Use context if available, otherwise calculate total
        if let Some(&remaining) = context.remaining_work.get(&task.id) {
            remaining as f64
        } else {
            // Default: assume all work remaining
            task.activities
                .iter()
                .map(|a| a.duration.total_ms() as f64)
                .sum()
        }
    }
}

/// MWKR - Most Work Remaining
///
/// Prioritizes tasks with the most remaining work.
/// Can help prevent starvation of long tasks.
///
/// Score = negative remaining work
#[derive(Debug, Clone, Copy, Default)]
pub struct Mwkr;

impl DispatchingRule for Mwkr {
    fn name(&self) -> &'static str {
        "MWKR"
    }

    fn description(&self) -> &'static str {
        "Most Work Remaining - prioritize tasks with most work left"
    }

    fn evaluate(&self, task: &Task, context: &SchedulingContext) -> RuleScore {
        let remaining = if let Some(&r) = context.remaining_work.get(&task.id) {
            r as f64
        } else {
            task.activities
                .iter()
                .map(|a| a.duration.total_ms() as f64)
                .sum()
        };
        -remaining // Negate so more work = lower score = higher priority
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Activity, ActivityDuration};

    fn make_task(id: &str, durations: &[i64]) -> Task {
        let activities = durations
            .iter()
            .enumerate()
            .map(|(i, &d)| {
                Activity::new(&format!("{}-A{}", id, i), id, i as i32 + 1)
                    .with_duration(ActivityDuration::fixed(d))
            })
            .collect();

        Task {
            id: id.to_string(),
            name: id.to_string(),
            category: String::new(),
            priority: 0,
            deadline: None,
            release_time: None,
            activities,
            attributes: Default::default(),
        }
    }

    #[test]
    fn test_spt_prioritizes_shorter_tasks() {
        let short = make_task("short", &[1000, 2000]); // 3000ms total
        let long = make_task("long", &[5000, 5000]);   // 10000ms total

        let ctx = SchedulingContext::default();
        let spt = Spt;

        assert!(spt.evaluate(&short, &ctx) < spt.evaluate(&long, &ctx));
    }

    #[test]
    fn test_lpt_prioritizes_longer_tasks() {
        let short = make_task("short", &[1000, 2000]);
        let long = make_task("long", &[5000, 5000]);

        let ctx = SchedulingContext::default();
        let lpt = Lpt;

        // LPT: longer task should have LOWER score (higher priority)
        assert!(lpt.evaluate(&long, &ctx) < lpt.evaluate(&short, &ctx));
    }

    #[test]
    fn test_lwkr_uses_context_remaining_work() {
        let task1 = make_task("T1", &[10000]);
        let task2 = make_task("T2", &[10000]);

        let ctx = SchedulingContext::default()
            .with_remaining_work("T1", 2000)  // Almost done
            .with_remaining_work("T2", 8000); // Lots remaining

        let lwkr = Lwkr;

        // T1 has less remaining work, should have lower score
        assert!(lwkr.evaluate(&task1, &ctx) < lwkr.evaluate(&task2, &ctx));
    }

    #[test]
    fn test_mwkr_prioritizes_most_remaining() {
        let task1 = make_task("T1", &[10000]);
        let task2 = make_task("T2", &[10000]);

        let ctx = SchedulingContext::default()
            .with_remaining_work("T1", 2000)
            .with_remaining_work("T2", 8000);

        let mwkr = Mwkr;

        // T2 has more remaining work, should have lower (better) score in MWKR
        assert!(mwkr.evaluate(&task2, &ctx) < mwkr.evaluate(&task1, &ctx));
    }
}
