# Story: 查看时序/流水图

---
status: draft
updated: 2026-06-11
---

## 用户目标

作为 FPGA 开发人员，我希望看到选中阶段的时序/流水图，以便理解 latency、握手信号流动和流水线行为。

## 业务背景

时序/流水图回答"数据在该阶段如何随时间流动"：各 stage 的 latency 是多少、valid/ready 如何握手、是否存在流水线 overlap、状态机如何切换。这对于理解 RTL 阶段的实现尤为关键。

## 前置条件

- ImplementationUnderstanding 已生成
- timing_view 部分已填充

## 触发入口

用户在理解结果概览面板中点击"查看时序图"按钮，或在结构图/数据流图视图中切换到时序图标签页。

## 主流程

1. 用户触发时序图展示
2. 系统从 ImplementationUnderstanding 中提取 timing_view 数据
3. 系统生成可视化规范
4. 系统渲染时序/流水图
5. 用户与时序图交互（点击查看节点详情）

## 功能点清单

### VT-001 触发时序图展示

- **用户动作**：用户点击"查看时序图"按钮
- **系统必须**：
  - 切换到时序图视图面板
  - 显示加载状态（如有必要）
  - 如 timing_view 未生成，提示"时序信息未生成"
- **成功结果**：时序图视图面板展示
- **失败表现**：无数据时显示空状态提示
- **evidence 要求**：不需要
- **MVP 必须**：是

### VT-002 节点类型定义

- **系统动作**：定义时序图中支持的节点类型
- **系统必须**：时序图包含以下节点类型：
  - **流水线阶段节点**（PipelineStage）：流水线中的一个 stage
  - **寄存器节点**（Register）：数据寄存或状态保持点
  - **状态机状态节点**（FSMState）：有限状态机的状态
  - **握手信号节点**（Handshake）：valid/ready 等握手信号点
  - **时钟域节点**（ClockDomain）：不同时钟域的边界
- **成功结果**：节点类型清晰可区分
- **失败表现**：无
- **evidence 要求**：每个节点必须携带 evidence_refs
- **MVP 必须**：是

### VT-003 边类型定义

- **系统动作**：定义时序图中支持的边类型
- **系统必须**：时序图包含以下边类型：
  - **时序边**（Sequential）：表示一个时钟周期内的数据传递
  - **组合边**（Combinational）：表示组合逻辑路径（无时钟延迟）
  - **控制边**（Control）：表示控制信号流动（如 valid/ready）
  - **状态转换边**（Transition）：表示状态机状态转换
- **成功结果**：边类型清晰可区分
- **失败表现**：无
- **evidence 要求**：每条边必须携带 evidence_refs
- **MVP 必须**：是

### VT-004 节点携带 evidence_refs

- **系统动作**：确保每个时序节点都有 evidence 关联
- **系统必须**：
  - 每个 PipelineStage 节点携带 `evidence_refs`，指向对应的 RTL module 或 always 块
  - 每个 Register 节点携带 `evidence_refs`，指向寄存器声明
  - 每个 FSMState 节点携带 `evidence_refs`，指向状态定义
  - 用户点击节点时可查看关联的 evidence 列表
- **成功结果**：节点与 evidence 的关联可查询
- **失败表现**：无 evidence 关联的节点标注为 `unknown`
- **evidence 要求**：每个节点至少有一个 evidence_id 或标注为 `unknown`
- **MVP 必须**：是

### VT-005 边携带 evidence_refs

- **系统动作**：确保每条时序边都有 evidence 关联
- **系统必须**：
  - 每条时序边携带 `evidence_refs`，指向数据传递的代码位置
  - 每条控制边携带 `evidence_refs`，指向信号赋值
  - 用户点击边时可查看关联的 evidence 列表
- **成功结果**：边与 evidence 的关联可查询
- **失败表现**：无 evidence 关联的边标注为 `inferred`
- **evidence 要求**：每条边至少有一个 evidence_id 或标注为 `inferred`
- **MVP 必须**：是

### VT-006 时序图渲染

- **系统动作**：将 timing_view 数据渲染为可视化图
- **系统必须**：
  - 展示流水线 stage 的横向排列（时间轴方向）
  - 展示数据在 stage 间的传递路径
  - 展示握手信号（valid/ready）的流动
  - 如存在状态机，展示状态转换图
  - 不同节点类型使用不同的视觉样式
  - 时序边使用带箭头的实线，标注 latency（如"1 cycle"）
  - 组合边使用不带箭头的细线
  - 控制边使用带箭头的虚线
  - 节点标签显示 stage 名/信号名/状态名
  - 节点旁标注置信度指示器
- **成功结果**：用户可直观理解时序/流水行为
- **失败表现**：渲染失败时显示"无法渲染时序图"
- **evidence 要求**：渲染数据来自 timing_view 中的 evidence_refs
- **MVP 必须**：是

