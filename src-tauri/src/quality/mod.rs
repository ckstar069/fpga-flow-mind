//! Phase 7 质量评估模块。
//!
//! Batch A（P7-T01~P7-T02）范围：质量数据模型（`models`）与确定性
//! `QualityReporter`（`reporter`）。
//!
//! 边界（与 `docs/planning/phase-7-implementation-plan.md` Batch A 一致）：
//! - 仅新增本评估层，只读消费既有 evidence/understanding/view/qa 产物；
//! - **不含** Tauri command、UI、evaluator Batch B/C/D/E；
//! - 不修改 evidence/understanding/view/qa 既有逻辑；
//! - 不接真实 LLM、不写目标项目。

pub mod evidence_evaluator;
pub mod issue_builder;
pub mod models;
pub mod qa_evaluator;
pub mod reporter;
pub mod stage_evaluator;
pub mod understanding_evaluator;
pub mod view_evaluator;

pub use models::{
    ArtifactKind, DetectionMethod, EvidenceQualityReport, ExpectedStageEntry, FileTypeDistribution,
    IssueStatus, MetricSnapshot, QaEvaluationQuestion, QaEvaluationQuestionSet, QaExpectedAnswerability,
    QaQualityReport, QualityAcceptanceStatus, QualityIssue, QualityIssueKind, QualityIssuePolarity,
    QualityReport, QualityRunSummary, QualitySeverity, RealProjectSample, SampleScaleMetrics,
    StageEvaluationTarget, SummaryQuality, UnderstandingQualityReport, UncoveredFile, ViewQualityReport,
};
pub use issue_builder::{
    is_label_sane, is_noisy, make_guardrail, make_issue, sanitize_scope, trace_ref_ok,
};
pub use reporter::{QualityReportInput, QualityReporter, StageQualityInput};
pub use stage_evaluator::{StageEvaluator, StageEvaluatorInput};
pub use evidence_evaluator::{EvidenceEvaluator, EvidenceEvaluatorInput};
pub use understanding_evaluator::{UnderstandingEvaluator, UnderstandingEvaluatorInput};
pub use view_evaluator::{ViewEvaluator, ViewEvaluatorInput};
pub use qa_evaluator::{QaEvaluator, QaEvaluatorInput};
