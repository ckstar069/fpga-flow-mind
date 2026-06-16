# Phase 8 产品级工作台验证设计

---
status: active
updated: 2026-06-16
---

> 本文档定义 Phase 8（产品级 UI/UX 工作台重构）的验证与验收设计：前端构建与组件验证、可用性验收、既有能力回归、视觉语义一致性、真实桌面验收、安全回归、完成标准。
>
> 验证目标是"工作台是否把已有能力组织成用户可高效理解一个阶段的产品级形态，且不发生回归、不越界"，**不是**验证目标项目正确性。
>
> 本文 status 为 `active`，是 Phase 8 验证与验收设计的生效文档。**Phase 8 Batch A（P8-T01~P8-T02）已实现/进入审核收口**；**Phase 8 Batch B（P8-T03~P8-T04）已实现/进入审核收口**；**Phase 8 Batch C（P8-T05~P8-T06）已实现/进入审核收口**；Batch D/E 与 Phase 9/10/11 均未开始。

## 1. 验证策略总览

| 层 | 方式 | 目的 |
|----|------|------|
| 前端构建与类型 | `npm run build` + `npx tsc --noEmit` | 重构不破坏编译与类型 |
| 后端回归 | `cargo test --lib` + `cargo check` | 展示性调整不破坏后端契约（应无变化或最小） |
| 组件/状态验证 | 关键组件渲染、焦点状态机、状态隔离 | 信息架构与状态正确 |
| 可用性验收 | 真实项目全流程无说明完成 | 产品级可用度 |
| 既有能力回归 | Phase 4/5/6/7 能力桌面验收 | 零退化 |
| 视觉语义一致性 | confidence/strength/unknown/severity 一致性检查 | 视觉系统落地一致 |
| 安全回归 | checksum + rg | 只读、不接 LLM、不运行工具链、不输出审计结论 |

## 2. 前端构建与组件验证方向

| 项 | 方向 |
|----|------|
| 构建 | `npm run build` + `npx tsc --noEmit` 通过 |
| 组件 | AppShell / LeftNav / StageWorkspace / 各 Section / 卡片 / 概览条 / 筛选条 在空/加载/有数据/错误态渲染正常 |
| 焦点状态机 | mode/active_stage_id 转移正确；切换阶段清空下游；Artifact tab 切换保持选中态（Batch B 为前端局部状态，未持久化） |
| 状态隔离 | 阶段切换 / 重生成时 downstream maps 清除 + guard 过滤旧响应（延续 Phase 7 P2） |
| 持久化恢复 | session 重载回到原焦点；旧 session 缺字段取默认不崩溃 |
| 文案 | rg 验证前端文案不含"正确/错误""PASS/HOLD""审计结论""通过/失败裁决" |

## 3. 可用性验收（AgentScope 风格 + 信息架构）

验收工作台是否落实需求 R8-001~R8-008 与 UI/UX 文档 §2 的 9 条 AgentScope 设计语言：

| 验收点 | 预期 |
|--------|------|
| 三段式骨架 | LeftNav（深色固定）+ MainWorkspace（浅色），不再单页长堆叠 |
| 左侧导航 | 项目/阶段/功能区一次点击可达；阶段状态有视觉标记 |
| 阶段分区 | Overview/Evidence/Understanding/Views/Trace/Q&A/Quality 分区，不同屏堆叠 |
| 顶部概览 | 阶段关键指标一眼可见（evidence/claim/视图/quality） |
| 中部筛选 | 按 confidence/source_kind/severity/view type 筛选生效 |
| 可展开列表 | evidence/claim/node/issue 可展开显示追溯字段 |
| 卡片化 | 对象与面板用卡片，无营销式大留白 |
| 蓝色强调 | 主操作/选中/可追溯链接蓝色；无红绿裁决色 |
| 密度层级 | 高密度可读，层级清晰 |
| 一体化工作流 | 收集→生成→视图→trace→Q&A 顺滑，无需记忆步骤 |
| Artifact tabs 不长堆叠 | Evidence/Understanding/Views/Trace/Q&A/Quality 作为 tab，质量报告/trace/source excerpt/Q&A 不向下堆叠 |
| ContextPanel 联动 | 选中 evidence/trace/source excerpt/quality issue 进右侧面板；阶段切换/重生成清空 |
| warnings 降噪 | scan_timeout 等警告折叠/分类/计数/可展开，不长期占底部 |

