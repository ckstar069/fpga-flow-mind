//! Phase 7 Batch B: 形式化视图质量评估器（View Quality Evaluator）。
//!
//! 对 `ViewGraph` 执行确定性质量评估：
//! - 节点/边 trace_refs 可解析性
//! - 孤立节点检测
//! - 错连嫌疑计数（预留）
//!
//! 仅输出 `ViewQualityReport` + `QualityIssue` 列表，issue_id 留空由 reporter 统一填充。

use std::collections::HashSet;

use crate::quality::issue_builder::{make_issue, trace_ref_ok};
use crate::quality::models::{
    ArtifactKind, QualityIssue, QualityIssueKind, QualitySeverity, ViewQualityReport,
};
use crate::views::models::{ViewGraph, ViewTraceRef};

/// 评估器输入 — 引用外部数据，零拷贝。
#[derive(Debug, Clone)]
pub struct ViewEvaluatorInput<'a> {
    pub sample_id: &'a str,
    pub stage_id: &'a str,
    pub view: &'a ViewGraph,
    pub evidence_id_set: &'a HashSet<String>,
    pub claim_id_set: &'a HashSet<String>,
}

/// 形式化视图质量评估器（无状态）。
pub struct ViewEvaluator;

impl ViewEvaluator {
    /// 对单个阶段的 `ViewGraph` 执行质量评估。
    ///
    /// 返回 `(ViewQualityReport, Vec<QualityIssue>)`：
    /// - `issue_id` 全部留空，由 reporter 统一分配。
    pub fn evaluate(input: &ViewEvaluatorInput<'_>) -> (ViewQualityReport, Vec<QualityIssue>) {
        let mut issues: Vec<QualityIssue> = Vec::new();
        let view_type_str = format!("{:?}", input.view.view_type).to_lowercase();

        // 1. 空视图
        if input.view.nodes.is_empty() {
            issues.push(make_issue(
                input.sample_id,
                input.stage_id,
                ArtifactKind::View,
                QualityIssueKind::EmptyOrUnhelpfulView,
                QualitySeverity::Medium,
                None,
                None,
                None,
                None,
                None,
                &format!("视图 {} 为空（无节点）", view_type_str),
            ));

            let report = ViewQualityReport {
                sample_id: input.sample_id.to_string(),
                stage_id: input.stage_id.to_string(),
                view_type: view_type_str,
                trace_resolvable_ratio: 0.0,
                isolated_node_count: 0,
                suspected_misconnection_count: 0,
                issue_refs: Vec::new(), // reporter 填充
            };
            return (report, issues);
        }

        // 2. 逐节点/边评估 trace_refs
        let mut resolvable_artifacts: u32 = 0;
        let mut total_artifacts: u32 = 0;

        for node in &input.view.nodes {
            total_artifacts += 1;
            if node.trace_refs.is_empty() {
                issues.push(make_issue(
                    input.sample_id,
                    input.stage_id,
                    ArtifactKind::View,
                    QualityIssueKind::EmptyOrUnhelpfulView,
                    QualitySeverity::Medium,
                    None,
                    None,
                    Some(&node.node_id),
                    None,
                    None,
                    &format!(
                        "视图 {} 节点缺少 trace_refs（node_id={}）",
                        view_type_str, node.node_id
                    ),
                ));
            } else if Self::artifact_has_resolvable_trace(&node.trace_refs, input.evidence_id_set, input.claim_id_set) {
                resolvable_artifacts += 1;
            } else {
                issues.push(make_issue(
                    input.sample_id,
                    input.stage_id,
                    ArtifactKind::View,
                    QualityIssueKind::EmptyOrUnhelpfulView,
                    QualitySeverity::Medium,
                    None,
                    None,
                    Some(&node.node_id),
                    None,
                    None,
                    &format!(
                        "视图 {} 节点 trace_refs 全部不可解析（node_id={}）",
                        view_type_str, node.node_id
                    ),
                ));
            }
        }

        for edge in &input.view.edges {
            total_artifacts += 1;
            if edge.trace_refs.is_empty() {
                issues.push(make_issue(
                    input.sample_id,
                    input.stage_id,
                    ArtifactKind::View,
                    QualityIssueKind::EmptyOrUnhelpfulView,
                    QualitySeverity::Medium,
                    None,
                    None,
                    None,
                    None,
                    None,
                    &format!(
                        "视图 {} 边缺少 trace_refs（edge={}）",
                        view_type_str, edge.edge_id
                    ),
                ));
            } else if Self::artifact_has_resolvable_trace(&edge.trace_refs, input.evidence_id_set, input.claim_id_set) {
                resolvable_artifacts += 1;
            } else {
                issues.push(make_issue(
                    input.sample_id,
                    input.stage_id,
                    ArtifactKind::View,
                    QualityIssueKind::EmptyOrUnhelpfulView,
                    QualitySeverity::Medium,
                    None,
                    None,
                    None,
                    None,
                    None,
                    &format!(
                        "视图 {} 边 trace_refs 全部不可解析（edge={}）",
                        view_type_str, edge.edge_id
                    ),
                ));
            }
        }

        let trace_resolvable_ratio = if total_artifacts > 0 {
            resolvable_artifacts as f32 / total_artifacts as f32
        } else {
            0.0
        };

        // 4. 孤立节点检测
        let mut isolated_node_count: u32 = 0;
        let mut first_isolated_emitted = false;
        for node in &input.view.nodes {
            let is_isolated = !input.view.edges.iter().any(|edge| {
                edge.source_node_id == node.node_id || edge.target_node_id == node.node_id
            });
            if is_isolated {
                isolated_node_count += 1;
                if !first_isolated_emitted {
                    first_isolated_emitted = true;
                    issues.push(make_issue(
                        input.sample_id,
                        input.stage_id,
                        ArtifactKind::View,
                        QualityIssueKind::EmptyOrUnhelpfulView,
                        QualitySeverity::Low,
                        None,
                        None,
                        Some(&node.node_id),
                        None,
                        None,
                        &format!(
                            "视图 {} 孤立节点（node_id={}）",
                            view_type_str, node.node_id
                        ),
                    ));
                }
            }
        }

        // 5. suspected_misconnection_count: 预留 Batch B+ 启发式
        // TODO: 在 Batch B+ 中引入语义错连检测（edge 端点语义不匹配，如 Module 与 Signal 的异常连接）。
        let suspected_misconnection_count: u32 = 0;

        let report = ViewQualityReport {
            sample_id: input.sample_id.to_string(),
            stage_id: input.stage_id.to_string(),
            view_type: view_type_str,
            trace_resolvable_ratio,
            isolated_node_count,
            suspected_misconnection_count,
            issue_refs: Vec::new(), // reporter 填充
        };

        (report, issues)
    }

