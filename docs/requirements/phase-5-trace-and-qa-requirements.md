# Phase 5 证据回链与 Grounded Q&A 需求

---
status: active
updated: 2026-06-14
---

> 本文档定义 Phase 5「证据回链与 grounded Q&A」的产品需求。Phase 5 的核心是：用户点击视图节点/边、claim 或 evidence 后，能够追溯到 evidence item、源码文件和行号；用户可以基于当前阶段的理解结果提出问题，系统给出有 evidence grounding 的回答，并明确区分 unknown / inferred / evidence_gap。
>
> 本文档已收口（status=active），是 Phase 5 编码依据。

## 1. 一句话目标

让 Phase 4 生成的三类视图和 Phase 3 生成的结构化理解结论真正可追溯到源码证据，并支持用户在证据边界内提出有 grounding 的追问。

## 2. 功能点

### F5-001 evidence_id / claim_id / node_id / edge_id 统一选择模型

| 维度 | 说明 |
|------|------|
| **用户动作** | 在前端点击视图节点、边、claim chip、evidence chip 中的任意可点击元素 |
| **前端输入** | 一个 `SelectedTraceTarget` 对象（见 `phase-5-trace-model.md`），描述被点击目标的类型和 ID |
| **后端输入** | 当前阶段的 `EvidenceCollection` + `ImplementationUnderstanding` + `ViewGraph[]` |
| **输出/状态变化** | 前端进入 trace 选中态，右侧/下方出现 TracePanel，展示该目标可解析到的 claim/evidence/source 列表 |
| **验收标准** | 1. 点击 node/edge/claim/evidence 均能生成有效的 `SelectedTraceTarget`；2. target 类型与 ID 一一对应；3. 切换选择时旧选择态清除；4. 切换阶段时选择态清空 |
| **非目标** | 不统一成自由文本搜索；不把 node_id 与 evidence_id 混为一谈 |

### F5-002 从 ViewGraph 节点/边追溯 trace_refs

| 维度 | 说明 |
|------|------|
| **用户动作** | 在 MultiViewPanel 中点击某个节点或边 |
| **前端输入** | `ViewNode.node_id` 或 `ViewEdge.edge_id` + 当前 `ViewGraph` |
| **后端输入** | `ViewGraph` + `EvidenceCollection` + `ImplementationUnderstanding` |
| **输出/状态变化** | TracePanel 展示该 node/edge 的 `trace_refs` 解析结果：每条 trace 对应到 claim 描述、evidence_id、confidence、relevance |
| **验收标准** | 1. 有 trace_refs 的节点/边能列出所有 claim/evidence；2. trace_refs 为空时显示"无证据追溯"；3. 解析失败的 evidence_id 显示为"证据缺失"；4. 点击某条 trace 可定位到 F5-005 source excerpt |
| **非目标** | 不在视图中直接显示源码片段；不修改 ViewGraph 的 trace_refs 结构 |

### F5-003 从 Understanding claim 追溯 evidence_refs

| 维度 | 说明 |
|------|------|
| **用户动作** | 在 UnderstandingPanel 中点击某条 claim 的 evidence chip |
| **前端输入** | `ImplementationClaim.claim_id` + 当前 `ImplementationUnderstanding` |
| **后端输入** | `ImplementationUnderstanding` + `EvidenceCollection` |
| **输出/状态变化** | TracePanel 高亮该 claim，并列出其 `evidence_refs` 每条引用的 evidence_id、relevance、对应 source_path/line_range |
| **验收标准** | 1. claim 的所有 evidence_refs 可展开；2. evidence_id 必须真实存在；3. has_evidence_gap=true 的 claim 显示 gap 说明；4. 点击 evidence_id 可进入 F5-005 source excerpt |
| **非目标** | 不重新生成 claim；不修改 claim 的 confidence |

### F5-004 EvidencePanel 高亮与定位

| 维度 | 说明 |
|------|------|
| **用户动作** | 在 TracePanel 中点击某条 evidence_id，或在视图节点/边中点击 evidence chip |
| **前端输入** | `evidence_id` + 当前 `EvidenceCollection` |
| **后端输入** | `EvidenceCollection`（无需额外后端调用即可高亮） |
| **输出/状态变化** | EvidencePanel 滚动到对应 evidence item 并高亮；若 EvidencePanel 未展开则自动展开 |
| **验收标准** | 1. 点击 evidence_id 后 EvidencePanel 中高亮对应 item；2. 高亮持续至用户切换选择或阶段；3. EvidencePanel 未打开时自动展开；4. 不存在的 evidence_id 给出错误提示 |
| **非目标** | 不高亮整个文件；不打开外部编辑器 |

### F5-005 Source excerpt / source location 展示

