# Phase 4 三类视图展示需求

---
status: draft
updated: 2026-06-12
---

> 本文档定义 Phase 4 "三类视图展示"的功能需求。Phase 4 消费 Phase 3 的 `ImplementationUnderstanding`，在前端生成结构图、数据流图和时序/流水图。

## 1. 用户目标

Phase 4 为 fpga-flow-mind 用户提供三重视图，帮助理解单个 FPGA 实现阶段：
- **结构关系**：该阶段有哪些模块/函数/接口/信号，它们之间如何组织
- **数据流向**：数据从哪里输入、经过什么处理步骤、输出到哪里
- **时序/流水**：处理步骤的执行顺序、流水线阶段、时钟相关约束

用户无需通读全部源码，通过三重视图即可理解阶段的整体实现逻辑。

## 2. Phase 4 输入

Phase 4 **只消费** Phase 3 的 `ImplementationUnderstanding`，不重复以下任何操作：
- 不重新扫描目标项目
- 不重新收集 evidence
- 不调用真实 LLM API
- 不修改目标项目文件

输入来源：`generate_understanding` Tauri command 返回的 `ImplementationUnderstanding`。

## 3. Phase 4 输出

### 3.1 结构图 (StructureView)

- 展示模块、函数/接口、信号之间的结构关系
- 节点类型：模块、函数/方法、接口定义、信号/端口
- 边类型：包含（containment）、调用（calls）、引用（references）
- 每个节点/边可追溯到 `claim_id` 或 `evidence_refs`
- 证据不足时标注 inferred / unknown / gap

### 3.2 数据流图 (DataflowView)

- 展示输入 → 处理步骤 → 输出的数据变换和流向
- 节点类型：输入源、处理步骤、中间数据、输出目标
- 边类型：数据流入、数据流出、依赖
- 每个节点/边可追溯到 `claim_id` 或 `evidence_refs`
- 无数据流信息时显式标注"数据流信息不足"

### 3.3 时序/流水图 (TimingView)

- 展示处理步骤的执行顺序、流水线阶段、clock/reset 相关线索
- 节点类型：处理阶段、流水级、时钟域、复位域
- 边类型：顺序依赖、流水传递、时钟驱动
- 每个节点/边可追溯到 `claim_id` 或 `evidence_refs`
- 无 timing 信息时显式标注，不强行推测

## 4. 功能点

### SV-001 结构图数据生成

| 维度 | 说明 |
|------|------|
| **输入** | `ImplementationUnderstanding`（module_summaries, signal_summaries, interface_summaries, claims） |
| **输出** | `ViewGraph`（structure 类型），含 nodes/edges + trace_refs |
| **后端责任** | 从 IU 确定性转换为 ViewGraph |
| **前端责任** | 渲染 ViewGraph 为可交互 SVG/HTML 结构图 |
| **状态** | 空 → 加载中 → 已展示 / 无数据 / 生成失败 |
| **验收标准** | 至少 3 个模块节点 + 关联信号节点可见；每个节点含名称 + confidence 标记 |
| **非目标** | 不实现自动布局引擎；不实现拖拽编辑 |

### SV-002 数据流图数据生成

| 维度 | 说明 |
|------|------|
| **输入** | `ImplementationUnderstanding`（processing_steps, claims, signal_summaries, interface_summaries） |
| **输出** | `ViewGraph`（dataflow 类型），含 nodes/edges + trace_refs |
| **后端责任** | 从 IU 确定性转换为 ViewGraph |
| **前端责任** | 渲染 ViewGraph 为可交互 SVG/HTML 数据流图 |
| **状态** | 空 → 加载中 → 已展示 / 无数据 / 生成失败 |
| **验收标准** | 至少 1 个输入 + 1 个处理 + 1 个输出节点可见；边带方向 |
| **非目标** | 不计算数据位宽；不推断隐式数据依赖 |

### SV-003 时序/流水图数据生成

| 维度 | 说明 |
|------|------|
| **输入** | `ImplementationUnderstanding`（processing_steps, claims, module_summaries） |
| **输出** | `ViewGraph`（timing 类型），含 nodes/edges + trace_refs |
| **后端责任** | 从 IU 确定性转换为 ViewGraph |
| **前端责任** | 渲染 ViewGraph 为可交互 SVG/HTML 时序流水图 |
| **状态** | 空 → 加载中 → 已展示 / 无数据 / 生成失败 |
| **验收标准** | 至少 1 条处理步骤顺序链可见；无 timing 信息时显式标注 |
| **非目标** | 不绘制波形图；不计算精确 latency cycle 数 |

