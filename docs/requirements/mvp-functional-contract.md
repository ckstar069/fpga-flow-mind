# MVP 功能契约

---
status: active
updated: 2026-06-11
---

## 文档目的

本文档定义 MVP 阶段跨 story 的统一数据对象、字段约束、枚举值、依赖关系和端到端验收契约。它不是产品愿景，也不是技术设计，而是需求层面各 story 之间的"接口契约"。

## 适用范围

- 适用于 MVP 阶段的全部 10 个 story
- 适用于设计文档中数据契约的派生来源
- 不适用于 Phase 6 之后的扩展功能

## 与其他需求文档的关系

| 文档 | 职责 | 读者 |
|------|------|------|
| [`product-scope.md`](product-scope.md) | 管产品边界、目标用户、非目标、成功标准 | 所有角色 |
| [`mvp-requirements.md`](mvp-requirements.md) | 管 MVP 范围、必须能力、验收标准清单 | 实施者、审核者 |
| [`stories/*.md`](stories/) | 管单个用户目标、功能点、异常处理 | 实施者 |
| **本文档** | 管跨 story 的统一对象、字段、枚举、依赖和端到端验收 | 设计师、实施者、审核者 |

**冲突处理规则**（按职责分层）：
- **产品边界冲突**（做什么/不做什么）→ 以 `product-scope.md` 为准。
- **MVP 范围冲突**（必须/暂不做能力）→ 以 `mvp-requirements.md` 为准。
- **对象字段、枚举、跨 story 数据流冲突** → 以本文档为准。
- **单 story 内交互细节和功能点** → 以对应 story 为准，但不得违反上层范围和契约。
- 后续技术设计不得重新定义需求对象，应从本文档派生。

---

## MVP 主链路顺序

```text
1. story-open-workspace.md      → workspace_profile.json
2. story-select-stage.md        → stage_context.json
3. story-collect-evidence.md    → evidence_index.json
4. story-generate-understanding.md → implementation_understanding.json
5. story-view-structure.md      → UI 状态（结构图渲染）
6. story-view-dataflow.md       → UI 状态（数据流图渲染）
7. story-view-timing.md         → UI 状态（时序图渲染）
8. story-trace-evidence.md      → UI 状态（evidence 面板）
9. story-ask-node-question.md   → qa_history.json
10. story-persist-and-reopen.md → 全部系统内产物持久化
```

> **关于 visualization_spec.json**：
> - `implementation_understanding.json` 是三类视图的**语义来源**。
> - `visualization_spec.json` 是可选的**派生渲染规格**，可由 story-view-structure/dataflow/timing 生成，也可由 UI 层根据 `implementation_understanding.json` 动态生成。
> - MVP 必须能展示三类视图，但**不强制必须持久化** `visualization_spec.json`，除非后续设计明确需要预计算布局。
> - `trace_index.json` 可由 `implementation_understanding.json` 中的 evidence_refs 直接派生，是否独立持久化由设计决定。

## Story 依赖表

| Story | 依赖输入 | 产出对象 | 下游消费者 | MVP 必须 |
|-------|---------|---------|-----------|---------|
| story-open-workspace | 用户选择的路径 | workspace_profile.json | story-select-stage, story-persist-and-reopen | 是 |
| story-select-stage | workspace_profile.json | stage_context.json | story-collect-evidence | 是 |
| story-collect-evidence | stage_context.json | evidence_index.json | story-generate-understanding | 是 |
| story-generate-understanding | evidence_index.json | implementation_understanding.json | story-view-* , story-trace-evidence, story-ask-node-question | 是 |
| story-view-structure | implementation_understanding.json + evidence_index.json | UI 状态 | story-trace-evidence, story-ask-node-question | 是 |
| story-view-dataflow | implementation_understanding.json + evidence_index.json | UI 状态 | story-trace-evidence, story-ask-node-question | 是 |
| story-view-timing | implementation_understanding.json + evidence_index.json | UI 状态 | story-trace-evidence, story-ask-node-question | 是 |
| story-trace-evidence | evidence_index.json + 用户点击的节点/边 | UI 状态（evidence 面板） | — | 是 |
| story-ask-node-question | evidence_index.json + implementation_understanding.json + 用户问题 | qa_history.json | — | 是 |
| story-persist-and-reopen | 全部系统内产物 | 持久化文件 | story-open-workspace（加载时） | 是 |

