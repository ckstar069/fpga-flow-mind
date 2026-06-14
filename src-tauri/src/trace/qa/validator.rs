use crate::evidence::models::EvidenceCollection;
use crate::trace::models::{
    GroundedAnswer, GroundedAnswerClaim, GroundedQaContext,
};
use crate::understanding::models::{ClaimConfidence, ImplementationUnderstanding};

/// 验证结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

/// Grounded Q&A 答案校验器。
///
/// 负责在返回前端之前检查 answer 结构、citation 绑定与内容安全。
pub struct GroundedQaValidator;

impl GroundedQaValidator {
    pub fn validate(
        answer: &GroundedAnswer,
        context: &GroundedQaContext,
    ) -> ValidationResult {
        let mut errors = Vec::new();

        Self::validate_basic(answer, &mut errors);
        Self::validate_claims(answer, &mut errors);
        Self::validate_citations(answer, context, &mut errors);
        Self::validate_content_safety(answer, &mut errors);

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
        }
    }

    fn validate_basic(answer: &GroundedAnswer, errors: &mut Vec<String>) {
        if answer.text.trim().is_empty() {
            errors.push("回答文本不能为空".to_string());
        }

        if !Self::is_valid_confidence(answer.confidence) {
            errors.push(format!("非法的 confidence: {:?}", answer.confidence));
        }
    }

    fn validate_claims(answer: &GroundedAnswer, errors: &mut Vec<String>) {
        for (i, claim) in answer.claims.iter().enumerate() {
            Self::validate_single_claim(claim, answer, i, errors);
        }
    }

    fn validate_single_claim(
        claim: &GroundedAnswerClaim,
        answer: &GroundedAnswer,
        index: usize,
        errors: &mut Vec<String>,
    ) {
        if claim.text.trim().is_empty() {
            errors.push(format!("claim[{}] 文本不能为空", index));
        }

        if !Self::is_valid_confidence(claim.confidence) {
            errors.push(format!("claim[{}] 非法 confidence: {:?}", index, claim.confidence));
        }

        match claim.confidence {
            ClaimConfidence::Unknown => {
                if !claim.citation_indices.is_empty() {
                    errors.push(format!(
                        "unknown claim[{}] 必须 citation_indices = []",
                        index
                    ));
                }
                if claim.reason.as_ref().map(|r| r.trim().is_empty()).unwrap_or(true) {
                    errors.push(format!("unknown claim[{}] 必须提供非空 reason", index));
                }
            }
            _ => {
                if claim.citation_indices.is_empty() {
                    errors.push(format!(
                        "非 unknown claim[{}] 必须至少包含一个 citation index",
                        index
                    ));
                }

                for &idx in &claim.citation_indices {
                    if idx >= answer.citations.len() {
                        errors.push(format!(
                            "claim[{}] citation index {} 越界（citations 长度 {}）",
                            index,
                            idx,
                            answer.citations.len()
                        ));
                    }
                }
            }
        }
    }

    fn validate_citations(
        answer: &GroundedAnswer,
        context: &GroundedQaContext,
        errors: &mut Vec<String>,
    ) {
        let known_evidence_ids: std::collections::HashSet<String> = context
            .evidence_collection
            .evidence_items
            .iter()
            .map(|e| e.evidence_id.clone())
            .collect();
        let known_claim_ids: std::collections::HashSet<String> = context
            .claims
            .iter()
            .map(|c| c.claim_id.clone())
            .collect();

        // 整体 confidence 非 unknown 时，citations 必须非空
        if answer.confidence != ClaimConfidence::Unknown && answer.citations.is_empty() {
            errors.push("非 unknown 回答必须包含至少一个 citation".to_string());
        }

        // unknown 回答 citations 必须为空
        if answer.confidence == ClaimConfidence::Unknown && !answer.citations.is_empty() {
            errors.push("unknown 回答不得伪造 citation".to_string());
        }

        for (i, citation) in answer.citations.iter().enumerate() {
            if citation.excerpt_summary.trim().is_empty() {
                errors.push(format!("citation[{}] excerpt_summary 不能为空", i));
            }

            let has_evidence = citation
                .evidence_id
                .as_ref()
                .map(|id| known_evidence_ids.contains(id))
                .unwrap_or(false);
            let has_claim = citation
                .claim_id
                .as_ref()
                .map(|id| known_claim_ids.contains(id))
                .unwrap_or(false);
            let has_location = citation.source_location.is_some();

            if !has_evidence && !has_claim && !has_location {
                errors.push(format!(
                    "citation[{}] 未引用任何有效 evidence_id / claim_id / source_location",
                    i
                ));
            }

            if let Some(id) = &citation.evidence_id {
                if !known_evidence_ids.contains(id) {
                    errors.push(format!(
                        "citation[{}] 引用不存在的 evidence_id: {}",
                        i, id
                    ));
                }
            }

            if let Some(id) = &citation.claim_id {
                if !known_claim_ids.contains(id) {
                    errors.push(format!("citation[{}] 引用不存在的 claim_id: {}", i, id));
                }
            }
        }
    }

    fn validate_content_safety(answer: &GroundedAnswer, errors: &mut Vec<String>) {
        let forbidden = ["PASS", "HOLD", "正确", "错误", "审计"];
        let text = answer.text.to_uppercase();
        for word in &forbidden {
            if text.contains(&word.to_uppercase()) {
                errors.push(format!("回答包含禁用审计用语: {}", word));
            }
        }

        for (i, claim) in answer.claims.iter().enumerate() {
            let claim_text = claim.text.to_uppercase();
            for word in &forbidden {
                if claim_text.contains(&word.to_uppercase()) {
                    errors.push(format!("claim[{}] 包含禁用审计用语: {}", i, word));
                }
            }
        }
    }

    fn is_valid_confidence(confidence: ClaimConfidence) -> bool {
        matches!(
            confidence,
            ClaimConfidence::Confirmed
                | ClaimConfidence::Supported
                | ClaimConfidence::Inferred
                | ClaimConfidence::Unknown
                | ClaimConfidence::Conflicting
        )
    }
}

