# Phase 2 实施计划：证据索引与 Evidence Model

---
status: active
updated: 2026-06-11
---

> 本文档是 Phase 2（证据索引与 evidence model）的实施计划，包含入口条件、任务拆解（P2-T01~P2-T10）、每任务规格、编码顺序、验证顺序和退出标准。
> 不写产品代码，不实现 evidence/graph/LLM/Q&A。

## 1. Phase 2 目标

为 `fpga-flow-mind` 建立源码证据收集、索引和展示的基础能力：

1. 从阶段目录的源码/测试/文档/配置文件中提取结构化 evidence item
2. 为每个 evidence item 分配唯一 ID，记录源文件路径和行号范围
3. 建立按文件/类型/符号的交叉索引
4. 前端展示 evidence 摘要列表
5. 确保证据收集不修改目标项目

## 2. 入口条件

| 条件 | 状态 | 证据 |
|------|------|------|
| Phase 1 编码完成 | ✅ | P1-T01~P1-T12 done |
| Phase 1 验收通过 | ✅ | `phase-1-completion-review.md` status=active |
| Phase 1 测试通过 | ✅ | `cargo test` 65 passed |
| Phase 1 文档同步 | ✅ | 所有 Phase 1 文档 status=active |
| Phase 2 文档就绪 | ✅ | 需求/设计/验证/UI/计划 5 文档 status=draft |
| 允许进入 Phase 2 | ✅ | Phase 1 completion review 结论 |

## 3. 允许修改范围

### 3.1 新增文件

| 路径 | 说明 |
|------|------|
| `src-tauri/src/evidence/mod.rs` | evidence 模块声明 |
| `src-tauri/src/evidence/models.rs` | EvidenceItem / EvidenceCollection / EvidenceStrength |
| `src-tauri/src/evidence/id_generator.rs` | evidence_id 生成器 |
| `src-tauri/src/evidence/excerpt.rs` | summary 提取与截断 |
| `src-tauri/src/evidence/index_builder.rs` | 索引构建 |
| `src-tauri/src/evidence/collector.rs` | 核心收集调度器 |
| `src-tauri/src/evidence/extractors/mod.rs` | 提取器 trait + 分派 |
| `src-tauri/src/evidence/extractors/python.rs` | Python 提取器 |
| `src-tauri/src/evidence/extractors/verilog.rs` | Verilog 提取器 |
| `src-tauri/src/evidence/extractors/systemverilog.rs` | SystemVerilog 提取器 |
| `src-tauri/src/evidence/extractors/markdown.rs` | Markdown 提取器 |
| `src-tauri/src/evidence/extractors/config.rs` | TCL/XDC 提取器 |
| `src-tauri/src/commands/collect_evidence.rs` | Tauri command |
| `src/features/evidence/EvidencePanel.tsx` | 证据面板 |
| `src/features/evidence/EvidenceStatsBar.tsx` | 统计概要 |
| `src/features/evidence/EvidenceFilterBar.tsx` | 筛选栏 |
| `src/features/evidence/EvidenceItemList.tsx` | 证据项列表 |
| `src/features/evidence/EvidenceItemCard.tsx` | 证据项卡片 |
| `src/features/evidence/EvidenceWarningList.tsx` | 警告列表 |
| `src/features/workspace/components/CollectEvidenceButton.tsx` | 收集按钮 |

### 3.2 修改文件

| 路径 | 修改内容 |
|------|----------|
| `src-tauri/src/commands/mod.rs` | 新增 `collect_evidence` 模块声明 |
| `src-tauri/src/lib.rs` | 新增 `evidence` 模块声明 + command 注册 |
| `src-tauri/src/models/enums.rs` | 新增 ErrorCode 变体（Phase 2 错误码） |
| `src/types/workspace.ts` | 新增 EvidenceItem / EvidenceCollection 等类型 |
| `src/lib/tauriCommands.ts` | 新增 `collectEvidence()` 函数 |
| `src/features/workspace/WorkspacePage.tsx` | 新增 collecting/collected 状态、evidence 面板集成 |
| `src/features/workspace/components/StageDetail.tsx` | 新增"收集证据"按钮位置 |

