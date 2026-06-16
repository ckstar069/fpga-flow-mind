# Phase 8 产品级工作台 UI/UX 设计

---
status: active
updated: 2026-06-16
---

> 本文档定义 Phase 8 产品级工作台的 **UI/UX 设计**：信息架构、布局、导航、视觉语义系统、交互状态、空/错误状态、文案规范。它把用户提供的 **AgentScope 风格参考**正式写入设计依据，并落实需求 R8-001~R8-008。
>
> Phase 8 是 UI 重构阶段，**不做工作台之外的能力扩张**。所有视觉与交互服从本项目定位：理解与可视化工具，非审计 dashboard，非大而全可视化平台，不输出 PASS/HOLD/正确性裁决。
>
> 本文 status 为 `active`，是 Phase 8 UI/UX 设计的生效文档。**当前仅允许进入 Phase 8 Batch A（P8-T01~P8-T02）**；Batch B/C/D/E 与 Phase 9/10/11 均未开始。Phase 8 编码尚未开始。

## 1. 设计目标

1. **以"理解一个阶段"为主线**：信息架构围绕用户认知流程，不再按后端能力机械堆叠面板。
2. **高信息密度 + 层级清晰**：屏幕利用率高，但通过导航/分区/卡片/视觉语义建立清晰层级，不混乱。
3. **一次可达**：项目、阶段、功能区一次点击可达。
4. **诚实可见**：confidence / 证据强度 / unknown / 退化视图 / quality 一致地层级化表达，不掩盖也不裁决。
5. **产品级体验**：session / 最近项目 / 空 / 错误状态达到真实可用。
6. **工作台化，不是新增更多卡片**：把已有能力重组为工作台信息架构（Artifact tabs + ContextPanel），而非堆砌更多面板/卡片。

## 2. AgentScope 风格参考（正式写入）

Phase 8 UI 以用户提供的 AgentScope 风格截图为**形态与信息架构参考**（非逐像素复刻）。提炼为 9 条设计语言，作为本设计与验收的依据：

| # | 设计语言 | Phase 8 落实 |
|---|----------|--------------|
| 1 | **深色左侧固定导航** | LeftNav 采用深色主题，常驻可达项目/阶段/功能（§4） |
| 2 | **浅色主工作区** | MainWorkspace 采用浅色主题，承载当前焦点高密度内容（§5） |
| 3 | **顶部指标概览** | 阶段工作区顶部 StageOverviewBar 呈现关键指标（§5.2） |
| 4 | **中部筛选 / 分组 / 切换** | StageFilterBar 提供按 confidence/source_kind/severity/view type 的快速收敛（§5.3） |
| 5 | **对象列表可展开** | evidence/claim/node/issue 以可展开列表呈现，展开显示追溯字段（§6） |
| 6 | **信息密度高但层级清晰** | 卡片 + 间距 + 字号 + 颜色建立层级，拒绝空旷与混乱（§3/§6） |
| 7 | **蓝色强调** | 主操作/当前焦点/可追溯链接统一蓝色系；不用红绿裁决色（§8） |
| 8 | **卡片用于对象和面板** | 内容用卡片承载，不做营销式大留白页面（§6） |
| 9 | **不再纵向堆叠所有面板** | 导航 + 分区 + Tab 替代长滚动（§3） |

> 参考约束：本设计服从技术栈（Tauri + React/TS，视图 SVG+CSS）与产品定位。AgentScope 的"指标概览/对象列表"在本项目中映射为"阶段理解指标 + evidence/claim/node 列表"，**不是**把产品做成监控 dashboard。
>
> 只参考 AgentScope 的信息密度与布局方式，**不照搬其品牌或无关功能**。

## 3. 信息架构与布局

### 3.1 三段式骨架

```text
┌─────────────┬──────────────────────────────────────────────┐
│  LeftNav    │  WorkspaceTopBar（项目/阶段标题、session 状态）│
│  (深色固定)  ├──────────────────────────────────────────────┤
│             │  StageOverviewBar（顶部指标概览）              │
│  项目入口    ├──────────────────────────────────────────────┤
│  阶段列表    │  StageFilterBar（筛选/分组/视图切换）          │
│  功能入口    ├──────────────────────────────────────────────┤
│             │  StageContentArea（分区内容）                  │
│             │   Overview / Evidence / Understanding /        │
│             │   Views / Trace / Q&A / Quality                │
└─────────────┴──────────────────────────────────────────────┘
```

- **LeftNav**：固定宽度、深色、常驻。
- **MainWorkspace**：自适应剩余宽度、浅色，内含 **Artifact tabs**（Overview/Evidence/Understanding/Views/Trace/Q&A/Quality，**功能等价迁移、不长堆叠**）+ 顶部概览 + 中部筛选。
- **ContextPanel（右侧）**：承载当前选中的 evidence / trace / source excerpt / quality issue，与 Artifact tabs 联动。
- **不再**在 MainWorkspace 同屏纵向堆叠全部维度（含质量报告/trace/source excerpt/Q&A）；用 Artifact tabs + ContextPanel 聚焦。