/// 便捷函数：直接校验 answer 与原始输入（understanding + evidence）。
pub fn validate_answer_against_inputs(
    answer: &GroundedAnswer,
    understanding: &ImplementationUnderstanding,
    evidence_collection: &EvidenceCollection,
) -> ValidationResult {
    let context = GroundedQaContext {
        question: String::new(),
        stage_id: understanding.stage_id.clone(),
        selected_target: None,
        understanding_summary: understanding.summary.short.clone(),
        claims: understanding.claims.clone(),
        evidence_collection: evidence_collection.clone(),
        available_citations: vec![],
        relevant_claims: vec![],
        relevant_evidence: vec![],
        warnings: vec![],
    };
    GroundedQaValidator::validate(answer, &context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::models::{
        EvidenceCollection, EvidenceItem, EvidenceStats, EvidenceStrength, LineRange,
    };
    use crate::models::enums::{Language, SourceKind};
    use crate::trace::models::{
        GroundedAnswerClaim, GroundedAnswerCitation, GroundedQaContext, GroundedQaWarning,
        SourceLocation,
    };
    use crate::understanding::models::{
        ClaimCategory, ClaimConfidence, EvidenceRef, ImplementationClaim,
    };
    use std::collections::HashMap;

    fn make_context() -> GroundedQaContext {
        let evidence = EvidenceItem {
            evidence_id: "EV-L0-0001".to_string(),
            source_path: "/project/L0.v".to_string(),
            language: Language::Verilog,
            source_kind: SourceKind::Rtl,
            line_range: LineRange { start: 10, end: 20 },
            symbol: Some("data".to_string()),
            summary: "定义 8 bit 数据信号".to_string(),
            strength: EvidenceStrength::Direct,
        };
        let claim = ImplementationClaim {
            claim_id: "CL-L0-0001".to_string(),
            category: ClaimCategory::SignalDefinition,
            description: "数据位宽为 8 bit".to_string(),
            confidence: ClaimConfidence::Supported,
            evidence_refs: vec![EvidenceRef {
                evidence_id: "EV-L0-0001".to_string(),
                relevance: Some("支撑".to_string()),
            }],
            has_evidence_gap: false,
        };

        GroundedQaContext {
            question: "位宽是多少".to_string(),
            stage_id: "L0".to_string(),
            selected_target: None,
            understanding_summary: "L0 阶段".to_string(),
            claims: vec![claim],
            evidence_collection: EvidenceCollection {
                stage_id: "L0".to_string(),
                evidence_items: vec![evidence],
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
            available_citations: vec![GroundedAnswerCitation {
                index: 1,
                evidence_id: Some("EV-L0-0001".to_string()),
                claim_id: None,
                source_location: Some(SourceLocation {
                    source_path: "/project/L0.v".to_string(),
                    line_range: LineRange { start: 10, end: 20 },
                    evidence_id: Some("EV-L0-0001".to_string()),
                }),
                excerpt_summary: "定义 8 bit 数据信号".to_string(),
            }],
            relevant_claims: vec![],
            relevant_evidence: vec![],
            warnings: vec![],
        }
    }

    fn make_answer_with_citation() -> GroundedAnswer {
        GroundedAnswer {
            answer_id: "A-001".to_string(),
            generated_at: "2026-06-14T10:00:00Z".to_string(),
            text: "位宽为 8 bit".to_string(),
            claims: vec![GroundedAnswerClaim {
                text: "位宽为 8 bit".to_string(),
                confidence: ClaimConfidence::Supported,
                citation_indices: vec![0],
                reason: None,
            }],
            citations: vec![GroundedAnswerCitation {
                index: 1,
                evidence_id: Some("EV-L0-0001".to_string()),
                claim_id: None,
                source_location: Some(SourceLocation {
                    source_path: "/project/L0.v".to_string(),
                    line_range: LineRange { start: 10, end: 20 },
                    evidence_id: Some("EV-L0-0001".to_string()),
                }),
                excerpt_summary: "定义 8 bit 数据信号".to_string(),
            }],
            confidence: ClaimConfidence::Supported,
            warnings: vec![],
            provider: "mock".to_string(),
            is_degraded: true,
        }
    }

    fn make_unknown_answer() -> GroundedAnswer {
        GroundedAnswer {
            answer_id: "A-002".to_string(),
            generated_at: "2026-06-14T10:00:00Z".to_string(),
            text: "无法确定".to_string(),
            claims: vec![GroundedAnswerClaim {
                text: "无法确定位宽".to_string(),
                confidence: ClaimConfidence::Unknown,
                citation_indices: vec![],
                reason: Some("当前证据不足".to_string()),
            }],
            citations: vec![],
            confidence: ClaimConfidence::Unknown,
            warnings: vec![GroundedQaWarning {
                code: "evidence_gap".to_string(),
                message: "当前阶段证据不足以回答该问题".to_string(),
            }],
            provider: "mock".to_string(),
            is_degraded: true,
        }
    }

    #[test]
    fn valid_answer_with_citation_passes() {
        let context = make_context();
        let answer = make_answer_with_citation();
        let result = GroundedQaValidator::validate(&answer, &context);
        assert!(result.is_valid);
    }

    #[test]
    fn non_unknown_without_citation_fails() {
        let context = make_context();
        let mut answer = make_answer_with_citation();
        answer.citations.clear();
        answer.claims[0].citation_indices.clear();
        let result = GroundedQaValidator::validate(&answer, &context);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("citation")));
    }

    #[test]
    fn unknown_without_citation_passes() {
        let context = make_context();
        let answer = make_unknown_answer();
        let result = GroundedQaValidator::validate(&answer, &context);
        assert!(result.is_valid);
    }

    #[test]
    fn unknown_with_fabricated_citation_fails() {
        let context = make_context();
        let mut answer = make_unknown_answer();
        answer.citations = vec![GroundedAnswerCitation {
            index: 1,
            evidence_id: Some("EV-L0-0001".to_string()),
            claim_id: None,
            source_location: None,
            excerpt_summary: "伪造".to_string(),
        }];
        answer.claims[0].citation_indices = vec![0];
        let result = GroundedQaValidator::validate(&answer, &context);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("不得伪造")));
    }

    #[test]
    fn citation_index_out_of_bounds_fails() {
        let context = make_context();
        let mut answer = make_answer_with_citation();
        answer.claims[0].citation_indices = vec![99];
        let result = GroundedQaValidator::validate(&answer, &context);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("越界")));
    }

    #[test]
    fn citation_evidence_id_not_exist_fails() {
        let context = make_context();
        let mut answer = make_answer_with_citation();
        answer.citations[0].evidence_id = Some("EV-UNKNOWN".to_string());
        let result = GroundedQaValidator::validate(&answer, &context);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("不存在")));
    }

    #[test]
    fn empty_answer_text_fails() {
        let context = make_context();
        let mut answer = make_answer_with_citation();
        answer.text = "   ".to_string();
        let result = GroundedQaValidator::validate(&answer, &context);
        assert!(!result.is_valid);
    }

    #[test]
    fn audit_words_rejected() {
        let context = make_context();
        let mut answer = make_answer_with_citation();
        answer.text = "该设计正确".to_string();
        let result = GroundedQaValidator::validate(&answer, &context);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("正确")));
    }
}
