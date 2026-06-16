# Phase 8 产品级工作台需求

---
status: draft
updated: 2026-06-16
---

> 本文档定义 Phase 8（产品级 UI/UX 工作台重构）的产品需求。
>
> **Phase 8 的目标不是新增语义分析能力，而是把 Phase 1~7 已有的能力重新组织成产品级本地理解工作台。** Phase 7 已在真实 `ai_project_template` 项目上把分析质量提升到可信（dataflow 非空可追溯、timing 诚实表达、quality 信号可分类），但当前 UI 仍是工程调试式长页面堆叠，没有把这些能力组织成"用户能高效理解一个阶段"的工作台。Phase 8 解决这个产品可用性缺口。
>
> 本文档 status 为 `draft`，是 Phase 8 需求的编制草案，**审核转 `active` 后才允许进入 Phase 8 编码**。Phase 8 编码尚未开始。

## 1. 用户目标

- 用户打开项目后，被工作台自然引导完成"理解一个阶段"，而不是面对一堆按后端能力划分的调试面板自己拼流程。
- 用户在**同一工作台上下文**中完成"看概览 → 看图 → 点节点 → 读证据 → 追问"闭环，不再在纵向长页面里上下翻找。
- 用户能一眼区分"哪些结论由强证据支持、哪些是推断、哪些是未知"，不被高置信度推断误导。
- 用户能快速在项目 / 阶段 / 功能区之间切换，最近项目与阶段可达一次点击。
- session / 最近项目 / 空状态 / 错误状态达到产品级可用度，降低使用摩擦。
- 用户感知到工作台"信息密度高但层级清晰"——既不空旷浪费屏幕，也不混乱难读。

## 2. 业务背景

### 2.1 当前 UI 痛点（MVP / Phase 7 后）

Phase 7 已提升真实项目分析质量，但 UI 仍停留在工程调试形态。具体痛点（详见 [`phase-8-overview-product-ui-workbench.md`](../planning/phase-8-overview-product-ui-workbench.md) §1）：

| 痛点 | 现状 |
|------|------|
| 单页长堆叠 | `WorkspacePage.tsx` 约 1300 行，把文件列表、证据、理解、三类视图、trace、Q&A、质量报告、警告全部纵向堆叠在一个长页面里 |
| 面板按后端能力划分 | EvidencePanel / TracePanel / MultiViewPanel / GroundedQAPanel / QualityReviewPanel 等按"后端产出"组织，而非按"用户理解一个阶段"的认知流程 |
| 阶段工作流不连贯 | 收集证据、生成理解、生成视图是分散步骤，用户需手动串联，缺乏一体化引导 |
| 导航薄弱 | 无固定主导航；项目/阶段切换路径长；最近项目体验单薄 |
| 视觉层级薄弱 | confidence / 证据强度 / unknown / evidence_gap 未被一致地层级化表达 |
| 空/错误状态粗糙 | `source_changed` / `source_missing` / `stage_empty` 有横幅但体验粗糙 |

用户反馈归结为：**"界面粗糙、功能看起来弱、实际项目分析可用性仍存疑"**——即便 Phase 7 已让分析可信，UI 形态掩盖了这份可信度。

### 2.2 参考形态：AgentScope 风格工作台

Phase 8 UI 以用户提供的 AgentScope 风格截图为参考形态，提炼为以下设计语言（详见 [`phase-8-product-workbench-view.md`](../ui-ux/phase-8-product-workbench-view.md)）：

- **深色左侧固定导航**：项目 / 阶段 / 功能区常驻可达；
- **浅色主工作区**：承载当前焦点的高密度内容；
- **顶部指标概览**：阶段级关键指标一眼可见；
- **中部筛选 / 分组 / 切换**：对对象列表与视图做快速过滤；
- **对象列表可展开**：evidence / claim / node 等对象以可展开列表呈现，层级清晰；
- **信息密度高但层级清晰**：不空旷、不混乱；
- **蓝色强调**：主操作、当前焦点、可追溯链接统一蓝色系；
- **卡片用于对象和面板**：内容用卡片承载，不做营销式大留白页面；
- **不再纵向堆叠所有面板**：用导航 + 分区 + Tab 替代长滚动。

