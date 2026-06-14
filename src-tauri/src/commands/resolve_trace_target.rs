/// `resolve_trace_target` Tauri command
///
/// 根据用户选择目标解析追溯引用。不访问目标项目文件系统，只引用已有的
/// ImplementationUnderstanding、EvidenceCollection 和 ViewGraph。

use crate::models::enums::ErrorCode;
use crate::models::error::{CommandError, CommandResult};
use crate::trace::models::{SelectedTraceTarget, TraceRefResolved};
use crate::trace::resolver::{TraceError, TraceResolver};
use crate::understanding::models::ImplementationUnderstanding;
use crate::evidence::models::EvidenceCollection;
use crate::views::models::ViewGraph;

#[tauri::command]
pub fn resolve_trace_target(
    target: SelectedTraceTarget,
    understanding: ImplementationUnderstanding,
    evidence_collection: EvidenceCollection,
    views: Vec<ViewGraph>,
) -> CommandResult<Vec<TraceRefResolved>> {
    match TraceResolver::resolve(
        &target,
        &understanding,
        &evidence_collection,
        &views,
    ) {
        Ok(traces) => CommandResult {
            success: true,
            data: Some(traces),
            error: None,
            warnings: Vec::new(),
        },
        Err(err) => CommandResult {
            success: false,
            data: None,
            error: Some(map_trace_error(err)),
            warnings: Vec::new(),
        },
    }
}

