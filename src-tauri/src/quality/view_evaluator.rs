//! Phase 7 Batch B/D: 视图质量评估器（View Quality Evaluator）。
//!
//! 对 `ViewGraph` 执行确定性质量评估：
//! - 节点/边 trace_refs 可解析性
//! - 孤立节点检测
//! - 语义多样性检查
//!
//! **P2 校准：** 引入更细退化分类，区分"诚实空图"（`ExpectedEmptyTiming`）、
//! "追溯缺口"（`TraceabilityGap`）、"孤立图"（`IsolatedOrUnconnectedView`）、
//! "语义多样性不足"（`LowSemanticDiversity`）等不同退化模式。
//!
//! 仅输出 `ViewQualityReport` + `QualityIssue` 列表，issue_id 留空由 reporter 统一填充。

use std::collections::HashSet;

use crate::quality::issue_builder::{make_issue, trace_ref_ok};
use crate::quality::models::{
    ArtifactKind, QualityIssue, QualityIssueKind, QualitySeverity, ViewQualityReport,
};
use crate::views::models::{ViewGraph, ViewTraceRef, ViewType};

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
            // P2: 区分 expected_empty_timing 与其他空图
            if is_expected_empty_timing(&input.view.view_type, &input.view.meta.empty_reason) {
                issues.push(make_issue(
                    input.sample_id,
                    input.stage_id,
                    ArtifactKind::View,
                    QualityIssueKind::ExpectedEmptyTiming,
                    QualitySeverity::Low,
                    None,
                    None,
                    None,
                    None,
                    None,
                    &format!(
                        "视图 {} 为空：无 cycle/latency/clock/pipeline 等可追溯时序证据（预期行为）",
                        view_type_str
                    ),
                ));
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
                    &format!("视图 {} 为空（无节点）", view_type_str),
                ));
            }

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
                // P2: 缺 trace_refs → TraceabilityGap，不再归类为 EmptyOrUnhelpfulView
                issues.push(make_issue(
                    input.sample_id,
                    input.stage_id,
                    ArtifactKind::View,
                    QualityIssueKind::TraceabilityGap,
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
                // P2: 不可解析 trace_refs → TraceabilityGap
                issues.push(make_issue(
                    input.sample_id,
                    input.stage_id,
                    ArtifactKind::View,
                    QualityIssueKind::TraceabilityGap,
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
                // P2: 缺 trace_refs → TraceabilityGap
                issues.push(make_issue(
                    input.sample_id,
                    input.stage_id,
                    ArtifactKind::View,
                    QualityIssueKind::TraceabilityGap,
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
                // P2: 不可解析 trace_refs → TraceabilityGap
                issues.push(make_issue(
                    input.sample_id,
                    input.stage_id,
                    ArtifactKind::View,
                    QualityIssueKind::TraceabilityGap,
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

        // 3. 语义多样性检查（P2: 检测标签高度重复）
        let label_duplicate_count = Self::count_duplicate_labels(&input.view.nodes);
        if label_duplicate_count > 0 {
            let node_count = input.view.nodes.len();
            let unique_count = node_count - label_duplicate_count;
            // 当重复节点占比 >= 50% 时视为语义多样性不足
            if node_count > 1 && (unique_count as f32) / (node_count as f32) < 0.5 {
                issues.push(make_issue(
                    input.sample_id,
                    input.stage_id,
                    ArtifactKind::View,
                    QualityIssueKind::LowSemanticDiversity,
                    QualitySeverity::Low,
                    None,
                    None,
                    None,
                    None,
                    None,
                    &format!(
                        "视图 {} 节点标签高度重复（共 {} 节点，唯一标签 {} 个），信息价值低",
                        view_type_str, node_count, unique_count
                    ),
                ));
            }
        }

        // 4. 孤立节点检测
        let mut isolated_node_count: u32 = 0;
        let mut first_isolated_emitted = false;
        let node_count = input.view.nodes.len() as u32;
        for node in &input.view.nodes {
            let is_isolated = !input.view.edges.iter().any(|edge| {
                edge.source_node_id == node.node_id || edge.target_node_id == node.node_id
            });
            if is_isolated {
                isolated_node_count += 1;
                if !first_isolated_emitted {
                    first_isolated_emitted = true;
                    // P2: 孤立节点 → IsolatedOrUnconnectedView
                    // 若孤立率 > 50%，直接标记 Medium 汇总（不再添加 Low 个体+Medium 汇总两个 issue）
                    let isolated_ratio = isolated_node_count as f32 / node_count as f32;
                    let severity = if isolated_ratio > 0.5 {
                        QualitySeverity::Medium
                    } else {
                        QualitySeverity::Low
                    };
                    issues.push(make_issue(
                        input.sample_id,
                        input.stage_id,
                        ArtifactKind::View,
                        QualityIssueKind::IsolatedOrUnconnectedView,
                        severity,
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

    /// 计算重复标签的数量（出现 > 1 次的标签，其额外出现次数）。
    fn count_duplicate_labels(nodes: &[crate::views::models::ViewNode]) -> usize {
        let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
        let mut duplicate_count = 0usize;
        for node in nodes {
            if !seen.insert(node.label.as_str()) {
                duplicate_count += 1;
            }
        }
        duplicate_count
    }
}

/// 判断一个空图是否属于"预期空时序"（P2 校准）。
///
/// 条件：
/// - 视图类型为 Timing
/// - empty_reason 中含有预期关键词（无时序证据、processing_steps 为函数顺序等）
fn is_expected_empty_timing(view_type: &ViewType, empty_reason: &Option<String>) -> bool {
    if *view_type != ViewType::Timing {
        return false;
    }
    if let Some(reason) = empty_reason {
        let lower = reason.to_lowercase();
        // 以下模式表明 timing 为空属于预期行为：
        // 1. "无 cycle/latency/clock/pipeline 等可追溯时序证据"
        // 2. "时序为空：当前阶段无 processing_steps"
        // 3. "时序信息不足：..."
        if lower.contains("cycle") || lower.contains("latency") || lower.contains("clock")
            || lower.contains("pipeline") || lower.contains("processing_steps")
            || lower.contains("时序为") || lower.contains("时序信息不足")
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quality::models::QualityIssueKind;
    use crate::understanding::models::ClaimConfidence;
    use crate::views::models::{ViewEdge, ViewGraph, ViewMeta, ViewNode, ViewTraceRef, ViewType};

    fn make_view_graph(view_type: ViewType, nodes: Vec<ViewNode>, edges: Vec<ViewEdge>, empty_reason: Option<&str>) -> ViewGraph {
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
                empty_reason: empty_reason.map(|s| s.to_string()),
            },
        }
    }

    fn make_node(node_id: &str, label: &str, trace_refs: Vec<ViewTraceRef>) -> ViewNode {
        ViewNode {
            node_id: node_id.to_string(),
            node_type: crate::views::models::NodeType::Module,
            label: label.to_string(),
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

    // ─── 原有测试（适配 P2 校准） ──────────────────────────────────────

    #[test]
    fn empty_view_ratio_zero() {
        let graph = make_view_graph(ViewType::Structure, vec![], vec![], None);
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
        // 结构图空 → EmptyOrUnhelpfulView（非 Timing）
        assert_eq!(issues[0].kind, QualityIssueKind::EmptyOrUnhelpfulView);
        assert_eq!(issues[0].severity, QualitySeverity::Medium);
    }

    #[test]
    fn node_without_trace_emits_traceability_gap() {
        let node = make_node("N-1", "mod", vec![]);
        let edge = make_edge("E-1", "N-1", "N-1", vec![make_trace_ref(Some("EV-1"), None)]);
        let graph = make_view_graph(ViewType::Structure, vec![node], vec![edge], None);
        let (ev_set, cl_set) = real_sets();
        let input = ViewEvaluatorInput {
            sample_id: "S1",
            stage_id: "L0",
            view: &graph,
            evidence_id_set: &ev_set,
            claim_id_set: &cl_set,
        };
        let (report, issues) = ViewEvaluator::evaluate(&input);

        // 1 node (no trace → TraceabilityGap) + 1 edge (valid trace)
        let node_issues: Vec<_> = issues.iter().filter(|i| i.node_id.as_deref() == Some("N-1")).collect();
        assert_eq!(node_issues.len(), 1);
        // P2: 缺 trace_refs → TraceabilityGap，不再是 EmptyOrUnhelpfulView
        assert_eq!(node_issues[0].kind, QualityIssueKind::TraceabilityGap);
        assert_eq!(node_issues[0].node_id.as_deref(), Some("N-1"));
        assert_eq!(report.trace_resolvable_ratio, 0.5);
    }

    #[test]
    fn edge_without_trace_emits_traceability_gap() {
        let node = make_node("N-1", "mod", vec![make_trace_ref(Some("EV-1"), None)]);
        let edge = make_edge("E-1", "N-1", "N-2", vec![]);
        let graph = make_view_graph(ViewType::Structure, vec![node], vec![edge], None);
        let (ev_set, cl_set) = real_sets();
        let input = ViewEvaluatorInput {
            sample_id: "S1",
            stage_id: "L0",
            view: &graph,
            evidence_id_set: &ev_set,
            claim_id_set: &cl_set,
        };
        let (report, issues) = ViewEvaluator::evaluate(&input);

        assert_eq!(report.trace_resolvable_ratio, 0.5);
        // P2: 边缺 trace_refs → TraceabilityGap
        let edge_issues: Vec<_> = issues.iter().filter(|i| i.description.contains("边缺少 trace_refs")).collect();
        assert_eq!(edge_issues.len(), 1);
        assert_eq!(edge_issues[0].kind, QualityIssueKind::TraceabilityGap);
    }

    #[test]
    fn invalid_trace_ref_lowers_ratio() {
        let node = make_node("N-1", "mod", vec![make_trace_ref(Some("EV-FAKE"), None)]);
        let edge = make_edge("E-1", "N-1", "N-1", vec![make_trace_ref(Some("EV-1"), None)]);
        let graph = make_view_graph(ViewType::Structure, vec![node], vec![edge], None);
        let (ev_set, cl_set) = real_sets();
        let input = ViewEvaluatorInput {
            sample_id: "S1",
            stage_id: "L0",
            view: &graph,
            evidence_id_set: &ev_set,
            claim_id_set: &cl_set,
        };
        let (report, issues) = ViewEvaluator::evaluate(&input);

        assert_eq!(report.trace_resolvable_ratio, 0.5);
        let node_issues: Vec<_> = issues.iter().filter(|i| i.node_id.as_deref() == Some("N-1")).collect();
        assert_eq!(node_issues.len(), 1);
        // P2: 不可解析 trace_refs → TraceabilityGap
        assert_eq!(node_issues[0].kind, QualityIssueKind::TraceabilityGap);
        assert!(node_issues[0].description.contains("全部不可解析"));
    }

    #[test]
    fn valid_node_edge_trace_ratio_one() {
        let node = make_node("N-1", "mod", vec![make_trace_ref(Some("EV-1"), None)]);
        let edge = make_edge("E-1", "N-1", "N-2", vec![make_trace_ref(None, Some("CL-1"))]);
        let graph = make_view_graph(ViewType::Structure, vec![node], vec![edge], None);
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
        let n1 = make_node("N-1", "mod1", vec![make_trace_ref(Some("EV-1"), None)]);
        let n2 = make_node("N-2", "mod2", vec![make_trace_ref(Some("EV-1"), None)]);
        let edge = make_edge("E-1", "N-1", "N-3", vec![make_trace_ref(None, Some("CL-1"))]);
        let graph = make_view_graph(ViewType::Structure, vec![n1, n2], vec![edge], None);
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
        assert_eq!(report.trace_resolvable_ratio, 1.0);
        // P2: 孤立节点 → IsolatedOrUnconnectedView
        let isolated_issues: Vec<_> = issues.iter().filter(|i| i.description.contains("孤立节点")).collect();
        assert_eq!(isolated_issues.len(), 1);
        assert_eq!(isolated_issues[0].node_id.as_deref(), Some("N-2"));
        assert_eq!(isolated_issues[0].kind, QualityIssueKind::IsolatedOrUnconnectedView);
        assert_eq!(isolated_issues[0].severity, QualitySeverity::Low);
    }

    // ─── P0-3: 非空且 trace 可解析的 dataflow/timing 不应被判 empty_or_unhelpful ──

    #[test]
    fn non_empty_traceable_dataflow_not_flagged_empty() {
        use crate::views::models::{EdgeType, NodeType};
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
        let mut ev = HashSet::new();
        ev.insert("EV-1".to_string());
        ev.insert("EV-2".to_string());
        let graph = make_view_graph(ViewType::Dataflow, vec![n1, n2], vec![edge], None);
        let input = ViewEvaluatorInput {
            sample_id: "S1",
            stage_id: "L0",
            view: &graph,
            evidence_id_set: &ev,
            claim_id_set: &HashSet::new(),
        };
        let (report, issues) = ViewEvaluator::evaluate(&input);

        // 非空 → 不应有任何 EmptyOrUnhelpfulView
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
        use crate::views::models::NodeType;
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
        let graph = make_view_graph(ViewType::Timing, vec![n], vec![], None);
        let input = ViewEvaluatorInput {
            sample_id: "S1",
            stage_id: "RTL",
            view: &graph,
            evidence_id_set: &ev,
            claim_id_set: &HashSet::new(),
        };
        let (report, issues) = ViewEvaluator::evaluate(&input);

        // 非空、trace 可解析 → Medium EmptyOrUnhelpfulView 不应存在
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
        // 孤立节点提示应为 IsolatedOrUnconnectedView
        // 1/1 孤立 = 100% > 50% 阈值 → Medium（非 EmptyOrUnhelpfulView）
        for i in &issues {
            if i.description.contains("孤立节点") {
                assert_eq!(i.kind, QualityIssueKind::IsolatedOrUnconnectedView);
            }
        }
    }

    // ─── P2: 新增校准测试 ──────────────────────────────────────────────

    /// P2: Python L0/L1 timing 空图 → ExpectedEmptyTiming（Low），非 EmptyOrUnhelpfulView（Medium）
    #[test]
    fn expected_empty_timing_not_flagged_medium() {
        // Timing 视图，空，含预期 empty_reason（模拟 Python 阶段）
        let graph = make_view_graph(
            ViewType::Timing,
            vec![],
            vec![],
            Some("无 cycle/latency/clock/pipeline 等可追溯时序证据，未生成 timing 图"),
        );
        let (ev_set, cl_set) = empty_sets();
        let input = ViewEvaluatorInput {
            sample_id: "S1",
            stage_id: "L0",
            view: &graph,
            evidence_id_set: &ev_set,
            claim_id_set: &cl_set,
        };
        let (report, issues) = ViewEvaluator::evaluate(&input);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, QualityIssueKind::ExpectedEmptyTiming);
        assert_eq!(issues[0].severity, QualitySeverity::Low);
        assert_eq!(report.trace_resolvable_ratio, 0.0);
    }

    /// P2: 非 Timing 空图仍为 EmptyOrUnhelpfulView（Medium）
    #[test]
    fn non_timing_empty_still_flagged_medium() {
        // Dataflow 视图空，但 empty_reason 无时序关键词
        let graph = make_view_graph(
            ViewType::Dataflow,
            vec![],
            vec![],
            Some("数据流为空：缺少 processing_steps"),
        );
        let (ev_set, cl_set) = empty_sets();
        let input = ViewEvaluatorInput {
            sample_id: "S1",
            stage_id: "L0",
            view: &graph,
            evidence_id_set: &ev_set,
            claim_id_set: &cl_set,
        };
        let (report, issues) = ViewEvaluator::evaluate(&input);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, QualityIssueKind::EmptyOrUnhelpfulView);
        assert_eq!(issues[0].severity, QualitySeverity::Medium);
        assert_eq!(report.trace_resolvable_ratio, 0.0);
    }

    /// P2: 非空但缺 trace_refs 的视图 → TraceabilityGap
    #[test]
    fn non_empty_missing_trace_refs_emits_traceability_gap() {
        let node = make_node("N-1", "mod", vec![]);
        let graph = make_view_graph(ViewType::Structure, vec![node], vec![], None);
        let (ev_set, cl_set) = empty_sets();
        let input = ViewEvaluatorInput {
            sample_id: "S1",
            stage_id: "L0",
            view: &graph,
            evidence_id_set: &ev_set,
            claim_id_set: &cl_set,
        };
        let (report, issues) = ViewEvaluator::evaluate(&input);

        // 节点缺 trace_refs → TraceabilityGap；无边；节点孤立 → IsolatedOrUnconnectedView
        let trace_issues: Vec<_> = issues.iter().filter(|i| i.kind == QualityIssueKind::TraceabilityGap).collect();
        assert!(!trace_issues.is_empty(), "缺 trace_refs 应产生 TraceabilityGap");
        assert!(trace_issues.iter().any(|i| i.node_id.as_deref() == Some("N-1")));
        assert_eq!(report.trace_resolvable_ratio, 0.0);
    }

    /// P2: 非空 dataflow 且 traceable → 不被误判
    #[test]
    fn traceable_dataflow_no_unhelpful_issue() {
        use crate::views::models::{EdgeType, NodeType};
        let n1 = ViewNode {
            node_id: "N-1".to_string(),
            node_type: NodeType::InputSource,
            label: "din".to_string(),
            description: "输入数据".to_string(),
            confidence: ClaimConfidence::Confirmed,
            trace_refs: vec![make_trace_ref(Some("EV-1"), None)],
            layout: None,
        };
        let n2 = ViewNode {
            node_id: "N-2".to_string(),
            node_type: NodeType::ProcessingStep,
            label: "process".to_string(),
            description: "处理".to_string(),
            confidence: ClaimConfidence::Confirmed,
            trace_refs: vec![make_trace_ref(Some("EV-2"), None)],
            layout: None,
        };
        let edge = ViewEdge {
            edge_id: "E-1".to_string(),
            edge_type: EdgeType::DataFlow,
            source_node_id: "N-1".to_string(),
            target_node_id: "N-2".to_string(),
            label: None,
            description: "data flow".to_string(),
            confidence: ClaimConfidence::Inferred,
            trace_refs: vec![make_trace_ref(Some("EV-1"), None)],
        };
        let mut ev = HashSet::new();
        ev.insert("EV-1".to_string());
        ev.insert("EV-2".to_string());
        let graph = make_view_graph(ViewType::Dataflow, vec![n1, n2], vec![edge], None);
        let input = ViewEvaluatorInput {
            sample_id: "S1",
            stage_id: "L0",
            view: &graph,
            evidence_id_set: &ev,
            claim_id_set: &HashSet::new(),
        };
        let (report, issues) = ViewEvaluator::evaluate(&input);

        // 不应有 EmptyOrUnhelpfulView
        assert!(
            !issues.iter().any(|i| i.kind == QualityIssueKind::EmptyOrUnhelpfulView),
            "traceable dataflow 不应有 EmptyOrUnhelpfulView: {:?}",
            issues
        );
        assert_eq!(report.trace_resolvable_ratio, 1.0);
    }

    /// P2: 标签高度重复 → LowSemanticDiversity
    #[test]
    fn duplicate_labels_emit_low_semantic_diversity() {
        let n1 = make_node("N-1", "mod_same", vec![make_trace_ref(Some("EV-1"), None)]);
        let n2 = make_node("N-2", "mod_same", vec![make_trace_ref(Some("EV-2"), None)]);
        let n3 = make_node("N-3", "mod_same", vec![make_trace_ref(Some("EV-3"), None)]);
        // 3 个节点 + 3 个标签全相同 = 2 duplicates / 3 unique = 33% unique → < 50% → 触发
        let edge = make_edge("E-1", "N-1", "N-2", vec![make_trace_ref(Some("EV-1"), None)]);
        let graph = make_view_graph(ViewType::Structure, vec![n1, n2, n3], vec![edge], None);
        let mut ev = HashSet::new();
        ev.insert("EV-1".to_string());
        ev.insert("EV-2".to_string());
        ev.insert("EV-3".to_string());
        let input = ViewEvaluatorInput {
            sample_id: "S1",
            stage_id: "L0",
            view: &graph,
            evidence_id_set: &ev,
            claim_id_set: &HashSet::new(),
        };
        let (report, _issues) = ViewEvaluator::evaluate(&input);

        let diversity_issues: Vec<_> = _issues.iter().filter(|i| i.kind == QualityIssueKind::LowSemanticDiversity).collect();
        assert!(
            !diversity_issues.is_empty(),
            "重复标签应产生 LowSemanticDiversity: {:?}",
            _issues
        );
        // 追溯可解析率应为 1.0
        assert_eq!(report.trace_resolvable_ratio, 1.0);
    }

    /// P2: 标签多样性正常时不触发 LowSemanticDiversity
    #[test]
    fn varied_labels_no_low_semantic_diversity() {
        let n1 = make_node("N-1", "module_a", vec![make_trace_ref(Some("EV-1"), None)]);
        let n2 = make_node("N-2", "module_b", vec![make_trace_ref(Some("EV-2"), None)]);
        let edge = make_edge("E-1", "N-1", "N-2", vec![make_trace_ref(Some("EV-1"), None)]);
        let graph = make_view_graph(ViewType::Structure, vec![n1, n2], vec![edge], None);
        let mut ev = HashSet::new();
        ev.insert("EV-1".to_string());
        ev.insert("EV-2".to_string());
        let input = ViewEvaluatorInput {
            sample_id: "S1",
            stage_id: "L0",
            view: &graph,
            evidence_id_set: &ev,
            claim_id_set: &HashSet::new(),
        };
        let (_, issues) = ViewEvaluator::evaluate(&input);

        let diversity_issues: Vec<_> = issues.iter().filter(|i| i.kind == QualityIssueKind::LowSemanticDiversity).collect();
        assert!(
            diversity_issues.is_empty(),
            "正常多样性不应产生 LowSemanticDiversity: {:?}",
            issues
        );
    }
}