> 参考是"形态与信息架构风格"，不是逐像素复刻。Phase 8 仍服从本项目产品定位（理解与可视化工具，非审计 dashboard，非大而全可视化平台）。

## 3. 功能点

### R8-001 产品级工作台信息架构

| 维度 | 说明 |
|------|------|
| **目标** | 用"导航 + 主工作区 + 分区"的三段式信息架构替代单页长堆叠；主工作区按当前焦点（项目 / 阶段 / 功能）切换内容，而非把所有面板纵向铺开 |
| **输入** | 用户当前选中的项目、阶段、功能区 |
| **输出** | 工作台布局：固定导航 + 主工作区（含顶部概览、中部筛选、对象/视图区） |
| **验收标准** | 单一视图不再同时纵向呈现文件列表 + 证据 + 理解 + 三类视图 + trace + Q&A + 质量报告；任一时刻主工作区聚焦一个明确焦点 |
| **非目标** | 不做成多窗口 / 多标签浏览器；不做营销式落地页 |

### R8-002 左侧项目 / 阶段 / 功能导航

| 维度 | 说明 |
|------|------|
| **目标** | 深色左侧固定导航，分"项目 / 阶段 / 功能"层级；支持最近项目、阶段快速切换、当前阶段高亮 |
| **输入** | `WorkspaceProfile`（阶段列表 + StageStatus）、最近项目（session 列表）、当前选中 |
| **输出** | 常驻导航树：项目入口 → 阶段列表（含 empty/missing/naming_anomaly 标记）→ 功能区入口 |
| **验收标准** | 从任意工作区状态一次点击可达：最近项目、任一阶段、当前阶段的功能区；阶段状态（available/empty/missing/naming_anomaly）有视觉区分 |
| **非目标** | 不做跨项目对比导航（属 Phase 10+）；不做多项目同时打开 |

### R8-003 阶段工作区分区

| 维度 | 说明 |
|------|------|
| **目标** | 选中阶段后，主工作区按"概览 / 证据 / 理解 / 视图 / 追溯 / 问答 / 质量"分区呈现，而非全部堆叠；用户在分区内完成该维度的理解任务 |
| **输入** | 阶段的 `EvidenceCollection` / `ImplementationUnderstanding` / `ViewGraph[]` / trace 产物 / Q&A 历史 / `QualityReport` |
| **输出** | 阶段工作区 Tab（或等价分区）：Overview / Evidence / Understanding / Views / Trace / Q&A / Quality |
| **验收标准** | 每个分区聚焦一个维度，分区之间可一键切换且保留当前节点/对象选中态；退化视图、空 timing、quality issue 在对应分区内就地表达 |
| **非目标** | 不强制固定分区顺序为唯一形态（详细设计可收敛为 Tab / 分栏，但不得回到长堆叠） |

### R8-004 阶段理解一体化工作流

| 维度 | 说明 |
|------|------|
| **目标** | 把"收集证据 → 生成理解 → 生成视图 → trace 回链 → 追问"整合为顺滑的单阶段理解流，减少手动串联；工作流状态可视、可续 |
| **输入** | Phase 1~5 既有 commands（collect_evidence / generate_understanding / generate_views / resolve_trace / ask_grounded_question） |
| **输出** | 工作台对阶段理解流的引导：步骤可见、产物就绪后自动衔接、过期产物有提示 |
| **验收标准** | 用户无需记忆"下一步点哪个按钮"；产物过期（源码变更）有明确提示与一键重跑；重新生成时下游产物正确失效（延续 Phase 7 P2 状态隔离） |
| **非目标** | 不做全自动批量流水线（仍由用户驱动单阶段）；不改后端 command 语义 |

