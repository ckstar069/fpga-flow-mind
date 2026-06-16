# Phase 8 工作台架构设计

---
status: active
updated: 2026-06-16
---

> 本文档定义 Phase 8（产品级 UI/UX 工作台重构）的**前端架构设计**：如何把当前工程调试式单页长堆叠（`WorkspacePage.tsx` ≈ 1300 行）重构为产品级工作台的组件树、布局、路由与分层，以及如何在重构中守住"不破坏语义契约、不改分析能力、不引入重依赖"的边界。
>
> 本文档 status 为 `active`，是 Phase 8 前端架构设计的生效文档。**当前仅允许进入 Phase 8 Batch A（P8-T01~P8-T02）**；Batch B/C/D/E 与 Phase 9/10/11 均未开始。Phase 8 编码尚未开始。
>
> 本文是**前端为主**的架构文档；后端改动仅限"支撑新 UI 所需的最小展示性调整"（见 §6）。语义模型 / 分析能力 / 安全边界不变。

## 1. 设计原则

1. **前端为主，后端最小展示性调整**：Phase 8 是 UI 重构阶段，不借机改语义模型或分析能力。
2. **契约稳定**：`ImplementationUnderstanding`、evidence model、`confidence` 枚举、trace 模型、QualityReport 模型的字段语义与序列化契约不变；视觉与组织改变 ≠ 模型改变。
3. **渐进重构、零回归**：分 Batch 拆解，每批可独立验收，不破坏 Phase 4/5/6/7 已验收能力。
4. **不引入重依赖**：视图仍纯 SVG + CSS；状态管理用 React 内置能力 + 必要时轻量 context，不引入 Redux/Zustand 等除非详细论证。
5. **可持久化状态与瞬态状态分离**：session 可恢复的 UI state 与不可恢复的瞬态状态分层，避免持久化膨胀。
6. **可追溯不丢失**：重构后 evidence/claim/node 仍可一键追溯到 source_path + line_range。

## 2. 当前架构现状与问题

| 维度 | 现状 | 问题 |
|------|------|------|
| 顶层组件 | `WorkspacePage.tsx` ≈ 1300 行单组件 | 单组件持有所有状态与渲染，难维护、难聚焦 |
| 面板组织 | 15 个面板组件（EvidencePanel/TracePanel/MultiViewPanel/GroundedQAPanel/QualityReviewPanel/StageDetail...）按后端产出纵向堆叠 | 按"后端能力"而非"用户理解流程"组织 |
| 状态管理 | React state + guard refs（quality/trace/excerpt/qa guard）散落在单组件 | 状态耦合，阶段切换/重生成隔离靠手工 guard 维持 |
| 路由/焦点 | 无显式路由；当前阶段/视图靠内部 state | 缺乏"工作台焦点"概念，所有内容同屏 |
| 持久化 UI state | Phase 6 `PersistedUiState`（active stage、active view type 等） | 可复用，但需在新信息架构下重新映射 |

> 现状是 MVP 为快速验证链路的技术选择，Phase 8 在链路已验证、分析质量已提升（Phase 7）后，把它重构为可用工作台。

## 3. 目标架构

### 3.1 三段式骨架

```text
AppShell
 ├── LeftNav（深色固定导航）
 │    ├── 项目入口 + 最近项目
 │    ├── 阶段列表（StageStatus 标记）
 │    └── 当前阶段功能区入口
 └── MainWorkspace（浅色主工作区）
      ├── WorkspaceTopBar（项目/阶段标题、session 状态、关键操作）
      ├── StageWorkspace（当选中阶段时）
      │    ├── StageOverviewBar（顶部指标概览）
      │    ├── StageFilterBar（中部筛选/分组/视图切换）
      │    └── StageContentArea（分区内容：Overview/Evidence/Understanding/Views/Trace/Q&A/Quality）
      └── ProjectOverview（未选阶段时：项目概览 + 最近项目）
```

- **AppShell**：常驻布局壳，承载导航与主工作区切换；不持有业务产物，只持有"焦点"（当前项目 / 阶段 / 功能区）。
- **LeftNav**：深色固定导航，项目 / 阶段 / 功能三层可达。
- **MainWorkspace**：浅色主区，按焦点渲染 StageWorkspace 或 ProjectOverview。

