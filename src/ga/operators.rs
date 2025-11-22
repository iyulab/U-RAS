//! Genetic Operators - Crossover and Mutation
//!
//! Domain-agnostic genetic operators for scheduling

use super::chromosome::{ActivityInfo, Chromosome};
use rand::prelude::*;

/// Genetic operators configuration
#[derive(Debug, Clone)]
pub struct GeneticOperators {
    pub crossover_type: CrossoverType,
    pub mutation_type: MutationType,
}

/// Crossover types
#[derive(Debug, Clone, Copy)]
pub enum CrossoverType {
    /// Precedence Operation Crossover
    POX,
    /// Linear Order Crossover
    LOX,
    /// Job-based Order Crossover
    JOX,
}

/// Mutation types
#[derive(Debug, Clone, Copy)]
pub enum MutationType {
    /// Swap two positions
    Swap,
    /// Insert at new position
    Insert,
    /// Invert segment
    Invert,
}

impl Default for GeneticOperators {
    fn default() -> Self {
        Self {
            crossover_type: CrossoverType::POX,
            mutation_type: MutationType::Swap,
        }
    }
}

/// Tournament selection
pub fn tournament_selection<'a>(
    population: &'a [Chromosome],
    tournament_size: usize,
    rng: &mut impl Rng,
) -> &'a Chromosome {
    let mut best_idx = rng.gen_range(0..population.len());

    for _ in 1..tournament_size {
        let idx = rng.gen_range(0..population.len());
        if population[idx].fitness < population[best_idx].fitness {
            best_idx = idx;
        }
    }

    &population[best_idx]
}

impl GeneticOperators {
    /// Perform crossover
    pub fn crossover(
        &self,
        parent1: &Chromosome,
        parent2: &Chromosome,
        activities: &[ActivityInfo],
        rng: &mut impl Rng,
    ) -> (Chromosome, Chromosome) {
        match self.crossover_type {
            CrossoverType::POX => self.pox_crossover(parent1, parent2, activities, rng),
            CrossoverType::LOX => self.lox_crossover(parent1, parent2, activities, rng),
            CrossoverType::JOX => self.jox_crossover(parent1, parent2, activities, rng),
        }
    }

    /// Perform mutation
    pub fn mutate(
        &self,
        chromosome: &mut Chromosome,
        activities: &[ActivityInfo],
        rng: &mut impl Rng,
    ) {
        // OSV mutation
        match self.mutation_type {
            MutationType::Swap => self.swap_mutation(&mut chromosome.osv, rng),
            MutationType::Insert => self.insert_mutation(&mut chromosome.osv, rng),
            MutationType::Invert => self.invert_mutation(&mut chromosome.osv, rng),
        }

        // MAV mutation - change random resource
        if !chromosome.mav.is_empty() && !activities.is_empty() {
            let idx = rng.gen_range(0..chromosome.mav.len().min(activities.len()));
            if !activities[idx].candidates.is_empty() {
                chromosome.mav[idx] = activities[idx].candidates.choose(rng).unwrap().clone();
            }
        }

        chromosome.fitness = f64::INFINITY;
    }

