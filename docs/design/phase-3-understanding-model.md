# Phase 3 ImplementationUnderstanding 数据结构设计

---
status: active
updated: 2026-06-12
---

> 本文档定义 Phase 3 `ImplementationUnderstanding` 及其子对象的数据结构，覆盖 Rust 和 TypeScript 字段设计。设计约束来自 [`phase-3-understanding-requirements.md`](../requirements/phase-3-understanding-requirements.md) 和 [`phase-2-evidence-model.md`](phase-2-evidence-model.md)。
>
> **本文档已审核收口，作为 Phase 3 编码依据。**

## 1. 核心对象概览

```text
ImplementationUnderstanding
├── stage_id: String
├── version: String
├── summary: StageSummary
├── claims: Vec<ImplementationClaim>
├── module_summaries: Vec<ModuleSummary>
├── signal_summaries: Vec<SignalSummary>
├── interface_summaries: Vec<InterfaceSummary>
├── processing_steps: Vec<ProcessingStepSummary>
├── unknowns: Vec<UnknownItem>
├── evidence_gaps: Vec<EvidenceGap>
├── generation_meta: GenerationMeta
└── stats: UnderstandingStats
```

## 2. ImplementationUnderstanding

### 2.1 Rust 定义

```rust
/// 单阶段结构化理解产物
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationUnderstanding {
    /// 阶段 ID（来自 StageContext）
    pub stage_id: String,
    /// 版本号，格式 "3.0.0"
    pub version: String,
    /// 阶段摘要（short + detailed）
    pub summary: StageSummary,
    /// 实现声明列表
    pub claims: Vec<ImplementationClaim>,
    /// 模块摘要
    pub module_summaries: Vec<ModuleSummary>,
    /// 信号摘要
    pub signal_summaries: Vec<SignalSummary>,
    /// 接口摘要
    pub interface_summaries: Vec<InterfaceSummary>,
    /// 处理步骤摘要
    pub processing_steps: Vec<ProcessingStepSummary>,
    /// 无法从 evidence 推断的项
    pub unknowns: Vec<UnknownItem>,
    /// 证据缺失项
    pub evidence_gaps: Vec<EvidenceGap>,
    /// 生成元信息
    pub generation_meta: GenerationMeta,
    /// 统计信息
    pub stats: UnderstandingStats,
}
```

### 2.2 TypeScript 定义

```typescript
interface ImplementationUnderstanding {
  stage_id: string;
  version: string;
  summary: StageSummary;
  claims: ImplementationClaim[];
  module_summaries: ModuleSummary[];
  signal_summaries: SignalSummary[];
  interface_summaries: InterfaceSummary[];
  processing_steps: ProcessingStepSummary[];
  unknowns: UnknownItem[];
  evidence_gaps: EvidenceGap[];
  generation_meta: GenerationMeta;
  stats: UnderstandingStats;
}
```

## 3. StageSummary

### 3.1 Rust 定义

```rust
/// 阶段摘要 — 分 short（一句话）和 detailed（详细描述）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageSummary {
    /// 一句话摘要，建议 ≤ 80 字
    pub short: String,
    /// 详细摘要，建议 ≤ 500 字
    pub detailed: String,
}
```

### 3.2 TypeScript 定义

```typescript
interface StageSummary {
  short: string;   // 一句话摘要，≤ 80 字
  detailed: string; // 详细摘要，≤ 500 字
}
```

## 4. ImplementationClaim

### 4.1 Rust 定义

```rust
/// 实现声明 — 描述阶段实现的某个方面
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationClaim {
    /// 声明唯一 ID，格式 "CL-<stage_id>-<6位序号>"
    pub claim_id: String,
    /// 声明类别
    pub category: ClaimCategory,
    /// 声明描述（中文）
    pub description: String,
    /// 置信度
    pub confidence: ClaimConfidence,
    /// 证据引用列表（至少一条，或标注 evidence_gap）
    pub evidence_refs: Vec<EvidenceRef>,
    /// 是否有 evidence gap
    pub has_evidence_gap: bool,
}
```

### 4.2 TypeScript 定义

```typescript
interface ImplementationClaim {
  claim_id: string;
  category: ClaimCategory;
  description: string;
  confidence: ClaimConfidence;
  evidence_refs: EvidenceRef[];
  has_evidence_gap: boolean;
}
```

## 5. ClaimConfidence