## 4. 禁止事项

| 禁止 | 说明 |
|------|------|
| 不运行 Vivado / synthesis / implementation / bitstream | 始终禁止 |
| 不运行目标项目脚本 | 始终禁止 |
| 不修改目标项目文件 | 始终只读 |
| 不做 AST 复杂语义分析 | Phase 2 只做正则/行级匹配 |
| 不做 LLM 调用 | Phase 2 不调用任何大模型 API |
| 不做正确/错误判断 | Phase 2 不判断代码逻辑是否正确 |
| 不做结构图/数据流图/时序图 | Phase 4 范围 |
| 不做 Q&A / 追问 | Phase 5 范围 |
| 不做持久化 | Phase 6 范围 |
| 不做跨阶段关联 | Phase 2 只处理当前选中阶段 |

## 5. 任务拆解

### P2-T01 定义 Rust 数据模型与枚举

**目标**：在 `src-tauri/src/evidence/models.rs` 中定义 Phase 2 所有数据结构。

**规格**：
- `EvidenceItem` struct：evidence_id, source_path, language, source_kind, line_range, symbol, summary, strength
- `LineRange` struct：start (u32), end (u32)
- `EvidenceStrength` 枚举：Direct, Indirect, Weak, Conflicting, Missing
- `EvidenceCollection` struct：stage_id, evidence_items, index_by_path, index_by_kind, index_by_symbol, warnings, stats, version
- `EvidenceWarning` struct：error_code, message, source_path
- `EvidenceStats` struct：files_processed, files_skipped, total_items, items_by_kind, items_by_strength
- `RawExtraction` struct（提取器中间产物）：symbol, line_range, raw_excerpt, strength
- 所有 struct 使用 `#[derive(Debug, Clone, Serialize, Deserialize)]`
- serde 输出 snake_case JSON（与 Phase 1 一致）
- 在 `src-tauri/src/models/enums.rs` 中扩展 `ErrorCode` 枚举：新增 `EvidenceCollectionFailed`, `SourceExcerptTruncated`, `BinaryFileSkipped`, `NonUtf8FileSkipped`

**验收**：`cargo check` 通过，`cargo test` 编译通过。

**依赖**：无。

### P2-T02 实现 evidence_id 生成器和 excerpt 模块

**目标**：实现 `id_generator.rs` 和 `excerpt.rs`。

**规格**：

`id_generator.rs`：
- `EvidenceIdGenerator` struct：stage_id (String), counter (u32)
- `new(stage_id: &str) -> Self`
- `next_id(&mut self) -> String`：格式 `EV-<stage_id>-{:06}`
- 单元测试：格式正确、连续递增、不同 stage、唯一性（1000 次调用）

`excerpt.rs`：
- `ExcerptProcessor` struct（无状态）
- `process(raw: &str, total_lines: usize) -> String`：summary 截断逻辑
  - raw.len() ≤ 500 → 原样返回
  - raw.len() > 500 → 前 400 字符 + `"...(已截断，共 N 行)"`
- `file_summary(content: &str, total_lines: usize) -> String`：整文件摘要
  - 前 200 字符 + `"...(共 N 行)"`（如果超出）
- 单元测试：短文本、恰好 500、超出、整文件、空内容

**验收**：`cargo test` 包含 id_generator 和 excerpt 的 ~9 个测试通过。

**依赖**：P2-T01。

### P2-T03 实现提取器 trait 和 Python 提取器

**目标**：定义 `EvidenceExtractor` trait，实现 Python 提取器。

**规格**：

`extractors/mod.rs`：
- `EvidenceExtractor` trait：`fn extract(&self, content, source_path, language, source_kind) -> Vec<RawExtraction>`
- `dispatch_extractor(language: &Language) -> Box<dyn EvidenceExtractor>`：分派逻辑
- `FallbackExtractor`：整文件级 evidence，strength=indirect

