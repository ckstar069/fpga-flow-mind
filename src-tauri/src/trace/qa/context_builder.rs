use std::collections::HashSet;

use crate::trace::models::{
    GroundedAnswerCitation, GroundedQaContext, GroundedQaWarning, GroundedQuestion,
};
use crate::understanding::models::ImplementationClaim;
use crate::views::models::ViewGraph;

/// 构建 Grounded Q&A 所需的确定性上下文。
///
/// 输入包含完整 understanding / evidence / 可选 views / 可选 trace resolved，输出只保留
/// 回答所依赖的：问题、阶段、已选目标、可用 citation、相关 claim、相关 evidence、warnings。
pub struct GroundedQaContextBuilder;

impl GroundedQaContextBuilder {
    pub fn build(
        question: &GroundedQuestion,
        views: Option<&[ViewGraph]>,
        resolved_traces: Option<&[crate::trace::models::TraceRefResolved]>,
    ) -> GroundedQaContext {
        let available_citations = Self::build_available_citations(question, views, resolved_traces);
        let relevant_claims = Self::filter_relevant_claims(question);
        let relevant_evidence = Self::filter_relevant_evidence(question);
        let warnings = Self::build_warnings(question, &available_citations);

        GroundedQaContext {
            question: question.question.clone(),
            stage_id: question.stage_id.clone(),
            selected_target: question.selected_target.clone(),
            understanding_summary: question.understanding.summary.short.clone(),
            claims: question.understanding.claims.clone(),
            evidence_collection: question.evidence_collection.clone(),
            available_citations,
            relevant_claims,
            relevant_evidence,
            warnings,
        }
    }

    /// 构建可用 citation 列表：优先从 resolved trace、其次从 evidence、claim 提取。
    fn build_available_citations(
        question: &GroundedQuestion,
        _views: Option<&[ViewGraph]>,
        resolved_traces: Option<&[crate::trace::models::TraceRefResolved]>,
    ) -> Vec<GroundedAnswerCitation> {
        let mut citations: Vec<GroundedAnswerCitation> = Vec::new();
        let mut used_ids: HashSet<String> = HashSet::new();

        // 1. 如果存在 resolved traces，优先把其中的 claim / evidence 作为 citation
        if let Some(traces) = resolved_traces {
            for trace in traces {
                if let Some(claim) = &trace.claim {
                    if used_ids.insert(claim.claim_id.clone()) {
                        citations.push(GroundedAnswerCitation {
                            index: citations.len() + 1,
                            evidence_id: None,
                            claim_id: Some(claim.claim_id.clone()),
                            source_location: None,
                            excerpt_summary: format!(
                                "claim {} ({}): {}",
                                claim.claim_id,
                                format_category(&claim.category),
                                Self::truncate(&claim.description, 80)
                            ),
                        });
                    }
                }
                if let Some(evidence) = &trace.evidence {
                    if used_ids.insert(evidence.evidence_id.clone()) {
                        citations.push(GroundedAnswerCitation {
                            index: citations.len() + 1,
                            evidence_id: Some(evidence.evidence_id.clone()),
                            claim_id: None,
                            source_location: Some(crate::trace::models::SourceLocation {
                                source_path: evidence.source_path.clone(),
                                line_range: evidence.line_range,
                                evidence_id: Some(evidence.evidence_id.clone()),
                            }),
                            excerpt_summary: format!(
                                "evidence {}: {} [{} 行 {}–{}]",
                                evidence.evidence_id,
                                Self::truncate(&evidence.summary, 60),
                                evidence.source_path.split('/').next_back().unwrap_or(&evidence.source_path),
                                evidence.line_range.start,
                                evidence.line_range.end
                            ),
                        });
                    }
                }
            }
        }

        // 2. 补充 evidence_collection 中的 evidence（去重）
        for item in &question.evidence_collection.evidence_items {
            if used_ids.insert(item.evidence_id.clone()) {
                citations.push(GroundedAnswerCitation {
                    index: citations.len() + 1,
                    evidence_id: Some(item.evidence_id.clone()),
                    claim_id: None,
                    source_location: Some(crate::trace::models::SourceLocation {
                        source_path: item.source_path.clone(),
                        line_range: item.line_range,
                        evidence_id: Some(item.evidence_id.clone()),
                    }),
                    excerpt_summary: format!(
                        "evidence {}: {} [{} 行 {}–{}]",
                        item.evidence_id,
                        Self::truncate(&item.summary, 60),
                        item.source_path.split('/').next_back().unwrap_or(&item.source_path),
                        item.line_range.start,
                        item.line_range.end
                    ),
                });
            }
        }

        // 3. 补充 understanding.claims（去重）
        for claim in &question.understanding.claims {
            if used_ids.insert(claim.claim_id.clone()) {
                citations.push(GroundedAnswerCitation {
                    index: citations.len() + 1,
                    evidence_id: None,
                    claim_id: Some(claim.claim_id.clone()),
                    source_location: None,
                    excerpt_summary: format!(
                        "claim {} ({}): {}",
                        claim.claim_id,
                        format_category(&claim.category),
                        Self::truncate(&claim.description, 80)
                    ),
                });
            }
        }

        let _ = _views; // views 已通过 resolved_traces 间接提供 citation，此处保留扩展点
        citations
    }

