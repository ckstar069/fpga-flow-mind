# Phase 8 编码实施计划

---
status: active
updated: 2026-06-16
---

> 本文档定义 Phase 8（产品级 UI/UX 工作台重构）的编码实施计划：任务拆解（P8-T01~P8-T10）、依赖关系、Batch 划分（A~E）、进入/退出条件、安全边界。
>
> Phase 8 是**前端为主**的阶段：把 Phase 1~7 已有能力重组为产品级工作台（**工作台化，不是新增更多卡片**），**不新增语义分析能力，不接真实 LLM，不做跨阶段映射**。
>
> 本文档 status 为 `active`，是 Phase 8 编码实施计划的生效文档。**Phase 8 Batch A（P8-T01~P8-T02）已完成/进入审核收口**；**Phase 8 Batch B（P8-T03~P8-T04）已实现/进入审核收口**；**Phase 8 Batch C（P8-T05~P8-T06）已实现/进入审核收口**；Batch D/E 与 Phase 9/10/11 均未开始。Phase 7 已完成（completion review active）。

## 0. 核心重构方向（先读）

Phase 8 不是普通 UI 改造，而是**信息架构工作台化**。三个关键设计对象贯穿全部 Batch：

- **Artifact tabs**：把 Evidence / Understanding / Views / Trace / Q&A / Quality 等现有产物从**调试式长页面堆叠**迁移到 tabs / section 模式，**功能等价**（不丢能力、不改语义），只是不再全部纵向堆在一个长页面里。
- **ContextPanel**：右侧上下文面板，承载当前选中的 evidence / trace / source excerpt / quality issue；阶段切换与产物重生成必须清空旧上下文。
- **warnings 降噪**：scan_timeout 等警告**折叠 / 分类 / 计数 / 可展开查看**，不再长期占据底部。

视觉参考 AgentScope 的信息密度与布局方式（深色左侧导航、浅色密集主工作区、顶部概览指标、对象列表 / 分组、蓝色强调），**不照搬其品牌或无关功能**。

## 1. 进入条件

| 条件 | 当前状态 |
|------|----------|
| Phase 7 completion review 已完成 | ✅（active） |
| Phase 8 overview 已编制 | ✅ `phase-8-overview-product-ui-workbench.md`（draft） |
| Phase 8 需求文档 active | ⏳ `phase-8-product-workbench-requirements.md`（draft，待审核转 active） |
| Phase 8 架构设计 active | ⏳ `phase-8-workbench-architecture.md`（draft，待审核转 active） |
| Phase 8 UI 状态/导航设计 active | ⏳ `phase-8-ui-state-and-navigation-design.md`（draft，待审核转 active） |
| Phase 8 UI/UX 设计 active | ⏳ `phase-8-product-workbench-view.md`（draft，待审核转 active） |
| Phase 8 验证文档 active | ⏳ `phase-8-product-workbench-validation.md`（draft，待审核转 active） |
| Phase 8 实施计划 active | ⏳ 本文档（draft，待审核转 active） |
| **以上 Phase 8 详细文档全部转为 active** | ⏳ 待审核（当前 draft 已完成，正在审核收口） |

> 纪律：Phase 8 详细文档全部审核转 active 后，方允许进入 Batch A 编码。**Phase 8 Batch A（P8-T01~P8-T02）已实现/进入审核收口**；**Phase 8 Batch B（P8-T03~P8-T04）已实现/进入审核收口**；**Phase 8 Batch C（P8-T05~P8-T06）已实现/进入审核收口**；Batch D/E 与 Phase 9/10/11 均未开始。

## 2. 任务拆分

### P8-T01 AppShell + LeftNav + Header + 焦点状态机（Batch A）