---

## 统一对象契约

### workspace_profile.json

| 属性 | 说明 | 必填 | 约束 |
|------|------|------|------|
| `workspace_name` | 目录名 | 是 | 非空字符串 |
| `root_path` | 绝对路径 | 是 | 绝对路径，目录存在 |
| `stages[]` | 候选阶段列表 | 是 | **允许为空**（`no_stage_found` 场景），空数组时 `validity` 应为 `unlikely` |
| `stages[].stage_id` | 阶段标识 | 是 | 目录名或规范化标识 |
| `stages[].source_path` | 阶段目录绝对路径 | 是 | 绝对路径 |
| `stages[].file_count` | 文件数量统计 | 是 | 非负整数 |
| `stages[].status` | 阶段状态 | 是 | 枚举：见 `stage_status` |
| `file_type_stats` | 文件类型统计 | 是 | 对象，键为扩展名，值为数量 |
| `external_refs[]` | 外部模块引用 | 否 | 字符串数组 |
| `validity` | 合法性判断 | 是 | 枚举：见 `workspace_validity` |
| `validity_reasons[]` | 判断理由 | 否 | 字符串数组 |
| `warnings[]` | 扫描过程中的非致命警告 | 否 | 对象数组，每个含 `error_code` 和 `message` |
| `error_codes[]` | 扫描过程中的错误码 | 否 | 字符串数组，枚举：见 `error_code` |
| `scan_timestamp` | 扫描时间戳 | 是 | ISO 8601 格式 |
| `version` | 产物格式版本 | 是 | 字符串，MVP 为 `"1.0.0"` |

**异常场景说明**：
- `validity = uncertain` 或 `unlikely` 时，仍可允许用户强制继续，除非路径不可读（`permission_denied`）。
- `stages[]` 为空时，应记录 `no_stage_found` 错误码，并提示用户重新选择或强制继续。

**生产者**：story-open-workspace  
**消费者**：story-select-stage, story-persist-and-reopen

### stage_context.json

| 属性 | 说明 | 必填 | 约束 |
|------|------|------|------|
| `stage_id` | 选中的阶段标识 | 是 | 与 workspace_profile.stages[].stage_id 一致 |
| `source_path` | 阶段目录绝对路径 | 是 | 绝对路径 |
| `files[]` | 阶段目录下文件列表 | 是 | **允许为空数组**（概览上下文），但进入 evidence 收集前必须非空 |
| `files[].source_path` | 文件绝对路径 | 是 | 绝对路径 |
| `files[].language` | 文件语言 | 是 | 枚举：见 `language` |
| `files[].source_kind` | 来源类型 | 是 | 枚举：见 `source_kind` |
| `files[].size_bytes` | 文件大小 | 否 | 非负整数 |
| `external_deps[]` | 外部依赖标识 | 否 | 字符串数组 |
| `upstream_refs[]` | 上游阶段引用 | 否 | 对象数组，含 stage_id 和 interface_file_path |
| `error_code` | 阶段状态错误码 | 否 | 枚举：见 `error_code`（如 `stage_empty`、`stage_unreadable`） |

**阶段上下文区分**：
- **阶段概览上下文**：用于展示阶段概览面板，允许 `files[]` 为空（空阶段）。此时应展示空状态提示，不触发"开始分析"按钮。
- **可分析上下文**：用于进入 evidence 收集，要求 `files[]` 非空。空阶段不应进入 evidence 收集，除非用户明确强制继续且后续结果为 `evidence_missing` / `unknown`。

**生产者**：story-select-stage  
**消费者**：story-collect-evidence

