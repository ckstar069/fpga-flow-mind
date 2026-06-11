# Phase 1 Workspace 与阶段选择 UI/UX 轻量设计

---
status: draft
updated: 2026-06-11
---

> 本文档是 Phase 1 的 UI/UX 轻量设计，覆盖 workspace 打开、概览展示、阶段选择、阶段概览的界面交互规则。
> Phase 1 指 `fpga-flow-mind` **本项目**的开发推进阶段，不是业务项目的 `L0` / `L1` / `RTL` 实现阶段。
> 不写高保真视觉稿、CSS 或组件代码。

## 1. 设计目标

Phase 1 UI/UX 只覆盖：

- 选择 workspace 目录
- 展示 workspace 概览（名称、路径、文件统计、外部引用）
- 展示阶段列表及状态
- 展示 warnings / errors
- 选择单个阶段
- 展示阶段概览（文件列表、外部依赖、上游引用）
- "开始分析"按钮作为禁用或占位

Phase 1 UI/UX **不覆盖**：

- evidence 面板（Phase 2）
- 结构图 / 数据流图 / 时序图（Phase 4）
- grounded Q&A（Phase 5）
- 持久化回放 UI（Phase 6）
- 高保真视觉设计、CSS 规范、动效

## 2. 需求与设计来源

| 来源文档 | 本文档引用内容 |
|---------|-------------|
| [`story-open-workspace.md`](../requirements/stories/story-open-workspace.md) | WS-001~007 功能点与异常状态 |
| [`story-select-stage.md`](../requirements/stories/story-select-stage.md) | ST-001~008 功能点与空状态处理 |
| [`mvp-functional-contract.md`](../requirements/mvp-functional-contract.md) | 枚举值、错误码、对象字段 |
| [`phase-1-architecture.md`](../design/phase-1-architecture.md) | 前端模块划分与数据流 |
| [`phase-1-data-and-api-contract.md`](../design/phase-1-data-and-api-contract.md) | TypeScript 类型与 CommandResult 语义 |
| [`phase-1-scanner-detail-design.md`](../design/phase-1-scanner-detail-design.md) | 边界条件与 validity 判定规则 |

## 3. 用户流程

```text
1. 用户打开应用 → 看到初始欢迎状态
2. 点击"打开项目" → 弹出系统文件选择器
3. 选择目录（或取消）
4. 系统扫描 → loading 状态
5. 返回 WorkspaceProfile → 展示 workspace 概览
   5a. success=false → ErrorBanner 展示错误，允许重新选择
   5b. success=true + warnings → 正常展示 + WarningList 展示非致命问题
6. 用户查看阶段列表、文件统计、warnings
7. 用户点击一个阶段 → loading 状态
8. 返回 StageContext → 展示阶段概览
   8a. 阶段为空 → 空状态提示
   8b. 阶段不可读 → ErrorBanner
9. 用户查看阶段文件列表、外部依赖、上游引用
10. "开始分析"按钮保持禁用或占位，不触发 evidence 收集
```

## 4. 页面信息架构

```text
┌─────────────────────────────────────────────────────────┐
│  顶部工具栏：[打开项目] | workspace 名称 | validity 标识  │
├──────────────────────┬──────────────────────────────────┤
│                      │                                  │
│  workspace 概览      │  阶段概览 / 详情                  │
│  · 根路径            │  · 阶段名称 + 路径                │
│  · 文件类型统计      │  · 文件列表（按类型分组）          │
│  · 外部引用标识      │  · 外部依赖                       │
│                      │  · 上游引用                       │
│  阶段列表            │                                  │
│  · L0  (available)   │  ──────────────────              │
│  · L1  (available)   │  [开始分析]（禁用/占位）          │
│  · L2  (empty, 警告) │                                  │
│  · RTL (命名异常)    │                                  │
│                      │                                  │
├──────────────────────┴──────────────────────────────────┤
│  warnings / errors 面板（可折叠）                        │
│  · file_too_large: xxx.v 超过 5MB                       │
│  · 缺失阶段: L3 未找到                                  │
└─────────────────────────────────────────────────────────┘
```

**区域说明**：
- **顶部工具栏**：固定区域，包含打开项目入口、当前 workspace 名称和路径、validity 状态标识
- **左栏**：workspace 概览信息 + 阶段列表（上下排列）
- **右栏**：阶段概览详情（选中阶段后展示）
- **底部**：warnings / errors 面板，可折叠展开

## 5. 核心组件

### WorkspacePage
- **输入**：`CommandResult<WorkspaceProfile>`
- **显示**：整体页面布局，协调子组件
- **交互**：无直接交互，由子组件承载
- **空状态**：初始未打开项目时显示欢迎引导

