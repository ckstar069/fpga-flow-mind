use crate::understanding::models::{
    ClaimConfidence, EvidenceRef, ImplementationUnderstanding, ProcessingStepSummary,
};
use crate::views::models::{
    EdgeType, NodeType, ViewEdge, ViewGraph, ViewLayoutHint, ViewMeta, ViewNode, ViewTraceRef,
    ViewType,
};

/// 从 ImplementationUnderstanding 构建数据流图
pub fn build_dataflow_view(iu: &ImplementationUnderstanding) -> ViewGraph {
    let mut node_counter: u32 = 0;
    let mut edge_counter: u32 = 0;
    let mut nodes: Vec<ViewNode> = Vec::new();
    let mut edges: Vec<ViewEdge> = Vec::new();

    // ── 输入源节点 ──
    for iface in &iu.interface_summaries {
        let name_lower = iface.name.to_lowercase();
        let is_input =
            name_lower.contains("in") || name_lower.contains("input") || name_lower.contains("接收");
        if is_input {
            node_counter += 1;
            let node_id = format!("N-dataflow-{:04}", node_counter);
            nodes.push(ViewNode {
                node_id,
                node_type: NodeType::InputSource,
                label: iface.name.clone(),
                description: iface.description.clone(),
                confidence: ClaimConfidence::Supported,
                trace_refs: build_trace_refs(&iface.evidence_refs),
                layout: Some(ViewLayoutHint {
                    column: Some(0),
                    row: Some(node_counter - 1),
                    depth: Some(0),
                    group: None,
                }),
            });
        }
    }

    // ── 处理步骤节点（按 order 排序） ──
    let mut sorted_steps: Vec<&ProcessingStepSummary> = iu.processing_steps.iter().collect();
    sorted_steps.sort_by_key(|s| s.order);

    for step in &sorted_steps {
        node_counter += 1;
        let node_id = format!("N-dataflow-{:04}", node_counter);
        nodes.push(ViewNode {
            node_id,
            node_type: NodeType::ProcessingStep,
            label: step.name.clone(),
            description: step.description.clone(),
            confidence: step.confidence,
            trace_refs: build_trace_refs(&step.evidence_refs),
            layout: Some(ViewLayoutHint {
                column: Some(1),
                row: Some(node_counter - 1),
                depth: Some(0),
                group: None,
            }),
        });
    }

    // ── 输出目标节点 ──
    for iface in &iu.interface_summaries {
        let name_lower = iface.name.to_lowercase();
        let is_output = name_lower.contains("out") || name_lower.contains("output") || name_lower.contains("发送");
        if is_output {
            node_counter += 1;
            let node_id = format!("N-dataflow-{:04}", node_counter);
            nodes.push(ViewNode {
                node_id,
                node_type: NodeType::OutputTarget,
                label: iface.name.clone(),
                description: iface.description.clone(),
                confidence: ClaimConfidence::Supported,
                trace_refs: build_trace_refs(&iface.evidence_refs),
                layout: Some(ViewLayoutHint {
                    column: Some(2),
                    row: Some(node_counter - 1),
                    depth: Some(0),
                    group: None,
                }),
            });
        }
    }

    // ── 中间数据节点（非输入/非输出信号） ──
    for sig in &iu.signal_summaries {
        let name_lower = sig.name.to_lowercase();
        let is_io = name_lower.contains("in") || name_lower.contains("out") || name_lower.contains("input") || name_lower.contains("output");
        if !is_io {
            node_counter += 1;
            let node_id = format!("N-dataflow-{:04}", node_counter);
            nodes.push(ViewNode {
                node_id,
                node_type: NodeType::IntermediateData,
                label: sig.name.clone(),
                description: sig.description.clone(),
                confidence: sig.confidence,
                trace_refs: build_trace_refs(&sig.evidence_refs),
                layout: Some(ViewLayoutHint {
                    column: Some(1),
                    row: Some(node_counter - 1),
                    depth: Some(1),
                    group: None,
                }),
            });
        }
    }

    // ── 边：input → first processing step ──
    let input_ids: Vec<String> = nodes
        .iter()
        .filter(|n| n.node_type == NodeType::InputSource)
        .map(|n| n.node_id.clone())
        .collect();
    let step_ids: Vec<String> = nodes
        .iter()
        .filter(|n| n.node_type == NodeType::ProcessingStep)
        .map(|n| n.node_id.clone())
        .collect();

    if !input_ids.is_empty() && !step_ids.is_empty() {
        edge_counter += 1;
        edges.push(ViewEdge {
            edge_id: format!("E-dataflow-{:04}", edge_counter),
            edge_type: EdgeType::DataFlow,
            source_node_id: input_ids[0].clone(),
            target_node_id: step_ids[0].clone(),
            label: None,
            description: "输入数据流入第一个处理步骤".to_string(),
            confidence: ClaimConfidence::Inferred,
            trace_refs: vec![],
        });
    }

    // ── 边：processing_step[i] → processing_step[i+1] ──
    for i in 0..step_ids.len().saturating_sub(1) {
        edge_counter += 1;
        edges.push(ViewEdge {
            edge_id: format!("E-dataflow-{:04}", edge_counter),
            edge_type: EdgeType::DataFlow,
            source_node_id: step_ids[i].clone(),
            target_node_id: step_ids[i + 1].clone(),
            label: None,
            description: format!("处理步骤 {} → {}", i + 1, i + 2),
            confidence: ClaimConfidence::Inferred,
            trace_refs: vec![],
        });
    }

    // ── 边：last processing step → output ──
    let output_ids: Vec<String> = nodes
        .iter()
        .filter(|n| n.node_type == NodeType::OutputTarget)
        .map(|n| n.node_id.clone())
        .collect();

    if !step_ids.is_empty() && !output_ids.is_empty() {
        edge_counter += 1;
        edges.push(ViewEdge {
            edge_id: format!("E-dataflow-{:04}", edge_counter),
            edge_type: EdgeType::DataFlow,
            source_node_id: step_ids[step_ids.len() - 1].clone(),
            target_node_id: output_ids[0].clone(),
            label: None,
            description: "最后一个处理步骤输出数据".to_string(),
            confidence: ClaimConfidence::Inferred,
            trace_refs: vec![],
        });
    }

    // ── 元信息 ──
    let empty_reason = if nodes.is_empty() {
        Some(
            "数据流信息不足：无 processing_steps 且无可识别的输入/输出信号".to_string(),
        )
    } else {
        None
    };

    let meta = ViewMeta {
        stage_id: iu.stage_id.clone(),
        view_type: ViewType::Dataflow,
        source_provider: iu.generation_meta.provider.clone(),
        is_degraded_source: iu.generation_meta.is_degraded,
        generated_at: chrono::Utc::now().to_rfc3339(),
        empty_reason,
    };

    ViewGraph {
        view_type: ViewType::Dataflow,
        stage_id: iu.stage_id.clone(),
        nodes,
        edges,
        meta,
    }
}

