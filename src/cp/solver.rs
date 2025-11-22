//! CP Solver - Constraint Programming Solver Interface

use crate::cp::model::CpModel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Solver 상태
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SolverStatus {
    /// 최적해 발견
    Optimal,
    /// 실행 가능 해 발견
    Feasible,
    /// 실행 불가능
    Infeasible,
    /// 모델 정의 오류
    ModelInvalid,
    /// 시간 초과
    Timeout,
    /// 알 수 없음
    Unknown,
}

/// 간격 변수 해
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntervalSolution {
    /// 시작 시간
    pub start: i64,
    /// 종료 시간
    pub end: i64,
    /// 기간
    pub duration: i64,
    /// 수행 여부 (선택적인 경우)
    pub is_present: bool,
}

/// CP Solver 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpSolution {
    /// 상태
    pub status: SolverStatus,
    /// 목적 함수 값
    pub objective_value: Option<f64>,
    /// 간격 변수 해
    pub intervals: HashMap<String, IntervalSolution>,
    /// 정수 변수 해
    pub int_vars: HashMap<String, i64>,
    /// 불리언 변수 해
    pub bool_vars: HashMap<String, bool>,
    /// 해결 시간 (밀리초)
    pub solve_time_ms: i64,
    /// 탐색된 노드 수
    pub num_nodes: u64,
}

impl CpSolution {
    /// 빈 해 생성
    pub fn empty(status: SolverStatus) -> Self {
        Self {
            status,
            objective_value: None,
            intervals: HashMap::new(),
            int_vars: HashMap::new(),
            bool_vars: HashMap::new(),
            solve_time_ms: 0,
            num_nodes: 0,
        }
    }

    /// 최적/실행가능 여부
    pub fn is_solution_found(&self) -> bool {
        matches!(self.status, SolverStatus::Optimal | SolverStatus::Feasible)
    }

    /// Makespan 계산
    pub fn makespan(&self) -> i64 {
        self.intervals
            .values()
            .filter(|sol| sol.is_present)
            .map(|sol| sol.end)
            .max()
            .unwrap_or(0)
    }
}

/// Solver 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverConfig {
    /// 최대 실행 시간 (밀리초)
    pub time_limit_ms: i64,
    /// 최대 탐색 노드 수
    pub max_nodes: u64,
    /// 병렬 스레드 수
    pub num_workers: usize,
    /// 로그 출력
    pub log_search: bool,
    /// 첫 해만 찾기
    pub stop_after_first: bool,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            time_limit_ms: 60_000, // 1분
            max_nodes: 1_000_000,
            num_workers: 4,
            log_search: false,
            stop_after_first: false,
        }
    }
}

/// CP Solver 트레이트
pub trait CpSolver {
    /// 모델 해결
    fn solve(&self, model: &CpModel, config: &SolverConfig) -> CpSolution;
}

/// 기본 CP Solver (간단한 휴리스틱)
pub struct SimpleCpSolver;

impl SimpleCpSolver {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleCpSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl CpSolver for SimpleCpSolver {
    fn solve(&self, model: &CpModel, _config: &SolverConfig) -> CpSolution {
        // 모델 유효성 검사
        if let Err(_) = model.validate() {
            return CpSolution::empty(SolverStatus::ModelInvalid);
        }

        // 간단한 그리디 휴리스틱
        // 실제 구현에서는 CP solver를 사용
        let mut solution = CpSolution::empty(SolverStatus::Feasible);
        let mut current_time: HashMap<String, i64> = HashMap::new();

        // 모든 간격을 순차적으로 배치 (매우 단순한 휴리스틱)
        for (name, interval) in &model.intervals {
            let start = interval.start.min;
            let duration = interval.duration.fixed.unwrap_or(interval.duration.min);
            let end = start + duration;

            solution.intervals.insert(
                name.clone(),
                IntervalSolution {
                    start,
                    end,
                    duration,
                    is_present: true,
                },
            );

            current_time.insert(name.clone(), end);
        }

        solution.objective_value = Some(solution.makespan() as f64);
        solution
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cp::variables::IntervalVar;

    #[test]
    fn test_simple_solver() {
        let mut model = CpModel::new("test", 1_000_000);

        model.add_interval(IntervalVar::new("op1", 0, 100_000, 50_000, 200_000));
        model.add_interval(IntervalVar::new("op2", 0, 100_000, 30_000, 200_000));
        model.minimize_makespan();

        let solver = SimpleCpSolver::new();
        let solution = solver.solve(&model, &SolverConfig::default());

        assert!(solution.is_solution_found());
        assert_eq!(solution.intervals.len(), 2);
    }

    #[test]
    fn test_solution_makespan() {
        let mut solution = CpSolution::empty(SolverStatus::Feasible);

        solution.intervals.insert(
            "op1".into(),
            IntervalSolution {
                start: 0,
                end: 50_000,
                duration: 50_000,
                is_present: true,
            },
        );

        solution.intervals.insert(
            "op2".into(),
            IntervalSolution {
                start: 10_000,
                end: 80_000,
                duration: 70_000,
                is_present: true,
            },
        );

        assert_eq!(solution.makespan(), 80_000);
    }

    #[test]
    fn test_invalid_model() {
        let mut model = CpModel::new("test", 1_000_000);
        model.add_no_overlap(vec!["undefined".into()]);

        let solver = SimpleCpSolver::new();
        let solution = solver.solve(&model, &SolverConfig::default());

        assert_eq!(solution.status, SolverStatus::ModelInvalid);
    }
}
