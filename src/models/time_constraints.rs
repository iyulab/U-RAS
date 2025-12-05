//! Time Constraints and Duration Estimation
//!
//! Domain-agnostic time constraint models including:
//! - TimeWindow: Hard/soft time boundaries
//! - Violation: Constraint violation tracking
//! - PERT: 3-point duration estimation
//! - Probabilistic scheduling support

use serde::{Deserialize, Serialize};

// ================================
// TimeWindow Constraints
// ================================

/// Time window constraint type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimeWindowType {
    /// Must be satisfied (schedule invalid if violated)
    Hard,
    /// Should be satisfied (penalty if violated)
    Soft,
}

impl Default for TimeWindowType {
    fn default() -> Self {
        TimeWindowType::Soft
    }
}

/// Time window constraint for activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    /// Earliest allowed start time (ms)
    pub earliest_start_ms: Option<i64>,
    /// Latest allowed start time (ms)
    pub latest_start_ms: Option<i64>,
    /// Earliest allowed end time (ms)
    pub earliest_end_ms: Option<i64>,
    /// Latest allowed end time (ms) - due date
    pub latest_end_ms: Option<i64>,
    /// Constraint type
    pub window_type: TimeWindowType,
    /// Penalty per millisecond of violation (for soft constraints)
    pub penalty_per_ms: f64,
}

impl TimeWindow {
    /// Create empty time window
    pub fn new() -> Self {
        Self {
            earliest_start_ms: None,
            latest_start_ms: None,
            earliest_end_ms: None,
            latest_end_ms: None,
            window_type: TimeWindowType::Soft,
            penalty_per_ms: 1.0,
        }
    }

    /// Create time window with start/end boundaries
    pub fn bounded(start_ms: i64, end_ms: i64) -> Self {
        Self {
            earliest_start_ms: Some(start_ms),
            latest_start_ms: None,
            earliest_end_ms: None,
            latest_end_ms: Some(end_ms),
            window_type: TimeWindowType::Soft,
            penalty_per_ms: 1.0,
        }
    }

    /// Create hard deadline (must complete by)
    pub fn deadline(deadline_ms: i64) -> Self {
        Self {
            earliest_start_ms: None,
            latest_start_ms: None,
            earliest_end_ms: None,
            latest_end_ms: Some(deadline_ms),
            window_type: TimeWindowType::Hard,
            penalty_per_ms: 0.0,
        }
    }

    /// Create release time (cannot start before)
    pub fn release(release_ms: i64) -> Self {
        Self {
            earliest_start_ms: Some(release_ms),
            latest_start_ms: None,
            earliest_end_ms: None,
            latest_end_ms: None,
            window_type: TimeWindowType::Hard,
            penalty_per_ms: 0.0,
        }
    }

    /// Set as hard constraint
    pub fn hard(mut self) -> Self {
        self.window_type = TimeWindowType::Hard;
        self.penalty_per_ms = 0.0;
        self
    }

    /// Set as soft constraint with penalty
    pub fn soft(mut self, penalty_per_ms: f64) -> Self {
        self.window_type = TimeWindowType::Soft;
        self.penalty_per_ms = penalty_per_ms;
        self
    }

    /// Set earliest start
    pub fn with_earliest_start(mut self, ms: i64) -> Self {
        self.earliest_start_ms = Some(ms);
        self
    }

    /// Set latest start
    pub fn with_latest_start(mut self, ms: i64) -> Self {
        self.latest_start_ms = Some(ms);
        self
    }

    /// Set latest end (due date)
    pub fn with_due_date(mut self, ms: i64) -> Self {
        self.latest_end_ms = Some(ms);
        self
    }

