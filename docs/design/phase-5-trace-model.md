# Phase 5 Trace & Grounded Q&A 数据模型设计

---
status: active
updated: 2026-06-14
---

> 本文档定义 Phase 5 所需的数据结构：选择目标、追溯解析结果、源码片段、面板状态、grounded 问答输入输出。所有 Rust/TypeScript 草案与 `mvp-functional-contract.md`、`phase-2-evidence-model.md`、`phase-3-understanding-model.md`、`phase-4-view-model.md` 对齐。
>
> 本文档已收口（status=active），是 Phase 5 编码依据。

## 1. 设计目标

- 统一描述用户在视图中点击的节点/边、claim、evidence 等选择目标。
- 通过已存在的 `evidence_id`、`claim_id`、`node_id`、`edge_id` 解析出可展示的追溯信息，不伪造 ID。
- 源码片段只读读取，行号 1-based 闭区间，支持截断与异常边界。
- Grounded Q&A 的回答必须绑定 citations（非 unknown 时）或明确 reason + warning（unknown 时），明确 confidence。

## 2. 选择模型

### 2.1 SelectedTraceTarget

描述前端的一次点击选择。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SelectedTraceTarget {
    ViewNode {
        view_type: ViewType,
        node_id: String,
    },
    ViewEdge {
        view_type: ViewType,
        edge_id: String,
    },
    Claim {
        claim_id: String,
    },
    Evidence {
        evidence_id: String,
    },
}
```

```typescript
type SelectedTraceTarget =
  | { kind: 'view_node'; view_type: ViewType; node_id: string }
  | { kind: 'view_edge'; view_type: ViewType; edge_id: string }
  | { kind: 'claim'; claim_id: string }
  | { kind: 'evidence'; evidence_id: string };
```

**约束**：
- `node_id` / `edge_id` 必须来自当前已加载的 `ViewGraph`。
- `claim_id` 必须来自当前 `ImplementationUnderstanding.claims`。
- `evidence_id` 必须来自当前 `EvidenceCollection.evidence_items`。

## 3. 追溯解析模型

### 3.1 TraceRefResolved

将 `ViewTraceRef` 或 `EvidenceRef` 解析为可展示对象。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceRefResolved {
    /// 解析来源（view_node / view_edge / claim / evidence）
    pub source_kind: TraceSourceKind,
    /// 关联 claim，可能为空
    pub claim: Option<ClaimSnapshot>,
    /// 关联 evidence，可能为空
    pub evidence: Option<EvidenceSnapshot>,
    /// 综合 confidence
    pub confidence: ClaimConfidence,
    /// 关联说明
    pub relevance: Option<String>,
    /// 解析状态
    pub resolution: TraceResolution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceResolution {
    /// claim 和 evidence 均存在
    Resolved,
    /// 只有 claim，无 evidence（evidence_gap）
    ClaimOnly,
    /// 只有 evidence，无 claim
    EvidenceOnly,
    /// 引用的 claim_id 不存在
    MissingClaim,
    /// 引用的 evidence_id 不存在
    MissingEvidence,
}
```

```typescript
interface TraceRefResolved {
  source_kind: TraceSourceKind;
  claim?: ClaimSnapshot;
  evidence?: EvidenceSnapshot;
  confidence: ClaimConfidence;
  relevance?: string;
  resolution: 'resolved' | 'claim_only' | 'evidence_only' | 'missing_claim' | 'missing_evidence';
}
```

### 3.2 ClaimSnapshot

claim 的轻量展示形态，不含完整 evidence_refs 列表（避免循环过大）。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimSnapshot {
    pub claim_id: String,
    pub category: ClaimCategory,
    pub description: String,
    pub confidence: ClaimConfidence,
    pub evidence_ref_count: usize,
    pub has_evidence_gap: bool,
}
```

```typescript
interface ClaimSnapshot {
  claim_id: string;
  category: ClaimCategory;
  description: string;
  confidence: ClaimConfidence;
  evidence_ref_count: number;
  has_evidence_gap: boolean;
}
```

### 3.3 EvidenceSnapshot

evidence 的轻量展示形态。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSnapshot {
    pub evidence_id: String,
    pub source_path: String,
    pub language: Language,
    pub source_kind: SourceKind,
    pub line_range: LineRange,
    pub symbol: Option<String>,
    pub summary: String,
    pub strength: EvidenceStrength,
}
```

```typescript
interface EvidenceSnapshot {
  evidence_id: string;
  source_path: string;
  language: Language;
  source_kind: SourceKind;
  line_range: LineRange;
  symbol?: string;
  summary: string;
  strength: EvidenceStrength;
}
```

## 4. 源码位置与片段模型

