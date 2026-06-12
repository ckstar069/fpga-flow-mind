# Phase 3 理解生成器后端设计

---
status: draft
updated: 2026-06-12
---

> 本文档定义 Phase 3 后端理解生成流程，包含确定性预打包层（context builder）和 agent/LLM 生成层（generator），以及 schema 验证和 hallucination 防护。
>
> **Phase 3 不编码**。本文档是 draft，编码前需审核收口。

## 1. 整体架构

```text
输入: EvidenceCollection (Phase 2 产出)
  │
  ▼
┌─────────────────────────────────┐
│  1. ContextBuilder              │  ← 确定性预打包
│     组装 LLM 输入上下文         │
└─────────────────────────────────┘
  │
  ▼
┌─────────────────────────────────┐
│  2. Generator (Provider)        │  ← agent/LLM 生成
│     调用 LLM 或 mock provider   │
└─────────────────────────────────┘
  │
  ▼
┌─────────────────────────────────┐
│  3. SchemaValidator             │  ← 输出验证
│     JSON schema + evidence_id   │
│     existence check             │
└─────────────────────────────────┘
  │
  ▼
输出: ImplementationUnderstanding
```

两层设计：
- **确定性层**：ContextBuilder 将 EvidenceCollection 转换为结构化的 LLM 输入
- **语义层**：Generator（通过 Provider trait）执行实际的语义理解

## 2. 模块布局

```text
src-tauri/src/
├── understanding/
│   ├── mod.rs                    ← 模块入口，re-export
│   ├── models.rs                 ← ImplementationUnderstanding 等数据结构
│   ├── context_builder.rs        ← EvidenceCollection → LLM 输入上下文
│   ├── schema_validator.rs       ← 输出 JSON schema 验证 + evidence_id check
│   └── generator.rs              ← 理解生成主流程 + Provider trait
├── commands/
│   └── generate_understanding.rs ← Tauri command
└── lib.rs                        ← 注册 command
```

## 3. ContextBuilder

### 3.1 职责

将 `EvidenceCollection` 转换为 LLM 可消费的结构化输入。这一层是**完全确定性**的，不涉及 LLM 调用。

### 3.2 输入

```rust
pub struct GeneratorInput {
    /// 阶段 ID
    pub stage_id: String,
    /// 所有 evidence items（按原始顺序）
    pub evidence_items: Vec<EvidenceItem>,
    /// 按文件分组的索引
    pub index_by_path: HashMap<String, Vec<String>>,
    /// 按类型分组的索引
    pub index_by_kind: HashMap<String, Vec<String>>,
    /// 按符号分组的索引
    pub index_by_symbol: HashMap<String, Vec<String>>,
    /// 所有 evidence_id 集合（用于 existence check）
    pub known_evidence_ids: HashSet<String>,
}
```

### 3.3 输出

```rust
pub struct GeneratorOutput {
    /// Prompt（含 system prompt + user prompt）
    pub prompt: String,
    /// JSON schema（约束 LLM 输出格式）
    pub output_schema: serde_json::Value,
    /// 已知的 evidence_id 集合（传给 validator）
    pub known_evidence_ids: HashSet<String>,
}
```

### 3.4 ContextBuilder 实现

```rust
pub struct ContextBuilder;

impl ContextBuilder {
    /// 从 EvidenceCollection 构建 LLM 输入
    pub fn build(collection: &EvidenceCollection) -> GeneratorOutput {
        let known_ids: HashSet<String> = collection
            .evidence_items
            .iter()
            .map(|item| item.evidence_id.clone())
            .collect();

        let prompt = Self::build_prompt(collection);
        let schema = Self::build_output_schema();

        GeneratorOutput {
            prompt,
            output_schema: schema,
            known_evidence_ids: known_ids,
        }
    }

    fn build_prompt(collection: &EvidenceCollection) -> String {
        // 1. system prompt：角色定义、输出格式要求、约束规则
        // 2. user prompt：evidence items 结构化列表
        // 3. 输出 JSON schema 约束
        todo!() // 编码时实现
    }

    fn build_output_schema() -> serde_json::Value {
        // 返回 ImplementationUnderstanding 的 JSON schema
        todo!() // 编码时实现
    }
}
```

### 3.5 prompt 结构

```text
=== System Prompt ===
你是一个 FPGA 实现理解助手。你的任务是基于提供的 evidence 生成结构化理解。

约束：
1. 每条 claim 必须引用 evidence_id
2. evidence_id 必须在提供的 evidence 列表中真实存在
3. 无法推断的内容标注为 unknown
4. 缺失的 evidence 标注为 evidence_gap
5. 不使用"正确/错误"、"PASS/HOLD"等审计用语
6. confidence 语义：confirmed（充分证据）、inferred（有限证据）、unknown（证据不足）、conflicting（证据矛盾）

=== User Prompt ===
阶段 ID: {stage_id}
证据总数: {count}

{每个 evidence item 的结构化摘要}

=== 输出格式 ===
{JSON schema}
```

