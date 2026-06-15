//! 确定性质量报告生成器（Phase 7 Batch B，P7-T02 / P7-T05）。
//!
//! `QualityReporter` 只读消费既有内存产物（`EvidenceCollection` /
//! `ImplementationUnderstanding` / `ViewGraph[]` / `GroundedAnswer`），并调用
//! Batch B 形式化 evaluator 模块产出确定性 `QualityReport`。
//!
//! 严格边界：
//! - 不读取目标项目文件、不扫描 workspace；
//! - 不调用 LLM / 不读取 api_key / 不调用 OpenAI / Anthropic；
//! - 不调用 Tauri command、不做 UI；
//! - 不做任何文件系统写入；
//! - 不取系统时间——`generated_at` 由调用方传入的确定性标记填充；
//! - report_id / issue_id 确定性生成（无 random / uuid / Date）。
//!
//! 输出只表达"工具理解质量"与"不确定性/缺口"，不做对错裁决。

use std::collections::HashSet;

use crate::evidence::models::EvidenceCollection;
use crate::trace::models::GroundedAnswer;
use crate::understanding::models::ImplementationUnderstanding;
use crate::views::models::ViewGraph;

use super::evidence_evaluator::{EvidenceEvaluator, EvidenceEvaluatorInput};
use super::issue_builder::sanitize_scope;
use super::models::{
    EvidenceQualityReport, IssueStatus, MetricSnapshot, QaQualityReport, QualityAcceptanceStatus,
    QualityIssue, QualityIssueKind, QualityIssuePolarity, QualityReport, QualityRunSummary,
    UnderstandingQualityReport, ViewQualityReport,
};
use super::qa_evaluator::{QaEvaluator, QaEvaluatorInput};
use super::stage_evaluator::{StageEvaluator, StageEvaluatorInput};
use super::understanding_evaluator::{UnderstandingEvaluator, UnderstandingEvaluatorInput};
use super::view_evaluator::{ViewEvaluator, ViewEvaluatorInput};

/// 单个阶段的质量评估输入（只读引用既有 Phase 1~6 产物）。
#[derive(Debug, Clone, Default)]
pub struct StageQualityInput<'a> {
    pub stage_id: String,
    /// 实际识别到的 StageStatus（snake_case 字符串，来自 WorkspaceProfile）。
    pub recognized_status: String,
    /// 人工期望的 StageStatus（来自 RealProjectSample，用于阶段识别比对）。
    pub expected_status: Option<String>,
    pub evidence: Option<&'a EvidenceCollection>,
    pub understanding: Option<&'a ImplementationUnderstanding>,
    pub views: Vec<&'a ViewGraph>,
    pub grounded_answer: Option<&'a GroundedAnswer>,
}

/// 一次质量评估的输入。
#[derive(Debug, Clone, Default)]
pub struct QualityReportInput<'a> {
    pub sample_id: String,
    /// 调用方传入的确定性时间标记（reporter 不取系统时间）。
    pub generated_at_marker: String,
    pub stages: Vec<StageQualityInput<'a>>,
}

/// 确定性质量报告生成器。
#[derive(Debug, Clone, Default)]
pub struct QualityReporter;

impl QualityReporter {
    pub fn new() -> Self {
        Self
    }

    /// 对输入执行确定性质量评估，返回 `QualityReport`。
    pub fn evaluate(&self, input: &QualityReportInput) -> QualityReport {
        let scope = sanitize_scope(&input.sample_id);
        let report_id = format!("QR-{}-000001", scope);

        let mut all_issues: Vec<QualityIssue> = Vec::new();
        let mut evidence_reports: Vec<EvidenceQualityReport> = Vec::new();
        let mut understanding_reports: Vec<UnderstandingQualityReport> = Vec::new();
        let mut view_reports: Vec<ViewQualityReport> = Vec::new();
        let mut qa_reports: Vec<QaQualityReport> = Vec::new();
        let mut metric_snapshots: Vec<MetricSnapshot> = Vec::new();
        let mut stage_ids: Vec<String> = Vec::new();

        // 逐阶段评估，阶段顺序由输入决定（确定性）。
        for stage in &input.stages {
            stage_ids.push(stage.stage_id.clone());
            let stage_issues = evaluate_stage(
                stage,
                &mut evidence_reports,
                &mut understanding_reports,
                &mut view_reports,
                &mut qa_reports,
                &mut metric_snapshots,
                &input.sample_id,
            );
            all_issues.extend(stage_issues);
        }

        // 为每个 stage 内 issue 按（阶段处理顺序、检查顺序）分配确定性 6 位序号。
        assign_issue_ids(&mut all_issues);

        // dimension report 的 issue_refs 由已分配 ID 按 kind 过滤（确定性）。
        for r in evidence_reports.iter_mut() {
            r.issue_refs = dimension_refs(&all_issues, &r.stage_id, &EVIDENCE_KINDS);
        }
        for r in understanding_reports.iter_mut() {
            r.issue_refs = dimension_refs(&all_issues, &r.stage_id, &UNDERSTANDING_KINDS);
        }
        for r in view_reports.iter_mut() {
            r.issue_refs = dimension_refs(&all_issues, &r.stage_id, &VIEW_KINDS);
        }
        for r in qa_reports.iter_mut() {
            r.issue_refs = dimension_refs(&all_issues, &r.stage_id, &QA_KINDS);
        }

        let summary = build_run_summary(&report_id, &input.sample_id, &all_issues, metric_snapshots);
        let total_open_problem = all_issues
            .iter()
            .filter(|i| i.polarity == QualityIssuePolarity::Problem && i.status == IssueStatus::Open)
            .count() as u32;
        // Batch B 占位门槛：无未闭环负向问题即视为达到门槛（具体阈值由后续 Batch 收敛）。
        let acceptance = if total_open_problem == 0 {
            QualityAcceptanceStatus::MeetsGate
        } else {
            QualityAcceptanceStatus::BelowGate
        };

        QualityReport {
            report_id,
            sample_id: input.sample_id.clone(),
            stage_ids,
            generated_at: input.generated_at_marker.clone(),
            evidence_reports,
            understanding_reports,
            view_reports,
            qa_reports,
            issues: all_issues,
            summary,
            acceptance,
        }
    }
}