### 5.1 枚举定义

```rust
/// 声明置信度 — 与 EvidenceStrength 是不同层级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ClaimConfidence {
    /// 有充分证据直接支持（≥ 2 条 direct evidence，无矛盾）
    Confirmed,
    /// 有证据支撑，需辅助推断或上下文解释
    Supported,
    /// 有 indirect evidence 或仅单条 direct evidence 支持
    Inferred,
    /// evidence 不足或无法从 evidence 推断
    Unknown,
    /// evidence 之间存在矛盾
    Conflicting,
}
```

```typescript
type ClaimConfidence = 'confirmed' | 'supported' | 'inferred' | 'unknown' | 'conflicting';
```

### 5.2 ClaimConfidence vs EvidenceStrength

| 概念 | 字段名 | 枚举值 | 生成阶段 |
|------|--------|--------|----------|
| **evidence strength** | `EvidenceItem.strength` | `direct` / `indirect` / `weak` / `conflicting` / `missing` | Phase 2 静态提取 |
| **claim confidence** | `ImplementationClaim.confidence` | `confirmed` / `supported` / `inferred` / `unknown` / `conflicting` | Phase 3 语义理解 |

**两者关系**：
- evidence strength 描述单条证据的可靠性
- claim confidence 描述基于多条证据得出的结论的置信度
- confirmed claim 通常需要多条 direct evidence
- inferred claim 可能基于 indirect evidence 或少量 evidence
- **Phase 2 不生成 claim confidence**

### 5.3 confidence 语义

| 值 | 含义 | evidence 要求 |
|----|------|--------------|
| `confirmed` | 有充分证据直接支持 | ≥ 2 条 direct evidence，且无矛盾 |
| `supported` | 有证据支撑，需辅助推断 | ≥ 1 条 direct evidence + 辅助上下文，但不满足 confirmed |
| `inferred` | 有证据支持但不够充分 | ≥ 1 条 evidence（direct 或 indirect），但不满足 confirmed 或 supported |
| `unknown` | 无法从现有 evidence 推断 | evidence 不足或无法理解 |
| `conflicting` | evidence 之间存在矛盾 | ≥ 2 条 evidence 但互相矛盾 |

## 6. ClaimCategory

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ClaimCategory {
    /// 模块结构（module、class）
    ModuleStructure,
    /// 信号定义（wire、reg、port）
    SignalDefinition,
    /// 接口描述（输入/输出、接口协议）
    InterfaceDescription,
    /// 数据处理流程（算法、变换、流水线）
    DataProcessing,
    /// 配置与约束（时钟约束、综合参数）
    Configuration,
    /// 文档与注释（从文档中提取的实现描述）
    Documentation,
    /// 测试覆盖（测试用例、断言）
    TestCoverage,
    /// 其他
    Other,
}
```

```typescript
type ClaimCategory =
  | 'module_structure'
  | 'signal_definition'
  | 'interface_description'
  | 'data_processing'
  | 'configuration'
  | 'documentation'
  | 'test_coverage'
  | 'other';
```

## 7. EvidenceRef

```rust
/// 证据引用 — 通过 evidence_id 回链到 Phase 2 EvidenceCollection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRef {
    /// 引用的 evidence_id（必须存在于 EvidenceCollection 中）
    pub evidence_id: String,
    /// 引用相关性描述（可选）
    pub relevance: Option<String>,
}
```

```typescript
interface EvidenceRef {
  evidence_id: string;
  relevance?: string;
}
```

**关键约束**：
- `evidence_id` 必须在输入 `EvidenceCollection.evidence_items` 中真实存在
- `source_path` 和 `line_range` 不在 claim 中重复，通过 `evidence_id` 回链到 EvidenceItem 获取
- hallucination guard：输出 claim 中 `evidence_refs` 里的 `evidence_id` 必须通过 existence check

## 8. UnknownItem

```rust
/// 无法从现有 evidence 推断的信息项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownItem {
    /// 唯一 ID，格式 "UNK-<stage_id>-<6位序号>"
    pub unknown_id: String,
    /// 描述无法推断的内容
    pub description: String,
    /// 相关 evidence（可选，可能有部分证据但不足以推断）
    pub related_evidence_refs: Vec<EvidenceRef>,
    /// 原因说明
    pub reason: String,
}
```

```typescript
interface UnknownItem {
  unknown_id: string;
  description: string;
  related_evidence_refs: EvidenceRef[];
  reason: string;
}
```

**约束**：
- unknown 不允许绑定伪造 evidence_id
- `related_evidence_refs` 可以为空（完全无证据）
- `related_evidence_refs` 中的 evidence_id 也必须通过 existence check

## 9. EvidenceGap

```rust
/// 期望存在但缺失的证据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceGap {
    /// 唯一 ID，格式 "GAP-<stage_id>-<6位序号>"
    pub gap_id: String,
    /// 期望什么 evidence
    pub expected_evidence: String,
    /// 为什么期望这个 evidence
    pub reason: String,
    /// 相关的已有 evidence（可选）
    pub related_evidence_refs: Vec<EvidenceRef>,
}
```

```typescript
interface EvidenceGap {
  gap_id: string;
  expected_evidence: string;
  reason: string;
  related_evidence_refs: EvidenceRef[];
}
```

## 10. 摘要对象

### 10.1 ModuleSummary

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSummary {
    /// 模块名称
    pub name: String,
    /// 模块描述
    pub description: String,
    /// 证据引用
    pub evidence_refs: Vec<EvidenceRef>,
    /// 置信度
    pub confidence: ClaimConfidence,
}
```

