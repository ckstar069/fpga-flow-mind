# UI/UX 文档索引

---
status: active
updated: 2026-06-11
---

## UI/UX 目录用途

本目录存放 `fpga-flow-mind` 的用户体验与界面设计文档。UI/UX 文档描述"用户如何与产品交互、信息如何组织、视图如何呈现"，不描述后端实现细节。

## 建议文档类型

| 文档类型 | 说明 | 示例 |
|----------|------|------|
| `user flows` | 用户操作流程 | 从打开项目到查看证据的完整流程 |
| `information architecture` | 信息架构 | 视图层级、导航结构、内容组织 |
| `views and panels` | 视图与面板定义 | 每个视图的布局、内容、交互 |
| `interaction states` | 交互状态 | 加载中、无数据、错误、选中、hover 等状态 |
| `visual grammar for evidence/confidence/uncertainty` | 视觉语义规范 | 如何用颜色、图标、线型表达证据强度、置信度、不确定性 |

## UI/UX 设计原则

### 1. 服务理解，不做炫技图形界面

- 界面的目标是帮助用户理解 FPGA 阶段实现，不是展示视觉效果
- 优先清晰、准确、可追溯，其次美观

### 2. 三类核心视图优先

首发版本优先支持：

- **结构图** — 模块、接口、层级关系
- **数据流图** — 数据从哪里来、经过什么变换、流向哪里
- **时序/流水图** — latency、握手信号、流水线行为

### 3. 节点、边、解释必须能回链 evidence

- 图中每个节点和边应可点击，跳转至对应的源码证据
- 解释文本应标注证据来源和置信度

### 4. unknown / inferred / conflicting 必须可见

- 不确定性不应被隐藏或淡化
- 用户应能一眼区分 confirmed、inferred 和 unknown 内容
- conflicting 证据应显式提示，不自动裁决

### 5. 默认中文界面文案

- 所有面向用户的默认文案使用简体中文
- 代码标识符、技术术语可保留英文，但应提供中文解释

## 明确不要做

- **不要把产品做成 JSON viewer** — 用户看到的应是图和解释，不是原始 JSON
- **不要把产品做成 Markdown report viewer** — 用户看到的应是交互式视图，不是静态报告
- **不要追求复杂可视化效果** — 清晰理解优先于视觉炫技

| [`phase-6-session-and-mvp-view.md`](phase-6-session-and-mvp-view.md) | `draft` | Phase 6 Session 管理与 MVP 验收 UI/UX 设计：顶部标题栏保存状态、最近项目列表、加载失败状态、删除确认、MVP 验收 UI 流程、文案规范 | Phase 6 前端编码前必读 |

## UI/UX 文档层级关系

Phase 1 UI/UX 设计由以下文档组成：

| 文档 | 定位 |
|------|------|
| [`phase-1-workspace-and-stage-flow.md`](phase-1-workspace-and-stage-flow.md) | Phase 1 workspace 与阶段选择流程 UI/UX 轻量设计 |

> 后续 Phase 的 UI/UX 文档（evidence 面板、三类视图、Q&A 等）将在对应 Phase 实施前补充。

## 当前文档列表

| 文档 | 状态 | 说明 | 推荐阅读时机 |
|------|------|------|-------------|
| [`phase-1-workspace-and-stage-flow.md`](phase-1-workspace-and-stage-flow.md) | `active` | Phase 1 workspace 与阶段选择流程 UI/UX 轻量设计：页面结构、组件定义、状态展示规则、空状态处理 | Phase 1 前端实施依据 |
| [`phase-2-evidence-view.md`](phase-2-evidence-view.md) | `active` | Phase 2 evidence view 前端设计：EvidencePanel 组件、CollectEvidenceButton、统计概要、筛选栏、证据项卡片、警告列表、状态管理、前后端边界 | Phase 2 前端编码前必读 |
| [`phase-3-understanding-view.md`](phase-3-understanding-view.md) | `active` | Phase 3 UnderstandingPanel 前端设计：状态展示、claim 列表、confidence 颜色映射（5 值含 supported 琥珀色）、StageSummary 两层展示、evidence 回链、unknown/gap 区域、生成按钮、禁止用语、TypeScript 类型 | Phase 3 前端编码前必读 |
| [`phase-4-multi-view-panel.md`](phase-4-multi-view-panel.md) | `active` | Phase 4 三视图面板前端设计：MultiViewPanel 布局、Tab bar（结构图/数据流/时序流水）、SVG 渲染方案、节点/边颜色形状置信度编码、hover tooltip、空状态、交互规范 | Phase 4 前端编码前必读 |
| [`phase-5-trace-and-qa-view.md`](phase-5-trace-and-qa-view.md) | `active` | Phase 5 证据回链与 Grounded Q&A 前端设计：MultiViewPanel 选中态、TracePanel、SourceExcerptPanel、EvidencePanel 高亮、GroundedQAPanel、视觉语义 | Phase 5 前端编码前必读 |
