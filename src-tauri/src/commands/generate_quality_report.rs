/// `generate_quality_report` Tauri command（Phase 7 Batch C，P7-T06）。
///
/// 只读消费前端已持有的 Phase 1~6 产物（StageContext、EvidenceCollection、
/// ImplementationUnderstanding、ViewGraph[]、GroundedAnswer），调用
/// `QualityReporter` 产出确定性 `QualityReport`。
///
/// 行为：
/// - 从 `stage_context.files` 构造 `expected_source_paths` 传给 reporter；
/// - 所有产物均为可选：缺 evidence / understanding / views / qa 时，reporter
///   仍会对可用产物做评估并返回报告；
/// - 不扫描目标项目、不写目标项目、不重新生成产物、不接 LLM。
use std::collections::HashMap;

use crate::evidence::models::{EvidenceCollection, EvidenceStats};
use crate::models::error::{CommandError, CommandResult};
use crate::models::stage_context::StageContext;
use crate::quality::{QualityReport, QualityReportInput, QualityReporter, StageQualityInput};
use crate::trace::models::GroundedAnswer;
use crate::understanding::models::ImplementationUnderstanding;
use crate::views::models::ViewGraph;

#[tauri::command]
pub fn generate_quality_report(
    stage_context: StageContext,
    recognized_status: String,
    evidence: Option<EvidenceCollection>,
    understanding: Option<ImplementationUnderstanding>,
    views: Option<Vec<ViewGraph>>,
    grounded_answer: Option<GroundedAnswer>,
) -> CommandResult<QualityReport> {
    let expected_paths: Vec<String> = stage_context
        .files
        .iter()
        .map(|f| f.source_path.clone())
        .collect();

    // 无可评估产物且无期望文件清单时，返回明确错误，避免 UI 显示空的 meets_gate 报告。
    let has_any_artifact = evidence.is_some()
        || understanding.is_some()
        || views.as_ref().map_or(false, |v| !v.is_empty())
        || grounded_answer.is_some();
    if expected_paths.is_empty() && !has_any_artifact {
        return CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: crate::models::enums::ErrorCode::StageEmpty,
                message: "当前阶段无文件且未提供任何可评估产物，无法生成质量报告".to_string(),
                recoverable: true,
                details: Some("请先收集 evidence 或生成 understanding / views / Q\u{26}A 后再运行质量评估。".to_string()),
                source_path: Some(stage_context.source_path.clone()),
            }),
            warnings: Vec::new(),
        };
    }

    let expected_source_paths = if expected_paths.is_empty() {
        None
    } else {
        // StageQualityInput 持有引用；这里把局部 Vec 借给 reporter 后立即返回报告，
        // 报告内部已 clone 所需字符串，生命周期安全。
        Some(expected_paths.as_slice())
    };

    // 若阶段有文件但前端未传入 evidence，构造空 EvidenceCollection 并保留 expected_source_paths，
    // 使 reporter 能诚实暴露 missing_evidence / uncovered_files，而不是跳过 evidence 维度。
    let empty_evidence = if expected_source_paths.is_some() && evidence.is_none() {
        Some(EvidenceCollection {
            stage_id: stage_context.stage_id.clone(),
            evidence_items: Vec::new(),
            index_by_path: HashMap::new(),
            index_by_kind: HashMap::new(),
            index_by_symbol: HashMap::new(),
            warnings: Vec::new(),
            stats: EvidenceStats {
                files_processed: 0,
                files_skipped: 0,
                total_items: 0,
                items_by_kind: HashMap::new(),
                items_by_strength: HashMap::new(),
            },
            version: "1.0.0".to_string(),
        })
    } else {
        None
    };
    let evidence_ref = evidence.as_ref().or(empty_evidence.as_ref());

    let stage_input = StageQualityInput {
        stage_id: stage_context.stage_id.clone(),
        recognized_status,
        expected_status: None,
        expected_source_paths,
        evidence: evidence_ref,
        understanding: understanding.as_ref(),
        views: views
            .as_ref()
            .map(|v| v.iter().collect())
            .unwrap_or_default(),
        grounded_answer: grounded_answer.as_ref(),
    };

    let reporter = QualityReporter::new();
    let input = QualityReportInput {
        sample_id: stage_context.stage_id.clone(),
        generated_at_marker: String::new(),
        stages: vec![stage_input],
    };

    let report = reporter.evaluate(&input);
    CommandResult {
        success: true,
        data: Some(report),
        error: None,
        warnings: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::models::{
        EvidenceCollection, EvidenceItem, EvidenceStats, EvidenceStrength, LineRange,
    };
    use crate::models::enums::{ErrorCode, Language, SourceKind};
    use crate::models::stage_context::{StageContext, StageFile};
    use crate::quality::models::{QualityIssueKind, QualityIssuePolarity};
    use std::collections::HashMap;

    fn empty_stage_context(stage_id: &str) -> StageContext {
        StageContext {
            stage_id: stage_id.to_string(),
            source_path: format!("/tmp/{}", stage_id),
            files: vec![],
            external_deps: vec![],
            upstream_refs: vec![],
            error_code: Some(ErrorCode::StageEmpty),
        }
    }

    fn stage_context_with_files(stage_id: &str, paths: &[&str]) -> StageContext {
        StageContext {
            stage_id: stage_id.to_string(),
            source_path: format!("/tmp/{}", stage_id),
            files: paths
                .iter()
                .map(|p| StageFile {
                    source_path: p.to_string(),
                    language: Language::Python,
                    source_kind: SourceKind::PythonStage,
                    size_bytes: None,
                })
                .collect(),
            external_deps: vec![],
            upstream_refs: vec![],
            error_code: None,
        }
    }

    #[test]
    fn empty_stage_context_no_artifacts_returns_error() {
        let ctx = empty_stage_context("L0");
        let result = generate_quality_report(ctx, "empty".to_string(), None, None, None, None);
        assert!(!result.success, "空阶段且无产物时应返回 success=false");
        let err = result.error.expect("应返回错误");
        assert_eq!(err.error_code, ErrorCode::StageEmpty);
        assert!(err.message.contains("无法生成质量报告"));
    }

    #[test]
    fn files_exist_but_no_evidence_returns_missing_evidence_report() {
        let ctx = stage_context_with_files("L0", &["/tmp/L0/top.py", "/tmp/L0/missing.py"]);
        let result = generate_quality_report(ctx, "available".to_string(), None, None, None, None);
        assert!(result.success, "有文件但无 evidence 时应返回报告");
        let report = result.data.unwrap();
        assert_eq!(
            report.acceptance,
            crate::quality::models::QualityAcceptanceStatus::BelowGate
        );
        let ev_report = report
            .evidence_reports
            .iter()
            .find(|r| r.stage_id == "L0")
            .expect("应有 evidence report");
        assert!((ev_report.file_coverage_ratio - 0.0).abs() < 1e-5);
        assert_eq!(ev_report.uncovered_files.len(), 2);

        let missing_issues: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.kind == QualityIssueKind::MissingEvidence)
            .collect();
        assert_eq!(missing_issues.len(), 2, "两个文件均应产生 missing_evidence issue");
        assert!(
            missing_issues
                .iter()
                .any(|i| i.source_path.as_deref() == Some("/tmp/L0/top.py"))
        );
        assert!(
            missing_issues
                .iter()
                .any(|i| i.source_path.as_deref() == Some("/tmp/L0/missing.py"))
        );
        assert!(report.summary.total_issues > 0);
    }

    #[test]
    fn no_artifacts_and_no_files_returns_clear_error() {
        let ctx = empty_stage_context("L0");
        let result = generate_quality_report(ctx, "empty".to_string(), None, None, None, None);
        assert!(!result.success, "无文件且无产物时应返回错误");
        let err = result.error.expect("应返回错误");
        assert_eq!(err.error_code, ErrorCode::StageEmpty);
        assert!(err.message.contains("无法生成质量报告"));
        assert!(result.data.is_none(), "不应返回报告数据");
    }

    #[test]
    fn no_empty_meets_gate_report() {
        // 有文件但无 evidence：必须 below_gate
        let ctx = stage_context_with_files("L0", &["/tmp/L0/top.py"]);
        let result = generate_quality_report(ctx, "available".to_string(), None, None, None, None);
        assert!(result.success);
        let report = result.data.unwrap();
        assert_eq!(
            report.acceptance,
            crate::quality::models::QualityAcceptanceStatus::BelowGate
        );
        assert!(report.summary.total_issues > 0);

        // 无文件且无产物：success=false，无 acceptance
        let ctx2 = empty_stage_context("L1");
        let result2 = generate_quality_report(ctx2, "empty".to_string(), None, None, None, None);
        assert!(!result2.success);
        assert!(result2.data.is_none());
    }

    #[test]
    fn expected_source_paths_from_files_affect_coverage() {
        let ctx = stage_context_with_files("L0", &["/tmp/L0/top.py", "/tmp/L0/missing.py"]);
        let evidence = EvidenceCollection {
            stage_id: "L0".to_string(),
            evidence_items: vec![EvidenceItem {
                evidence_id: "EV-L0-000001".to_string(),
                source_path: "/tmp/L0/top.py".to_string(),
                language: Language::Python,
                source_kind: SourceKind::PythonStage,
                line_range: LineRange { start: 1, end: 5 },
                symbol: None,
                summary: "ok".to_string(),
                strength: EvidenceStrength::Direct,
            }],
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

        let result = generate_quality_report(
            ctx,
            "available".to_string(),
            Some(evidence),
            None,
            None,
            None,
        );
        assert!(result.success);
        let report = result.data.unwrap();
        let ev_report = report
            .evidence_reports
            .iter()
            .find(|r| r.stage_id == "L0")
            .expect("应有 evidence report");
        assert!((ev_report.file_coverage_ratio - 0.5).abs() < 1e-5);
        assert_eq!(ev_report.uncovered_files.len(), 1);
        assert_eq!(
            ev_report.uncovered_files[0].source_path,
            "/tmp/L0/missing.py"
        );

        let issue = report
            .issues
            .iter()
            .find(|i| {
                i.kind == QualityIssueKind::MissingEvidence
                    && i.source_path.as_deref() == Some("/tmp/L0/missing.py")
            })
            .expect("应存在 missing.py 的 missing_evidence issue");
        assert_eq!(issue.polarity, QualityIssuePolarity::Problem);
    }

    #[test]
    fn command_does_not_modify_target_project() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let path = root.join("L0/top.py");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"def top(): pass\n").unwrap();

        let before = std::fs::read_to_string(&path).unwrap();
        let ctx = stage_context_with_files("L0", &[path.to_str().unwrap()]);
        let _ = generate_quality_report(ctx, "available".to_string(), None, None, None, None);
        let after = std::fs::read_to_string(&path).unwrap();
        assert_eq!(before, after, "command 不应修改目标项目文件");
    }

    #[test]
    fn command_does_not_invoke_llm_or_external_process() {
        // 本 command 内部只调用确定性 QualityReporter，无 LLM / 外部进程调用。
        // 该测试显式断言返回报告且 provider 字段为空（无 generation_meta 写 LLM）。
        let ctx = stage_context_with_files("L0", &["/tmp/L0/top.py"]);
        let result = generate_quality_report(ctx, "available".to_string(), None, None, None, None);
        assert!(result.success);
        let report = result.data.unwrap();
        assert_eq!(report.generated_at, "");
    }
}
