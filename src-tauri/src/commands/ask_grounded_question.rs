/// `ask_grounded_question` Tauri command（Phase 5 Batch D mock 闭环）
///
/// 基于当前阶段已有的 understanding + evidence（以及可选的 views / resolved traces）生成
/// 确定性 mock 回答，不调用任何真实 LLM / 外部 API，不访问目标项目文件系统。

use crate::models::enums::ErrorCode;
use crate::models::error::{CommandError, CommandResult};
use crate::trace::models::{GroundedAnswer, GroundedQuestion};
use crate::trace::qa::context_builder::GroundedQaContextBuilder;
use crate::trace::qa::mock_provider::MockProvider;
use crate::trace::qa::provider::GroundedQaProvider;
use crate::trace::qa::validator::GroundedQaValidator;
use crate::views::models::ViewGraph;

#[tauri::command]
pub fn ask_grounded_question(
    question: GroundedQuestion,
    views: Option<Vec<ViewGraph>>,
    resolved_traces: Option<Vec<crate::trace::models::TraceRefResolved>>,
) -> CommandResult<GroundedAnswer> {
    // 1. 问题非空校验
    if question.question.trim().is_empty() {
        return CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::QaValidationFailed,
                message: "问题不能为空".to_string(),
                recoverable: true,
                details: Some("请输入非空问题后再提问".to_string()),
                source_path: None,
            }),
            warnings: Vec::new(),
        };
    }

    // 2. 无 evidence / understanding 时返回可恢复错误（按 active 文档：无输入上下文则无法回答）
    if question.evidence_collection.evidence_items.is_empty()
        && question.understanding.claims.is_empty()
    {
        return CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::QaGenerationFailed,
                message: "当前阶段缺少 evidence 与 understanding，无法回答".to_string(),
                recoverable: true,
                details: Some("请先收集证据并生成理解后再提问".to_string()),
                source_path: None,
            }),
            warnings: Vec::new(),
        };
    }

    // 3. 构建上下文
    let context = GroundedQaContextBuilder::build(
        &question,
        views.as_deref(),
        resolved_traces.as_deref(),
    );

    // 4. MockProvider 生成 answer
    let provider = MockProvider;
    let answer = match provider.generate_answer(&context) {
        Ok(ans) => ans,
        Err(err) => {
            return CommandResult {
                success: false,
                data: None,
                error: Some(CommandError {
                    error_code: ErrorCode::QaGenerationFailed,
                    message: err.to_string(),
                    recoverable: true,
                    details: Some("MockProvider 生成失败".to_string()),
                    source_path: None,
                }),
                warnings: Vec::new(),
            }
        }
    };

    // 5. Validator 校验
    let validation = GroundedQaValidator::validate(&answer, &context);
    if !validation.is_valid {
        return CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::QaValidationFailed,
                message: "回答未通过 grounded 校验".to_string(),
                recoverable: true,
                details: Some(validation.errors.join("; ")),
                source_path: None,
            }),
            warnings: Vec::new(),
        };
    }

    CommandResult {
        success: true,
        data: Some(answer),
        error: None,
        warnings: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::evidence::models::{
        EvidenceCollection, EvidenceItem, EvidenceStats, EvidenceStrength, LineRange,
    };
    use crate::models::enums::{Language, SourceKind};
    use crate::trace::models::SelectedTraceTarget;
    use crate::understanding::models::{
        ClaimCategory, ClaimConfidence, EvidenceRef, ImplementationClaim, ImplementationUnderstanding,
        StageSummary, UnderstandingStats,
    };

    fn make_evidence(id: &str, summary: &str) -> EvidenceItem {
        EvidenceItem {
            evidence_id: id.to_string(),
            source_path: format!("/project/{}.v", id),
            language: Language::Verilog,
            source_kind: SourceKind::Rtl,
            line_range: LineRange { start: 10, end: 20 },
            symbol: Some("data".to_string()),
            summary: summary.to_string(),
            strength: EvidenceStrength::Direct,
        }
    }

    fn make_claim(id: &str, description: &str) -> ImplementationClaim {
        ImplementationClaim {
            claim_id: id.to_string(),
            category: ClaimCategory::SignalDefinition,
            description: description.to_string(),
            confidence: ClaimConfidence::Supported,
            evidence_refs: vec![EvidenceRef {
                evidence_id: "EV-L0-0001".to_string(),
                relevance: Some("支撑".to_string()),
            }],
            has_evidence_gap: false,
        }
    }

    fn make_understanding() -> ImplementationUnderstanding {
        ImplementationUnderstanding {
            stage_id: "L0".to_string(),
            version: "3.0.0".to_string(),
            summary: StageSummary {
                short: "L0 阶段".to_string(),
                detailed: "L0 阶段实现数据通路".to_string(),
            },
            claims: vec![make_claim("CL-L0-0001", "数据位宽为 8 bit")],
            module_summaries: vec![],
            signal_summaries: vec![],
            interface_summaries: vec![],
            processing_steps: vec![],
            unknowns: vec![],
            evidence_gaps: vec![],
            generation_meta: crate::understanding::models::GenerationMeta {
                provider: "mock".to_string(),
                generated_at: "2026-06-14T10:00:00Z".to_string(),
                input_evidence_count: 1,
                generation_time_ms: 0,
                is_degraded: true,
            },
            stats: UnderstandingStats {
                total_claims: 1,
                claims_by_confidence: HashMap::new(),
                claims_by_category: HashMap::new(),
                module_count: 0,
                signal_count: 0,
                interface_count: 0,
                processing_step_count: 0,
                unknown_count: 0,
                evidence_gap_count: 0,
            },
        }
    }

    fn make_evidence_collection() -> EvidenceCollection {
        EvidenceCollection {
            stage_id: "L0".to_string(),
            evidence_items: vec![make_evidence("EV-L0-0001", "定义 8 bit 数据信号")],
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
        }
    }

    fn make_question(question: &str) -> GroundedQuestion {
        GroundedQuestion {
            question: question.to_string(),
            stage_id: "L0".to_string(),
            selected_target: None,
            understanding: make_understanding(),
            evidence_collection: make_evidence_collection(),
        }
    }

    #[test]
    fn cmd_ask_question_returns_success_with_citation() {
        let question = make_question("位宽是多少");
        let result = ask_grounded_question(question, None, None);

        assert!(result.success);
        let answer = result.data.unwrap();
        assert_ne!(answer.confidence, ClaimConfidence::Unknown);
        assert!(!answer.citations.is_empty());
        assert!(!answer.claims.is_empty());
        assert!(answer.claims[0].citation_indices.len() > 0);
    }

    #[test]
    fn cmd_ask_question_unknown_answer_without_fabricated_citation() {
        let question = make_question("宇宙常数是多少");
        let result = ask_grounded_question(question, None, None);

        assert!(result.success);
        let answer = result.data.unwrap();
        assert_eq!(answer.confidence, ClaimConfidence::Unknown);
        assert!(answer.citations.is_empty());
        assert!(answer.claims[0].citation_indices.is_empty());
        assert!(answer.warnings.iter().any(|w| w.code == "evidence_gap"));
    }

    #[test]
    fn cmd_ask_question_empty_question_fails() {
        let mut question = make_question("位宽是多少");
        question.question = "   ".to_string();
        let result = ask_grounded_question(question, None, None);

        assert!(!result.success);
        assert_eq!(result.error.unwrap().error_code, ErrorCode::QaValidationFailed);
    }

    #[test]
    fn cmd_ask_question_no_evidence_no_understanding_fails() {
        let mut question = make_question("位宽是多少");
        question.evidence_collection.evidence_items.clear();
        question.understanding.claims.clear();
        let result = ask_grounded_question(question, None, None);

        assert!(!result.success);
        assert_eq!(result.error.unwrap().error_code, ErrorCode::QaGenerationFailed);
    }

    #[test]
    fn cmd_ask_question_with_target_returns_success() {
        let mut question = make_question("位宽是多少");
        question.selected_target = Some(SelectedTraceTarget::Claim {
            claim_id: "CL-L0-0001".to_string(),
        });
        let result = ask_grounded_question(question, None, None);

        assert!(result.success);
        assert!(result.data.is_some());
    }

    #[test]
    fn cmd_ask_question_does_not_fabricate_citation() {
        let question = make_question("位宽是多少");
        let result = ask_grounded_question(question, None, None);

        let answer = result.data.unwrap();
        for claim in &answer.claims {
            if claim.confidence == ClaimConfidence::Unknown {
                assert!(claim.citation_indices.is_empty());
            }
        }
        for (i, idx) in answer.claims.iter().flat_map(|c| c.citation_indices.iter()).enumerate() {
            assert!(*idx < answer.citations.len(), "citation index {} (claim #{}) 越界", idx, i);
        }
    }

    #[test]
    fn cmd_ask_question_empty_evidence_collection_returns_unknown() {
        let mut question = make_question("位宽是多少");
        question.evidence_collection.evidence_items.clear();
        // understanding 仍有 claim，可继续生成
        let result = ask_grounded_question(question, None, None);

        // evidence 为空时 MockProvider 对位宽问题仍可基于 claim 回答
        assert!(result.success);
    }
}