fn map_trace_error(err: TraceError) -> CommandError {
    CommandError {
        error_code: err.error_code,
        message: err.message,
        recoverable: matches!(err.error_code, ErrorCode::TraceTargetNotFound),
        details: None,
        source_path: None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::evidence::models::{
        EvidenceCollection, EvidenceItem, EvidenceStats, EvidenceStrength,
    };
    use crate::models::enums::{Language, SourceKind};
    use crate::understanding::models::{
        ClaimCategory, ClaimConfidence, EvidenceRef, ImplementationClaim,
        ImplementationUnderstanding, StageSummary, UnderstandingStats,
    };
    use crate::views::models::{
        NodeType, ViewEdge, ViewGraph, ViewMeta, ViewNode, ViewTraceRef, ViewType,
    };

    fn make_evidence_collection(items: Vec<EvidenceItem>) -> EvidenceCollection {
        EvidenceCollection {
            stage_id: "L0".to_string(),
            evidence_items: items,
            index_by_path: HashMap::new(),
            index_by_kind: HashMap::new(),
            index_by_symbol: HashMap::new(),
            warnings: vec![],
            stats: EvidenceStats {
                files_processed: 0,
                files_skipped: 0,
                total_items: 0,
                items_by_kind: HashMap::new(),
                items_by_strength: HashMap::new(),
            },
            version: "1.0.0".to_string(),
        }
    }

    fn make_understanding(claims: Vec<ImplementationClaim>) -> ImplementationUnderstanding {
        ImplementationUnderstanding {
            stage_id: "L0".to_string(),
            version: "3.0.0".to_string(),
            summary: StageSummary {
                short: "测试".to_string(),
                detailed: "测试".to_string(),
            },
            claims,
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
        }
    }

    fn make_evidence_item(id: &str, path: &str) -> EvidenceItem {
        EvidenceItem {
            evidence_id: id.to_string(),
            source_path: path.to_string(),
            language: Language::Verilog,
            source_kind: SourceKind::Rtl,
            line_range: crate::evidence::models::LineRange { start: 1, end: 5 },
            symbol: None,
            summary: "test".to_string(),
            strength: EvidenceStrength::Direct,
        }
    }

    fn make_claim(id: &str, evidence_refs: Vec<EvidenceRef>) -> ImplementationClaim {
        ImplementationClaim {
            claim_id: id.to_string(),
            category: ClaimCategory::ModuleStructure,
            description: "测试 claim".to_string(),
            confidence: ClaimConfidence::Confirmed,
            evidence_refs,
            has_evidence_gap: false,
        }
    }

    fn make_node(node_id: &str, trace_refs: Vec<ViewTraceRef>) -> ViewNode {
        ViewNode {
            node_id: node_id.to_string(),
            node_type: NodeType::Module,
            label: node_id.to_string(),
            description: "test".to_string(),
            confidence: ClaimConfidence::Confirmed,
            trace_refs,
            layout: None,
        }
    }

    fn make_view_graph(view_type: ViewType, nodes: Vec<ViewNode>, edges: Vec<ViewEdge>) -> ViewGraph {
        ViewGraph {
            view_type,
            stage_id: "L0".to_string(),
            nodes,
            edges,
            meta: ViewMeta {
                stage_id: "L0".to_string(),
                view_type,
                source_provider: "mock".to_string(),
                is_degraded_source: true,
                generated_at: "2026-06-14T10:00:00Z".to_string(),
                empty_reason: None,
            },
        }
    }

    // ─── command 测试 ────────────────────────────────────────────────

    #[test]
    fn cmd_resolve_view_node_ok() {
        let evidence = make_evidence_item("EV-L0-000001", "/tmp/test.v");
        let collection = make_evidence_collection(vec![evidence]);
        let claim = make_claim(
            "CL-L0-000001",
            vec![EvidenceRef {
                evidence_id: "EV-L0-000001".to_string(),
                relevance: Some("模块定义".to_string()),
            }],
        );
        let understanding = make_understanding(vec![claim]);
        let node = make_node(
            "N-structure-0001",
            vec![ViewTraceRef {
                claim_id: Some("CL-L0-000001".to_string()),
                evidence_id: Some("EV-L0-000001".to_string()),
                confidence: ClaimConfidence::Confirmed,
                relevance: Some("定义了模块".to_string()),
            }],
        );
        let views = vec![make_view_graph(ViewType::Structure, vec![node], vec![])];

        let target = SelectedTraceTarget::ViewNode {
            view_type: ViewType::Structure,
            node_id: "N-structure-0001".to_string(),
        };
        let result = resolve_trace_target(target, understanding, collection, views);

        assert!(result.success);
        let traces = result.data.unwrap();
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].resolution, crate::trace::models::TraceResolution::Resolved);
    }

    #[test]
    fn cmd_resolve_claim_ok() {
        let evidence = make_evidence_item("EV-L0-000001", "/tmp/test.v");
        let collection = make_evidence_collection(vec![evidence]);
        let claim = make_claim(
            "CL-L0-000001",
            vec![EvidenceRef {
                evidence_id: "EV-L0-000001".to_string(),
                relevance: None,
            }],
        );
        let understanding = make_understanding(vec![claim]);

        let target = SelectedTraceTarget::Claim {
            claim_id: "CL-L0-000001".to_string(),
        };
        let result = resolve_trace_target(target, understanding, collection, vec![]);

        assert!(result.success);
        let traces = result.data.unwrap();
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].evidence.as_ref().unwrap().evidence_id, "EV-L0-000001");
    }

    #[test]
    fn cmd_resolve_evidence_ok() {
        let evidence = make_evidence_item("EV-L0-000001", "/tmp/test.v");
        let collection = make_evidence_collection(vec![evidence]);
        let understanding = make_understanding(vec![]);

        let target = SelectedTraceTarget::Evidence {
            evidence_id: "EV-L0-000001".to_string(),
        };
        let result = resolve_trace_target(target, understanding, collection, vec![]);

        assert!(result.success);
        let traces = result.data.unwrap();
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].source_kind, crate::trace::models::TraceSourceKind::Evidence);
    }

    #[test]
    fn cmd_resolve_missing_target_returns_error() {
        let understanding = make_understanding(vec![]);
        let collection = make_evidence_collection(vec![]);
        let views = vec![];

        let target = SelectedTraceTarget::ViewNode {
            view_type: ViewType::Structure,
            node_id: "N-structure-0001".to_string(),
        };
        let result = resolve_trace_target(target, understanding, collection, views);

        assert!(!result.success);
        let err = result.error.unwrap();
        assert_eq!(err.error_code, ErrorCode::TraceTargetNotFound);
        assert!(err.recoverable);
    }
}