`extractors/python.rs`：
- 匹配 `^def\s+(\w+)\s*\(` → symbol=函数名
- 匹配 `^class\s+(\w+)` → symbol=类名
- 函数边界推断：基于 `def` 行缩进级别，到下一个同级 `def` 或 EOF
- strength：关键字匹配 → direct，边界推断 → indirect
- 单元测试：简单函数、多函数、类定义、嵌套函数、空文件、注释

**验收**：`cargo test` 包含 Python 提取器的 ~5 个测试通过。

**依赖**：P2-T01。

### P2-T04 实现 Verilog / SystemVerilog / Markdown / Config 提取器

**目标**：实现剩余 4 个语言提取器。

**规格**：

`extractors/verilog.rs`：
- 匹配 `^\s*module\s+(\w+)` → symbol=module 名
- 匹配 `^\s*endmodule` → module 结束
- module 边界：从 `module` 到 `endmodule`
- 单元测试：单 module、多 module、空 module、无 module

`extractors/systemverilog.rs`：
- 匹配 module / interface / class / package 及对应 end 关键字
- 单元测试：module、interface、class

`extractors/markdown.rs`：
- 匹配 `^#{1,3}\s+(.+)` → symbol=标题文本
- 章节范围：从标题到下一个同级/上级标题前一行
- 单元测试：多级标题、空文件、纯文本

`extractors/config.rs`：
- 匹配 `^\s*proc\s+(\w+)` → symbol=TCL 过程名
- 匹配 `^\s*(set_property|create_clock|set_input_delay|set_output_delay)` → indirect
- 单元测试：TCL proc、XDC 约束、空文件

**验收**：`cargo test` 包含 4 个提取器的 ~13 个测试通过。

**依赖**：P2-T03。

### P2-T05 实现 index_builder 和 collector

**目标**：实现索引构建和核心收集调度器。

**规格**：

`index_builder.rs`：
- `build(items: &[EvidenceItem]) -> (IndexByPath, IndexByKind, IndexBySymbol)`
- index_by_path：所有 item 必须出现
- index_by_kind：所有 item 必须出现，key 为 source_kind 的 snake_case 字符串
- index_by_symbol：仅 symbol 非 None 的 item
- 单元测试：索引覆盖、symbol 索引、空输入、同文件多 item

`collector.rs`：
- `EvidenceCollector` struct
- `new(stage_id: &str) -> Self`
- `collect(stage_context: &StageContext) -> EvidenceCollection`
- 执行流程：遍历 files → 文件预检 → 读取内容 → 分派提取器 → 分配 ID → 截断 summary → 构建索引 → 计算 stats
- 非致命错误进入 warnings，不阻断
- 单元测试：正常收集、混合语言、大文件跳过、非 UTF-8 跳过、warnings 传播、空结果

**验收**：`cargo test` 包含 index_builder (~4) + collector (~6) 的 ~10 个测试通过。

**依赖**：P2-T02, P2-T04。

### P2-T06 实现 collect_evidence Tauri command

**目标**：实现 `collect_evidence` command，注册到 Tauri 应用。

**规格**：

`commands/collect_evidence.rs`：
- 签名：`fn collect_evidence(root_path: String, stage_id: String) -> Result<CommandResult<EvidenceCollection>, String>`
- 参数校验：复用 safety_guard
- 获取 StageContext：复用 select_stage 逻辑
- 调用 collector.collect()
- 返回 CommandResult<EvidenceCollection>

注册：
- `src-tauri/src/lib.rs`：新增 `evidence` 模块声明
- `src-tauri/src/commands/mod.rs`：新增 `collect_evidence` 模块声明
- Tauri builder：`.invoke_handler(tauri::generate_handler![..., collect_evidence])`

集成测试：
- 正常收集、空阶段、阶段不存在、路径不存在、混合语言、大文件跳过、非 UTF-8、warnings 传播、空结果、evidence_id 唯一