## 4. 既有能力回归（零退化）

UI 重构不得破坏 Phase 4/5/6/7 已验收能力：

| 能力 | 来源 | 回归验收 |
|------|------|----------|
| 三类视图渲染（structure/dataflow/timing） | Phase 4 | 真实项目 Views 分区正常；SVG+CSS 不变 |
| trace 回链 + source excerpt | Phase 5 | 节点点击→Trace 分区→证据高亮→source excerpt |
| Grounded Q&A | Phase 5 | Q&A 分区提问 + citation + unknown/gap 诚实 |
| session 持久化与恢复 | Phase 6 | 保存/重载回到原焦点；app-owned 只写 |
| Quality Review | Phase 7 | Quality 分区 summary + issue list + 分类 |
| 真实项目识别 + 深层扫描 | Phase 7 | 真实项目阶段识别、深层源码、噪声跳过不变 |
| dataflow 非空 / timing 诚实空 | Phase 7 | 视图行为不退化 |

> 真实样本：主样本 `fpga_project_coarse_sync`、副样本 `fpga_project_fft`（延续 Phase 7）。

## 5. 视觉语义一致性检查

| 检查 | 预期 |
|------|------|
| confidence 五值视觉 | confirmed/supported/inferred/unknown/conflicting 在 evidence/claim/node/Q&A 一致 |
| 证据强度 / severity | 不用红绿裁决色；High/Medium/Low 字重区分 |
| unknown / evidence_gap | 灰色 + 图标，与 confirmed 区隔明显 |
| 退化视图 | empty timing / 孤立节点 / 标签重复就地诚实表达 |
| 蓝色边界 | 蓝色仅用于操作/选中/可追溯，不用于裁决 |

## 6. 真实桌面验收步骤

| 步骤 | 验收内容 | 预期 |
|------|----------|------|
| 1 | 打开真实样本，LeftNav 识别阶段 + 状态标记 | 与 Phase 7 识别一致 |
| 2 | 选 L0/L1/RTL，StageWorkspace 分区切换 | 各分区内容正确，不堆叠 |
| 3 | 顶部概览指标 | evidence/claim/视图/quality 指标可见 |
| 4 | Evidence 分区筛选 + 可展开 | 筛选生效，展开显示追溯字段 |
| 5 | Views 分区三类视图 + 节点点击 | 渲染正常，点击→Trace 联动 |
| 6 | Trace 分区 source excerpt + 高亮 | 回链正确 |
| 7 | Q&A 分区提问 | citation + unknown/gap 诚实 |
| 8 | Quality 分区 | summary + issue 分类（含 Phase 7 新分类） |
| 9 | 阶段切换 / 重生成 | 下游清除 + 无旧响应回写 |
| 10 | 空阶段 / 命名异常 / 源码变更状态 | 清晰表达 + 可操作 |
| 11 | session 保存 / 重载 | 回到原焦点；checksum 不变 |
| 12 | 最近项目切换 | 一次点击切换 |
| 13 | warnings 折叠/分类/计数 | scan_timeout 等警告不再长期占底部，可折叠计数 + 展开查看 |

## 7. 安全回归

```bash
# 不运行 Vivado / synthesis / implementation / bitstream（重构不应引入）
rg "Vivado|synthesis|implementation|bitstream" src src-tauri/src
# 期望：仅禁用语境，无实际调用

# 不调用真实 LLM
rg "OpenAI|Anthropic|api_key" src src-tauri/src
# 期望：无产品代码匹配

# 不输出 PASS/HOLD / 正确性裁决
rg "PASS|HOLD|正确|错误|审计" src src-tauri/src
# 期望：仅守卫代码/注释/错误处理语境，非用户可见裁决

# 不引入重可视化库
rg "react-flow|reactflow|d3|mermaid" src package.json
# 期望：无匹配（视图 SVG+CSS）

# 不写目标项目
rg "std::fs::write|create_dir|remove_file|remove_dir|rename|copy" src-tauri/src
# 期望：仅测试模块/app-owned persistence
```

