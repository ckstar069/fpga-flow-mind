# Phase 8 产品级工作台验证设计

---
status: active
updated: 2026-06-16
---

> 本文档定义 Phase 8（产品级 UI/UX 工作台重构）的验证与验收设计：前端构建与组件验证、可用性验收、既有能力回归、视觉语义一致性、真实桌面验收、安全回归、完成标准。
>
> 验证目标是"工作台是否把已有能力组织成用户可高效理解一个阶段的产品级形态，且不发生回归、不越界"，**不是**验证目标项目正确性。
>
> 本文 status 为 `active`，是 Phase 8 验证与验收设计的生效文档。**当前仅允许进入 Phase 8 Batch A（P8-T01~P8-T02）**；Batch B/C/D/E 与 Phase 9/10/11 均未开始。Phase 8 编码尚未开始。

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
| 焦点状态机 | mode/stage_tab 转移正确；切换阶段清空下游；Tab 切换保持选中态 |
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
- [`../planning/phase-8-implementation-plan.md`](../planning/phase-8-implementation-plan.md) — 编码实施计划（draft）
- [`phase-7-real-project-quality-validation.md`](phase-7-real-project-quality-validation.md) — Phase 7 验证（active，回归基线）

## 12. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-16 | 初始 draft：验证策略、AgentScope 风格可用性验收、既有能力零回归、视觉语义一致性、真实桌面验收 12 步、安全回归（含重库检查）、完成标准、进入 Phase 9 条件。审核转 active 后方允许编码。 | Claude |
| 2026-06-16 | 审核通过，status 从 draft 转 active；允许进入 Phase 8 Batch A（P8-T01~P8-T02）；Phase 8 编码尚未开始；Batch B/C/D/E 与 Phase 9/10/11 未开始。 | Claude |
