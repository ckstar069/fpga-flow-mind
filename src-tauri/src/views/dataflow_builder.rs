use crate::understanding::models::{
    ClaimConfidence, EvidenceRef, ImplementationUnderstanding, ProcessingStepSummary,
};
use crate::views::models::{
    EdgeType, NodeType, ViewEdge, ViewGraph, ViewLayoutHint, ViewMeta, ViewNode, ViewTraceRef,
    ViewType,
};

// ─── 输入/输出名称识别 helper ─────────────────────────────────────────

/// 判断名称是否表示输入信号/接口（基于 token 匹配，不使用纯 contains）
fn is_input_name(name: &str) -> bool {
    let tokens = tokenize(name);
    tokens.iter().any(|t| {
        matches!(
            t.as_str(),
            "input" | "in" | "rx" | "din" | "data_in" | "in_data" | "输入" | "接收"
        )
    }) || name.starts_with("in_")
        || name.ends_with("_in")
}

/// 判断名称是否表示输出信号/接口（基于 token 匹配）
fn is_output_name(name: &str) -> bool {
    let tokens = tokenize(name);
    tokens.iter().any(|t| {
        matches!(
            t.as_str(),
            "output" | "out" | "tx" | "dout" | "data_out" | "out_data" | "输出" | "发送"
        )
    }) || name.starts_with("out_")
        || name.ends_with("_out")
}

/// 将名称按分隔符拆分为 token
fn tokenize(name: &str) -> Vec<String> {
    name.to_lowercase()
        .split(|c: char| c == '_' || c == '-' || c == '.' || c == ' ')
        .map(|s| s.to_string())
        .collect()
}

/// 判断 SignalSummary.direction 是否表示输入
fn is_input_direction(direction: Option<&str>) -> bool {
    matches!(direction, Some("input") | Some("in") | Some("输入"))
}

/// 判断 SignalSummary.direction 是否表示输出
fn is_output_direction(direction: Option<&str>) -> bool {
    matches!(direction, Some("output") | Some("out") | Some("输出"))
}

// ─── 主构建函数 ───────────────────────────────────────────────────────