### VT-007 空视图处理

- **系统动作**：当 timing_view 无数据时处理
- **系统必须**：
  - 显示空状态提示"未识别到时序/流水信息"
  - 提示可能原因（如阶段为 Python 阶段无 RTL、代码中无时序逻辑）
  - 提供"重新收集证据"的入口
- **成功结果**：用户了解无时序数据的原因
- **失败表现**：无
- **evidence 要求**：不需要
- **MVP 必须**：是

### VT-008 证据不足处理

- **系统动作**：当部分时序节点或边 evidence 不足时处理
- **系统必须**：
  - evidence 不足的节点以降级样式展示
  - 标注为 `unknown` 的节点显示"?"标记
  - 用户悬停时显示"时序信息推断，证据不足"提示
- **成功结果**：用户了解哪些时序信息缺乏证据
- **失败表现**：无
- **evidence 要求**：unknown 节点仍应尽可能关联 source path
- **MVP 必须**：是

### VT-009 冲突证据处理

- **系统动作**：当存在冲突 evidence（如代码与文档描述的 latency 不一致）时处理
- **系统必须**：
  - 冲突节点/边以警示样式展示
  - 用户点击时显示冲突详情
  - 不自动裁决哪一方正确
- **成功结果**：用户了解存在冲突的证据
- **失败表现**：无
- **evidence 要求**：冲突详情应列出冲突的 evidence_ids
- **MVP 必须**：是

### VT-010 节点点击交互

- **用户动作**：用户点击时序图中的节点
- **系统必须**：
  - 高亮选中节点及其时序路径
  - 显示节点详情面板：名称、类型、描述、latency 信息、evidence 列表
  - 提供"查看证据"按钮
  - 提供"追问"按钮
- **成功结果**：用户可深入了解时序节点的 evidence 和上下文
- **失败表现**：无 evidence 时提示"该节点暂无关联证据"
- **evidence 要求**：节点详情必须展示 evidence_id、source_path、line_range
- **MVP 必须**：是

## 输入

| 输入项 | 来源 | 类型 |
|--------|------|------|
| implementation_understanding.json | story-generate-understanding 输出 | 结构化数据 |
| evidence_index.json | story-collect-evidence 输出 | 结构化数据 |

## 输出

| 输出项 | 类型 | 说明 |
|--------|------|------|
| 时序/流水图可视化 | UI 渲染 | 流水线 stage、信号流、状态转换的图形展示 |
| 节点详情面板 | UI 状态 | 选中节点的详细信息 |

## 异常 / 空状态

| 场景 | 处理 |
|------|------|
| timing_view 为空 | 显示"未识别到时序/流水信息"，提供重新分析入口 |
| 渲染失败 | 显示"无法渲染时序图"，提供重试按钮 |
| 所有节点均为 unknown | 显示"证据不足，无法确定时序信息" |
| 存在冲突节点 | 以警示样式展示，点击显示冲突详情 |

## 证据与追溯要求

- 每个节点必须携带 evidence_refs
- 每条边必须携带 evidence_refs
- 用户点击节点/边后，必须能查看到对应的 source_path 和 line_range

## 不确定性表达要求

- `unknown` 节点以降级样式展示
- `inferred` 边以虚线展示
- `conflicting` 节点/边以警示样式展示
- 所有不确定项必须在图中可见

## MVP 验收标准

- [ ] 时序图展示流水线 stage 和信号流动
- [ ] 节点类型可区分（PipelineStage/Register/FSMState/Handshake/ClockDomain）
- [ ] 边类型可区分（Sequential/Combinational/Control/Transition）
- [ ] 每个节点携带 evidence_refs，可点击查看详情
- [ ] 每条边携带 evidence_refs
- [ ] 空视图有恰当提示（区分 Python 阶段和 RTL 阶段）
- [ ] 证据不足的节点以降级样式展示
- [ ] 冲突节点以警示样式展示
- [ ] 不允许将图做成自由文本拼接或 JSON viewer

## 非目标

- 不追求 cycle-accurate 仿真波形
- 不做交互式拖拽编辑
- 不做 3D 可视化或动画
- 不把时序图做成静态 Markdown 报告

## 关联文档

- [`../mvp-functional-contract.md`](../mvp-functional-contract.md) — 跨 story 对象契约与验收场景
- [`story-generate-understanding.md`](story-generate-understanding.md) — 前置：生成结构化理解
- [`story-trace-evidence.md`](story-trace-evidence.md) — 相关：追溯证据
- [`story-ask-node-question.md`](story-ask-node-question.md) — 相关：围绕节点追问
- [`../mvp-requirements.md`](../mvp-requirements.md) — MVP 时序视图要求
