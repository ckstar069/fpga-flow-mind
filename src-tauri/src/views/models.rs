use serde::{Deserialize, Serialize};

use crate::understanding::models::ClaimConfidence;

// ─── 枚举 ────────────────────────────────────────────────────────────

/// 视图类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewType {
    Structure,
    Dataflow,
    Timing,
}

/// 节点类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    // 通用于三类视图
    Module,
    Function,
    Interface,
    Signal,
    ProcessingStep,
    // 结构图专用
    Class,
    Constant,
    // 数据流图专用
    InputSource,
    OutputTarget,
    IntermediateData,
    // 时序图专用
    PipelineStage,
    ClockDomain,
    ResetDomain,
}

/// 边类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    // 通用于三类视图
    Contains,
    Calls,
    References,
    DependsOn,
    // 数据流图专用
    DataFlow,
    // 时序图专用
    SequentialOrder,
    PipelineForward,
    ClockDriven,
}

// ─── 证据追溯 ────────────────────────────────────────────────────────

/// 证据追溯引用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewTraceRef {
    /// 关联的 claim_id
    pub claim_id: Option<String>,
    /// 关联的 evidence_id
    pub evidence_id: Option<String>,
    /// 置信度
    pub confidence: ClaimConfidence,
    /// 关联说明
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relevance: Option<String>,
}

// ─── 布局提示 ────────────────────────────────────────────────────────

/// 布局提示 — 前端使用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewLayoutHint {
    /// 建议列位置（0-based）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
    /// 建议行位置（0-based）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row: Option<u32>,
    /// 建议层级（0=顶层）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<u32>,
    /// 分组标识
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

// ─── 元信息 ──────────────────────────────────────────────────────────

/// 视图元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewMeta {
    /// 来源 stage_id
    pub stage_id: String,
    /// 视图类型
    pub view_type: ViewType,
    /// 来源 IU 的 provider
    pub source_provider: String,
    /// 是否来自 degraded IU
    pub is_degraded_source: bool,
    /// 生成时间 (ISO 8601)
    pub generated_at: String,
    /// 空视图原因（nodes=[] 且 edges=[] 时说明）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub empty_reason: Option<String>,
}

// ─── 核心结构 ────────────────────────────────────────────────────────

/// 视图节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewNode {
    /// 节点唯一标识
    pub node_id: String,
    /// 节点类型
    pub node_type: NodeType,
    /// 显示标签
    pub label: String,
    /// 描述文本
    pub description: String,
    /// 置信度
    pub confidence: ClaimConfidence,
    /// 证据追溯列表
    pub trace_refs: Vec<ViewTraceRef>,
    /// 布局提示
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<ViewLayoutHint>,
}

/// 视图边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewEdge {
    /// 边唯一标识
    pub edge_id: String,
    /// 边类型
    pub edge_type: EdgeType,
    /// 来源 node_id
    pub source_node_id: String,
    /// 目标 node_id
    pub target_node_id: String,
    /// 边标签
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// 边描述
    pub description: String,
    /// 置信度
    pub confidence: ClaimConfidence,
    /// 证据追溯列表
    pub trace_refs: Vec<ViewTraceRef>,
}

/// 视图图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewGraph {
    /// 视图类型
    pub view_type: ViewType,
    /// 来源 stage_id
    pub stage_id: String,
    /// 节点列表
    pub nodes: Vec<ViewNode>,
    /// 边列表
    pub edges: Vec<ViewEdge>,
    /// 元信息
    pub meta: ViewMeta,
}

// ─── 辅助 ────────────────────────────────────────────────────────────

impl ViewTraceRef {
    /// 从 EvidenceRef 转换（Phase 3 证据引用 → Phase 4 追溯引用）
    pub fn from_evidence_ref(
        evidence_id: &str,
        confidence: ClaimConfidence,
        relevance: Option<String>,
    ) -> Self {
        Self {
            claim_id: None,
            evidence_id: Some(evidence_id.to_string()),
            confidence,
            relevance,
        }
    }
}

