//! Population Management for Genetic Algorithm
//!
//! Population creation, selection, and generation management

use crate::ga::chromosome::{ActivityInfo, Chromosome};
use crate::ga::operators::{tournament_selection, GeneticOperators};
use crate::models::Resource;
use rand::prelude::*;

/// Population manager
#[derive(Debug)]
pub struct Population {
    /// Current population
    pub individuals: Vec<Chromosome>,
    /// GA parameters
    pub params: GaParams,
    /// Genetic operators
    pub operators: GeneticOperators,
    /// Current generation
    pub generation: usize,
    /// Best individual
    pub best: Option<Chromosome>,
    /// Fitness history per generation
    pub fitness_history: Vec<f64>,
}

/// GA parameters
#[derive(Debug, Clone)]
pub struct GaParams {
    /// Population size
    pub population_size: usize,
    /// Maximum generations
    pub max_generations: usize,
    /// Elite ratio (0.0 ~ 1.0)
    pub elite_ratio: f64,
    /// Tournament size
    pub tournament_size: usize,
    /// Convergence check generations
    pub convergence_generations: usize,
    /// Convergence threshold
    pub convergence_threshold: f64,
}

impl Default for GaParams {
    fn default() -> Self {
        Self {
            population_size: 100,
            max_generations: 500,
            elite_ratio: 0.1,
            tournament_size: 3,
            convergence_generations: 50,
            convergence_threshold: 0.001,
        }
    }
}

impl Population {
    /// Create initial population
    pub fn new(
        activities: &[ActivityInfo],
        resources: &[Resource],
        params: GaParams,
        operators: GeneticOperators,
        rng: &mut impl Rng,
    ) -> Self {
        let mut individuals = Vec::with_capacity(params.population_size);

        // Mix different initialization strategies
        let random_count = params.population_size / 2;
        let load_balanced_count = params.population_size / 4;
        let shortest_time_count = params.population_size - random_count - load_balanced_count;

        // Random generation (50%)
        for _ in 0..random_count {
            individuals.push(Chromosome::random(activities, rng));
        }

        // Load balanced (25%)
        for _ in 0..load_balanced_count {
            individuals.push(Chromosome::with_load_balancing(activities, resources, rng));
        }

        // Shortest processing time (25%)
        let process_times = build_process_times(activities);
        for _ in 0..shortest_time_count {
            individuals.push(Chromosome::with_shortest_time(
                activities,
                &process_times,
                rng,
            ));
        }

        Self {
            individuals,
            params,
            operators,
            generation: 0,
            best: None,
            fitness_history: Vec::new(),
        }
    }

