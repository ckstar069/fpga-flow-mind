# Phase 5 Trace & Grounded Q&A 后端设计

---
status: draft
updated: 2026-06-13
---

> 本文档定义 Phase 5 的后端概要设计、核心流程、Tauri commands、安全边界和 Provider 设计。实施前必须与 `phase-5-trace-model.md` 和 `phase-5-trace-and-qa-requirements.md` 对齐。
>
> 本文档为 draft，需审核收口后方可进入编码。

## 1. 概要设计

### 1.1 组件分层

```text
前端 selection
  │
  ▼
TraceResolver ──────────────→ TraceRefResolved[]
  │
  ▼
SourceExcerptResolver ────────→ SourceExcerpt
  │
  ▼
GroundedQaContextBuilder ─────→ GroundedQuestion
  │
  ▼
GroundedQaProvider / MockProvider ─→ GroundedAnswer
  │
  ▼
GroundedQaValidator ──────────→ GroundedAnswer (validated)
```

### 1.2 组件职责

| 组件 | 职责 |
|------|------|
| `TraceResolver` | 根据 `SelectedTraceTarget` 解析出 `TraceRefResolved[]`，只引用已有 claim/evidence，不伪造 |
| `SourceExcerptResolver` | 根据 `evidence_id` 或 `SourceLocation` 安全读取目标项目文件，返回 `SourceExcerpt` |
| `GroundedQaContextBuilder` | 从 `ImplementationUnderstanding` + `EvidenceCollection` + `SelectedTraceTarget` 构建 Q&A 上下文 |
| `GroundedQaProvider` | trait：接收 context，返回 `GroundedAnswer` |
| `MockProvider` | 确定性 mock，验证 answer 数据结构和 citation 绑定 |
| `GroundedQaValidator` | 检查 answer 是否包含 citations、confidence 是否合法、是否出现审计用语 |

## 2. 详细流程

### 2.1 点击 ViewNode

1. 前端记录 `SelectedTraceTarget::ViewNode { view_type, node_id }`。
2. 调用 `resolve_trace_target(...)`。
3. 后端在对应 `ViewGraph.nodes` 中找到 node，读取 `trace_refs`。
4. 对每个 `ViewTraceRef`：
   - 若 `claim_id` 存在，查找 `ImplementationUnderstanding.claims`，生成 `ClaimSnapshot`。
   - 若 `evidence_id` 存在，查找 `EvidenceCollection.evidence_items`，生成 `EvidenceSnapshot`。
   - 若都不存在，返回 `TraceResolution::MissingEvidence` 或 `MissingClaim`。
5. 返回 `TraceRefResolved[]`。

### 2.2 点击 ViewEdge

流程同 ViewNode，但目标为 `ViewGraph.edges`。

### 2.3 点击 claim

1. 前端记录 `SelectedTraceTarget::Claim { claim_id }`。
2. 调用 `resolve_trace_target(...)`。
3. 后端查找 `ImplementationUnderstanding.claims`，生成 `ClaimSnapshot`。
4. 对其 `evidence_refs` 逐条解析为 `EvidenceSnapshot`。
5. 若 `has_evidence_gap=true`，生成一条 `TraceResolution::ClaimOnly` 的 resolved trace。

### 2.4 点击 evidence item

1. 前端记录 `SelectedTraceTarget::Evidence { evidence_id }`。
2. 调用 `resolve_trace_target(...)`。
3. 后端直接返回该 evidence 的 `EvidenceSnapshot`。
4. 用户可进一步点击"查看源码片段"调用 `get_source_excerpt(...)`。

### 2.5 提问 grounded Q&A

1. 前端组装 `GroundedQuestion`：question + stage_id + selected_target + understanding + evidence_collection。
2. 调用 `ask_grounded_question(...)`。
3. 后端 `GroundedQaContextBuilder` 构建 prompt/context。
4. `MockProvider` 基于关键词/模板生成 `GroundedAnswer`（确定性）。
5. `GroundedQaValidator` 检查：
   - 至少一个 citation；
   - confidence 在合法枚举内；
   - 文本不含"PASS/HOLD""正确/错误"等审计用语。
6. 返回 `GroundedAnswer`。

## 3. Tauri Commands 草案

### 3.1 resolve_trace_target

```rust
#[tauri::command]
pub fn resolve_trace_target(
    target: SelectedTraceTarget,
    understanding: ImplementationUnderstanding,
    evidence_collection: EvidenceCollection,
    views: Vec<ViewGraph>,
) -> CommandResult<Vec<TraceRefResolved>> {
    // 不访问目标项目文件系统
}
```

| 维度 | 说明 |
|------|------|
| 输入 | target + understanding + evidence_collection + views |
| 输出 | `Vec<TraceRefResolved>` |
| 错误分支 | 找不到对应 node/edge/claim/evidence；views 中无对应 view_type |
| 访问目标项目 | ❌ 否 |

### 3.2 get_source_excerpt

```rust
#[tauri::command]
pub fn get_source_excerpt(
    location: SourceLocation,
    root_path: String,
) -> CommandResult<SourceExcerpt> {
    // 只读访问目标项目文件
}
```

