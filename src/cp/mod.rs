//! CP - Constraint Programming Module
//!
//! Domain-agnostic constraint programming infrastructure

mod model;
mod solver;
mod variables;

pub use model::*;
pub use solver::*;
pub use variables::*;