> 命中点逐条核对：Vivado/LLM/PASS-HOLD/重库只允许出现在禁用/守卫语境；真实样本 checksum 验收前后一致。

## 8. Phase 8 完成标准

| 条件 | 验证方式 |
|------|----------|
| R8-001~R8-010 实现 | 需求验收 |
| AgentScope 风格 9 条设计语言落地 | 可用性验收 §3 |
| 既有能力零回归 | 回归验收 §4 + 真实桌面验收 §6 |
| 视觉语义一致 | §5 |
| 真实用户无说明完成全流程 | 桌面验收 |
| `npm run build` / `npx tsc --noEmit` / `cargo test --lib` / `cargo check` 通过 | 命令 |
| 安全边界保持 | checksum + rg §7 |
| Phase 8 completion review 转 active | 文档 |

## 9. 进入 Phase 9 的条件

Phase 8 完成后，方允许考虑进入：

- **Phase 9（真实 LLM grounding）**：以 Phase 8 的视觉语义系统承载真实 LLM 的 inferred/unknown/citation；需 Phase 9 详细文档 active。
- **不得**在 Phase 8 未完成时启动 Phase 9/10/11 编码。

## 10. 安全边界汇总

- 目标项目只读，checksum 一致。
- 不运行 Vivado / synthesis / implementation / bitstream。
- 不调用真实 LLM（不读取 `api_key`、不调用 OpenAI / Anthropic）。
- 持久化只写 app-owned storage。
- 不输出 PASS/HOLD/正确性裁决/审计结论。
- 不引入重可视化库（视图 SVG+CSS）。

## 11. 关联文档

- [`../requirements/phase-8-product-workbench-requirements.md`](../requirements/phase-8-product-workbench-requirements.md) — 需求（draft）
- [`../design/phase-8-workbench-architecture.md`](../design/phase-8-workbench-architecture.md) — 工作台架构（draft）
- [`../design/phase-8-ui-state-and-navigation-design.md`](../design/phase-8-ui-state-and-navigation-design.md) — UI 状态与导航（draft）
- [`../ui-ux/phase-8-product-workbench-view.md`](../ui-ux/phase-8-product-workbench-view.md) — UI/UX 设计（draft）
- [`../planning/phase-8-implementation-plan.md`](../planning/phase-8-implementation-plan.md) — 编码实施计划（active）
- [`phase-7-real-project-quality-validation.md`](phase-7-real-project-quality-validation.md) — Phase 7 验证（active，回归基线）

## 13. Batch A 验证记录（P8-T01~P8-T02）

> 记录 Phase 8 Batch A 工作台骨架实现后的实际验证结果，作为 Batch B/C/D/E 的基线。

### 13.1 实现范围

- 新增 `src/features/workspace/components/AppShell.tsx`：外层 flex 布局容器（header + leftNav + main + footer）。
- 新增 `src/features/workspace/components/AppHeader.tsx`：顶部标题、项目路径输入、打开项目按钮、SessionStatusIndicator。
- 新增 `src/features/workspace/components/LeftNav.tsx`：深色固定左侧导航（项目信息、阶段列表、最近项目、加载错误）。
- 新增 `src/features/workspace/components/StageWorkspace.tsx`：浅色主工作区 + StageOverviewBar/StageFilterBar/Artifact tabs/ContextPanel 占位容器；主内容区继续渲染 `StageDetail`。
- 修改 `src/features/workspace/WorkspacePage.tsx`：接线以上组件，保留所有状态机、回调、guard 逻辑与底部 warnings footer。

### 13.2 自动化验证结果

| 命令 | 结果 |
|------|------|
| `npm run build` | ✅ 通过 |
| `npx tsc --noEmit` | ✅ 无类型错误 |
| `cd src-tauri && cargo test --lib` | ✅ 全通过 |
| `cd src-tauri && cargo check` | ✅ 无 warning |
| `cargo test --test real_project_validation -- --ignored` | ✅ 主/副样本 5 项测试通过 |
| 边界 rg（ReactFlow/D3/Mermaid/OpenAI/Anthropic/api_key/Vivado/synthesis/implementation/bitstream/PASS/HOLD/审计结论） | ✅ 仅既有禁用/守卫/注释语境命中，Batch A 无新增 |
| 目标项目 checksum 一致性 | ✅ 主样本 `fpga_project_coarse_sync` checksum 一致；目标项目仅含既有未跟踪文件，无本次修改 |