### 3.2 组件树分解原则

- 按信息架构（导航 / 概览 / 筛选 / 内容分区）拆分容器组件，而非按后端产出。
- 每个分区容器（EvidenceSection / UnderstandingSection / ViewsSection / TraceSection / QaSection / QualitySection）聚焦一个维度，复用既有展示组件（EvidencePanel / UnderstandingPanel / MultiViewPanel / TracePanel / GroundedQAPanel / QualityReviewPanel 的渲染逻辑），但不再纵向同屏堆叠。
- 既有展示组件的**渲染逻辑尽量复用**，重构的是"组织与导航"，避免推倒重写导致回归。

### 3.3 路由 / 焦点模型

引入显式"工作台焦点"状态（不引入浏览器路由库，用应用内状态机）：

```text
WorkspaceFocus
 ├── mode: project_overview | stage_workspace
 ├── active_stage_id: StageId | null
 └── stage_tab: overview | evidence | understanding | views | trace | qa | quality
```

- 焦点状态驱动 MainWorkspace 渲染。
- `stage_tab` 替代"同屏堆叠所有维度"。
- 焦点状态可序列化进 `PersistedUiState`（见 §4），实现 session 恢复时回到原焦点。

## 4. 状态管理设计

### 4.1 状态分层

| 层 | 内容 | 持久化 | 载体 |
|----|------|--------|------|
| Session 产物状态 | workspace profile、各阶段 evidence/understanding/views/qa/quality 产物 | 是（app-owned，延续 Phase 6） | Tauri command 读取 + 前端缓存 |
| 可恢复 UI 状态 | active_stage_id、stage_tab、active view type、选中节点等 | 是（`PersistedUiState`） | `PersistedUiState`（Phase 6 扩展） |
| 瞬态 UI 状态 | 加载中、筛选条件、展开态、hover、对话框开关 | 否 | 组件 local state / context |
| 异步守卫状态 | quality/trace/excerpt/qa guard（防旧请求回写） | 否 | ref（延续 Phase 7 P2） |

### 4.2 状态承载

- **优先 React 内置**：`useReducer` / `useState` / `useContext`。
- 引入一个轻量 `WorkspaceContext` 承载"焦点 + 当前阶段产物缓存 + 操作动作"，替代 1300 行单组件内的散落 state。
- **不引入全局状态库**（Redux/Zustand 等），除非详细设计论证其必要性并审核；当前规模 React 内置 + context 足够。
- 异步 guard 延续 Phase 7 P2 的 ref 模式，确保阶段切换 / 重生成时旧响应不回写。

### 4.3 阶段切换 / 重生成的状态隔离

延续并强化 Phase 7 P2 状态隔离：

- 切换阶段：清空下游 maps（understanding / views / qa / quality），焦点切到新阶段的 `overview` tab（或恢复该阶段持久化的 tab）。
- 重新收集 evidence：清空 understanding / views / qa / quality（downstream）。
- 重新生成 understanding：清空 views / qa。
- 重新生成 views：清空 qa。
- 所有异步请求经 guard/version 过滤旧响应。

## 5. 与既有 Phase 1~7 产物对接

工作台**只读消费**下列既有产物（不修改其语义）：

| 既有产物 / 能力 | 来源 | 工作台对接 |
|-----------------|------|-----------|
| `WorkspaceProfile` / StageStatus | Phase 1 | LeftNav 阶段列表 + 状态标记 |
| `EvidenceCollection` | Phase 2 | Evidence 分区 + 概览指标 |
| `ImplementationUnderstanding` | Phase 3 | Understanding 分区 + confidence 视觉 |
| `ViewGraph[]`（structure/dataflow/timing） | Phase 4 | Views 分区（SVG + CSS 不变） |
| trace / source excerpt | Phase 5 | Trace 分区 + 节点点击回链 |
| Grounded Q&A | Phase 5 | Q&A 分区 |
| session 持久化 / `PersistedUiState` | Phase 6 | session 恢复 + 焦点恢复 |
| `QualityReport` / QualityIssue | Phase 7 | Quality 分区 + 概览 acceptance |