### evidence_index.json

| 属性 | 说明 | 必填 | 约束 |
|------|------|------|------|
| `evidence_items[]` | evidence item 列表 | 是 | 非空数组 |
| `evidence_items[].evidence_id` | 唯一标识 | 是 | 全局唯一，格式 `"EV-<uuid>"` 或 `"EV-<序号>"` |
| `evidence_items[].source_path` | 源码文件绝对路径 | 是 | 绝对路径 |
| `evidence_items[].language` | 语言 | 是 | 枚举：见 `language` |
| `evidence_items[].source_kind` | 来源类型 | 是 | 枚举：见 `source_kind` |
| `evidence_items[].line_range` | 行号范围 | 是 | 对象：`{ start: number, end: number }`，`start <= end` |
| `evidence_items[].symbol` | 符号名称 | 否 | 字符串，可为空 |
| `evidence_items[].summary` | 代码片段或描述 | 是 | 字符串，不超过 2000 字符 |
| `evidence_items[].strength` | 证据强度 | 是 | 枚举：见 `evidence_strength` |
| `index_by_kind` | 按 source_kind 分组索引 | 是 | 对象，键为 source_kind，值为 evidence_id 数组 |
| `index_by_path` | 按文件路径分组索引 | 是 | 对象，键为 source_path，值为 evidence_id 数组 |
| `index_by_symbol` | 按 symbol 反向索引 | 否 | 对象，键为 symbol，值为 evidence_id 数组 |
| `version` | 产物格式版本 | 是 | 字符串，MVP 为 `"1.0.0"` |

**生产者**：story-collect-evidence  
**消费者**：story-generate-understanding, story-trace-evidence, story-ask-node-question, story-view-*

### implementation_understanding.json

| 属性 | 说明 | 必填 | 约束 |
|------|------|------|------|
| `stage_id` | 阶段标识 | 是 | 与 stage_context.stage_id 一致 |
| `summary` | 阶段摘要 | 是 | 对象：`{ short: string, detailed: string }` |
| `structure_view` | 结构视图数据 | 是 | 对象，可为空（无结构时） |
| `structure_view.nodes[]` | 节点列表 | 是 | 数组 |
| `structure_view.nodes[].node_id` | 节点唯一标识 | 是 | 字符串 |
| `structure_view.nodes[].name` | 节点名称 | 是 | 字符串 |
| `structure_view.nodes[].type` | 节点类型 | 是 | 见各视图 story 定义 |
| `structure_view.nodes[].evidence_refs[]` | 关联 evidence_id 列表 | 是 | 字符串数组，可为空（标注 unknown） |
| `structure_view.nodes[].confidence` | 节点置信度 | 是 | 枚举：见 `node_confidence` |
| `structure_view.edges[]` | 边列表 | 是 | 数组 |
| `structure_view.edges[].edge_id` | 边唯一标识 | 是 | 字符串 |
| `structure_view.edges[].source` | 源节点 id | 是 | 字符串 |
| `structure_view.edges[].target` | 目标节点 id | 是 | 字符串 |
| `structure_view.edges[].type` | 边类型 | 是 | 见各视图 story 定义 |
| `structure_view.edges[].evidence_refs[]` | 关联 evidence_id 列表 | 是 | 字符串数组 |
| `structure_view.edges[].confidence` | 边置信度 | 是 | 枚举：见 `node_confidence` |
| `dataflow_view` | 数据流视图数据 | 是 | 结构同 structure_view，节点/边类型不同 |
| `timing_view` | 时序视图数据 | 是 | 结构同 structure_view，节点/边类型不同 |
| `concepts[]` | 关键概念列表 | 否 | 数组，每个含 name、description、evidence_refs |
| `formulas[]` | 关键公式列表 | 否 | 数组，每个含 expression、description、evidence_refs |
| `signals[]` | 关键信号列表 | 否 | 数组，每个含 name、type、source、evidence_refs |
| `uncertainties[]` | 不确定项列表 | 是 | 数组，每个含 description、type、related_evidence_refs |
| `version` | 产物格式版本 | 是 | 字符串，MVP 为 `"1.0.0"` |

