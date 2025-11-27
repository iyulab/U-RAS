//! Dispatching Rules - Domain-agnostic task prioritization framework
//!
//! Provides a flexible rule-based system for determining task execution order.
//! Rules can be combined into multi-layer strategies with tie-breaking.
//!
//! # Supported Rule Categories
//!
//! - **Time-based**: SPT, LPT, LWKR, MWKR
//! - **Due Date**: EDD, MST, CR, S/RO
//! - **Queue/Load**: FIFO, WINQ, LPUL
//!
//! # Example
//!
//! ```rust
//! use u_ras::dispatching::{RuleEngine, SchedulingContext, rules};
//!
//! let engine = RuleEngine::new()
//!     .with_rule(rules::Spt)
//!     .with_tie_breaker(rules::Fifo);
//!
//! let context = SchedulingContext::default();
//! // let sorted = engine.sort(&tasks, &context);
//! ```

mod context;
mod engine;
mod rule;
pub mod rules;

pub use context::*;
pub use engine::*;
pub use rule::*;

#[cfg(test)]
mod tests;
