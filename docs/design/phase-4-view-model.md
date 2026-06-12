# Phase 4 视图数据模型设计

---
status: active
updated: 2026-06-12
---

> 本文档定义 Phase 4 三类视图（结构图、数据流图、时序/流水图）的数据结构。ViewGraph 从 `ImplementationUnderstanding` 确定性派生，不含渲染逻辑。

## 1. 核心对象概览

```text
ViewGraph
├── view_type: ViewType          (structure | dataflow | timing)
├── stage_id: String
├── nodes: Vec<ViewNode>
├── edges: Vec<ViewEdge>
├── layout_hints: Vec<ViewLayoutHint>
└── meta: ViewMeta
```

## 2. ViewType

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewType {
    Structure,
    Dataflow,
    Timing,
}
```

| TypeScript | 值 |
|------------|-----|
| `ViewType` | `'structure' \| 'dataflow' \| 'timing'` |

## 3. ViewNode

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewNode {
    /// 节点唯一标识（如 "N-structure-0001"）
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
    pub layout: Option<ViewLayoutHint>,
}
```

```typescript
interface ViewNode {
  node_id: string;
  node_type: NodeType;
  label: string;
  description: string;
  confidence: ClaimConfidence;
  trace_refs: ViewTraceRef[];
  layout?: ViewLayoutHint;
}
```

### 3.1 NodeType

```rust
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
```

| TypeScript | 值 |
|------------|-----|
| `NodeType` | `'module' \| 'function' \| 'interface' \| 'signal' \| 'processing_step' \| 'class' \| 'constant' \| 'input_source' \| 'output_target' \| 'intermediate_data' \| 'pipeline_stage' \| 'clock_domain' \| 'reset_domain'` |

### 3.2 视图 → 节点类型映射

| ViewType | 使用 NodeType |
|----------|--------------|
| structure | Module, Function, Interface, Signal, Class, Constant |
| dataflow | InputSource, OutputTarget, ProcessingStep, IntermediateData, Signal, Module |
| timing | PipelineStage, ClockDomain, ResetDomain, ProcessingStep |

## 4. ViewEdge

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewEdge {
    /// 边唯一标识（如 "E-structure-0001"）
    pub edge_id: String,
    /// 来源 node_id
    pub source_node_id: String,
    /// 目标 node_id
    pub target_node_id: String,
    /// 边类型
    pub edge_type: EdgeType,
    /// 边标签
    pub label: Option<String>,
    /// 边描述
    pub description: String,
    /// 置信度
    pub confidence: ClaimConfidence,
    /// 证据追溯列表
    pub trace_refs: Vec<ViewTraceRef>,
}
```

```typescript
interface ViewEdge {
  edge_id: string;
  source_node_id: string;
  target_node_id: string;
  edge_type: EdgeType;
  label?: string;
  description: string;
  confidence: ClaimConfidence;
  trace_refs: ViewTraceRef[];
}
```

### 4.1 EdgeType

```rust
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
```

| TypeScript | 值 |
|------------|-----|
| `EdgeType` | `'contains' \| 'calls' \| 'references' \| 'depends_on' \| 'data_flow' \| 'sequential_order' \| 'pipeline_forward' \| 'clock_driven'` |

### 4.2 视图 → 边类型映射

| ViewType | 使用 EdgeType |
|----------|--------------|
| structure | Contains, Calls, References |
| dataflow | DataFlow, DependsOn |
| timing | SequentialOrder, PipelineForward, ClockDriven |

## 5. ViewTraceRef

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewTraceRef {
    /// 关联的 claim_id
    pub claim_id: Option<String>,
    /// 关联的 evidence_id
    pub evidence_id: Option<String>,
    /// 置信度
    pub confidence: ClaimConfidence,
    /// 关联说明（如 "定义了模块接口"）
    pub relevance: Option<String>,
}
```

```typescript
interface ViewTraceRef {
  claim_id?: string;
  evidence_id?: string;
  confidence: ClaimConfidence;
  relevance?: string;
}
```

**约束**：
- `claim_id` 和 `evidence_id` 至少有一个非空
- 如果 `claim_id` 存在，必须指向 IU 中存在的 claim
- `confidence` 与对应 claim 的 confidence 一致

