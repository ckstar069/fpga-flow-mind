//! Evidence 质量评估器（Phase 7 Batch B）。
//!
//! 对单个阶段的 `EvidenceCollection` 进行形式化质量评估，输出
//! `EvidenceQualityReport` 与 evidence 相关 `QualityIssue`。
//!
//! 使用既有类型：
//! - `EvidenceCollection`、`EvidenceItem`（`crate::evidence::models`）
//! - `EvidenceQualityReport`、`UncoveredFile`、`QualityIssue`、`QualityIssueKind`、
//!   `QualitySeverity`（`crate::quality::models`）
//! - `make_issue`、`is_noisy`、`is_label_sane`（`crate::quality::issue_builder`）

use std::collections::HashSet;

use crate::evidence::models::EvidenceCollection;
use crate::quality::issue_builder::{is_label_sane, is_noisy, make_issue};
use crate::quality::models::{
    ArtifactKind, EvidenceQualityReport, QualityIssue, QualityIssueKind, QualitySeverity, UncoveredFile,
};

/// Evidence 评估输入。
#[derive(Debug, Clone)]
pub struct EvidenceEvaluatorInput<'a> {
    pub sample_id: &'a str,
    pub stage_id: &'a str,
    pub collection: &'a EvidenceCollection,
    /// 可选的期望源文件路径列表。若提供，则用于计算文件覆盖率与未覆盖文件；
    /// 若未提供，则使用 fallback 语义，不伪造未覆盖文件。
    pub expected_source_paths: Option<&'a [String]>,
}

/// Evidence 质量评估器（无状态）。
pub struct EvidenceEvaluator;

