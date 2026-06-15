//! 确定性质量报告生成器（Phase 7 Batch A，P7-T02）。
//!
//! `QualityReporter` 只读消费既有内存产物（`EvidenceCollection` /
//! `ImplementationUnderstanding` / `ViewGraph[]` / `GroundedAnswer`），产出
//! 确定性 `QualityReport`。
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

use crate::evidence::models::{EvidenceCollection, EvidenceItem, LineRange};
use crate::models::enums::{Language, SourceKind};
use crate::trace::models::GroundedAnswer;
use crate::understanding::models::{ClaimConfidence, ImplementationUnderstanding};
use crate::views::models::ViewGraph;

use super::models::{
    ArtifactKind, DetectionMethod, EvidenceQualityReport, IssueStatus, MetricSnapshot, QaQualityReport,
    QualityAcceptanceStatus, QualityIssue, QualityIssueKind, QualityIssuePolarity, QualityReport,
    QualityRunSummary, QualitySeverity, SummaryQuality, UnderstandingQualityReport, ViewQualityReport,
};

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
        // Batch A 占位门槛：无未闭环负向问题即视为达到门槛（具体阈值由后续 Batch 收敛）。
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
    if let Some(expected) = &stage.expected_status {
        if !expected.is_empty() && expected.as_str() != stage.recognized_status.as_str() {
            issues.push(make_issue(
                sample_id,
                &stage.stage_id,
                ArtifactKind::Stage,
                QualityIssueKind::StageIdentificationMismatch,
                QualitySeverity::High,
                None,
                None,
                None,
                None,
                None,
                &format!(
                    "阶段识别状态与期望不符：识别为 \"{}\"，期望 \"{}\"（识别缺口）",
                    stage.recognized_status, expected
                ),
            ));
        }
    }

    // —— evidence（RQ-003）——
    let evidence_id_set: HashSet<String> = match stage.evidence {
        Some(ec) => {
            let set: HashSet<String> = ec.evidence_items.iter().map(|i| i.evidence_id.clone()).collect();
            let (report, ev_issues) = evaluate_evidence(sample_id, &stage.stage_id, ec);
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
        let (report, un_issues) = evaluate_understanding(sample_id, &stage.stage_id, iu, &evidence_id_set);
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
        let (report, v_issues) = evaluate_view(sample_id, &stage.stage_id, view, &evidence_id_set, &claim_id_set);
        push_metric(metric_snapshots, "view_trace_resolvable", &stage.stage_id, report.trace_resolvable_ratio);
        view_reports.push(report);
        issues.extend(v_issues);
    }

    // —— Q&A（RQ-006）——
    if let Some(answer) = stage.grounded_answer {
        let (report, q_issues) = evaluate_qa(sample_id, &stage.stage_id, answer, &evidence_id_set);
        push_metric(metric_snapshots, "qa_citation_validity", &stage.stage_id, report.citation_validity_ratio);
        qa_reports.push(report);
        issues.extend(q_issues);
    }

    // issue_id 由主流程统一确定性分配（issues 已按阶段处理顺序、检查顺序自然排列）。
    issues
}

fn evaluate_evidence(
    sample_id: &str,
    stage_id: &str,
    ec: &EvidenceCollection,
) -> (EvidenceQualityReport, Vec<QualityIssue>) {
    let mut issues = Vec::new();
    let items = &ec.evidence_items;

    // 文件覆盖率：Batch A 以"是否收集到 evidence"为代理（无真实 ground-truth 文件清单）。
    let file_coverage_ratio = if items.is_empty() { 0.0f32 } else { 1.0f32 };
    if items.is_empty() {
        issues.push(make_issue(
            sample_id, stage_id, ArtifactKind::Evidence, QualityIssueKind::MissingEvidence,
            QualitySeverity::High, None, None, None, None, None,
            "该阶段未收集到任何 evidence（coverage 缺口）",
        ));
    }

    let mut valid_line_range = 0u32;
    let mut label_sane = 0u32;
    for item in items {
        if item.line_range.start >= 1 && item.line_range.end >= item.line_range.start {
            valid_line_range += 1;
        }
        if is_label_sane(item) {
            label_sane += 1;
        } else {
            issues.push(make_issue(
                sample_id, stage_id, ArtifactKind::Evidence, QualityIssueKind::WrongSourceKind,
                QualitySeverity::Medium,
                Some(item.evidence_id.as_str()), None, None, Some(item.source_path.as_str()), Some(item.line_range),
                &format!(
                    "evidence source_kind/language 标注与文件内容可能不一致（evidence={}）",
                    item.evidence_id
                ),
            ));
        }
        if is_noisy(&item.summary) {
            issues.push(make_issue(
                sample_id, stage_id, ArtifactKind::Evidence, QualityIssueKind::NoisyEvidence,
                QualitySeverity::Medium,
                Some(item.evidence_id.as_str()), None, None, Some(item.source_path.as_str()), Some(item.line_range),
                &format!(
                    "evidence 含噪声标记（TODO/FIXME/注释块）被当作主证据（evidence={}）",
                    item.evidence_id
                ),
            ));
        }
    }

    let total = items.len() as f32;
    let line_range_accuracy = if total > 0.0 { valid_line_range as f32 / total } else { 0.0 };
    let label_sanity_ratio = if total > 0.0 { label_sane as f32 / total } else { 1.0 };

    let report = EvidenceQualityReport {
        sample_id: sample_id.to_string(),
        stage_id: stage_id.to_string(),
        file_coverage_ratio,
        line_range_accuracy,
        label_sanity_ratio,
        uncovered_files: Vec::new(),
        issue_refs: Vec::new(),
    };
    (report, issues)
}