**生产者**：story-generate-understanding  
**消费者**：story-view-structure, story-view-dataflow, story-view-timing, story-trace-evidence, story-ask-node-question

### visualization_spec.json（可选）

| 属性 | 说明 | 必填 | 约束 |
|------|------|------|------|
| `stage_id` | 阶段标识 | 是 | 与 implementation_understanding.stage_id 一致 |
| `view_kind` | 视图类型 | 是 | 枚举：见 `view_kind` |
| `layout` | 布局数据 | 是 | 对象，含节点位置、边路径 |
| `layout.nodes[].node_id` | 节点 id | 是 | 与 implementation_understanding 中对应 |
| `layout.nodes[].x` | X 坐标 | 是 | 数字 |
| `layout.nodes[].y` | Y 坐标 | 是 | 数字 |
| `layout.nodes[].width` | 宽度 | 否 | 数字 |
| `layout.nodes[].height` | 高度 | 否 | 数字 |
| `styles` | 样式定义 | 否 | 对象，按节点/边类型定义颜色、形状、线型 |
| `version` | 产物格式版本 | 是 | 字符串 |

**说明**：
- `implementation_understanding.json` 是三类视图的**语义来源**。
- `visualization_spec.json` 是可选的**派生渲染规格**，可由 story-view-structure/dataflow/timing 生成，也可由 UI 层根据 `implementation_understanding.json` 动态生成。
- MVP 必须能展示三类视图，但**不强制必须持久化** `visualization_spec.json`，除非后续设计明确需要预计算布局。

**生产者**：story-view-*（可由前端动态生成，可选）  
**消费者**：UI 渲染层

### trace_index.json

| 属性 | 说明 | 必填 | 约束 |
|------|------|------|------|
| `stage_id` | 阶段标识 | 是 | — |
| `node_to_evidence` | 节点到 evidence 的映射 | 是 | 对象，键为 node_id，值为 evidence_id 数组 |
| `edge_to_evidence` | 边到 evidence 的映射 | 是 | 对象，键为 edge_id，值为 evidence_id 数组 |
| `claim_to_evidence` | 结论到 evidence 的映射 | 否 | 对象，键为 claim_id，值为 evidence_id 数组 |
| `version` | 产物格式版本 | 是 | 字符串 |

**生产者**：story-generate-understanding（可独立生成）  
**消费者**：story-trace-evidence

### qa_history.json

| 属性 | 说明 | 必填 | 约束 |
|------|------|------|------|
| `stage_id` | 阶段标识 | 是 | — |
| `entries[]` | 问答记录 | 是 | 数组 |
| `entries[].entry_id` | 记录唯一标识 | 是 | 字符串 |
| `entries[].timestamp` | 时间戳 | 是 | ISO 8601 |
| `entries[].question` | 用户问题 | 是 | 字符串 |
| `entries[].answer` | 系统回答 | 是 | 字符串 |
| `entries[].evidence_refs[]` | 回答引用的 evidence_id | 是 | 字符串数组，可为空 |
| `entries[].confidence` | 回答置信度 | 是 | 枚举：见 `claim_confidence` |
| `entries[].node_id` | 关联节点 id | 否 | 字符串 |
| `version` | 产物格式版本 | 是 | 字符串 |

**生产者**：story-ask-node-question  
**消费者**：story-persist-and-reopen（加载时恢复）

---

## 关键枚举与状态

### workspace_validity

| 值 | 含义 |
|----|------|
| `likely_valid` | 符合 `ai_project_template` 特征 |
| `uncertain` | 部分特征匹配 |
| `unlikely` | 不符合特征 |

### stage_status

