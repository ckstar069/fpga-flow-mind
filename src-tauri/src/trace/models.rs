use serde::{Deserialize, Serialize};

use crate::evidence::models::{EvidenceCollection, EvidenceItem, LineRange};
use crate::models::enums::Language;
use crate::understanding::models::{
    ImplementationClaim, ImplementationUnderstanding,
};
use crate::views::models::ViewType;

// 为了模块内代码能统一引用，重导出常用枚举
pub use crate::understanding::models::{ClaimCategory, ClaimConfidence};

// ─── 选择模型 ────────────────────────────────────────────────────────

/// 前端一次点击选择的目标
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SelectedTraceTarget {
    ViewNode {
        view_type: ViewType,
        node_id: String,
    },
    ViewEdge {
        view_type: ViewType,
        edge_id: String,
    },
    Claim {
        claim_id: String,
    },
    Evidence {
        evidence_id: String,
    },
}

// ─── 追溯解析模型 ──────────────────────────────────────────────────

/// 解析来源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceSourceKind {
    ViewNode,
    ViewEdge,
    Claim,
    Evidence,
}

/// 追溯解析状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceResolution {
    /// claim 和 evidence 均存在
    Resolved,
    /// 只有 claim，无 evidence（evidence_gap）
    ClaimOnly,
    /// 只有 evidence，无 claim
    EvidenceOnly,
    /// 引用的 claim_id 不存在
    MissingClaim,
    /// 引用的 evidence_id 不存在
    MissingEvidence,
}

/// claim 的轻量展示形态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimSnapshot {
    pub claim_id: String,
    pub category: ClaimCategory,
    pub description: String,
    pub confidence: ClaimConfidence,
    pub evidence_ref_count: usize,
    pub has_evidence_gap: bool,
}

impl From<&ImplementationClaim> for ClaimSnapshot {
    fn from(claim: &ImplementationClaim) -> Self {
        Self {
            claim_id: claim.claim_id.clone(),
            category: claim.category,
            description: claim.description.clone(),
            confidence: claim.confidence,
            evidence_ref_count: claim.evidence_refs.len(),
            has_evidence_gap: claim.has_evidence_gap,
        }
    }
}

/// evidence 的轻量展示形态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSnapshot {
    pub evidence_id: String,
    pub source_path: String,
    pub language: Language,
    pub source_kind: crate::models::enums::SourceKind,
    pub line_range: LineRange,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    pub summary: String,
    pub strength: crate::evidence::models::EvidenceStrength,
}

impl From<&EvidenceItem> for EvidenceSnapshot {
    fn from(item: &EvidenceItem) -> Self {
        Self {
            evidence_id: item.evidence_id.clone(),
            source_path: item.source_path.clone(),
            language: item.language,
            source_kind: item.source_kind,
            line_range: item.line_range,
            symbol: item.symbol.clone(),
            summary: item.summary.clone(),
            strength: item.strength,
        }
    }
}

/// 将 ViewTraceRef 或 EvidenceRef 解析为可展示对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceRefResolved {
    pub source_kind: TraceSourceKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claim: Option<ClaimSnapshot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<EvidenceSnapshot>,
    pub confidence: ClaimConfidence,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relevance: Option<String>,
    pub resolution: TraceResolution,
}

// ─── 源码位置与片段模型 ────────────────────────────────────────────

/// 源码位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub source_path: String,
    pub line_range: LineRange,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_id: Option<String>,
}

