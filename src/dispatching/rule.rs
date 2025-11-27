//! DispatchingRule trait - Core abstraction for task prioritization

use crate::models::Task;
use super::SchedulingContext;
use std::fmt::Debug;

/// Score returned by a dispatching rule
/// Lower scores have higher priority (will be scheduled first)
pub type RuleScore = f64;

/// A dispatching rule that evaluates task priority
///
/// Rules return a score where lower values indicate higher priority.
/// This follows the convention of most dispatching rules (e.g., SPT = shortest first).
pub trait DispatchingRule: Send + Sync + Debug {
    /// Unique identifier for this rule
    fn name(&self) -> &'static str;

    /// Evaluate the priority score for a task
    ///
    /// # Arguments
    /// * `task` - The task to evaluate
    /// * `context` - Current scheduling context (time, resource state, etc.)
    ///
    /// # Returns
    /// A score where lower values = higher priority
    fn evaluate(&self, task: &Task, context: &SchedulingContext) -> RuleScore;

    /// Human-readable description of the rule
    fn description(&self) -> &'static str {
        self.name()
    }
}

/// Boxed rule for dynamic dispatch
pub type BoxedRule = Box<dyn DispatchingRule>;

/// Implement DispatchingRule for BoxedRule to allow seamless usage
impl DispatchingRule for BoxedRule {
    fn name(&self) -> &'static str {
        (**self).name()
    }

    fn evaluate(&self, task: &Task, context: &SchedulingContext) -> RuleScore {
        (**self).evaluate(task, context)
    }

    fn description(&self) -> &'static str {
        (**self).description()
    }
}

/// Convert any DispatchingRule into a boxed version
pub fn boxed<R: DispatchingRule + 'static>(rule: R) -> BoxedRule {
    Box::new(rule)
}