    /// Check if a scheduled time violates the window
    pub fn check_violation(&self, start_ms: i64, end_ms: i64) -> Option<TimeWindowViolation> {
        let mut total_early_ms = 0i64;
        let mut total_late_ms = 0i64;

        // Check start time
        if let Some(earliest) = self.earliest_start_ms {
            if start_ms < earliest {
                total_early_ms += earliest - start_ms;
            }
        }
        if let Some(latest) = self.latest_start_ms {
            if start_ms > latest {
                total_late_ms += start_ms - latest;
            }
        }

        // Check end time
        if let Some(earliest) = self.earliest_end_ms {
            if end_ms < earliest {
                total_early_ms += earliest - end_ms;
            }
        }
        if let Some(latest) = self.latest_end_ms {
            if end_ms > latest {
                total_late_ms += end_ms - latest;
            }
        }

        if total_early_ms == 0 && total_late_ms == 0 {
            return None;
        }

        Some(TimeWindowViolation {
            early_ms: total_early_ms,
            late_ms: total_late_ms,
            severity: if self.window_type == TimeWindowType::Hard {
                ViolationSeverity::Critical
            } else {
                ViolationSeverity::Minor
            },
            penalty: (total_early_ms + total_late_ms) as f64 * self.penalty_per_ms,
        })
    }
}

impl Default for TimeWindow {
    fn default() -> Self {
        Self::new()
    }
}

// ================================
// Violation Model
// ================================

/// Severity of constraint violation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ViolationSeverity {
    /// Informational - no impact
    Info,
    /// Minor - small penalty
    Minor,
    /// Major - significant penalty
    Major,
    /// Critical - schedule may be invalid
    Critical,
}

/// Time window violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindowViolation {
    /// Amount of time too early (ms)
    pub early_ms: i64,
    /// Amount of time too late (ms)
    pub late_ms: i64,
    /// Severity level
    pub severity: ViolationSeverity,
    /// Calculated penalty value
    pub penalty: f64,
}

impl TimeWindowViolation {
    /// Total violation time (absolute)
    pub fn total_violation_ms(&self) -> i64 {
        self.early_ms.abs() + self.late_ms.abs()
    }

    /// Check if this is a tardiness (late) violation
    pub fn is_tardy(&self) -> bool {
        self.late_ms > 0
    }

    /// Check if this is an early start violation
    pub fn is_early(&self) -> bool {
        self.early_ms > 0
    }
}

/// General constraint violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintViolation {
    /// Type of violation
    pub violation_type: ViolationType,
    /// Related entity IDs
    pub related_ids: Vec<String>,
    /// Severity level
    pub severity: ViolationSeverity,
    /// Violation message
    pub message: String,
    /// Penalty value
    pub penalty: f64,
}

/// Types of constraint violations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ViolationType {
    /// Time window violated
    TimeWindow,
    /// Resource capacity exceeded
    CapacityExceeded,
    /// Precedence constraint violated
    PrecedenceViolated,
    /// Resource unavailable
    ResourceUnavailable,
    /// Skill requirement not met
    SkillMismatch,
    /// Certification expired
    CertificationExpired,
    /// Material not available
    MaterialShortage,
    /// Other custom violation
    Custom(String),
}

impl ConstraintViolation {
    /// Create time window violation
    pub fn time_window(
        activity_id: &str,
        tardiness_ms: i64,
        severity: ViolationSeverity,
        penalty: f64,
    ) -> Self {
        Self {
            violation_type: ViolationType::TimeWindow,
            related_ids: vec![activity_id.to_string()],
            severity,
            message: format!("Activity {} is {} ms late", activity_id, tardiness_ms),
            penalty,
        }
    }

    /// Create capacity violation
    pub fn capacity_exceeded(resource_id: &str, exceeded_by: i32) -> Self {
        Self {
            violation_type: ViolationType::CapacityExceeded,
            related_ids: vec![resource_id.to_string()],
            severity: ViolationSeverity::Critical,
            message: format!("Resource {} capacity exceeded by {}", resource_id, exceeded_by),
            penalty: exceeded_by as f64 * 1000.0,
        }
    }

