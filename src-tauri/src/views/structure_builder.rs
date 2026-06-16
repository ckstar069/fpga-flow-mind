use crate::understanding::models::{
    ClaimConfidence, EvidenceRef, ImplementationUnderstanding,
};
use crate::views::models::{
    EdgeType, NodeType, ViewEdge, ViewGraph, ViewLayoutHint, ViewMeta, ViewNode, ViewTraceRef,
    ViewType,
};

/// 从 ImplementationUnderstanding 构建结构图
pub fn build_structure_view(iu: &ImplementationUnderstanding) -> ViewGraph {
    let mut node_counter: u32 = 0;
    let mut edge_counter: u32 = 0;
    let mut nodes: Vec<ViewNode> = Vec::new();
    let mut edges: Vec<ViewEdge> = Vec::new();

    // ── 模块节点 ──
    for (i, m) in iu.module_summaries.iter().enumerate() {
        node_counter += 1;
        let node_id = format!("N-structure-{:04}", node_counter);
        nodes.push(ViewNode {
            node_id: node_id.clone(),
            node_type: NodeType::Module,
            label: m.name.clone(),
            description: m.description.clone(),
            confidence: m.confidence,
            trace_refs: build_trace_refs(&m.evidence_refs, &iu.claims, &m.name),
            layout: Some(ViewLayoutHint {
                column: Some(0),
                row: Some(i as u32),
                depth: Some(0),
                group: None,
            }),
        });
    }

    // ── 信号节点 ──
    for (i, s) in iu.signal_summaries.iter().enumerate() {
        node_counter += 1;
        let node_id = format!("N-structure-{:04}", node_counter);
        nodes.push(ViewNode {
            node_id: node_id.clone(),
            node_type: NodeType::Signal,
            label: s.name.clone(),
            description: s.description.clone(),
            confidence: s.confidence,
            trace_refs: build_trace_refs(&s.evidence_refs, &iu.claims, &s.name),
            layout: Some(ViewLayoutHint {
                column: Some(1),
                row: Some(i as u32),
                depth: Some(0),
                group: None,
            }),
        });
    }

    // ── 接口节点 ──
    for (i, iface) in iu.interface_summaries.iter().enumerate() {
        node_counter += 1;
        let node_id = format!("N-structure-{:04}", node_counter);
        nodes.push(ViewNode {
            node_id: node_id.clone(),
            node_type: NodeType::Interface,
            label: iface.name.clone(),
            description: iface.description.clone(),
            confidence: iface.confidence,
            trace_refs: build_trace_refs(&iface.evidence_refs, &iu.claims, &iface.name),
            layout: Some(ViewLayoutHint {
                column: Some(2),
                row: Some(i as u32),
                depth: Some(0),
                group: None,
            }),
        });
    }

    // ── 处理步骤节点 ──
    let step_offset = nodes.len();
    for (i, step) in iu.processing_steps.iter().enumerate() {
        node_counter += 1;
        let node_id = format!("N-structure-{:04}", node_counter);
        nodes.push(ViewNode {
            node_id: node_id.clone(),
            node_type: NodeType::ProcessingStep,
            label: step.name.clone(),
            description: step.description.clone(),
            confidence: step.confidence,
            trace_refs: build_trace_refs(&step.evidence_refs, &iu.claims, &step.name),
            layout: Some(ViewLayoutHint {
                column: Some(0),
                row: Some((step_offset + i) as u32),
                depth: Some(1),
                group: None,
            }),
        });
    }

    // ── 边：module → signal（通过 evidence_id 匹配） ──
    for module_node in &nodes {
        if module_node.node_type != NodeType::Module {
            continue;
        }
        let module_ev_ids: std::collections::HashSet<Option<&str>> = module_node
            .trace_refs
            .iter()
            .map(|t| t.evidence_id.as_deref())
            .collect();

        for signal_node in &nodes {
            if signal_node.node_type != NodeType::Signal {
                continue;
            }
            // 只在有共同 evidence_id 时创建边
            let shares_evidence = signal_node
                .trace_refs
                .iter()
                .any(|t| module_ev_ids.contains(&t.evidence_id.as_deref()));
            if shares_evidence {
                edge_counter += 1;
                edges.push(ViewEdge {
                    edge_id: format!("E-structure-{:04}", edge_counter),
                    edge_type: EdgeType::References,
                    source_node_id: module_node.node_id.clone(),
                    target_node_id: signal_node.node_id.clone(),
                    label: None,
                    description: format!("模块 {} 关联信号 {}", module_node.label, signal_node.label),
                    confidence: ClaimConfidence::Inferred,
                    trace_refs: vec![],
                });
            }
        }
    }

    // ── 边：module → interface ──
    for module_node in &nodes {
        if module_node.node_type != NodeType::Module {
            continue;
        }
        let module_ev_ids: std::collections::HashSet<Option<&str>> = module_node
            .trace_refs
            .iter()
            .map(|t| t.evidence_id.as_deref())
            .collect();

        for iface_node in &nodes {
            if iface_node.node_type != NodeType::Interface {
                continue;
            }
            // 共享 evidence_id 或模块/接口名匹配
            let shares_evidence = iface_node
                .trace_refs
                .iter()
                .any(|t| module_ev_ids.contains(&t.evidence_id.as_deref()));
            let name_match = iface_node
                .label
                .to_lowercase()
                .contains(&module_node.label.to_lowercase().replace("module_", ""));
            if shares_evidence || name_match {
                edge_counter += 1;
                edges.push(ViewEdge {
                    edge_id: format!("E-structure-{:04}", edge_counter),
                    edge_type: EdgeType::References,
                    source_node_id: module_node.node_id.clone(),
                    target_node_id: iface_node.node_id.clone(),
                    label: None,
                    description: format!("模块 {} 使用接口 {}", module_node.label, iface_node.label),
                    confidence: ClaimConfidence::Inferred,
                    trace_refs: vec![],
                });
            }
        }
    }

    // ── 边：processing_step → module（通过 evidence_id 匹配） ──
    for step_node in &nodes {
        if step_node.node_type != NodeType::ProcessingStep {
            continue;
        }
        let step_ev_ids: std::collections::HashSet<Option<&str>> = step_node
            .trace_refs
            .iter()
            .map(|t| t.evidence_id.as_deref())
            .collect();

        for mod_node in &nodes {
            if mod_node.node_type != NodeType::Module {
                continue;
            }
            let shares_evidence = mod_node
                .trace_refs
                .iter()
                .any(|t| step_ev_ids.contains(&t.evidence_id.as_deref()));
            if shares_evidence {
                edge_counter += 1;
                edges.push(ViewEdge {
                    edge_id: format!("E-structure-{:04}", edge_counter),
                    edge_type: EdgeType::Contains,
                    source_node_id: step_node.node_id.clone(),
                    target_node_id: mod_node.node_id.clone(),
                    label: None,
                    description: format!("处理步骤 {} 在模块 {} 内", step_node.label, mod_node.label),
                    confidence: ClaimConfidence::Inferred,
                    trace_refs: vec![],
                });
            }
        }
    }

    // ── 元信息 ──
    let empty_reason = if nodes.is_empty() {
        Some("结构图数据不足：当前阶段无 module/signal/interface/processing_step 信息".to_string())
    } else {
        None
    };

    let meta = ViewMeta {
        stage_id: iu.stage_id.clone(),
        view_type: ViewType::Structure,
        source_provider: iu.generation_meta.provider.clone(),
        is_degraded_source: iu.generation_meta.is_degraded,
        generated_at: chrono::Utc::now().to_rfc3339(),
        empty_reason,
    };

    ViewGraph {
        view_type: ViewType::Structure,
        stage_id: iu.stage_id.clone(),
        nodes,
        edges,
        meta,
    }
}