**验收**：`cargo test` 包含 command 的 ~10 个集成测试通过，`cargo check` 通过。

**依赖**：P2-T05。

### P2-T07 实现前端类型定义和 Tauri command 调用

**目标**：新增 TypeScript 类型，实现 `collectEvidence()` 调用。

**规格**：

`src/types/workspace.ts` 新增：
- `EvidenceStrength` type
- `EvidenceItem` interface
- `LineRange` interface
- `EvidenceCollection` interface
- `EvidenceWarning` interface
- `EvidenceStats` interface

`src/lib/tauriCommands.ts` 新增：
- `collectEvidence(rootPath: string, stageId: string): Promise<EvidenceCollection>`
- 使用 `invoke<CommandResult<EvidenceCollection>>('collect_evidence', { rootPath, stageId })`
- 注意 Tauri v2 camelCase 参数

**验收**：`npm run build` 通过（tsc + vite build）。

**依赖**：P2-T06（后端 command 已注册）。

### P2-T08 实现前端 EvidencePanel 组件

**目标**：实现证据面板相关的所有前端组件。

**规格**：

`CollectEvidenceButton.tsx`：
- 5 种状态：disabled / idle / loading / done / error
- 调用 `collectEvidence(rootPath, stageId)`
- 展示证据数量（done 状态）

`EvidencePanel.tsx`：
- 容器组件，包含 StatsBar + FilterBar + ItemList + WarningList
- Tab 切换：阶段详情 / 证据

`EvidenceStatsBar.tsx`：
- 展示 total_items、items_by_kind、items_by_strength、files_skipped
- 空状态："未收集到证据"

`EvidenceFilterBar.tsx`：
- 筛选维度：全部 / 按文件 / 按类型 / 按符号
- 下拉选择器

`EvidenceItemList.tsx`：
- 渲染 EvidenceItemCard 列表
- 根据筛选条件过滤

`EvidenceItemCard.tsx`：
- 展示 evidence_id、strength 标签、文件路径（截断 + hover tooltip）、行号范围、symbol、summary
- Strength 颜色：green=direct, blue=indirect
- 点击展开/收起 summary

`EvidenceWarningList.tsx`：
- 可折叠，默认折叠
- 展示 warning 的 error_code + message + source_path

**验收**：`npm run build` 通过，手工 `cargo tauri dev` 验证组件渲染。

**依赖**：P2-T07。

### P2-T09 集成到 WorkspacePage 状态机

**目标**：在 `WorkspacePage` 中集成 evidence 收集的完整状态流转。

**规格**：

状态机扩展：
- 新增 `collecting_evidence`、`evidence_collected`、`evidence_failed` 三个状态
- 状态转换：`stage_selected` → `collecting_evidence` → `evidence_collected` / `evidence_failed`
- 重新收集：`evidence_collected` → `collecting_evidence`
- 切换阶段：清除 evidence 状态

`StageDetail.tsx` 改造：
- 新增"收集证据"按钮位置（文件列表上方）
- 按钮状态绑定 WorkspacePage 状态

Tab 切换：
- 收集完成后右栏切换为 Tab 布局：阶段详情 / 证据(N)
- 默认切换到"证据" tab

**验收**：`cargo tauri dev` 中完整链路验证（打开 → 选阶段 → 收集 → 查看 → 筛选 → 警告）。

**依赖**：P2-T08。

### P2-T10 执行 Phase 2 验收与文档同步

**目标**：运行所有验证，同步文档，确认退出条件。

**规格**：

验证清单：
- `cargo test`：所有测试通过（Phase 1 原有 65 + Phase 2 新增 ~50 ≈ 115）
- `npm run build`：前端构建通过
- `cargo check`：Rust 检查通过
- `rg` 安全检查：无写入/执行/Vivado API
- `rg` 越界检查：无 Phase 3+ 关键字（LLM/Q&A/graph/ImplementationUnderstanding）
- 手工 Tauri 桌面验收：10 步验收全部通过
- 目标目录无变化

