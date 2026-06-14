use crate::models::enums::ErrorCode;
use crate::trace::models::{
    ClaimSnapshot, EvidenceSnapshot, SelectedTraceTarget, TraceRefResolved, TraceResolution,
    TraceSourceKind,
};
use crate::understanding::models::{ClaimConfidence, ImplementationUnderstanding};
use crate::views::models::{ViewGraph, ViewType};
use crate::evidence::models::EvidenceCollection;

/// TraceResolver 内部错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceError {
    pub error_code: ErrorCode,
    pub message: String,
}

impl TraceError {
    pub fn target_not_found(message: impl Into<String>) -> Self {
        Self {
            error_code: ErrorCode::TraceTargetNotFound,
            message: message.into(),
        }
    }
}

/// 根据用户选择目标解析追溯信息
pub struct TraceResolver;

impl TraceResolver {
    pub fn resolve(
        target: &SelectedTraceTarget,
        understanding: &ImplementationUnderstanding,
        evidence_collection: &EvidenceCollection,
        views: &[ViewGraph],
    ) -> Result<Vec<TraceRefResolved>, TraceError> {
        match target {
            SelectedTraceTarget::ViewNode { view_type, node_id } => {
                Self::resolve_view_node(view_type, node_id, views, understanding, evidence_collection)
            }
            SelectedTraceTarget::ViewEdge { view_type, edge_id } => {
                Self::resolve_view_edge(view_type, edge_id, views, understanding, evidence_collection)
            }
            SelectedTraceTarget::Claim { claim_id } => {
                Self::resolve_claim(claim_id, understanding, evidence_collection)
            }
            SelectedTraceTarget::Evidence { evidence_id } => {
                Self::resolve_evidence(evidence_id, evidence_collection)
            }
        }
    }

    fn resolve_view_node(
        view_type: &ViewType,
        node_id: &str,
        views: &[ViewGraph],
        understanding: &ImplementationUnderstanding,
        evidence_collection: &EvidenceCollection,
    ) -> Result<Vec<TraceRefResolved>, TraceError> {
        let graph = Self::find_view(view_type, views)?;
        let node = graph
            .nodes
            .iter()
            .find(|n| n.node_id == node_id)
            .ok_or_else(|| TraceError::target_not_found(format!("node {} not found in {:?}", node_id, view_type)))?;

        let mut results = Vec::new();
        for trace_ref in &node.trace_refs {
            results.push(Self::resolve_trace_ref(
                TraceSourceKind::ViewNode,
                trace_ref.claim_id.as_deref(),
                trace_ref.evidence_id.as_deref(),
                trace_ref.confidence,
                trace_ref.relevance.clone(),
                understanding,
                evidence_collection,
            ));
        }
        Ok(results)
    }

    fn resolve_view_edge(
        view_type: &ViewType,
        edge_id: &str,
        views: &[ViewGraph],
        understanding: &ImplementationUnderstanding,
        evidence_collection: &EvidenceCollection,
    ) -> Result<Vec<TraceRefResolved>, TraceError> {
        let graph = Self::find_view(view_type, views)?;
        let edge = graph
            .edges
            .iter()
            .find(|e| e.edge_id == edge_id)
            .ok_or_else(|| TraceError::target_not_found(format!("edge {} not found in {:?}", edge_id, view_type)))?;

        let mut results = Vec::new();
        for trace_ref in &edge.trace_refs {
            results.push(Self::resolve_trace_ref(
                TraceSourceKind::ViewEdge,
                trace_ref.claim_id.as_deref(),
                trace_ref.evidence_id.as_deref(),
                trace_ref.confidence,
                trace_ref.relevance.clone(),
                understanding,
                evidence_collection,
            ));
        }
        Ok(results)
    }

