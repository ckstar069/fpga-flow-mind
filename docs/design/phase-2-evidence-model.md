# Phase 2 Evidence Model 数据结构设计

---
status: active
updated: 2026-06-11
---

> 本文档定义 Phase 2 evidence model 的数据结构，包括 `EvidenceItem`、`EvidenceCollection`、`EvidenceId` 生成规则、`line_range` 规则、`source excerpt` 规则、strength 语义和错误结构。
> 不写产品代码。数据结构与 [`mvp-functional-contract.md`](../requirements/mvp-functional-contract.md) 的 `evidence_index.json` 对齐。
> **术语区分**：`EvidenceItem.strength` = evidence 层证据强度（direct/indirect/weak/conflicting/missing），与 Phase 3+ 的 claim confidence（confirmed/inferred/unknown/conflicting）是不同层的概念。Phase 2 不生成 claim，不产生 confirmed/inferred/unknown 结论。

## 1. 设计目标

将 [`mvp-functional-contract.md`](../requirements/mvp-functional-contract.md) 中定义的 `evidence_index.json` 落到 Rust/TypeScript 的具体数据结构，明确每个字段的含义、约束、来源和默认值。

### 术语约定：evidence strength vs claim confidence

| 概念 | 层级 | 枚举值 | 何时使用 |
|------|------|--------|----------|
| **evidence strength** (`EvidenceItem.strength`) | Evidence 层 | `direct` / `indirect` / `weak` / `conflicting` / `missing` | Phase 2 静态提取时标注 |
| **claim confidence** | 语义结论层（Phase 3+） | `confirmed` / `inferred` / `unknown` / `conflicting` | Phase 3 大模型生成结论时标注 |

**Phase 2 只涉及 evidence strength，不生成 claim confidence。** 用户在 Phase 2 evidence 面板看到的是 `strength` 标签，不是 claim confidence。

## 2. EvidenceItem 数据结构

### 2.1 Rust struct 草案

```rust
/// 单条证据项
struct EvidenceItem {
    /// 全局唯一标识，格式 "EV-<stage_id>-<6位序号>"
    /// 示例："EV-L0-000001"、"EV-RTL-000003"
    evidence_id: String,

    /// 源码文件绝对路径
    source_path: String,

    /// 语言，继承自 Phase 1 file_classifier
    language: Language,

    /// 来源类型，继承自 Phase 1 file_classifier
    source_kind: SourceKind,

    /// 行号范围（1-based，闭区间）
    line_range: LineRange,

    /// 符号名称（函数名/类名/module 名/信号名等）
    /// None 表示整文件级证据或无法确定符号
    symbol: Option<String>,

    /// 代码片段或描述
    /// 最大长度 500 字符，超出截断并追加 "..."
    summary: String,

    /// 证据强度（evidence strength）
    /// 与 mvp-functional-contract.md 的 evidence_strength 对齐
    /// Phase 2 只生成 direct / indirect
    /// 完整枚举保留 weak / conflicting / missing 供后续阶段使用
    strength: EvidenceStrength,
}

/// 行号范围（1-based，闭区间）
struct LineRange {
    /// 起始行号，>= 1
    start: u32,
    /// 结束行号，>= start
    end: u32,
}
```

### 2.2 TypeScript interface 草案

```typescript
interface EvidenceItem {
  /** 全局唯一标识，格式 "EV-<stage_id>-<6位序号>" */
  evidence_id: string;

  /** 源码文件绝对路径 */
  source_path: string;

  /** 语言，继承自 Phase 1 */
  language: Language;

  /** 来源类型，继承自 Phase 1 */
  source_kind: SourceKind;

  /** 行号范围（1-based，闭区间） */
  line_range: LineRange;

  /** 符号名称，可选 */
  symbol?: string;

  /** 代码片段或描述，最大 500 字符 */
  summary: string;

  /** 证据强度（evidence strength），不是 claim confidence */
  strength: EvidenceStrength;
}

interface LineRange {
  /** 起始行号，>= 1 */
  start: number;
  /** 结束行号，>= start */
  end: number;
}
```

### 2.3 字段说明

| 字段 | 必填 | 类型 | 来源 | 约束 |
|------|------|------|------|------|
| `evidence_id` | 是 | `string` | Phase 2 生成 | 格式 `EV-<stage_id>-<6位序号>`，全局唯一 |
| `source_path` | 是 | `string` | Phase 1 `StageFile.source_path` | 绝对路径 |
| `language` | 是 | `Language` | Phase 1 `StageFile.language` | 枚举值 |
| `source_kind` | 是 | `SourceKind` | Phase 1 `StageFile.source_kind` | 枚举值 |
| `line_range` | 是 | `LineRange` | Phase 2 提取计算 | `start >= 1`，`start <= end` |
| `symbol` | 否 | `string` | Phase 2 提取 | 无 symbol 时为 `null`/`undefined` |
| `summary` | 是 | `string` | Phase 2 提取 | 最大 500 字符，超出截断 |
| `strength` | 是 | `EvidenceStrength` | Phase 2 判定 | Phase 2 只生成 `direct` / `indirect` |

