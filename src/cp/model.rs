//! CP Model - Constraint Programming Model Definition

use crate::cp::variables::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// CP 제약 조건
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constraint {
    /// 비중첩 제약 (같은 자원에서)
    NoOverlap {
        intervals: Vec<String>,
        transition_matrix: Option<TransitionMatrix>,
    },
    /// 누적 제약 (자원 용량)
    Cumulative {
        intervals: Vec<String>,
        demands: Vec<i64>,
        capacity: i64,
    },
    /// 선행 제약
    Precedence {
        before: String,
        after: String,
        min_delay: i64,
    },
    /// 동시 시작
    SameStart {
        interval1: String,
        interval2: String,
    },
    /// 동시 종료
    SameEnd {
        interval1: String,
        interval2: String,
    },
    /// 선택적 제약 (하나만 선택)
    Alternative {
        main: String,
        alternatives: Vec<String>,
    },
}

/// 전환 행렬 (Setup Time)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionMatrix {
    /// 제품/공정 유형 목록
    pub types: Vec<String>,
    /// 전환 시간 행렬 [from][to]
    pub times: Vec<Vec<i64>>,
}

impl TransitionMatrix {
    pub fn new(types: Vec<String>) -> Self {
        let n = types.len();
        Self {
            types,
            times: vec![vec![0; n]; n],
        }
    }

    pub fn set_time(&mut self, from: &str, to: &str, time: i64) {
        if let (Some(i), Some(j)) = (
            self.types.iter().position(|t| t == from),
            self.types.iter().position(|t| t == to),
        ) {
            self.times[i][j] = time;
        }
    }

    pub fn get_time(&self, from: &str, to: &str) -> i64 {
        if let (Some(i), Some(j)) = (
            self.types.iter().position(|t| t == from),
            self.types.iter().position(|t| t == to),
        ) {
            self.times[i][j]
        } else {
            0
        }
    }
}

/// CP 목적 함수
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Objective {
    /// Makespan 최소화
    MinimizeMakespan,
    /// 총 지연 최소화
    MinimizeTotalTardiness {
        due_dates: HashMap<String, i64>,
        weights: HashMap<String, f64>,
    },
    /// 가중합 최소화
    MinimizeWeightedSum { terms: Vec<(String, f64)> },
    /// 다목적 (계층적)
    Hierarchical { objectives: Vec<Objective> },
}

/// CP 모델
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpModel {
    /// 모델 이름
    pub name: String,
    /// 간격 변수
    pub intervals: HashMap<String, IntervalVar>,
    /// 정수 변수
    pub int_vars: HashMap<String, IntVar>,
    /// 불리언 변수
    pub bool_vars: HashMap<String, BoolVar>,
    /// 제약 조건
    pub constraints: Vec<Constraint>,
    /// 목적 함수
    pub objective: Option<Objective>,
    /// 계획 수평선 (Horizon)
    pub horizon: i64,
}

impl CpModel {
    /// 새 CP 모델 생성
    pub fn new(name: impl Into<String>, horizon: i64) -> Self {
        Self {
            name: name.into(),
            intervals: HashMap::new(),
            int_vars: HashMap::new(),
            bool_vars: HashMap::new(),
            constraints: Vec::new(),
            objective: None,
            horizon,
        }
    }

    /// 간격 변수 추가
    pub fn add_interval(&mut self, var: IntervalVar) {
        self.intervals.insert(var.name.clone(), var);
    }

    /// 정수 변수 추가
    pub fn add_int_var(&mut self, var: IntVar) {
        self.int_vars.insert(var.name.clone(), var);
    }

    /// 불리언 변수 추가
    pub fn add_bool_var(&mut self, var: BoolVar) {
        self.bool_vars.insert(var.name.clone(), var);
    }

    /// 제약 조건 추가
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    /// 비중첩 제약 추가
    pub fn add_no_overlap(&mut self, intervals: Vec<String>) {
        self.constraints.push(Constraint::NoOverlap {
            intervals,
            transition_matrix: None,
        });
    }