// ─── 辅助函数 ─────────────────────────────────────────────────────────


/// 从 evidence_refs 构建 trace_refs，并从 claims 匹配追加 claim_id
fn build_trace_refs(
    evidence_refs: &[EvidenceRef],
    claims: &[crate::understanding::models::ImplementationClaim],
    node_name: &str,
) -> Vec<ViewTraceRef> {
    let mut refs: Vec<ViewTraceRef> = evidence_refs
        .iter()
        .map(|r| ViewTraceRef::from_evidence_ref(
            &r.evidence_id,
            ClaimConfidence::Confirmed,
            r.relevance.clone(),
        ))
        .collect();

    // 从 claims 匹配 description 含节点名称的 claim
    for claim in claims {
        if claim.description.contains(node_name) {
            // 避免重复
            let already_has = refs.iter().any(|r| r.claim_id.as_deref() == Some(&claim.claim_id));
            if !already_has {
                refs.push(ViewTraceRef {
                    claim_id: Some(claim.claim_id.clone()),
                    evidence_id: None,
                    confidence: claim.confidence.clone(),
                    relevance: Some("claim 描述匹配".to_string()),
                });
            }
        }
    }

    refs
}

// ─── 测试 ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::understanding::models::{
        ClaimCategory, GenerationMeta, ImplementationClaim,
        ImplementationUnderstanding, ModuleSummary, ProcessingStepSummary, SignalSummary,
        InterfaceSummary, StageSummary, UnderstandingStats,
    };
    use std::collections::HashMap;

    fn make_iu(
        modules: Vec<ModuleSummary>,
        signals: Vec<SignalSummary>,
        interfaces: Vec<InterfaceSummary>,
        steps: Vec<ProcessingStepSummary>,
        claims: Vec<ImplementationClaim>,
    ) -> ImplementationUnderstanding {
        ImplementationUnderstanding {
            stage_id: "L0".to_string(),
            version: "3.0.0".to_string(),
            summary: StageSummary {
                short: "test".to_string(),
                detailed: "test".to_string(),
            },
            claims,
            module_summaries: modules,
            signal_summaries: signals,
            interface_summaries: interfaces,
            processing_steps: steps,
            unknowns: vec![],
            evidence_gaps: vec![],
            generation_meta: GenerationMeta {
                provider: "mock".to_string(),
                generated_at: "2026-06-12T10:00:00Z".to_string(),
                input_evidence_count: 5,
                generation_time_ms: 10,
                is_degraded: false,
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

    fn make_module(name: &str, desc: &str, ev_id: &str) -> ModuleSummary {
        ModuleSummary {
            name: name.to_string(),
            description: desc.to_string(),
            evidence_refs: vec![EvidenceRef {
                evidence_id: ev_id.to_string(),
                relevance: None,
            }],
            confidence: ClaimConfidence::Confirmed,
        }
    }

    fn make_signal(name: &str, desc: &str, ev_id: &str) -> SignalSummary {
        SignalSummary {
            name: name.to_string(),
            description: desc.to_string(),
            direction: None,
            evidence_refs: vec![EvidenceRef {
                evidence_id: ev_id.to_string(),
                relevance: None,
            }],
            confidence: ClaimConfidence::Supported,
        }
    }

    // ─── str_01: 正常 IU → nodes 正确 ────────────────────────────────

    #[test]
    fn str_01_normal_iu_generates_nodes() {
        let iu = make_iu(
            vec![
                make_module("mod_a", "模块 A", "EV-000001"),
                make_module("mod_b", "模块 B", "EV-000002"),
            ],
            vec![make_signal("clk", "时钟信号", "EV-000003")],
            vec![],
            vec![],
            vec![],
        );

        let graph = build_structure_view(&iu);
        assert_eq!(graph.view_type, ViewType::Structure);
        assert_eq!(graph.stage_id, "L0");
        assert_eq!(graph.nodes.len(), 3); // 2 modules + 1 signal

        let module_nodes: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Module)
            .collect();
        assert_eq!(module_nodes.len(), 2);
        assert_eq!(module_nodes[0].label, "mod_a");
        assert_eq!(module_nodes[1].label, "mod_b");

        let signal_nodes: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Signal)
            .collect();
        assert_eq!(signal_nodes.len(), 1);
        assert_eq!(signal_nodes[0].label, "clk");
    }

    // ─── str_02: node_id 唯一 + 单一计数器递增 ───────────────────────

    #[test]
    fn str_02_node_ids_unique_and_sequential() {
        let iu = make_iu(
            vec![make_module("mod_a", "A", "EV-001")],
            vec![make_signal("s1", "S1", "EV-002"), make_signal("s2", "S2", "EV-003")],
            vec![],
            vec![],
            vec![],
        );
        let graph = build_structure_view(&iu);

        let ids: Vec<&str> = graph.nodes.iter().map(|n| n.node_id.as_str()).collect();
        // module 0001, signal 0002, signal 0003 (单一计数器)
        assert_eq!(ids, vec!["N-structure-0001", "N-structure-0002", "N-structure-0003"]);

        // 去重验证
        let mut dedup: Vec<&str> = ids.clone();
        dedup.sort();
        dedup.dedup();
        assert_eq!(dedup.len(), ids.len(), "node_id 不应重复");
    }

    // ─── str_03: edge endpoint 全部存在 ──────────────────────────────

    #[test]
    fn str_03_edge_endpoints_exist() {
        let iu = make_iu(
            vec![make_module("mod_a", "A", "EV-001")],
            vec![make_signal("clk", "clk", "EV-002")],
            vec![],
            vec![],
            vec![],
        );
        let graph = build_structure_view(&iu);

        let node_ids: std::collections::HashSet<&str> =
            graph.nodes.iter().map(|n| n.node_id.as_str()).collect();
        for edge in &graph.edges {
            assert!(
                node_ids.contains(edge.source_node_id.as_str()),
                "edge {} source {} not in nodes",
                edge.edge_id,
                edge.source_node_id
            );
            assert!(
                node_ids.contains(edge.target_node_id.as_str()),
                "edge {} target {} not in nodes",
                edge.edge_id,
                edge.target_node_id
            );
        }
    }

    // ─── str_04: 空 IU → 空图不 panic ────────────────────────────────

    #[test]
    fn str_04_empty_iu_no_panic() {
        let iu = make_iu(vec![], vec![], vec![], vec![], vec![]);
        let graph = build_structure_view(&iu);
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
        assert!(graph.meta.empty_reason.is_some());
    }

    // ─── str_05: trace_refs 包含 evidence_id ─────────────────────────

    #[test]
    fn str_05_trace_refs_contain_evidence_id() {
        let iu = make_iu(
            vec![make_module("mod_a", "A", "EV-000001")],
            vec![],
            vec![],
            vec![],
            vec![],
        );
        let graph = build_structure_view(&iu);
        let node = &graph.nodes[0];
        assert!(!node.trace_refs.is_empty());
        assert_eq!(
            node.trace_refs[0].evidence_id.as_deref(),
            Some("EV-000001")
        );
    }

    // ─── str_06: claim 匹配追加 claim_id ─────────────────────────────

    #[test]
    fn str_06_claim_matching_adds_claim_id() {
        let iu = make_iu(
            vec![make_module("top_mod", "顶层模块", "EV-001")],
            vec![],
            vec![],
            vec![],
            vec![ImplementationClaim {
                claim_id: "CL-L0-000001".to_string(),
                category: ClaimCategory::ModuleStructure,
                description: "top_mod 包含顶层信号处理逻辑".to_string(),
                confidence: ClaimConfidence::Confirmed,
                evidence_refs: vec![EvidenceRef {
                    evidence_id: "EV-001".to_string(),
                    relevance: None,
                }],
                has_evidence_gap: false,
            }],
        );
        let graph = build_structure_view(&iu);
        let node = &graph.nodes[0];

        let has_claim = node
            .trace_refs
            .iter()
            .any(|r| r.claim_id.as_deref() == Some("CL-L0-000001"));
        assert!(has_claim, "应包含匹配的 claim_id");
    }
}