### SV-004 三视图 Tab 切换

| 维度 | 说明 |
|------|------|
| **输入** | 三个 ViewGraph |
| **输出** | 三 tab 面板：结构图 / 数据流 / 时序流水 |
| **前端责任** | segmented control 或 tab bar 切换视图；切换不触发后端重新计算 |
| **后端责任** | 无（纯前端交互） |
| **状态** | 默认选中结构图 tab；三个 tab 始终可见 |
| **验收标准** | 点击 tab 切换视图，无闪烁；同屏保留 stage context |
| **非目标** | 不同时展示多视图；不实现拖拽分屏 |

### SV-005 Node/Edge 证据追溯展示

| 维度 | 说明 |
|------|------|
| **输入** | ViewNode / ViewEdge 中的 trace_refs |
| **输出** | hover tooltip 显示 claim_id + evidence_id + confidence |
| **前端责任** | hover/click 展示 trace 信息；不实现点击跳转源码（Phase 5） |
| **后端责任** | 无（trace_refs 已在 ViewGraph 中） |
| **状态** | hover tooltip；无 trace 时标注"无证据追溯" |
| **验收标准** | hover 节点显示关联 claim_id 和 evidence_id 列表 |
| **非目标** | 不实现点击打开源码；不实现 EvidencePanel 高亮回链（Phase 5） |

## 5. 异常 / 空状态

| 场景 | 行为 |
|------|------|
| IU 为 degraded mode | 三视图 tab 仍显示，但每个 view 标注"降级生成，视图数据不足" |
| claims 为空 | 结构图显示"无声明数据"，数据流图/时序图同理 |
| module_summaries 为空 | 结构图显示"无模块信息" |
| processing_steps 为空 | 数据流图/时序图显示"无处理步骤信息" |
| 三视图生成失败 | 对应 tab 内容显示错误面板，其他 tab 不受影响 |
| IU 尚未生成 | 三视图 tab 不显示（等待 generate understanding 完成后才出现） |

## 6. 证据与追溯要求

- 每个 ViewNode 必须包含 `trace_refs` 字段，指向原始 claim_id 和 evidence_id
- 每个 ViewEdge 必须包含 `trace_refs` 字段，指向关联的 claim
- 用户 hover 节点/边时显示关联的 claim_id / evidence_id / confidence
- 不实现点击跳转源码（Phase 5 实现）
- 不实现 EvidencePanel 高亮回链（Phase 5 实现）

## 7. MVP 验收标准

| # | 标准 | 验证方式 |
|---|------|----------|
| 1 | 三视图 tab 可切换 | 桌面验收 |
| 2 | 结构图展示模块/信号/接口节点 | 桌面验收 + 自动化测试 |
| 3 | 数据流图展示输入/处理/输出 | 桌面验收 + 自动化测试 |
| 4 | 时序流水图展示处理顺序 | 桌面验收 + 自动化测试 |
| 5 | 节点 hover 显示 evidence_id / claim_id | 桌面验收 |
| 6 | 空数据显式标注，不空白 | 自动化测试 |
| 7 | degraded mode 不崩溃 | 自动化测试 |
| 8 | 目标项目只读 | 自动化测试 |

## 8. 非目标

- 不实现自动布局算法（使用固定模板/简单网格布局）
- 不实现拖拽/缩放/平移（MVP 使用静态 SVG/CSS 布局）
- 不实现 evidence_id 点击跳转源码（Phase 5）
- 不实现 EvidencePanel 高亮回链（Phase 5）
- 不引入 React Flow / D3 / Mermaid / Cytoscape 等大型图形库
- 不做实时刷新/流式更新
- 不做跨阶段比较视图

## 9. 关联设计文档

- UI/UX：`docs/ui-ux/phase-4-multi-view-panel.md`
- 数据模型：`docs/design/phase-4-view-model.md`
- 生成器设计：`docs/design/phase-4-view-generator-design.md`
- 测试：`docs/testing/phase-4-view-validation.md`
- 实施计划：`docs/planning/phase-4-implementation-plan.md`

## 10. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-12 | 初始创建（draft）：定义 Phase 4 三类视图功能需求 SV-001~SV-005 | Claude |