文档同步：
- Phase 2 所有文档 status: draft → active
- `phase-2-evidence-validation.md` 填入实际验证结果
- 新增 `phase-2-completion-review.md`
- 更新 `docs/planning/README.md` 状态
- 更新 `AGENTS.md` Phase 2 evidence 规则

**验收**：Phase 2 completion review 结论="允许进入 Phase 3"。

**依赖**：P2-T09。

## 6. 编码顺序

```text
P2-T01 (数据模型)
  │
  ├── P2-T02 (ID 生成器 + excerpt)
  │     │
  ├── P2-T03 (trait + Python)
  │     │
  │     └── P2-T04 (Verilog/SV/MD/Config)
  │           │
  │           └── P2-T05 (index_builder + collector)
  │                 │
  │                 └── P2-T06 (command + 注册)
  │                       │
  │                       └── P2-T07 (TS 类型 + 调用)
  │                             │
  │                             └── P2-T08 (前端组件)
  │                                   │
  │                                   └── P2-T09 (状态机集成)
  │                                         │
  │                                         └── P2-T10 (验收 + 文档)
  │
  └── 可并行：P2-T02 和 P2-T03 可同时开始（都只依赖 T01）
```

## 7. 验证顺序

每完成一个任务，运行对应验证：

| 任务完成后 | 验证 |
|-----------|------|
| T01 | `cargo check` |
| T02 | `cargo test evidence::id_generator evidence::excerpt` |
| T03 | `cargo test evidence::extractors::python` |
| T04 | `cargo test evidence::extractors` |
| T05 | `cargo test evidence::index_builder evidence::collector` |
| T06 | `cargo test collect_evidence` + `cargo check` |
| T07 | `npm run build` |
| T08 | `npm run build` |
| T09 | `cargo tauri dev` 手工验证 |
| T10 | 全量验证 |

## 8. 退出标准

| 标准 | 验证方式 |
|------|----------|
| Rust 全量测试通过 | `cargo test`（~115 passed） |
| 前端构建通过 | `npm run build` |
| Tauri 桌面验收通过 | 手工 10 步验证 |
| 安全约束满足 | `rg` 检查无写入/执行 API |
| Phase 1 功能无回归 | 原有 65 个测试仍通过 |
| 代码越界检查 | 无 Phase 3+ 关键字 |
| 文档同步 | Phase 2 文档 status=active |
| 目标目录无变化 | 收集前后 `git status` 无变化 |

## 9. 风险与回滚

### 9.1 风险

| 风险 | 概率 | 影响 | 缓解 |
|------|------|------|------|
| Python 缩进推断不准 | 中 | 部分函数边界标记为 indirect | Phase 2 允许 indirect，不追求完美 |
| SystemVerilog 复杂语法 | 低 | 部分结构未提取 | Phase 2 只提取基础结构，复杂留给 Phase 3 |
| 前端 EvidencePanel 性能 | 低 | 大量 evidence 时渲染慢 | Phase 2 文件数量有限，不做虚拟滚动 |
| Phase 1 回归 | 低 | 新代码影响已有功能 | 每步验证 Phase 1 测试 |

### 9.2 回滚策略

- Phase 2 新增代码在独立模块 `evidence/` 中，不修改 Phase 1 核心逻辑
- 如需回滚，移除 `evidence/` 模块、`collect_evidence` command、前端 evidence 组件即可
- Phase 1 核心链路（workspace scanner、stage detector、select_stage）不受影响

## 10. 偏离产品方向的风险检查

| 检查项 | 结果 |
|--------|------|
| 是否偏离"理解工具"定位？ | ❌ evidence 收集是理解的基础 |
| 是否引入不必要的复杂度？ | ❌ 正则提取，无 AST |
| 是否保持目标项目只读？ | ✅ 只使用 `std::fs::read_to_string` |
| 是否做了正确/错误判断？ | ❌ 只提取事实性证据 |
| 是否运行了 Vivado/脚本？ | ❌ 禁止 |