### OpenWorkspaceButton
- **输入**：无
- **显示**："打开项目"按钮文案
- **交互**：点击触发 Tauri 文件选择器；扫描中变为 loading 态
- **空状态**：始终可见

### WorkspaceSummary
- **输入**：`WorkspaceProfile`
- **显示**：workspace 名称、根路径、validity 标识、外部引用数量
- **交互**：无
- **空状态**：workspace 未打开时不展示

### FileTypeStats
- **输入**：`WorkspaceProfile.file_type_stats`
- **显示**：按扩展名统计的文件数量列表（如 .py: 12, .v: 8, .md: 3）
- **交互**：无
- **空状态**：无文件时显示"未扫描到可识别文件"

### StagePanel
- **输入**：`WorkspaceProfile.stages[]`
- **显示**：阶段列表容器，按排序规则展示子项
- **交互**：管理单选状态
- **空状态**：`stages[]` 为空时显示"未识别到阶段目录"

### StageListItem
- **输入**：`StageSummary`（单个阶段）
- **显示**：阶段名称、文件数量、状态标识（图标或标签）
- **交互**：点击选中（仅 `available` / `naming_anomaly` 可选中）；empty/unreadable 点击提示原因
- **空状态**：不适用

### StageOverview
- **输入**：`CommandResult<StageContext>`
- **显示**：阶段名称、路径、文件列表（按 source_kind 分组）、外部依赖、上游引用
- **交互**：无直接交互
- **空状态**：未选中阶段时显示"请从左侧选择一个阶段"；`files[]` 为空时显示"该阶段无文件"

### WarningList
- **输入**：`WorkspaceWarning[]`
- **显示**：warning 条目列表，每条含图标 + error_code + message + source_path
- **交互**：可折叠/展开
- **空状态**：无 warnings 时折叠或不展示

### ErrorBanner
- **输入**：`CommandError`
- **显示**：错误文案 + 错误码 + 允许重新选择按钮
- **交互**：点击"重新选择"触发文件选择器
- **空状态**：无错误时不展示

### AnalysisPlaceholderButton
- **输入**：`StageContext`（可选）
- **显示**："开始分析"按钮，灰显或标注"Phase 2 后可用"
- **交互**：点击无响应或显示"功能开发中"提示
- **空状态**：未选中阶段时不展示

## 6. 阶段状态展示规则

| status | 文案 | 可点击 | 可选中 | 视觉标识 |
|--------|------|--------|--------|---------|
| `available` | 阶段名 + 文件数 | 是 | 是 | 正常样式 |
| `empty` | 阶段名 + "为空" | 是（提示原因） | 否 | 灰色降级 + 警告图标 |
| `naming_anomaly` | 阶段名 + "命名异常" | 是 | 是 | 黄色/橙色标签提示 |
| `unreadable` | 阶段名 + "不可读" | 是（提示原因） | 否 | 灰色禁用 + 锁定图标 |
| `missing` | 不在阶段列表中展示 | — | — | 仅在 warnings 区域展示"未找到" |

> `missing` 不作为阶段列表条目。缺失阶段信息通过 `warnings[]` 和 `validity_reasons[]` 在 WarningList 中展示。

**图标规范**：实现时优先使用项目选定图标库中的语义图标（如 `warning`、`lock`），不以 emoji 作为正式 UI 图标规范。

## 7. validity / warnings / error_codes 展示规则

### validity 展示（WorkspaceSummary 中）

| validity | 文案 | 颜色语义 | 强制继续 |
|----------|------|---------|---------|
| `likely_valid` | "项目结构符合预期" | 绿色/正常 | 不需要 |
| `uncertain` | "项目结构部分匹配，阶段可能不完整" | 黄色/橙色 | 可提供"继续"入口 |
| `unlikely` | "项目结构不符合预期模板" | 红色/警告 | 可提供"强制继续"入口 |

### 错误码 → 展示组件映射

| error_code | 展示组件 | 阻塞范围 | 可继续操作 | 用户操作 |
|-----------|---------|---------|-----------|---------|
| `path_not_found` | ErrorBanner | 全局（workspace 级） | 否 | 重新选择目录 |
| `not_directory` | ErrorBanner | 全局（workspace 级） | 否 | 重新选择目录 |
| `permission_denied` | ErrorBanner | 全局（workspace 级） | 否 | 重新选择目录 |
| `stage_unreadable` | 阶段级错误提示 | 仅该阶段 | 是（可选择其他阶段） | 选择其他阶段；workspace 概览和其他阶段不受影响 |
| `no_stage_found` | WarningList + 提示横幅 | 不阻塞 | 是（可强制继续） | 可强制继续浏览 |
| `stage_empty` | WarningList | 仅该阶段 | 是 | 不可进入分析；可选择其他阶段或查看空状态说明 |
| `file_unreadable` | WarningList | 不阻塞 | — | 仅展示 |
| `file_too_large` | WarningList | 不阻塞 | — | 仅展示 |
| `scan_timeout` | WarningList | 不阻塞 | — | 仅展示 |