### 4.1 SourceLocation

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    /// 绝对路径，必须属于当前 workspace root
    pub source_path: String,
    /// 1-based 闭区间
    pub line_range: LineRange,
    /// 所属 evidence_id（若有）
    pub evidence_id: Option<String>,
}
```

```typescript
interface SourceLocation {
  source_path: string;
  line_range: LineRange;
  evidence_id?: string;
}
```

### 4.2 SourceExcerpt

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceExcerpt {
    pub location: SourceLocation,
    pub language: Language,
    /// 源码行列表，每行含原始行号
    pub lines: Vec<SourceLine>,
    /// 是否被截断
    pub is_truncated: bool,
    /// 截断原因/提示
    pub truncation_reason: Option<String>,
    /// 读取过程中的 warning
    pub warnings: Vec<ExcerptWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLine {
    /// 1-based 行号
    pub line_number: u32,
    /// 行内容（不含换行符）
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcerptWarning {
    pub error_code: String,
    pub message: String,
}
```

```typescript
interface SourceExcerpt {
  location: SourceLocation;
  language: Language;
  lines: SourceLine[];
  is_truncated: boolean;
  truncation_reason?: string;
  warnings: ExcerptWarning[];
}

interface SourceLine {
  line_number: number;
  content: string;
}

interface ExcerptWarning {
  error_code: string;
  message: string;
}
```

### 4.3 读取边界

| 参数 | 建议值 | 说明 |
|------|--------|------|
| 单次 excerpt 最大行数 | 100 行 | 超过则截断并提示 |
| 单次 excerpt 最大字符数 | 8192 字符 | 超过则截断 |
| 单个文件最大读取大小 | 5 MB | 与 Phase 2 一致 |
| 越界 line_range | 拒绝 | `end > 文件总行数` 时返回错误 |
| 二进制文件 | 拒绝 | 返回 `binary_file_skipped` |
| 非 UTF-8 文件 | 拒绝 | 返回 `non_utf8_file_skipped` |
| symlink / path traversal | 拒绝 | source_path 必须真实位于 workspace root 下 |

## 5. 面板状态模型