    /// Setup Time이 있는 비중첩 제약 추가
    pub fn add_no_overlap_with_setup(&mut self, intervals: Vec<String>, matrix: TransitionMatrix) {
        self.constraints.push(Constraint::NoOverlap {
            intervals,
            transition_matrix: Some(matrix),
        });
    }

    /// 누적 제약 추가
    pub fn add_cumulative(&mut self, intervals: Vec<String>, demands: Vec<i64>, capacity: i64) {
        self.constraints.push(Constraint::Cumulative {
            intervals,
            demands,
            capacity,
        });
    }

    /// 선행 제약 추가
    pub fn add_precedence(&mut self, before: String, after: String, min_delay: i64) {
        self.constraints.push(Constraint::Precedence {
            before,
            after,
            min_delay,
        });
    }

    /// 목적 함수 설정
    pub fn set_objective(&mut self, objective: Objective) {
        self.objective = Some(objective);
    }

    /// Makespan 최소화 설정
    pub fn minimize_makespan(&mut self) {
        self.objective = Some(Objective::MinimizeMakespan);
    }

    /// 모델 유효성 검사
    pub fn validate(&self) -> Result<(), String> {
        // 모든 제약의 간격이 정의되어 있는지 확인
        for constraint in &self.constraints {
            match constraint {
                Constraint::NoOverlap { intervals, .. } => {
                    for name in intervals {
                        if !self.intervals.contains_key(name) {
                            return Err(format!("Undefined interval: {}", name));
                        }
                    }
                }
                Constraint::Cumulative {
                    intervals, demands, ..
                } => {
                    if intervals.len() != demands.len() {
                        return Err("Cumulative: intervals and demands length mismatch".into());
                    }
                    for name in intervals {
                        if !self.intervals.contains_key(name) {
                            return Err(format!("Undefined interval: {}", name));
                        }
                    }
                }
                Constraint::Precedence { before, after, .. } => {
                    if !self.intervals.contains_key(before) {
                        return Err(format!("Undefined interval: {}", before));
                    }
                    if !self.intervals.contains_key(after) {
                        return Err(format!("Undefined interval: {}", after));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cp_model_creation() {
        let mut model = CpModel::new("test", 1_000_000);

        let op1 = IntervalVar::new("op1", 0, 100_000, 50_000, 200_000);
        let op2 = IntervalVar::new("op2", 0, 100_000, 30_000, 200_000);

        model.add_interval(op1);
        model.add_interval(op2);

        model.add_no_overlap(vec!["op1".into(), "op2".into()]);
        model.minimize_makespan();

        assert_eq!(model.intervals.len(), 2);
        assert_eq!(model.constraints.len(), 1);
        assert!(model.objective.is_some());
        assert!(model.validate().is_ok());
    }

    #[test]
    fn test_transition_matrix() {
        let mut matrix = TransitionMatrix::new(vec!["A".into(), "B".into(), "C".into()]);

        matrix.set_time("A", "B", 10_000);
        matrix.set_time("B", "C", 5_000);

        assert_eq!(matrix.get_time("A", "B"), 10_000);
        assert_eq!(matrix.get_time("B", "C"), 5_000);
        assert_eq!(matrix.get_time("A", "A"), 0);
    }

    #[test]
    fn test_precedence_constraint() {
        let mut model = CpModel::new("test", 1_000_000);

        model.add_interval(IntervalVar::new("op1", 0, 100_000, 50_000, 200_000));
        model.add_interval(IntervalVar::new("op2", 0, 100_000, 30_000, 200_000));

        model.add_precedence("op1".into(), "op2".into(), 0);

        assert!(model.validate().is_ok());
    }

    #[test]
    fn test_validation_error() {
        let mut model = CpModel::new("test", 1_000_000);

        // 정의되지 않은 간격 참조
        model.add_no_overlap(vec!["undefined".into()]);

        assert!(model.validate().is_err());
    }
}