```typescript
interface ModuleSummary {
  name: string;
  description: string;
  evidence_refs: EvidenceRef[];
  confidence: ClaimConfidence;
}
```

### 10.2 SignalSummary

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalSummary {
    /// 信号名称
    pub name: String,
    /// 信号描述
    pub description: String,
    /// 信号方向（input / output / internal）
    pub direction: Option<String>,
    /// 证据引用
    pub evidence_refs: Vec<EvidenceRef>,
    /// 置信度
    pub confidence: ClaimConfidence,
}
```

```typescript
interface SignalSummary {
  name: string;
  description: string;
  direction?: string;
  evidence_refs: EvidenceRef[];
  confidence: ClaimConfidence;
}
```

### 10.3 InterfaceSummary

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceSummary {
    /// 接口名称
    pub name: String,
    /// 接口描述
    pub description: String,
    /// 接口类型（port / bus / protocol / api）
    pub interface_type: Option<String>,
    /// 证据引用
    pub evidence_refs: Vec<EvidenceRef>,
    /// 置信度
    pub confidence: ClaimConfidence,
}
```

```typescript
interface InterfaceSummary {
  name: string;
  description: string;
  interface_type?: string;
  evidence_refs: EvidenceRef[];
  confidence: ClaimConfidence;
}
```

### 10.4 ProcessingStepSummary

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStepSummary {
    /// 步骤名称
    pub name: String,
    /// 步骤描述
    pub description: String,
    /// 步骤序号（用于排序）
    pub order: u32,
    /// 证据引用
    pub evidence_refs: Vec<EvidenceRef>,
    /// 置信度
    pub confidence: ClaimConfidence,
}
```

```typescript
interface ProcessingStepSummary {
  name: string;
  description: string;
  order: number;
  evidence_refs: EvidenceRef[];
  confidence: ClaimConfidence;
}
```

## 11. GenerationMeta

```rust
/// 生成元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMeta {
    /// 生成使用的 provider 类型
    pub provider: String,
    /// 生成时间戳（ISO 8601）
    pub generated_at: String,
    /// 输入的 EvidenceCollection 中 evidence_items 总数
    pub input_evidence_count: u32,
    /// 生成耗时（毫秒）
    pub generation_time_ms: u64,
    /// 是否为 degraded mode（无 LLM 时的降级模式）
    pub is_degraded: bool,
}
```

```typescript
interface GenerationMeta {
  provider: string;
  generated_at: string;
  input_evidence_count: number;
  generation_time_ms: number;
  is_degraded: boolean;
}
```

## 12. UnderstandingStats

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnderstandingStats {
    /// claim 总数
    pub total_claims: u32,
    /// 按 confidence 分组计数
    pub claims_by_confidence: std::collections::HashMap<String, u32>,
    /// 按 category 分组计数
    pub claims_by_category: std::collections::HashMap<String, u32>,
    /// 模块摘要数
    pub module_count: u32,
    /// 信号摘要数
    pub signal_count: u32,
    /// 接口摘要数
    pub interface_count: u32,
    /// 处理步骤数
    pub processing_step_count: u32,
    /// unknown 项数
    pub unknown_count: u32,
    /// evidence gap 项数
    pub evidence_gap_count: u32,
}
```