const EVIDENCE_KINDS: [QualityIssueKind; 3] = [
    QualityIssueKind::MissingEvidence,
    QualityIssueKind::NoisyEvidence,
    QualityIssueKind::WrongSourceKind,
];
const UNDERSTANDING_KINDS: [QualityIssueKind; 3] = [
    QualityIssueKind::WeakSummary,
    QualityIssueKind::UnsupportedClaim,
    QualityIssueKind::HallucinatedClaimBlocked,
];
const VIEW_KINDS: [QualityIssueKind; 1] = [QualityIssueKind::EmptyOrUnhelpfulView];
const QA_KINDS: [QualityIssueKind; 2] = [
    QualityIssueKind::QaUnansweredWhenEvidenceExists,
    QualityIssueKind::QaAnswerWithoutValidCitation,
];

/// 对单个阶段执行全部检查，返回尚未分配 ID 的 issue 列表，并把分维度报告写入对应 Vec。
fn evaluate_stage(
    stage: &StageQualityInput,
    evidence_reports: &mut Vec<EvidenceQualityReport>,
    understanding_reports: &mut Vec<UnderstandingQualityReport>,
    view_reports: &mut Vec<ViewQualityReport>,
    qa_reports: &mut Vec<QaQualityReport>,
    metric_snapshots: &mut Vec<MetricSnapshot>,
    sample_id: &str,
) -> Vec<QualityIssue> {
    let mut issues: Vec<QualityIssue> = Vec::new();

    // —— 阶段识别（RQ-002）——
    let stage_input = StageEvaluatorInput {
        sample_id,
        stage_id: &stage.stage_id,
        recognized_status: &stage.recognized_status,
        expected_status: stage.expected_status.as_deref(),
    };
    let (_target, stage_issues) = StageEvaluator::evaluate(&stage_input);
    issues.extend(stage_issues);

    // —— evidence（RQ-003）——
    let evidence_id_set: HashSet<String> = match stage.evidence {
        Some(ec) => {
            let set: HashSet<String> = ec.evidence_items.iter().map(|i| i.evidence_id.clone()).collect();
            let ev_input = EvidenceEvaluatorInput {
                sample_id,
                stage_id: &stage.stage_id,
                collection: ec,
                expected_source_paths: None,
            };
            let (report, ev_issues) = EvidenceEvaluator::evaluate(&ev_input);
            push_metric(metric_snapshots, "evidence_file_coverage", &stage.stage_id, report.file_coverage_ratio);
            push_metric(metric_snapshots, "evidence_line_range_accuracy", &stage.stage_id, report.line_range_accuracy);
            push_metric(metric_snapshots, "evidence_label_sanity", &stage.stage_id, report.label_sanity_ratio);
            evidence_reports.push(report);
            issues.extend(ev_issues);
            set
        }
        None => HashSet::new(),
    };

    // —— understanding（RQ-004）——
    if let Some(iu) = stage.understanding {
        let un_input = UnderstandingEvaluatorInput {
            sample_id,
            stage_id: &stage.stage_id,
            understanding: iu,
            evidence_id_set: &evidence_id_set,
        };
        let (report, un_issues) = UnderstandingEvaluator::evaluate(&un_input);
        push_metric(metric_snapshots, "claim_existence_check", &stage.stage_id, report.claim_existence_check_ratio);
        push_metric(metric_snapshots, "uncertainty_expression", &stage.stage_id, report.uncertainty_expression_ratio);
        push_metric(metric_snapshots, "confidence_calibration", &stage.stage_id, report.confidence_calibration_ratio);
        understanding_reports.push(report);
        issues.extend(un_issues);
    }

    // —— views（RQ-005）——
    let claim_id_set: HashSet<String> = match stage.understanding {
        Some(iu) => iu.claims.iter().map(|c| c.claim_id.clone()).collect(),
        None => HashSet::new(),
    };
    for view in &stage.views {
        let v_input = ViewEvaluatorInput {
            sample_id,
            stage_id: &stage.stage_id,
            view,
            evidence_id_set: &evidence_id_set,
            claim_id_set: &claim_id_set,
        };
        let (report, v_issues) = ViewEvaluator::evaluate(&v_input);
        push_metric(metric_snapshots, "view_trace_resolvable", &stage.stage_id, report.trace_resolvable_ratio);
        view_reports.push(report);
        issues.extend(v_issues);
    }

    // —— Q&A（RQ-006）——
    if let Some(answer) = stage.grounded_answer {
        let qa_input = QaEvaluatorInput {
            sample_id,
            stage_id: &stage.stage_id,
            answer,
            evidence_id_set: &evidence_id_set,
            claim_id_set: &claim_id_set,
            question_set: None,
        };
        let (report, q_issues) = QaEvaluator::evaluate(&qa_input);
        push_metric(metric_snapshots, "qa_citation_validity", &stage.stage_id, report.citation_validity_ratio);
        qa_reports.push(report);
        issues.extend(q_issues);
    }

    // issue_id 由主流程统一确定性分配（issues 已按阶段处理顺序、检查顺序自然排列）。
    issues
}

// ─── 辅助 ────────────────────────────────────────────────────────────

fn assign_issue_ids(issues: &mut [QualityIssue]) {
    let mut counter: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    for issue in issues.iter_mut() {
        let seq = counter.entry(issue.stage_id.clone()).or_insert(0);
        *seq += 1;
        issue.issue_id = format!("QI-{}-{:06}", issue.stage_id, *seq);
    }
}

fn dimension_refs(issues: &[QualityIssue], stage_id: &str, kinds: &[QualityIssueKind]) -> Vec<String> {
    let mut refs: Vec<String> = issues
        .iter()
        .filter(|i| i.stage_id == stage_id && kinds.contains(&i.kind))
        .map(|i| i.issue_id.clone())
        .collect();
    refs.sort();
    refs
}