### 13.3 手工验证（无头环境可执行部分）

- 启动 `npm run tauri dev`，前端 dev server 在 `http://localhost:1420/` 正常响应，Rust 后端编译运行无崩溃。
- `open_workspace` 命令单元测试 4/4 通过。
- 真实项目验证测试 5/5 通过，阶段识别与 checksum 一致。
- 目标项目 `fpga_project_coarse_sync`、`fpga_project_fft` 未被修改（git status 无修改项）。

> 注：完整桌面 UI 交互（点击、布局可视、响应式宽度）需在真实桌面环境进行；本 Batch A 已通过构建/类型/后端/样本测试保证骨架无回归，待 Batch E 做最终桌面验收 12 步。

### 13.4 当前未完成范围

- P8-T03~P8-T04（Artifact tabs 内容迁移、概览/筛选/可展开列表）未开始。
- P8-T05~P8-T06（ContextPanel 联动、状态隔离）未开始。
- P8-T07~P8-T09（视觉 polish、warnings 降噪、空/错误/session 恢复）未开始。
- P8-T10（回归 + 桌面验收 + completion review）未开始。

### 13.5 审核收口修复（Batch A）

- 修复 `docs/planning/README.md` 中两处 Phase 8 状态说明的 Markdown 加粗未闭合问题。
- 修正 `src/features/workspace/components/StageWorkspace.tsx` 的 Artifact tabs 占位交互：移除 `activeTab` / `setActiveTab` 状态与点击响应，tab 改为非交互 `<span>` 展示，并新增占位说明"Artifact tabs 内容迁移将在 Batch B 实现；当前主内容区保留原 StageDetail 兼容视图"，避免用户误以为 Batch B 内容迁移已完成。

## 14. Batch B 验证记录（P8-T03~P8-T04）

> 记录 Phase 8 Batch B Artifact tabs 内容迁移与 UI 增强后的实际验证结果。

### 14.1 实现范围

- 新增 `src/features/workspace/components/StageOverviewBar.tsx`：从当前阶段产物前端派生轻量指标（文件数、evidence 数、claim 数、视图数、Q&A 次数、quality issue 数、阶段状态）。
- 新增 `src/features/workspace/components/StageFilterBar.tsx`：按 active tab 渲染前端筛选控件（evidence 按 source_kind/strength/文本搜索；quality 按 severity/kind/status）。
- 修改 `src/features/workspace/components/StageWorkspace.tsx`：实现真实 Artifact tabs 切换（Overview/Evidence/Understanding/Views/Trace/Q&A/Quality），集成 StageOverviewBar 与 StageFilterBar，保留右侧 ContextPanel 占位。
- 修改 `src/features/workspace/components/StageDetail.tsx`：由纵向堆叠改为按 `activeTab` switch 渲染各 tab 内容；保留所有既有入口按钮、回调、trace、Q&A、质量报告能力。
- 修改 `src/features/workspace/components/EvidencePanel.tsx`：增加按 `source_kind` 分组的可展开列表，支持由 StageFilterBar 传入的前端筛选。
- 修改 `src/features/workspace/WorkspacePage.tsx`：向 StageWorkspace/StageDetail 透传 stage status、qaHistory、filter 等；保持状态机、guard refs、保存/自动保存逻辑不变。
- `src/types/workspace.ts`：`PersistedUiState` 未接入 `stage_tab`；Batch B 的 active tab 状态为 `StageWorkspace` 前端局部状态，Batch C/D 如需持久化需前后端同步设计。

### 14.2 自动化验证结果

| 命令 | 结果 |
|------|------|
| `npm run build` | ✅ 通过 |
| `npx tsc --noEmit` | ✅ 无类型错误 |
| `cd src-tauri && cargo test --lib` | ✅ 544 passed; 0 failed; 1 ignored |
| `cd src-tauri && cargo check` | ✅ 无 warning |
| `cargo test --test real_project_validation -- --ignored` | ✅ 主/副样本 5 项测试通过 |
| 边界 rg（ReactFlow/D3/Mermaid/OpenAI/Anthropic/api_key/Vivado/synthesis/implementation/bitstream/PASS/HOLD/审计结论） | ✅ 仅既有禁用/守卫/注释语境命中，Batch B 无新增 |
| 目标项目 checksum 一致性 | ✅ 主样本 `fpga_project_coarse_sync` checksum 一致；目标项目无本次修改 |