| 值 | 含义 |
|----|------|
| `available` | 存在且有内容可分析 |
| `empty` | 目录存在但无文件 |
| `missing` | 预期存在但未找到 |
| `naming_anomaly` | 命名不符合预期模式 |
| `unreadable` | 目录存在但无法读取 |

### source_kind

| 值 | 含义 |
|----|------|
| `python_stage` | Python 阶段代码 |
| `rtl` | Verilog / SystemVerilog RTL |
| `test` | 测试文件 |
| `doc` | 文档文件 |
| `config` | 配置文件 |
| `external_module` | 外部模块引用 |

### language

| 值 | 含义 |
|----|------|
| `python` | Python |
| `verilog` | Verilog |
| `systemverilog` | SystemVerilog |
| `markdown` | Markdown |
| `text` | 纯文本 |
| `json` | JSON |
| `yaml` | YAML |
| `toml` | TOML |
| `unknown` | 无法识别 |

### evidence_strength

| 值 | 含义 |
|----|------|
| `direct` | 直接源码证据（如 module 定义） |
| `indirect` | 间接证据（如调用关系推断） |
| `weak` | 弱证据（如注释提及） |
| `conflicting` | 与其他证据矛盾 |
| `missing` | 证据缺失 |

### claim_confidence / node_confidence

| 值 | 含义 | 使用场景 |
|----|------|---------|
| `confirmed` | 强源码证据直接支撑 | claim、节点、边 |
| `supported` | 有证据支撑，需辅助推断 | claim、节点、边 |
| `inferred` | 基于间接证据推断 | claim、节点、边 |
| `unknown` | 证据不足 | claim、节点、边、回答 |
| `conflicting` | 存在矛盾证据 | claim、节点、边、回答 |

### view_kind

| 值 | 含义 |
|----|------|
| `structure` | 结构视图 |
| `dataflow` | 数据流视图 |
| `timing` | 时序/流水视图 |

### error_code（MVP 最小集）

| 值 | 场景 | 来源 story |
|----|------|-----------|
| `path_not_found` | 路径不存在 | story-open-workspace |
| `not_directory` | 路径不是目录 | story-open-workspace |
| `permission_denied` | 无读权限 | story-open-workspace, story-collect-evidence |
| `no_stage_found` | 未识别到阶段目录 | story-open-workspace |
| `stage_empty` | 阶段目录为空 | story-select-stage |
| `stage_unreadable` | 阶段目录不可读 | story-select-stage |
| `file_unreadable` | 单个文件不可读 | story-collect-evidence |
| `file_too_large` | 文件超过大小上限 | story-collect-evidence |
| `evidence_missing` | 无 evidence 可分析 | story-collect-evidence |
| `model_output_invalid` | 模型返回格式错误 | story-generate-understanding |
| `grounding_failed` | grounding 检查失败 | story-generate-understanding |
| `persist_failed` | 持久化保存失败 | story-persist-and-reopen |
| `load_failed` | 产物加载失败 | story-persist-and-reopen |
| `source_changed` | 源文件已变更 | story-persist-and-reopen |

---

## 跨 Story 数据流

```text
[用户选择路径]
    ↓
story-open-workspace ──→ workspace_profile.json ──┐
    ↓                                              │
story-select-stage ────→ stage_context.json ──────┤
    ↓                                              │
story-collect-evidence → evidence_index.json ─────┤
    ↓                                              │
story-generate-understanding ──→ implementation_understanding.json
                                    ↓
                    ┌───────────────┼───────────────┐
                    ↓               ↓               ↓
            story-view-structure  story-view-dataflow  story-view-timing
                    ↓               ↓               ↓
                    └───────────────┼───────────────┘
                                    ↓
                          story-trace-evidence
                          story-ask-node-question ──→ qa_history.json
                                    ↓
                          story-persist-and-reopen
                                    ↓
                          [持久化到 app-owned 目录]
```