### 3.2 响应式与最小宽度

- 桌面优先（macOS / Linux）；定义最小可用宽度，低于阈值时 LeftNav 可折叠为图标条。
- 不做移动端 / Web 响应式（非目标）。

## 4. 左侧导航（深色）

### 4.1 内容

- **顶部**：当前项目名 + 切换/最近项目入口。
- **中部**：阶段列表（L0~L6 + RTL 等），每项含 StageStatus 视觉标记 + 当前选中高亮（蓝色）。
- **底部**：当选中阶段时，显示该阶段功能区快捷入口（Overview/Evidence/.../Quality），等价于 stage_tab 快速跳转。

### 4.2 阶段状态视觉标记

| StageStatus | 标记 | 含义 |
|-------------|------|------|
| available | 实心圆（蓝） | 可用 |
| empty | 空心圆（灰） | 空阶段 |
| missing | 虚线圆（灰） | 缺失 |
| naming_anomaly | 三角（琥珀） | 命名异常已识别 |
| unreadable | 锁形（灰） | 不可读 |

> 标记色不用红绿"通过/失败"；琥珀仅表"需注意"，非裁决。

## 5. 主工作区（浅色）

### 5.1 WorkspaceTopBar

- 项目名 / 当前阶段名 / session 保存状态。
- 全局提示位（source_changed 横幅、source_missing 错误）。
- 主操作（如"重新生成"）。

### 5.2 StageOverviewBar（顶部指标概览）

当选中阶段时，顶部一行关键指标（卡片或紧凑指标块）：

- evidence 数 / 覆盖档位
- claim 数 / confidence 分布（confirmed/supported/inferred/unknown/conflicting 计数）
- 视图节点/边数（structure/dataflow/timing）
- quality acceptance（meets_gate / below_gate）+ 主要 issue 分类计数

> 指标为内部质量与规模提示，**不对目标项目做评价**；带评分/比例的指标标注"内部质量指标"。

### 5.3 StageFilterBar（中部筛选 / 分组 / 切换）

- Views Tab：structure / dataflow / timing 子切换。
- Evidence/Understanding/Quality：按 confidence / source_kind / severity / kind 筛选；可分组。
- 筛选状态为瞬态（当前阶段内有效，不持久化）。

## 6. 对象列表与卡片

### 6.1 可展开对象列表

- Evidence / Claim / ViewNode / QualityIssue 等对象以**可展开列表**呈现（AgentScope 风格 #5）。
- 折叠态：标题 + 关键标签（confidence / source_kind / severity / evidence_id）。
- 展开态：追溯字段（source_path / line_range / claim_id / node_id / description）+ 操作（跳转证据 / 追问）。

### 6.2 卡片化

- 对象与面板用**卡片**承载（#8）：统一圆角、边框、内边距、阴影层级。
- 卡片分级：主内容卡 / 概览指标卡 / 详情卡，层级用阴影/边框区分。
- 不做营销式大留白、大图、英雄区。

### 6.3 密度与可读性（#6）

- 紧凑但可读：字号阶梯（标题/正文/次要/标签）、行高、间距规范统一。
- 高对比度，深色导航与浅色主区对比明确。
- 卡片间距建立分组层级，避免"一片混沌"。

## 7. 视觉语义系统

### 7.1 confidence 视觉（延续 Phase 3/4/5，统一应用）

| confidence | 视觉 |
|------------|------|
| confirmed | 强调色实心标签（高确信） |
| supported | 中性强调标签 |
| inferred | 弱化/描边标签（提示推断） |
| unknown | 灰色 + 问号图标 |
| conflicting | 琥珀/双箭头图标（冲突提示） |

### 7.2 证据强度 / severity / unknown

- evidence strength（direct/indirect）：用图标或描边粗细表达，不用红绿。
- quality severity（High/Medium/Low）：High 加粗强调、Medium 普通、Low 弱化；**不用 PASS/HOLD 红绿裁决色**。
- unknown / evidence_gap：灰色 + 图标，与 confirmed 视觉区隔明显。
- 正向 guardrail（hallucinated_claim_blocked）：中性"守卫生效"标记，非通过/失败。

### 7.3 退化视图表达

- empty timing（ExpectedEmptyTiming）：就地提示 + empty_reason，不显示空图骨架。
- 孤立节点（IsolatedOrUnconnectedView）：节点标记 + Low 提示。
- 标签重复（LowSemanticDiversity）：列表提示。

> 视觉语义只表达"质量提示强度"与"不确定性"，**绝不表达"目标项目通过/失败"**。

## 8. 蓝色强调规范（#7）