    /// Create precedence violation
    pub fn precedence_violated(before_id: &str, after_id: &str, overlap_ms: i64) -> Self {
        Self {
            violation_type: ViolationType::PrecedenceViolated,
            related_ids: vec![before_id.to_string(), after_id.to_string()],
            severity: ViolationSeverity::Critical,
            message: format!(
                "Activity {} must complete before {} (overlap: {} ms)",
                before_id, after_id, overlap_ms
            ),
            penalty: overlap_ms as f64 * 10.0,
        }
    }
}

// ================================
// PERT 3-Point Estimation
// ================================

/// PERT (Program Evaluation and Review Technique) duration estimation
///
/// Uses three-point estimation:
/// - Optimistic (O): Best-case scenario
/// - Most Likely (M): Normal conditions
/// - Pessimistic (P): Worst-case scenario
///
/// Mean = (O + 4M + P) / 6
/// StdDev = (P - O) / 6
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PertEstimate {
    /// Optimistic duration (ms)
    pub optimistic_ms: i64,
    /// Most likely duration (ms)
    pub most_likely_ms: i64,
    /// Pessimistic duration (ms)
    pub pessimistic_ms: i64,
}

impl PertEstimate {
    /// Create new PERT estimate
    pub fn new(optimistic_ms: i64, most_likely_ms: i64, pessimistic_ms: i64) -> Self {
        Self {
            optimistic_ms,
            most_likely_ms,
            pessimistic_ms,
        }
    }

    /// Create from percentage variance
    /// e.g., from_variance(1000, 0.2) creates O=800, M=1000, P=1200
    pub fn from_variance(base_ms: i64, variance_ratio: f64) -> Self {
        let variance = (base_ms as f64 * variance_ratio) as i64;
        Self {
            optimistic_ms: base_ms - variance,
            most_likely_ms: base_ms,
            pessimistic_ms: base_ms + variance,
        }
    }

    /// Create symmetric estimate
    pub fn symmetric(most_likely_ms: i64, spread_ms: i64) -> Self {
        Self {
            optimistic_ms: most_likely_ms - spread_ms,
            most_likely_ms,
            pessimistic_ms: most_likely_ms + spread_ms,
        }
    }

    /// Calculate PERT mean (expected duration)
    /// Mean = (O + 4M + P) / 6
    pub fn mean_ms(&self) -> f64 {
        (self.optimistic_ms as f64 + 4.0 * self.most_likely_ms as f64 + self.pessimistic_ms as f64)
            / 6.0
    }

    /// Calculate PERT standard deviation
    /// StdDev = (P - O) / 6
    pub fn std_dev_ms(&self) -> f64 {
        (self.pessimistic_ms - self.optimistic_ms) as f64 / 6.0
    }

    /// Calculate variance
    pub fn variance_ms(&self) -> f64 {
        let sd = self.std_dev_ms();
        sd * sd
    }

    /// Get duration at specified confidence level
    /// Uses normal distribution approximation
    /// 50% -> mean, 85% -> mean + 1*sd, 95% -> mean + 1.65*sd, 99% -> mean + 2.33*sd
    pub fn duration_at_confidence(&self, confidence: f64) -> i64 {
        let z_score = confidence_to_z(confidence);
        (self.mean_ms() + z_score * self.std_dev_ms()) as i64
    }

    /// Calculate probability of completing within given duration
    pub fn probability_of_completion(&self, duration_ms: i64) -> f64 {
        let z = (duration_ms as f64 - self.mean_ms()) / self.std_dev_ms();
        standard_normal_cdf(z)
    }

    /// Get 50th percentile (median) duration
    pub fn p50(&self) -> i64 {
        self.mean_ms() as i64
    }

    /// Get 85th percentile duration
    pub fn p85(&self) -> i64 {
        self.duration_at_confidence(0.85)
    }

    /// Get 95th percentile duration
    pub fn p95(&self) -> i64 {
        self.duration_at_confidence(0.95)
    }
}

impl Default for PertEstimate {
    fn default() -> Self {
        Self {
            optimistic_ms: 0,
            most_likely_ms: 0,
            pessimistic_ms: 0,
        }
    }
}

// ================================
// Probabilistic Scheduling Support
// ================================

