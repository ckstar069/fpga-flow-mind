# Phase 8 编码实施计划

---
status: draft
updated: 2026-06-16
---

> 本文档定义 Phase 8（产品级 UI/UX 工作台重构）的编码实施计划：任务拆解（P8-T01~P8-T10）、依赖关系、Batch 划分（A~E）、进入/退出条件、安全边界。
>
> Phase 8 是**前端为主**的阶段：把 Phase 1~7 已有能力重组为产品级工作台，**不新增语义分析能力，不接真实 LLM，不做跨阶段映射**。
>
> 本文档 status 为 `draft`，是 Phase 8 编码的实施计划草案，**审核转 `active` 后才允许进入 Phase 8 编码**。Phase 8 编码尚未开始。Phase 7 已完成（completion review active）。

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
| **以上 Phase 8 详细文档全部转为 active** | ⏳ 待审核（本轮仅编制 draft） |

> 纪律：本轮**只编制 Phase 8 详细文档（draft）**，不编码、不改 `src/` 与 `src-tauri/`。Phase 8 详细文档全部审核转 active 后，方允许进入编码。Phase 9/10/11 未开始。

## 2. 任务拆分

### P8-T01 AppShell + LeftNav + 焦点状态机

| 维度 | 说明 |
|------|------|
| **目标** | 三段式骨架：AppShell + 深色 LeftNav + WorkspaceFocus 状态机（mode/active_stage_id/stage_tab） |
| **输入文档** | 架构 §3、UI 状态 §2~§3、UI/UX §3~§4 |
| **预计修改文件** | `src/features/workspace/`（新增 AppShell/LeftNav + 焦点 context，拆分 WorkspacePage） |
| **验收命令** | `npm run build` + `npx tsc --noEmit` + 组件渲染检查 |
| **不做什么** | 不改后端；不引入路由库；不破坏既有面板渲染逻辑 |

### P8-T02 StageWorkspace 三段式 + Tab 分区

| 维度 | 说明 |
|------|------|
| **目标** | 选中阶段后 MainWorkspace 渲染 StageWorkspace（TopBar + OverviewBar + FilterBar + ContentArea），Tab 分区替代长堆叠 |
| **输入文档** | 架构 §3.2、UI 状态 §4、UI/UX §5 |
| **预计修改文件** | `src/features/workspace/`（StageWorkspace 容器 + 各 Section 复用既有展示组件） |
| **验收命令** | `npm run build` + 桌面验收分区切换 |
| **不做什么** | 不重写既有展示组件渲染逻辑（复用，避免回归） |

### P8-T03 顶部概览 + 中部筛选 + 可展开对象列表

| 维度 | 说明 |
|------|------|
| **目标** | StageOverviewBar（指标）+ StageFilterBar（筛选/分组/视图切换）+ 可展开 evidence/claim/node/issue 列表 |
| **输入文档** | 需求 R8-005、UI/UX §5~§6 |
| **预计修改文件** | `src/features/workspace/`（概览条/筛选条/可展开列表组件） |
| **验收命令** | `npm run build` + 筛选/展开验收 |
| **不做什么** | 指标仅内部质量/规模，不对目标项目评价；不新增 command 除非必要 |

### P8-T04 卡片化 + 蓝色强调 + 视觉语义系统

| 维度 | 说明 |
|------|------|
| **目标** | 统一卡片样式 + 蓝色强调规范 + confidence/strength/unknown/severity 视觉语义在工作台一致应用 |
| **输入文档** | 需求 R8-006/R8-007、UI/UX §6~§8 |
| **预计修改文件** | `src/features/workspace/`（卡片/标签/视觉语义组件 + 样式） |
| **验收命令** | `npm run build` + 视觉语义一致性检查 |
| **不做什么** | 不用红绿裁决色；视觉改变不等于模型改变 |

### P8-T05 阶段理解一体化工作流 + 状态隔离迁移

| 维度 | 说明 |
|------|------|
| **目标** | 收集→生成→视图→trace→Q&A 工作流引导 + downstream 清除 + guard 迁移到 WorkspaceContext |
| **输入文档** | 需求 R8-004、架构 §4.3、UI 状态 §8 |
| **预计修改文件** | `src/features/workspace/`（context action 层 + 工作流引导 UI） |
| **验收命令** | `npm run build` + 状态隔离验收 |
| **不做什么** | 不改后端 command 语义；不做全自动流水线 |

### P8-T06 空 / 错误 / 加载状态产品化

| 维度 | 说明 |
|------|------|
| **目标** | 空阶段/命名异常/源码变更/路径不允许/timing 诚实空/Q&A 无证据/session 加载失败等产品级状态 |
| **输入文档** | 需求 R8-008、UI 状态 §5、UI/UX §10 |
| **预计修改文件** | `src/features/workspace/`（状态组件 + 文案） |
| **验收命令** | `npm run build` + 空/错误状态验收 |
| **不做什么** | 文案禁用审计用语；不掩盖退化 |

### P8-T07 session / 最近项目体验 + UI state 恢复

| 维度 | 说明 |
|------|------|
| **目标** | 最近项目/项目切换体验 + PersistedUiState 展示性扩展（stage_tab/selected_node_id）+ 焦点恢复 |
| **输入文档** | 需求 R8-002/R8-008、UI 状态 §6、架构 §4 |
| **预计修改文件** | `src/features/workspace/`、`src/types/`（UI state 类型扩展，向后兼容） |
| **验收命令** | `npm run build` + session 恢复验收 |
| **不做什么** | 不破坏 Phase 6 持久化兼容性；只写 app-owned |

