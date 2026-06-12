use crate::understanding::models::{
    ClaimConfidence, EvidenceRef, ImplementationUnderstanding,
};
use crate::views::models::{
    EdgeType, NodeType, ViewEdge, ViewGraph, ViewLayoutHint, ViewMeta, ViewNode, ViewTraceRef,
    ViewType,
};

/// 从 ImplementationUnderstanding 构建时序/流水图
pub fn build_timing_view(iu: &ImplementationUnderstanding) -> ViewGraph {
    let mut node_counter: u32 = 0;
    let mut edge_counter: u32 = 0;
    let mut nodes: Vec<ViewNode> = Vec::new();
    let mut edges: Vec<ViewEdge> = Vec::new();

    // ── 流水阶段节点（按 order 排序） ──
    let mut sorted_steps: Vec<_> = iu.processing_steps.iter().collect();
    sorted_steps.sort_by_key(|s| s.order);

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

    // ── 边：PipelineStage[i] → PipelineStage[i+1] ──
    let stage_ids: Vec<String> = nodes
        .iter()
        .filter(|n| n.node_type == NodeType::PipelineStage)
        .map(|n| n.node_id.clone())
        .collect();

    for i in 0..stage_ids.len().saturating_sub(1) {
        edge_counter += 1;
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
            trace_refs: vec![],
        });
    }

    // ── 边：ClockDomain → PipelineStage[0] ──
    for clock_id in &clock_node_ids {
        if let Some(first_stage) = stage_ids.first() {
            edge_counter += 1;
            edges.push(ViewEdge {
                edge_id: format!("E-timing-{:04}", edge_counter),
                edge_type: EdgeType::ClockDriven,
                source_node_id: clock_id.clone(),
                target_node_id: first_stage.clone(),
                label: None,
                description: "时钟驱动第一个流水级".to_string(),
                confidence: ClaimConfidence::Inferred,
                trace_refs: vec![],
            });
        }
    }

    // ── 元信息 ──
    let empty_reason = if nodes.is_empty() {
        Some(
            "时序信息不足：当前阶段无 processing_steps 且无 clock/reset 相关声明".to_string(),
        )
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

// ─── 测试 ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::understanding::models::{
        ClaimCategory, EvidenceRef, GenerationMeta, ImplementationClaim,
        ImplementationUnderstanding, ProcessingStepSummary, StageSummary, UnderstandingStats,
    };
    use std::collections::HashMap;

    fn make_iu(
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
            module_summaries: vec![],
            signal_summaries: vec![],
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

    // ─── tm_01: processing_steps → pipeline_stage nodes ──────────────

    #[test]
    fn tm_01_steps_become_pipeline_stages() {
        let iu = make_iu(
            vec![
                make_step("fetch", "取指", 1),
                make_step("decode", "译码", 2),
                make_step("execute", "执行", 3),
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

    // ─── tm_03: pipeline stage 顺序边 ────────────────────────────────

    #[test]
    fn tm_03_sequential_edges_correct() {
        let iu = make_iu(
            vec![
                make_step("s1", "阶段 1", 1),
                make_step("s2", "阶段 2", 2),
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
        assert!(graph
            .meta
            .empty_reason
            .as_deref()
            .unwrap()
            .contains("时序信息不足"));
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

    // ─── tm_06: node_id 唯一，edge endpoint 存在 ─────────────────────

    #[test]
    fn tm_06_node_ids_unique_edge_endpoints_exist() {
        let iu = make_iu(
            vec![
                make_step("a", "A", 1),
                make_step("b", "B", 2),
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
}
