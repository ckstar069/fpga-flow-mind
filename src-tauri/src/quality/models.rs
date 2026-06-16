//! Phase 7 质量评估数据模型。
//!
//! 本模块定义 Phase 7（真实项目评估与 evidence/understanding 质量补强）的评估层
//! 数据结构。所有对象都是 **Phase 7 质量评估产物**，描述"工具理解得怎么样"，
//! 不是用户业务项目的审计结论，不对目标项目做正确性判断。
//!
//! 对齐文档：`docs/design/phase-7-real-project-evaluation-model.md`（active）。
//! 既有类型引用（保持稳定，不重定义）：`LineRange`（1-based 闭区间，见 evidence model）。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::evidence::models::LineRange;

// ─── 枚举：被评估产物类型 / 发现方式 / 处置状态 ──────────────────────

/// 被评估产物类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    Workspace,
    Stage,
    Evidence,
    Understanding,
    View,
    Qa,
    Ui,
}

/// 质量记录的发现方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionMethod {
    Automated,
    Manual,
    DesktopAcceptance,
}

/// 负向问题的处置状态（仅对 polarity=Problem 适用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueStatus {
    Open,
    Fixed,
    AcceptedAsKnownLimitation,
}

impl IssueStatus {
    /// 稳定的 snake_case 字符串 key（与 serde 一致）。
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueStatus::Open => "open",
            IssueStatus::Fixed => "fixed",
            IssueStatus::AcceptedAsKnownLimitation => "accepted_as_known_limitation",
        }
    }
}

// ─── 枚举：分类 / 严重程度 / 极性 ─────────────────────────────────────

/// 工具理解质量记录分类。
///
/// 全部围绕"工具是否理解到位"，不涉及目标项目正确性。
/// 其中 `HallucinatedClaimBlocked` 为正向 guardrail（polarity=PositiveGuardrail），
/// 其余为负向问题（polarity=Problem）。
///
/// **P2 校准说明：** 新增 `ExpectedEmptyTiming` / `IsolatedOrUnconnectedView` /
/// `TraceabilityGap` / `LowSemanticDiversity` 四个更细分类，用于区分"诚实空图"
/// 与"应生成但为空图"，以及 node/edge 追溯缺失和语义多样性不足等不同退化模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityIssueKind {
    /// 应被覆盖的证据未被收集（文件/符号级缺口）
    MissingEvidence,
    /// 证据被收集但含噪声（TODO/注释块/实验代码被当主证据）
    NoisyEvidence,
    /// evidence 的 source_kind / language 标注与实际不符
    WrongSourceKind,
    /// 阶段识别与人工期望不符（如命名异常被判 missing、空阶段误判、阶段漏识别）
    StageIdentificationMismatch,
    /// StageSummary 过于空洞或未抓住阶段核心
    WeakSummary,
    /// claim 缺少 evidence_refs 或未通过 existence check
    UnsupportedClaim,
    /// 无证据 claim 被 hallucination guard 拦截——正向 guardrail 记录（polarity=PositiveGuardrail），不计入负向 backlog
    HallucinatedClaimBlocked,
    /// 视图退化为孤立方块/空图/无信息
    EmptyOrUnhelpfulView,
    /// 【P2 新增】timing 视图为空但属于预期行为（如 Python 阶段无 cycle/latency/clock 等时序证据）
    ExpectedEmptyTiming,
    /// 【P2 新增】非空视图但大部分节点孤立或缺少合理边
    IsolatedOrUnconnectedView,
    /// 【P2 新增】node/edge 缺 trace_refs 或 trace_refs 不完整
    TraceabilityGap,
    /// 【P2 新增】多个节点标签/类型高度重复，信息价值低
    LowSemanticDiversity,
    /// 有证据支持的问题，Q&A 未能给出回答
    QaUnansweredWhenEvidenceExists,
    /// Q&A 回答的 citation 指向不存在/不相关的 evidence
    QaAnswerWithoutValidCitation,
    /// UI 状态令人困惑（空状态/加载/降级提示不清）；仅用于 UI 状态表达问题，不用于阶段识别误判
    ConfusingUiState,
}