    /// 按关键词筛选可能相关的 claim（简单确定性匹配）。
    fn filter_relevant_claims(question: &GroundedQuestion) -> Vec<ImplementationClaim> {
        let q = question.question.to_lowercase();
        question
            .understanding
            .claims
            .iter()
            .filter(|c| {
                let desc = c.description.to_lowercase();
                let cat = format_category(&c.category).to_lowercase();
                desc.contains(&q)
                    || q.contains(&desc)
                    || cat.contains(&q)
                    || q.contains(&c.claim_id.to_lowercase())
                    || Self::has_shared_word(&q, &desc)
            })
            .cloned()
            .collect()
    }

    /// 检查两个字符串是否共享长度 ≥2 的词。
    /// 对 ASCII 按非字母数字分词；对 CJK 按单字分词后检查是否有连续双字匹配。
    fn has_shared_word(a: &str, b: &str) -> bool {
        let tokens_a = Self::tokenize(a);
        let tokens_b = Self::tokenize(b);
        tokens_a.iter().any(|ta| tokens_b.iter().any(|tb| ta == tb))
    }

    fn tokenize(s: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let lower = s.to_lowercase();
        let chars: Vec<char> = lower.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c.is_ascii_alphanumeric() {
                let start = i;
                while i < chars.len() && chars[i].is_ascii_alphanumeric() {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                if word.len() >= 2 {
                    tokens.push(word);
                }
            } else if is_cjk(c) {
                // CJK：生成连续双字 token
                if i + 1 < chars.len() && is_cjk(chars[i + 1]) {
                    let token: String = chars[i..i + 2].iter().collect();
                    tokens.push(token);
                }
                i += 1;
            } else {
                i += 1;
            }
        }
        tokens
    }

    /// 按关键词筛选可能相关的 evidence（简单确定性匹配）。
    fn filter_relevant_evidence(question: &GroundedQuestion) -> Vec<crate::evidence::models::EvidenceItem> {
        let q = question.question.to_lowercase();
        question
            .evidence_collection
            .evidence_items
            .iter()
            .filter(|e| {
                let summary = e.summary.to_lowercase();
                let path = e.source_path.to_lowercase();
                summary.contains(&q)
                    || q.contains(&summary)
                    || path.contains(&q)
                    || q.contains(&path)
                    || e.symbol.as_ref().map(|s| {
                        let sym = s.to_lowercase();
                        sym.contains(&q) || q.contains(&sym) || Self::has_shared_word(&q, &sym)
                    }).unwrap_or(false)
                    || q.contains(&e.evidence_id.to_lowercase())
                    || Self::has_shared_word(&q, &summary)
            })
            .cloned()
            .collect()
    }