### R8-005 顶部指标概览 + 中部筛选 + 可展开对象列表

| 维度 | 说明 |
|------|------|
| **目标** | 阶段工作区顶部呈现关键指标概览（如 evidence 数、claim 数、视图节点/边数、quality acceptance、coverage 档位）；中部提供筛选 / 分组 / 视图切换；evidence / claim / node / issue 等对象以可展开列表呈现 |
| **输入** | 阶段各维度产物统计 |
| **输出** | 概览指标条 + 筛选/分组控件 + 可展开对象列表 |
| **验收标准** | 关键指标一眼可见；筛选能按 confidence / source_kind / severity 等维度收敛对象列表；对象列表展开后呈现追溯详情（evidence_id / source_path / line_range / claim_id / node_id） |
| **非目标** | 不做自定义 dashboard 编辑器；指标仅作内部质量与规模提示，不对目标项目做评价 |

### R8-006 卡片化对象 / 面板与蓝色强调视觉系统

| 维度 | 说明 |
|------|------|
| **目标** | 内容用卡片承载（evidence 卡、claim 卡、node 卡、issue 卡、面板卡）；主操作 / 当前焦点 / 可追溯链接统一蓝色强调；整体信息密度高但层级清晰 |
| **输入** | 各维度产物对象 |
| **输出** | 统一的卡片样式与蓝色强调规范（详见 UI/UX 文档） |
| **验收标准** | 同类对象卡片样式一致；蓝色仅用于可操作 / 可追溯 / 当前焦点，不滥用；高密度下仍可读（间距、字号、对比度达标） |
| **非目标** | 不引入营销式大留白页面；不追求炫技视觉 |

### R8-007 confidence / 证据强度 / unknown 视觉语义层级系统

| 维度 | 说明 |
|------|------|
| **目标** | 建立并一致应用 confidence（confirmed/supported/inferred/unknown/conflicting）、evidence strength（direct/indirect）、unknown/evidence_gap、quality severity 的视觉层级（颜色 / 图标 / 字重 / 标签） |
| **输入** | 既有 confidence 枚举、EvidenceStrength、QualityIssue severity、unknown/gap 标记 |
| **输出** | 视觉语义规范（详见 UI/UX 文档 §视觉语义），并在 evidence / claim / node / issue / Q&A 一致应用 |
| **验收标准** | 用户能一眼区分强证据结论、推断、未知、冲突；视觉语义在工作台各分区一致，不与"目标项目通过/失败"语义混淆 |
| **非目标** | 不用 PASS/HOLD/通过/失败红绿裁决色；视觉表达改变不等于模型改变 |

### R8-008 产品化 session / 最近项目 / 空状态 / 错误状态

| 维度 | 说明 |
|------|------|
| **目标** | 最近项目、session 列表、加载 / 错误恢复、空状态达到产品级可用度；对 `source_changed` / `source_missing` / `source_path_not_allowed` / 空阶段 / 命名异常阶段给出清晰可操作的反馈 |
| **输入** | Phase 6 session 持久化产物、workspace warnings、StageStatus |
| **输出** | 产品级空 / 加载 / 错误 / 降级状态设计（详见 UI/UX 文档） |
| **验收标准** | 每种异常状态有明确文案 + 可操作下一步；session 加载失败可恢复；命名异常 / 空阶段不阻断合理使用 |
| **非目标** | 不改变持久化 schema 与安全边界（延续 Phase 6） |

### R8-009 既有能力零回归