    /// POX crossover
    fn pox_crossover(
        &self,
        p1: &Chromosome,
        p2: &Chromosome,
        activities: &[ActivityInfo],
        rng: &mut impl Rng,
    ) -> (Chromosome, Chromosome) {
        let task_ids: Vec<String> = activities
            .iter()
            .map(|a| a.task_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        if task_ids.is_empty() {
            return (p1.clone(), p2.clone());
        }

        // Random task set
        let set_size = rng.gen_range(1..=task_ids.len().max(1));
        let selected: std::collections::HashSet<String> =
            task_ids.choose_multiple(rng, set_size).cloned().collect();

        let child1_osv = self.pox_build_child(&p1.osv, &p2.osv, &selected);
        let child2_osv = self.pox_build_child(&p2.osv, &p1.osv, &selected);

        let mut child1 = Chromosome::random(activities, rng);
        let mut child2 = Chromosome::random(activities, rng);

        child1.osv = child1_osv;
        child2.osv = child2_osv;

        // Inherit MAV from parents
        for i in 0..child1.mav.len().min(p1.mav.len()) {
            child1.mav[i] = p1.mav[i].clone();
        }
        for i in 0..child2.mav.len().min(p2.mav.len()) {
            child2.mav[i] = p2.mav[i].clone();
        }

        (child1, child2)
    }

    fn pox_build_child(
        &self,
        p1: &[String],
        p2: &[String],
        selected: &std::collections::HashSet<String>,
    ) -> Vec<String> {
        let mut child = vec![String::new(); p1.len()];
        let mut p2_iter = p2.iter().filter(|t| !selected.contains(*t)).peekable();

        for (i, task) in p1.iter().enumerate() {
            if selected.contains(task) {
                child[i] = task.clone();
            } else if let Some(t) = p2_iter.next() {
                child[i] = t.clone();
            }
        }

        child
    }

    /// LOX crossover
    fn lox_crossover(
        &self,
        p1: &Chromosome,
        p2: &Chromosome,
        activities: &[ActivityInfo],
        rng: &mut impl Rng,
    ) -> (Chromosome, Chromosome) {
        if p1.osv.len() < 2 {
            return (p1.clone(), p2.clone());
        }

        let len = p1.osv.len();
        let mut points: Vec<usize> = (0..len).collect();
        points.shuffle(rng);
        let (start, end) = if points[0] < points[1] {
            (points[0], points[1])
        } else {
            (points[1], points[0])
        };

        let child1_osv = self.lox_build_child(&p1.osv, &p2.osv, start, end);
        let child2_osv = self.lox_build_child(&p2.osv, &p1.osv, start, end);

        let mut child1 = Chromosome::random(activities, rng);
        let mut child2 = Chromosome::random(activities, rng);

        child1.osv = child1_osv;
        child2.osv = child2_osv;

        (child1, child2)
    }

    fn lox_build_child(
        &self,
        p1: &[String],
        p2: &[String],
        start: usize,
        end: usize,
    ) -> Vec<String> {
        let mut child = vec![String::new(); p1.len()];
        let segment: Vec<String> = p1[start..=end].to_vec();

        // Copy segment
        for (i, item) in segment.iter().enumerate() {
            child[start + i] = item.clone();
        }

        // Fill rest from p2
        let mut child_idx = (end + 1) % p1.len();
        for item in p2.iter().cycle().skip(end + 1).take(p2.len()) {
            if child_idx == start {
                break;
            }
            if (!segment.contains(item)
                || segment.iter().filter(|&x| x == item).count()
                    < p2.iter().filter(|&x| x == item).count())
                && child[child_idx].is_empty()
            {
                child[child_idx] = item.clone();
                child_idx = (child_idx + 1) % p1.len();
            }
        }

        child
    }

    /// JOX crossover (simplified)
    fn jox_crossover(
        &self,
        p1: &Chromosome,
        p2: &Chromosome,
        activities: &[ActivityInfo],
        rng: &mut impl Rng,
    ) -> (Chromosome, Chromosome) {
        // Simplified: use POX
        self.pox_crossover(p1, p2, activities, rng)
    }

    /// Swap mutation
    fn swap_mutation(&self, osv: &mut Vec<String>, rng: &mut impl Rng) {
        if osv.len() < 2 {
            return;
        }
        let i = rng.gen_range(0..osv.len());
        let j = rng.gen_range(0..osv.len());
        osv.swap(i, j);
    }

    /// Insert mutation
    fn insert_mutation(&self, osv: &mut Vec<String>, rng: &mut impl Rng) {
        if osv.len() < 2 {
            return;
        }
        let from = rng.gen_range(0..osv.len());
        let to = rng.gen_range(0..osv.len());
        let item = osv.remove(from);
        osv.insert(to, item);
    }

    /// Invert mutation
    fn invert_mutation(&self, osv: &mut Vec<String>, rng: &mut impl Rng) {
        if osv.len() < 2 {
            return;
        }
        let mut i = rng.gen_range(0..osv.len());
        let mut j = rng.gen_range(0..osv.len());
        if i > j {
            std::mem::swap(&mut i, &mut j);
        }
        osv[i..=j].reverse();
    }
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
                candidates: vec!["R1".to_string()],
                process_time_ms: 1000,
            },
            ActivityInfo {
                task_id: "T1".to_string(),
                activity_id: "A2".to_string(),
                sequence: 2,
                candidates: vec!["R1".to_string()],
                process_time_ms: 1000,
            },
            ActivityInfo {
                task_id: "T2".to_string(),
                activity_id: "A3".to_string(),
                sequence: 1,
                candidates: vec!["R1".to_string()],
                process_time_ms: 1000,
            },
        ]
    }

    #[test]
    fn test_crossover() {
        let activities = create_test_activities();
        let mut rng = rand::thread_rng();
        let operators = GeneticOperators::default();

        let p1 = Chromosome::random(&activities, &mut rng);
        let p2 = Chromosome::random(&activities, &mut rng);

        let (c1, c2) = operators.crossover(&p1, &p2, &activities, &mut rng);

        assert_eq!(c1.osv.len(), 3);
        assert_eq!(c2.osv.len(), 3);
    }

    #[test]
    fn test_mutation() {
        let activities = create_test_activities();
        let mut rng = rand::thread_rng();
        let operators = GeneticOperators::default();

        let mut chromosome = Chromosome::random(&activities, &mut rng);
        operators.mutate(&mut chromosome, &activities, &mut rng);

        assert_eq!(chromosome.osv.len(), 3);
    }
}