impl EvidenceEvaluator {
    /// 评估 evidence 质量，返回报告与发现的质量记录。
    ///
    /// 行为详见模块文档与实现注释。
    pub fn evaluate(
        input: &EvidenceEvaluatorInput<'_>,
    ) -> (EvidenceQualityReport, Vec<QualityIssue>) {
        let mut issues = Vec::new();
        let mut uncovered_files = Vec::new();

        let total_items = input.collection.evidence_items.len() as u32;

        // ── 空 collection 处理 ──────────────────────────────────────────
        if input.collection.evidence_items.is_empty() {
            issues.push(make_issue(
                input.sample_id,
                input.stage_id,
                ArtifactKind::Evidence,
                QualityIssueKind::MissingEvidence,
                QualitySeverity::Medium,
                None,
                None,
                None,
                None,
                None,
                "该阶段未收集到任何 evidence（coverage 缺口）",
            ));

            let report = EvidenceQualityReport {
                sample_id: input.sample_id.to_string(),
                stage_id: input.stage_id.to_string(),
                file_coverage_ratio: 0.0,
                line_range_accuracy: 0.0,
                label_sanity_ratio: 1.0,
                uncovered_files,
                issue_refs: Vec::new(),
            };
            return (report, issues);
        }

        // ── 收集已覆盖的 source_path 集合 ──────────────────────────────
        let collected_paths: HashSet<&str> = input
            .collection
            .evidence_items
            .iter()
            .map(|item| item.source_path.as_str())
            .collect();

        // ── file_coverage_ratio 与 uncovered_files ─────────────────────
        let file_coverage_ratio = if let Some(expected) = input.expected_source_paths {
            let expected_set: HashSet<&str> = expected.iter().map(|s| s.as_str()).collect();
            let covered: HashSet<&str> = collected_paths
                .intersection(&expected_set)
                .copied()
                .collect();

            // 未覆盖文件
            for path in expected_set.difference(&collected_paths) {
                uncovered_files.push(UncoveredFile {
                    source_path: path.to_string(),
                    reason: "未收集到 evidence（coverage 缺口）".to_string(),
                });
                issues.push(make_issue(
                    input.sample_id,
                    input.stage_id,
                    ArtifactKind::Evidence,
                    QualityIssueKind::MissingEvidence,
                    QualitySeverity::Medium,
                    None,
                    None,
                    None,
                    Some(path),
                    None,
                    &format!("未收集到 evidence：{}（coverage 缺口）", path),
                ));
            }

            if expected_set.is_empty() {
                1.0
            } else {
                covered.len() as f32 / expected_set.len() as f32
            }
        } else {
            // fallback：有 evidence 则 1.0，无则 0.0（空 collection 已在上方处理）
            1.0
        };

        // ── 逐 item 检查 ──────────────────────────────────────────────
        let mut valid_line_range_count = 0u32;
        let mut sane_label_count = 0u32;

        for item in &input.collection.evidence_items {
            // line_range_accuracy
            if item.line_range.start >= 1 && item.line_range.end >= item.line_range.start {
                valid_line_range_count += 1;
            }

            // label_sanity_ratio
            if is_label_sane(item) {
                sane_label_count += 1;
            } else {
                issues.push(make_issue(
                    input.sample_id,
                    input.stage_id,
                    ArtifactKind::Evidence,
                    QualityIssueKind::WrongSourceKind,
                    QualitySeverity::Medium,
                    Some(&item.evidence_id),
                    None,
                    None,
                    Some(&item.source_path),
                    Some(item.line_range),
                    &format!(
                        "evidence {} 的 source_kind/language 标注与实际不符",
                        item.evidence_id
                    ),
                ));
            }

            // noisy evidence
            if is_noisy(&item.summary) {
                issues.push(make_issue(
                    input.sample_id,
                    input.stage_id,
                    ArtifactKind::Evidence,
                    QualityIssueKind::NoisyEvidence,
                    QualitySeverity::Low,
                    Some(&item.evidence_id),
                    None,
                    None,
                    Some(&item.source_path),
                    Some(item.line_range),
                    &format!("evidence {} 的 summary 包含噪声标记", item.evidence_id),
                ));
            }
        }

        let line_range_accuracy = if total_items > 0 {
            valid_line_range_count as f32 / total_items as f32
        } else {
            0.0
        };

        let label_sanity_ratio = if total_items > 0 {
            sane_label_count as f32 / total_items as f32
        } else {
            1.0
        };

        let report = EvidenceQualityReport {
            sample_id: input.sample_id.to_string(),
            stage_id: input.stage_id.to_string(),
            file_coverage_ratio,
            line_range_accuracy,
            label_sanity_ratio,
            uncovered_files,
            issue_refs: Vec::new(),
        };

        (report, issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::models::{EvidenceItem, EvidenceStats, EvidenceStrength, LineRange};
    use crate::models::enums::{Language, SourceKind};
    use crate::quality::models::{
        DetectionMethod, IssueStatus, QualityIssueKind, QualityIssuePolarity,
        QualitySeverity,
    };
    use std::collections::HashMap;

    fn empty_collection(stage_id: &str) -> EvidenceCollection {
        EvidenceCollection {
            stage_id: stage_id.to_string(),
            evidence_items: vec![],
            index_by_path: HashMap::new(),
            index_by_kind: HashMap::new(),
            index_by_symbol: HashMap::new(),
            warnings: vec![],
            stats: EvidenceStats {
                files_processed: 0,
                files_skipped: 0,
                total_items: 0,
                items_by_kind: HashMap::new(),
                items_by_strength: HashMap::new(),
            },
            version: "1.0.0".to_string(),
        }
    }

    fn make_item(
        evidence_id: &str,
        source_path: &str,
        language: Language,
        source_kind: SourceKind,
        line_range: LineRange,
        summary: &str,
    ) -> EvidenceItem {
        EvidenceItem {
            evidence_id: evidence_id.to_string(),
            source_path: source_path.to_string(),
            language,
            source_kind,
            line_range,
            symbol: None,
            summary: summary.to_string(),
            strength: EvidenceStrength::Direct,
        }
    }

    #[test]
    fn empty_evidence_emits_missing_evidence() {
        let collection = empty_collection("L0");
        let input = EvidenceEvaluatorInput {
            sample_id: "sample-001",
            stage_id: "L0",
            collection: &collection,
            expected_source_paths: None,
        };
        let (report, issues) = EvidenceEvaluator::evaluate(&input);
        assert_eq!(report.file_coverage_ratio, 0.0);
        assert_eq!(report.line_range_accuracy, 0.0);
        assert_eq!(report.label_sanity_ratio, 1.0);
        assert!(report.uncovered_files.is_empty());
        assert_eq!(issues.len(), 1);

        let issue = &issues[0];
        assert_eq!(issue.kind, QualityIssueKind::MissingEvidence);
        assert_eq!(issue.severity, QualitySeverity::Medium);
        assert_eq!(issue.polarity, QualityIssuePolarity::Problem);
        assert_eq!(issue.detected_by, DetectionMethod::Automated);
        assert_eq!(issue.status, IssueStatus::Open);
        assert!(issue.source_path.is_none());
        assert!(issue.line_range.is_none());
        assert_eq!(issue.description, "该阶段未收集到任何 evidence（coverage 缺口）");
    }

    #[test]
    fn invalid_line_range_affects_accuracy() {
        let collection = EvidenceCollection {
            stage_id: "L0".to_string(),
            evidence_items: vec![
                make_item(
                    "EV-L0-000001",
                    "/tmp/a.py",
                    Language::Python,
                    SourceKind::PythonStage,
                    LineRange { start: 1, end: 5 },
                    "valid range",
                ),
                make_item(
                    "EV-L0-000002",
                    "/tmp/b.py",
                    Language::Python,
                    SourceKind::PythonStage,
                    LineRange { start: 5, end: 3 }, // invalid: end < start
                    "invalid range",
                ),
            ],
            index_by_path: HashMap::new(),
            index_by_kind: HashMap::new(),
            index_by_symbol: HashMap::new(),
            warnings: vec![],
            stats: EvidenceStats {
                files_processed: 2,
                files_skipped: 0,
                total_items: 2,
                items_by_kind: HashMap::new(),
                items_by_strength: HashMap::new(),
            },
            version: "1.0.0".to_string(),
        };
        let input = EvidenceEvaluatorInput {
            sample_id: "sample-002",
            stage_id: "L0",
            collection: &collection,
            expected_source_paths: None,
        };
        let (report, issues) = EvidenceEvaluator::evaluate(&input);
        assert_eq!(report.file_coverage_ratio, 1.0); // fallback, non-empty
        assert_eq!(report.line_range_accuracy, 0.5); // 1 valid out of 2
        assert_eq!(report.label_sanity_ratio, 1.0);
        assert!(issues.is_empty()); // no label/noisy issues
    }

    #[test]
    fn wrong_source_kind_emits_issue() {
        let collection = EvidenceCollection {
            stage_id: "L0".to_string(),
            evidence_items: vec![make_item(
                "EV-L0-000001",
                "/tmp/a.py",
                Language::Python,
                SourceKind::Rtl, // wrong: Python should be PythonStage
                LineRange { start: 1, end: 5 },
                "wrong kind",
            )],
            index_by_path: HashMap::new(),
            index_by_kind: HashMap::new(),
            index_by_symbol: HashMap::new(),
            warnings: vec![],
            stats: EvidenceStats {
                files_processed: 1,
                files_skipped: 0,
                total_items: 1,
                items_by_kind: HashMap::new(),
                items_by_strength: HashMap::new(),
            },
            version: "1.0.0".to_string(),
        };
        let input = EvidenceEvaluatorInput {
            sample_id: "sample-003",
            stage_id: "L0",
            collection: &collection,
            expected_source_paths: None,
        };
        let (report, issues) = EvidenceEvaluator::evaluate(&input);
        assert_eq!(report.label_sanity_ratio, 0.0);
        assert_eq!(issues.len(), 1);

        let issue = &issues[0];
        assert_eq!(issue.kind, QualityIssueKind::WrongSourceKind);
        assert_eq!(issue.severity, QualitySeverity::Medium);
        assert_eq!(issue.evidence_id, Some("EV-L0-000001".to_string()));
        assert_eq!(issue.source_path, Some("/tmp/a.py".to_string()));
        assert_eq!(issue.line_range, Some(LineRange { start: 1, end: 5 }));
    }

    #[test]
    fn noisy_evidence_emits_issue() {
        let collection = EvidenceCollection {
            stage_id: "L0".to_string(),
            evidence_items: vec![make_item(
                "EV-L0-000001",
                "/tmp/a.py",
                Language::Python,
                SourceKind::PythonStage,
                LineRange { start: 1, end: 5 },
                "TODO: fix this later", // noisy
            )],
            index_by_path: HashMap::new(),
            index_by_kind: HashMap::new(),
            index_by_symbol: HashMap::new(),
            warnings: vec![],
            stats: EvidenceStats {
                files_processed: 1,
                files_skipped: 0,
                total_items: 1,
                items_by_kind: HashMap::new(),
                items_by_strength: HashMap::new(),
            },
            version: "1.0.0".to_string(),
        };
        let input = EvidenceEvaluatorInput {
            sample_id: "sample-004",
            stage_id: "L0",
            collection: &collection,
            expected_source_paths: None,
        };
        let (_report, issues) = EvidenceEvaluator::evaluate(&input);
        assert_eq!(issues.len(), 1);

        let issue = &issues[0];
        assert_eq!(issue.kind, QualityIssueKind::NoisyEvidence);
        assert_eq!(issue.severity, QualitySeverity::Low);
        assert_eq!(issue.evidence_id, Some("EV-L0-000001".to_string()));
        assert_eq!(issue.source_path, Some("/tmp/a.py".to_string()));
        assert_eq!(issue.line_range, Some(LineRange { start: 1, end: 5 }));
    }

    #[test]
    fn expected_source_paths_uncovered_file_emits_missing_evidence() {
        let collection = EvidenceCollection {
            stage_id: "L0".to_string(),
            evidence_items: vec![make_item(
                "EV-L0-000001",
                "/tmp/a.py",
                Language::Python,
                SourceKind::PythonStage,
                LineRange { start: 1, end: 5 },
                "ok",
            )],
            index_by_path: HashMap::new(),
            index_by_kind: HashMap::new(),
            index_by_symbol: HashMap::new(),
            warnings: vec![],
            stats: EvidenceStats {
                files_processed: 1,
                files_skipped: 0,
                total_items: 1,
                items_by_kind: HashMap::new(),
                items_by_strength: HashMap::new(),
            },
            version: "1.0.0".to_string(),
        };
        let expected = vec!["/tmp/a.py".to_string(), "/tmp/b.py".to_string()];
        let input = EvidenceEvaluatorInput {
            sample_id: "sample-005",
            stage_id: "L0",
            collection: &collection,
            expected_source_paths: Some(&expected),
        };
        let (report, issues) = EvidenceEvaluator::evaluate(&input);
        assert_eq!(report.file_coverage_ratio, 0.5); // 1 covered / 2 expected
        assert_eq!(report.uncovered_files.len(), 1);
        assert_eq!(report.uncovered_files[0].source_path, "/tmp/b.py");
        assert_eq!(
            report.uncovered_files[0].reason,
            "未收集到 evidence（coverage 缺口）"
        );

        // 1 MissingEvidence for uncovered file
        assert_eq!(issues.len(), 1);
        let issue = &issues[0];
        assert_eq!(issue.kind, QualityIssueKind::MissingEvidence);
        assert_eq!(issue.source_path, Some("/tmp/b.py".to_string()));
        assert!(issue.line_range.is_none());
    }

    #[test]
    fn no_expected_files_uses_fallback_without_fake_uncovered_files() {
        let collection = EvidenceCollection {
            stage_id: "L0".to_string(),
            evidence_items: vec![make_item(
                "EV-L0-000001",
                "/tmp/a.py",
                Language::Python,
                SourceKind::PythonStage,
                LineRange { start: 1, end: 5 },
                "ok",
            )],
            index_by_path: HashMap::new(),
            index_by_kind: HashMap::new(),
            index_by_symbol: HashMap::new(),
            warnings: vec![],
            stats: EvidenceStats {
                files_processed: 1,
                files_skipped: 0,
                total_items: 1,
                items_by_kind: HashMap::new(),
                items_by_strength: HashMap::new(),
            },
            version: "1.0.0".to_string(),
        };
        let input = EvidenceEvaluatorInput {
            sample_id: "sample-006",
            stage_id: "L0",
            collection: &collection,
            expected_source_paths: None,
        };
        let (report, issues) = EvidenceEvaluator::evaluate(&input);
        assert_eq!(report.file_coverage_ratio, 1.0); // fallback: non-empty -> 1.0
        assert!(report.uncovered_files.is_empty()); // no fake uncovered files
        assert!(issues.is_empty()); // no issues for a clean collection
    }
}
