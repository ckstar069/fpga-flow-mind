//! Phase 7 质量评估模块。
//!
//! 本模块包含：
//! - `models`：质量评估数据模型（QualityReport、QualityIssue、各维度报告等）。
//! - `reporter`：确定性质量报告生成器（`QualityReporter`）。
//! - `issue_builder`：共享的 `QualityIssue` 构造辅助函数。
//! - Batch B 形式化 evaluator：`stage_evaluator`、`evidence_evaluator`、
//!   `understanding_evaluator`、`view_evaluator`、`qa_evaluator`。
//!
//! 边界：
//! - 只读消费既有 evidence/understanding/view/qa 产物；
//! - **不含** Tauri command、UI；
//! - 不修改 evidence/understanding/view/qa 既有逻辑；
//! - 不接真实 LLM / OpenAI / Anthropic / api_key；
//! - 不写目标项目、不做 Vivado / synthesis / implementation / bitstream；
//! - 不输出 PASS/HOLD/正确/错误等审计裁决。

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