| 维度 | 说明 |
|------|------|
| **目标** | 工作台骨架：AppShell + 深色 LeftNav（项目/阶段/功能）+ Header（WorkspaceTopBar：项目/阶段标题、session 状态、主操作）+ WorkspaceFocus 状态机（mode/active_stage_id/stage_tab） |
| **输入文档** | 架构 §3、UI 状态 §2~§3、UI/UX §3~§4 |
| **预计修改文件** | `src/features/workspace/`（新增 AppShell/LeftNav/Header + 焦点 context，拆分 WorkspacePage） |
| **验收命令** | `npm run build` + `npx tsc --noEmit` + 组件渲染检查 |
| **不做什么** | 不改后端；不引入路由库；不破坏既有面板渲染逻辑 |

### P8-T02 StageWorkspace 骨架 + 占位容器（Batch A）

| 维度 | 说明 |
|------|------|
| **目标** | 选中阶段后 MainWorkspace 渲染 StageWorkspace 三段式骨架（OverviewBar + FilterBar + ContentArea），并为 Artifact tabs 与 ContextPanel 预留占位容器 |
| **输入文档** | 架构 §3.2、UI 状态 §4、UI/UX §3/§5 |
| **预计修改文件** | `src/features/workspace/`（StageWorkspace 容器） |
| **验收命令** | `npm run build` + 骨架渲染检查 |
| **不做什么** | 本任务只搭骨架与占位，不迁移产物内容（B/C 才迁移） |

### P8-T03 Artifact tabs 重组（Batch B）

| 维度 | 说明 |
|------|------|
| **目标** | 将 Evidence / Understanding / Views / Trace / Q&A / Quality 等现有产物从长页面堆叠迁移到 Artifact tabs / section 模式，**功能等价**（复用既有展示组件渲染逻辑，不丢能力、不改语义），只是不再全部纵向堆叠 |
| **输入文档** | 需求 R8-003/§4、UI/UX §3/§5、架构 §3.2 |
| **预计修改文件** | `src/features/workspace/`（Artifact tabs 容器 + 各 Section 复用既有展示组件） |
| **验收命令** | `npm run build` + 桌面验收 tab 切换 + 功能等价回归 |
| **不做什么** | 不重写既有展示组件的渲染逻辑；不借迁移改语义/契约 |

### P8-T04 顶部概览 + 中部筛选 + 可展开对象列表（Batch B）

| 维度 | 说明 |
|------|------|
| **目标** | StageOverviewBar（指标概览）+ StageFilterBar（筛选/分组/视图切换）+ 可展开 evidence/claim/node/issue 列表 |
| **输入文档** | 需求 R8-005、UI/UX §5~§6 |
| **预计修改文件** | `src/features/workspace/`（概览条/筛选条/可展开列表组件） |
| **验收命令** | `npm run build` + 筛选/展开验收 |
| **不做什么** | 指标仅内部质量/规模，不对目标项目评价；不新增 command 除非必要 |

### P8-T05 ContextPanel 选中上下文（Batch C）

| 维度 | 说明 |
|------|------|
| **目标** | 右侧 ContextPanel 承载当前选中的 evidence / trace / source excerpt / quality issue；点击 Artifact tabs 内对象 → ContextPanel 联动展示详情，不再让质量报告/trace/source excerpt/Q&A 全部向下堆叠 |
| **输入文档** | 需求 §4、UI 状态 §4.2/§7、UI/UX §3/§6 |
| **预计修改文件** | `src/features/workspace/`（ContextPanel + 选中态联动） |
| **验收命令** | `npm run build` + 选中联动验收 |
| **不做什么** | 不改 trace/source excerpt 后端语义；ContextPanel 只读展示 |

### P8-T06 状态隔离：阶段切换 / 重生成清空上下文（Batch C）

| 维度 | 说明 |
|------|------|
| **目标** | 阶段切换与产物重生成必须清空 ContextPanel 选中态 + downstream maps（understanding/views/qa/quality）+ guard 过滤旧异步响应（延续 Phase 7 P2） |
| **输入文档** | 需求 R8-004、架构 §4.3、UI 状态 §8 |
| **预计修改文件** | `src/features/workspace/`（context action 层 + 工作流引导） |
| **验收命令** | `npm run build` + 状态隔离验收 |
| **不做什么** | 不改后端 command 语义；不做全自动流水线 |

### P8-T07 视觉 polish：AgentScope 风格（Batch D）