## 3. EvidenceCollection 数据结构

`EvidenceCollection` 对应 `mvp-functional-contract.md` 的 `evidence_index.json`。

### 3.1 Rust struct 草案

```rust
/// 证据集合（单阶段）
struct EvidenceCollection {
    /// 阶段标识
    stage_id: String,

    /// 证据项列表
    evidence_items: Vec<EvidenceItem>,

    /// 按文件路径分组索引
    /// key = source_path，value = evidence_id[]
    index_by_path: HashMap<String, Vec<String>>,

    /// 按来源类型分组索引
    /// key = source_kind（snake_case 字符串），value = evidence_id[]
    index_by_kind: HashMap<String, Vec<String>>,

    /// 按符号名称反向索引
    /// key = symbol，value = evidence_id[]
    /// 仅包含 symbol 非 None 的 item
    index_by_symbol: HashMap<String, Vec<String>>,

    /// 收集过程中的非致命警告
    warnings: Vec<EvidenceWarning>,

    /// 收集统计
    stats: EvidenceStats,

    /// 产物格式版本
    version: String,  // "1.0.0"
}

/// 证据收集警告
struct EvidenceWarning {
    error_code: EvidenceErrorCode,
    message: String,
    source_path: Option<String>,
}

/// 证据收集统计
struct EvidenceStats {
    /// 处理的文件总数
    files_processed: u32,
    /// 跳过的文件数（二进制、不可读等）
    files_skipped: u32,
    /// evidence item 总数
    total_items: u32,
    /// 按 source_kind 分组的 item 计数
    items_by_kind: HashMap<String, u32>,
    /// 按 strength 分组的 item 计数
    items_by_strength: HashMap<String, u32>,
}
```

### 3.2 TypeScript interface 草案

```typescript
interface EvidenceCollection {
  stage_id: string;
  evidence_items: EvidenceItem[];
  index_by_path: Record<string, string[]>;
  index_by_kind: Record<string, string[]>;
  index_by_symbol: Record<string, string[]>;
  warnings: EvidenceWarning[];
  stats: EvidenceStats;
  version: string;
}

interface EvidenceWarning {
  error_code: EvidenceErrorCode;
  message: string;
  source_path?: string;
}

interface EvidenceStats {
  files_processed: number;
  files_skipped: number;
  total_items: number;
  items_by_kind: Record<string, number>;
  items_by_strength: Record<string, number>;
}
```

### 3.3 索引字段说明

| 索引 | 必填 | Key | Value | 说明 |
|------|------|-----|-------|------|
| `index_by_path` | 是 | `source_path` | `evidence_id[]` | 按文件分组，每个文件至少 0 条 evidence |
| `index_by_kind` | 是 | `source_kind` | `evidence_id[]` | 按类型分组，覆盖所有 item |
| `index_by_symbol` | 是 | `symbol` | `evidence_id[]` | 按 symbol 分组，仅包含有 symbol 的 item；无 symbol 的 item 不建立条目 |

## 4. EvidenceId 生成规则

| 规则 | 说明 |
|------|------|
| 格式 | `EV-<stage_id>-<6位序号>` |
| stage_id | 与 `StageContext.stage_id` 一致，如 `L0`、`L1`、`RTL` |
| 序号 | 6 位十进制数字，左补零，从 `000001` 开始 |
| 唯一性 | 同一次 `collect_evidence` 调用内，序号递增保证唯一 |
| 示例 | `EV-L0-000001`、`EV-L0-000002`、`EV-RTL-000001` |

**设计理由**：
- 不使用 UUID，因为 Phase 2 不涉及持久化和跨会话唯一性
- 序号可读性好，方便调试和前端展示
- 6 位数字支持单阶段最多 999999 条 evidence，远超实际需求

## 5. line_range 规则

| 规则 | 说明 |
|------|------|
| 计数方式 | 1-based（与编辑器行号一致） |
| 闭区间 | `start` 和 `end` 都是包含的 |
| 单行 | `start == end`（如单行 `assign` 语句） |
| 多行 | `start < end`（如多行 `module` 定义、`def` 函数体） |
| 整文件 | `start = 1`，`end = 文件总行数`（用于文件级证据） |
| 约束 | `start >= 1`，`start <= end`，`end <= 文件总行数` |

**Phase 2 行范围确定策略**：
- **函数/类定义**（Python）：从 `def`/`class` 行开始，到下一个同级 `def`/`class` 或文件结束
- **module 定义**（Verilog）：从 `module` 行开始，到 `endmodule` 行
- **文档章节**（Markdown）：从 `#` 标题行开始，到下一个同级/上级标题前一行
- **最小启发式**：不精确的行范围标记为 `indirect` strength

