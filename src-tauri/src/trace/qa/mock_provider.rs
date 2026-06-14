use chrono::Utc;

use crate::trace::models::{
    ClaimConfidence, GroundedAnswer, GroundedAnswerClaim, GroundedAnswerCitation,
    GroundedQaContext, GroundedQaWarning,
};
use crate::trace::qa::provider::{GroundedQaError, GroundedQaProvider};

/// Phase 5 专用 MockProvider。
///
/// 确定性生成 `GroundedAnswer`，仅用于验证 UI 闭环与 citation 绑定，不调用任何真实 LLM。
pub struct MockProvider;

impl GroundedQaProvider for MockProvider {
    fn generate_answer(
        &self,
        context: &GroundedQaContext,
    ) -> Result<GroundedAnswer, GroundedQaError> {
        if context.question.trim().is_empty() {
            return Err(GroundedQaError::EmptyQuestion);
        }

        let q = context.question.to_lowercase();

        // 位宽相关问题：基于第一个可用 evidence/claim 回答
        if q.contains("位宽") || q.contains("width") {
            return Self::answer_with_citations(
                context,
                "根据当前 evidence，该信号/数据通路的位宽为 8 bit。",
                ClaimConfidence::Supported,
                "位宽为 8 bit",
            );
        }

        // 功能相关问题：基于 understanding summary
        if q.contains("做什么") || q.contains("功能") || q.contains("作用") {
            let text = format!(
                "根据 understanding summary，{}。",
                context.understanding_summary
            );
            return Self::answer_with_citations(
                context,
                &text,
                ClaimConfidence::Supported,
                &context.understanding_summary,
            );
        }

        // 无法匹配：返回 unknown，不伪造 citation
        Self::answer_unknown(context)
    }
}

impl MockProvider {
    fn answer_with_citations(
        context: &GroundedQaContext,
        text: &str,
        confidence: ClaimConfidence,
        claim_text: &str,
    ) -> Result<GroundedAnswer, GroundedQaError> {
        if context.available_citations.is_empty() {
            return Self::answer_unknown(context);
        }

        let citations: Vec<GroundedAnswerCitation> = context
            .available_citations
            .iter()
            .enumerate()
            .map(|(idx, c)| GroundedAnswerCitation {
                index: idx + 1,
                evidence_id: c.evidence_id.clone(),
                claim_id: c.claim_id.clone(),
                source_location: c.source_location.clone(),
                excerpt_summary: c.excerpt_summary.clone(),
            })
            .collect();

        let claim = GroundedAnswerClaim {
            text: claim_text.to_string(),
            confidence,
            citation_indices: (0..citations.len()).collect(),
            reason: None,
        };

        Ok(GroundedAnswer {
            answer_id: format!("A-{}", Utc::now().timestamp_millis()),
            generated_at: Utc::now().to_rfc3339(),
            text: text.to_string(),
            claims: vec![claim],
            citations,
            confidence,
            warnings: context.warnings.clone(),
            provider: "mock".to_string(),
            is_degraded: true,
        })
    }

    fn answer_unknown(context: &GroundedQaContext) -> Result<GroundedAnswer, GroundedQaError> {
        let mut warnings = context.warnings.clone();
        warnings.push(GroundedQaWarning {
            code: "evidence_gap".to_string(),
            message: "当前阶段证据不足以回答该问题".to_string(),
        });

        Ok(GroundedAnswer {
            answer_id: format!("A-{}", Utc::now().timestamp_millis()),
            generated_at: Utc::now().to_rfc3339(),
            text: "根据当前证据无法确定。".to_string(),
            claims: vec![GroundedAnswerClaim {
                text: "无法从当前证据中得出结论".to_string(),
                confidence: ClaimConfidence::Unknown,
                citation_indices: vec![],
                reason: Some("问题与当前阶段证据/理解不匹配，且无可用 citation".to_string()),
            }],
            citations: vec![],
            confidence: ClaimConfidence::Unknown,
            warnings,
            provider: "mock".to_string(),
            is_degraded: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::models::{
        EvidenceCollection, EvidenceItem, EvidenceStats, EvidenceStrength, LineRange,
    };
    use crate::models::enums::{Language, SourceKind};
    use crate::trace::models::GroundedQuestion;
    use crate::trace::qa::context_builder::GroundedQaContextBuilder;
    use crate::understanding::models::{
        ClaimCategory, ClaimConfidence, EvidenceRef, ImplementationClaim, ImplementationUnderstanding,
        StageSummary, UnderstandingStats,
    };
    use std::collections::HashMap;

    fn make_evidence(id: &str, summary: &str) -> EvidenceItem {
        EvidenceItem {
            evidence_id: id.to_string(),
            source_path: format!("/project/{}.v", id),
            language: Language::Verilog,
            source_kind: SourceKind::Rtl,
            line_range: LineRange { start: 10, end: 20 },
            symbol: Some("sample".to_string()),
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

    fn make_question(question: &str) -> GroundedQuestion {
        GroundedQuestion {
            question: question.to_string(),
            stage_id: "L0".to_string(),
            selected_target: None,
            understanding: ImplementationUnderstanding {
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
            },
            evidence_collection: EvidenceCollection {
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
            },
        }
    }

    #[test]
    fn mock_answer_width_question_with_citation() {
        let question = make_question("位宽是多少");
        let context = GroundedQaContextBuilder::build(&question, None, None);
        let provider = MockProvider;
        let answer = provider.generate_answer(&context).unwrap();

        assert_ne!(answer.confidence, ClaimConfidence::Unknown);
        assert!(!answer.citations.is_empty());
        assert_eq!(answer.claims[0].citation_indices.len(), answer.citations.len());
        assert!(answer.provider == "mock");
        assert!(answer.is_degraded);
    }

    #[test]
    fn mock_answer_function_question_with_citation() {
        let question = make_question("这个阶段做什么");
        let context = GroundedQaContextBuilder::build(&question, None, None);
        let provider = MockProvider;
        let answer = provider.generate_answer(&context).unwrap();

        assert_ne!(answer.confidence, ClaimConfidence::Unknown);
        assert!(!answer.citations.is_empty());
    }

    #[test]
    fn mock_answer_unknown_when_no_keyword_match() {
        let question = make_question("宇宙常数是多少");
        let context = GroundedQaContextBuilder::build(&question, None, None);
        let provider = MockProvider;
        let answer = provider.generate_answer(&context).unwrap();

        assert_eq!(answer.confidence, ClaimConfidence::Unknown);
        assert!(answer.citations.is_empty());
        assert!(answer.claims[0].citation_indices.is_empty());
        assert!(answer.claims[0].reason.is_some());
        assert!(answer.warnings.iter().any(|w| w.code == "evidence_gap"));
    }

    #[test]
    fn mock_answer_unknown_when_no_citations_available() {
        let mut question = make_question("位宽是多少");
        question.evidence_collection.evidence_items.clear();
        question.understanding.claims.clear();
        let context = GroundedQaContextBuilder::build(&question, None, None);
        let provider = MockProvider;
        let answer = provider.generate_answer(&context).unwrap();

        assert_eq!(answer.confidence, ClaimConfidence::Unknown);
        assert!(answer.citations.is_empty());
    }

    #[test]
    fn mock_empty_question_fails() {
        let mut question = make_question("位宽是多少");
        question.question = "   ".to_string();
        let context = GroundedQaContextBuilder::build(&question, None, None);
        let provider = MockProvider;
        let result = provider.generate_answer(&context);

        assert_eq!(result.unwrap_err(), GroundedQaError::EmptyQuestion);
    }
}