| 维度 | 说明 |
|------|------|
| **目标** | UI 大重构不得破坏 Phase 4/5/6/7 已验收能力：三类视图渲染、trace 回链、source excerpt、Grounded Q&A、session 持久化与恢复、Quality Review、真实项目识别与深层扫描 |
| **输入** | Phase 1~7 既有产物与 commands |
| **输出** | 完整回归验收通过（详见测试文档） |
| **验收标准** | 重构后真实项目（主样本 `fpga_project_coarse_sync` / 副样本 `fpga_project_fft`）仍能完成完整链路；既有自动化测试与桌面验收不退化 |
| **非目标** | 不借重构改语义模型 / 分析能力 |

### R8-010 Phase 8 退出标准

| 维度 | 说明 |
|------|------|
| **目标** | 定义 Phase 8 何时视为完成：工作台信息架构落地、AgentScope 风格视觉系统应用、真实用户可用性验收通过、既有能力零回归、completion review 转 active |
| **输入** | R8-001~R8-009 验收结果 |
| **输出** | Phase 8 完成判定与允许进入 Phase 9 的条件 |
| **验收标准** | 见 §7 与 [`phase-8-product-workbench-validation.md`](../testing/phase-8-product-workbench-validation.md) |
| **非目标** | 不承诺一次重构到"完美"；遗留体验问题以已知限制形式记录 |

## 4. 信息架构与导航要求（AgentScope 风格细化）

> 本节把 §2.2 的参考形态细化为可验收的信息架构要求。视觉与交互细节见 UI/UX 文档。

1. **三段式骨架**：固定左侧导航 + 主工作区 + （主工作区内）顶部概览/中部筛选/对象视图区。不得回到单页纵向长堆叠。
2. **左侧导航内容**：项目入口（含最近项目快速切换）、阶段列表（含 StageStatus 视觉标记）、当前阶段功能区入口。
3. **主工作区焦点**：任一时刻聚焦一个明确对象（一个阶段的一个分区），不在同屏并列堆叠全部维度。
4. **对象可展开列表**：evidence / claim / view node / quality issue 等以可展开列表呈现，展开后显示追溯字段。
5. **筛选 / 分组 / 切换**：对对象列表与视图提供按 confidence / source_kind / severity / view type 的快速收敛。
6. **密度与层级**：高信息密度（屏幕利用率高）但通过卡片、间距、字号、颜色建立清晰层级，避免混乱。
7. **蓝色强调约束**：蓝色限定于主操作、当前焦点、可追溯链接；不得用于表达"目标项目通过/失败"。

## 5. 异常 / 空状态

| 场景 | 处理 |
|------|------|
| 无项目打开 | 工作台空状态：引导打开项目 / 最近项目，不报错 |
| 阶段为空（empty） | 阶段工作区诚实显示空，不伪造产物；引导查看相邻阶段 |
| 阶段缺失（missing）/ 命名异常（naming_anomaly） | 视觉标记 + 说明，不阻断使用 |
| 源码变更（source_changed） | 明确提示产物过期 + 一键重跑，不静默使用旧产物 |
| 源码缺失 / 路径不允许（source_missing / source_path_not_allowed） | 清晰错误 + 可操作下一步 |
| 视图退化（empty timing / 孤立节点） | 就地诚实表达（延续 Phase 7 ExpectedEmptyTiming / IsolatedOrUnconnectedView 信号），不掩盖 |
| Q&A 无证据 | 诚实返回 unknown/gap，不伪造回答 |
| session 加载失败 | 可恢复提示，不崩溃 |

## 6. 证据与追溯要求（延续既有契约）

- 工作台所有用户可见的主要结论仍绑定 evidence_id / claim_id，可追溯到 `source_path` + `line_range`。
- 视觉语义系统不改变 confidence / strength / severity 的语义，只改变表达。
- Quality Report 仍描述"工具理解质量"，不描述"目标项目正确性"；不输出 PASS/HOLD/正确性裁决。
- 重构不得伪造或修改既有 evidence_id / claim_id / source_path / line_range 绑定。

## 7. Phase 8 退出标准

