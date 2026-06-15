//! 阶段识别评估器（Phase 7 Batch B）。
//!
//! 对单个阶段进行形式化阶段识别评估：比较 `recognized_status` 与人工期望，
//! 输出 `StageEvaluationTarget` 与阶段级 `QualityIssue`。
//!
//! 本模块**不**检查 evidence / understanding / view / qa 的存在性，
//! 仅负责阶段识别状态比对；存在性由 reporter 或其他 evaluator 填充。

use crate::quality::issue_builder::make_issue;
use crate::quality::models::{
    ArtifactKind, QualityIssue, QualityIssueKind, QualitySeverity, StageEvaluationTarget,
};

/// 阶段识别评估输入。
#[derive(Debug, Clone)]
pub struct StageEvaluatorInput<'a> {
    pub sample_id: &'a str,
    pub stage_id: &'a str,
    pub recognized_status: &'a str,
    pub expected_status: Option<&'a str>,
}

/// 阶段识别评估器（无状态）。
pub struct StageEvaluator;

impl StageEvaluator {
    /// 评估阶段识别状态，返回目标对象与发现的质量记录。
    ///
    /// 行为：
    /// 1. 始终构造 `StageEvaluationTarget`，`evidence_collection_present` /
    ///    `understanding_present` / `qa_history_present` 固定为 `false`，
    ///    `view_graph_types` 为空（本 evaluator 不感知这些维度）。
    /// 2. 若 `expected_status` 为 `Some(非空)` 且与 `recognized_status` 不同，
    ///    则产生一条 `StageIdentificationMismatch`（High）。
    /// 3. 若 `expected_status` 为 `None`、`Some("")` 或与 `recognized_status` 相同，
    ///    不产生 issue。
    pub fn evaluate(input: &StageEvaluatorInput<'_>) -> (StageEvaluationTarget, Vec<QualityIssue>) {
        let target = StageEvaluationTarget {
            sample_id: input.sample_id.to_string(),
            stage_id: input.stage_id.to_string(),
            recognized_status: input.recognized_status.to_string(),
            evidence_collection_present: false,
            understanding_present: false,
            view_graph_types: Vec::new(),
            qa_history_present: false,
        };

        let mut issues = Vec::new();

        if let Some(expected) = input.expected_status {
            if !expected.is_empty() && expected != input.recognized_status {
                let description = format!(
                    "阶段识别状态与期望不符：识别为 \"{}\"，期望 \"{}\"（识别缺口）",
                    input.recognized_status, expected
                );
                issues.push(make_issue(
                    input.sample_id,
                    input.stage_id,
                    ArtifactKind::Stage,
                    QualityIssueKind::StageIdentificationMismatch,
                    QualitySeverity::High,
                    None,
                    None,
                    None,
                    None,
                    None,
                    &description,
                ));
            }
        }

        (target, issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quality::models::{
        ArtifactKind, DetectionMethod, IssueStatus, QualityIssueKind, QualityIssuePolarity,
        QualitySeverity,
    };

    #[test]
    fn status_match_no_issue() {
        let input = StageEvaluatorInput {
            sample_id: "sample-001",
            stage_id: "L0",
            recognized_status: "available",
            expected_status: Some("available"),
        };
        let (target, issues) = StageEvaluator::evaluate(&input);
        assert_eq!(target.sample_id, "sample-001");
        assert_eq!(target.stage_id, "L0");
        assert_eq!(target.recognized_status, "available");
        assert!(!target.evidence_collection_present);
        assert!(!target.understanding_present);
        assert!(target.view_graph_types.is_empty());
        assert!(!target.qa_history_present);
        assert!(issues.is_empty());
    }

    #[test]
    fn status_mismatch_emits_stage_identification_mismatch() {
        let input = StageEvaluatorInput {
            sample_id: "sample-002",
            stage_id: "L1",
            recognized_status: "missing",
            expected_status: Some("available"),
        };
        let (target, issues) = StageEvaluator::evaluate(&input);
        assert_eq!(target.recognized_status, "missing");
        assert_eq!(issues.len(), 1);

        let issue = &issues[0];
        assert_eq!(issue.sample_id, "sample-002");
        assert_eq!(issue.stage_id, "L1");
        assert_eq!(issue.artifact_kind, ArtifactKind::Stage);
        assert_eq!(issue.kind, QualityIssueKind::StageIdentificationMismatch);
        assert_eq!(issue.severity, QualitySeverity::High);
        assert_eq!(issue.polarity, QualityIssuePolarity::Problem);
        assert_eq!(issue.detected_by, DetectionMethod::Automated);
        assert_eq!(issue.status, IssueStatus::Open);
        assert!(issue.issue_id.is_empty());
        assert_eq!(
            issue.description,
            "阶段识别状态与期望不符：识别为 \"missing\"，期望 \"available\"（识别缺口）"
        );
    }

    #[test]
    fn missing_expected_status_no_issue() {
        let input = StageEvaluatorInput {
            sample_id: "sample-003",
            stage_id: "L2",
            recognized_status: "empty",
            expected_status: None,
        };
        let (target, issues) = StageEvaluator::evaluate(&input);
        assert_eq!(target.recognized_status, "empty");
        assert!(issues.is_empty());
    }

    #[test]
    fn empty_expected_status_no_issue() {
        let input = StageEvaluatorInput {
            sample_id: "sample-004",
            stage_id: "L3",
            recognized_status: "naming_anomaly",
            expected_status: Some(""),
        };
        let (target, issues) = StageEvaluator::evaluate(&input);
        assert_eq!(target.recognized_status, "naming_anomaly");
        assert!(issues.is_empty());
    }
}