/// Duration distribution for probabilistic scheduling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DurationDistribution {
    /// Fixed duration (deterministic)
    Fixed(i64),
    /// PERT-based distribution
    Pert(PertEstimate),
    /// Uniform distribution between min and max
    Uniform { min_ms: i64, max_ms: i64 },
    /// Triangular distribution
    Triangular {
        min_ms: i64,
        mode_ms: i64,
        max_ms: i64,
    },
    /// Log-normal (common for task durations)
    LogNormal { mu: f64, sigma: f64 },
}

impl DurationDistribution {
    /// Get expected (mean) duration
    pub fn expected_duration_ms(&self) -> f64 {
        match self {
            DurationDistribution::Fixed(d) => *d as f64,
            DurationDistribution::Pert(p) => p.mean_ms(),
            DurationDistribution::Uniform { min_ms, max_ms } => (*min_ms + *max_ms) as f64 / 2.0,
            DurationDistribution::Triangular {
                min_ms,
                mode_ms,
                max_ms,
            } => (*min_ms + *mode_ms + *max_ms) as f64 / 3.0,
            DurationDistribution::LogNormal { mu, sigma } => {
                (mu + sigma.powi(2) / 2.0).exp()
            }
        }
    }

    /// Get duration at confidence level
    pub fn duration_at_confidence(&self, confidence: f64) -> i64 {
        match self {
            DurationDistribution::Fixed(d) => *d,
            DurationDistribution::Pert(p) => p.duration_at_confidence(confidence),
            DurationDistribution::Uniform { min_ms, max_ms } => {
                let range = max_ms - min_ms;
                min_ms + (range as f64 * confidence) as i64
            }
            DurationDistribution::Triangular {
                min_ms,
                mode_ms,
                max_ms,
            } => {
                // Simplified triangular quantile
                let fc = (*mode_ms - *min_ms) as f64 / (*max_ms - *min_ms) as f64;
                if confidence < fc {
                    *min_ms + ((*max_ms - *min_ms) as f64 * (*mode_ms - *min_ms) as f64 * confidence).sqrt() as i64
                } else {
                    *max_ms - ((*max_ms - *min_ms) as f64 * (*max_ms - *mode_ms) as f64 * (1.0 - confidence)).sqrt() as i64
                }
            }
            DurationDistribution::LogNormal { mu, sigma } => {
                let z = confidence_to_z(confidence);
                (mu + z * sigma).exp() as i64
            }
        }
    }

    /// Create from PERT estimates
    pub fn from_pert(optimistic: i64, most_likely: i64, pessimistic: i64) -> Self {
        DurationDistribution::Pert(PertEstimate::new(optimistic, most_likely, pessimistic))
    }
}

impl Default for DurationDistribution {
    fn default() -> Self {
        DurationDistribution::Fixed(0)
    }
}

// ================================
// Helper Functions
// ================================

/// Convert confidence level to Z-score (standard normal)
fn confidence_to_z(confidence: f64) -> f64 {
    // Common confidence levels
    if confidence <= 0.5 {
        return 0.0;
    }
    if confidence >= 0.999 {
        return 3.09;
    }

    // Approximation using Abramowitz and Stegun formula
    let p = confidence;
    let t = (-2.0 * (1.0 - p).ln()).sqrt();
    let c0 = 2.515517;
    let c1 = 0.802853;
    let c2 = 0.010328;
    let d1 = 1.432788;
    let d2 = 0.189269;
    let d3 = 0.001308;

    t - (c0 + c1 * t + c2 * t * t) / (1.0 + d1 * t + d2 * t * t + d3 * t * t * t)
}