    /// Evolve to next generation
    pub fn evolve(&mut self, activities: &[ActivityInfo], rng: &mut impl Rng) {
        // Sort by fitness (lower is better)
        self.individuals.sort_by(|a, b| {
            a.fitness
                .partial_cmp(&b.fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Update best individual
        if let Some(best_individual) = self.individuals.first() {
            match &self.best {
                None => self.best = Some(best_individual.clone()),
                Some(current_best) if best_individual.fitness < current_best.fitness => {
                    self.best = Some(best_individual.clone());
                }
                _ => {}
            }
            self.fitness_history.push(best_individual.fitness);
        }

        let mut new_population = Vec::with_capacity(self.params.population_size);

        // Elite preservation
        let elite_count = (self.params.population_size as f64 * self.params.elite_ratio) as usize;
        for i in 0..elite_count.min(self.individuals.len()) {
            new_population.push(self.individuals[i].clone());
        }

        // Generate rest through selection, crossover, mutation
        while new_population.len() < self.params.population_size {
            // Selection
            let parent1 = tournament_selection(&self.individuals, self.params.tournament_size, rng);
            let parent2 = tournament_selection(&self.individuals, self.params.tournament_size, rng);

            // Crossover
            let (mut child1, mut child2) =
                self.operators.crossover(parent1, parent2, activities, rng);

            // Mutation
            self.operators.mutate(&mut child1, activities, rng);
            self.operators.mutate(&mut child2, activities, rng);

            // Add children
            if new_population.len() < self.params.population_size {
                new_population.push(child1);
            }
            if new_population.len() < self.params.population_size {
                new_population.push(child2);
            }
        }

        self.individuals = new_population;
        self.generation += 1;
    }

    /// Check if converged
    pub fn is_converged(&self) -> bool {
        if self.fitness_history.len() < self.params.convergence_generations {
            return false;
        }

        let recent = &self.fitness_history
            [self.fitness_history.len() - self.params.convergence_generations..];

        if recent.is_empty() {
            return false;
        }

        let first = recent[0];
        let last = recent[recent.len() - 1];

        if first == 0.0 {
            return true;
        }

        let improvement = (first - last).abs() / first;
        improvement < self.params.convergence_threshold
    }

    /// Get population statistics
    pub fn statistics(&self) -> PopulationStats {
        if self.individuals.is_empty() {
            return PopulationStats::default();
        }

        let fitnesses: Vec<f64> = self.individuals.iter().map(|c| c.fitness).collect();
        let sum: f64 = fitnesses.iter().sum();
        let mean = sum / fitnesses.len() as f64;

        let variance: f64 =
            fitnesses.iter().map(|f| (f - mean).powi(2)).sum::<f64>() / fitnesses.len() as f64;

        let std_dev = variance.sqrt();

        PopulationStats {
            generation: self.generation,
            best_fitness: fitnesses.iter().cloned().fold(f64::INFINITY, f64::min),
            worst_fitness: fitnesses.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            mean_fitness: mean,
            std_dev,
        }
    }

    /// Get best individual
    pub fn get_best(&self) -> Option<&Chromosome> {
        self.best.as_ref()
    }
}

/// Population statistics
#[derive(Debug, Clone, Default)]
pub struct PopulationStats {
    pub generation: usize,
    pub best_fitness: f64,
    pub worst_fitness: f64,
    pub mean_fitness: f64,
    pub std_dev: f64,
}

/// Build process times map
fn build_process_times(
    activities: &[ActivityInfo],
) -> std::collections::HashMap<(String, String), i64> {
    let mut times = std::collections::HashMap::new();

    for act in activities {
        for resource in &act.candidates {
            times.insert(
                (act.activity_id.clone(), resource.clone()),
                act.process_time_ms,
            );
        }
    }

    times
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_activities() -> Vec<ActivityInfo> {
        vec![
            ActivityInfo {
                task_id: "T1".to_string(),
                activity_id: "A1".to_string(),
                sequence: 1,
                candidates: vec!["R1".to_string(), "R2".to_string()],
                process_time_ms: 30000,
            },
            ActivityInfo {
                task_id: "T1".to_string(),
                activity_id: "A2".to_string(),
                sequence: 2,
                candidates: vec!["R2".to_string(), "R3".to_string()],
                process_time_ms: 45000,
            },
            ActivityInfo {
                task_id: "T2".to_string(),
                activity_id: "A3".to_string(),
                sequence: 1,
                candidates: vec!["R1".to_string(), "R3".to_string()],
                process_time_ms: 20000,
            },
        ]
    }

    fn create_test_resources() -> Vec<Resource> {
        vec![
            Resource::new("R1", crate::models::ResourceType::Primary),
            Resource::new("R2", crate::models::ResourceType::Primary),
            Resource::new("R3", crate::models::ResourceType::Primary),
        ]
    }

    #[test]
    fn test_population_creation() {
        let activities = create_test_activities();
        let resources = create_test_resources();
        let mut rng = rand::thread_rng();

        let params = GaParams {
            population_size: 20,
            ..Default::default()
        };

        let population = Population::new(
            &activities,
            &resources,
            params,
            GeneticOperators::default(),
            &mut rng,
        );

        assert_eq!(population.individuals.len(), 20);
        assert_eq!(population.generation, 0);
    }

    #[test]
    fn test_population_evolution() {
        let activities = create_test_activities();
        let resources = create_test_resources();
        let mut rng = rand::thread_rng();

        let params = GaParams {
            population_size: 10,
            ..Default::default()
        };

        let mut population = Population::new(
            &activities,
            &resources,
            params,
            GeneticOperators::default(),
            &mut rng,
        );

        // Set fitness (simulation)
        for (i, individual) in population.individuals.iter_mut().enumerate() {
            individual.fitness = (i * 1000) as f64;
        }

        population.evolve(&activities, &mut rng);

        assert_eq!(population.generation, 1);
        assert_eq!(population.individuals.len(), 10);
        assert!(population.best.is_some());
    }

    #[test]
    fn test_population_statistics() {
        let activities = create_test_activities();
        let resources = create_test_resources();
        let mut rng = rand::thread_rng();

        let params = GaParams {
            population_size: 10,
            ..Default::default()
        };

        let mut population = Population::new(
            &activities,
            &resources,
            params,
            GeneticOperators::default(),
            &mut rng,
        );

        // Set fitness
        for (i, individual) in population.individuals.iter_mut().enumerate() {
            individual.fitness = ((i + 1) * 100) as f64;
        }

        let stats = population.statistics();

        assert_eq!(stats.best_fitness, 100.0);
        assert_eq!(stats.worst_fitness, 1000.0);
        assert!(stats.mean_fitness > 0.0);
    }

    #[test]
    fn test_convergence_detection() {
        let activities = create_test_activities();
        let resources = create_test_resources();
        let mut rng = rand::thread_rng();

        let params = GaParams {
            population_size: 10,
            convergence_generations: 5,
            convergence_threshold: 0.01,
            ..Default::default()
        };

        let mut population = Population::new(
            &activities,
            &resources,
            params,
            GeneticOperators::default(),
            &mut rng,
        );

        // Simulate convergence with same fitness
        population.fitness_history = vec![100.0, 100.0, 100.0, 100.0, 100.0];

        assert!(population.is_converged());

        // Still improving
        population.fitness_history = vec![100.0, 90.0, 80.0, 70.0, 60.0];
        assert!(!population.is_converged());
    }

    #[test]
    fn test_elite_preservation() {
        let activities = create_test_activities();
        let resources = create_test_resources();
        let mut rng = rand::thread_rng();

        let params = GaParams {
            population_size: 10,
            elite_ratio: 0.2, // 20% = 2 individuals
            ..Default::default()
        };

        let mut population = Population::new(
            &activities,
            &resources,
            params,
            GeneticOperators::default(),
            &mut rng,
        );

        // Set fitness - 0 is best
        for (i, individual) in population.individuals.iter_mut().enumerate() {
            individual.fitness = (i * 100) as f64;
        }

        let best_before = population.individuals[0].clone();
        population.evolve(&activities, &mut rng);

        // Best individual should be preserved
        assert!(population
            .individuals
            .iter()
            .any(|c| c.osv == best_before.osv));
    }
}