| 维度 | 说明 |
|------|------|
| **目标** | 视觉 polish 落地 AgentScope 风格：深色左侧导航、浅色密集主工作区、顶部概览指标、对象列表/分组、蓝色强调、卡片化；高信息密度但层级清晰 |
| **输入文档** | 需求 R8-006/R8-007、UI/UX §2/§6~§8 |
| **预计修改文件** | `src/features/workspace/`（卡片/标签/视觉语义/样式） |
| **验收命令** | `npm run build` + 视觉语义一致性检查 |
| **不做什么** | 不用红绿裁决色；不引图形库；不照搬 AgentScope 品牌/无关功能 |

### P8-T08 warnings 降噪（Batch D）

| 维度 | 说明 |
|------|------|
| **目标** | scan_timeout 等警告折叠 / 分类 / 计数 / 可展开查看，不再长期占据底部横幅；按严重度/类别分组，默认折叠，展开看详情 |
| **输入文档** | 需求 §4/§5、UI/UX §10 |
| **预计修改文件** | `src/features/workspace/`（warnings 聚合/折叠组件，替代底部长期堆叠） |
| **验收命令** | `npm run build` + warnings 折叠/计数验收 |
| **不做什么** | 不删除既有 warnings 数据；只改呈现与降噪 |

### P8-T09 空/错误/session/UI state 恢复（Batch D）

| 维度 | 说明 |
|------|------|
| **目标** | 空/错误/加载状态产品化（空阶段/命名异常/源码变更/路径不允许/timing 诚实空/Q&A 无证据/session 加载失败）+ 最近项目/项目切换 + PersistedUiState 展示性扩展（stage_tab/selected_node_id）+ 焦点恢复 |
| **输入文档** | 需求 R8-002/R8-008、UI 状态 §5~§6、UI/UX §10 |
| **预计修改文件** | `src/features/workspace/`、`src/types/`（UI state 类型扩展，向后兼容） |
| **验收命令** | `npm run build` + session 恢复验收 |
| **不做什么** | 不破坏 Phase 6 持久化兼容性 |

### P8-T10 回归 + 桌面验收 + completion review（Batch E）

| 维度 | 说明 |
|------|------|
| **目标** | 既有能力（三类视图/trace/QA/quality/持久化/真实项目识别）零回归 + 真实项目（主/副样本）全流程无说明完成 + completion review |
| **输入文档** | 测试 §3/§4/§6/§8 |
| **预计修改文件** | `docs/planning/phase-8-completion-review.md`（新增）、各 index 更新 |
| **验收命令** | `npm run build` + `npx tsc --noEmit` + `cargo test --lib` + `real_project_validation --ignored` + 桌面验收 12 步 + checksum + rg |
| **不做什么** | 不进入 Phase 9 编码 |

## 3. 依赖关系

```text
Batch A：骨架
  P8-T01 (AppShell + LeftNav + Header + 焦点)
    └── P8-T02 (StageWorkspace 骨架 + 占位)
          │
Batch B：Artifact tabs 重组
          ├── P8-T03 (Artifact tabs 迁移，功能等价)
          └── P8-T04 (概览 + 筛选 + 可展开列表)
                │
Batch C：ContextPanel + 状态隔离
          ├── P8-T05 (ContextPanel 选中上下文)
          └── P8-T06 (阶段切换/重生成清空)
                │
Batch D：视觉 polish + warnings 降噪
          ├── P8-T07 (AgentScope 视觉 polish)
          ├── P8-T08 (warnings 折叠/分类/计数)
          └── P8-T09 (空/错误/session/恢复)
                │
Batch E：验收 + completion
          └── P8-T10 (回归 + 桌面验收 + completion review)
```

## 4. Batch 划分

### 4.1 Batch A：AppShell / LeftNav / Header / StageWorkspace 骨架

| 任务 | 内容 |
|------|------|
| P8-T01 | AppShell + 深色 LeftNav + Header + 焦点状态机 |
| P8-T02 | StageWorkspace 三段式骨架 + Artifact tabs/ContextPanel 占位容器 |

