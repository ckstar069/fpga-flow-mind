# Phase 2 证据索引与 Evidence Model 需求

---
status: active
updated: 2026-06-11
---

> 本文档定义 Phase 2（证据索引与 evidence model）的需求范围、功能点拆解和验收标准。
> Phase 2 基于 Phase 1 产出的 `StageContext`，为后续结构化理解、图视图和 grounded Q&A 建立可追溯证据基础设施。
> 不写产品代码，不实现 evidence/graph/LLM/Q&A。

## 1. Phase 2 目标

为 `fpga-flow-mind` 建立源码证据收集、索引和展示的基础能力。用户选择一个阶段后，系统能够：

- 从阶段目录中的源码、测试、文档、配置文件中提取结构化 evidence item
- 为每个 evidence item 分配唯一 ID、记录源文件路径和行号范围
- 建立按文件、按类型、按符号的交叉索引
- 在前端展示 evidence 摘要列表
- 确保证据收集过程不修改目标项目

## 2. Phase 2 用户价值

| 用户价值 | 说明 |
|---------|------|
| 可追溯证据基础 | 后续所有语义结论（Phase 3+）都能追溯到源码文件和行号 |
| 透明性 | 用户能看到系统收集了哪些证据、跳过了哪些、哪些标注为不确定 |
| 安全感 | 证据收集过程不触碰目标项目文件 |
| 可检查性 | 用户可以在进入 Phase 3 前审查证据质量 |

## 3. Phase 2 做什么 / 不做什么

### 3.1 做什么

- 从 `StageContext.files[]` 中读取文件内容
- 按语言类型（Python / Verilog / SystemVerilog / Markdown / Config）提取 evidence item
- 为每个 evidence item 生成唯一 `evidence_id`
- 记录 `source_path`、`line_range`、`language`、`source_kind`
- 标注 strength：`direct` / `indirect`
- 建立按文件路径、按 source_kind、按 symbol 的索引
- 前端展示 evidence 面板（摘要列表、分组统计、空状态、错误状态）
- 错误和不确定项的显式表达

### 3.2 不做什么

- **不做 AST 复杂语义分析**：Phase 2 只做行级/文件级/最小启发式证据提取，不做完整语法树
- **不做 LLM / 大模型调用**：Phase 2 不调用任何大模型 API
- **不做正确/错误判断**：Phase 2 不判断代码是否正确，只提取事实性证据
- **不做结构图/数据流图/时序图**：那是 Phase 4 的范围
- **不做 Q&A / 追问**：那是 Phase 5 的范围
- **不做持久化**：Phase 2 证据保留在内存中，持久化留给 Phase 6
- **不做跨阶段证据关联**：Phase 2 只处理当前选中阶段
- **不做 Vivado / synthesis / implementation / bitstream**：始终禁止
- **不运行目标项目脚本**：始终禁止

## 4. 功能点拆解

### EV-001 从 StageContext 收集源码证据

| 维度 | 说明 |
|------|------|
| **输入** | `StageContext`（Phase 1 产出），含 `stage_id`、`source_path`、`files[]`、`external_deps[]`、`upstream_refs[]` |
| **输出** | `EvidenceCollection`（含 `evidence_items[]`、`warnings[]`、`stats`） |
| **用户可见表现** | 用户在阶段详情中点击"收集证据"按钮后，系统开始收集；加载状态提示；收集完成后切换到 evidence 面板 |
| **后端责任** | `collect_evidence(root_path, stage_id)` Tauri command；校验输入；遍历 `StageContext.files[]` 读取内容；调用提取器；组装 `EvidenceCollection` |
| **前端责任** | 按钮状态管理（只在 `files[]` 非空时启用）；调用 command；展示加载/完成/错误状态 |
| **验收标准** | 输入有效 `StageContext` → 返回非空 `EvidenceCollection`（即使 `evidence_items[]` 为空也应返回 `success=true`）；空阶段（`stage_empty`）不提供收集入口 |
| **非目标** | 不做增量收集；不做自动触发；不做跨阶段收集 |

### EV-002 为文件/代码片段生成 EvidenceItem