/// 源码片段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceExcerpt {
    pub location: SourceLocation,
    pub language: Language,
    pub lines: Vec<SourceLine>,
    pub is_truncated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation_reason: Option<String>,
    pub warnings: Vec<ExcerptWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLine {
    pub line_number: u32,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcerptWarning {
    pub error_code: String,
    pub message: String,
}

// ─── 面板状态模型 ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracePanelState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<SelectedTraceTarget>,
    pub resolved_traces: Vec<TraceRefResolved>,
    pub status: TracePanelStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<crate::models::error::CommandError>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TracePanelStatus {
    Empty,
    Loading,
    Loaded,
    Error,
}

// ─── Grounded Q&A 模型 ─────────────────────────────────────────────

/// Q&A 输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedQuestion {
    pub question: String,
    pub stage_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_target: Option<SelectedTraceTarget>,
    pub understanding: ImplementationUnderstanding,
    pub evidence_collection: EvidenceCollection,
}

/// Q&A 输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedAnswer {
    pub answer_id: String,
    pub generated_at: String,
    pub text: String,
    pub claims: Vec<GroundedAnswerClaim>,
    pub citations: Vec<GroundedAnswerCitation>,
    pub confidence: ClaimConfidence,
    pub warnings: Vec<GroundedQaWarning>,
    pub provider: String,
    pub is_degraded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedAnswerClaim {
    pub text: String,
    pub confidence: ClaimConfidence,
    pub citation_indices: Vec<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedAnswerCitation {
    pub index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claim_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_location: Option<SourceLocation>,
    pub excerpt_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedQaWarning {
    pub code: String,
    pub message: String,
}

/// Provider 上下文（Batch A 只定义类型，不实现 Provider）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedQaContext {
    pub question: String,
    pub stage_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_target: Option<SelectedTraceTarget>,
    pub understanding_summary: String,
    pub claims: Vec<ImplementationClaim>,
    pub evidence_collection: EvidenceCollection,
    pub available_citations: Vec<GroundedAnswerCitation>,
    pub relevant_claims: Vec<ImplementationClaim>,
    pub relevant_evidence: Vec<crate::evidence::models::EvidenceItem>,
    pub warnings: Vec<GroundedQaWarning>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_trace_target_serde_roundtrip() {
        let target = SelectedTraceTarget::ViewNode {
            view_type: ViewType::Structure,
            node_id: "N-structure-0001".to_string(),
        };
        let json = serde_json::to_string(&target).unwrap();
        let back: SelectedTraceTarget = serde_json::from_str(&json).unwrap();
        match back {
            SelectedTraceTarget::ViewNode { view_type, node_id } => {
                assert_eq!(view_type, ViewType::Structure);
                assert_eq!(node_id, "N-structure-0001");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn trace_resolution_snake_case() {
        assert_eq!(
            serde_json::to_string(&TraceResolution::MissingEvidence).unwrap(),
            "\"missing_evidence\""
        );
        assert_eq!(
            serde_json::to_string(&TraceResolution::ClaimOnly).unwrap(),
            "\"claim_only\""
        );
    }

    #[test]
    fn source_excerpt_truncation_optional() {
        let excerpt = SourceExcerpt {
            location: SourceLocation {
                source_path: "/tmp/test.v".to_string(),
                line_range: LineRange { start: 1, end: 5 },
                evidence_id: None,
            },
            language: Language::Verilog,
            lines: vec![SourceLine {
                line_number: 1,
                content: "module test;".to_string(),
            }],
            is_truncated: false,
            truncation_reason: None,
            warnings: vec![],
        };
        let json = serde_json::to_string(&excerpt).unwrap();
        assert!(!json.contains("truncation_reason"));
    }

    #[test]
    fn grounded_answer_unknown_allows_empty_citations() {
        let answer = GroundedAnswer {
            answer_id: "A-001".to_string(),
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
                message: "当前阶段证据不足".to_string(),
            }],
            provider: "mock".to_string(),
            is_degraded: true,
        };
        let json = serde_json::to_string(&answer).unwrap();
        let back: GroundedAnswer = serde_json::from_str(&json).unwrap();
        assert_eq!(back.confidence, ClaimConfidence::Unknown);
        assert!(back.citations.is_empty());
        assert_eq!(back.claims[0].reason, Some("当前证据不足".to_string()));
    }
}