impl QualityIssueKind {
    /// 该分类的默认极性。
    pub fn default_polarity(self) -> QualityIssuePolarity {
        match self {
            QualityIssueKind::HallucinatedClaimBlocked => QualityIssuePolarity::PositiveGuardrail,
            _ => QualityIssuePolarity::Problem,
        }
    }

    /// 区分"诚实空图"（expected）与"应生成但为空"（problematic）。
    /// 仅对视图类 issue 有效，返回 true 表示该 issue 属于预期行为而非质量退化。
    pub fn is_expected_empty_or_signal(&self) -> bool {
        matches!(self, QualityIssueKind::ExpectedEmptyTiming)
    }

    /// 稳定的 snake_case 字符串 key（与 serde 一致，无运行时分配）。
    pub fn as_str(&self) -> &'static str {
        match self {
            QualityIssueKind::MissingEvidence => "missing_evidence",
            QualityIssueKind::NoisyEvidence => "noisy_evidence",
            QualityIssueKind::WrongSourceKind => "wrong_source_kind",
            QualityIssueKind::StageIdentificationMismatch => "stage_identification_mismatch",
            QualityIssueKind::WeakSummary => "weak_summary",
            QualityIssueKind::UnsupportedClaim => "unsupported_claim",
            QualityIssueKind::HallucinatedClaimBlocked => "hallucinated_claim_blocked",
            QualityIssueKind::EmptyOrUnhelpfulView => "empty_or_unhelpful_view",
            QualityIssueKind::ExpectedEmptyTiming => "expected_empty_timing",
            QualityIssueKind::IsolatedOrUnconnectedView => "isolated_or_unconnected_view",
            QualityIssueKind::TraceabilityGap => "traceability_gap",
            QualityIssueKind::LowSemanticDiversity => "low_semantic_diversity",
            QualityIssueKind::QaUnansweredWhenEvidenceExists => "qa_unanswered_when_evidence_exists",
            QualityIssueKind::QaAnswerWithoutValidCitation => "qa_answer_without_valid_citation",
            QualityIssueKind::ConfusingUiState => "confusing_ui_state",
        }
    }
}

/// 问题严重程度（仅用于补强优先级排序，不用于目标项目评价）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualitySeverity {
    /// 轻微：不影响追溯，体验瑕疵
    Low,
    /// 中等：局部理解质量受损，但可追溯
    Medium,
    /// 重要：理解质量系统性受损或追溯链断裂
    High,
}

impl QualitySeverity {
    /// 稳定的 snake_case 字符串 key（与 serde 一致）。
    pub fn as_str(&self) -> &'static str {
        match self {
            QualitySeverity::Low => "low",
            QualitySeverity::Medium => "medium",
            QualitySeverity::High => "high",
        }
    }
}

/// 质量记录极性。
///
/// 区分负向质量问题与正向守卫生效记录。
/// `PositiveGuardrail` 记录（如 `HallucinatedClaimBlocked`）不计入负向 backlog、
/// 不参与门槛判定，因此"守卫生效"不会导致 Phase 7 被判为质量问题未闭环。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityIssuePolarity {
    /// 负向质量问题：进入补强 backlog，参与门槛判定
    Problem,
    /// 正向守卫生效记录：仅作为"守卫工作正常"的证据，不计入 backlog、不参与门槛判定
    PositiveGuardrail,
}

// ─── 真实项目样本登记 ────────────────────────────────────────────────