**说明**：
- 实线箭头表示系统内产物（JSON 文件）的传递
- 三个视图 story 消费同一 implementation_understanding.json，渲染为 UI 状态（视图渲染）
- `visualization_spec.json` 为可选产物，是否持久化由设计决定
- `trace_index.json` 可由 implementation_understanding.json 中的 evidence_refs 派生，是否独立持久化由设计决定
- trace-evidence 和 ask-node-question 依赖 evidence_index.json + implementation_understanding.json
- persist-and-reopen 将**必须产物**（workspace_profile、evidence_index、implementation_understanding、qa_history）保存到磁盘，可选产物（visualization_spec、trace_index）视设计而定

---

## MVP 必须项 / 可选项

### 必须项（主链路闭环）

| 项 | story | 说明 |
|----|-------|------|
| 路径选择与验证 | story-open-workspace | 否则无法开始 |
| 阶段识别与选择 | story-select-stage | 否则无法聚焦 |
| evidence 收集 | story-collect-evidence | 否则无证据基础 |
| 结构化理解生成 | story-generate-understanding | 否则无理解产物 |
| 三类视图展示 | story-view-structure/dataflow/timing | 核心用户价值 |
| 证据追溯 | story-trace-evidence | 核心用户价值 |
| 追问能力 | story-ask-node-question | 核心用户价值 |
| 自动持久化 | story-persist-and-reopen | 基础体验 |

### 可选项（首发非必须，可后续补充）

| 项 | story | 说明 |
|----|-------|------|
| 阶段缺失提示 | story-select-stage | 不阻塞主链路 |
| 阶段命名异常处理 | story-select-stage | 不阻塞主链路 |
| 源码变更自动检测 | story-persist-and-reopen | 可手动重新分析替代 |
| 产物版本兼容性检查 | story-persist-and-reopen | MVP 初期只有一个版本 |
| 手动保存按钮 | story-persist-and-reopen | 自动保存已足够 |
| 产物清理管理 | story-persist-and-reopen | 可手动删除文件替代 |

---

## 端到端验收场景

### 场景 1：合法业务项目 + 标准阶段 + evidence 充足

**输入条件**：
- 由 `ai_project_template` 创建的业务项目
- 包含标准命名的阶段目录（L0、L1、...、RTL）
- 各阶段包含 Python 和/或 Verilog 代码文件
- 代码结构清晰，可被静态提取识别

**执行步骤**：
1. 用户打开项目 → 系统识别为 `likely_valid`
2. 用户选择 L3 阶段 → 系统列出该阶段文件
3. 用户点击"收集证据" → 系统提取所有 evidence item
4. 用户点击"生成理解" → 系统生成 ImplementationUnderstanding
5. 用户查看结构图、数据流图、时序图 → 图中节点和边均有 evidence 关联
6. 用户点击模块节点 → evidence 面板展示源码证据
7. 用户追问"这个模块的输入位宽是多少" → 系统基于证据回答并引用 evidence_id
8. 关闭应用后重新打开 → 系统自动加载之前的分析结果

**期望输出对象**：
- workspace_profile.json：`validity = likely_valid`，stages 包含 L0~RTL
- evidence_index.json：evidence_items 数量 > 50，覆盖 python_stage 和 rtl
- implementation_understanding.json：structure_view / dataflow_view / timing_view 均有节点和边，confidence 以 confirmed 和 supported 为主
- qa_history.json：至少一条问答记录，answer.confidence 为 confirmed 或 supported

**期望 UI 表现**：
- workspace 概览显示阶段列表和文件统计
- 阶段概览显示文件列表和"开始分析"按钮
- 三类视图正确渲染，节点/边可点击
- evidence 面板展示代码片段和行号范围
- 问答面板展示对话历史和 evidence 引用

**期望安全结果**：
- 目标项目目录无新增/修改/删除文件
- 所有写入发生在 app-owned 目录

**不应发生的行为**：
- 不应将项目误判为 `unlikely` 并阻止用户继续
- 不应在无 evidence 时生成 confirmed 结论
- 不应隐藏 unknown/inferred 节点
- 不应运行 Vivado 或 synthesis

---