/// 从 ImplementationUnderstanding 构建数据流图
pub fn build_dataflow_view(iu: &ImplementationUnderstanding) -> ViewGraph {
    let mut node_counter: u32 = 0;
    let mut edge_counter: u32 = 0;
    let mut nodes: Vec<ViewNode> = Vec::new();
    let mut edges: Vec<ViewEdge> = Vec::new();

    // ── 输入源节点（interface_summaries + signal_summaries） ──
    for iface in &iu.interface_summaries {
        if is_input_name(&iface.name) {
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

    for sig in &iu.signal_summaries {
        let is_input = is_input_direction(sig.direction.as_deref())
            || (sig.direction.is_none() && is_input_name(&sig.name));
        if is_input {
            node_counter += 1;
            let node_id = format!("N-dataflow-{:04}", node_counter);
            nodes.push(ViewNode {
                node_id,
                node_type: NodeType::InputSource,
                label: sig.name.clone(),
                description: sig.description.clone(),
                confidence: sig.confidence,
                trace_refs: build_trace_refs(&sig.evidence_refs),
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

    // ── 输出目标节点（interface_summaries + signal_summaries） ──
    for iface in &iu.interface_summaries {
        if is_output_name(&iface.name) {
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

    for sig in &iu.signal_summaries {
        let is_output = is_output_direction(sig.direction.as_deref())
            || (sig.direction.is_none() && is_output_name(&sig.name));
        if is_output {
            node_counter += 1;
            let node_id = format!("N-dataflow-{:04}", node_counter);
            nodes.push(ViewNode {
                node_id,
                node_type: NodeType::OutputTarget,
                label: sig.name.clone(),
                description: sig.description.clone(),
                confidence: sig.confidence,
                trace_refs: build_trace_refs(&sig.evidence_refs),
                layout: Some(ViewLayoutHint {
                    column: Some(2),
                    row: Some(node_counter - 1),
                    depth: Some(0),
                    group: None,
                }),
            });
        }
    }

    // ── 中间数据节点（非输入/非输出的信号） ──
    for sig in &iu.signal_summaries {
        let dir_input = is_input_direction(sig.direction.as_deref());
        let dir_output = is_output_direction(sig.direction.as_deref());
        let name_input = is_input_name(&sig.name);
        let name_output = is_output_name(&sig.name);
        let is_internal = sig
            .direction
            .as_deref()
            .map(|d| d == "internal")
            .unwrap_or(false);

        // 中间数据：direction=internal，或方向/名称都不是 I/O
        if is_internal || (!dir_input && !dir_output && !name_input && !name_output) {
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
        GenerationMeta, ImplementationUnderstanding, InterfaceSummary, ProcessingStepSummary,
        SignalSummary, StageSummary, UnderstandingStats,
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
            summary: StageSummary { short: "test".to_string(), detailed: "test".to_string() },
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
                total_claims: 0, claims_by_confidence: HashMap::new(), claims_by_category: HashMap::new(),
                module_count: 0, signal_count: 0, interface_count: 0, processing_step_count: 0,
                unknown_count: 0, evidence_gap_count: 0,
            },
        }
    }

    fn make_step(name: &str, desc: &str, order: u32) -> ProcessingStepSummary {
        ProcessingStepSummary {
            name: name.to_string(), description: desc.to_string(), order,
            evidence_refs: vec![], confidence: ClaimConfidence::Supported,
        }
    }

    fn make_iface(name: &str, desc: &str) -> InterfaceSummary {
        InterfaceSummary {
            name: name.to_string(), description: desc.to_string(),
            interface_type: None, evidence_refs: vec![], confidence: ClaimConfidence::Supported,
        }
    }

    fn make_signal(name: &str, desc: &str, direction: Option<&str>) -> SignalSummary {
        SignalSummary {
            name: name.to_string(), description: desc.to_string(),
            direction: direction.map(|s| s.to_string()),
            evidence_refs: vec![], confidence: ClaimConfidence::Supported,
        }
    }

    // ─── df_01: 正常 IU → input/step/output nodes ────────────────────
    #[test]
    fn df_01_normal_iu_generates_dataflow_nodes() {
        let iu = make_iu(
            vec![make_step("normalize", "归一化", 1), make_step("filter", "滤波", 2)],
            vec![make_iface("data_input", "输入"), make_iface("data_output", "输出")],
            vec![],
        );
        let graph = build_dataflow_view(&iu);
        assert_eq!(graph.view_type, ViewType::Dataflow);
        assert!(graph.nodes.iter().any(|n| n.node_type == NodeType::InputSource));
        assert!(graph.nodes.iter().any(|n| n.node_type == NodeType::OutputTarget));
        assert_eq!(graph.nodes.iter().filter(|n| n.node_type == NodeType::ProcessingStep).count(), 2);
    }

    // ─── df_02: processing steps 顺序边 ──────────────────────────────
    #[test]
    fn df_02_sequential_edges_correct() {
        let iu = make_iu(
            vec![make_step("a", "A", 1), make_step("b", "B", 2), make_step("c", "C", 3)],
            vec![make_iface("input_port", "输入"), make_iface("output_port", "输出")],
            vec![],
        );
        let graph = build_dataflow_view(&iu);
        let step_edges: Vec<_> = graph.edges.iter().filter(|e| e.edge_type == EdgeType::DataFlow).collect();
        assert!(step_edges.len() >= 2, "至少应有 input→s1 和 s1→s2");
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
            vec![make_iface("input_port", "输入"), make_iface("output_port", "输出")],
            vec![],
        );
        let graph = build_dataflow_view(&iu);
        let ids: std::collections::HashSet<&str> = graph.nodes.iter().map(|n| n.node_id.as_str()).collect();
        assert_eq!(ids.len(), graph.nodes.len());
        for e in &graph.edges {
            assert!(ids.contains(e.source_node_id.as_str()));
            assert!(ids.contains(e.target_node_id.as_str()));
        }
    }

    // ─── df_05: steps 按 order 排序 ──────────────────────────────────
    #[test]
    fn df_05_steps_sorted_by_order() {
        let iu = make_iu(
            vec![make_step("third", "三", 3), make_step("first", "一", 1), make_step("second", "二", 2)],
            vec![],
            vec![],
        );
        let graph = build_dataflow_view(&iu);
        let steps: Vec<_> = graph.nodes.iter().filter(|n| n.node_type == NodeType::ProcessingStep).collect();
        assert_eq!(steps[0].label, "first");
        assert_eq!(steps[1].label, "second");
        assert_eq!(steps[2].label, "third");
    }

    // ─── df_06: main_bus 不应被误判为 input ─────────────────────────
    #[test]
    fn df_06_main_bus_not_input() {
        let graph = build_dataflow_view(&make_iu(
            vec![],
            vec![make_iface("main_bus", "主总线")],
            vec![],
        ));
        assert!(graph.nodes.is_empty(), "main_bus 不应被识别为 input");
        assert!(graph.meta.empty_reason.is_some());
    }

    // ─── df_07: filtering_stage 不应被误判为 input ──────────────────
    #[test]
    fn df_07_filtering_stage_not_input() {
        let graph = build_dataflow_view(&make_iu(
            vec![],
            vec![make_iface("filtering_stage", "滤波阶段")],
            vec![],
        ));
        // filtering_stage 含 "in" 片段但不应被误判
        assert!(graph.nodes.is_empty(), "filtering_stage 不应被识别为 input");
    }

    // ─── df_08: signal direction=input → InputSource ─────────────────
    #[test]
    fn df_08_signal_direction_input() {
        let iu = make_iu(
            vec![],
            vec![],
            vec![make_signal("data", "数据", Some("input"))],
        );
        let graph = build_dataflow_view(&iu);
        assert!(graph.nodes.iter().any(|n| n.node_type == NodeType::InputSource));
    }

    // ─── df_09: signal direction=output → OutputTarget ──────────────
    #[test]
    fn df_09_signal_direction_output() {
        let iu = make_iu(
            vec![],
            vec![],
            vec![make_signal("result", "结果", Some("output"))],
        );
        let graph = build_dataflow_view(&iu);
        assert!(graph.nodes.iter().any(|n| n.node_type == NodeType::OutputTarget));
    }

    // ─── df_10: signal direction=internal → IntermediateData ─────────
    #[test]
    fn df_10_signal_direction_internal() {
        let iu = make_iu(
            vec![],
            vec![],
            vec![make_signal("temp", "临时", Some("internal"))],
        );
        let graph = build_dataflow_view(&iu);
        assert!(graph.nodes.iter().any(|n| n.node_type == NodeType::IntermediateData));
    }

    // ─── df_11: 无 steps 但有 io signal → nodes 非空 ────────────────
    #[test]
    fn df_11_no_steps_with_io_signals() {
        let iu = make_iu(
            vec![],
            vec![],
            vec![
                make_signal("rx_data", "接收数据", None),       // ends_with _in? no, starts with rx_? yes rx token
                make_signal("tx_data", "发送数据", None),       // tx token
                make_signal("reg_val", "寄存器值", Some("internal")),
            ],
        );
        let graph = build_dataflow_view(&iu);
        // rx_data: token "rx" → InputSource
        // tx_data: token "tx" → OutputTarget
        // reg_val: direction=internal → IntermediateData
        assert!(graph.nodes.iter().any(|n| n.node_type == NodeType::InputSource), "rx 应识别为 input");
        assert!(graph.nodes.iter().any(|n| n.node_type == NodeType::OutputTarget), "tx 应识别为 output");
        assert!(graph.nodes.iter().any(|n| n.node_type == NodeType::IntermediateData), "internal 应为中间数据");
        assert!(!graph.nodes.is_empty());
        // 无 steps → 不臆造边
        let data_edges: Vec<_> = graph.edges.iter().filter(|e| e.edge_type == EdgeType::DataFlow).collect();
        assert_eq!(data_edges.len(), 0, "无 processing_steps 时不臆造 data flow 边");
    }

    // ─── df_12: 完全无数据 → 空图 + empty_reason ────────────────────
    #[test]
    fn df_12_completely_empty() {
        let iu = make_iu(
            vec![],
            vec![],
            vec![make_signal("clk", "时钟", None)],
        );
        // clk 不含 io token，无 direction → 应为 IntermediateData
        let graph = build_dataflow_view(&iu);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].node_type, NodeType::IntermediateData);
        assert!(graph.meta.empty_reason.is_none(), "有节点时不应有 empty_reason");
    }
}