/// 一个被纳入 Phase 7 评估的真实（或等价本地只读）样本登记记录。
///
/// 仅登记"评估输入是什么"，不携带任何对样本项目正确性的判断。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealProjectSample {
    pub sample_id: String,
    pub root_path: String,
    pub source_description: String,
    pub expected_stages: Vec<ExpectedStageEntry>,
    pub file_type_distribution: FileTypeDistribution,
    pub scale_metrics: SampleScaleMetrics,
    pub trait_tags: Vec<String>,
    /// 登记时间（ISO8601，由调用方传入，评估运行不自行取系统时间）
    pub registered_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedStageEntry {
    pub stage_id: String,
    /// 人工期望的 StageStatus：available / empty / missing / naming_anomaly
    pub expected_status: String,
    pub expected_languages: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeDistribution {
    pub by_language: HashMap<String, u32>,
    pub by_source_kind: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleScaleMetrics {
    pub stage_count: u32,
    pub file_count: u32,
    pub total_lines: u32,
}

/// 对单个阶段执行质量评估的目标对象（绑定该阶段既有 Phase 1~6 产物的存在性）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageEvaluationTarget {
    pub sample_id: String,
    pub stage_id: String,
    pub recognized_status: String,
    pub evidence_collection_present: bool,
    pub understanding_present: bool,
    pub view_graph_types: Vec<String>,
    pub qa_history_present: bool,
}

// ─── 分维度质量报告 ──────────────────────────────────────────────────

/// evidence 覆盖率与缺口评估结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceQualityReport {
    pub sample_id: String,
    pub stage_id: String,
    /// 被覆盖的源文件数 / 该阶段应覆盖源文件数（内部门槛指标，不评价目标项目）
    pub file_coverage_ratio: f32,
    /// line_range 准确性比例（落在真实行范围内的比例）
    pub line_range_accuracy: f32,
    /// strength / source_kind / language 标注合理性比例
    pub label_sanity_ratio: f32,
    /// 未覆盖文件及原因
    pub uncovered_files: Vec<UncoveredFile>,
    /// 该阶段证据相关 issue 的引用（kind ∈ missing_evidence/noisy_evidence/wrong_source_kind）
    pub issue_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncoveredFile {
    pub source_path: String,
    pub reason: String,
}

/// ImplementationUnderstanding 质量评估结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnderstandingQualityReport {
    pub sample_id: String,
    pub stage_id: String,
    /// claim 通过 existence check 的比例（evidence_id 真实存在）
    pub claim_existence_check_ratio: f32,
    /// unknown / evidence_gap 表达合理性比例（证据不足处被诚实表达）
    pub uncertainty_expression_ratio: f32,
    /// claim 中 confidence 标注合理性比例（与 supporting evidence 是否一致）
    pub confidence_calibration_ratio: f32,
    /// StageSummary 质量评估（weak_summary 计数等）
    pub summary_quality: SummaryQuality,
    pub issue_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryQuality {
    pub total_summaries: u32,
    pub weak_summary_count: u32,
}

/// 三类视图可解释性评估结果（每类视图一份）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewQualityReport {
    pub sample_id: String,
    pub stage_id: String,
    pub view_type: String,
    /// 节点/边 trace_refs 可解析回 claim/evidence 的比例
    pub trace_resolvable_ratio: f32,
    /// 孤立节点数（无连边）
    pub isolated_node_count: u32,
    /// 错连嫌疑计数
    pub suspected_misconnection_count: u32,
    pub issue_refs: Vec<String>,
}

/// Grounded Q&A 可用性评估结果（基于 MockProvider）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QaQualityReport {
    pub sample_id: String,
    pub stage_id: String,
    /// citation 指向真实 evidence/claim 的比例
    pub citation_validity_ratio: f32,
    /// 对"有证据支持问题"的回答命中率
    pub answerable_hit_ratio: f32,
    /// 对"无证据问题"诚实返回 unknown/gap 的比例
    pub unknown_honesty_ratio: f32,
    pub issue_refs: Vec<String>,
}

// ─── Q&A 评估问题集（MockProvider 基线）──────────────────────────────

/// 单条 Q&A 评估问题。
///
/// 仅表达"工具在此问题上预期是否可回答、应引用哪些证据"，不携带目标项目正确性判断。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QaEvaluationQuestion {
    pub question: String,
    pub stage_id: String,
    pub expected_answerability: QaExpectedAnswerability,
    /// 预期应引用的 evidence_id（answerable 时填，用于核对 citation 有效性）
    pub expected_evidence_ids: Vec<String>,
    /// 预期应引用的 claim_id（可选）
    pub expected_claim_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QaExpectedAnswerability {
    Answerable,
    NotAnswerable,
}

/// 一组 Q&A 评估问题。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QaEvaluationQuestionSet {
    pub set_id: String,
    pub sample_id: String,
    pub questions: Vec<QaEvaluationQuestion>,
}

// ─── 统一质量问题记录 ────────────────────────────────────────────────