**目标**：建立工作台信息架构骨架，替代单页长堆叠的容器基础。
**允许范围**：搭骨架与焦点路由，复用既有展示组件，不重写渲染逻辑。
**禁止越界**：不改后端；不引入路由库/状态库；不破坏既有面板；不迁移产物内容（留给 B）。

### 4.2 Batch B：Artifact tabs 重组

| 任务 | 内容 |
|------|------|
| P8-T03 | Evidence/Understanding/Views/Trace/Q&A/Quality 从长页面堆叠迁移到 Artifact tabs/section，功能等价 |
| P8-T04 | 顶部概览指标 + 中部筛选/分组/视图切换 + 可展开对象列表 |

**目标**：把现有产物从调试式长堆叠迁到 tabs/section，**功能等价**（不丢能力、不改语义），只是不再全部纵向堆在一个长页面里。
**允许范围**：复用既有展示组件组织进 tabs；加概览/筛选/可展开列表。
**禁止越界**：不重写渲染逻辑；不借迁移改语义/契约；不用红绿裁决色；不引图形库。

### 4.3 Batch C：ContextPanel + 状态隔离

| 任务 | 内容 |
|------|------|
| P8-T05 | 右侧 ContextPanel 承载选中的 evidence/trace/source excerpt/quality issue |
| P8-T06 | 阶段切换/产物重生成清空 ContextPanel + downstream + guard |

**目标**：质量报告/trace/source excerpt/Q&A 不再向下堆叠，选中对象进入右侧 ContextPanel；阶段切换与重生成必须清空旧上下文。
**允许范围**：ContextPanel + 选中联动 + 状态隔离迁移到 context action 层。
**禁止越界**：不改 trace/source excerpt 后端语义；ContextPanel 只读；不做全自动流水线。

### 4.4 Batch D：视觉 polish + warnings 降噪

| 任务 | 内容 |
|------|------|
| P8-T07 | AgentScope 风格视觉 polish（深色导航/浅色密集主区/顶部概览/对象列表分组/蓝色强调/卡片） |
| P8-T08 | warnings 折叠/分类/计数/可展开，不再长期占据底部 |
| P8-T09 | 空/错误/session/UI state 恢复产品化 |

**目标**：参考 AgentScope 信息密度与布局 polish 视觉；warnings 降噪不再喧宾夺主；产品级状态体验。
**允许范围**：视觉系统应用 + warnings 聚合折叠 + 状态体验 + 持久化兼容扩展。
**禁止越界**：不照搬 AgentScope 品牌/无关功能；不用红绿裁决色；不引图形库；不破坏 Phase 6 持久化兼容。

### 4.5 Batch E：桌面验收与 completion review

| 任务 | 内容 |
|------|------|
| P8-T10 | 既有能力零回归 + 真实桌面可用性验收 + completion review |

**目标**：全量验证、文档同步、完成审查。
**允许范围**：回归修复（不改语义）+ 真实样本桌面验收 + checksum 只读 + completion review。
**禁止越界**：不修改目标项目；不运行 Vivado/synthesis/implementation/bitstream；不进入 Phase 9 编码。

## 5. 退出条件

| 条件 | 验证方式 |
|------|----------|
| 前端构建 + 类型通过 | `npm run build` + `npx tsc --noEmit` |
| 后端契约不破坏 | `cargo test --lib` + `cargo check` |
| Artifact tabs 迁移功能等价 | tab 切换 + 既有能力回归 |
| ContextPanel + 状态隔离生效 | 选中联动 + 切换/重生成清空 |
| warnings 降噪 | 折叠/分类/计数验收 |
| AgentScope 风格落地 | 可用性验收 |
| 既有能力零回归 | 回归 + 桌面验收 |
| 真实用户无说明完成全流程 | 桌面验收 |
| 目标项目只读 | checksum + rg |
| 无真实 LLM / 无审计用语 / 无重库 | rg |
| Phase 8 completion review 完成 | 文档（转 active） |

## 6. 安全边界（权威，各 Batch 不重复列举）