### P8-T08 既有能力回归

| 维度 | 说明 |
|------|------|
| **目标** | 验证三类视图/trace/QA/quality/持久化/真实项目识别在重构后零退化 |
| **输入文档** | 测试 §4 |
| **预计修改文件** | 必要的适配修复（不改语义） |
| **验收命令** | `npm run build` + `cargo test --lib` + `real_project_validation --ignored` + 桌面回归 |
| **不做什么** | 不借回归改语义/能力 |

### P8-T09 真实桌面可用性验收

| 维度 | 说明 |
|------|------|
| **目标** | 真实项目（主/副样本）全流程无说明完成；AgentScope 风格 9 条落地；checksum 只读 |
| **输入文档** | 测试 §3/§6 |
| **预计修改文件** | 文档（验收记录） |
| **验收命令** | 桌面验收 12 步 + checksum + rg |
| **不做什么** | 不修改目标项目；不运行工具链 |

### P8-T10 Phase 8 completion review

| 维度 | 说明 |
|------|------|
| **目标** | 全量验证、文档状态更新、完成审查、明确进入 Phase 9 条件 |
| **输入文档** | 测试 §8~§9 |
| **预计修改文件** | `docs/planning/phase-8-completion-review.md`（新增）、各 index 更新 |
| **验收命令** | 全量测试 + rg + 桌面验收 + checksum |
| **不做什么** | 不进入 Phase 9 编码 |

## 3. 依赖关系

```text
P8-T01 (AppShell + 焦点)
  │
  └── P8-T02 (StageWorkspace + 分区)
        │
        ├── P8-T03 (概览 + 筛选 + 列表)
        ├── P8-T04 (卡片 + 视觉语义)
        └── P8-T05 (工作流 + 状态隔离)
              │
              ▼
        P8-T06 (空/错误状态)
              │
              ▼
        P8-T07 (session + 恢复)
              │
              ▼
        P8-T08 (回归)
              │
              ▼
        P8-T09 (桌面验收)
              │
              ▼
        P8-T10 (completion review)
```

## 4. Batch 划分（保守）

### 4.1 Batch A：工作台骨架

| 任务 | 内容 |
|------|------|
| P8-T01 | AppShell + LeftNav + 焦点状态机 |
| P8-T02 | StageWorkspace 三段式 + Tab 分区 |

**允许范围**：搭骨架与焦点路由，复用既有展示组件，不重写渲染逻辑。
**禁止越界**：不改后端；不引入路由库/状态库；不破坏既有面板。

### 4.2 Batch B：内容形态

| 任务 | 内容 |
|------|------|
| P8-T03 | 顶部概览 + 中部筛选 + 可展开列表 |
| P8-T04 | 卡片化 + 蓝色强调 + 视觉语义系统 |
| P8-T05 | 工作流引导 + 状态隔离迁移 |

**允许范围**：落实 AgentScope 风格内容形态与视觉系统。
**禁止越界**：不用红绿裁决色；不引图形库；不改语义模型。

### 4.3 Batch C：状态体验

| 任务 | 内容 |
|------|------|
| P8-T06 | 空/错误/加载状态产品化 |
| P8-T07 | session/最近项目 + UI state 恢复 |

**允许范围**：产品级状态体验 + 持久化兼容扩展。
**禁止越界**：不破坏 Phase 6 持久化兼容；只写 app-owned。

### 4.4 Batch D：回归与桌面验收

| 任务 | 内容 |
|------|------|
| P8-T08 | 既有能力零回归 |
| P8-T09 | 真实桌面可用性验收 |

**允许范围**：回归修复（不改语义）+ 真实样本桌面验收 + checksum 只读。
**禁止越界**：不修改目标项目；不运行 Vivado/synthesis/implementation/bitstream。

### 4.5 Batch E：completion review

| 任务 | 内容 |
|------|------|
| P8-T10 | 全量验证、文档同步、完成审查 |

**允许范围**：文档状态更新、completion review、进入 Phase 9 条件说明。
**禁止越界**：不进入 Phase 9 编码。

## 5. 退出条件

| 条件 | 验证方式 |
|------|----------|
| 前端构建 + 类型通过 | `npm run build` + `npx tsc --noEmit` |
| 后端契约不破坏 | `cargo test --lib` + `cargo check` |
| AgentScope 风格 9 条落地 | 可用性验收 |
| 既有能力零回归 | 回归 + 桌面验收 |
| 真实用户无说明完成全流程 | 桌面验收 |
| 视觉语义一致 | 一致性检查 |
| 目标项目只读 | checksum + rg |
| 无真实 LLM / 无审计用语 / 无重库 | rg |
| Phase 8 completion review 完成 | 文档（转 active） |

## 6. 安全边界

- 不修改目标项目；checksum 一致。
- 不运行 Vivado / synthesis / implementation / bitstream。
- 不调用真实 LLM（不读取 `api_key`、不调用 OpenAI / Anthropic）。
- 持久化只写 app-owned storage。
- 不破坏核心语义契约（`ImplementationUnderstanding`/confidence 枚举/evidence/view/trace/quality 模型字段语义稳定）。
- 不引入重可视化库（视图 SVG+CSS）。
- 不输出 PASS/HOLD/正确性裁决/审计结论。

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
| 2026-06-16 | 初始 draft：定义 P8-T01~P8-T10、5 个 Batch（A~E）划分与允许/禁止边界、依赖关系、进入/退出条件、安全边界、进入 Phase 9 条件。本轮仅编制 draft，不编码。 | Claude |
