/// `ask_grounded_question` Tauri command（Phase 5 Batch B 壳层）
///
/// 当前实现仅返回明确的未实现错误，不生成任何伪造 answer、citation，不调用
/// Provider / LLM / Validator。真正的 Q&A 逻辑在 Batch D 实现。

use crate::models::enums::ErrorCode;
use crate::models::error::{CommandError, CommandResult};
use crate::trace::models::{GroundedAnswer, GroundedQuestion};

#[tauri::command]
pub fn ask_grounded_question(question: GroundedQuestion) -> CommandResult<GroundedAnswer> {
    let _ = question;
    CommandResult {
        success: false,
        data: None,
        error: Some(CommandError {
            error_code: ErrorCode::QaGenerationFailed,
            message: "Grounded Q&A 尚未实现，将在 Phase 5 后续 Batch 提供".to_string(),
            recoverable: true,
            details: Some(
                "当前 Batch B 仅暴露 command 壳层。MockProvider 与 Validator 将在 Batch D 实现。"
                    .to_string(),
            ),
            source_path: None,
        }),
        warnings: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::evidence::models::EvidenceCollection;
    use crate::trace::models::SelectedTraceTarget;
    use crate::understanding::models::{
        ImplementationUnderstanding, StageSummary, UnderstandingStats,
    };

    fn make_empty_question() -> GroundedQuestion {
        GroundedQuestion {
            question: "测试问题".to_string(),
            stage_id: "L0".to_string(),
            selected_target: None,
            understanding: ImplementationUnderstanding {
                stage_id: "L0".to_string(),
                version: "3.0.0".to_string(),
                summary: StageSummary {
                    short: "测试".to_string(),
                    detailed: "测试".to_string(),
                },
                claims: vec![],
                module_summaries: vec![],
                signal_summaries: vec![],
                interface_summaries: vec![],
                processing_steps: vec![],
                unknowns: vec![],
                evidence_gaps: vec![],
                generation_meta: crate::understanding::models::GenerationMeta {
                    provider: "mock".to_string(),
                    generated_at: "2026-06-14T10:00:00Z".to_string(),
                    input_evidence_count: 0,
                    generation_time_ms: 0,
                    is_degraded: true,
                },
                stats: UnderstandingStats {
                    total_claims: 0,
                    claims_by_confidence: HashMap::new(),
                    claims_by_category: HashMap::new(),
                    module_count: 0,
                    signal_count: 0,
                    interface_count: 0,
                    processing_step_count: 0,
                    unknown_count: 0,
                    evidence_gap_count: 0,
                },
            },
            evidence_collection: EvidenceCollection {
                stage_id: "L0".to_string(),
                evidence_items: vec![],
                index_by_path: HashMap::new(),
                index_by_kind: HashMap::new(),
                index_by_symbol: HashMap::new(),
                warnings: vec![],
                stats: crate::evidence::models::EvidenceStats {
                    files_processed: 0,
                    files_skipped: 0,
                    total_items: 0,
                    items_by_kind: HashMap::new(),
                    items_by_strength: HashMap::new(),
                },
                version: "1.0.0".to_string(),
            },
        }
    }

    // ─── command 测试 ────────────────────────────────────────────────

    #[test]
    fn cmd_ask_question_returns_not_implemented() {
        let question = make_empty_question();
        let result = ask_grounded_question(question);

        assert!(!result.success);
        assert!(result.data.is_none());
        let err = result.error.unwrap();
        assert_eq!(err.error_code, ErrorCode::QaGenerationFailed);
        assert!(err.recoverable);
        assert!(err.details.unwrap().contains("Batch D"));
    }

    #[test]
    fn cmd_ask_question_does_not_fabricate_citation() {
        let question = make_empty_question();
        let result = ask_grounded_question(question);

        assert!(result.data.is_none());
        assert!(result.error.is_some());
    }

    #[test]
    fn cmd_ask_question_with_target_still_not_implemented() {
        let mut question = make_empty_question();
        question.selected_target = Some(SelectedTraceTarget::Claim {
            claim_id: "CL-L0-000001".to_string(),
        });

        let result = ask_grounded_question(question);

        assert!(!result.success);
        assert_eq!(result.error.unwrap().error_code, ErrorCode::QaGenerationFailed);
    }

    #[test]
    fn cmd_ask_question_empty_evidence_collection_still_not_implemented() {
        let question = make_empty_question();
        let result = ask_grounded_question(question);

        assert!(!result.success);
        assert!(result.data.is_none());
    }
}