### 5.1 TracePanelState

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracePanelState {
    /// 当前选中的目标
    pub selected: Option<SelectedTraceTarget>,
    /// 解析后的 trace 列表
    pub resolved_traces: Vec<TraceRefResolved>,
    /// 面板加载状态
    pub status: TracePanelStatus,
    /// 错误信息
    pub error: Option<UiError>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TracePanelStatus {
    Empty,
    Loading,
    Loaded,
    Error,
}
```

```typescript
interface TracePanelState {
  selected?: SelectedTraceTarget;
  resolved_traces: TraceRefResolved[];
  status: 'empty' | 'loading' | 'loaded' | 'error';
  error?: UiError;
}
```

## 6. Grounded Q&A 模型

### 6.1 GroundedQuestion

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedQuestion {
    /// 用户问题
    pub question: String,
    /// 当前阶段 ID
    pub stage_id: String,
    /// 可选：问题关联的已选目标
    pub selected_target: Option<SelectedTraceTarget>,
    /// 当前理解产物
    pub understanding: ImplementationUnderstanding,
    /// 当前证据集合
    pub evidence_collection: EvidenceCollection,
}
```

```typescript
interface GroundedQuestion {
  question: string;
  stage_id: string;
  selected_target?: SelectedTraceTarget;
  understanding: ImplementationUnderstanding;
  evidence_collection: EvidenceCollection;
}
```

### 6.2 GroundedAnswer

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedAnswer {
    /// 回答唯一 ID
    pub answer_id: String,
    /// 生成时间戳
    pub generated_at: String,
    /// 回答文本
    pub text: String,
    /// 回答拆分出的 claim 列表
    pub claims: Vec<GroundedAnswerClaim>,
    /// 引用证据列表；当整体 confidence = unknown 时可为空，但 warnings 必须非空
    pub citations: Vec<GroundedAnswerCitation>,
    /// 整体 confidence
    pub confidence: ClaimConfidence,
    /// 警告/提示；unknown 回答必须包含说明 evidence 不足或问题越界的 warning
    pub warnings: Vec<GroundedQaWarning>,
    /// 使用的 provider 类型
    pub provider: String,
    /// 是否 degraded（MockProvider 时为 true）
    pub is_degraded: bool,
}
```

```typescript
interface GroundedAnswer {
  answer_id: string;
  generated_at: string;
  text: string;
  claims: GroundedAnswerClaim[];
  /// 引用证据列表；当整体 confidence = unknown 时可为空，但 warnings 必须非空
  citations: GroundedAnswerCitation[];
  confidence: ClaimConfidence;
  /// 警告/提示；unknown 回答必须包含说明 evidence 不足或问题越界的 warning
  warnings: GroundedQaWarning[];
  provider: string;
  is_degraded: boolean;
}
```

### 6.3 GroundedAnswerClaim

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedAnswerClaim {
    /// claim 文本
    pub text: String,
    /// confidence
    pub confidence: ClaimConfidence,
    /// 支撑该 claim 的 citation index 列表；非 unknown claim 必须非空
    pub citation_indices: Vec<usize>,
    /// 若为 unknown，说明原因，且 citation_indices 必须为空
    pub reason: Option<String>,
}
```

```typescript
interface GroundedAnswerClaim {
  text: string;
  confidence: ClaimConfidence;
  /// 支撑该 claim 的 citation index 列表；非 unknown claim 必须非空
  citation_indices: number[];
  /// 若为 unknown，说明原因，且 citation_indices 必须为空
  reason?: string;
}
```

### 6.4 GroundedAnswerCitation

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedAnswerCitation {
    /// citation 在回答中的展示编号
    pub index: usize,
    /// 引用的 evidence_id（优先）
    pub evidence_id: Option<String>,
    /// 引用的 claim_id
    pub claim_id: Option<String>,
    /// 直接引用的源码位置
    pub source_location: Option<SourceLocation>,
    /// 引用片段摘要；unknown 回答中若引用 inspected evidence，摘要必须注明"已检查但不足以支撑结论"
    pub excerpt_summary: String,
}
```

```typescript
interface GroundedAnswerCitation {
  index: number;
  evidence_id?: string;
  claim_id?: string;
  source_location?: SourceLocation;
  /// 引用片段摘要；unknown 回答中若引用 inspected evidence，摘要必须注明"已检查但不足以支撑结论"
  excerpt_summary: string;
}
```

### 6.5 GroundedQaWarning

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedQaWarning {
    pub code: String,
    pub message: String,
}
```

```typescript
interface GroundedQaWarning {
  code: string;
  message: string;
}
```

## 7. ID 关系约束

```text
EvidenceCollection.evidence_items[].evidence_id
  ← 被 ImplementationUnderstanding.claims[].evidence_refs[].evidence_id 引用
  ← 被 ViewNode.trace_refs[].evidence_id 引用
  ← 被 ViewEdge.trace_refs[].evidence_id 引用

ImplementationUnderstanding.claims[].claim_id
  ← 被 ViewNode.trace_refs[].claim_id 引用
  ← 被 ViewEdge.trace_refs[].claim_id 引用

ViewGraph.nodes[].node_id / ViewGraph.edges[].edge_id
  ← 仅在前端选择时使用，不向后端持久化
```

**关键规则**：
- `trace_refs` 只可解析到已有 evidence/claim，不能伪造。
- resolver 发现引用不存在时，应返回 `TraceResolution::MissingEvidence` / `MissingClaim`，不得 panic。
- source_path 只读，line_range 1-based 闭区间。

## 8. unknown / inferred / missing evidence 表达

| 场景 | 表达对象 | UI 展示 |
|------|----------|---------|
| claim 无 evidence 且 has_evidence_gap=false | `TraceResolution::MissingEvidence` | 红色错误提示" claim 未绑定证据" |
| claim has_evidence_gap=true | `TraceResolution::ClaimOnly` | "证据缺失：..." |
| 节点 trace_refs 为空 | `resolved_traces = []` | "无证据追溯" |
| 节点 trace_refs 只有 inferred | `confidence = inferred` | 灰色/虚线样式 |
| Q&A 无法回答 | `GroundedAnswer.confidence = unknown` + `warnings` 非空 | "根据当前证据无法确定" |
| Q&A unknown claim 引用 inspected evidence | `GroundedAnswerClaim.confidence = unknown`，`citation_indices` 为空，`reason` 非空 | "已检查以下证据，但不足以回答：..." |

**Q&A citation 规则总结**：
- confirmed / supported / inferred / conflicting claim：必须至少有一个有效 citation。
- unknown claim：不允许伪造 citation；`citation_indices` 必须为空；`reason` 必须非空；`GroundedAnswer.warnings` 必须包含 evidence_gap 或 out_of_context 语义。
- 整体 `GroundedAnswer.citations` 为空仅当整体 `confidence = unknown` 且 `warnings` 非空。

## 9. 版本与兼容性

- Phase 5 新增类型版本跟随 `ImplementationUnderstanding` version "3.0.0"，不单独设版本号。
- 所有新增 struct 通过 serde 序列化，字段增加需保持向后兼容。

## 10. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-13 | 初始创建（draft）：定义 SelectedTraceTarget、TraceRefResolved、SourceExcerpt、GroundedQuestion/Answer 等 Rust/TS 模型 | Claude |
