//! Integration tests for dispatching rules

use super::*;
use crate::models::{Activity, ActivityDuration, Task};
use chrono::{TimeZone, Utc};

fn make_task(id: &str, duration_ms: i64, deadline_ms: Option<i64>) -> Task {
    Task {
        id: id.to_string(),
        name: id.to_string(),
        category: String::new(),
        priority: 0,
        deadline: deadline_ms.map(|d| Utc.timestamp_millis_opt(d).unwrap()),
        release_time: None,
        activities: vec![
            Activity::new(&format!("{}-A1", id), id, 1)
                .with_duration(ActivityDuration::fixed(duration_ms))
        ],
        attributes: Default::default(),
    }
}

#[test]
fn test_rule_engine_sort_by_spt() {
    let t1 = make_task("T1", 5000, None);  // 5s
    let t2 = make_task("T2", 2000, None);  // 2s (shortest)
    let t3 = make_task("T3", 8000, None);  // 8s

    let engine = RuleEngine::new().with_rule(rules::Spt);
    let ctx = SchedulingContext::default();

    let tasks: Vec<&Task> = vec![&t1, &t2, &t3];
    let sorted = engine.sort(&tasks, &ctx);

    // Sorted by processing time: T2 < T1 < T3
    assert_eq!(sorted[0].id, "T2");
    assert_eq!(sorted[1].id, "T1");
    assert_eq!(sorted[2].id, "T3");
}

#[test]
fn test_rule_engine_sort_by_edd() {
    let t1 = make_task("T1", 1000, Some(10000)); // Due at 10s
    let t2 = make_task("T2", 1000, Some(5000));  // Due at 5s (earliest)
    let t3 = make_task("T3", 1000, None);        // No deadline (last)

    let engine = RuleEngine::new().with_rule(rules::Edd);
    let ctx = SchedulingContext::default();

    let tasks: Vec<&Task> = vec![&t1, &t2, &t3];
    let sorted = engine.sort(&tasks, &ctx);

    assert_eq!(sorted[0].id, "T2"); // Earliest deadline
    assert_eq!(sorted[1].id, "T1");
    assert_eq!(sorted[2].id, "T3"); // No deadline = last
}

#[test]
fn test_rule_engine_multi_layer_tie_breaking() {
    // All tasks have same deadline (EDD will tie)
    let t1 = make_task("T1", 5000, Some(10000));
    let t2 = make_task("T2", 2000, Some(10000)); // Shorter (wins SPT)
    let t3 = make_task("T3", 5000, Some(10000));

    // EDD first, then SPT as tie-breaker
    let engine = RuleEngine::new()
        .with_rule(rules::Edd)
        .with_tie_breaker(rules::Spt);

    let ctx = SchedulingContext::default();

    let tasks: Vec<&Task> = vec![&t1, &t2, &t3];
    let sorted = engine.sort(&tasks, &ctx);

    // All have same deadline, so SPT breaks tie: T2 is shortest
    assert_eq!(sorted[0].id, "T2");
}

#[test]
fn test_rule_engine_weighted_mode() {
    let t1 = make_task("T1", 5000, Some(20000)); // duration=5000, deadline=20000
    let t2 = make_task("T2", 2000, Some(5000));  // duration=2000, deadline=5000

    // Weighted combination: 0.5*EDD + 0.5*SPT
    let engine = RuleEngine::new()
        .with_mode(EvaluationMode::Weighted)
        .with_weighted_rule(rules::Edd, 0.5)
        .with_weighted_rule(rules::Spt, 0.5);

    let ctx = SchedulingContext::default();

    // T1: 0.5*20000 + 0.5*5000 = 10000 + 2500 = 12500
    // T2: 0.5*5000 + 0.5*2000 = 2500 + 1000 = 3500
    // T2 should win (lower weighted score)

    let tasks: Vec<&Task> = vec![&t1, &t2];
    let sorted = engine.sort(&tasks, &ctx);

    assert_eq!(sorted[0].id, "T2");
}

#[test]
fn test_rule_engine_select_best() {
    let t1 = make_task("T1", 5000, None);
    let t2 = make_task("T2", 1000, None); // Shortest

    let engine = RuleEngine::new().with_rule(rules::Spt);
    let ctx = SchedulingContext::default();

    let tasks: Vec<&Task> = vec![&t1, &t2];
    let best = engine.select_best(&tasks, &ctx);

    assert!(best.is_some());
    assert_eq!(best.unwrap().id, "T2");
}

#[test]
fn test_rule_engine_empty_tasks() {
    let engine = RuleEngine::new().with_rule(rules::Spt);
    let ctx = SchedulingContext::default();

    let tasks: Vec<&Task> = vec![];
    let sorted = engine.sort(&tasks, &ctx);

    assert!(sorted.is_empty());
}

#[test]
fn test_rule_engine_deterministic_tie_breaker() {
    // Identical tasks should be sorted deterministically by ID
    let t1 = make_task("B", 1000, None);
    let t2 = make_task("A", 1000, None);
    let t3 = make_task("C", 1000, None);

    let engine = RuleEngine::new()
        .with_rule(rules::Spt)
        .with_final_tie_breaker(TieBreaker::ById);

    let ctx = SchedulingContext::default();

    let tasks: Vec<&Task> = vec![&t1, &t2, &t3];
    let sorted = engine.sort(&tasks, &ctx);

    // All have same duration, sorted by ID
    assert_eq!(sorted[0].id, "A");
    assert_eq!(sorted[1].id, "B");
    assert_eq!(sorted[2].id, "C");
}

#[test]
fn test_complex_scenario_with_context() {
    // Simulate a real scheduling scenario
    let t1 = make_task("urgent", 3000, Some(5000));   // Urgent deadline
    let t2 = make_task("short", 1000, Some(10000));   // Short task
    let t3 = make_task("critical", 2000, Some(3000)); // Very tight deadline

    // Context: t1 is almost done, t3 has most work remaining
    let ctx = SchedulingContext::new(Utc.timestamp_millis_opt(1000).unwrap())
        .with_remaining_work("urgent", 500)    // Almost done
        .with_remaining_work("short", 1000)    // Full work
        .with_remaining_work("critical", 2000) // Full work
        .with_arrival_time("urgent", Utc.timestamp_millis_opt(0).unwrap())
        .with_arrival_time("short", Utc.timestamp_millis_opt(100).unwrap())
        .with_arrival_time("critical", Utc.timestamp_millis_opt(200).unwrap());

    // Use MST (Minimum Slack Time) with FIFO tie-breaker
    let engine = RuleEngine::new()
        .with_rule(rules::Mst)
        .with_tie_breaker(rules::Fifo);

    let tasks: Vec<&Task> = vec![&t1, &t2, &t3];
    let sorted = engine.sort(&tasks, &ctx);

    // MST calculation at t=1000:
    // urgent: deadline=5000, remaining=500, slack = (5000-1000) - 500 = 3500
    // short: deadline=10000, remaining=1000, slack = (10000-1000) - 1000 = 8000
    // critical: deadline=3000, remaining=2000, slack = (3000-1000) - 2000 = 0

    // Critical has least slack, should be first
    assert_eq!(sorted[0].id, "critical");
    assert_eq!(sorted[1].id, "urgent");
    assert_eq!(sorted[2].id, "short");
}