/// 一条工具理解质量记录。
///
/// 仅描述"工具理解质量"，不描述"目标项目正确/错误"。
/// 每条必须可追溯到 stage_id + artifact_kind + 可选 evidence_id/claim_id/node_id/source_path/line_range。
/// polarity 区分负向问题（problem）与正向守卫生效记录（positive_guardrail）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    /// 记录唯一标识，格式 "QI-<stage_id>-<6位序号>"
    pub issue_id: String,
    pub sample_id: String,
    pub stage_id: String,
    pub artifact_kind: ArtifactKind,
    pub kind: QualityIssueKind,
    pub polarity: QualityIssuePolarity,
    /// 严重程度（仅对 polarity=Problem 有意义；正向 guardrail 一律 Low）
    pub severity: QualitySeverity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claim_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_range: Option<LineRange>,
    /// 问题描述（客观、避免审计用语）
    pub description: String,
    pub detected_by: DetectionMethod,
    /// 处置状态（仅对 polarity=Problem 适用；正向 guardrail 固定 Open）
    pub status: IssueStatus,
}

impl QualityIssue {
    /// 是否绑定到具体追溯信息（evidence/claim/node/source location）。
    ///
    /// 阶段级或摘要级 issue（如 stage_identification_mismatch / weak_summary）天然是
    /// gap/uncertainty 表达，仅以 stage_id 追溯，不要求具体 trace 字段。
    pub fn has_specific_trace(&self) -> bool {
        self.evidence_id.is_some()
            || self.claim_id.is_some()
            || self.node_id.is_some()
            || self.source_path.is_some()
    }

    /// 该 issue 是否可追溯（含 stage_id 必填 + 必要时具体 trace）。
    pub fn is_traceable(&self) -> bool {
        // stage_id 恒为必填（构造时保证）；下列 kind 天然是 gap/uncertainty 表达，
        // 仅以 stage_id 追溯即视为可追溯（无需具体 evidence/claim/node/source 字段）。
        let inherently_gap = matches!(
            self.kind,
            QualityIssueKind::StageIdentificationMismatch
                | QualityIssueKind::WeakSummary
                | QualityIssueKind::MissingEvidence
                | QualityIssueKind::EmptyOrUnhelpfulView
                | QualityIssueKind::ConfusingUiState
                | QualityIssueKind::ExpectedEmptyTiming
                | QualityIssueKind::IsolatedOrUnconnectedView
                | QualityIssueKind::TraceabilityGap
                | QualityIssueKind::LowSemanticDiversity
        );
        inherently_gap || self.has_specific_trace()
    }
}

// ─── 运行汇总与门槛判定 ──────────────────────────────────────────────

/// 一次 Phase 7 评估运行的汇总。
///
/// `total_issues` / `issues_by_severity` / `issues_by_status` 仅统计 polarity=Problem 的负向问题；
/// `positive_guardrail_event_count` 单独统计正向守卫生效记录，不计入负向 backlog、不参与门槛判定。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRunSummary {
    pub run_id: String,
    pub sample_ids: Vec<String>,
    /// 负向问题（polarity=Problem）总数；进入 backlog 与门槛判定
    pub total_issues: u32,
    /// 正向守卫生效记录（polarity=PositiveGuardrail）计数；不进入 backlog、不参与门槛判定
    pub positive_guardrail_event_count: u32,
    pub issues_by_kind: HashMap<String, u32>,
    /// 仅统计 polarity=Problem 的严重程度分布
    pub issues_by_severity: HashMap<String, u32>,
    /// 仅统计 polarity=Problem 的处置状态分布
    pub issues_by_status: HashMap<String, u32>,
    /// 各维度汇总指标（覆盖率/命中率等，内部门槛用）
    pub metric_snapshots: Vec<MetricSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshot {
    pub metric_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage_id: Option<String>,
    pub value: f32,
}

/// Phase 7 质量门槛判定结果。
///
/// 仅表达"质量补强是否达到 Phase 7 退出门槛"，不输出 PASS/HOLD，不评价目标项目。
/// 门槛判定只看 polarity=Problem 的负向问题是否闭环。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityAcceptanceStatus {
    /// 尚未达到门槛，仍需补强
    BelowGate,
    /// 达到门槛，满足 Phase 7 退出条件
    MeetsGate,
}

// ─── 顶层报告（reporter 输出）────────────────────────────────────────