    fn resolve_claim(
        claim_id: &str,
        understanding: &ImplementationUnderstanding,
        evidence_collection: &EvidenceCollection,
    ) -> Result<Vec<TraceRefResolved>, TraceError> {
        let claim = understanding
            .claims
            .iter()
            .find(|c| c.claim_id == claim_id)
            .ok_or_else(|| TraceError::target_not_found(format!("claim {} not found", claim_id)))?;

        let mut results = Vec::new();

        if claim.evidence_refs.is_empty() {
            if claim.has_evidence_gap {
                results.push(TraceRefResolved {
                    source_kind: TraceSourceKind::Claim,
                    claim: Some(ClaimSnapshot::from(claim)),
                    evidence: None,
                    confidence: claim.confidence,
                    relevance: None,
                    resolution: TraceResolution::ClaimOnly,
                });
            } else {
                // 无 evidence 且无 gap：视为 missing evidence
                results.push(TraceRefResolved {
                    source_kind: TraceSourceKind::Claim,
                    claim: Some(ClaimSnapshot::from(claim)),
                    evidence: None,
                    confidence: ClaimConfidence::Unknown,
                    relevance: Some("claim 未绑定 evidence".to_string()),
                    resolution: TraceResolution::MissingEvidence,
                });
            }
        } else {
            for evidence_ref in &claim.evidence_refs {
                let evidence = evidence_collection
                    .evidence_items
                    .iter()
                    .find(|e| e.evidence_id == evidence_ref.evidence_id);

                let (evidence_snapshot, resolution) = match evidence {
                    Some(item) => (Some(EvidenceSnapshot::from(item)), TraceResolution::Resolved),
                    None => (None, TraceResolution::MissingEvidence),
                };

                results.push(TraceRefResolved {
                    source_kind: TraceSourceKind::Claim,
                    claim: Some(ClaimSnapshot::from(claim)),
                    evidence: evidence_snapshot,
                    confidence: claim.confidence,
                    relevance: evidence_ref.relevance.clone(),
                    resolution,
                });
            }
        }

        Ok(results)
    }

    fn resolve_evidence(
        evidence_id: &str,
        evidence_collection: &EvidenceCollection,
    ) -> Result<Vec<TraceRefResolved>, TraceError> {
        let evidence = evidence_collection
            .evidence_items
            .iter()
            .find(|e| e.evidence_id == evidence_id)
            .ok_or_else(|| TraceError::target_not_found(format!("evidence {} not found", evidence_id)))?;

        Ok(vec![TraceRefResolved {
            source_kind: TraceSourceKind::Evidence,
            claim: None,
            evidence: Some(EvidenceSnapshot::from(evidence)),
            confidence: ClaimConfidence::from(evidence.strength),
            relevance: None,
            resolution: TraceResolution::EvidenceOnly,
        }])
    }

    fn resolve_trace_ref(
        source_kind: TraceSourceKind,
        claim_id: Option<&str>,
        evidence_id: Option<&str>,
        confidence: ClaimConfidence,
        relevance: Option<String>,
        understanding: &ImplementationUnderstanding,
        evidence_collection: &EvidenceCollection,
    ) -> TraceRefResolved {
        let claim = claim_id.and_then(|id| understanding.claims.iter().find(|c| c.claim_id == id));
        let evidence = evidence_id.and_then(|id| evidence_collection.evidence_items.iter().find(|e| e.evidence_id == id));

        let resolution = match (&claim,
            &evidence,
            claim_id.is_some() && claim.is_none(),
            evidence_id.is_some() && evidence.is_none(),
        ) {
            (Some(_), Some(_), _, _) => TraceResolution::Resolved,
            (Some(_), None, _, false) => TraceResolution::ClaimOnly,
            (None, Some(_), false, _) => TraceResolution::EvidenceOnly,
            (_, _, true, _) => TraceResolution::MissingClaim,
            (_, _, false, true) => TraceResolution::MissingEvidence,
            (None, None, false, false) => TraceResolution::MissingEvidence,
        };

        TraceRefResolved {
            source_kind,
            claim: claim.map(ClaimSnapshot::from),
            evidence: evidence.map(EvidenceSnapshot::from),
            confidence,
            relevance,
            resolution,
        }
    }

    fn find_view<'a>(view_type: &ViewType, views: &'a [ViewGraph]) -> Result<&'a ViewGraph, TraceError> {
        views
            .iter()
            .find(|v| &v.view_type == view_type)
            .ok_or_else(|| TraceError::target_not_found(format!("view {:?} not found", view_type)))
    }
}