/// Standard normal CDF approximation
fn standard_normal_cdf(x: f64) -> f64 {
    // Approximation using error function
    let t = 1.0 / (1.0 + 0.2316419 * x.abs());
    let d = 0.3989423 * (-x * x / 2.0).exp();
    let p = d * t * (0.3193815 + t * (-0.3565638 + t * (1.781478 + t * (-1.821256 + t * 1.330274))));

    if x > 0.0 {
        1.0 - p
    } else {
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_window_basic() {
        let window = TimeWindow::bounded(1000, 5000);

        // Within window - no violation
        assert!(window.check_violation(1000, 4000).is_none());

        // Starts too early
        let v = window.check_violation(500, 4000).unwrap();
        assert_eq!(v.early_ms, 500);
        assert!(v.is_early());

        // Ends too late
        let v = window.check_violation(2000, 6000).unwrap();
        assert_eq!(v.late_ms, 1000);
        assert!(v.is_tardy());
    }

    #[test]
    fn test_time_window_hard_vs_soft() {
        let hard = TimeWindow::deadline(5000).hard();
        let soft = TimeWindow::deadline(5000).soft(2.0);

        // Both detect violation
        let vh = hard.check_violation(0, 6000).unwrap();
        let vs = soft.check_violation(0, 6000).unwrap();

        // Hard has Critical severity
        assert_eq!(vh.severity, ViolationSeverity::Critical);
        // Soft has Minor severity with penalty
        assert_eq!(vs.severity, ViolationSeverity::Minor);
        assert!((vs.penalty - 2000.0).abs() < 0.01); // 1000ms * 2.0
    }

    #[test]
    fn test_pert_calculation() {
        // Classic PERT example: O=4, M=6, P=14 (in hours, but same math applies)
        let pert = PertEstimate::new(4000, 6000, 14000);

        // Mean = (4 + 4*6 + 14) / 6 = 42/6 = 7
        let mean = pert.mean_ms();
        assert!((mean - 7000.0).abs() < 0.01);

        // StdDev = (14 - 4) / 6 = 10/6 ≈ 1.667
        let sd = pert.std_dev_ms();
        assert!((sd - 1666.67).abs() < 1.0);
    }

    #[test]
    fn test_pert_from_variance() {
        let pert = PertEstimate::from_variance(10000, 0.2);

        assert_eq!(pert.optimistic_ms, 8000);
        assert_eq!(pert.most_likely_ms, 10000);
        assert_eq!(pert.pessimistic_ms, 12000);
    }

    #[test]
    fn test_pert_confidence_levels() {
        let pert = PertEstimate::new(6000, 10000, 14000);

        // P50 ≈ mean
        let p50 = pert.p50();
        let mean = pert.mean_ms() as i64;
        assert!((p50 - mean).abs() < 100);

        // P95 > P85 > P50
        assert!(pert.p95() > pert.p85());
        assert!(pert.p85() > pert.p50());
    }

    #[test]
    fn test_duration_distribution_expected() {
        let fixed = DurationDistribution::Fixed(5000);
        assert!((fixed.expected_duration_ms() - 5000.0).abs() < 0.01);

        let uniform = DurationDistribution::Uniform {
            min_ms: 4000,
            max_ms: 6000,
        };
        assert!((uniform.expected_duration_ms() - 5000.0).abs() < 0.01);

        let triangular = DurationDistribution::Triangular {
            min_ms: 3000,
            mode_ms: 5000,
            max_ms: 7000,
        };
        assert!((triangular.expected_duration_ms() - 5000.0).abs() < 0.01);
    }

    #[test]
    fn test_constraint_violation_creation() {
        let tw_v = ConstraintViolation::time_window("OP-001", 5000, ViolationSeverity::Minor, 500.0);
        assert_eq!(tw_v.violation_type, ViolationType::TimeWindow);
        assert!(tw_v.message.contains("OP-001"));

        let cap_v = ConstraintViolation::capacity_exceeded("M-001", 3);
        assert_eq!(cap_v.violation_type, ViolationType::CapacityExceeded);
        assert_eq!(cap_v.severity, ViolationSeverity::Critical);
    }

    #[test]
    fn test_violation_severity_ordering() {
        assert!(ViolationSeverity::Critical > ViolationSeverity::Major);
        assert!(ViolationSeverity::Major > ViolationSeverity::Minor);
        assert!(ViolationSeverity::Minor > ViolationSeverity::Info);
    }
}
