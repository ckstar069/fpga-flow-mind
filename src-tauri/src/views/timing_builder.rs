use crate::understanding::models::{
    ClaimConfidence, EvidenceRef, ImplementationUnderstanding,
};
use crate::views::models::{
    EdgeType, NodeType, ViewEdge, ViewGraph, ViewLayoutHint, ViewMeta, ViewNode, ViewTraceRef,
    ViewType,
};

/// 判断 processing_steps 或 claims/signals 是否含明确时序证据。
///
/// 明确时序依据包括：
/// - step.description / step.name / claim 中出现 cycle / latency / clock / pipeline /
///   stage / tick / clk / rst / reset / posedge / negedge 等时序关键词；
/// - stage_id 或 source_kind 明确属于 RTL / L3_pipeline / L4_cycle_acc；
/// - evidence 内容显示 RTL clock/reset/always_ff/posedge/negedge。
fn has_temporal_evidence(iu: &ImplementationUnderstanding) -> bool {
    let temporal_keywords = [
        "cycle", "latency", "clock", "pipeline", "stage", "tick",
        "clk", "rst", "reset", "posedge", "negedge", "always_ff",
        "always@", "always @", "时钟", "流水", "时序", "复位",
        // Phase 8: L4 cycle-accurate 相关关键词
        "_stage_", "cycle_count", "cycle_acc", "cycle-accurate", "pipelinetiming",
    ];

    // 1. 检查 processing_steps 的 name / description
    for step in &iu.processing_steps {
        let text = format!("{} {}", step.name, step.description).to_lowercase();
        if temporal_keywords.iter().any(|kw| text.contains(kw)) {
            return true;
        }
    }

    // 2. 检查 claims 的 description（clock/reset 相关声明）
    for claim in &iu.claims {
        let desc_lower = claim.description.to_lowercase();
        if temporal_keywords.iter().any(|kw| desc_lower.contains(kw)) {
            return true;
        }
    }

    // 3. 检查 signal_summaries 的 name（clk/rst 信号）
    for sig in &iu.signal_summaries {
        let name_lower = sig.name.to_lowercase();
        if name_lower.contains("clk") || name_lower.contains("clock")
            || name_lower.contains("rst") || name_lower.contains("reset")
        {
            return true;
        }
    }

    // 4. L4 / cycle_acc 语义门控：阶段明确为周期精确时，
    //    只有 processing_steps 的 name/description 命中周期精确语义关键词才视为时序证据。
    //    不再“有 processing_steps 就生成 timing”，避免普通函数顺序被伪造成硬件时序。
    let stage_lower = iu.stage_id.to_lowercase();
    let is_l4_or_cycle_acc = stage_lower.starts_with("l4") || stage_lower.contains("cycle_acc");
    if is_l4_or_cycle_acc {
        let l4_semantic_keywords = [
            "input", "correlation", "energy", "metric", "detection", "output",
            "_stage_", "cycle", "latency", "pipeline", "clock", "stage",
            "s_valid", "s_data", "s_last", "s_ready",
            "m_valid", "m_data", "m_last", "m_ready",
        ];
        for step in &iu.processing_steps {
            let text = format!("{} {}", step.name, step.description).to_lowercase();
            if l4_semantic_keywords.iter().any(|kw| text.contains(kw)) {
                return true;
            }
        }
    }

    false
}

