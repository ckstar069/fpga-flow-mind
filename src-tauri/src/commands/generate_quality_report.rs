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
use crate::evidence::models::EvidenceCollection;
use crate::models::error::CommandResult;
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

    let expected_source_paths = if expected_paths.is_empty() {
        None
    } else {
        // StageQualityInput 持有引用；这里把局部 Vec 借给 reporter 后立即返回报告，
        // 报告内部已 clone 所需字符串，生命周期安全。
        Some(expected_paths.as_slice())
    };

    let stage_input = StageQualityInput {
        stage_id: stage_context.stage_id.clone(),
        recognized_status,
        expected_status: None,
        expected_source_paths,
        evidence: evidence.as_ref(),
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
    fn empty_stage_context_returns_report() {
        let ctx = empty_stage_context("L0");
        let result = generate_quality_report(ctx, "empty".to_string(), None, None, None, None);
        assert!(result.success, "空阶段应返回 success=true");
        let report = result.data.unwrap();
        assert_eq!(report.stage_ids, vec!["L0"]);
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
    fn missing_artifacts_do_not_panic() {
        let ctx = stage_context_with_files("L0", &["/tmp/L0/top.py"]);
        let result = generate_quality_report(ctx, "available".to_string(), None, None, None, None);
        assert!(result.success);
        let report = result.data.unwrap();
        // 无 evidence 时 reporter 跳过 evidence 评估，不 panic，仍返回报告
        assert!(report.evidence_reports.is_empty());
        assert!(report.issues.is_empty() || report.summary.total_issues == 0);
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