## 6. source excerpt 规则

### 6.1 是否保存源码片段

**是**。每条 `EvidenceItem.summary` 保存对应 `line_range` 的源码片段摘要。

### 6.2 最大长度

| 参数 | 值 | 说明 |
|------|-----|------|
| `summary` 最大长度 | 500 字符 | 超出截断，追加 `"...(已截断)"` |
| `line_range` 对应源码最大读取行数 | 100 行 | 超过 100 行的 block 只读前 100 行 |

### 6.3 如何避免大段复制

- 截断规则：源码片段超过 500 字符时，保留前 400 字符 + `"...(已截断，共 N 行)"`
- 整文件证据：只保存前 200 字符 + `"...(共 N 行)"`
- 不保存原始文件内容：`summary` 是摘要，不是文件全文

### 6.4 非 UTF-8 处理

- 非 UTF-8 文件**不生成 EvidenceItem**，而是通过 `EvidenceCollection.warnings[]` 表达（`non_utf8_file_skipped`）
- 该文件的跳过计入 `EvidenceStats.files_skipped`
- 不使用 `strength` 来表达解析失败——`strength` 只描述成功提取的证据的强度

## 7. evidence strength 语义

### 7.1 EvidenceStrength 枚举

与 [`mvp-functional-contract.md`](../requirements/mvp-functional-contract.md) 的 `evidence_strength` 枚举完全对齐：

| 值 | 含义 | Phase 2 生成条件 |
|----|------|-----------------|
| `direct` | 直接源码证据 | 正则/行级匹配提取（如 `def` 关键字、`module` 关键字） |
| `indirect` | 间接证据 | 启发式推断（如基于缩进推断函数边界、章节范围推断） |
| `weak` | 弱证据 | Phase 2 **不生成**，留给 Phase 3+ |
| `conflicting` | 与其他证据矛盾 | Phase 2 **不生成**，留给 Phase 3+ |
| `missing` | 证据缺失 | Phase 2 **不生成**，留给 Phase 3+ |

> **注意**：`unknown` **不是** `EvidenceStrength` 的值。解析失败/文件不可读等场景通过 `EvidenceCollection.warnings[]` 和 `EvidenceStats.files_skipped` 表达，不产生 EvidenceItem。

### 7.2 evidence strength vs claim confidence

| 概念 | 字段名 | 枚举值 | 生成阶段 |
|------|--------|--------|----------|
| **evidence strength** | `EvidenceItem.strength` | `direct` / `indirect` / `weak` / `conflicting` / `missing` | Phase 2 静态提取 |
| **claim confidence** | Phase 3+ 语义结论层的字段（如 `node_confidence`） | `confirmed` / `inferred` / `unknown` / `conflicting` | Phase 3+ 大模型生成 |

**Phase 2 不生成 claim confidence。** Phase 2 evidence 面板展示 `strength` 标签，不展示 claim confidence。

### 7.3 设计理由

Phase 2 是静态提取阶段，只产生 `direct`（确定匹配）和 `indirect`（启发式推断）两种 strength。`weak`、`conflicting`、`missing` 需要语义层面的判断（大模型或跨 evidence 对比），属于 Phase 3+ 的职责。解析失败不作为 strength 值，而是通过 warnings 表达。

## 8. error / warning 结构

### 8.1 Phase 2 错误码

| 错误码 | 场景 | success | 严重性 |
|--------|------|---------|--------|
| `file_unreadable` | 单个文件不可读 | true | warning |
| `file_too_large` | 文件超过 5MB | true | warning |
| `source_excerpt_truncated` | 摘要被截断 | true | warning |
| `evidence_collection_failed` | 整体收集失败（如阶段目录不可读） | false | error |
| `binary_file_skipped` | 二进制文件被跳过 | true | warning |
| `non_utf8_file_skipped` | 非 UTF-8 文件被跳过 | true | warning |

> 复用 Phase 1 的 `file_unreadable`、`file_too_large`。新增 `source_excerpt_truncated`、`evidence_collection_failed`、`binary_file_skipped`、`non_utf8_file_skipped`。

### 8.2 CommandResult 语义

| 场景 | `success` | `data` | `error` | `warnings` |
|------|-----------|--------|---------|------------|
| 正常收集 | `true` | `Some(EvidenceCollection)` | `None` | 非致命问题列表 |
| 空结果（所有文件跳过但仍可返回） | `true` | `Some(EvidenceCollection)`（`evidence_items=[]`） | `None` | 跳过原因列表 |
| 阶段不存在 | `false` | `None` | `Some(CommandError)` | 空 |
| 路径校验失败 | `false` | `None` | `Some(CommandError)` | 空 |

