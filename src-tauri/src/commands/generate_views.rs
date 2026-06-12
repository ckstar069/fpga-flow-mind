/// `generate_views` Tauri command
///
/// 纯 IU → ViewGraph 转换。不访问目标项目文件系统，不调用 generate_understanding。
/// 前端必须先持有 ImplementationUnderstanding 才能调用。
///
/// 返回策略：
/// - 正常 IU → success=true, data=[structure, dataflow, timing]
/// - 空 IU → success=true, data=3 个空 ViewGraph（含 empty_reason）
/// - degraded IU → success=true, data=3 个 ViewGraph（is_degraded_source=true）

use crate::models::error::CommandResult;
use crate::understanding::models::ImplementationUnderstanding;
use crate::views::generator::ViewGraphGenerator;
use crate::views::models::ViewGraph;

#[tauri::command]
pub fn generate_views(
    understanding: ImplementationUnderstanding,
) -> CommandResult<Vec<ViewGraph>> {
    let graphs = ViewGraphGenerator::generate_all(&understanding);

    CommandResult {
        success: true,
        data: Some(graphs),
        error: None,
        warnings: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::understanding::models::{
        ClaimConfidence, GenerationMeta, ModuleSummary, SignalSummary, StageSummary,
        UnderstandingStats,
    };
    use crate::views::models::ViewType;
    use std::collections::HashMap;

    fn make_iu(is_degraded: bool) -> ImplementationUnderstanding {
        ImplementationUnderstanding {
            stage_id: "L0".to_string(),
            version: "3.0.0".to_string(),
            summary: StageSummary {
                short: "test".to_string(),
                detailed: "test".to_string(),
            },
            claims: vec![],
            module_summaries: vec![ModuleSummary {
                name: "mod_a".to_string(),
                description: "模块 A".to_string(),
                evidence_refs: vec![],
                confidence: ClaimConfidence::Confirmed,
            }],
            signal_summaries: vec![SignalSummary {
                name: "clk".to_string(),
                description: "时钟".to_string(),
                direction: None,
                evidence_refs: vec![],
                confidence: ClaimConfidence::Supported,
            }],
            interface_summaries: vec![],
            processing_steps: vec![],
            unknowns: vec![],
            evidence_gaps: vec![],
            generation_meta: GenerationMeta {
                provider: "mock".to_string(),
                generated_at: "2026-06-12T10:00:00Z".to_string(),
                input_evidence_count: 2,
                generation_time_ms: 10,
                is_degraded,
            },
            stats: UnderstandingStats {
                total_claims: 0,
                claims_by_confidence: HashMap::new(),
                claims_by_category: HashMap::new(),
                module_count: 1,
                signal_count: 1,
                interface_count: 0,
                processing_step_count: 0,
                unknown_count: 0,
                evidence_gap_count: 0,
            },
        }
    }

    // ─── cmd_01: 正常 understanding → 3 个 graph ─────────────────────

    #[test]
    fn cmd_01_normal_understanding_returns_three_graphs() {
        let iu = make_iu(false);
        let result = generate_views(iu);
        assert!(result.success);
        let graphs = result.data.unwrap();
        assert_eq!(graphs.len(), 3);
        assert_eq!(graphs[0].view_type, ViewType::Structure);
        assert_eq!(graphs[1].view_type, ViewType::Dataflow);
        assert_eq!(graphs[2].view_type, ViewType::Timing);
        assert!(result.warnings.is_empty());
    }

    // ─── cmd_02: 空 understanding → 3 个 graph，不 panic ──────────────

    #[test]
    fn cmd_02_empty_understanding_returns_three_graphs() {
        let iu = ImplementationUnderstanding {
            stage_id: "L0".to_string(),
            version: "3.0.0".to_string(),
            summary: StageSummary {
                short: "test".to_string(),
                detailed: "test".to_string(),
            },
            claims: vec![],
            module_summaries: vec![],
            signal_summaries: vec![],
            interface_summaries: vec![],
            processing_steps: vec![],
            unknowns: vec![],
            evidence_gaps: vec![],
            generation_meta: GenerationMeta {
                provider: "mock".to_string(),
                generated_at: "2026-06-12T10:00:00Z".to_string(),
                input_evidence_count: 0,
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
        };
        let result = generate_views(iu);
        assert!(result.success);
        let graphs = result.data.unwrap();
        assert_eq!(graphs.len(), 3);
    }

    // ─── cmd_03: degraded understanding → 3 个 graph ──────────────────

    #[test]
    fn cmd_03_degraded_understanding_returns_three_graphs() {
        let iu = make_iu(true);
        let result = generate_views(iu);
        assert!(result.success);
        let graphs = result.data.unwrap();
        assert_eq!(graphs.len(), 3);
        for g in &graphs {
            assert!(g.meta.is_degraded_source);
        }
    }

    // ─── cmd_04: result.error 为 None ─────────────────────────────────

    #[test]
    fn cmd_04_result_error_is_none() {
        let iu = make_iu(false);
        let result = generate_views(iu);
        assert!(result.success);
        assert!(result.error.is_none());
    }

    // ─── cmd_05: graph.warnings 为空 ──────────────────────────────────

    #[test]
    fn cmd_05_warnings_are_empty() {
        let iu = make_iu(false);
        let result = generate_views(iu);
        assert!(result.warnings.is_empty());
    }
}