| 维度 | 说明 |
|------|------|
| **输入** | 单个文件路径 + 文件内容（字符串）+ `language` + `source_kind` |
| **输出** | `Vec<EvidenceItem>`（一个文件可产出 0~N 个 evidence item） |
| **用户可见表现** | Evidence 面板中列出每个文件产出的 evidence item 数量 |
| **后端责任** | 按语言类型分派到对应提取器；提取函数定义、类定义、module 定义、port 声明等结构；每个提取结果生成一个 `EvidenceItem` |
| **前端责任** | 展示 evidence item 列表，含文件路径、行号范围、类型标签 |
| **验收标准** | Python 文件至少提取 `def`/`class` 定义；Verilog 文件至少提取 `module` 定义；文档文件提取章节标题；空文件返回 0 个 evidence item |
| **非目标** | 不做完整 AST；不做跨文件符号解析；不做类型推断 |

### EV-003 记录 evidence_id、source_path、line_range、language、source_kind

| 维度 | 说明 |
|------|------|
| **输入** | 提取结果（符号名、行号范围等） |
| **输出** | 每条 `EvidenceItem` 包含完整元数据 |
| **用户可见表现** | Evidence 面板中每个 item 展示 ID、路径、行号、语言类型 |
| **后端责任** | 生成全局唯一 `evidence_id`（格式 `EV-<stage_id>-<序号>`）；记录 `source_path`（绝对路径）；计算 `line_range`（`{start, end}`，1-based，`start <= end`）；从 Phase 1 分类结果继承 `language` 和 `source_kind` |
| **前端责任** | 展示行号范围时使用用户友好格式（如 "L10-25"）；路径可截断展示 |
| **验收标准** | 每个 `evidence_id` 全局唯一；`line_range.start >= 1`；`line_range.start <= line_range.end`；`source_path` 为绝对路径 |
| **非目标** | 不做 evidence_id 的跨会话持久化格式（Phase 6 解决） |

### EV-004 支持 direct / indirect 的 strength 标记

| 维度 | 说明 |
|------|------|
| **输入** | 提取方式和可靠性 |
| **输出** | `EvidenceItem.strength` 字段 |
| **用户可见表现** | Evidence 面板中用颜色/标签区分：绿色=direct、蓝色=indirect |
| **后端责任** | 基于提取方式设置 strength：正则/行级匹配 → `direct`；启发式推断 → `indirect`；解析失败不生成 EvidenceItem，通过 warnings 表达 |
| **前端责任** | 用视觉语义区分 strength 等级 |
| **验收标准** | 正则提取的 `def`/`module` 定义标记为 `direct`；启发式推断的模块边界标记为 `indirect`；解析失败不产生 EvidenceItem，而是记录 warning |
| **非目标** | 不做 LLM 语义判断 strength；不做跨 evidence 矛盾检测（Phase 3+） |

> **注**：`mvp-functional-contract.md` 定义 `evidence_strength` 枚举为 `direct / indirect / weak / conflicting / missing`。Phase 2 只生成 `direct` 和 `indirect` 两个值。`weak`、`conflicting`、`missing` 留给 Phase 3+ 大模型语义判断后使用。数据结构层面保留完整枚举定义（不含 `unknown`），Phase 2 只生成前两个值。解析失败通过 `EvidenceCollection.warnings[]` 和 `EvidenceStats.files_skipped` 表达，不作为 strength 值。

### EV-005 记录 evidence 与阶段、文件、后续 claim 的关系

| 维度 | 说明 |
|------|------|
| **输入** | 所有 `EvidenceItem` |
| **输出** | `EvidenceCollection` 中的索引：`index_by_path`、`index_by_kind`、`index_by_symbol` |
| **用户可见表现** | Evidence 面板支持按文件/类型/符号筛选 |
| **后端责任** | 收集完成后建立三组索引；索引 key 为 `source_path` / `source_kind` / `symbol`，value 为 `evidence_id[]` |
| **前端责任** | 提供筛选/分组交互（按文件、按类型、按符号） |
| **验收标准** | 每个 `evidence_id` 至少出现在 `index_by_path` 和 `index_by_kind` 中；`index_by_symbol` 对无 symbol 的 item 不建立条目；索引覆盖所有 evidence item |
| **非目标** | 不做 evidence → claim 的反向索引（Phase 3 产出 claim 后建立）；不做跨阶段 evidence 关联 |