## 6. ViewLayoutHint

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewLayoutHint {
    /// 建议列位置（0-based）
    pub column: Option<u32>,
    /// 建议行位置（0-based）
    pub row: Option<u32>,
    /// 建议层级（0=顶层，数值越大越深）
    pub depth: Option<u32>,
}
```

```typescript
interface ViewLayoutHint {
  column?: number;
  row?: number;
  depth?: number;
}
```

> MVP 阶段使用简单 grid 布局，`layout_hints` 提供初始排列建议。前端可根据 hints 计算渲染位置，不实现自动布局算法。

## 7. ViewMeta

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewMeta {
    /// 来源 stage_id
    pub stage_id: String,
    /// 视图类型
    pub view_type: ViewType,
    /// 来源 IU 的 generation_meta.provider
    pub source_provider: String,
    /// 是否来自 degraded IU
    pub is_degraded_source: bool,
    /// 生成时间
    pub generated_at: String,
    /// 空视图原因（nodes=[] 且 edges=[] 时说明为何无数据）
    /// 非空视图时为 None
    #[serde(skip_serializing_if = "Option::is_none")]
    pub empty_reason: Option<String>,
}
```

```typescript
interface ViewMeta {
  stage_id: string;
  view_type: ViewType;
  source_provider: string;
  is_degraded_source: boolean;
  generated_at: string;
  empty_reason?: string;  // nodes/edges 为空时的原因说明
}
```

## 8. ViewGraph

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewGraph {
    pub view_type: ViewType,
    pub stage_id: String,
    pub nodes: Vec<ViewNode>,
    pub edges: Vec<ViewEdge>,
    pub meta: ViewMeta,
}
```

```typescript
interface ViewGraph {
  view_type: ViewType;
  stage_id: string;
  nodes: ViewNode[];
  edges: ViewEdge[];
  meta: ViewMeta;
}
```

**约束**：
- `nodes` 和 `edges` 允许为空（表示无数据可生成视图，此时 `meta.empty_reason` 应非空）
- 同一 `ViewGraph` 内所有 `node_id` 必须唯一
- 所有 `source_node_id` 和 `target_node_id` 必须引用当前 `nodes` 中已存在的 `node_id`
- 每个 ViewGraph 专属于一个 stage + 一种 ViewType

## 9. 与 ImplementationUnderstanding 的关系

```text
ImplementationUnderstanding (Phase 3)
  │
  ├── module_summaries ──────→ ViewGraph(structure).nodes
  ├── signal_summaries ──────→ ViewGraph(structure).nodes + ViewGraph(dataflow).nodes
  ├── interface_summaries ───→ ViewGraph(structure).nodes
  ├── processing_steps ──────→ ViewGraph(dataflow).nodes + ViewGraph(timing).nodes
  ├── claims ────────────────→ ViewNode.trace_refs + ViewEdge.trace_refs
  └── unknowns / gaps ───────→ ViewMeta.is_degraded_source + 空状态标注
```

## 10. TypeScript 类型完整定义

```typescript
// 与 Rust 保持一致，由 workspace.ts 扩展

export type ViewType = 'structure' | 'dataflow' | 'timing';

export type NodeType =
  | 'module' | 'function' | 'interface' | 'signal' | 'processing_step'
  | 'class' | 'constant'
  | 'input_source' | 'output_target' | 'intermediate_data'
  | 'pipeline_stage' | 'clock_domain' | 'reset_domain';

export type EdgeType =
  | 'contains' | 'calls' | 'references' | 'depends_on'
  | 'data_flow'
  | 'sequential_order' | 'pipeline_forward' | 'clock_driven';

export interface ViewTraceRef {
  claim_id?: string;
  evidence_id?: string;
  confidence: ClaimConfidence;
  relevance?: string;
}

export interface ViewLayoutHint {
  column?: number;
  row?: number;
  depth?: number;
}

export interface ViewNode {
  node_id: string;
  node_type: NodeType;
  label: string;
  description: string;
  confidence: ClaimConfidence;
  trace_refs: ViewTraceRef[];
  layout?: ViewLayoutHint;
}

export interface ViewEdge {
  edge_id: string;
  source_node_id: string;
  target_node_id: string;
  edge_type: EdgeType;
  label?: string;
  description: string;
  confidence: ClaimConfidence;
  trace_refs: ViewTraceRef[];
}

export interface ViewMeta {
  stage_id: string;
  view_type: ViewType;
  source_provider: string;
  is_degraded_source: boolean;
  generated_at: string;
}

export interface ViewGraph {
  view_type: ViewType;
  stage_id: string;
  nodes: ViewNode[];
  edges: ViewEdge[];
  meta: ViewMeta;
}
```

## 11. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-12 | 初始创建（draft）：定义 ViewGraph/ViewNode/ViewEdge/ViewTraceRef/ViewLayoutHint/ViewMeta + NodeType/EdgeType/ViewType 枚举 | Claude |