- 既有 Tauri commands（open_workspace / select_stage / collect_evidence / generate_understanding / generate_views / resolve_trace_target / get_source_excerpt / ask_grounded_question / generate_quality_report / save_session / load_session 等）**语义不变**，工作台直接调用。
- 工作台不重新扫描 / 收集 / 生成语义判断，只组织展示与引导工作流。

## 6. 展示性 command 调整边界

Phase 8 **允许**对 Tauri command 做**展示性**返回结构调整，**仅当**：

1. 不改变语义模型字段；
2. 不破坏既有序列化契约与持久化兼容性；
3. 不改变安全边界；
4. 改动可追溯并在此文档（或实施计划）记录。

候选展示性调整（实施时按需收敛，非强制）：

- 为"顶部指标概览"提供聚合统计（如 stage 维度计数），可由前端从既有产物计算，**优先前端计算**，仅在性能必要时新增只读聚合 command。
- 禁止为 UI 便利而改动 evidence / understanding / view / qa / quality 的字段语义。

> 默认倾向：**前端从既有产物派生展示数据**，新增 command 是最后手段。

## 7. 依赖与图形库论证

| 项 | 决策 | 理由 |
|----|------|------|
| 视图渲染 | **保持纯 SVG + CSS**（延续 Phase 4） | 三类视图规模可控，SVG+CSS 足够；引入 React Flow/D3/Mermaid 偏离技术栈约束且增回归风险 |
| 状态管理 | **React 内置 + 轻量 context** | 当前规模无需全局状态库；引入需额外论证 |
| 路由 | **应用内焦点状态机**（不引入路由库） | 桌面单页应用，焦点状态足够；浏览器路由无收益 |
| UI 组件库 | **审慎**：可用轻量无样式/原子样式自建卡片与导航，**不引入重型 UI 框架**（如 Ant Design / MUI 全家桶）除非论证 | 重型框架带来体积与风格不可控；AgentScope 风格用自建卡片+蓝色强调更可控 |

> 若实施中发现某项确有必要引入，须在实施计划 / 变更记录论证并审核，不得 silent 引入。

## 8. 性能与可维护性

- `WorkspacePage` 拆分后，单组件行数显著下降，焦点清晰。
- 阶段产物按需加载（选中阶段才读取该阶段产物），避免一次性持有全项目产物。
- 视图渲染保持 Phase 4 既有性能策略（grid 布局 + layout_hints，不做自动布局引擎）。
- 重构配套类型检查（`tsc --noEmit`）与构建（`npm run build`）作为每批验收门槛。

## 9. 安全边界

- 工作台不引入对目标项目的写入；所有追溯点击只读定位。
- 不调用真实 LLM；不读取 `api_key`。
- 不运行 Vivado / synthesis / implementation / bitstream。
- 持久化只写 app-owned storage（延续 Phase 6）。
- 文案不输出审计裁决；视觉语义不表达"目标项目通过/失败"。
- 不引入外部网络库 / 重依赖偏离技术栈。

## 10. 关联文档

- [`phase-8-ui-state-and-navigation-design.md`](phase-8-ui-state-and-navigation-design.md) — UI 状态与导航设计（draft）
- [`../ui-ux/phase-8-product-workbench-view.md`](../ui-ux/phase-8-product-workbench-view.md) — 工作台 UI/UX 设计（draft）
- [`../requirements/phase-8-product-workbench-requirements.md`](../requirements/phase-8-product-workbench-requirements.md) — 需求（draft）
- [`../testing/phase-8-product-workbench-validation.md`](../testing/phase-8-product-workbench-validation.md) — 验证与验收（draft）
- [`../planning/phase-8-implementation-plan.md`](../planning/phase-8-implementation-plan.md) — 编码实施计划（draft）

## 11. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-16 | 初始 draft：三段式骨架、组件树分解、焦点路由模型、状态分层、与 Phase 1~7 产物只读对接、展示性 command 边界、依赖/图形库论证、安全边界。审核转 active 后方允许编码。 | Claude |
| 2026-06-16 | 审核通过，status 从 draft 转 active；允许进入 Phase 8 Batch A（P8-T01~P8-T02）；Phase 8 编码尚未开始；Batch B/C/D/E 与 Phase 9/10/11 未开始。 | Claude |
