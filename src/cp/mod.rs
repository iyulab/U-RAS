//! CP - Constraint Programming Module
//!
//! Constraint satisfaction and optimization
//!
//! This module will contain:
//! - CP model definition
//! - Interval variables
//! - CP solver
//! - CP-SAT scheduler

// TODO: Move from engine/src/cp/
// - model.rs
// - solver.rs
// - variables.rs
// - cpsat.rs

/// Placeholder for CP objective
#[derive(Debug, Clone)]
pub enum Objective {
    Minimize(String),
    Maximize(String),
}