fn build_run_summary(
    run_id: &str,
    sample_id: &str,
    issues: &[QualityIssue],
    metric_snapshots: Vec<MetricSnapshot>,
) -> QualityRunSummary {
    use std::collections::HashMap;
    let mut issues_by_kind: HashMap<String, u32> = HashMap::new();
    let mut issues_by_severity: HashMap<String, u32> = HashMap::new();
    let mut issues_by_status: HashMap<String, u32> = HashMap::new();
    let mut total_issues = 0u32;
    let mut positive_guardrail_event_count = 0u32;

    for i in issues {
        *issues_by_kind.entry(i.kind.as_str().to_string()).or_insert(0) += 1;
        if i.polarity == QualityIssuePolarity::Problem {
            total_issues += 1;
            *issues_by_severity.entry(i.severity.as_str().to_string()).or_insert(0) += 1;
            *issues_by_status.entry(i.status.as_str().to_string()).or_insert(0) += 1;
        } else {
            positive_guardrail_event_count += 1;
        }
    }

    QualityRunSummary {
        run_id: run_id.to_string(),
        sample_ids: vec![sample_id.to_string()],
        total_issues,
        positive_guardrail_event_count,
        issues_by_kind,
        issues_by_severity,
        issues_by_status,
        metric_snapshots,
    }
}

