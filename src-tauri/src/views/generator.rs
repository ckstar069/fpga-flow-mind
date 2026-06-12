use crate::understanding::models::ImplementationUnderstanding;
use crate::views::dataflow_builder::build_dataflow_view;
use crate::views::models::ViewGraph;
use crate::views::structure_builder::build_structure_view;
use crate::views::timing_builder::build_timing_view;

/// ViewGraph 总调度器 — 从 IU 生成全部三类 ViewGraph
///
/// 纯转换函数：不访问文件系统、不调用 LLM、不调用 generate_understanding。
pub struct ViewGraphGenerator;

impl ViewGraphGenerator {
    /// 从 ImplementationUnderstanding 生成三类 ViewGraph
    ///
    /// 返回顺序：structure → dataflow → timing
    pub fn generate_all(iu: &ImplementationUnderstanding) -> Vec<ViewGraph> {
        vec![
            build_structure_view(iu),
            build_dataflow_view(iu),
            build_timing_view(iu),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::understanding::models::{
        ClaimConfidence, GenerationMeta, ImplementationClaim, ModuleSummary, ProcessingStepSummary,
        SignalSummary, StageSummary, UnderstandingStats,
    };
    use crate::views::models::{NodeType, ViewType};
    use std::collections::HashMap;

    fn make_module(name: &str, desc: &str) -> ModuleSummary {
        ModuleSummary {
            name: name.to_string(),
            description: desc.to_string(),
            evidence_refs: vec![],
            confidence: ClaimConfidence::Confirmed,
        }
    }

    fn make_signal(name: &str, desc: &str) -> SignalSummary {
        SignalSummary {
            name: name.to_string(),
            description: desc.to_string(),
            direction: None,
            evidence_refs: vec![],
            confidence: ClaimConfidence::Supported,
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

    fn make_iu(
        stage_id: &str,
        modules: Vec<ModuleSummary>,
        signals: Vec<SignalSummary>,
        steps: Vec<ProcessingStepSummary>,
        is_degraded: bool,
    ) -> ImplementationUnderstanding {
        ImplementationUnderstanding {
            stage_id: stage_id.to_string(),
            version: "3.0.0".to_string(),
            summary: StageSummary {
                short: "test".to_string(),
                detailed: "test".to_string(),
            },
            claims: vec![],
            module_summaries: modules,
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
                is_degraded,
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

    // ─── gen_01: 正常 IU → 3 个 graph ────────────────────────────────

    #[test]
    fn gen_01_normal_iu_returns_three_graphs() {
        let iu = make_iu(
            "L0",
            vec![make_module("mod_a", "模块 A")],
            vec![make_signal("clk", "时钟")],
            vec![make_step("proc", "处理", 1)],
            false,
        );
        let graphs = ViewGraphGenerator::generate_all(&iu);
        assert_eq!(graphs.len(), 3);
    }

    // ─── gen_02: 顺序 structure → dataflow → timing ──────────────────

    #[test]
    fn gen_02_order_is_structure_dataflow_timing() {
        let iu = make_iu("L0", vec![], vec![], vec![], false);
        let graphs = ViewGraphGenerator::generate_all(&iu);
        assert_eq!(graphs[0].view_type, ViewType::Structure);
        assert_eq!(graphs[1].view_type, ViewType::Dataflow);
        assert_eq!(graphs[2].view_type, ViewType::Timing);
    }

    // ─── gen_03: degraded IU → 三个 graph 均标记 is_degraded_source ──

    #[test]
    fn gen_03_degraded_iu_marks_all_graphs() {
        let iu = make_iu("L0", vec![], vec![], vec![], true);
        let graphs = ViewGraphGenerator::generate_all(&iu);
        for g in &graphs {
            assert!(g.meta.is_degraded_source, "degraded IU 的 graph 应标记");
        }
    }

    // ─── gen_04: 空 IU → 三个 graph 均返回，不 panic ─────────────────

    #[test]
    fn gen_04_empty_iu_no_panic() {
        let iu = make_iu("L0", vec![], vec![], vec![], false);
        let graphs = ViewGraphGenerator::generate_all(&iu);
        assert_eq!(graphs.len(), 3);
        // 空 IU 下每个 graph 应有 empty_reason
        for g in &graphs {
            assert!(g.meta.empty_reason.is_some(), "{:?} 应有 empty_reason", g.view_type);
        }
    }

    // ─── gen_05: graph.stage_id 与 iu.stage_id 一致 ───────────────────

    #[test]
    fn gen_05_stage_id_matches_iu() {
        let iu = make_iu(
            "RTL",
            vec![make_module("top", "顶层")],
            vec![],
            vec![],
            false,
        );
        let graphs = ViewGraphGenerator::generate_all(&iu);
        for g in &graphs {
            assert_eq!(g.stage_id, "RTL");
        }
    }

    // ─── gen_06: 带数据的 IU → structure graph 有 nodes ──────────────

    #[test]
    fn gen_06_structure_has_nodes_from_iu() {
        let iu = make_iu(
            "L0",
            vec![make_module("mod_a", "A"), make_module("mod_b", "B")],
            vec![make_signal("s1", "S1")],
            vec![],
            false,
        );
        let graphs = ViewGraphGenerator::generate_all(&iu);
        let structure = &graphs[0];
        assert_eq!(structure.view_type, ViewType::Structure);
        assert!(!structure.nodes.is_empty(), "structure 应有节点");
        assert!(structure.nodes.iter().any(|n| n.node_type == NodeType::Module));
    }
}