### 14.3 当前未完成范围

- P8-T07~P8-T09（视觉 polish、warnings 降噪、空/错误/session 恢复）未开始。
- P8-T10（回归 + 桌面验收 + completion review）未开始。

## 15. Batch C 验证记录（P8-T05~P8-T06）

> 记录 Phase 8 Batch C ContextPanel 真实联动与状态隔离实现后的实际验证结果。

### 15.1 实现范围

- 新增 `src/features/workspace/components/contextPanelTypes.ts`：定义前端局部 `ContextSelection` 类型（kind/stageId/payload tag union），不进入 `PersistedUiState` / Rust 持久化契约。
- 新增 `src/features/workspace/components/contextPanelUtils.ts`：`ContextPanel` 中文标签、颜色映射等辅助函数。
- 新增 `src/features/workspace/components/ContextPanel.tsx`：右侧只读上下文面板，支持 evidence / trace_target / source_excerpt / quality_issue / view_node / view_edge / qa_citation 七种对象类型，提供空状态、查看源码片段、定位 evidence 动作入口。
- 修改 `src/features/workspace/components/StageWorkspace.tsx`：接收 `contextSelection` / `onContextSelectionChange` / `onViewSource` / `onLocateEvidence` props；右侧从占位容器替换为真实 `ContextPanel`；`renderContent` 扩展 `onContextSelectionChange` 透传。
- 修改 `src/features/workspace/components/StageDetail.tsx`：新增 `stageId` / `onContextSelectionChange` props，并向下透传给 `EvidenceTab` / `ViewsTab` / `QualityTab`。
- 修改 `src/features/workspace/components/EvidencePanel.tsx`：点击 evidence 时先保持原高亮逻辑，再调用 `onContextSelection` 更新 ContextPanel。
- 修改 `src/features/workspace/components/MultiViewPanel.tsx`：点击 node/edge 时先调用 `onSelectTarget`，再调用 `onContextSelection` 设置 `view_node` / `view_edge`。
- 修改 `src/features/workspace/components/QualityReviewPanel.tsx`：点击 issue 时调用 `onContextSelection` 设置 `quality_issue`。
- 修改 `src/features/workspace/WorkspacePage.tsx`：新增 `contextSelection` 状态；在 `handleSelectTraceTarget` / `handleViewSource` / `handleGroundedCitationClick` 中设置对应上下文；在 `clearTraceState` / `handleGenerateQualityReport` 中清空上下文（覆盖打开项目、选阶段、加载 session、重新收集/生成、清空 trace target 等隔离路径）。

### 15.2 自动化验证结果

| 命令 | 结果 |
|------|------|
| `npm run build` | ✅ 通过 |
| `npx tsc --noEmit` | ✅ 无类型错误 |
| `cd src-tauri && cargo test --lib` | ✅ 544 passed; 0 failed; 1 ignored |
| `cd src-tauri && cargo check` | ✅ 无 warning |
| 边界 rg（stage_tab / stageTab / active_stage_tab） | ✅ 无新增匹配 |
| 边界 rg（ReactFlow/D3/Mermaid/OpenAI/Anthropic/api_key/Vivado/synthesis/implementation/bitstream/PASS/HOLD/审计结论） | ✅ 仅既有禁用/守卫/注释语境命中，Batch C 无新增 |
| 边界 rg（warnings 降噪 / Batch D / Batch E / Phase 9 / Phase 10 / Phase 11） | ✅ `src/features/workspace` 无匹配 |
| 目标项目 checksum 一致性 | ✅ 主样本 `fpga_project_coarse_sync` checksum 一致；目标项目无本次修改 |

### 15.3 当前未完成范围

- P8-T07~P8-T09（视觉 polish、warnings 降噪、空/错误/session 恢复）未开始。
- P8-T10（回归 + 桌面验收 + completion review）未开始。

### 15.4 Batch C 审核收口修复