## 4. Provider Trait

### 4.1 定义

```rust
/// 理解生成 Provider 抽象
pub trait UnderstandingProvider: Send + Sync {
    /// 调用生成
    fn generate(&self, input: &GeneratorOutput) -> Result<serde_json::Value, ProviderError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("LLM 调用失败: {0}")]
    LlmCallFailed(String),
    #[error("LLM 返回格式错误: {0}")]
    InvalidFormat(String),
    #[error("LLM 超时")]
    Timeout,
    #[error("Provider 未配置")]
    NotConfigured,
}
```

### 4.2 Mock Provider

```rust
/// Mock provider — 用于测试和开发
pub struct MockProvider {
    /// 预设的返回值
    response: serde_json::Value,
}

impl UnderstandingProvider for MockProvider {
    fn generate(&self, _input: &GeneratorOutput) -> Result<serde_json::Value, ProviderError> {
        Ok(self.response.clone())
    }
}
```

### 4.3 Manual Provider

```rust
/// Manual provider — 手动输入（开发调试用）
/// 用户通过前端 UI 手动编辑 JSON
pub struct ManualProvider;

impl UnderstandingProvider for ManualProvider {
    fn generate(&self, _input: &GeneratorOutput) -> Result<serde_json::Value, ProviderError> {
        Err(ProviderError::NotConfigured)
    }
}
```

### 4.4 设计决策：Provider 选择

Phase 3 编码阶段决定具体使用哪个 provider。当前设计约束：

| Provider | 状态 | 说明 |
|----------|------|------|
| MockProvider | 开发/测试 | 预设返回值，验证 pipeline |
| ManualProvider | 开发调试 | 手动输入 JSON |
| LLM Provider | 未来 | 实际调用 LLM API，Phase 3 不实现 |

**关键约束**：Phase 3 规划文档阶段不引入任何新依赖。

## 5. SchemaValidator

### 5.1 职责

对 generator 输出进行两层验证：

1. **JSON Schema 验证**：确保输出符合 ImplementationUnderstanding schema
2. **evidence_id existence check**：确保所有 evidence_refs 中的 evidence_id 在输入 EvidenceCollection 中存在

### 5.2 验证流程

```text
Generator 输出 (JSON)
  │
  ▼
┌─────────────────────────┐
│ 1. JSON Schema 验证     │
│    - 字段完整性         │
│    - 类型正确性         │
│    - 枚举值合法性       │
└─────────────────────────┘
  │ pass
  ▼
┌─────────────────────────┐
│ 2. evidence_id 检查     │
│    - 收集所有           │
│      evidence_refs中的  │
│      evidence_id        │
│    - 与 known_ids 比对  │
│    - 拒绝不存在的 ID   │
└─────────────────────────┘
  │ pass
  ▼
┌─────────────────────────┐
│ 3. 业务规则检查         │
│    - claim 数量 > 0     │
│    - 无 refs 的 claim   │
│      必须 has_gap=true  │
│    - unknown 无伪造 ID  │
└─────────────────────────┘
  │ pass
  ▼
反序列化 → ImplementationUnderstanding
```

### 5.3 ValidationResult

```rust
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

pub enum ValidationError {
    /// JSON schema 验证失败
    SchemaViolation { path: String, message: String },
    /// evidence_id 不存在
    UnknownEvidenceId { evidence_id: String, location: String },
    /// claim 无 evidence_refs 且无 evidence_gap
    ClaimWithoutEvidence { claim_id: String },
    /// unknown 绑定了不存在的 evidence_id
    UnknownWithFakeEvidence { unknown_id: String, evidence_id: String },
}

pub enum ValidationWarning {
    /// unknown 项数量过多（超过 claim 数量）
    TooManyUnknowns { count: usize, claim_count: usize },
    /// evidence gap 数量过多
    TooManyGaps { count: usize },
}
```

### 5.4 hallucination guard

hallucination guard 是 schema validator 的一部分，核心机制：

1. **收集 evidence_id**：遍历输出中所有 `evidence_refs` 和 `related_evidence_refs`
2. **existence check**：与 ContextBuilder 传入的 `known_evidence_ids` 逐一比对
3. **拒绝策略**：发现不存在的 evidence_id → `ValidationError::UnknownEvidenceId` → 验证失败
4. **不自动修复**：验证失败直接返回错误，不尝试修正

## 6. Generator 主流程

### 6.1 职责

编排 ContextBuilder → Provider → SchemaValidator 的完整流程。