    /// 检查一个 artifact（节点或边）的 trace_refs 中是否至少有一个可解析。
    fn artifact_has_resolvable_trace(
        trace_refs: &[ViewTraceRef],
        evidence_id_set: &HashSet<String>,
        claim_id_set: &HashSet<String>,
    ) -> bool {
        trace_refs.iter().any(|tr| trace_ref_ok(&tr.evidence_id, &tr.claim_id, evidence_id_set, claim_id_set))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quality::models::QualityIssueKind;
    use crate::understanding::models::ClaimConfidence;
    use crate::views::models::{ViewEdge, ViewGraph, ViewMeta, ViewNode, ViewTraceRef, ViewType};

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
                is_degraded_source: false,
                generated_at: "2026-06-15T00:00:00Z".to_string(),
                empty_reason: None,
            },
        }
    }

    fn make_node(node_id: &str, trace_refs: Vec<ViewTraceRef>) -> ViewNode {
        ViewNode {
            node_id: node_id.to_string(),
            node_type: crate::views::models::NodeType::Module,
            label: node_id.to_string(),
            description: "test node".to_string(),
            confidence: ClaimConfidence::Confirmed,
            trace_refs,
            layout: None,
        }
    }

    fn make_edge(edge_id: &str, source: &str, target: &str, trace_refs: Vec<ViewTraceRef>) -> ViewEdge {
        ViewEdge {
            edge_id: edge_id.to_string(),
            edge_type: crate::views::models::EdgeType::Contains,
            source_node_id: source.to_string(),
            target_node_id: target.to_string(),
            label: None,
            description: "test edge".to_string(),
            confidence: ClaimConfidence::Confirmed,
            trace_refs,
        }
    }

    fn make_trace_ref(ev_id: Option<&str>, cl_id: Option<&str>) -> ViewTraceRef {
        ViewTraceRef {
            claim_id: cl_id.map(|s| s.to_string()),
            evidence_id: ev_id.map(|s| s.to_string()),
            confidence: ClaimConfidence::Confirmed,
            relevance: None,
        }
    }

    fn empty_sets() -> (HashSet<String>, HashSet<String>) {
        (HashSet::new(), HashSet::new())
    }

    fn real_sets() -> (HashSet<String>, HashSet<String>) {
        let mut ev = HashSet::new();
        ev.insert("EV-1".to_string());
        let mut cl = HashSet::new();
        cl.insert("CL-1".to_string());
        (ev, cl)
    }

    #[test]
    fn empty_view_ratio_zero() {
        let graph = make_view_graph(ViewType::Structure, vec![], vec![]);
        let (ev_set, cl_set) = empty_sets();
        let input = ViewEvaluatorInput {
            sample_id: "S1",
            stage_id: "L0",
            view: &graph,
            evidence_id_set: &ev_set,
            claim_id_set: &cl_set,
        };
        let (report, issues) = ViewEvaluator::evaluate(&input);

        assert_eq!(report.trace_resolvable_ratio, 0.0);
        assert_eq!(report.isolated_node_count, 0);
        assert_eq!(report.suspected_misconnection_count, 0);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, QualityIssueKind::EmptyOrUnhelpfulView);
        assert_eq!(issues[0].severity, QualitySeverity::Medium);
    }

    #[test]
    fn node_without_trace_emits_issue() {
        let node = make_node("N-1", vec![]);
        // Add a self-loop edge so N-1 is not isolated (test focuses on trace_refs, not isolation)
        let edge = make_edge("E-1", "N-1", "N-1", vec![make_trace_ref(Some("EV-1"), None)]);
        let graph = make_view_graph(ViewType::Structure, vec![node], vec![edge]);
        let (ev_set, cl_set) = real_sets();
        let input = ViewEvaluatorInput {
            sample_id: "S1",
            stage_id: "L0",
            view: &graph,
            evidence_id_set: &ev_set,
            claim_id_set: &cl_set,
        };
        let (report, issues) = ViewEvaluator::evaluate(&input);

        // Only the node trace issue; edge trace is valid
        let node_issues: Vec<_> = issues.iter().filter(|i| i.node_id.as_deref() == Some("N-1")).collect();
        assert_eq!(node_issues.len(), 1);
        assert_eq!(node_issues[0].kind, QualityIssueKind::EmptyOrUnhelpfulView);
        assert_eq!(node_issues[0].node_id.as_deref(), Some("N-1"));
        // 1 node (no trace) + 1 edge (valid trace) = 1/2 resolvable
        assert_eq!(report.trace_resolvable_ratio, 0.5);
    }

    #[test]
    fn edge_without_trace_emits_issue() {
        let node = make_node("N-1", vec![make_trace_ref(Some("EV-1"), None)]);
        let edge = make_edge("E-1", "N-1", "N-2", vec![]);
        let graph = make_view_graph(ViewType::Structure, vec![node], vec![edge]);
        let (ev_set, cl_set) = real_sets();
        let input = ViewEvaluatorInput {
            sample_id: "S1",
            stage_id: "L0",
            view: &graph,
            evidence_id_set: &ev_set,
            claim_id_set: &cl_set,
        };
        let (report, issues) = ViewEvaluator::evaluate(&input);

        // 节点 trace 有效，边 trace 缺失 → 1 resolvable / 2 total = 0.5
        assert_eq!(report.trace_resolvable_ratio, 0.5);
        let edge_issues: Vec<_> = issues.iter().filter(|i| i.description.contains("边缺少 trace_refs")).collect();
        assert_eq!(edge_issues.len(), 1);
        assert_eq!(edge_issues[0].kind, QualityIssueKind::EmptyOrUnhelpfulView);
    }

    #[test]
    fn invalid_trace_ref_lowers_ratio() {
        let node = make_node("N-1", vec![make_trace_ref(Some("EV-FAKE"), None)]);
        // Add a self-loop edge with valid trace so node is not isolated
        let edge = make_edge("E-1", "N-1", "N-1", vec![make_trace_ref(Some("EV-1"), None)]);
        let graph = make_view_graph(ViewType::Structure, vec![node], vec![edge]);
        let (ev_set, cl_set) = real_sets();
        let input = ViewEvaluatorInput {
            sample_id: "S1",
            stage_id: "L0",
            view: &graph,
            evidence_id_set: &ev_set,
            claim_id_set: &cl_set,
        };
        let (report, issues) = ViewEvaluator::evaluate(&input);

        // 1 node (fake trace, unresolvable) + 1 edge (valid trace) = 1/2 resolvable
        assert_eq!(report.trace_resolvable_ratio, 0.5);
        let node_issues: Vec<_> = issues.iter().filter(|i| i.node_id.as_deref() == Some("N-1")).collect();
        assert_eq!(node_issues.len(), 1);
        assert_eq!(node_issues[0].kind, QualityIssueKind::EmptyOrUnhelpfulView);
        assert!(node_issues[0].description.contains("全部不可解析"));
    }

    #[test]
    fn valid_node_edge_trace_ratio_one() {
        let node = make_node("N-1", vec![make_trace_ref(Some("EV-1"), None)]);
        let edge = make_edge("E-1", "N-1", "N-2", vec![make_trace_ref(None, Some("CL-1"))]);
        let graph = make_view_graph(ViewType::Structure, vec![node], vec![edge]);
        let (ev_set, cl_set) = real_sets();
        let input = ViewEvaluatorInput {
            sample_id: "S1",
            stage_id: "L0",
            view: &graph,
            evidence_id_set: &ev_set,
            claim_id_set: &cl_set,
        };
        let (report, issues) = ViewEvaluator::evaluate(&input);

        assert_eq!(report.trace_resolvable_ratio, 1.0);
        assert!(issues.is_empty());
    }

    #[test]
    fn isolated_node_count_detected() {
        let n1 = make_node("N-1", vec![make_trace_ref(Some("EV-1"), None)]);
        let n2 = make_node("N-2", vec![make_trace_ref(Some("EV-1"), None)]);
        // N-1 connected, N-2 isolated
        let edge = make_edge("E-1", "N-1", "N-3", vec![make_trace_ref(None, Some("CL-1"))]);
        let graph = make_view_graph(ViewType::Structure, vec![n1, n2], vec![edge]);
        let (ev_set, cl_set) = real_sets();
        let input = ViewEvaluatorInput {
            sample_id: "S1",
            stage_id: "L0",
            view: &graph,
            evidence_id_set: &ev_set,
            claim_id_set: &cl_set,
        };
        let (report, issues) = ViewEvaluator::evaluate(&input);

        assert_eq!(report.isolated_node_count, 1);
        assert_eq!(report.trace_resolvable_ratio, 1.0); // all traces valid
        let isolated_issues: Vec<_> = issues.iter().filter(|i| i.description.contains("孤立节点")).collect();
        assert_eq!(isolated_issues.len(), 1);
        assert_eq!(isolated_issues[0].node_id.as_deref(), Some("N-2"));
        assert_eq!(isolated_issues[0].severity, QualitySeverity::Low);
    }

    // ─── P0-3: 非空且 trace 可解析的 dataflow/timing 不应被判 empty_or_unhelpful ──

    #[test]
    fn non_empty_traceable_dataflow_not_flagged_empty() {
        use crate::views::models::{EdgeType, NodeType};
        // 模拟 P0-3 后的 dataflow 视图：2 个 processing_step 节点 + 1 条带 trace 的顺序边
        let n1 = ViewNode {
            node_id: "N-dataflow-0001".to_string(),
            node_type: NodeType::ProcessingStep,
            label: "load".to_string(),
            description: "载入".to_string(),
            confidence: ClaimConfidence::Supported,
            trace_refs: vec![make_trace_ref(Some("EV-1"), None)],
            layout: None,
        };
        let n2 = ViewNode {
            node_id: "N-dataflow-0002".to_string(),
            node_type: NodeType::ProcessingStep,
            label: "corr".to_string(),
            description: "相关".to_string(),
            confidence: ClaimConfidence::Supported,
            trace_refs: vec![make_trace_ref(Some("EV-2"), None)],
            layout: None,
        };
        // 边 trace_refs 指向端点节点的 evidence（P0-3 行为）
        let edge = ViewEdge {
            edge_id: "E-dataflow-0001".to_string(),
            edge_type: EdgeType::DataFlow,
            source_node_id: "N-dataflow-0001".to_string(),
            target_node_id: "N-dataflow-0002".to_string(),
            label: None,
            description: "处理步骤 1 → 2".to_string(),
            confidence: ClaimConfidence::Inferred,
            trace_refs: vec![make_trace_ref(Some("EV-1"), None)],
        };
        // evidence 集含 EV-1/EV-2
        let mut ev = HashSet::new();
        ev.insert("EV-1".to_string());
        ev.insert("EV-2".to_string());
        let graph = make_view_graph(ViewType::Dataflow, vec![n1, n2], vec![edge]);
        let input = ViewEvaluatorInput {
            sample_id: "S1",
            stage_id: "L0",
            view: &graph,
            evidence_id_set: &ev,
            claim_id_set: &HashSet::new(),
        };
        let (report, issues) = ViewEvaluator::evaluate(&input);

        // 非空 → 不应有 EmptyOrUnhelpfulView（节点+边均可解析）
        let empty_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.kind == QualityIssueKind::EmptyOrUnhelpfulView)
            .collect();
        assert!(
            empty_issues.is_empty(),
            "非空且 trace 可解析的 dataflow 不应被判 empty_or_unhelpful, issues: {:?}",
            issues
        );
        assert_eq!(report.trace_resolvable_ratio, 1.0);
    }

    #[test]
    fn non_empty_traceable_timing_not_flagged_empty() {
        use crate::views::models::{EdgeType, NodeType};
        // RTL 时序回退：1 个 ClockDomain 节点（trace 到 EV-RTL-1），无 step → 无边
        let n = ViewNode {
            node_id: "N-timing-0001".to_string(),
            node_type: NodeType::ClockDomain,
            label: "clk".to_string(),
            description: "时钟".to_string(),
            confidence: ClaimConfidence::Inferred,
            trace_refs: vec![make_trace_ref(Some("EV-RTL-1"), None)],
            layout: None,
        };
        let mut ev = HashSet::new();
        ev.insert("EV-RTL-1".to_string());
        let graph = make_view_graph(ViewType::Timing, vec![n], vec![]);
        let input = ViewEvaluatorInput {
            sample_id: "S1",
            stage_id: "RTL",
            view: &graph,
            evidence_id_set: &ev,
            claim_id_set: &HashSet::new(),
        };
        let (report, issues) = ViewEvaluator::evaluate(&input);

        // 非空、trace 可解析 → 不应被判 Medium 严重度的"视图为空"。
        // 孤立节点提示（Low）是诚实信号，可接受，但不应是 Medium empty。
        let medium_empty: Vec<_> = issues
            .iter()
            .filter(|i| {
                i.kind == QualityIssueKind::EmptyOrUnhelpfulView
                    && i.severity == QualitySeverity::Medium
            })
            .collect();
        assert!(
            medium_empty.is_empty(),
            "非空且 trace 可解析的 timing 不应有 Medium empty_or_unhelpful, issues: {:?}",
            issues
        );
        assert_eq!(report.trace_resolvable_ratio, 1.0);
        // 孤立节点提示（如有）严重度应为 Low
        for i in &issues {
            if i.description.contains("孤立节点") {
                assert_eq!(i.severity, QualitySeverity::Low);
            }
        }
    }
}