fn evaluate_understanding(
    sample_id: &str,
    stage_id: &str,
    iu: &ImplementationUnderstanding,
    evidence_id_set: &HashSet<String>,
) -> (UnderstandingQualityReport, Vec<QualityIssue>) {
    let mut issues = Vec::new();

    let mut claims_all_refs_ok = 0u32;
    let mut claims_with_refs = 0u32;
    for claim in &iu.claims {
        if claim.evidence_refs.is_empty() {
            if !claim.has_evidence_gap {
                issues.push(make_issue(
                    sample_id, stage_id, ArtifactKind::Understanding, QualityIssueKind::UnsupportedClaim,
                    QualitySeverity::High, None, Some(claim.claim_id.as_str()), None, None, None,
                    &format!("claim 未引用任何 evidence 且未声明 evidence_gap（claim={}）", claim.claim_id),
                ));
            } else {
                // 正向 guardrail：诚实声明 gap，未伪造 evidence 引用。
                issues.push(make_guardrail(
                    sample_id, stage_id, ArtifactKind::Understanding,
                    QualityIssueKind::HallucinatedClaimBlocked,
                    Some(claim.claim_id.as_str()),
                    &format!("claim 在证据不足时声明 evidence_gap，未伪造 evidence 引用（守卫生效，claim={}）", claim.claim_id),
                ));
            }
            continue;
        }
        claims_with_refs += 1;
        let mut refs_ok = true;
        for r in &claim.evidence_refs {
            if !evidence_id_set.contains(&r.evidence_id) {
                refs_ok = false;
                issues.push(make_issue(
                    sample_id, stage_id, ArtifactKind::Understanding, QualityIssueKind::UnsupportedClaim,
                    QualitySeverity::High,
                    Some(r.evidence_id.as_str()), Some(claim.claim_id.as_str()), None, None, None,
                    &format!("claim 引用了不存在的 evidence_id（claim={}，evidence={}）", claim.claim_id, r.evidence_id),
                ));
            }
        }
        if refs_ok {
            claims_all_refs_ok += 1;
        }
    }
    let claim_existence_check_ratio = if claims_with_refs > 0 {
        claims_all_refs_ok as f32 / claims_with_refs as f32
    } else if iu.claims.is_empty() {
        1.0
    } else {
        0.0
    };

    let unknown_claims = iu.claims.iter().filter(|c| c.confidence == ClaimConfidence::Unknown).count();
    let honest_unknown = iu
        .claims
        .iter()
        .filter(|c| c.confidence == ClaimConfidence::Unknown && c.has_evidence_gap)
        .count();
    let uncertainty_expression_ratio = if unknown_claims > 0 {
        honest_unknown as f32 / unknown_claims as f32
    } else {
        1.0
    };

    let confidence_calibration_ratio = claim_existence_check_ratio;

    let short = iu.summary.short.trim();
    let detailed = iu.summary.detailed.trim();
    let weak = short.is_empty() || detailed.len() < 10;
    if weak {
        issues.push(make_issue(
            sample_id, stage_id, ArtifactKind::Understanding, QualityIssueKind::WeakSummary,
            QualitySeverity::Medium, None, None, None, None, None,
            "StageSummary 内容过短或空洞，未充分概括阶段（理解缺口）",
        ));
    }

    let report = UnderstandingQualityReport {
        sample_id: sample_id.to_string(),
        stage_id: stage_id.to_string(),
        claim_existence_check_ratio,
        uncertainty_expression_ratio,
        confidence_calibration_ratio,
        summary_quality: SummaryQuality {
            total_summaries: 1,
            weak_summary_count: if weak { 1 } else { 0 },
        },
        issue_refs: Vec::new(),
    };
    (report, issues)
}