> `stage_unreadable` 是阶段级错误，只阻断该阶段的读取，不阻断 workspace 概览展示和其他阶段选择。展示位置为阶段详情区域或阶段列表中的状态提示，不作为全局 ErrorBanner。

## 8. 空状态与边界状态

| 状态 | 场景 | 展示方式 |
|------|------|---------|
| 初始未打开 | 应用刚启动 | 欢迎页："点击"打开项目"选择一个 FPGA 业务项目目录" |
| 用户取消选择 | 文件选择器取消 | 保持当前状态不变，不报错 |
| 空目录 | 无阶段、无代码、无文档 | validity=`unlikely` + WarningList 展示 `no_stage_found` + "目录为空或不是业务项目" |
| 无阶段但有代码 | 有 .py/.v 但无阶段目录 | validity=`uncertain` + WarningList 展示 `no_stage_found` + "存在代码文件但未识别到标准阶段" |
| 阶段缺失 | 部分标准阶段未找到 | WarningList 展示每个缺失阶段 + "未找到阶段 X" |
| 命名异常阶段 | `rtl_final` 等非标准名 | 阶段列表中标注"命名异常"标签，可正常点击选择 |
| 空阶段 | 阶段目录无文件 | 灰色降级 + "为空"标签，点击提示原因 |
| 不可读阶段 | 权限不足 | 灰色禁用 + "不可读"标签，点击提示原因 |
| 大目录截断 | 单目录 >1000 文件 | WarningList 展示扫描范围 warning + "单目录文件数量超过上限，部分文件未展示或已跳过" |
| 扫描超时 | >30 秒 | WarningList 展示 `scan_timeout` + "扫描超时，已返回部分结果" |
| 仅 Python 无 RTL | 早期阶段项目 | validity=`likely_valid` + 正常展示 |
| 仅 RTL 无 Python | 硬件阶段项目 | validity=`likely_valid` + 正常展示 |
| 仅文档无代码 | 仅有 .md/.txt 等文档文件，无可分析源码 | 按技术设计规则降级展示（有阶段目录则 `uncertain`，无阶段目录则 `unlikely`）；UI 提示"未发现可分析源码"，不展示 JSON dump |

## 9. 不展示原始 JSON 的规则

- `workspace_profile.json` 和 `stage_context.json` 是**系统内部数据对象**，不直接作为用户主界面内容展示
- UI 应将结构化数据**提炼**为：名称、路径、阶段列表、文件统计、状态标识、warning/error 文案
- **不允许**在 MVP 用户界面中嵌入 JSON viewer 或原始数据面板
- **不生成** Markdown 格式的分析报告
- 开发调试阶段可在开发工具中查看结构化数据，但不应暴露给终端用户
- 文件类型统计展示为可读的列表或简单图表，不展示 JSON key-value

## 10. 与后续 Phase 的衔接

| 当前 Phase 1 | 后续 Phase |
|-------------|-----------|
| "开始分析"按钮禁用或占位 | Phase 2 后激活，触发 evidence 收集 |
| 阶段概览仅展示文件列表 | Phase 2 后增加 evidence 索引面板 |
| 无图形视图 | Phase 4 后增加结构图 / 数据流图 / 时序图 |
| 无问答交互 | Phase 5 后增加 grounded Q&A 面板 |
| 无持久化 UI | Phase 6 后增加历史项目列表和加载 |

Phase 1 的组件结构（WorkspacePage / StagePanel / WarningList 等）应在设计上**预留扩展空间**，但不提前实现后续功能。

## 11. Phase 1 UI/UX 验收标准

- [ ] 用户能清楚知道当前打开的是哪个 workspace（名称 + 路径可见）
- [ ] 用户能看到阶段列表，每个阶段有状态标识
- [ ] 用户能理解 warnings / errors（文案清晰，不暴露技术细节）
- [ ] 用户能选择一个可用阶段并查看其概览
- [ ] 空状态不表现为崩溃、空白页或 JSON dump
- [ ] 不展示原始 JSON 或 Markdown report
- [ ] 不暗示已进行 evidence 收集或语义分析
- [ ] 不提供会修改目标项目的操作按钮
- [ ] 所有面向用户的文案为简体中文
- [ ] validity 状态有视觉区分（颜色或图标）
- [ ] loading 状态有明确反馈（不出现假死）