fn push_metric(snapshots: &mut Vec<MetricSnapshot>, name: &str, stage_id: &str, value: f32) {
    snapshots.push(MetricSnapshot {
        metric_name: name.to_string(),
        stage_id: Some(stage_id.to_string()),
        value,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::models::{EvidenceCollection, EvidenceItem, EvidenceStats, EvidenceStrength, LineRange};
    use crate::models::enums::{Language, SourceKind};
    use crate::trace::models::GroundedAnswerCitation;
    use crate::understanding::models::{
        ClaimCategory, ClaimConfidence, EvidenceRef, GenerationMeta, ImplementationClaim,
        ImplementationUnderstanding, StageSummary, UnderstandingStats,
    };
    use crate::views::models::{EdgeType, NodeType, ViewEdge, ViewGraph, ViewMeta, ViewNode, ViewTraceRef, ViewType};
    use std::collections::HashMap as StdHashMap;

    fn empty_evidence(stage_id: &str) -> EvidenceCollection {
        EvidenceCollection {
            stage_id: stage_id.to_string(),
            evidence_items: vec![],
            index_by_path: StdHashMap::new(),
            index_by_kind: StdHashMap::new(),
            index_by_symbol: StdHashMap::new(),
            warnings: vec![],
            stats: EvidenceStats {
                files_processed: 0, files_skipped: 0, total_items: 0,
                items_by_kind: StdHashMap::new(), items_by_strength: StdHashMap::new(),
            },
            version: "1.0.0".to_string(),
        }
    }

    fn ev_item(id: &str, path: &str, lang: Language, kind: SourceKind, summary: &str) -> EvidenceItem {
        EvidenceItem {
            evidence_id: id.to_string(),
            source_path: path.to_string(),
            language: lang,
            source_kind: kind,
            line_range: LineRange { start: 1, end: 5 },
            symbol: None,
            summary: summary.to_string(),
            strength: EvidenceStrength::Direct,
        }
    }

    fn minimal_iu(stage_id: &str, claims: Vec<ImplementationClaim>) -> ImplementationUnderstanding {
        ImplementationUnderstanding {
            stage_id: stage_id.to_string(),
            version: "3.0.0".to_string(),
            summary: StageSummary { short: "正常摘要".to_string(), detailed: "这是一段足够长的详细摘要".to_string() },
            claims,
            module_summaries: vec![], signal_summaries: vec![], interface_summaries: vec![],
            processing_steps: vec![], unknowns: vec![], evidence_gaps: vec![],
            generation_meta: GenerationMeta {
                provider: "mock".to_string(), generated_at: "2026-06-15T00:00:00Z".to_string(),
                input_evidence_count: 1, generation_time_ms: 1, is_degraded: false,
            },
            stats: UnderstandingStats {
                total_claims: 0, claims_by_confidence: StdHashMap::new(), claims_by_category: StdHashMap::new(),
                module_count: 0, signal_count: 0, interface_count: 0, processing_step_count: 0,
                unknown_count: 0, evidence_gap_count: 0,
            },
        }
    }

    fn claim(id: &str, conf: ClaimConfidence, refs: Vec<&str>, gap: bool) -> ImplementationClaim {
        ImplementationClaim {
            claim_id: id.to_string(),
            category: ClaimCategory::ModuleStructure,
            description: "d".to_string(),
            confidence: conf,
            evidence_refs: refs
                .iter()
                .map(|r| EvidenceRef { evidence_id: r.to_string(), relevance: None })
                .collect(),
            has_evidence_gap: gap,
        }
    }

    fn base_input<'a>(sample: &str, stages: Vec<StageQualityInput<'a>>) -> QualityReportInput<'a> {
        QualityReportInput {
            sample_id: sample.to_string(),
            generated_at_marker: "2026-06-15T00:00:00Z".to_string(),
            stages,
        }
    }

    fn reporter() -> QualityReporter {
        QualityReporter::new()
    }

    fn view_node(id: &str, trace_refs: Vec<ViewTraceRef>) -> ViewNode {
        ViewNode {
            node_id: id.to_string(),
            node_type: NodeType::Module,
            label: id.to_string(),
            description: "".to_string(),
            confidence: ClaimConfidence::Confirmed,
            trace_refs,
            layout: None,
        }
    }

    fn view_edge(id: &str, source: &str, target: &str, trace_refs: Vec<ViewTraceRef>) -> ViewEdge {
        ViewEdge {
            edge_id: id.to_string(),
            edge_type: EdgeType::Contains,
            source_node_id: source.to_string(),
            target_node_id: target.to_string(),
            label: None,
            description: "".to_string(),
            confidence: ClaimConfidence::Confirmed,
            trace_refs,
        }
    }

    fn view_trace_evidence(evidence_id: &str) -> ViewTraceRef {
        ViewTraceRef {
            claim_id: None,
            evidence_id: Some(evidence_id.to_string()),
            confidence: ClaimConfidence::Confirmed,
            relevance: None,
        }
    }

    fn view_trace_claim(claim_id: &str) -> ViewTraceRef {
        ViewTraceRef {
            claim_id: Some(claim_id.to_string()),
            evidence_id: None,
            confidence: ClaimConfidence::Confirmed,
            relevance: None,
        }
    }

    fn structure_view(stage_id: &str, nodes: Vec<ViewNode>, edges: Vec<ViewEdge>) -> ViewGraph {
        ViewGraph {
            view_type: ViewType::Structure,
            stage_id: stage_id.to_string(),
            nodes,
            edges,
            meta: ViewMeta {
                stage_id: stage_id.to_string(),
                view_type: ViewType::Structure,
                source_provider: "mock".to_string(),
                is_degraded_source: false,
                generated_at: "2026-06-15T00:00:00Z".to_string(),
                empty_reason: None,
            },
        }
    }

    #[test]
    fn empty_input_does_not_crash() {
        let report = reporter().evaluate(&base_input("sample-empty", vec![]));
        assert_eq!(report.issues.len(), 0);
        assert_eq!(report.summary.total_issues, 0);
        assert_eq!(report.acceptance, QualityAcceptanceStatus::MeetsGate);
        assert_eq!(report.report_id, "QR-sample-empty-000001");
        assert!(report.stage_ids.is_empty());
    }

    #[test]
    fn stage_with_no_evidence_emits_coverage_gap() {
        let stage = StageQualityInput {
            stage_id: "L0".to_string(),
            recognized_status: "available".to_string(),
            expected_status: None,
            evidence: Some(&empty_evidence("L0")),
            understanding: None,
            views: vec![],
            grounded_answer: None,
        };
        let report = reporter().evaluate(&base_input("sample-gap", vec![stage]));
        assert!(report.issues.iter().any(|i| i.kind == QualityIssueKind::MissingEvidence));
        assert!(report.issues.iter().all(|i| i.is_traceable()));
        assert_eq!(report.evidence_reports[0].file_coverage_ratio, 0.0);
    }

    #[test]
    fn hallucinated_claim_emits_unsupported_issue() {
        let iu = minimal_iu("L0", vec![claim("CL-L0-000001", ClaimConfidence::Supported, vec!["EV-L0-999999"], false)]);
        let ec = empty_evidence("L0");
        let stage = StageQualityInput {
            stage_id: "L0".to_string(),
            recognized_status: "available".to_string(),
            expected_status: None,
            evidence: Some(&ec),
            understanding: Some(&iu),
            views: vec![],
            grounded_answer: None,
        };
        let report = reporter().evaluate(&base_input("sample-hall", vec![stage]));
        assert!(report.issues.iter().any(|i| {
            i.kind == QualityIssueKind::UnsupportedClaim && i.polarity == QualityIssuePolarity::Problem
        }));
        assert_eq!(report.acceptance, QualityAcceptanceStatus::BelowGate);
    }

    #[test]
    fn unknown_claim_with_gap_is_guardrail_not_error() {
        let iu = minimal_iu("L0", vec![claim("CL-L0-000001", ClaimConfidence::Unknown, vec![], true)]);
        let stage = StageQualityInput {
            stage_id: "L0".to_string(),
            recognized_status: "available".to_string(),
            expected_status: None,
            evidence: None,
            understanding: Some(&iu),
            views: vec![],
            grounded_answer: None,
        };
        let report = reporter().evaluate(&base_input("sample-grd", vec![stage]));
        assert_eq!(report.summary.total_issues, 0, "诚实声明 gap 不应进入负向 backlog");
        assert!(report.summary.positive_guardrail_event_count >= 1);
        let grd = report
            .issues
            .iter()
            .find(|i| i.kind == QualityIssueKind::HallucinatedClaimBlocked)
            .expect("应存在 hallucinated_claim_blocked 正向记录");
        assert_eq!(grd.polarity, QualityIssuePolarity::PositiveGuardrail);
        assert_eq!(report.acceptance, QualityAcceptanceStatus::MeetsGate);
    }

    #[test]
    fn stage_identification_mismatch_emits_issue() {
        let stage = StageQualityInput {
            stage_id: "rtl_final".to_string(),
            recognized_status: "missing".to_string(),
            expected_status: Some("naming_anomaly".to_string()),
            evidence: None,
            understanding: None,
            views: vec![],
            grounded_answer: None,
        };
        let report = reporter().evaluate(&base_input("sample-stage", vec![stage]));
        let m = report
            .issues
            .iter()
            .find(|i| i.kind == QualityIssueKind::StageIdentificationMismatch)
            .expect("阶段识别不一致应生成 stage_identification_mismatch");
        assert_eq!(m.polarity, QualityIssuePolarity::Problem);
        assert!(m.is_traceable());
    }

    #[test]
    fn deterministic_output_stable_ids_and_order() {
        let ec = EvidenceCollection {
            stage_id: "L0".to_string(),
            evidence_items: vec![
                ev_item("EV-L0-000001", "/tmp/p/a.py", Language::Python, SourceKind::PythonStage, "def foo(): pass"),
                ev_item("EV-L0-000002", "/tmp/p/b.py", Language::Python, SourceKind::PythonStage, "TODO: implement"),
            ],
            index_by_path: StdHashMap::new(), index_by_kind: StdHashMap::new(), index_by_symbol: StdHashMap::new(),
            warnings: vec![],
            stats: EvidenceStats {
                files_processed: 2, files_skipped: 0, total_items: 2,
                items_by_kind: StdHashMap::new(), items_by_strength: StdHashMap::new(),
            },
            version: "1.0.0".to_string(),
        };
        let iu = minimal_iu("L0", vec![claim("CL-L0-000001", ClaimConfidence::Confirmed, vec!["EV-L0-000001"], false)]);
        let build = || StageQualityInput {
            stage_id: "L0".to_string(),
            recognized_status: "available".to_string(),
            expected_status: Some("available".to_string()),
            evidence: Some(&ec),
            understanding: Some(&iu),
            views: vec![],
            grounded_answer: None,
        };
        let r1 = reporter().evaluate(&base_input("sample-det", vec![build()]));
        let r2 = reporter().evaluate(&base_input("sample-det", vec![build()]));
        assert_eq!(r1.report_id, r2.report_id);
        assert_eq!(r1.issues.len(), r2.issues.len());
        assert_eq!(
            r1.issues.iter().map(|i| i.issue_id.clone()).collect::<Vec<_>>(),
            r2.issues.iter().map(|i| i.issue_id.clone()).collect::<Vec<_>>()
        );
        assert!(r1
            .issues
            .iter()
            .all(|i| i.issue_id.starts_with("QI-L0-") && i.issue_id.len() == "QI-L0-".len() + 6));
        let mut d1: Vec<&String> = r1.issues.iter().map(|i| &i.description).collect();
        let mut d2: Vec<&String> = r2.issues.iter().map(|i| &i.description).collect();
        d1.sort();
        d2.sort();
        assert_eq!(d1, d2);
        assert!(r1.issues.iter().any(|i| i.kind == QualityIssueKind::NoisyEvidence));
    }

    #[test]
    fn issues_always_traceable() {
        let ec = EvidenceCollection {
            stage_id: "L0".to_string(),
            evidence_items: vec![ev_item("EV-L0-000001", "/tmp/p/a.py", Language::Python, SourceKind::Rtl, "ok")],
            index_by_path: StdHashMap::new(), index_by_kind: StdHashMap::new(), index_by_symbol: StdHashMap::new(),
            warnings: vec![],
            stats: EvidenceStats {
                files_processed: 1, files_skipped: 0, total_items: 1,
                items_by_kind: StdHashMap::new(), items_by_strength: StdHashMap::new(),
            },
            version: "1.0.0".to_string(),
        };
        let iu = minimal_iu("L0", vec![
            claim("CL-L0-000001", ClaimConfidence::Unknown, vec![], true),
            claim("CL-L0-000002", ClaimConfidence::Supported, vec!["EV-L0-000001"], false),
        ]);
        let stage = StageQualityInput {
            stage_id: "L0".to_string(),
            recognized_status: "naming_anomaly".to_string(),
            expected_status: Some("available".to_string()),
            evidence: Some(&ec),
            understanding: Some(&iu),
            views: vec![],
            grounded_answer: None,
        };
        let report = reporter().evaluate(&base_input("sample-trace", vec![stage]));
        assert!(!report.issues.is_empty());
        for i in &report.issues {
            assert!(i.is_traceable(), "issue 不可追溯: {:?} {:?}", i.kind, i.issue_id);
        }
    }

    #[test]
    fn no_audit_verdict_terms_in_descriptions() {
        let ec = empty_evidence("L0");
        let iu = minimal_iu("L0", vec![claim("CL-L0-000001", ClaimConfidence::Unknown, vec![], true)]);
        let stage = StageQualityInput {
            stage_id: "L0".to_string(),
            recognized_status: "missing".to_string(),
            expected_status: Some("available".to_string()),
            evidence: Some(&ec),
            understanding: Some(&iu),
            views: vec![],
            grounded_answer: None,
        };
        let report = reporter().evaluate(&base_input("sample-terms", vec![stage]));
        assert!(!report.issues.is_empty());
        for i in &report.issues {
            assert!(!i.description.contains("PASS"), "含 PASS: {}", i.description);
            assert!(!i.description.contains("HOLD"), "含 HOLD: {}", i.description);
            assert!(!i.description.contains("正确"), "含 正确: {}", i.description);
            assert!(!i.description.contains("错误"), "含 错误: {}", i.description);
        }
    }

    #[test]
    fn view_empty_emits_issue() {
        let empty_view = ViewGraph {
            view_type: ViewType::Structure,
            stage_id: "L0".to_string(),
            nodes: vec![],
            edges: vec![],
            meta: ViewMeta {
                stage_id: "L0".to_string(),
                view_type: ViewType::Structure,
                source_provider: "mock".to_string(),
                is_degraded_source: false,
                generated_at: "2026-06-15T00:00:00Z".to_string(),
                empty_reason: Some("no claims".to_string()),
            },
        };
        let stage = StageQualityInput {
            stage_id: "L0".to_string(),
            recognized_status: "available".to_string(),
            expected_status: None,
            evidence: None,
            understanding: None,
            views: vec![&empty_view],
            grounded_answer: None,
        };
        let report = reporter().evaluate(&base_input("sample-view", vec![stage]));
        assert!(report.issues.iter().any(|i| i.kind == QualityIssueKind::EmptyOrUnhelpfulView));
    }

    #[test]
    fn qa_invalid_citation_emits_issue() {
        let ec = EvidenceCollection {
            stage_id: "L0".to_string(),
            evidence_items: vec![ev_item("EV-L0-000001", "/p/a.py", Language::Python, SourceKind::PythonStage, "ok")],
            index_by_path: StdHashMap::new(), index_by_kind: StdHashMap::new(), index_by_symbol: StdHashMap::new(),
            warnings: vec![],
            stats: EvidenceStats {
                files_processed: 1, files_skipped: 0, total_items: 1,
                items_by_kind: StdHashMap::new(), items_by_strength: StdHashMap::new(),
            },
            version: "1.0.0".to_string(),
        };
        let answer = GroundedAnswer {
            answer_id: "A-1".to_string(),
            generated_at: "2026-06-15T00:00:00Z".to_string(),
            text: "某回答".to_string(),
            claims: vec![],
            citations: vec![GroundedAnswerCitation {
                index: 0,
                evidence_id: Some("EV-L0-999999".to_string()),
                claim_id: None,
                source_location: None,
                excerpt_summary: "...".to_string(),
            }],
            confidence: ClaimConfidence::Supported,
            warnings: vec![],
            provider: "mock".to_string(),
            is_degraded: false,
        };
        let stage = StageQualityInput {
            stage_id: "L0".to_string(),
            recognized_status: "available".to_string(),
            expected_status: None,
            evidence: Some(&ec),
            understanding: None,
            views: vec![],
            grounded_answer: Some(&answer),
        };
        let report = reporter().evaluate(&base_input("sample-qa", vec![stage]));
        assert!(report.issues.iter().any(|i| i.kind == QualityIssueKind::QaAnswerWithoutValidCitation));
        assert!(report.qa_reports[0].citation_validity_ratio < 1.0);
    }

    #[test]
    fn connected_view_with_missing_node_trace_is_not_fully_resolvable() {
        // 节点 N1 无 trace_refs，节点 N2 有有效 trace_ref，比率应为 0.5 并生成 issue。
        let n1 = view_node("N1", vec![]);
        let n2 = view_node("N2", vec![view_trace_evidence("EV-L0-000001")]);
        let e1 = view_edge("E1", "N1", "N2", vec![view_trace_evidence("EV-L0-000001")]);
        let v = structure_view("L0", vec![n1, n2], vec![e1]);
        let ec = EvidenceCollection {
            stage_id: "L0".to_string(),
            evidence_items: vec![ev_item("EV-L0-000001", "/p/a.py", Language::Python, SourceKind::PythonStage, "ok")],
            index_by_path: StdHashMap::new(), index_by_kind: StdHashMap::new(), index_by_symbol: StdHashMap::new(),
            warnings: vec![],
            stats: EvidenceStats {
                files_processed: 1, files_skipped: 0, total_items: 1,
                items_by_kind: StdHashMap::new(), items_by_strength: StdHashMap::new(),
            },
            version: "1.0.0".to_string(),
        };
        let stage = StageQualityInput {
            stage_id: "L0".to_string(),
            recognized_status: "available".to_string(),
            expected_status: None,
            evidence: Some(&ec),
            understanding: None,
            views: vec![&v],
            grounded_answer: None,
        };
        let report = reporter().evaluate(&base_input("sample-view-missing-node-trace", vec![stage]));
        let view_report = report.view_reports.iter().find(|r| r.stage_id == "L0").expect("应有 view report");
        assert!(
            (view_report.trace_resolvable_ratio - 0.666_666_7).abs() < 1e-5,
            "期望 2/3 可解析，实际 {}",
            view_report.trace_resolvable_ratio
        );
        let node_issue = report
            .issues
            .iter()
            .find(|i| i.kind == QualityIssueKind::EmptyOrUnhelpfulView && i.description.contains("node_id=N1"));
        assert!(node_issue.is_some(), "应生成针对 N1 节点缺 trace_refs 的 issue");
    }

    #[test]
    fn edge_without_trace_refs_emits_issue() {
        let n1 = view_node("N1", vec![view_trace_evidence("EV-L0-000001")]);
        let n2 = view_node("N2", vec![view_trace_evidence("EV-L0-000001")]);
        // 边 E1 缺少 trace_refs
        let e1 = view_edge("E1", "N1", "N2", vec![]);
        let v = structure_view("L0", vec![n1, n2], vec![e1]);
        let ec = EvidenceCollection {
            stage_id: "L0".to_string(),
            evidence_items: vec![ev_item("EV-L0-000001", "/p/a.py", Language::Python, SourceKind::PythonStage, "ok")],
            index_by_path: StdHashMap::new(), index_by_kind: StdHashMap::new(), index_by_symbol: StdHashMap::new(),
            warnings: vec![],
            stats: EvidenceStats {
                files_processed: 1, files_skipped: 0, total_items: 1,
                items_by_kind: StdHashMap::new(), items_by_strength: StdHashMap::new(),
            },
            version: "1.0.0".to_string(),
        };
        let stage = StageQualityInput {
            stage_id: "L0".to_string(),
            recognized_status: "available".to_string(),
            expected_status: None,
            evidence: Some(&ec),
            understanding: None,
            views: vec![&v],
            grounded_answer: None,
        };
        let report = reporter().evaluate(&base_input("sample-view-edge-no-trace", vec![stage]));
        let view_report = report.view_reports.iter().find(|r| r.stage_id == "L0").expect("应有 view report");
        assert!(
            (view_report.trace_resolvable_ratio - 0.666_666_7).abs() < 1e-5,
            "期望 2/3 可解析，实际 {}",
            view_report.trace_resolvable_ratio
        );
        let edge_issue = report
            .issues
            .iter()
            .find(|i| i.kind == QualityIssueKind::EmptyOrUnhelpfulView && i.description.contains("edge=E1"));
        assert!(edge_issue.is_some(), "应生成针对 E1 边缺 trace_refs 的 issue");
    }

    #[test]
    fn view_with_valid_node_and_edge_traces_reports_ratio_1() {
        let n1 = view_node("N1", vec![view_trace_evidence("EV-L0-000001")]);
        let n2 = view_node("N2", vec![view_trace_claim("CL-L0-000001")]);
        let e1 = view_edge("E1", "N1", "N2", vec![view_trace_evidence("EV-L0-000001")]);
        let v = structure_view("L0", vec![n1, n2], vec![e1]);
        let ec = EvidenceCollection {
            stage_id: "L0".to_string(),
            evidence_items: vec![ev_item("EV-L0-000001", "/p/a.py", Language::Python, SourceKind::PythonStage, "ok")],
            index_by_path: StdHashMap::new(), index_by_kind: StdHashMap::new(), index_by_symbol: StdHashMap::new(),
            warnings: vec![],
            stats: EvidenceStats {
                files_processed: 1, files_skipped: 0, total_items: 1,
                items_by_kind: StdHashMap::new(), items_by_strength: StdHashMap::new(),
            },
            version: "1.0.0".to_string(),
        };
        let iu = minimal_iu("L0", vec![claim("CL-L0-000001", ClaimConfidence::Confirmed, vec!["EV-L0-000001"], false)]);
        let stage = StageQualityInput {
            stage_id: "L0".to_string(),
            recognized_status: "available".to_string(),
            expected_status: None,
            evidence: Some(&ec),
            understanding: Some(&iu),
            views: vec![&v],
            grounded_answer: None,
        };
        let report = reporter().evaluate(&base_input("sample-view-all-trace", vec![stage]));
        let view_report = report.view_reports.iter().find(|r| r.stage_id == "L0").expect("应有 view report");
        assert!(
            (view_report.trace_resolvable_ratio - 1.0).abs() < 1e-5,
            "所有节点/边 trace 均可解析，应返回 1.0，实际 {}",
            view_report.trace_resolvable_ratio
        );
        assert!(
            !report.issues.iter().any(|i| {
                i.kind == QualityIssueKind::EmptyOrUnhelpfulView
                    && (i.description.contains("node_id=N1") || i.description.contains("edge_id=E1"))
            }),
            "不应为 N1/E1 生成 trace 缺失 issue"
        );
    }

    #[test]
    fn run_summary_uses_snake_case_issue_keys() {
        let ec = EvidenceCollection {
            stage_id: "L0".to_string(),
            evidence_items: vec![ev_item("EV-L0-000001", "/p/a.py", Language::Python, SourceKind::PythonStage, "TODO fix")],
            index_by_path: StdHashMap::new(), index_by_kind: StdHashMap::new(), index_by_symbol: StdHashMap::new(),
            warnings: vec![],
            stats: EvidenceStats {
                files_processed: 1, files_skipped: 0, total_items: 1,
                items_by_kind: StdHashMap::new(), items_by_strength: StdHashMap::new(),
            },
            version: "1.0.0".to_string(),
        };
        let iu = minimal_iu("L0", vec![
            claim("CL-L0-000001", ClaimConfidence::Unknown, vec![], true),
            claim("CL-L0-000002", ClaimConfidence::Supported, vec!["EV-L0-999999"], false),
        ]);
        let stage = StageQualityInput {
            stage_id: "L0".to_string(),
            recognized_status: "available".to_string(),
            expected_status: Some("missing".to_string()),
            evidence: Some(&ec),
            understanding: Some(&iu),
            views: vec![],
            grounded_answer: None,
        };
        let report = reporter().evaluate(&base_input("sample-summary-keys", vec![stage]));
        let keys: Vec<&String> = report.summary.issues_by_kind.keys().collect();
        for k in &keys {
            assert!(
                !k.contains("stageidentificationmismatch")
                    && !k.contains("unsupportedclaim")
                    && !k.contains("hallucinatedclaimblocked")
                    && !k.contains("noisyevidence"),
                "发现驼峰/拼接 key: {}",
                k
            );
        }
        assert!(report.summary.issues_by_kind.contains_key("stage_identification_mismatch"));
        assert!(report.summary.issues_by_kind.contains_key("unsupported_claim"));
        assert!(report.summary.issues_by_kind.contains_key("hallucinated_claim_blocked"));
        assert!(report.summary.issues_by_kind.contains_key("noisy_evidence"));
    }

    #[test]
    fn run_summary_uses_snake_case_status_keys() {
        let ec = empty_evidence("L0");
        let stage = StageQualityInput {
            stage_id: "L0".to_string(),
            recognized_status: "available".to_string(),
            expected_status: None,
            evidence: Some(&ec),
            understanding: None,
            views: vec![],
            grounded_answer: None,
        };
        let report = reporter().evaluate(&base_input("sample-status-keys", vec![stage]));
        let keys: Vec<&String> = report.summary.issues_by_status.keys().collect();
        for k in &keys {
            assert!(
                !k.contains("acceptedasknownlimitation") && !k.contains("openfixed"),
                "发现拼接 status key: {}",
                k
            );
        }
        assert!(report.summary.issues_by_status.contains_key("open"));
    }

    #[test]
    fn run_summary_uses_snake_case_severity_keys() {
        let ec = empty_evidence("L0");
        let stage = StageQualityInput {
            stage_id: "L0".to_string(),
            recognized_status: "available".to_string(),
            expected_status: None,
            evidence: Some(&ec),
            understanding: None,
            views: vec![],
            grounded_answer: None,
        };
        let report = reporter().evaluate(&base_input("sample-severity-keys", vec![stage]));
        let keys: Vec<&String> = report.summary.issues_by_severity.keys().collect();
        for k in &keys {
            assert!(!k.chars().any(|c| c.is_uppercase()), "severity key 应全小写: {}", k);
        }
        assert!(report.summary.issues_by_severity.contains_key("medium"));
    }

    // ─── Batch B integration tests ──────────────────────────────────────

    #[test]
    fn reporter_calls_formal_evaluators() {
        let ec = EvidenceCollection {
            stage_id: "L0".to_string(),
            evidence_items: vec![ev_item("EV-L0-000001", "/p/a.py", Language::Python, SourceKind::PythonStage, "ok")],
            index_by_path: StdHashMap::new(), index_by_kind: StdHashMap::new(), index_by_symbol: StdHashMap::new(),
            warnings: vec![],
            stats: EvidenceStats {
                files_processed: 1, files_skipped: 0, total_items: 1,
                items_by_kind: StdHashMap::new(), items_by_strength: StdHashMap::new(),
            },
            version: "1.0.0".to_string(),
        };
        let iu = minimal_iu("L0", vec![claim("CL-L0-000001", ClaimConfidence::Confirmed, vec!["EV-L0-000001"], false)]);
        let n1 = view_node("N1", vec![view_trace_evidence("EV-L0-000001")]);
        let n2 = view_node("N2", vec![view_trace_claim("CL-L0-000001")]);
        let e1 = view_edge("E1", "N1", "N2", vec![view_trace_evidence("EV-L0-000001")]);
        let v = structure_view("L0", vec![n1, n2], vec![e1]);
        let answer = GroundedAnswer {
            answer_id: "A-1".to_string(),
            generated_at: "2026-06-15T00:00:00Z".to_string(),
            text: "回答".to_string(),
            claims: vec![],
            citations: vec![GroundedAnswerCitation {
                index: 0,
                evidence_id: Some("EV-L0-000001".to_string()),
                claim_id: Some("CL-L0-000001".to_string()),
                source_location: None,
                excerpt_summary: "...".to_string(),
            }],
            confidence: ClaimConfidence::Confirmed,
            warnings: vec![],
            provider: "mock".to_string(),
            is_degraded: false,
        };
        let stage = StageQualityInput {
            stage_id: "L0".to_string(),
            recognized_status: "available".to_string(),
            expected_status: Some("available".to_string()),
            evidence: Some(&ec),
            understanding: Some(&iu),
            views: vec![&v],
            grounded_answer: Some(&answer),
        };
        let report = reporter().evaluate(&base_input("sample-all-dims", vec![stage]));

        assert!(!report.evidence_reports.is_empty(), "应生成 evidence 维度报告");
        assert!(!report.understanding_reports.is_empty(), "应生成 understanding 维度报告");
        assert!(!report.view_reports.is_empty(), "应生成 view 维度报告");
        assert!(!report.qa_reports.is_empty(), "应生成 qa 维度报告");
        assert!(report.issues.iter().all(|i| i.issue_id.starts_with("QI-L0-") && i.issue_id.len() == "QI-L0-".len() + 6));
        assert_eq!(report.acceptance, QualityAcceptanceStatus::MeetsGate);
    }

    #[test]
    fn deterministic_output_stable_after_evaluator_split() {
        let ec = EvidenceCollection {
            stage_id: "L0".to_string(),
            evidence_items: vec![
                ev_item("EV-L0-000001", "/p/a.py", Language::Python, SourceKind::PythonStage, "ok"),
                ev_item("EV-L0-000002", "/p/b.py", Language::Python, SourceKind::Rtl, "TODO"),
            ],
            index_by_path: StdHashMap::new(), index_by_kind: StdHashMap::new(), index_by_symbol: StdHashMap::new(),
            warnings: vec![],
            stats: EvidenceStats {
                files_processed: 2, files_skipped: 0, total_items: 2,
                items_by_kind: StdHashMap::new(), items_by_strength: StdHashMap::new(),
            },
            version: "1.0.0".to_string(),
        };
        let iu = minimal_iu("L0", vec![
            claim("CL-L0-000001", ClaimConfidence::Unknown, vec![], true),
            claim("CL-L0-000002", ClaimConfidence::Supported, vec!["EV-L0-000001"], false),
        ]);
        let stage = StageQualityInput {
            stage_id: "L0".to_string(),
            recognized_status: "available".to_string(),
            expected_status: Some("missing".to_string()),
            evidence: Some(&ec),
            understanding: Some(&iu),
            views: vec![],
            grounded_answer: None,
        };
        let build = || base_input("sample-stable", vec![stage.clone()]);
        let r1 = reporter().evaluate(&build());
        let r2 = reporter().evaluate(&build());
        assert_eq!(r1.report_id, r2.report_id);
        assert_eq!(r1.issues.len(), r2.issues.len());
        assert_eq!(
            r1.issues.iter().map(|i| i.issue_id.clone()).collect::<Vec<_>>(),
            r2.issues.iter().map(|i| i.issue_id.clone()).collect::<Vec<_>>()
        );
        assert_eq!(r1.summary.total_issues, r2.summary.total_issues);
        assert_eq!(r1.acceptance, r2.acceptance);
    }

    #[test]
    fn run_summary_still_uses_snake_case_keys() {
        let ec = EvidenceCollection {
            stage_id: "L0".to_string(),
            evidence_items: vec![ev_item("EV-L0-000001", "/p/a.py", Language::Python, SourceKind::PythonStage, "TODO fix")],
            index_by_path: StdHashMap::new(), index_by_kind: StdHashMap::new(), index_by_symbol: StdHashMap::new(),
            warnings: vec![],
            stats: EvidenceStats {
                files_processed: 1, files_skipped: 0, total_items: 1,
                items_by_kind: StdHashMap::new(), items_by_strength: StdHashMap::new(),
            },
            version: "1.0.0".to_string(),
        };
        let iu = minimal_iu("L0", vec![
            claim("CL-L0-000001", ClaimConfidence::Unknown, vec![], true),
            claim("CL-L0-000002", ClaimConfidence::Supported, vec!["EV-L0-999999"], false),
        ]);
        let stage = StageQualityInput {
            stage_id: "L0".to_string(),
            recognized_status: "available".to_string(),
            expected_status: Some("missing".to_string()),
            evidence: Some(&ec),
            understanding: Some(&iu),
            views: vec![],
            grounded_answer: None,
        };
        let report = reporter().evaluate(&base_input("sample-summary-stable", vec![stage]));
        for k in report.summary.issues_by_kind.keys() {
            assert!(
                k.chars().all(|c| c.is_lowercase() || c == '_'),
                "issue kind key 必须是 snake_case: {}",
                k
            );
        }
        assert!(report.summary.issues_by_kind.contains_key("stage_identification_mismatch"));
        assert!(report.summary.issues_by_kind.contains_key("unsupported_claim"));
        assert!(report.summary.issues_by_kind.contains_key("hallucinated_claim_blocked"));
        assert!(report.summary.issues_by_kind.contains_key("noisy_evidence"));
    }

    #[test]
    fn positive_guardrail_not_counted_as_problem() {
        let iu = minimal_iu("L0", vec![claim("CL-L0-000001", ClaimConfidence::Unknown, vec![], true)]);
        let stage = StageQualityInput {
            stage_id: "L0".to_string(),
            recognized_status: "available".to_string(),
            expected_status: None,
            evidence: None,
            understanding: Some(&iu),
            views: vec![],
            grounded_answer: None,
        };
        let report = reporter().evaluate(&base_input("sample-guardrail-split", vec![stage]));
        assert_eq!(report.summary.total_issues, 0, "正向 guardrail 不应计入负向问题");
        assert!(
            report.issues.iter().any(|i| {
                i.kind == QualityIssueKind::HallucinatedClaimBlocked
                    && i.polarity == QualityIssuePolarity::PositiveGuardrail
            }),
            "应存在 HallucinatedClaimBlocked 正向记录"
        );
        assert_eq!(report.acceptance, QualityAcceptanceStatus::MeetsGate);
    }
}