fn evaluate_view(
    sample_id: &str,
    stage_id: &str,
    view: &ViewGraph,
    evidence_id_set: &HashSet<String>,
    claim_id_set: &HashSet<String>,
) -> (ViewQualityReport, Vec<QualityIssue>) {
    let mut issues = Vec::new();
    let view_type = format!("{:?}", view.view_type).to_lowercase();

    if view.nodes.is_empty() {
        issues.push(make_issue(
            sample_id, stage_id, ArtifactKind::View, QualityIssueKind::EmptyOrUnhelpfulView,
            QualitySeverity::Medium, None, None, None, None, None,
            &format!("视图 {} 退化为空图，可解释性低（视图缺口）", view_type),
        ));
        return (
            ViewQualityReport {
                sample_id: sample_id.to_string(),
                stage_id: stage_id.to_string(),
                view_type,
                trace_resolvable_ratio: 1.0,
                isolated_node_count: 0,
                suspected_misconnection_count: 0,
                issue_refs: Vec::new(),
            },
            issues,
        );
    }

    let mut total_refs = 0u32;
    let mut resolvable_refs = 0u32;
    for n in &view.nodes {
        for tr in &n.trace_refs {
            total_refs += 1;
            if trace_ref_ok(&tr.evidence_id, &tr.claim_id, evidence_id_set, claim_id_set) {
                resolvable_refs += 1;
            }
        }
    }
    for e in &view.edges {
        for tr in &e.trace_refs {
            total_refs += 1;
            if trace_ref_ok(&tr.evidence_id, &tr.claim_id, evidence_id_set, claim_id_set) {
                resolvable_refs += 1;
            }
        }
    }
    let trace_resolvable_ratio = if total_refs > 0 {
        resolvable_refs as f32 / total_refs as f32
    } else {
        1.0
    };

    // 孤立节点（无连边）
    let mut connected: HashSet<String> = HashSet::new();
    for e in &view.edges {
        connected.insert(e.source_node_id.clone());
        connected.insert(e.target_node_id.clone());
    }
    let mut isolated_node_count = 0u32;
    let mut first_iso: Option<String> = None;
    for n in &view.nodes {
        if !connected.contains(&n.node_id) {
            isolated_node_count += 1;
            if first_iso.is_none() {
                first_iso = Some(n.node_id.clone());
            }
        }
    }
    if let Some(iso) = first_iso {
        issues.push(make_issue(
            sample_id, stage_id, ArtifactKind::View, QualityIssueKind::EmptyOrUnhelpfulView,
            QualitySeverity::Low, None, None, Some(iso.as_str()), None, None,
            &format!("视图 {} 含孤立节点，连接性缺口（node={}）", view_type, iso),
        ));
    }

    (
        ViewQualityReport {
            sample_id: sample_id.to_string(),
            stage_id: stage_id.to_string(),
            view_type,
            trace_resolvable_ratio,
            isolated_node_count,
            suspected_misconnection_count: 0,
            issue_refs: Vec::new(),
        },
        issues,
    )
}

fn evaluate_qa(
    sample_id: &str,
    stage_id: &str,
    answer: &GroundedAnswer,
    evidence_id_set: &HashSet<String>,
) -> (QaQualityReport, Vec<QualityIssue>) {
    let mut issues = Vec::new();

    let mut total_cit = 0u32;
    let mut valid_cit = 0u32;
    for c in &answer.citations {
        if let Some(ev) = &c.evidence_id {
            if !ev.is_empty() {
                total_cit += 1;
                if evidence_id_set.contains(ev) {
                    valid_cit += 1;
                } else {
                    issues.push(make_issue(
                        sample_id, stage_id, ArtifactKind::Qa, QualityIssueKind::QaAnswerWithoutValidCitation,
                        QualitySeverity::High, Some(ev.as_str()), c.claim_id.as_deref(), None, None, None,
                        &format!("Q&A 回答引用了不存在的 evidence（evidence={}）", ev),
                    ));
                }
            }
        }
    }
    let citation_validity_ratio = if total_cit > 0 {
        valid_cit as f32 / total_cit as f32
    } else {
        1.0
    };

    // Batch A：无 QaEvaluationQuestionSet 输入时，以回答置信度为代理。
    let answerable_hit_ratio = if answer.is_degraded || answer.confidence == ClaimConfidence::Unknown {
        0.0f32
    } else {
        1.0
    };
    let unknown_honesty_ratio = if answer.confidence == ClaimConfidence::Unknown { 1.0 } else { 0.0 };

    (
        QaQualityReport {
            sample_id: sample_id.to_string(),
            stage_id: stage_id.to_string(),
            citation_validity_ratio,
            answerable_hit_ratio,
            unknown_honesty_ratio,
            issue_refs: Vec::new(),
        },
        issues,
    )
}