/// 从 ImplementationUnderstanding 构建时序/流水图
pub fn build_timing_view(iu: &ImplementationUnderstanding) -> ViewGraph {
    let mut node_counter: u32 = 0;
    let mut edge_counter: u32 = 0;
    let mut nodes: Vec<ViewNode> = Vec::new();
    let mut edges: Vec<ViewEdge> = Vec::new();

    // ── 流水阶段节点（按 order 排序） ──
    // P0-3 收口：禁止将普通 Python 函数顺序伪造成硬件时序图。
    // 只有存在明确时序依据时，才允许从 processing_steps 生成 PipelineStage 节点。
    let has_temporal = has_temporal_evidence(iu);
    let mut sorted_steps: Vec<_> = iu.processing_steps.iter().collect();
    sorted_steps.sort_by_key(|s| s.order);

    if has_temporal {
        for step in &sorted_steps {
            node_counter += 1;
            let node_id = format!("N-timing-{:04}", node_counter);
            nodes.push(ViewNode {
                node_id,
                node_type: NodeType::PipelineStage,
                label: step.name.clone(),
                description: step.description.clone(),
                confidence: step.confidence,
                trace_refs: build_trace_refs(&step.evidence_refs),
                layout: Some(ViewLayoutHint {
                    column: Some(0),
                    row: Some(node_counter - 1),
                    depth: Some(0),
                    group: None,
                }),
            });
        }
    }

    // 记录由 processing_steps 派生的 stage 数量，用于 RTL 回退判断
    let stage_ids_from_steps: Vec<String> = nodes
        .iter()
        .filter(|n| n.node_type == NodeType::PipelineStage)
        .map(|n| n.node_id.clone())
        .collect();

    // ── 时钟/复位域节点（从 claims 匹配） ──
    let mut clock_node_ids: Vec<String> = Vec::new();
    let mut reset_node_ids: Vec<String> = Vec::new();

    for claim in &iu.claims {
        let desc_lower = claim.description.to_lowercase();
        if desc_lower.contains("clock") || desc_lower.contains("clk") {
            node_counter += 1;
            let node_id = format!("N-timing-{:04}", node_counter);
            nodes.push(ViewNode {
                node_id: node_id.clone(),
                node_type: NodeType::ClockDomain,
                label: format!("clock_{}", node_counter),
                description: claim.description.clone(),
                confidence: claim.confidence.clone(),
                trace_refs: build_trace_refs(&claim.evidence_refs),
                layout: Some(ViewLayoutHint {
                    column: Some(1),
                    row: Some(node_counter - 1),
                    depth: Some(0),
                    group: None,
                }),
            });
            clock_node_ids.push(node_id);
        }
        if desc_lower.contains("reset") || desc_lower.contains("rst") {
            node_counter += 1;
            let node_id = format!("N-timing-{:04}", node_counter);
            nodes.push(ViewNode {
                node_id: node_id.clone(),
                node_type: NodeType::ResetDomain,
                label: format!("reset_{}", node_counter),
                description: claim.description.clone(),
                confidence: claim.confidence.clone(),
                trace_refs: build_trace_refs(&claim.evidence_refs),
                layout: Some(ViewLayoutHint {
                    column: Some(1),
                    row: Some(node_counter - 1),
                    depth: Some(0),
                    group: None,
                }),
            });
            reset_node_ids.push(node_id);
        }
    }

    // ── P0-3 RTL 时序保守回退 ──
    // 当 processing_steps 为空（RTL 阶段通常如此）但 signal_summaries 中含
    // clk/rst 信号（由 MockProvider 从 RTL evidence 保守派生），生成 clock_domain /
    // pipeline_stage 节点，绑定 signal 的 evidence_refs。这是可追溯的最小时序图，
    // 不伪造硬件时序关系。
    if stage_ids_from_steps.is_empty() {
        for sig in &iu.signal_summaries {
            let name_lower = sig.name.to_lowercase();
            let is_clock = name_lower.contains("clk") || name_lower.contains("clock");
            let is_reset = name_lower.contains("rst") || name_lower.contains("reset");
            if !is_clock && !is_reset {
                continue;
            }
            node_counter += 1;
            let node_id = format!("N-timing-{:04}", node_counter);
            let node_type = if is_clock {
                NodeType::ClockDomain
            } else {
                NodeType::ResetDomain
            };
            nodes.push(ViewNode {
                node_id: node_id.clone(),
                node_type,
                label: sig.name.clone(),
                description: sig.description.clone(),
                confidence: sig.confidence,
                trace_refs: build_trace_refs(&sig.evidence_refs),
                layout: Some(ViewLayoutHint {
                    column: Some(1),
                    row: Some(node_counter - 1),
                    depth: Some(0),
                    group: None,
                }),
            });
            if is_clock {
                clock_node_ids.push(node_id);
            } else {
                reset_node_ids.push(node_id);
            }
        }
    }

    // ── 边：PipelineStage[i] → PipelineStage[i+1] ──
    let stage_ids: Vec<String> = nodes
        .iter()
        .filter(|n| n.node_type == NodeType::PipelineStage)
        .map(|n| n.node_id.clone())
        .collect();

    for i in 0..stage_ids.len().saturating_sub(1) {
        edge_counter += 1;
        let refs = merge_node_trace_refs(&nodes, &stage_ids[i], &stage_ids[i + 1]);
        let step_desc = if let (Some(a), Some(b)) = (
            nodes.iter().find(|n| n.node_id == stage_ids[i]),
            nodes.iter().find(|n| n.node_id == stage_ids[i + 1]),
        ) {
            // 检查是否含流水描述，决定边类型
            let is_pipeline = b.description.to_lowercase().contains("pipe")
                || b.description.to_lowercase().contains("流水")
                || b.description.to_lowercase().contains("pipeline");
            let edge_type = if is_pipeline {
                EdgeType::PipelineForward
            } else {
                EdgeType::SequentialOrder
            };
            (edge_type, format!("阶段 {} → {}", a.label, b.label))
        } else {
            (EdgeType::SequentialOrder, String::new())
        };

        edges.push(ViewEdge {
            edge_id: format!("E-timing-{:04}", edge_counter),
            edge_type: step_desc.0,
            source_node_id: stage_ids[i].clone(),
            target_node_id: stage_ids[i + 1].clone(),
            label: None,
            description: step_desc.1,
            confidence: ClaimConfidence::Inferred,
            trace_refs: refs,
        });
    }

    // ── 边：ClockDomain → PipelineStage[0] ──
    for clock_id in &clock_node_ids {
        if let Some(first_stage) = stage_ids.first() {
            edge_counter += 1;
            let refs = merge_node_trace_refs(&nodes, clock_id, first_stage);
            edges.push(ViewEdge {
                edge_id: format!("E-timing-{:04}", edge_counter),
                edge_type: EdgeType::ClockDriven,
                source_node_id: clock_id.clone(),
                target_node_id: first_stage.clone(),
                label: None,
                description: "时钟驱动第一个流水级".to_string(),
                confidence: ClaimConfidence::Inferred,
                trace_refs: refs,
            });
        }
    }

    // ── 元信息 ──
    // P0-3 收口：明确区分三种空图原因
    let empty_reason = if nodes.is_empty() {
        if !iu.processing_steps.is_empty() && !has_temporal {
            // 有 processing_steps 但无时序依据 → 明确说明这是 Python 函数顺序，非硬件时序
            Some(
                "无 cycle/latency/clock/pipeline 等可追溯时序证据，未生成 timing 图（当前 processing_steps 为算法/函数顺序，非硬件时序）"
                    .to_string(),
            )
        } else if iu.processing_steps.is_empty() && clock_node_ids.is_empty() && reset_node_ids.is_empty() {
            Some(
                "时序为空：当前阶段无 processing_steps，且无可追溯的 clock/reset/pipeline 证据"
                    .to_string(),
            )
        } else {
            Some(
                "时序信息不足：当前阶段无 processing_steps 且无 clock/reset 相关声明".to_string(),
            )
        }
    } else {
        None
    };

    let meta = ViewMeta {
        stage_id: iu.stage_id.clone(),
        view_type: ViewType::Timing,
        source_provider: iu.generation_meta.provider.clone(),
        is_degraded_source: iu.generation_meta.is_degraded,
        generated_at: chrono::Utc::now().to_rfc3339(),
        empty_reason,
    };

    ViewGraph {
        view_type: ViewType::Timing,
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

/// 合并两个端点节点的 trace_refs 作为边的 trace_refs（与 dataflow_builder 同语义）。
fn merge_node_trace_refs(
    nodes: &[ViewNode],
    source_id: &str,
    target_id: &str,
) -> Vec<ViewTraceRef> {
    let mut refs: Vec<ViewTraceRef> = Vec::new();
    let mut seen_ev: std::collections::HashSet<String> = std::collections::HashSet::new();
    for node in nodes {
        if node.node_id == source_id || node.node_id == target_id {
            for tr in &node.trace_refs {
                if let Some(ev) = &tr.evidence_id {
                    if seen_ev.insert(ev.clone()) {
                        refs.push(tr.clone());
                    }
                }
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
        ClaimCategory, EvidenceRef, GenerationMeta, ImplementationClaim,
        ImplementationUnderstanding, ProcessingStepSummary, SignalSummary, StageSummary,
        UnderstandingStats,
    };
    use std::collections::HashMap;

    fn make_iu(
        steps: Vec<ProcessingStepSummary>,
        claims: Vec<ImplementationClaim>,
    ) -> ImplementationUnderstanding {
        make_iu_full(steps, claims, vec![])
    }

    fn make_iu_full(
        steps: Vec<ProcessingStepSummary>,
        claims: Vec<ImplementationClaim>,
        signals: Vec<SignalSummary>,
    ) -> ImplementationUnderstanding {
        ImplementationUnderstanding {
            stage_id: "L0".to_string(),
            version: "3.0.0".to_string(),
            summary: StageSummary {
                short: "test".to_string(),
                detailed: "test".to_string(),
            },
            claims,
            module_summaries: vec![],
            signal_summaries: signals,
            interface_summaries: vec![],
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

    fn make_claim(id: &str, desc: &str) -> ImplementationClaim {
        ImplementationClaim {
            claim_id: id.to_string(),
            category: ClaimCategory::Configuration,
            description: desc.to_string(),
            confidence: ClaimConfidence::Supported,
            evidence_refs: vec![],
            has_evidence_gap: false,
        }
    }

    // ─── tm_01: processing_steps 含时序关键词 → pipeline_stage nodes ─────────

    #[test]
    fn tm_01_steps_become_pipeline_stages() {
        let iu = make_iu(
            vec![
                make_step("fetch", "取指阶段，每个 clock cycle 执行", 1),
                make_step("decode", "译码 pipeline stage", 2),
                make_step("execute", "执行阶段", 3),
            ],
            vec![],
        );
        let graph = build_timing_view(&iu);

        let stages: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::PipelineStage)
            .collect();
        assert_eq!(stages.len(), 3);
        assert_eq!(stages[0].label, "fetch");
        assert_eq!(stages[1].label, "decode");
        assert_eq!(stages[2].label, "execute");
    }

    // ─── tm_02: clock/reset claims → ClockDomain/ResetDomain ─────────

    #[test]
    fn tm_02_clock_claims_generate_clock_domain() {
        let iu = make_iu(
            vec![make_step("process", "处理", 1)],
            vec![
                make_claim("CL-001", "主时钟 clk_100mhz 驱动系统"),
                make_claim("CL-002", "复位信号 rst_n 初始化状态"),
            ],
        );
        let graph = build_timing_view(&iu);

        assert!(
            graph
                .nodes
                .iter()
                .any(|n| n.node_type == NodeType::ClockDomain),
            "应有 ClockDomain 节点"
        );
        assert!(
            graph
                .nodes
                .iter()
                .any(|n| n.node_type == NodeType::ResetDomain),
            "应有 ResetDomain 节点"
        );
    }

    // ─── tm_03: pipeline stage 顺序边（含时序关键词）────────────────────────

    #[test]
    fn tm_03_sequential_edges_correct() {
        let iu = make_iu(
            vec![
                make_step("s1", "pipeline 阶段 1", 1),
                make_step("s2", "pipeline 阶段 2", 2),
            ],
            vec![],
        );
        let graph = build_timing_view(&iu);

        let timing_edges: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| {
                e.edge_type == EdgeType::SequentialOrder
                    || e.edge_type == EdgeType::PipelineForward
            })
            .collect();
        assert!(!timing_edges.is_empty(), "应有顺序边");
    }

    // ─── tm_04: 无 timing 信息 → 空图 empty_reason ───────────────────

    #[test]
    fn tm_04_no_timing_empty_graph() {
        let iu = make_iu(vec![], vec![]);
        let graph = build_timing_view(&iu);
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
        assert!(graph.meta.empty_reason.is_some());
        // P0-3: 空图原因应明确说明缺少可追溯时序依据
        let reason = graph.meta.empty_reason.as_deref().unwrap();
        assert!(
            reason.contains("processing_steps") || reason.contains("clock") || reason.contains("时序"),
            "empty_reason 应说明时序依据缺失: {}",
            reason
        );
    }

    // ─── tm_05: 不生成伪节点 ─────────────────────────────────────────

    #[test]
    fn tm_05_no_pseudo_nodes() {
        let iu = make_iu(vec![], vec![]);
        let graph = build_timing_view(&iu);
        // 空 timing 应直接返回 nodes=[]，不创建任何占位节点
        assert_eq!(graph.nodes.len(), 0);
        // 确认没有无 trace_refs 的伪 timing 节点
        for node in &graph.nodes {
            assert!(
                !node.trace_refs.is_empty() || !node.description.is_empty(),
                "不应有既无 trace 又无描述的伪节点"
            );
        }
    }

    // ─── tm_06: node_id 唯一，edge endpoint 存在（含 clk claim 触发时序）────

    #[test]
    fn tm_06_node_ids_unique_edge_endpoints_exist() {
        let iu = make_iu(
            vec![
                make_step("a", "A pipeline stage", 1),
                make_step("b", "B pipeline stage", 2),
            ],
            vec![make_claim("CL-001", "clk signal")],
        );
        let graph = build_timing_view(&iu);

        let node_ids: std::collections::HashSet<&str> =
            graph.nodes.iter().map(|n| n.node_id.as_str()).collect();
        assert_eq!(node_ids.len(), graph.nodes.len());

        for edge in &graph.edges {
            assert!(node_ids.contains(edge.source_node_id.as_str()));
            assert!(node_ids.contains(edge.target_node_id.as_str()));
        }
    }

    // ─── P0-3: 边可追溯 + RTL 时钟回退测试 ──────────────────────────────

    fn make_signal_with_ev(name: &str, desc: &str, direction: Option<&str>, ev_id: &str) -> SignalSummary {
        SignalSummary {
            name: name.to_string(),
            description: desc.to_string(),
            direction: direction.map(|s| s.to_string()),
            evidence_refs: vec![EvidenceRef { evidence_id: ev_id.to_string(), relevance: None }],
            confidence: ClaimConfidence::Inferred,
        }
    }

    fn make_step_with_ev(name: &str, desc: &str, order: u32, ev_id: &str) -> ProcessingStepSummary {
        ProcessingStepSummary {
            name: name.to_string(),
            description: desc.to_string(),
            order,
            evidence_refs: vec![EvidenceRef { evidence_id: ev_id.to_string(), relevance: None }],
            confidence: ClaimConfidence::Supported,
        }
    }

    /// tm_07: RTL 阶段无 processing_steps，但有 clk/rst signal → 生成 clock/reset 节点
    #[test]
    fn tm_07_rtl_signal_fallback_clock_nodes() {
        // RTL 阶段：无 steps，但 signal_summaries 含 clk / rst_n（由 MockProvider 从 RTL evidence 派生）
        let iu = make_iu_full(
            vec![],
            vec![],
            vec![
                make_signal_with_ev("clk", "时钟", Some("input"), "EV-RTL-1"),
                make_signal_with_ev("rst_n", "复位", Some("input"), "EV-RTL-2"),
            ],
        );
        let graph = build_timing_view(&iu);

        // 应生成 ClockDomain 与 ResetDomain 节点
        assert!(
            graph.nodes.iter().any(|n| n.node_type == NodeType::ClockDomain),
            "RTL clk signal 应生成 ClockDomain 节点"
        );
        assert!(
            graph.nodes.iter().any(|n| n.node_type == NodeType::ResetDomain),
            "RTL rst signal 应生成 ResetDomain 节点"
        );
        // 每个节点绑定 evidence
        for node in &graph.nodes {
            assert!(!node.trace_refs.is_empty(), "节点 {} 缺少 trace_refs", node.node_id);
        }
        // 非空 → 无 empty_reason
        assert!(graph.meta.empty_reason.is_none(), "有节点时不应有 empty_reason");
    }

    /// tm_08: Python 阶段无时序依据 → 空，empty_reason 明确
    #[test]
    fn tm_08_python_no_timing_stays_empty() {
        // Python 阶段：无 steps，无 clk/rst 信号（普通数据信号不触发时序回退）
        let iu = make_iu_full(
            vec![],
            vec![],
            vec![make_signal_with_ev("rx_data", "接收数据", Some("input"), "EV-1")],
        );
        let graph = build_timing_view(&iu);
        assert!(graph.nodes.is_empty(), "Python 无时序依据应保持空");
        let reason = graph.meta.empty_reason.as_deref().unwrap();
        assert!(
            reason.contains("clock") || reason.contains("时序") || reason.contains("processing_steps"),
            "empty_reason 应说明缺时序依据: {}",
            reason
        );
    }

    /// tm_09: processing_steps 顺序边有 trace_refs（含时序关键词）
    #[test]
    fn tm_09_stage_edges_carry_trace_refs() {
        let iu = make_iu_full(
            vec![
                make_step_with_ev("fetch", "取指 pipeline stage", 1, "EV-1"),
                make_step_with_ev("decode", "译码 pipeline stage", 2, "EV-2"),
                make_step_with_ev("execute", "执行 pipeline stage", 3, "EV-3"),
            ],
            vec![],
            vec![],
        );
        let graph = build_timing_view(&iu);
        let seq_edges: Vec<_> = graph.edges.iter()
            .filter(|e| e.edge_type == EdgeType::SequentialOrder || e.edge_type == EdgeType::PipelineForward)
            .collect();
        assert!(seq_edges.len() >= 2, "应有至少 2 条顺序边");
        for e in &seq_edges {
            assert!(!e.trace_refs.is_empty(), "时序边 {} 缺少 trace_refs", e.edge_id);
        }
    }

    /// tm_10: RTL clk signal 节点的 trace_refs 指向 evidence_id
    #[test]
    fn tm_10_rtl_clock_trace_resolves_to_evidence() {
        let iu = make_iu_full(
            vec![],
            vec![],
            vec![make_signal_with_ev("clk", "主时钟", Some("input"), "EV-RTL-000001")],
        );
        let graph = build_timing_view(&iu);
        let clock_node = graph.nodes.iter().find(|n| n.node_type == NodeType::ClockDomain);
        assert!(clock_node.is_some(), "应有 ClockDomain 节点");
        let ev_ids: Vec<&str> = clock_node.unwrap()
            .trace_refs.iter()
            .filter_map(|t| t.evidence_id.as_deref())
            .collect();
        assert!(ev_ids.contains(&"EV-RTL-000001"), "ClockDomain 节点应 trace 到 EV-RTL-000001, 实际: {:?}", ev_ids);
    }

    /// tm_11: Python 阶段含多个 processing_steps 但无时序关键词 → timing 为空，empty_reason 明确
    #[test]
    fn tm_11_python_steps_no_temporal_keywords_empty() {
        // 模拟 MockProvider 从 Python 函数/类符号派生的 processing_steps
        // 这些是算法/调用顺序，不是硬件 timing
        let iu = make_iu_full(
            vec![
                make_step_with_ev("load_samples", "加载采样数据", 1, "EV-L0-000001"),
                make_step_with_ev("correlate", "计算互相关", 2, "EV-L0-000002"),
                make_step_with_ev("detect_peak", "检测峰值", 3, "EV-L0-000003"),
                make_step_with_ev("estimate_cfo", "估计 CFO", 4, "EV-L0-000004"),
            ],
            vec![],
            vec![],
        );
        let graph = build_timing_view(&iu);

        // 必须为空图：普通 Python 函数顺序不得伪造成硬件时序
        assert!(graph.nodes.is_empty(), "Python 函数顺序无时序关键词时应保持空图，实际节点: {:?}", graph.nodes.iter().map(|n| &n.label).collect::<Vec<_>>());
        assert!(graph.edges.is_empty(), "空图不应有边");

        // empty_reason 必须非空且明确说明原因
        let reason = graph.meta.empty_reason.as_deref().unwrap_or("");
        assert!(
            reason.contains("cycle") || reason.contains("latency") || reason.contains("clock")
                || reason.contains("pipeline") || reason.contains("时序"),
            "empty_reason 应明确说明缺少时序证据: {}", reason
        );
        assert!(
            reason.contains("processing_steps") || reason.contains("算法/函数顺序"),
            "empty_reason 应说明 processing_steps 为算法/函数顺序: {}", reason
        );
    }

    fn make_claim_with_ev(id: &str, desc: &str, ev_id: &str) -> ImplementationClaim {
        ImplementationClaim {
            claim_id: id.to_string(),
            category: ClaimCategory::Configuration,
            description: desc.to_string(),
            confidence: ClaimConfidence::Supported,
            evidence_refs: vec![EvidenceRef { evidence_id: ev_id.to_string(), relevance: None }],
            has_evidence_gap: false,
        }
    }

    /// tm_12: RTL 含 always_ff/posedge 证据 → timing 可生成非空图，trace_refs 完整
    #[test]
    fn tm_12_rtl_always_ff_timing_non_empty() {
        // RTL 阶段：claims 含 always_ff/posedge 时序声明
        let iu = make_iu_full(
            vec![
                make_step_with_ev("sample_reg", "在 posedge clk 采样输入", 1, "EV-RTL-000001"),
                make_step_with_ev("corr_reg", "在 posedge clk 更新相关值", 2, "EV-RTL-000002"),
            ],
            vec![
                make_claim_with_ev("CL-RTL-001", "主时钟 clk 100MHz 驱动所有 always_ff", "EV-RTL-000005"),
                make_claim_with_ev("CL-RTL-002", "异步复位 rst_n 低电平有效", "EV-RTL-000006"),
            ],
            vec![
                make_signal_with_ev("clk", "100MHz 时钟", Some("input"), "EV-RTL-000003"),
                make_signal_with_ev("rst_n", "异步复位", Some("input"), "EV-RTL-000004"),
            ],
        );
        let graph = build_timing_view(&iu);

        // 应生成非空图：PipelineStage + ClockDomain + ResetDomain
        assert!(!graph.nodes.is_empty(), "RTL 含时序证据时应生成非空 timing 图");
        assert!(
            graph.nodes.iter().any(|n| n.node_type == NodeType::PipelineStage),
            "应有 PipelineStage 节点"
        );
        assert!(
            graph.nodes.iter().any(|n| n.node_type == NodeType::ClockDomain),
            "应有 ClockDomain 节点"
        );
        assert!(
            graph.nodes.iter().any(|n| n.node_type == NodeType::ResetDomain),
            "应有 ResetDomain 节点"
        );

        // 所有节点必须有 trace_refs
        for node in &graph.nodes {
            assert!(!node.trace_refs.is_empty(), "节点 {} 缺少 trace_refs", node.node_id);
        }

        // 边必须有 trace_refs
        for edge in &graph.edges {
            assert!(!edge.trace_refs.is_empty(), "边 {} 缺少 trace_refs", edge.edge_id);
        }

        // 非空 → 无 empty_reason
        assert!(graph.meta.empty_reason.is_none(), "有节点时不应有 empty_reason");
    }

    /// tm_13: L4 阶段含 _stage_* processing_steps → has_temporal_evidence 为 true
    #[test]
    fn tm_13_l4_stage_steps_have_temporal_evidence() {
        let iu = make_iu_full(
            vec![
                make_step_with_ev("input", "input", 1, "EV-L4-000001"),
                make_step_with_ev("correlation", "_stage_correlation", 2, "EV-L4-000002"),
                make_step_with_ev("energy", "_stage_energy", 3, "EV-L4-000003"),
                make_step_with_ev("metric", "_stage_metric", 4, "EV-L4-000004"),
                make_step_with_ev("detection", "_stage_detection", 5, "EV-L4-000005"),
                make_step_with_ev("output", "output", 6, "EV-L4-000006"),
            ],
            vec![],
            vec![],
        );
        // 复写 stage_id 为 L4
        let mut iu = iu;
        iu.stage_id = "L4_cycle_acc".to_string();
        let graph = build_timing_view(&iu);
        assert!(
            !graph.nodes.is_empty(),
            "L4 阶段 _stage_* processing_steps 应触发非空 timing 图，实际节点: {:?}",
            graph.nodes.iter().map(|n| &n.label).collect::<Vec<_>>()
        );
        assert!(
            graph.nodes.iter().any(|n| n.node_type == NodeType::PipelineStage),
            "应生成 PipelineStage 节点"
        );
        assert!(graph.meta.empty_reason.is_none(), "非空 timing 图不应有 empty_reason");
    }
}