- R8-001~R8-010 全部定义并实现对应 UI / 状态结构调整。
- 真实用户（或代表用户的验收者）能无需说明完成"打开项目 → 选阶段 → 理解 → 查证据 → 追问"全流程。
- 信息架构以"理解一个阶段"为主线，单页长堆叠被替代。
- AgentScope 风格视觉系统（导航 / 概览 / 筛选 / 可展开列表 / 卡片 / 蓝色强调 / 密度层级）落地且一致。
- confidence / 证据强度 / unknown / evidence_gap 视觉语义一致。
- 既有能力（三类视图 / trace / Q&A / 持久化 / Quality Review / 真实项目识别）零回归。
- 全量 `npm run build` / `cargo test --lib` / `cargo check` 通过；前端类型检查通过。
- 真实桌面验收通过，安全边界保持。
- Phase 8 completion review 转 `active`。

## 8. 非目标

Phase 8 明确**不做**：

- **不改变核心语义模型**：`ImplementationUnderstanding`、evidence model、`confidence` 枚举、trace 模型字段语义与契约不变（视觉改变 ≠ 模型改变）。
- **不扩大分析能力**：不接真实 LLM（Phase 9）、不做跨阶段映射（Phase 10）、不做语义记忆（Phase 11）、不做测试覆盖图。
- **不引入重可视化库**：视图仍纯 SVG + CSS（沿用 Phase 4），不引入 React Flow / D3 / Mermaid，除非详细设计明确论证并审核。
- **不改安全边界**：目标项目只读、持久化只写 app-owned storage、不运行 Vivado/synthesis/implementation/bitstream。
- **不输出审计裁决**：不输出 PASS/HOLD/正确性裁决/审计结论；不评价目标项目正确性。
- **不做"大而全可视化平台"**（`product-scope.md` 非目标）：仍聚焦三类核心视图 + 证据 + Q&A 的可用化。
- **不做多窗口 / 多标签浏览器 / 自定义 dashboard 编辑器**。
- **不改变后端 command 的语义契约**：仅允许不改语义、不破坏既有契约的展示性结构调整。

## 9. 安全边界

- 目标项目只读：重构不引入任何对目标项目的写入。
- 不调用真实 LLM；不读取 `api_key`。
- 不运行 Vivado / synthesis / implementation / bitstream。
- 持久化只写 app-owned storage（延续 Phase 6）。
- 文案不输出审计裁决；视觉语义不表达"目标项目通过/失败"。
- 不引入外部网络库 / 重依赖偏离技术栈。

## 10. 关联文档

- [`../planning/phase-8-overview-product-ui-workbench.md`](../planning/phase-8-overview-product-ui-workbench.md) — Phase 8 overview（draft）
- [`../design/phase-8-workbench-architecture.md`](../design/phase-8-workbench-architecture.md) — 工作台架构设计（draft）
- [`../design/phase-8-ui-state-and-navigation-design.md`](../design/phase-8-ui-state-and-navigation-design.md) — UI 状态与导航设计（draft）
- [`../ui-ux/phase-8-product-workbench-view.md`](../ui-ux/phase-8-product-workbench-view.md) — 工作台 UI/UX 设计（draft）
- [`../testing/phase-8-product-workbench-validation.md`](../testing/phase-8-product-workbench-validation.md) — 验证与验收（draft）
- [`../planning/phase-8-implementation-plan.md`](../planning/phase-8-implementation-plan.md) — 编码实施计划（draft）
- [`../planning/post-mvp-roadmap.md`](../planning/post-mvp-roadmap.md) — Post-MVP 总体路线图（draft）
- [`../planning/phase-7-completion-review.md`](../planning/phase-7-completion-review.md) — Phase 7 完成状态（active，前置）

## 11. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-16 | 初始 draft：定义 R8-001~R8-010、AgentScope 风格信息架构细化、异常/空状态、退出标准、非目标、安全边界。审核转 active 后方允许进入 Phase 8 编码。 | Claude |