impl From<crate::evidence::models::EvidenceStrength> for ClaimConfidence {
    fn from(strength: crate::evidence::models::EvidenceStrength) -> Self {
        use crate::evidence::models::EvidenceStrength;
        match strength {
            EvidenceStrength::Direct => ClaimConfidence::Confirmed,
            EvidenceStrength::Indirect => ClaimConfidence::Supported,
            EvidenceStrength::Weak => ClaimConfidence::Inferred,
            EvidenceStrength::Conflicting => ClaimConfidence::Conflicting,
            EvidenceStrength::Missing => ClaimConfidence::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::evidence::models::{
        EvidenceCollection, EvidenceItem, EvidenceStats, EvidenceStrength, EvidenceWarning,
    };
    use crate::models::enums::{Language, SourceKind};
    use crate::understanding::models::{
        ClaimCategory, ClaimConfidence, EvidenceRef, ImplementationClaim, ImplementationUnderstanding,
    };
    use crate::views::models::{
        EdgeType, NodeType, ViewEdge, ViewGraph, ViewNode, ViewTraceRef, ViewType,
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
        use crate::understanding::models::{
            EvidenceGap, GenerationMeta, ModuleSummary, ProcessingStepSummary, SignalSummary,
            StageSummary, InterfaceSummary, UnknownItem, UnderstandingStats,
        };
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
            generation_meta: GenerationMeta {
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

    fn make_claim(id: &str, evidence_refs: Vec<EvidenceRef>, has_gap: bool) -> ImplementationClaim {
        ImplementationClaim {
            claim_id: id.to_string(),
            category: ClaimCategory::ModuleStructure,
            description: "测试 claim".to_string(),
            confidence: ClaimConfidence::Confirmed,
            evidence_refs,
            has_evidence_gap: has_gap,
        }
    }

    fn make_view_graph(view_type: ViewType, nodes: Vec<ViewNode>, edges: Vec<ViewEdge>) -> ViewGraph {
        use crate::views::models::ViewMeta;
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

    // ─── resolver 测试 ───────────────────────────────────────────────

    #[test]
    fn res_01_view_node_resolves_trace_refs() {
        let evidence = make_evidence_item("EV-L0-000001", "/tmp/test.v");
        let collection = make_evidence_collection(vec![evidence]);
        let claim = make_claim(
            "CL-L0-000001",
            vec![EvidenceRef {
                evidence_id: "EV-L0-000001".to_string(),
                relevance: Some("模块定义".to_string()),
            }],
            false,
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
        let result = TraceResolver::resolve(&target, &understanding, &collection, &views).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].resolution, TraceResolution::Resolved);
        assert!(result[0].claim.is_some());
        assert!(result[0].evidence.is_some());
    }

    #[test]
    fn res_02_view_edge_resolves_trace_refs() {
        let evidence = make_evidence_item("EV-L0-000001", "/tmp/test.v");
        let collection = make_evidence_collection(vec![evidence]);
        let understanding = make_understanding(vec![]);
        let edge = ViewEdge {
            edge_id: "E-structure-0001".to_string(),
            edge_type: EdgeType::References,
            source_node_id: "N-structure-0001".to_string(),
            target_node_id: "N-structure-0002".to_string(),
            label: None,
            description: "引用".to_string(),
            confidence: ClaimConfidence::Confirmed,
            trace_refs: vec![ViewTraceRef {
                claim_id: None,
                evidence_id: Some("EV-L0-000001".to_string()),
                confidence: ClaimConfidence::Supported,
                relevance: None,
            }],
        };
        let views = vec![make_view_graph(ViewType::Structure, vec![], vec![edge])];

        let target = SelectedTraceTarget::ViewEdge {
            view_type: ViewType::Structure,
            edge_id: "E-structure-0001".to_string(),
        };
        let result = TraceResolver::resolve(&target, &understanding, &collection, &views).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].resolution, TraceResolution::EvidenceOnly);
        assert!(result[0].claim.is_none());
        assert!(result[0].evidence.is_some());
    }

    #[test]
    fn res_03_claim_resolves_evidence_refs() {
        let evidence = make_evidence_item("EV-L0-000001", "/tmp/test.v");
        let collection = make_evidence_collection(vec![evidence]);
        let claim = make_claim(
            "CL-L0-000001",
            vec![EvidenceRef {
                evidence_id: "EV-L0-000001".to_string(),
                relevance: Some("模块定义".to_string()),
            }],
            false,
        );
        let understanding = make_understanding(vec![claim]);

        let target = SelectedTraceTarget::Claim {
            claim_id: "CL-L0-000001".to_string(),
        };
        let result = TraceResolver::resolve(
            &target,
            &understanding,
            &collection,
            &[],
        )
        .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].resolution, TraceResolution::Resolved);
        assert_eq!(result[0].evidence.as_ref().unwrap().evidence_id, "EV-L0-000001");
    }