// ─── 辅助 ────────────────────────────────────────────────────────────

fn trace_ref_ok(
    evidence_id: &Option<String>,
    claim_id: &Option<String>,
    evidence_set: &HashSet<String>,
    claim_set: &HashSet<String>,
) -> bool {
    let mut ok = true;
    if let Some(ev) = evidence_id {
        if !ev.is_empty() && !evidence_set.contains(ev) {
            ok = false;
        }
    }
    if let Some(cl) = claim_id {
        if !cl.is_empty() && !claim_set.contains(cl) {
            ok = false;
        }
    }
    ok
}

#[allow(clippy::too_many_arguments)]
fn make_issue(
    sample_id: &str,
    stage_id: &str,
    artifact_kind: ArtifactKind,
    kind: QualityIssueKind,
    severity: QualitySeverity,
    evidence_id: Option<&str>,
    claim_id: Option<&str>,
    node_id: Option<&str>,
    source_path: Option<&str>,
    line_range: Option<LineRange>,
    description: &str,
) -> QualityIssue {
    QualityIssue {
        issue_id: String::new(),
        sample_id: sample_id.to_string(),
        stage_id: stage_id.to_string(),
        artifact_kind,
        kind,
        polarity: kind.default_polarity(),
        severity,
        evidence_id: evidence_id.map(|s| s.to_string()),
        claim_id: claim_id.map(|s| s.to_string()),
        node_id: node_id.map(|s| s.to_string()),
        source_path: source_path.map(|s| s.to_string()),
        line_range,
        description: description.to_string(),
        detected_by: DetectionMethod::Automated,
        status: IssueStatus::Open,
    }
}

fn make_guardrail(
    sample_id: &str,
    stage_id: &str,
    artifact_kind: ArtifactKind,
    kind: QualityIssueKind,
    claim_id: Option<&str>,
    description: &str,
) -> QualityIssue {
    let mut issue = make_issue(
        sample_id, stage_id, artifact_kind, kind, QualitySeverity::Low,
        None, claim_id, None, None, None, description,
    );
    issue.polarity = QualityIssuePolarity::PositiveGuardrail;
    issue.status = IssueStatus::Open;
    issue
}

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
        *issues_by_kind.entry(format!("{:?}", i.kind).to_lowercase()).or_insert(0) += 1;
        if i.polarity == QualityIssuePolarity::Problem {
            total_issues += 1;
            *issues_by_severity.entry(format!("{:?}", i.severity).to_lowercase()).or_insert(0) += 1;
            *issues_by_status.entry(format!("{:?}", i.status).to_lowercase()).or_insert(0) += 1;
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

fn sanitize_scope(sample_id: &str) -> String {
    let s: String = sample_id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect();
    if s.is_empty() {
        "sample".to_string()
    } else {
        s
    }
}

fn is_noisy(summary: &str) -> bool {
    let upper = summary.to_uppercase();
    upper.contains("TODO") || upper.contains("FIXME") || upper.contains("XXX") || upper.contains("HACK")
}

/// evidence 的 source_kind 与 language 标注是否自洽（Batch A 启发式）。
fn is_label_sane(item: &EvidenceItem) -> bool {
    match item.language {
        Language::Python => item.source_kind == SourceKind::PythonStage,
        Language::Verilog | Language::SystemVerilog => item.source_kind == SourceKind::Rtl,
        Language::Markdown => item.source_kind == SourceKind::Doc,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::models::{EvidenceCollection, EvidenceItem, EvidenceStats, EvidenceStrength};
    use crate::models::enums::{Language, SourceKind};
    use crate::understanding::models::{
        ClaimCategory, ClaimConfidence, EvidenceRef, GenerationMeta, ImplementationClaim,
        ImplementationUnderstanding, StageSummary, UnderstandingStats,
    };
    use crate::views::models::{ViewGraph, ViewMeta, ViewType};
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
            citations: vec![crate::trace::models::GroundedAnswerCitation {
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
}
