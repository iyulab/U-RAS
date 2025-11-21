//! GA - Genetic Algorithm Module
//!
//! Evolutionary optimization algorithms for scheduling

mod chromosome;
mod operators;

pub use chromosome::*;
pub use operators::*;

/// GA parameters
#[derive(Debug, Clone)]
pub struct GaParams {
    pub population_size: usize,
    pub max_generations: usize,
    pub elite_ratio: f64,
    pub crossover_rate: f64,
    pub mutation_rate: f64,
}

impl Default for GaParams {
    fn default() -> Self {
        Self {
            population_size: 100,
            max_generations: 200,
            elite_ratio: 0.1,
            crossover_rate: 0.8,
            mutation_rate: 0.1,
        }
    }
}
