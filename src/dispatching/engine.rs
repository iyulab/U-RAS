//! RuleEngine - Multi-layer dispatching with tie-breaking

use crate::models::Task;
use super::{BoxedRule, DispatchingRule, SchedulingContext, RuleScore};

/// How to evaluate multiple rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EvaluationMode {
    /// Apply rules sequentially; use next rule only on ties
    #[default]
    Sequential,
    /// Compute weighted sum of all rule scores
    Weighted,
}

/// How to break ties when scores are equal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TieBreaker {
    /// Use the next rule in the chain
    #[default]
    NextRule,
    /// Random selection among tied tasks
    Random,
    /// Deterministic by task ID (for reproducibility)
    ById,
}

/// A rule with its weight (for weighted mode)
#[derive(Debug)]
pub struct WeightedRule {
    pub rule: BoxedRule,
    pub weight: f64,
}

impl WeightedRule {
    pub fn new(rule: BoxedRule, weight: f64) -> Self {
        Self { rule, weight }
    }
}

/// Engine for evaluating and sorting tasks by dispatching rules
#[derive(Debug, Default)]
pub struct RuleEngine {
    /// Ordered list of rules (primary → tie-breakers)
    rules: Vec<WeightedRule>,
    /// Evaluation strategy
    mode: EvaluationMode,
    /// Final tie-breaking strategy
    tie_breaker: TieBreaker,
    /// Tolerance for considering scores equal (for tie detection)
    epsilon: f64,
}

impl RuleEngine {
    /// Create a new empty rule engine
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            mode: EvaluationMode::Sequential,
            tie_breaker: TieBreaker::NextRule,
            epsilon: 1e-9,
        }
    }

    /// Add a rule with default weight 1.0
    pub fn with_rule<R: DispatchingRule + 'static>(mut self, rule: R) -> Self {
        self.rules.push(WeightedRule::new(Box::new(rule), 1.0));
        self
    }

    /// Add a rule with specific weight
    pub fn with_weighted_rule<R: DispatchingRule + 'static>(mut self, rule: R, weight: f64) -> Self {
        self.rules.push(WeightedRule::new(Box::new(rule), weight));
        self
    }

    /// Add a tie-breaker rule (weight 0, used only when primary rules tie)
    pub fn with_tie_breaker<R: DispatchingRule + 'static>(self, rule: R) -> Self {
        self.with_weighted_rule(rule, 0.0)
    }

    /// Set evaluation mode
    pub fn with_mode(mut self, mode: EvaluationMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set final tie-breaking strategy
    pub fn with_final_tie_breaker(mut self, tie_breaker: TieBreaker) -> Self {
        self.tie_breaker = tie_breaker;
        self
    }

    /// Check if engine has any rules
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Get the number of rules
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Evaluate a single task and return raw scores (without weights)
    fn evaluate_raw(&self, task: &Task, context: &SchedulingContext) -> Vec<RuleScore> {
        self.rules
            .iter()
            .map(|wr| wr.rule.evaluate(task, context))
            .collect()
    }

    /// Evaluate a single task and return weighted scores
    pub fn evaluate(&self, task: &Task, context: &SchedulingContext) -> Vec<RuleScore> {
        self.rules
            .iter()
            .map(|wr| wr.rule.evaluate(task, context) * wr.weight)
            .collect()
    }

    /// Sort tasks by priority (lowest score first)
    ///
    /// Returns a new vector with tasks sorted by their dispatching priority.
    pub fn sort<'a>(&self, tasks: &[&'a Task], context: &SchedulingContext) -> Vec<&'a Task> {
        if tasks.is_empty() || self.rules.is_empty() {
            return tasks.to_vec();
        }

        let mut scored: Vec<_> = tasks
            .iter()
            .map(|&task| {
                // Sequential mode uses raw scores; Weighted mode uses weighted scores
                let scores = match self.mode {
                    EvaluationMode::Sequential => self.evaluate_raw(task, context),
                    EvaluationMode::Weighted => self.evaluate(task, context),
                };
                (task, scores)
            })
            .collect();

        match self.mode {
            EvaluationMode::Sequential => {
                scored.sort_by(|(task_a, scores_a), (task_b, scores_b)| {
                    // Compare rule by rule until we find a difference
                    for (score_a, score_b) in scores_a.iter().zip(scores_b.iter()) {
                        if (score_a - score_b).abs() > self.epsilon {
                            return score_a.partial_cmp(score_b).unwrap();
                        }
                    }
                    // All rules tied, use final tie-breaker
                    match self.tie_breaker {
                        TieBreaker::NextRule => std::cmp::Ordering::Equal,
                        TieBreaker::Random => {
                            // For determinism in tests, we use a pseudo-random based on IDs
                            let hash_a = task_a.id.as_bytes().iter().map(|&b| b as usize).sum::<usize>();
                            let hash_b = task_b.id.as_bytes().iter().map(|&b| b as usize).sum::<usize>();
                            hash_a.cmp(&hash_b)
                        }
                        TieBreaker::ById => task_a.id.cmp(&task_b.id),
                    }
                });
            }
            EvaluationMode::Weighted => {
                scored.sort_by(|(_, scores_a), (_, scores_b)| {
                    let sum_a: f64 = scores_a.iter().sum();
                    let sum_b: f64 = scores_b.iter().sum();
                    sum_a.partial_cmp(&sum_b).unwrap()
                });
            }
        }

        scored.into_iter().map(|(task, _)| task).collect()
    }

    /// Select the highest priority task (first after sorting)
    pub fn select_best<'a>(&self, tasks: &[&'a Task], context: &SchedulingContext) -> Option<&'a Task> {
        self.sort(tasks, context).into_iter().next()
    }
}

#[cfg(test)]
mod engine_tests {
    use super::*;
    use crate::dispatching::rules::Spt;

    #[test]
    fn test_empty_engine() {
        let engine = RuleEngine::new();
        assert!(engine.is_empty());
        assert_eq!(engine.rule_count(), 0);
    }

    #[test]
    fn test_add_rules() {
        let engine = RuleEngine::new()
            .with_rule(Spt)
            .with_rule(Spt);

        assert!(!engine.is_empty());
        assert_eq!(engine.rule_count(), 2);
    }
}