> 本次收口严格限定在 Batch C 范围内，不进入 Batch D/E，不做 warnings 降噪，不接真实 LLM，不改目标项目。

| 问题 | 修复说明 |
|------|----------|
| 误提交 `.codegraph/.gitignore` | 执行 `git rm --cached .codegraph/.gitignore`，本地 `.codegraph/` 目录保留但仓库不再跟踪 |
| Q&A citation 被 source excerpt 异步覆盖 | `handleGroundedCitationClick` 不再自动调用 `handleViewSource`，citation 点击后 ContextPanel 稳定展示 `qa_citation`；用户可在 ContextPanel 内点击“查看源码片段”手动加载 source excerpt |
| quality report 失效后 stale `quality_issue` context | `clearQualityState()` 增加仅对 `quality_issue` 的 context 清理，不影响 evidence/trace/source_excerpt/qa_citation |
| 关闭 source excerpt 后 stale `source_excerpt` context | `handleCloseSourceExcerpt()` 在关闭时同步清空 `source_excerpt` context |
| QualityReviewPanel issue 点击受限 | 移除 issue 按钮的 `disabled` 与默认光标限制，所有 issue 均可点击进入 ContextPanel；有 evidence_id 时保留高亮 |

### 15.5 Batch C 交互收口：quality_issue 不再被异步 source_excerpt 覆盖

> 本轮收口严格限定在 Batch C 范围内，不进入 Batch D/E，不做 warnings 降噪，不接真实 LLM，不改目标项目。

| 问题 | 修复说明 |
|------|----------|
| quality_issue 点击后 ContextPanel 最终停留在 source_excerpt | `QualityReviewPanel.handleIssueClick` 不再自动调用 `onViewSource(...)`；仅保留 `evidence_id` 高亮与 `onContextSelection({ kind: 'quality_issue' })`。点击质量记录后右侧稳定展示 quality_issue，不被异步 source_excerpt 覆盖 |
| 查看源码入口收敛为显式动作 | `ContextPanel` 的 `QualityIssueBody` 中“查看源码片段”按钮作为唯一显式入口；用户点击 quality_issue → 显示 issue 详情 → 再点击“查看源码片段” → 才切换到 source_excerpt。无新增自动跳转逻辑 |
| 一并清理未使用的 prop | 移除 `QualityReviewPanelProps.onViewSource` 字段、`StageDetail` `QualityTab` 的 `onViewSource` 参数/类型/传参，以及 case 'quality' 的 `onViewSource` 传参（满足 `noUnusedParameters`） |

数据流（链路清晰）：

```text
QualityReviewPanel.handleIssueClick(issue)
  -> onEvidenceSelect(issue.evidence_id)  // 仅当存在
  -> onContextSelection({ kind: 'quality_issue', ... })
       -> ContextPanel 渲染 QualityIssueBody（稳定展示 quality_issue）
            -> 用户点击“查看源码片段”按钮（显式动作）
                 -> ContextPanel.onViewSource -> handleViewSource
                      -> 异步设置 source_excerpt context（用户预期内的切换）
```

## 12. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-16 | 初始 draft：验证策略、AgentScope 风格可用性验收、既有能力零回归、视觉语义一致性、真实桌面验收 12 步、安全回归（含重库检查）、完成标准、进入 Phase 9 条件。审核转 active 后方允许编码。 | Claude |
| 2026-06-16 | 审核通过，status 从 draft 转 active；允许进入 Phase 8 Batch A（P8-T01~P8-T02）；Phase 8 编码尚未开始；Batch B/C/D/E 与 Phase 9/10/11 未开始。 | Claude |
| 2026-06-16 | 追加 §13 Batch A 验证记录：P8-T01/P8-T02 骨架实现通过 npm build / tsc / cargo test --lib / cargo check / real_project_validation --ignored；边界 rg 与目标项目只读验证通过；Batch B/C/D/E 未开始。 | Claude |
| 2026-06-16 | 追加 §15.5 Batch C 交互收口：quality_issue 点击不再自动拉起源码片段，避免异步 source_excerpt 覆盖；源码查看收敛为 ContextPanel 的显式按钮动作；移除未使用的 `onViewSource` prop 链；验证命令与边界 rg 全通过；Batch D/E 与 Phase 9/10/11 未开始。 | Claude |