### EV-006 前端展示 evidence 摘要列表或 evidence 面板

| 维度 | 说明 |
|------|------|
| **输入** | `EvidenceCollection`（Phase 2 后端产出） |
| **输出** | UI 状态更新 |
| **用户可见表现** | 右栏展示 evidence 面板：统计概要（总数、按类型分组数）、evidence item 列表（ID、文件路径、行号、类型、strength、excerpt）、筛选/排序、空状态、错误状态 |
| **后端责任** | 无（纯前端展示） |
| **前端责任** | 新增 `EvidencePanel` 组件；在 `StageDetail` 中增加"收集证据"按钮；收集完成后替换/追加 evidence 面板；支持按文件/类型/符号筛选 |
| **验收标准** | 收集完成后 evidence 面板正确展示所有 item；空结果显示"未收集到证据"；错误状态展示错误信息；不展示原始 JSON |
| **非目标** | 不做代码高亮渲染；不做 evidence item 的可点击跳转到源码（Phase 5）；不做图/问答 |

### EV-007 错误/不确定项表达

| 维度 | 说明 |
|------|------|
| **输入** | 收集过程中的错误和不确定项 |
| **输出** | `EvidenceCollection.warnings[]`（非致命问题）；解析失败不生成 EvidenceItem |
| **用户可见表现** | warnings 面板展示截断/跳过/不可读信息 |
| **后端责任** | 文件不可读 → `file_unreadable` warning；文件过大截断 → `file_too_large` warning + `source_excerpt_truncated` warning；二进制跳过 → warning；解析失败 → 该文件不产出 EvidenceItem，记录 warning |
| **前端责任** | Warning 列表中展示收集阶段的 warning |
| **验收标准** | 所有非致命问题进入 `warnings[]`，不阻断收集；`evidence_items[]` 为空时 `success=true`（除非所有文件不可读） |
| **非目标** | 不做 warning 的自动修复；不做 warning 的分级（全部为非致命） |

### EV-008 evidence collection 不修改目标项目

| 维度 | 说明 |
|------|------|
| **输入** | 目标项目目录路径 |
| **输出** | 无文件系统副作用 |
| **用户可见表现** | 收集前后目标目录无变化 |
| **后端责任** | 只使用 `std::fs::read`、`std::fs::read_dir`、`std::fs::metadata`；禁止 `write`、`create`、`remove`、`rename`；所有产物仅保留在内存 |
| **前端责任** | 无（纯后端约束） |
| **验收标准** | `rg` 检查无 `std::fs::write` / `std::fs::create` / `std::fs::remove_file` / `std::fs::rename` 调用；收集前后 `git status` 无变化 |
| **非目标** | 不做写入 app-owned 目录（Phase 6 解决持久化） |

## 5. 与 Phase 1 的输入关系

Phase 2 消费 Phase 1 的 `StageContext`：

```text
Phase 1 select_stage(root_path, stage_id)
  → StageContext {
      stage_id,
      source_path,
      files: [{ source_path, language, source_kind, size_bytes }],
      external_deps,
      upstream_refs,
      error_code
    }

Phase 2 collect_evidence(root_path, stage_id)
  → 内部重新调用 select_stage 逻辑获取 StageContext
  → 遍历 files[] 读取内容
  → 生成 EvidenceCollection
```

**约束**：Phase 2 复用 Phase 1 的路径校验（`safety_guard`）和文件分类（`file_classifier`），不重复实现。

## 6. 与 Phase 3 的输出关系

Phase 2 产出 `EvidenceCollection`（对应 `mvp-functional-contract.md` 的 `evidence_index.json`），作为 Phase 3 的输入：

```text
Phase 2 产出:
  EvidenceCollection {
    stage_id,
    evidence_items: [{ evidence_id, source_path, line_range, language, source_kind, symbol, summary, strength }],
    index_by_path,
    index_by_kind,
    index_by_symbol,
    warnings,
    stats,
    version
  }

Phase 3 消费:
  → 大模型基于 evidence_items 进行结构化理解
  → 生成 ImplementationUnderstanding
  → 每个 claim 关联 evidence_id
```

## 7. 文档变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-11 | 初始创建 | Claude |