/// Phase 7 质量报告（reporter 的顶层确定性输出）。
///
/// 聚合分维度报告、统一 issue 列表、运行汇总与门槛判定。
/// 不含系统时间——`generated_at` 由调用方传入的确定性标记填充。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    /// 报告唯一标识，格式 "QR-<scope>-<6位序号>"，确定性生成
    pub report_id: String,
    pub sample_id: String,
    pub stage_ids: Vec<String>,
    /// 调用方传入的确定性时间标记（reporter 不取系统时间）
    pub generated_at: String,
    pub evidence_reports: Vec<EvidenceQualityReport>,
    pub understanding_reports: Vec<UnderstandingQualityReport>,
    pub view_reports: Vec<ViewQualityReport>,
    pub qa_reports: Vec<QaQualityReport>,
    /// 全部质量记录（problem + positive_guardrail），按 (stage_id, issue_id) 稳定排序
    pub issues: Vec<QualityIssue>,
    pub summary: QualityRunSummary,
    pub acceptance: QualityAcceptanceStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enums_serialize_snake_case() {
        assert_eq!(
            serde_json::to_string(&ArtifactKind::Understanding).unwrap(),
            "\"understanding\""
        );
        assert_eq!(
            serde_json::to_string(&DetectionMethod::DesktopAcceptance).unwrap(),
            "\"desktop_acceptance\""
        );
        assert_eq!(
            serde_json::to_string(&IssueStatus::AcceptedAsKnownLimitation).unwrap(),
            "\"accepted_as_known_limitation\""
        );
        assert_eq!(
            serde_json::to_string(&QualityIssueKind::StageIdentificationMismatch).unwrap(),
            "\"stage_identification_mismatch\""
        );
        assert_eq!(
            serde_json::to_string(&QualityIssueKind::HallucinatedClaimBlocked).unwrap(),
            "\"hallucinated_claim_blocked\""
        );
        assert_eq!(
            serde_json::to_string(&QualitySeverity::High).unwrap(),
            "\"high\""
        );
        assert_eq!(
            serde_json::to_string(&QualityIssuePolarity::PositiveGuardrail).unwrap(),
            "\"positive_guardrail\""
        );
        assert_eq!(
            serde_json::to_string(&QaExpectedAnswerability::NotAnswerable).unwrap(),
            "\"not_answerable\""
        );
        assert_eq!(
            serde_json::to_string(&QualityAcceptanceStatus::BelowGate).unwrap(),
            "\"below_gate\""
        );
    }

    #[test]
    fn enums_reject_invalid() {
        assert!(serde_json::from_str::<QualityIssueKind>("\"definitely_a_bug\"").is_err());
        assert!(serde_json::from_str::<QualitySeverity>("\"critical\"").is_err());
        assert!(serde_json::from_str::<QualityIssuePolarity>("\"neutral\"").is_err());
    }

    #[test]
    fn enum_as_str_matches_serde_snake_case() {
        for kind in [
            QualityIssueKind::MissingEvidence,
            QualityIssueKind::NoisyEvidence,
            QualityIssueKind::WrongSourceKind,
            QualityIssueKind::StageIdentificationMismatch,
            QualityIssueKind::WeakSummary,
            QualityIssueKind::UnsupportedClaim,
            QualityIssueKind::HallucinatedClaimBlocked,
            QualityIssueKind::EmptyOrUnhelpfulView,
            QualityIssueKind::ExpectedEmptyTiming,
            QualityIssueKind::IsolatedOrUnconnectedView,
            QualityIssueKind::TraceabilityGap,
            QualityIssueKind::LowSemanticDiversity,
            QualityIssueKind::QaUnansweredWhenEvidenceExists,
            QualityIssueKind::QaAnswerWithoutValidCitation,
            QualityIssueKind::ConfusingUiState,
        ] {
            let from_serde = serde_json::to_string(&kind).unwrap();
            assert_eq!(
                kind.as_str(),
                from_serde.trim_matches('"'),
                "{:?} as_str 与 serde 不一致",
                kind
            );
        }
        assert_eq!(QualitySeverity::Low.as_str(), "low");
        assert_eq!(QualitySeverity::Medium.as_str(), "medium");
        assert_eq!(QualitySeverity::High.as_str(), "high");
        assert_eq!(IssueStatus::Open.as_str(), "open");
        assert_eq!(IssueStatus::Fixed.as_str(), "fixed");
        assert_eq!(IssueStatus::AcceptedAsKnownLimitation.as_str(), "accepted_as_known_limitation");
    }

    #[test]
    fn issue_kind_default_polarity() {
        assert_eq!(
            QualityIssueKind::HallucinatedClaimBlocked.default_polarity(),
            QualityIssuePolarity::PositiveGuardrail
        );
        for kind in [
            QualityIssueKind::MissingEvidence,
            QualityIssueKind::NoisyEvidence,
            QualityIssueKind::WrongSourceKind,
            QualityIssueKind::StageIdentificationMismatch,
            QualityIssueKind::WeakSummary,
            QualityIssueKind::UnsupportedClaim,
            QualityIssueKind::EmptyOrUnhelpfulView,
            QualityIssueKind::ExpectedEmptyTiming,
            QualityIssueKind::IsolatedOrUnconnectedView,
            QualityIssueKind::TraceabilityGap,
            QualityIssueKind::LowSemanticDiversity,
            QualityIssueKind::QaUnansweredWhenEvidenceExists,
            QualityIssueKind::QaAnswerWithoutValidCitation,
            QualityIssueKind::ConfusingUiState,
        ] {
            assert_eq!(kind.default_polarity(), QualityIssuePolarity::Problem, "{:?} should be problem", kind);
        }
    }

    #[test]
    fn quality_issue_roundtrip() {
        let issue = QualityIssue {
            issue_id: "QI-L0-000001".to_string(),
            sample_id: "sample-001".to_string(),
            stage_id: "L0".to_string(),
            artifact_kind: ArtifactKind::Evidence,
            kind: QualityIssueKind::MissingEvidence,
            polarity: QualityIssuePolarity::Problem,
            severity: QualitySeverity::Medium,
            evidence_id: None,
            claim_id: None,
            node_id: None,
            source_path: Some("/tmp/proj/L0/missing.py".to_string()),
            line_range: None,
            description: "evidence 未覆盖文件 /tmp/proj/L0/missing.py（缺口）".to_string(),
            detected_by: DetectionMethod::Automated,
            status: IssueStatus::Open,
        };
        let json = serde_json::to_string(&issue).unwrap();
        let back: QualityIssue = serde_json::from_str(&json).unwrap();
        assert_eq!(issue.issue_id, back.issue_id);
        assert_eq!(issue.kind, back.kind);
        assert_eq!(issue.polarity, back.polarity);
        assert_eq!(issue.source_path, back.source_path);
        // None 字段被 skip
        assert!(!json.contains("\"evidence_id\""));
        assert!(!json.contains("\"line_range\""));
    }

    #[test]
    fn new_p2_kinds_serialize_snake_case() {
        assert_eq!(
            serde_json::to_string(&QualityIssueKind::ExpectedEmptyTiming).unwrap(),
            "\"expected_empty_timing\""
        );
        assert_eq!(
            serde_json::to_string(&QualityIssueKind::IsolatedOrUnconnectedView).unwrap(),
            "\"isolated_or_unconnected_view\""
        );
        assert_eq!(
            serde_json::to_string(&QualityIssueKind::TraceabilityGap).unwrap(),
            "\"traceability_gap\""
        );
        assert_eq!(
            serde_json::to_string(&QualityIssueKind::LowSemanticDiversity).unwrap(),
            "\"low_semantic_diversity\""
        );
    }

    #[test]
    fn is_expected_empty_or_signal_only_expected_empty_timing() {
        assert!(QualityIssueKind::ExpectedEmptyTiming.is_expected_empty_or_signal());
        assert!(!QualityIssueKind::EmptyOrUnhelpfulView.is_expected_empty_or_signal());
        assert!(!QualityIssueKind::TraceabilityGap.is_expected_empty_or_signal());
        assert!(!QualityIssueKind::IsolatedOrUnconnectedView.is_expected_empty_or_signal());
        assert!(!QualityIssueKind::LowSemanticDiversity.is_expected_empty_or_signal());
        assert!(!QualityIssueKind::MissingEvidence.is_expected_empty_or_signal());
    }

    #[test]
    fn issue_traceability_rules() {
        // 具体追溯：source_path 绑定
        let with_source = QualityIssue {
            issue_id: "QI-L0-000001".to_string(),
            sample_id: "s".to_string(),
            stage_id: "L0".to_string(),
            artifact_kind: ArtifactKind::Evidence,
            kind: QualityIssueKind::MissingEvidence,
            polarity: QualityIssuePolarity::Problem,
            severity: QualitySeverity::Medium,
            evidence_id: None,
            claim_id: None,
            node_id: None,
            source_path: Some("/x.py".to_string()),
            line_range: None,
            description: "gap".to_string(),
            detected_by: DetectionMethod::Automated,
            status: IssueStatus::Open,
        };
        assert!(with_source.has_specific_trace());
        assert!(with_source.is_traceable());

        // 阶段级 issue：无具体 trace 但仍可追溯（gap/uncertainty 表达）
        let stage_level = QualityIssue {
            issue_id: "QI-L0-000002".to_string(),
            sample_id: "s".to_string(),
            stage_id: "L0".to_string(),
            artifact_kind: ArtifactKind::Stage,
            kind: QualityIssueKind::StageIdentificationMismatch,
            polarity: QualityIssuePolarity::Problem,
            severity: QualitySeverity::High,
            evidence_id: None,
            claim_id: None,
            node_id: None,
            source_path: None,
            line_range: None,
            description: "阶段识别状态与期望不符（gap）".to_string(),
            detected_by: DetectionMethod::Automated,
            status: IssueStatus::Open,
        };
        assert!(!stage_level.has_specific_trace());
        assert!(stage_level.is_traceable());

        // missing_evidence 无具体 source_path（整阶段覆盖缺口）仍是可追溯的 gap
        let coverage_gap = QualityIssue {
            issue_id: "QI-L0-000003".to_string(),
            sample_id: "s".to_string(),
            stage_id: "L0".to_string(),
            artifact_kind: ArtifactKind::Evidence,
            kind: QualityIssueKind::MissingEvidence,
            polarity: QualityIssuePolarity::Problem,
            severity: QualitySeverity::Medium,
            evidence_id: None,
            claim_id: None,
            node_id: None,
            source_path: None,
            line_range: None,
            description: "该阶段未收集到任何 evidence（coverage 缺口）".to_string(),
            detected_by: DetectionMethod::Automated,
            status: IssueStatus::Open,
        };
        assert!(!coverage_gap.has_specific_trace());
        assert!(coverage_gap.is_traceable(), "missing_evidence 整阶段缺口应可追溯");
    }

    #[test]
    fn run_summary_roundtrip() {
        let summary = QualityRunSummary {
            run_id: "QR-sample-001-000001".to_string(),
            sample_ids: vec!["sample-001".to_string()],
            total_issues: 3,
            positive_guardrail_event_count: 1,
            issues_by_kind: HashMap::new(),
            issues_by_severity: HashMap::new(),
            issues_by_status: HashMap::new(),
            metric_snapshots: vec![MetricSnapshot {
                metric_name: "evidence_file_coverage".to_string(),
                stage_id: Some("L0".to_string()),
                value: 0.75,
            }],
        };
        let json = serde_json::to_string(&summary).unwrap();
        let back: QualityRunSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(summary.total_issues, back.total_issues);
        assert_eq!(
            summary.positive_guardrail_event_count,
            back.positive_guardrail_event_count
        );
    }

    #[test]
    fn real_project_sample_roundtrip() {
        let sample = RealProjectSample {
            sample_id: "sample-001".to_string(),
            root_path: "/tmp/proj".to_string(),
            source_description: "ai_project_template 生成项目".to_string(),
            expected_stages: vec![ExpectedStageEntry {
                stage_id: "L0".to_string(),
                expected_status: "available".to_string(),
                expected_languages: vec!["python".to_string()],
                note: None,
            }],
            file_type_distribution: FileTypeDistribution {
                by_language: HashMap::new(),
                by_source_kind: HashMap::new(),
            },
            scale_metrics: SampleScaleMetrics {
                stage_count: 1,
                file_count: 5,
                total_lines: 200,
            },
            trait_tags: vec!["naming_anomaly".to_string()],
            registered_at: "2026-06-15T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&sample).unwrap();
        let back: RealProjectSample = serde_json::from_str(&json).unwrap();
        assert_eq!(sample.sample_id, back.sample_id);
        assert_eq!(sample.expected_stages.len(), back.expected_stages.len());
        assert_eq!(sample.scale_metrics.file_count, back.scale_metrics.file_count);
    }
}