```typescript
interface UnderstandingStats {
  total_claims: number;
  claims_by_confidence: Record<string, number>;
  claims_by_category: Record<string, number>;
  module_count: number;
  signal_count: number;
  interface_count: number;
  processing_step_count: number;
  unknown_count: number;
  evidence_gap_count: number;
}
```

## 13. claim_id / unknown_id / gap_id 生成规则

| ID 类型 | 格式 | 说明 |
|---------|------|------|
| `claim_id` | `CL-<stage_id>-<6位序号>` | 如 `CL-L0-000001` |
| `unknown_id` | `UNK-<stage_id>-<6位序号>` | 如 `UNK-RTL-000001` |
| `gap_id` | `GAP-<stage_id>-<6位序号>` | 如 `GAP-L1-000001` |

生成器与 Phase 2 的 `EvidenceIdGenerator` 模式一致，各自独立的 counter。

## 14. 核心约束总结

1. **evidence_id 引用来自 Phase 2 EvidenceCollection**：claim / summary / unknown / gap 中的 evidence_refs 必须引用实际存在的 evidence_id
2. **source_path 和 line_range 通过 evidence_id 回链**：不在 claim 中重复存储，避免不一致
3. **每条用户可见主要 claim 必须有 evidence_refs 或明确 evidence_gap**：不允许无任何依据的 claim
4. **unknown 不允许绑定伪造 evidence_id**：related_evidence_refs 中的 evidence_id 也必须通过 existence check
5. **confidence 与 strength 是不同层级**：evidence strength 是 Phase 2 静态判定，claim confidence 是 Phase 3 语义判定
6. **confirmed / supported / inferred / unknown / conflicting 语义严格**：不使用"正确/错误"等审计用语

## 15. 版本字段与后续 Phase 消费关系

> **重要声明**：Phase 3 的 `ImplementationUnderstanding` 是**不含 `structure_view` / `dataflow_view` / `timing_view` 的中间产物**。[`mvp-functional-contract.md`](../requirements/mvp-functional-contract.md) 中定义的 `implementation_understanding.json`（含三类视图）是 Phase 4+ 扩展后的最终形态。Phase 4 将从 Phase 3 的 `module_summaries` / `interface_summaries` / `signal_summaries` / `processing_steps` / `claims` 生成视图结构。Phase 3 不做图视图。

```text
Phase 3: ImplementationUnderstanding (version "3.0.0")
  ↓ Phase 4 消费
    structure_view ← 从 module_summaries + interface_summaries 生成
    dataflow_view ← 从 processing_steps + signal_summaries 生成
    timing_view ← 从 processing_steps + claims 生成
  ↓ Phase 5 消费
    evidence trace ← 从 evidence_refs 回链到 EvidenceItem
    Q&A context ← 从 summary + claims + unknowns 构建
  ↓ Phase 6 消费
    持久化 ← ImplementationUnderstanding 整体序列化
```

**版本规则**：
- Phase 3 定义 version = "3.0.0"
- Phase 4 扩展 visualization spec 时更新 minor version
- 字段增加不破坏向后兼容

## 16. 与 Phase 2 EvidenceCollection 的输入关系

```text
Phase 2: EvidenceCollection
  - stage_id → ImplementationUnderstanding.stage_id
  - evidence_items[] → summary.short / summary.detailed（由生成器从 evidence 提炼）
  - evidence_items[] → claims[].evidence_refs[].evidence_id
  - evidence_items[] → module_summaries[].evidence_refs[].evidence_id
  - evidence_items[] → signal_summaries[].evidence_refs[].evidence_id
  - evidence_items[] → interface_summaries[].evidence_refs[].evidence_id
  - evidence_items[] → processing_steps[].evidence_refs[].evidence_id
  - evidence_items[] → unknowns[].related_evidence_refs[].evidence_id
  - evidence_items[] → evidence_gaps[].related_evidence_refs[].evidence_id
  - stats → generation_meta.input_evidence_count
```

## 17. 文档变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-12 | 收口修复：ClaimConfidence 补齐 supported（5→5 与 mvp-contract 对齐）；summary 改为 StageSummary { short, detailed }；§15 加中间产物显式声明；status draft → active | Claude |
| 2026-06-12 | 初始创建（draft） | Claude |