| 维度 | 说明 |
|------|------|
| **用户动作** | 在 TracePanel 或 EvidencePanel 中点击"查看源码片段" |
| **前端输入** | `evidence_id` 或直接的 `source_path` + `line_range` |
| **后端输入** | 目标项目文件系统（只读），通过 Tauri command `get_source_excerpt` 读取 |
| **输出/状态变化** | SourceExcerptPanel 展示：文件路径、语言、行号范围、源码片段（最多 N 行）、截断提示 |
| **验收标准** | 1. 行号与编辑器一致（1-based 闭区间）；2. 超大文件/超大范围自动截断；3. 二进制/非 UTF-8/越界路径拒绝并给出 warning；4. source_path 必须属于当前 workspace root |
| **非目标** | 不做语法高亮（MVP）；不编辑源码；不打开外部编辑器 |

### F5-006 Grounded Q&A 输入、上下文范围、回答结构

| 维度 | 说明 |
|------|------|
| **用户动作** | 在 GroundedQAPanel 中输入问题并提交 |
| **前端输入** | `question: string` + 可选 `selected_trace_target: SelectedTraceTarget` + 当前 `ImplementationUnderstanding` + `EvidenceCollection` |
| **后端输入** | `GroundedQuestion` 对象（见数据模型） |
| **输出/状态变化** | `GroundedAnswer` 对象：回答文本 + `claims[]`（每条含 confidence + citations/reason）+ `warnings[]` + `confidence` |
| **验收标准** | 1. 对 confirmed / supported / inferred / conflicting 的 answer claim，必须至少有一个有效 citation；2. 对 unknown answer / unknown claim，不允许伪造 citation，可以没有 citations，但必须包含 reason，并产生 `GroundedQaWarning`（如"当前阶段证据不足"）；3. citation 可点击跳转至 source excerpt；4. 无证据支撑时回答整体标注 unknown；5. 回答不超出当前阶段上下文 |
| **非目标** | 不做自由聊天；不跨阶段回答；不默认调用真实云端 LLM（先使用 MockProvider 验证数据结构和 UI 闭环）；unknown 回答不为过校验而强行引用无关 evidence |

### F5-007 unknown / inferred / evidence_gap 表达

| 维度 | 说明 |
|------|------|
| **用户动作** | 浏览 trace 结果或 Q&A 回答 |
| **前端输入** | 来自 resolver 的 `TraceRefResolved` 或 `GroundedAnswerClaim` |
| **后端输入** | `ImplementationUnderstanding` 中的 unknowns / evidence_gaps |
| **输出/状态变化** | UI 明确显示 confidence 标签：confirmed / supported / inferred / unknown / conflicting；evidence_gap 单独列出 |
| **验收标准** | 1. 每种 confidence 有固定颜色/线型语义；2. unknown 不可隐藏；3. evidence_gap 说明可见；4. Q&A 回答中的 unknown 必须说明原因，且不得把 inspected evidence 伪装成支撑证据；5. unknown 回答/claim 不产生伪造 citation |
| **非目标** | 不把 unknown 自动提升为 inferred；不做 PASS/HOLD 判断；unknown 回答不为过校验而强行引用无关 evidence |

### F5-008 问答安全边界：不修改目标项目、不运行脚本、不做 PASS/HOLD

| 维度 | 说明 |
|------|------|
| **用户动作** | 使用 Q&A 功能 |
| **前端输入** | 用户问题 |
| **后端输入** | `GroundedQuestion` |
| **输出/状态变化** | `GroundedAnswer` |
| **验收标准** | 1. Q&A 过程中不写入目标项目；2. 不运行 Vivado / synthesis / implementation / bitstream；3. 回答中不出现"正确/错误""PASS/HOLD"等审计结论；4. 对越界问题回答"当前上下文无法回答" |
| **非目标** | 不做代码审查结论；不自动修复代码；不把 Q&A 历史写回目标项目 |

## 3. Phase 5 明确不做

- **不打开外部编辑器**：所有 source excerpt 在应用内 SourceExcerptPanel 展示。
- **不修改源码**：对目标项目只读，任何写入操作必须通过用户显式授权且不在 Phase 5 范围内。
- **不运行 Vivado / synthesis / implementation / bitstream**。
- **不默认调用真实云端 LLM**：Phase 5 先实现 provider trait + MockProvider，真实 LLM 配置留到后续明确设计。
- **不做代码正确/错误判断**：不输出"这段代码是对的/错的"。
- **不做跨阶段对比**：除非只在文档"后续扩展"中说明，否则 Phase 5 只基于当前选中阶段。
- **不做持久化**：qa_history 的数据结构可定义，但持久化实现属于 Phase 6。
- **不做自动布局增强**：视图渲染仍沿用 Phase 4 的 SVG + layout_hints。

## 4. 关联文档

- [`phase-5-trace-model.md`](../design/phase-5-trace-model.md) — 数据模型
- [`phase-5-trace-and-qa-design.md`](../design/phase-5-trace-and-qa-design.md) — 后端与 command 设计
- [`phase-5-trace-and-qa-view.md`](../ui-ux/phase-5-trace-and-qa-view.md) — UI/UX 设计
- [`phase-5-trace-and-qa-validation.md`](../testing/phase-5-trace-and-qa-validation.md) — 验证设计
- [`phase-5-implementation-plan.md`](../planning/phase-5-implementation-plan.md) — 实施计划

## 5. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-13 | 初始创建（draft）：定义 F5-001~F5-008 及非目标 | Claude |