## 9. Rust 枚举扩展草案

Phase 2 需要扩展 Phase 1 的 `ErrorCode` 枚举：

```rust
// 在现有 ErrorCode 枚举中追加 Phase 2 错误码
enum ErrorCode {
    // Phase 1 已有（保留）
    PathNotFound,
    NotDirectory,
    PermissionDenied,
    NoStageFound,
    StageEmpty,
    StageUnreadable,
    FileUnreadable,
    FileTooLarge,
    ScanTimeout,

    // Phase 2 新增
    EvidenceCollectionFailed,   // 整体收集失败
    SourceExcerptTruncated,     // 摘要截断（warning 级别）
    BinaryFileSkipped,          // 二进制文件跳过（warning 级别）
    NonUtf8FileSkipped,         // 非 UTF-8 文件跳过（warning 级别）
}

/// 证据强度枚举（完整定义，Phase 2 只使用部分值）
/// 注意：不含 Unknown — 解析失败通过 warnings[] 表达，不产生 EvidenceItem
enum EvidenceStrength {
    Direct,      // 直接源码证据
    Indirect,    // 间接证据（启发式推断）
    Weak,        // 弱证据（Phase 3+ 使用）
    Conflicting, // 矛盾证据（Phase 3+ 使用）
    Missing,     // 证据缺失（Phase 3+ 使用）
}
```

## 10. TypeScript 枚举扩展草案

```typescript
// 扩展 ErrorCode
type ErrorCode =
  // Phase 1 已有
  | 'path_not_found' | 'not_directory' | 'permission_denied'
  | 'no_stage_found' | 'stage_empty' | 'stage_unreadable'
  | 'file_unreadable' | 'file_too_large' | 'scan_timeout'
  // Phase 2 新增
  | 'evidence_collection_failed'
  | 'source_excerpt_truncated'
  | 'binary_file_skipped'
  | 'non_utf8_file_skipped';

// 证据强度枚举
// 注意：不含 'unknown' — 解析失败通过 warnings[] 表达，不产生 EvidenceItem
type EvidenceStrength = 'direct' | 'indirect'
  | 'weak' | 'conflicting' | 'missing';
```

## 11. 与 Phase 1 StageContext 的输入关系

```text
Phase 1: select_stage(root_path, stage_id) → StageContext
  - stage_id: String
  - source_path: String
  - files: [{ source_path, language, source_kind, size_bytes }]
  - external_deps: String[]
  - upstream_refs: [{ stage_id, interface_file_path, inferred }]
  - error_code: Option<ErrorCode>

Phase 2: collect_evidence(root_path, stage_id) → EvidenceCollection
  输入：root_path + stage_id（内部复用 select_stage 获取 StageContext）
  输出：EvidenceCollection
    - stage_id ← StageContext.stage_id
    - evidence_items[].source_path ← StageContext.files[].source_path
    - evidence_items[].language ← StageContext.files[].language
    - evidence_items[].source_kind ← StageContext.files[].source_kind
    - evidence_items[].evidence_id ← Phase 2 新生成
    - evidence_items[].line_range ← Phase 2 提取计算
    - evidence_items[].symbol ← Phase 2 提取
    - evidence_items[].summary ← Phase 2 提取
    - evidence_items[].strength ← Phase 2 判定
    - index_by_path / index_by_kind / index_by_symbol ← Phase 2 索引构建
    - warnings ← Phase 2 收集过程中的非致命问题
    - stats ← Phase 2 统计
```

## 12. 与后续 Phase 3 ImplementationUnderstanding 的输出关系

```text
Phase 2 产出: EvidenceCollection
  ↓ 消费者
Phase 3: generate_understanding(evidence_collection) → ImplementationUnderstanding
  - structure_view.nodes[].evidence_refs[] ← 引用 evidence_id
  - structure_view.edges[].evidence_refs[] ← 引用 evidence_id
  - dataflow_view.nodes[].evidence_refs[] ← 引用 evidence_id
  - timing_view.nodes[].evidence_refs[] ← 引用 evidence_id
  - concepts[].evidence_refs[] ← 引用 evidence_id
  - signals[].evidence_refs[] ← 引用 evidence_id
  - uncertainties[].related_evidence_refs[] ← 引用 evidence_id
```

**契约**：Phase 3 通过 `evidence_id` 引用 Phase 2 的证据。Phase 2 必须保证 `evidence_id` 在一次收集内全局唯一。

## 13. 不做的事情

- **不做 AST 复杂语义**：Phase 2 的 Python 提取只做 `def`/`class` 关键字匹配 + 缩进推断函数边界，不做完整语法树
- **不做 LLM 判断**：strength 基于提取方式，不是语义判断
- **不做正确/错误结论**：Phase 2 不判断代码逻辑是否正确

## 14. 文档变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-11 | 初始创建 | Claude |