// ─── 测试 ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {

    // ─── mod_01: ViewType 序列化 ──────────────────────────────────────
    #[test]
    fn mod_01_view_type_serialization() {
        assert_eq!(
            serde_json::to_string(&super::ViewType::Structure).unwrap(),
            "\"structure\""
        );
        assert_eq!(
            serde_json::to_string(&super::ViewType::Dataflow).unwrap(),
            "\"dataflow\""
        );
        assert_eq!(
            serde_json::to_string(&super::ViewType::Timing).unwrap(),
            "\"timing\""
        );
    }

    // ─── mod_02: NodeType 序列化 ─────────────────────────────────────
    #[test]
    fn mod_02_node_type_roundtrip() {
        let types = vec![
            super::NodeType::Module,
            super::NodeType::Function,
            super::NodeType::Interface,
            super::NodeType::Signal,
            super::NodeType::ProcessingStep,
            super::NodeType::InputSource,
            super::NodeType::OutputTarget,
            super::NodeType::PipelineStage,
            super::NodeType::ClockDomain,
        ];
        for t in &types {
            let json = serde_json::to_string(t).unwrap();
            let back: super::NodeType = serde_json::from_str(&json).unwrap();
            assert_eq!(*t, back);
        }
    }

    // ─── mod_03: EdgeType 序列化 ─────────────────────────────────────
    #[test]
    fn mod_03_edge_type_roundtrip() {
        let types = vec![
            super::EdgeType::Contains,
            super::EdgeType::Calls,
            super::EdgeType::References,
            super::EdgeType::DependsOn,
            super::EdgeType::DataFlow,
            super::EdgeType::SequentialOrder,
            super::EdgeType::PipelineForward,
            super::EdgeType::ClockDriven,
        ];
        for t in &types {
            let json = serde_json::to_string(t).unwrap();
            let back: super::EdgeType = serde_json::from_str(&json).unwrap();
            assert_eq!(*t, back);
        }
    }

    // ─── mod_04: ViewGraph 完整序列化 ───────────────────────────────
    #[test]
    fn mod_04_viewgraph_roundtrip() {
        use crate::understanding::models::ClaimConfidence;

        let graph = super::ViewGraph {
            view_type: super::ViewType::Structure,
            stage_id: "L0".to_string(),
            nodes: vec![super::ViewNode {
                node_id: "N-structure-0001".to_string(),
                node_type: super::NodeType::Module,
                label: "mod_a".to_string(),
                description: "模块 A".to_string(),
                confidence: ClaimConfidence::Confirmed,
                trace_refs: vec![super::ViewTraceRef {
                    claim_id: Some("CL-L0-000001".to_string()),
                    evidence_id: Some("EV-L0-000001".to_string()),
                    confidence: ClaimConfidence::Confirmed,
                    relevance: Some("定义了模块".to_string()),
                }],
                layout: Some(super::ViewLayoutHint {
                    column: Some(0),
                    row: Some(0),
                    depth: Some(0),
                    group: None,
                }),
            }],
            edges: vec![super::ViewEdge {
                edge_id: "E-structure-0001".to_string(),
                edge_type: super::EdgeType::References,
                source_node_id: "N-structure-0001".to_string(),
                target_node_id: "N-structure-0002".to_string(),
                label: None,
                description: "模块引用信号".to_string(),
                confidence: ClaimConfidence::Confirmed,
                trace_refs: vec![],
            }],
            meta: super::ViewMeta {
                stage_id: "L0".to_string(),
                view_type: super::ViewType::Structure,
                source_provider: "mock".to_string(),
                is_degraded_source: false,
                generated_at: "2026-06-12T10:00:00Z".to_string(),
                empty_reason: None,
            },
        };

        let json = serde_json::to_string_pretty(&graph).unwrap();
        let back: super::ViewGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(back.view_type, super::ViewType::Structure);
        assert_eq!(back.stage_id, "L0");
        assert_eq!(back.nodes.len(), 1);
        assert_eq!(back.edges.len(), 1);
        assert!(back.nodes[0].layout.is_some());
        assert!(back.meta.empty_reason.is_none());
    }

    // ─── mod_05: empty_reason 可选字段 ────────────────────────────────
    #[test]
    fn mod_05_empty_reason_optional() {
        let json_with = r#"{"stage_id":"L0","view_type":"structure","source_provider":"mock","is_degraded_source":false,"generated_at":"2026-06-12T10:00:00Z","empty_reason":"时序信息不足"}"#;
        let meta: super::ViewMeta = serde_json::from_str(json_with).unwrap();
        assert_eq!(meta.empty_reason, Some("时序信息不足".to_string()));

        let json_without = r#"{"stage_id":"L0","view_type":"structure","source_provider":"mock","is_degraded_source":false,"generated_at":"2026-06-12T10:00:00Z"}"#;
        let meta2: super::ViewMeta = serde_json::from_str(json_without).unwrap();
        assert_eq!(meta2.empty_reason, None);
    }
}
