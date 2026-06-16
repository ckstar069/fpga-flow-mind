# Phase 8 UI 状态与导航设计

---
status: active
updated: 2026-06-16
---

> 本文档定义 Phase 8 工作台的**UI 状态机、导航结构、焦点切换与 UI state 持久化恢复**。它是 [`phase-8-workbench-architecture.md`](phase-8-workbench-architecture.md) 的状态/导航细化，落实需求 R8-002/R8-003/R8-004/R8-008。
>
> 本文 status 为 `active`，是 Phase 8 UI 状态与导航设计的生效文档。**当前仅允许进入 Phase 8 Batch A（P8-T01~P8-T02）**；Batch B/C/D/E 与 Phase 9/10/11 均未开始。Phase 8 编码尚未开始。

## 1. 设计目标

1. **显式焦点**：任一时刻工作台有明确焦点（项目概览 / 某阶段某分区），驱动主工作区渲染，替代同屏堆叠。
2. **导航一次可达**：最近项目、任一阶段、当前阶段功能区均一次点击可达。
3. **状态可恢复**：session 重载后回到原焦点（阶段 + 分区 + 选中节点）。
4. **切换隔离**：阶段切换 / 重新生成时，下游产物与异步响应正确隔离（延续 Phase 7 P2）。
5. **空/错误/加载态完整**：每种状态有明确表达与可操作下一步。

## 2. 工作台焦点状态机

### 2.1 焦点模型

```text
WorkspaceFocus = {
  mode: ProjectOverview | StageWorkspace,
  active_stage_id: StageId | null,
  stage_tab: Overview | Evidence | Understanding | Views | Trace | Qa | Quality,
}
```

- `mode=ProjectOverview`：主工作区渲染项目概览 + 最近项目；`active_stage_id=null`。
- `mode=StageWorkspace`：主工作区渲染某阶段的分区工作区；`active_stage_id` 非空。

### 2.2 焦点转移

| 触发 | 转移 |
|------|------|
| 打开项目 / 取消选中阶段 | → ProjectOverview |
| 在 LeftNav 点选阶段 | → StageWorkspace(stage, tab=该阶段持久化 tab 或 Overview) |
| 切换 stage_tab | mode 不变，stage_tab 变更 |
| 在 LeftNav 点"最近项目" | 切换项目 → 默认 ProjectOverview（或该项目最近焦点） |

### 2.3 分区内容状态

每个 `stage_tab` 分区内部有独立的内容状态机：

```text
SectionState = Empty | Loading | Ready(Degraded?) | Error(reason)
```

- `Empty`：该阶段无该维度产物（如未收集 evidence、timing 诚实空图）。
- `Loading`：异步请求进行中。
- `Ready`：产物就绪；若含退化项（孤立节点、expected_empty_timing）标记 `Degraded` 但仍属 Ready。
- `Error`：请求失败（source_missing / source_path_not_allowed / stage_empty 等）。

## 3. 左侧导航设计

### 3.1 结构

```text
LeftNav（深色固定）
 ├── 顶部：项目入口（当前项目名 + 切换/最近项目）
 ├── 中部：阶段列表
 │    ├── L0  [available]
 │    ├── L1  [available]
 │    ├── L2  [empty]
 │    ├── ... 
 │    └── RTL [available]
 └── 底部：当前阶段功能区快捷入口（当选中阶段时：Overview/Evidence/.../Quality）
```

### 3.2 阶段状态视觉标记

| StageStatus | 标记 | 可点选 |
|-------------|------|--------|
| available | 实心点（蓝） | 是 |
| empty | 空心点（灰） | 是（进入后诚实显示空） |
| missing | 虚线点（灰） | 是（显示缺失说明） |
| naming_anomaly | 警示点（琥珀） | 是（识别为阶段，可进入） |
| unreadable | 锁点（灰） | 否（显示原因） |

> 标记色不使用"通过/失败"红绿；琥珀仅表"命名异常需注意"，非裁决。

### 3.3 最近项目

- 项目入口展开"最近项目"列表（来自 Phase 6 session 列表）。
- 一次点击切换项目；切换前提示当前未保存状态（如有）。
- 空列表（无最近项目）显示引导打开项目。

## 4. 阶段工作区 Tab 状态

### 4.1 Tab 集合

`Overview | Evidence | Understanding | Views | Trace | Q&A | Quality`

- **Overview**：阶段概览（summary、关键指标、confidence 分布、quality acceptance、入口引导）。
- **Evidence**：evidence 列表（可展开、可按 source_kind/strength 筛选）。
- **Understanding**：claims / module / interface / signal summaries / unknowns / gaps。
- **Views**：三类视图（structure/dataflow/timing，SVG+CSS）+ 视图切换 + 退化提示。
- **Trace**：节点选中态 + source excerpt + evidence 高亮（复用 Phase 5 机制）。
- **Q&A**：grounded Q&A 历史 + 提问（MockProvider）。
- **Quality**：QualityReport summary + issue list（复用 Phase 7）。

### 4.2 跨 Tab 选中态保持

- 在某阶段内切换 Tab 时，保留"当前选中节点/对象"上下文（如 Views 选中 node → 切 Trace 仍定位该 node 的证据）。
- 切换阶段时清空选中态（焦点整体切换）。