// ─── 辅助函数 ─────────────────────────────────────────────────────────


fn build_trace_refs(evidence_refs: &[EvidenceRef]) -> Vec<ViewTraceRef> {
    evidence_refs
        .iter()
        .map(|r| ViewTraceRef::from_evidence_ref(
            &r.evidence_id,
            ClaimConfidence::Confirmed,
            r.relevance.clone(),
        ))
        .collect()
}

// ─── 测试 ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::understanding::models::{
        EvidenceRef, GenerationMeta, ImplementationUnderstanding, InterfaceSummary,
        ProcessingStepSummary, SignalSummary, StageSummary, UnderstandingStats,
    };
    use std::collections::HashMap;

    fn make_iu(
        steps: Vec<ProcessingStepSummary>,
        interfaces: Vec<InterfaceSummary>,
        signals: Vec<SignalSummary>,
    ) -> ImplementationUnderstanding {
        ImplementationUnderstanding {
            stage_id: "L0".to_string(),
            version: "3.0.0".to_string(),
            summary: StageSummary {
                short: "test".to_string(),
                detailed: "test".to_string(),
            },
            claims: vec![],
            module_summaries: vec![],
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

    fn make_step(name: &str, desc: &str, order: u32) -> ProcessingStepSummary {
        ProcessingStepSummary {
            name: name.to_string(),
            description: desc.to_string(),
            order,
            evidence_refs: vec![],
            confidence: ClaimConfidence::Supported,
        }
    }

    fn make_iface(name: &str, desc: &str) -> InterfaceSummary {
        InterfaceSummary {
            name: name.to_string(),
            description: desc.to_string(),
            interface_type: None,
            evidence_refs: vec![],
            confidence: ClaimConfidence::Supported,
        }
    }

    // ─── df_01: 正常 IU → input/step/output nodes ────────────────────

    #[test]
    fn df_01_normal_iu_generates_dataflow_nodes() {
        let iu = make_iu(
            vec![
                make_step("normalize", "信号归一化", 1),
                make_step("filter", "低通滤波", 2),
            ],
            vec![
                make_iface("data_input", "数据输入接口"),
                make_iface("data_output", "数据输出接口"),
            ],
            vec![],
        );
        let graph = build_dataflow_view(&iu);

        assert_eq!(graph.view_type, ViewType::Dataflow);
        assert!(
            graph.nodes.iter().any(|n| n.node_type == NodeType::InputSource),
            "应有输入源节点"
        );
        assert!(
            graph.nodes.iter().any(|n| n.node_type == NodeType::OutputTarget),
            "应有输出目标节点"
        );
        let steps: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::ProcessingStep)
            .collect();
        assert_eq!(steps.len(), 2);
    }

    // ─── df_02: processing steps 顺序边 ──────────────────────────────

    #[test]
    fn df_02_sequential_edges_correct() {
        let iu = make_iu(
            vec![
                make_step("step_a", "步骤 A", 1),
                make_step("step_b", "步骤 B", 2),
                make_step("step_c", "步骤 C", 3),
            ],
            vec![
                make_iface("input_signal", "输入"),
                make_iface("output_signal", "输出"),
            ],
            vec![],
        );
        let graph = build_dataflow_view(&iu);

        let step_edges: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| e.edge_type == EdgeType::DataFlow)
            .collect();
        assert!(!step_edges.is_empty(), "应有 data flow 边");
    }

    // ─── df_03: 无 processing_steps → 空图 ───────────────────────────

    #[test]
    fn df_03_no_steps_empty_graph() {
        let iu = make_iu(vec![], vec![], vec![]);
        let graph = build_dataflow_view(&iu);
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
        assert!(graph.meta.empty_reason.is_some());
    }

    // ─── df_04: node_id 唯一，edge endpoint 存在 ─────────────────────

    #[test]
    fn df_04_node_ids_unique_edge_endpoints_exist() {
        let iu = make_iu(
            vec![make_step("proc", "处理", 1)],
            vec![
                make_iface("input_port", "输入端口"),
                make_iface("output_port", "输出端口"),
            ],
            vec![],
        );
        let graph = build_dataflow_view(&iu);

        let node_ids: std::collections::HashSet<&str> =
            graph.nodes.iter().map(|n| n.node_id.as_str()).collect();
        assert_eq!(node_ids.len(), graph.nodes.len(), "node_id 不应重复");

        for edge in &graph.edges {
            assert!(
                node_ids.contains(edge.source_node_id.as_str()),
                "edge source not in nodes"
            );
            assert!(
                node_ids.contains(edge.target_node_id.as_str()),
                "edge target not in nodes"
            );
        }
    }

    // ─── df_05: steps 按 order 排序 ──────────────────────────────────

    #[test]
    fn df_05_steps_sorted_by_order() {
        let iu = make_iu(
            vec![
                make_step("third", "第三", 3),
                make_step("first", "第一", 1),
                make_step("second", "第二", 2),
            ],
            vec![],
            vec![],
        );
        let graph = build_dataflow_view(&iu);
        let step_nodes: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::ProcessingStep)
            .collect();
        assert_eq!(step_nodes[0].label, "first");
        assert_eq!(step_nodes[1].label, "second");
        assert_eq!(step_nodes[2].label, "third");
    }
}