- 目标项目只读：不修改目标项目，checksum 一致。
- 不运行 Vivado / synthesis / implementation / bitstream。
- 不调用真实 LLM（不读取 `api_key`、不调用 OpenAI / Anthropic）。
- 持久化只写 app-owned storage。
- 不破坏核心语义契约（`ImplementationUnderstanding`/confidence 枚举/evidence/view/trace/quality 模型字段语义稳定）。
- 不引入重可视化库（视图 SVG+CSS，不引 React Flow/D3/Mermaid）。
- 不输出 PASS/HOLD/正确性裁决/审计结论。

> 各 Batch 的"禁止越界"仅列该 Batch 特定约束；通用安全边界以本节为准，不重复。

## 7. 进入 Phase 9 的条件（预留）

- Phase 8 completion review 转 active。
- 真实桌面可用性验收通过。
- 既有能力零回归。
- 全量测试通过，安全约束满足。
- Phase 9 详细文档 active 后方可进入 Phase 9 编码。
- 不得在 Phase 8 未完成时启动 Phase 9/10/11 编码。

## 8. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-16 | 初始 draft：定义 P8-T01~P8-T10、5 个 Batch（A~E）。 | Claude |
| 2026-06-16 | 收口修复：Batch 结构对齐为 A=骨架 / B=Artifact tabs 重组（功能等价）/ C=ContextPanel+状态隔离 / D=视觉 polish+warnings 降噪 / E=验收+completion；新增 §0 核心重构方向（Artifact tabs/ContextPanel/warnings 降噪）；§6 安全边界去重为权威单处。本轮仅文档，Phase 8 编码尚未开始。 | Claude |
| 2026-06-16 | 审核通过，status 从 draft 转 active；允许进入 Phase 8 Batch A（P8-T01~P8-T02）；Phase 8 编码尚未开始；Batch B/C/D/E 与 Phase 9/10/11 未开始。 | Claude |
| 2026-06-16 | Batch A 编码完成：新增 AppShell / AppHeader / LeftNav / StageWorkspace，WorkspacePage 接线为 AgentScope 风格骨架；P8-T01/P8-T02 进入审核收口；Batch B/C/D/E 与 Phase 9/10/11 未开始。 | Claude |
| 2026-06-16 | Batch A 审核收口：明确 StageWorkspace 中 Artifact tabs 仅为视觉占位（无 activeTab 状态、不响应点击），Batch B 才进行 Evidence/Understanding/Views/Trace/Q&A/Quality 内容迁移；修复 docs/planning/README.md Markdown 加粗未闭合。 | Claude |
| 2026-06-16 | Batch B 编码完成：StageWorkspace 实现真实 Artifact tabs 切换 + StageOverviewBar + StageFilterBar；StageDetail 改为按 activeTab 分区渲染；EvidencePanel 增加可展开分组与前端筛选；P8-T03/P8-T04 进入审核收口；Batch C/D/E 与 Phase 9/10/11 未开始。 | Claude |
| 2026-06-16 | Batch C 审核收口：移除误提交的 `.codegraph/.gitignore`；修复 citation 点击后被异步 source excerpt 覆盖的竞态；`clearQualityState` 仅清理 `quality_issue` context；`handleCloseSourceExcerpt` 关闭时同步清空 `source_excerpt` context；`QualityReviewPanel` 所有 issue 均可点击进入 ContextPanel（无 evidence/source 的 issue 也能展示）；P8-T05/P8-T06 仍处于审核收口状态，Batch D/E 与 Phase 9/10/11 未开始。 | Claude |
| 2026-06-16 | Batch C 交互收口：`QualityReviewPanel.handleIssueClick` 不再自动调用 `onViewSource`，点击质量记录后 ContextPanel 稳定展示 `quality_issue`，源码查看收敛为 ContextPanel 内“查看源码片段”显式按钮；移除未使用的 `onViewSource` prop 链（QualityReviewPanel/StageDetail QualityTab）；P8-T05/P8-T06 仍处于审核收口状态，Batch D/E 与 Phase 9/10/11 未开始。 | Claude |