### 4.3 Views 子切换

Views Tab 内部含 `structure | dataflow | timing` 子切换（延续 Phase 4 active view type）；该子切换可持久化进 `PersistedUiState`。

## 5. 加载 / 错误 / 空状态转换

### 5.1 全局横幅与就地状态

- **全局**：session 加载/保存状态、源码变更（source_changed）提示——用 WorkspaceTopBar 横幅。
- **就地**：分区级 Empty/Loading/Error 在该分区内表达，不挤占全局。

### 5.2 关键状态文案与操作（示例，最终以 UI/UX 文档为准）

| 状态 | 文案方向 | 可操作下一步 |
|------|----------|--------------|
| 未收集 evidence | "该阶段尚未收集证据" | "收集证据" 按钮 |
| 源码变更 | "源码已变更，理解产物可能过期" | "重新生成" 按钮 |
| 空阶段 | "该阶段为空（empty）" | 引导查看相邻阶段 |
| 命名异常 | "该阶段命名非标准（naming_anomaly），已识别" | 进入分区 |
| timing 诚实空 | "无 cycle/latency/clock 证据，未生成 timing 图" | 查看 dataflow / evidence |
| 路径不允许 | "源码路径不在允许范围" | 检查项目路径 |
| Q&A 无证据 | "证据不足，无法回答（unknown/gap）" | 查看证据缺口 |

> 所有文案表达"工具状态/不确定性"，不评价目标项目正确性。

## 6. UI state 持久化与恢复

### 6.1 PersistedUiState 扩展（展示性，不改语义）

在 Phase 6 `PersistedUiState` 基础上映射新焦点：

```text
PersistedUiState（Phase 8 映射）
 ├── active_stage_id        # 延续
 ├── stage_tab              # 新增：当前阶段分区（Overview/.../Quality）
 ├── active_view_type       # 延续（Views 子切换 structure/dataflow/timing）
 └── selected_node_id?      # 可选：跨 Tab 选中节点（仅当前阶段）
```

- 扩展字段为**新增可选字段**，不破坏 Phase 6 既有持久化兼容性（旧 session 缺字段时取默认）。
- 持久化仅写 app-owned storage。

### 6.2 恢复流程

1. 加载 session → 恢复 workspace profile + 各阶段产物索引。
2. 恢复 `PersistedUiState` → 设置焦点（mode / active_stage_id / stage_tab / active_view_type）。
3. 若恢复的 stage 已不存在（项目结构变化）→ 回退 ProjectOverview + 提示。
4. 异步加载该焦点所需产物，经 guard 过滤。

## 7. 交互状态

| 状态 | 处理 |
|------|------|
| 节点/对象选中 | 蓝色高亮 + 触发 Trace/详情联动 |
| hover | tooltip（节点摘要 / evidence 摘要），不遮挡 |
| 列表展开/折叠 | 记忆当前阶段内的展开态（瞬态，不持久化） |
| 对话框 | 打开时遮罩 + 焦点锁定；确认/取消明确 |
| 异步进行中 | 按钮/区域 loading 态，禁用重复触发 |
| 旧响应回写 | guard/version 过滤（延续 Phase 7 P2） |

## 8. 与 Phase 7 guard 模式衔接

- 延续 `qualityGuardRef / traceGuardRef / excerptGuardRef / qaGuardRef` 模式：每次新请求递增 guard，旧响应丢弃。
- 焦点切换 / 重新生成时，对应 downstream maps 清除 + guard 递增（见架构文档 §4.3）。
- guard 逻辑从 1300 行单组件迁移到 `WorkspaceContext` 的 action 层，语义不变。

## 9. 安全边界

- 导航与状态切换不触发对目标项目的写入。
- 不调用真实 LLM；不读取 `api_key`。
- 不运行 Vivado / synthesis / implementation / bitstream。
- UI state 持久化只写 app-owned storage。
- 文案不输出审计裁决；状态标记色不表达"目标项目通过/失败"。

## 10. 关联文档

- [`phase-8-workbench-architecture.md`](phase-8-workbench-architecture.md) — 工作台架构（draft）
- [`../ui-ux/phase-8-product-workbench-view.md`](../ui-ux/phase-8-product-workbench-view.md) — UI/UX 设计（draft）
- [`../requirements/phase-8-product-workbench-requirements.md`](../requirements/phase-8-product-workbench-requirements.md) — 需求（draft）
- [`../testing/phase-8-product-workbench-validation.md`](../testing/phase-8-product-workbench-validation.md) — 验证与验收（draft）
- [`../planning/phase-8-implementation-plan.md`](../planning/phase-8-implementation-plan.md) — 编码实施计划（active）

## 11. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-16 | 初始 draft：焦点状态机、左侧导航结构与阶段状态标记、阶段 Tab 状态、加载/错误/空转换、PersistedUiState 展示性扩展、交互状态、与 Phase 7 guard 衔接、安全边界。审核转 active 后方允许编码。 | Claude |
| 2026-06-16 | 审核通过，status 从 draft 转 active；允许进入 Phase 8 Batch A（P8-T01~P8-T02）；Phase 8 编码尚未开始；Batch B/C/D/E 与 Phase 9/10/11 未开始。 | Claude |