### 场景 2：项目可读但阶段缺失或命名异常

**输入条件**：
- 由 `ai_project_template` 创建的业务项目
- 部分预期阶段目录缺失（如缺少 L2、L5）
- 部分阶段命名异常（如 `rtl_final` 而非 `RTL`）
- 其余阶段包含正常代码文件

**执行步骤**：
1. 用户打开项目 → 系统识别为 `uncertain`（因阶段不完整）
2. workspace 概览展示可用阶段，标注缺失阶段和命名异常阶段
3. 用户选择命名异常的 `rtl_final` 阶段 → 系统允许选择
4. 后续流程同场景 1

**期望输出对象**：
- workspace_profile.json：`validity = uncertain`，`validity_reasons` 包含阶段缺失说明
- stages 列表中：缺失阶段 `status = missing`，异常命名阶段 `status = naming_anomaly`
- 其余产物正常生成

**期望 UI 表现**：
- 概览面板显示"阶段不完整"警告（非错误）
- 命名异常阶段以降级样式展示但可点击
- 缺失阶段显示"未找到"但不阻塞其他阶段

**期望安全结果**：同场景 1

**不应发生的行为**：
- 不应因阶段缺失阻止用户分析可用阶段
- 不应将命名异常阶段自动重命名或删除
- 不应报错退出

---

### 场景 3：evidence 不足但仍可生成 unknown/inferred 视图

**输入条件**：
- 业务项目中某个阶段目录存在但文件极少
- 仅有 1-2 个代码文件，且缺少文档和测试
- 代码结构简单，静态提取可识别的符号有限

**执行步骤**：
1. 用户打开项目并选择该阶段
2. 收集证据 → evidence item 数量 < 10
3. 生成理解 → 大量 claim 标注为 `unknown` 或 `inferred`
4. 查看结构图 → 大部分节点为 `unknown`，边为 `inferred`
5. 点击 unknown 节点 → evidence 面板提示"证据不足"
6. 追问"这个阶段做了什么" → 系统回答"根据当前证据无法确定"

**期望输出对象**：
- evidence_index.json：evidence_items 数量 < 10，部分 `strength = weak`
- implementation_understanding.json：大量 `confidence = unknown/inferred`，`uncertainties[]` 非空
- visualization_spec.json：节点以降级样式渲染

**期望 UI 表现**：
- 图中大量节点显示为灰色/虚线/"?"标记
- 用户悬停时显示"证据不足"提示
- evidence 面板明确标注"暂无关联证据"
- 问答面板回答 `unknown` 并说明原因

**期望安全结果**：同场景 1

**不应发生的行为**：
- 不应隐藏 unknown/inferred 节点（必须可见）
- 不应将 inferred 呈现为 confirmed
- 不应编造 evidence 或结论
- 不应因 evidence 不足而报错退出

---

## 文档索引

本文档链接到的相关文档：

- [`product-scope.md`](product-scope.md) — 产品范围与边界
- [`mvp-requirements.md`](mvp-requirements.md) — MVP 范围与验收标准
- [`stories/story-open-workspace.md`](stories/story-open-workspace.md) — 打开项目
- [`stories/story-select-stage.md`](stories/story-select-stage.md) — 选择阶段
- [`stories/story-collect-evidence.md`](stories/story-collect-evidence.md) — 收集证据
- [`stories/story-generate-understanding.md`](stories/story-generate-understanding.md) — 生成理解
- [`stories/story-view-structure.md`](stories/story-view-structure.md) — 结构图
- [`stories/story-view-dataflow.md`](stories/story-view-dataflow.md) — 数据流图
- [`stories/story-view-timing.md`](stories/story-view-timing.md) — 时序图
- [`stories/story-trace-evidence.md`](stories/story-trace-evidence.md) — 追溯证据
- [`stories/story-ask-node-question.md`](stories/story-ask-node-question.md) — 追问
- [`stories/story-persist-and-reopen.md`](stories/story-persist-and-reopen.md) — 持久化