### 6.2 流程

```rust
pub struct UnderstandingGenerator {
    provider: Box<dyn UnderstandingProvider>,
}

impl UnderstandingGenerator {
    pub fn new(provider: Box<dyn UnderstandingProvider>) -> Self {
        Self { provider }
    }

    /// 从 EvidenceCollection 生成 ImplementationUnderstanding
    pub fn generate(
        &self,
        collection: &EvidenceCollection,
    ) -> Result<ImplementationUnderstanding, GeneratorError> {
        // 1. 确定性预打包
        let generator_input = ContextBuilder::build(collection);

        // 2. 调用 provider
        let raw_output = self.provider.generate(&generator_input)
            .map_err(GeneratorError::ProviderError)?;

        // 3. Schema 验证
        let validation = SchemaValidator::validate(
            &raw_output,
            &generator_input.known_evidence_ids,
        );

        if !validation.is_valid {
            return Err(GeneratorError::ValidationFailed(validation.errors));
        }

        // 4. 反序列化
        let understanding: ImplementationUnderstanding =
            serde_json::from_value(raw_output)
                .map_err(GeneratorError::DeserializationError)?;

        Ok(understanding)
    }
}
```

### 6.3 错误处理

```rust
#[derive(Debug, thiserror::Error)]
pub enum GeneratorError {
    #[error("Provider 错误: {0}")]
    ProviderError(ProviderError),
    #[error("验证失败: {0:?}")]
    ValidationFailed(Vec<ValidationError>),
    #[error("反序列化失败: {0}")]
    DeserializationError(#[from] serde_json::Error),
}
```

## 7. Tauri Command

### 7.1 generate_understanding

```rust
#[tauri::command]
pub fn generate_understanding(
    root_path: String,
    stage_id: String,
) -> CommandResult<ImplementationUnderstanding> {
    // 1. 获取 EvidenceCollection（调用 Phase 2 collect 逻辑）
    // 2. 构造 provider（根据配置选择 mock/LLM）
    // 3. 调用 generator
    // 4. 返回 CommandResult
}
```

### 7.2 与 Phase 2 的集成方式

两种方案（编码时决定）：

**方案 A**：command 内部先调用 collect 再 generate
```text
generate_understanding(root_path, stage_id)
  → resolve_stage_context
  → EvidenceCollector::collect
  → UnderstandingGenerator::generate
```

**方案 B**：前端先 collect，再 generate
```text
前端: collectEvidence(root_path, stage_id) → EvidenceCollection
前端: generateUnderstanding(evidenceCollection) → ImplementationUnderstanding
```

**推荐方案 B**：
- 前端控制粒度更细
- EvidenceCollection 可以缓存和复用
- 前端可以在 generate 前展示 evidence 给用户确认

## 8. 失败处理策略

| 失败类型 | 处理方式 |
|----------|----------|
| Provider 超时 | 返回 timeout error，前端提示重试 |
| Provider 未配置 | 返回 degraded 标志，前端展示"理解不可用" |
| Schema 验证失败 | 返回 validation errors，不降级 |
| evidence_id 不存在 | 返回 validation error（hallucination detected） |
| 反序列化失败 | 返回 internal error |
| EvidenceCollection 为空 | 返回成功但 ImplementationUnderstanding 为空（仅有 stats） |

## 9. Degraded Mode

当 LLM provider 不可用时，系统可以进入 degraded mode：

- `generation_meta.is_degraded = true`
- 生成一个基于规则的最小理解产物：
  - 从 evidence items 直接提取模块/信号/接口名称
  - 所有 claim 标注为 `unknown` confidence
  - 无 summary
  - 无 processing_steps

这确保即使没有 LLM，用户也能看到基本的证据结构。

## 10. 性能考量

| 关注点 | 策略 |
|--------|------|
| 大量 evidence items | prompt 中只包含精简摘要，不包含完整 excerpt |
| LLM 调用延迟 | 前端展示 loading 状态，后端不阻塞 UI |
| 生成超时 | 默认 30s 超时，可配置 |
| 内存占用 | ImplementationUnderstanding 是纯数据对象，无大内存风险 |

## 11. 安全约束

Phase 3 后端代码遵循与 Phase 2 相同的安全约束：

- **不使用** `std::fs::write`、`std::fs::create_dir`、`std::fs::remove_file`、`std::fs::rename`、`std::fs::copy`
- **不使用** `std::process::Command` 或 `Command::new`
- **不调用** Vivado / synthesis / implementation / bitstream
- **不调用** 目标项目中的脚本
- **不把** 输出写回目标项目目录
- **不调用** 外部 LLM API（Phase 3 编码阶段使用 mock provider）

## 12. 文档变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-12 | 初始创建（draft） | Claude |