    fn build_warnings(
        question: &GroundedQuestion,
        available_citations: &[GroundedAnswerCitation],
    ) -> Vec<GroundedQaWarning> {
        let mut warnings = Vec::new();

        if question.evidence_collection.evidence_items.is_empty() {
            warnings.push(GroundedQaWarning {
                code: "evidence_gap".to_string(),
                message: "当前阶段未收集到 evidence，回答只能基于 understanding summary".to_string(),
            });
        }

        if question.understanding.claims.is_empty() {
            warnings.push(GroundedQaWarning {
                code: "no_claims".to_string(),
                message: "当前 understanding 未生成 claim，回答置信度受限".to_string(),
            });
        }

        if available_citations.is_empty() {
            warnings.push(GroundedQaWarning {
                code: "no_citations".to_string(),
                message: "当前上下文无可用的 evidence/claim 引用".to_string(),
            });
        }

        if question.question.trim().len() < 4 {
            warnings.push(GroundedQaWarning {
                code: "short_question".to_string(),
                message: "问题过短，回答可能不准确".to_string(),
            });
        }

        warnings
    }

    fn truncate(s: &str, max_len: usize) -> String {
        if s.chars().count() <= max_len {
            s.to_string()
        } else {
            s.chars().take(max_len).collect::<String>() + "…"
        }
    }
}

fn format_category(category: &crate::understanding::models::ClaimCategory) -> &'static str {
    use crate::understanding::models::ClaimCategory;
    match category {
        ClaimCategory::ModuleStructure => "模块结构",
        ClaimCategory::SignalDefinition => "信号定义",
        ClaimCategory::InterfaceDescription => "接口描述",
        ClaimCategory::DataProcessing => "数据处理",
        ClaimCategory::Configuration => "配置",
        ClaimCategory::Documentation => "文档",
        ClaimCategory::TestCoverage => "测试覆盖",
        ClaimCategory::Other => "其他",
    }
}

fn is_cjk(c: char) -> bool {
    matches!(c, '\u{4e00}'..='\u{9fff}')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::models::{
        EvidenceCollection, EvidenceItem, EvidenceStats, EvidenceStrength, LineRange,
    };
    use crate::models::enums::{Language, SourceKind};
    use crate::trace::models::{EvidenceSnapshot, TraceRefResolved, TraceResolution, TraceSourceKind};
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

    fn make_question() -> GroundedQuestion {
        GroundedQuestion {
            question: "位宽是多少".to_string(),
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
    fn ctx_builds_available_citations_from_evidence_and_claims() {
        let question = make_question();
        let context = GroundedQaContextBuilder::build(&question, None, None);

        assert_eq!(context.available_citations.len(), 2);
        assert!(context.available_citations[0].evidence_id.is_some());
        assert!(context.available_citations[1].claim_id.is_some());
    }

    #[test]
    fn ctx_prefers_resolved_trace_citations() {
        let question = make_question();
        let resolved = vec![TraceRefResolved {
            source_kind: TraceSourceKind::Evidence,
            claim: None,
            evidence: Some(EvidenceSnapshot::from(&question.evidence_collection.evidence_items[0])),
            confidence: crate::understanding::models::ClaimConfidence::Supported,
            relevance: None,
            resolution: TraceResolution::Resolved,
        }];
        let context = GroundedQaContextBuilder::build(&question, None, Some(&resolved));

        assert_eq!(context.available_citations.len(), 2);
        assert_eq!(
            context.available_citations[0].evidence_id,
            Some("EV-L0-0001".to_string())
        );
    }

    #[test]
    fn ctx_relevant_claims_filter_by_keyword() {
        let question = make_question();
        let context = GroundedQaContextBuilder::build(&question, None, None);

        assert_eq!(context.relevant_claims.len(), 1);
        assert_eq!(context.relevant_claims[0].claim_id, "CL-L0-0001");
    }

    #[test]
    fn ctx_warnings_when_no_evidence() {
        let mut question = make_question();
        question.evidence_collection.evidence_items.clear();
        let context = GroundedQaContextBuilder::build(&question, None, None);

        assert!(context
            .warnings
            .iter()
            .any(|w| w.code == "evidence_gap"));
    }

    #[test]
    fn ctx_output_structure() {
        let question = make_question();
        let context = GroundedQaContextBuilder::build(&question, None, None);

        assert_eq!(context.question, "位宽是多少");
        assert_eq!(context.stage_id, "L0");
        assert_eq!(context.understanding_summary, "L0 阶段");
        assert!(!context.available_citations.is_empty());
    }
}