| 维度 | 说明 |
|------|------|
| 输入 | `SourceLocation`（含 source_path + line_range）+ `root_path` |
| 输出 | `SourceExcerpt` |
| 错误分支 | source_path 不在 root_path 下；文件不存在；symlink；二进制；非 UTF-8；超大文件；line_range 越界 |
| 访问目标项目 | ✅ 只读 |

### 3.3 ask_grounded_question

```rust
#[tauri::command]
pub fn ask_grounded_question(
    question: GroundedQuestion,
) -> CommandResult<GroundedAnswer> {
    // 不访问目标项目文件系统（依赖 question 中已携带的 understanding/evidence）
}
```

| 维度 | 说明 |
|------|------|
| 输入 | `GroundedQuestion` |
| 输出 | `GroundedAnswer` |
| 错误分支 | question 为空；context 中 evidence_collection 为空；provider 生成失败；validator 拒绝 |
| 访问目标项目 | ❌ 否（源码读取由 get_source_excerpt 负责） |

## 4. 只读安全设计

### 4.1 source_path 校验

读取目标项目文件前必须验证：

1. `source_path` 是绝对路径。
2. `source_path` 位于 `root_path` 之下（使用 `std::path::Path::starts_with` 解析真实路径）。
3. `source_path` 不是 symlink（使用 `std::fs::symlink_metadata` 检查 `file_type().is_symlink()`）。
4. `source_path` 存在且是文件。
5. `source_path` 大小 ≤ 5 MB。
6. `source_path` 内容可读且为 UTF-8。

### 4.2 line_range 校验

1. `start >= 1`。
2. `start <= end`。
3. `end <= 文件总行数`（或至少 `end` 不超过允许读取的最大行）。
4. 若 `end` 越界，返回 error 而不是读取到文件末尾静默截断。

### 4.3 拒绝操作

- 不调用 `std::fs::write`、`create_dir`、`remove_file`、`rename`、`copy`。
- 不调用 `std::process::Command`。
- 不运行 Vivado / synthesis / implementation / bitstream。

## 5. Provider 设计

### 5.1 GroundedQaProvider trait

```rust
pub trait GroundedQaProvider: Send + Sync {
    fn generate_answer(
        &self,
        context: &GroundedQaContext,
    ) -> Result<GroundedAnswer, GroundedQaError>;
}
```

### 5.2 GroundedQaContext

```rust
pub struct GroundedQaContext {
    pub question: String,
    pub stage_id: String,
    pub selected_target: Option<SelectedTraceTarget>,
    pub understanding_summary: String,
    pub claims: Vec<ImplementationClaim>,
    pub evidence_collection: EvidenceCollection,
}
```

### 5.3 MockProvider

- 目的：仅验证 `GroundedAnswer` 数据结构和 UI 闭环。
- 行为：
  - 若 question 包含"位宽""width"，返回一条关于位宽的 claim，引用某 signal evidence。
  - 若 question 包含"做什么""功能"，返回基于 `summary.short` 的回答。
  - 若 question 无法匹配任何关键词，返回 `confidence = unknown` 的回答。
- 所有返回必须包含至少一个 `GroundedAnswerCitation`。
- `is_degraded = true`。

### 5.4 真实 LLM

- Phase 5 不默认启用真实云端 LLM。
- 真实 LLM Provider 可后续实现，但需满足：
  - 显式配置（非默认开启）；
  - 调用可审计；
  - 返回结果经过 `GroundedQaValidator`。

## 6. GroundedQaValidator 规则

1. `claims` 非空或 `text` 非空。
2. 每条 `claim` 的 `confidence` 在合法枚举内。
3. `citations` 非空；每条 citation 至少包含 `evidence_id`、`claim_id`、`source_location` 之一。
4. citation 引用的 `evidence_id` / `claim_id` 必须在输入 context 中存在。
5. 回答文本不包含"PASS""HOLD""正确""错误""审计"等词汇。
6. 若 `confidence = unknown`，必须提供 `reason`。

## 7. 模块布局建议

```text
src-tauri/src/
├── trace/
│   ├── mod.rs
│   ├── models.rs           ← Phase 5 新增类型（SelectedTraceTarget, TraceRefResolved, ...）
│   ├── resolver.rs         ← TraceResolver
│   ├── source_resolver.rs  ← SourceExcerptResolver
│   └── qa/
│       ├── mod.rs
│       ├── context_builder.rs
│       ├── provider.rs
│       ├── mock_provider.rs
│       └── validator.rs
├── commands/
│   ├── generate_views.rs   ← 已有
│   ├── resolve_trace_target.rs
│   ├── get_source_excerpt.rs
│   └── ask_grounded_question.rs
└── lib.rs
```

## 8. 错误码扩展

| 错误码 | 场景 |
|--------|------|
| `trace_target_not_found` | 找不到对应 node/edge/claim/evidence |
| `source_path_not_allowed` | source_path 不在 root_path 下或为 symlink |
| `source_file_unreadable` | 文件不存在/无权限/二进制/非 UTF-8/超大 |
| `line_range_invalid` | line_range 越界或格式错误 |
| `qa_generation_failed` | provider 生成失败 |
| `qa_validation_failed` | validator 拒绝（无 citation、审计用语等） |

## 9. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-13 | 初始创建（draft）：定义 resolver、provider、commands、安全边界、validator、模块布局 | Claude |