    #[test]
    fn res_04_evidence_resolves_directly() {
        let evidence = make_evidence_item("EV-L0-000001", "/tmp/test.v");
        let collection = make_evidence_collection(vec![evidence]);
        let understanding = make_understanding(vec![]);

        let target = SelectedTraceTarget::Evidence {
            evidence_id: "EV-L0-000001".to_string(),
        };
        let result = TraceResolver::resolve(
            &target,
            &understanding,
            &collection,
            &[],
        )
        .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].resolution, TraceResolution::EvidenceOnly);
        assert_eq!(result[0].source_kind, TraceSourceKind::Evidence);
    }

    #[test]
    fn res_05_missing_evidence_id() {
        let understanding = make_understanding(vec![]);
        let collection = make_evidence_collection(vec![]);
        let node = make_node(
            "N-structure-0001",
            vec![ViewTraceRef {
                claim_id: None,
                evidence_id: Some("EV-L0-000001".to_string()),
                confidence: ClaimConfidence::Confirmed,
                relevance: None,
            }],
        );
        let views = vec![make_view_graph(ViewType::Structure, vec![node], vec![])];

        let target = SelectedTraceTarget::ViewNode {
            view_type: ViewType::Structure,
            node_id: "N-structure-0001".to_string(),
        };
        let result = TraceResolver::resolve(&target, &understanding, &collection, &views).unwrap();

        assert_eq!(result[0].resolution, TraceResolution::MissingEvidence);
        assert!(result[0].evidence.is_none());
    }

    #[test]
    fn res_06_missing_claim_id() {
        let understanding = make_understanding(vec![]);
        let collection = make_evidence_collection(vec![]);
        let node = make_node(
            "N-structure-0001",
            vec![ViewTraceRef {
                claim_id: Some("CL-L0-000001".to_string()),
                evidence_id: None,
                confidence: ClaimConfidence::Confirmed,
                relevance: None,
            }],
        );
        let views = vec![make_view_graph(ViewType::Structure, vec![node], vec![])];

        let target = SelectedTraceTarget::ViewNode {
            view_type: ViewType::Structure,
            node_id: "N-structure-0001".to_string(),
        };
        let result = TraceResolver::resolve(&target, &understanding, &collection, &views).unwrap();

        assert_eq!(result[0].resolution, TraceResolution::MissingClaim);
        assert!(result[0].claim.is_none());
    }

    #[test]
    fn res_07_claim_with_evidence_gap() {
        let collection = make_evidence_collection(vec![]);
        let claim = make_claim("CL-L0-000001", vec![], true);
        let understanding = make_understanding(vec![claim]);

        let target = SelectedTraceTarget::Claim {
            claim_id: "CL-L0-000001".to_string(),
        };
        let result = TraceResolver::resolve(
            &target,
            &understanding,
            &collection,
            &[],
        )
        .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].resolution, TraceResolution::ClaimOnly);
        assert!(result[0].claim.as_ref().unwrap().has_evidence_gap);
    }

    #[test]
    fn res_08_empty_trace_refs() {
        let understanding = make_understanding(vec![]);
        let collection = make_evidence_collection(vec![]);
        let node = make_node("N-structure-0001", vec![]);
        let views = vec![make_view_graph(ViewType::Structure, vec![node], vec![])];

        let target = SelectedTraceTarget::ViewNode {
            view_type: ViewType::Structure,
            node_id: "N-structure-0001".to_string(),
        };
        let result = TraceResolver::resolve(&target, &understanding, &collection, &views).unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn res_09_view_type_not_found() {
        let understanding = make_understanding(vec![]);
        let collection = make_evidence_collection(vec![]);
        let views = vec![];

        let target = SelectedTraceTarget::ViewNode {
            view_type: ViewType::Structure,
            node_id: "N-structure-0001".to_string(),
        };
        let result = TraceResolver::resolve(&target, &understanding, &collection, &views);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code, ErrorCode::TraceTargetNotFound);
    }

    #[test]
    fn res_10_node_id_not_found() {
        let understanding = make_understanding(vec![]);
        let collection = make_evidence_collection(vec![]);
        let views = vec![make_view_graph(ViewType::Structure, vec![make_node("other", vec![])], vec![])];

        let target = SelectedTraceTarget::ViewNode {
            view_type: ViewType::Structure,
            node_id: "N-structure-0001".to_string(),
        };
        let result = TraceResolver::resolve(&target, &understanding, &collection, &views);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code, ErrorCode::TraceTargetNotFound);
    }
}