- **蓝色用于**：主操作按钮、当前选中态（阶段/节点/Tab）、可追溯链接、焦点边框。
- **蓝色不用于**：confidence 裁决、通过/失败、目标项目评价。
- 强调色单一蓝色系（含 hover/active/disabled 状态），不混用多色强调。
- 危险/错误用中性警示色（如琥珀/灰），不用红绿对立。

## 9. 交互状态

| 状态 | 处理 |
|------|------|
| 选中节点/对象 | 蓝色高亮 + 触发 Trace/详情联动 |
| hover | tooltip（节点/evidence 摘要），不遮挡主内容 |
| 加载中 | 区域 loading 骨架/指示，禁用重复触发 |
| 列表展开/折叠 | 当前阶段内记忆（瞬态） |
| 对话框（确认/删除） | 遮罩 + 焦点锁定 + 明确确认/取消 |
| 旧响应 | 不回写（guard，§UI 状态文档） |

## 10. 空 / 错误状态设计

| 状态 | 视觉 | 文案方向 | 操作 |
|------|------|----------|------|
| 无项目 | 项目概览空状态插画/图标 | 引导打开项目 / 最近项目 | 打开项目 |
| 空阶段 | 空状态卡 | "该阶段为空（empty）" | 查看相邻阶段 |
| 命名异常 | 琥珀标记 + 说明 | "命名非标准，已识别为阶段" | 进入分区 |
| 源码变更 | TopBar 横幅（琥珀） | "源码已变更，产物可能过期" | 重新生成 |
| 路径不允许 | 错误卡 | "源码路径不在允许范围" | 检查路径 |
| timing 诚实空 | Views 内提示 | "无时序证据，未生成 timing 图" | 查看 dataflow |
| Q&A 无证据 | Q&A 内提示 | "证据不足，无法回答" | 查看缺口 |
| session 加载失败 | 错误卡 + 可恢复 | "会话加载失败" | 重试 / 新建 |

> **warnings 降噪**：scan_timeout 等警告不再长期占据底部横幅，而是**折叠 / 分类 / 计数 / 可展开查看**（默认折叠为计数徽标，展开看详情），与上述空/错误状态分层呈现，不喧宾夺主。

## 11. 文案规范

- **禁用**："正确""错误""PASS""HOLD""审计结论""通过/失败裁决"等用语。
- **使用**：客观描述工具状态与不确定性，如"该声明缺少引用真实证据""工具未能基于已有证据回答""覆盖率：x%（内部质量指标）"。
- 带评分/比例的文案标注"内部质量指标，不代表目标项目质量"。
- 默认简体中文（AGENTS.md §3）。

## 12. 前后端边界

- 工作台数据来自既有 Phase 1~7 产物（经 Tauri command 只读读取），前端不重新计算语义结论。
- 展示性数据（指标概览/筛选）优先前端从既有产物派生；新增 command 仅在必要时且不破坏契约（见架构文档 §6）。
- TypeScript 类型从既有模型派生，视觉语义不改模型字段语义。

## 13. 安全边界

- UI 不触发对目标项目的写入；追溯点击只读定位。
- 不调用真实 LLM；不读取 `api_key`。
- 不运行 Vivado / synthesis / implementation / bitstream。
- 文案不输出审计裁决；视觉语义不表达"目标项目通过/失败"。
- 不引入重可视化库（视图 SVG+CSS）。

## 14. 关联文档

- [`../requirements/phase-8-product-workbench-requirements.md`](../requirements/phase-8-product-workbench-requirements.md) — 需求（draft）
- [`../design/phase-8-workbench-architecture.md`](../design/phase-8-workbench-architecture.md) — 工作台架构（draft）
- [`../design/phase-8-ui-state-and-navigation-design.md`](../design/phase-8-ui-state-and-navigation-design.md) — UI 状态与导航（draft）
- [`../testing/phase-8-product-workbench-validation.md`](../testing/phase-8-product-workbench-validation.md) — 验证与验收（draft）
- [`../planning/phase-8-implementation-plan.md`](../planning/phase-8-implementation-plan.md) — 编码实施计划（draft）
- [`phase-7-quality-review-view.md`](phase-7-quality-review-view.md) — Phase 7 Quality Review 视图（active，复用其质量面板）

## 15. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-16 | 初始 draft：正式写入 AgentScope 风格 9 条设计语言为依据；三段式骨架；深色左导航 + 阶段状态标记；浅色主区 + 顶部概览 + 中部筛选；可展开对象列表 + 卡片化；confidence/strength/unknown/severity 视觉语义系统；蓝色强调规范；空/错误状态；文案规范；前后端边界；安全边界。审核转 active 后方允许编码。 | Claude |
| 2026-06-16 | 审核通过，status 从 draft 转 active；允许进入 Phase 8 Batch A（P8-T01~P8-T02）；Phase 8 编码尚未开始；Batch B/C/D/E 与 Phase 9/10/11 未开始。 | Claude |