## 11. 文档变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-11 | 初始创建 | Claude |
| 2026-06-11 | 文档收口：status draft → active；修复 ErrorCode 路径描述为精确文件路径 `src-tauri/src/models/enums.rs`；添加收口记录 | Claude |

## 12. 文档收口记录

**收口日期**：2026-06-11

### 收口范围

以下 6 份 Phase 2 文档从 `draft` 收口为 `active`，作为 Phase 2 编码的权威依据：

| 文档 | 路径 | 收口状态 |
|------|------|----------|
| Phase 2 需求 | `docs/requirements/phase-2-evidence-requirements.md` | ✅ active |
| Phase 2 数据模型设计 | `docs/design/phase-2-evidence-model.md` | ✅ active |
| Phase 2 后端收集器设计 | `docs/design/phase-2-evidence-collector-design.md` | ✅ active |
| Phase 2 前端面板设计 | `docs/ui-ux/phase-2-evidence-view.md` | ✅ active |
| Phase 2 验证计划 | `docs/testing/phase-2-evidence-validation.md` | ✅ active |
| Phase 2 实施计划 | `docs/planning/phase-2-implementation-plan.md` | ✅ active |

### 跨文档一致性确认

| 检查项 | 结果 |
|--------|------|
| `EvidenceItem` 字段一致（evidence_id, source_path, language, source_kind, line_range, symbol, summary, strength） | ✅ |
| `EvidenceStrength` 枚举一致（direct/indirect/weak/conflicting/missing，不含 unknown） | ✅ |
| `EvidenceCollection` 字段一致（stage_id, evidence_items, index_by_path, index_by_kind, index_by_symbol, warnings, stats, version） | ✅ |
| `EvidenceStats` 字段一致（files_processed, files_skipped, total_items, items_by_kind, items_by_strength） | ✅ |
| claim confidence 明确限定为 Phase 3+ 概念，Phase 2 不涉及 | ✅ |
| `collect_evidence` 签名一致（`root_path, stage_id`，无 `State<AppState>`） | ✅ |
| ErrorCode 新增路径为 `src-tauri/src/models/enums.rs` | ✅ |
| 解析失败通过 `warnings[]` + `files_skipped` 表达，不作为 strength 值 | ✅ |
| 前端使用 `strength` / `STRENGTH_STYLE` / `items_by_strength`，无 confidence 残留 | ✅ |
| 安全边界（只读、无 Vivado、无外部进程）一致 | ✅ |

### 收口前修复记录

| 修复项 | 文档 | 变更 |
|--------|------|------|
| confidence → strength | 全部 6 份 Phase 2 文档 | 字段名统一为 `strength` |
| 移除 `unknown`/`Unknown` | 全部 6 份 Phase 2 文档 | `EvidenceStrength` 枚举不含 unknown |
| State\<AppState\> 移除 | `phase-2-evidence-collector-design.md` | command 签名移除 `State<AppState>` |
| ErrorCode 路径修正 | `phase-2-implementation-plan.md` | `models/mod.rs` → `models/enums.rs`（两处） |
| evidence-model line 249 | `phase-2-evidence-model.md` | `indirect confidence` → `indirect strength` |

### README 索引同步

| README 文件 | 更新内容 |
|-------------|----------|
| `docs/design/README.md` | Phase 2 文档 draft → active，备注收口状态 |
| `docs/requirements/README.md` | phase-2-evidence-requirements.md draft → active |
| `docs/ui-ux/README.md` | phase-2-evidence-view.md draft → active |
| `docs/testing/README.md` | phase-2-evidence-validation.md draft → active |
| `docs/planning/README.md` | phase-2-implementation-plan.md draft → active，状态文本更新 |

### 收口结论

Phase 2 编码依据文档已全部收口为 `active`，跨文档一致性已确认。可以进入 Phase 2 编码实施。编码必须遵守本文档第 5 节的任务拆解、第 6 节的编码顺序和第 7 节的验证要求。
