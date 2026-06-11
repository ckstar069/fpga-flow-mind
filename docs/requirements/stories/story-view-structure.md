# Story: 查看结构图

---
status: draft
updated: 2026-06-11
---

## 用户目标

作为 FPGA 开发人员，我希望看到选中阶段的结构图，以便理解模块组成、接口关系和层级分布。

## 业务背景

结构图回答"这个阶段的代码组织成哪些模块、它们之间是什么关系、输入输出接口是什么"。这是理解阶段实现的第一步，帮助用户建立整体认知框架。

## 前置条件

- ImplementationUnderstanding 已生成
- structure_view 部分已填充

## 触发入口

用户在理解结果概览面板中点击"查看结构图"按钮，或在阶段分析完成后默认展示结构图。

## 主流程

1. 用户触发结构图展示
2. 系统从 ImplementationUnderstanding 中提取 structure_view 数据
3. 系统生成可视化规范
4. 系统渲染结构图
5. 用户与结构图交互（点击查看节点详情）

## 功能点清单

### VS-001 触发结构图展示

- **用户动作**：用户点击"查看结构图"按钮
- **系统必须**：
  - 切换到结构图视图面板
  - 显示加载状态（如有必要）
  - 如 structure_view 未生成，提示"结构信息未生成"
- **成功结果**：结构图视图面板展示
- **失败表现**：无数据时显示空状态提示
- **evidence 要求**：不需要
- **MVP 必须**：是

### VS-002 节点类型定义

- **系统动作**：定义结构图中支持的节点类型
- **系统必须**：结构图包含以下节点类型：
  - **输入接口节点**（Input）：阶段的输入信号/参数/数据入口
  - **模块节点**（Module）：主要功能模块（Python 函数/类、Verilog module）
  - **子模块节点**（Submodule）：模块内部的子组件
  - **输出接口节点**（Output）：阶段的输出信号/结果/数据出口
  - **辅助节点**（Auxiliary）：辅助功能（配置、常量、工具函数）
- **成功结果**：节点类型清晰可区分
- **失败表现**：无
- **evidence 要求**：每个节点必须携带 evidence_refs
- **MVP 必须**：是

### VS-003 边类型定义

- **系统动作**：定义结构图中支持的边类型
- **系统必须**：结构图包含以下边类型：
  - **包含边**（Contains）：父模块包含子模块
  - **调用边**（Calls）：模块 A 调用/实例化模块 B
  - **数据边**（Data）：信号/数据从上游流向下游
  - **依赖边**（Depends）：模块依赖外部模块/库
- **成功结果**：边类型清晰可区分
- **失败表现**：无
- **evidence 要求**：每条边必须携带 evidence_refs
- **MVP 必须**：是

### VS-004 节点携带 evidence_refs

- **系统动作**：确保每个节点都有 evidence 关联
- **系统必须**：
  - 每个节点显示时携带 `evidence_refs` 数组
  - 用户点击节点时可查看关联的 evidence 列表
  - evidence_refs 中的每个 id 对应 evidence_index 中的真实 item
- **成功结果**：节点与 evidence 的关联可查询
- **失败表现**：无 evidence 关联的节点标注为 `unknown`
- **evidence 要求**：每个节点至少有一个 evidence_id 或标注为 `unknown`
- **MVP 必须**：是

### VS-005 边携带 evidence_refs

- **系统动作**：确保每条边都有 evidence 关联
- **系统必须**：
  - 每条边显示时携带 `evidence_refs` 数组
  - 用户点击边时可查看关联的 evidence 列表
  - 边的关系（如"调用"）必须有源码证据支撑
- **成功结果**：边与 evidence 的关联可查询
- **失败表现**：无 evidence 关联的边标注为 `inferred`
- **evidence 要求**：每条边至少有一个 evidence_id 或标注为 `inferred`
- **MVP 必须**：是

### VS-006 结构图渲染

- **系统动作**：将 structure_view 数据渲染为可视化图
- **系统必须**：
  - 展示输入 → 模块 → 输出的整体流向
  - 模块按层级排列（父模块在上，子模块在下）
  - 输入接口在左侧，输出接口在右侧
  - 不同节点类型使用不同的视觉样式（颜色、形状、图标）
  - 不同边类型使用不同的线型（实线、虚线、箭头方向）
  - 节点标签显示名称（module 名/函数名/信号名）
  - 节点旁标注置信度指示器（confirmed/supported/inferred/unknown/conflicting）
- **成功结果**：用户可直观理解阶段结构
- **失败表现**：渲染失败时显示"无法渲染结构图"
- **evidence 要求**：渲染数据来自 structure_view 中的 evidence_refs
- **MVP 必须**：是

### VS-007 空视图处理

- **系统动作**：当 structure_view 无数据时处理
- **系统必须**：
  - 显示空状态提示"未识别到模块结构"
  - 提示可能原因（如阶段为空、代码过于简单、解析失败）
  - 提供"重新收集证据"的入口
- **成功结果**：用户了解无结构数据的原因
- **失败表现**：无
- **evidence 要求**：不需要
- **MVP 必须**：是

### VS-008 证据不足处理

- **系统动作**：当部分节点或边 evidence 不足时处理
- **系统必须**：
  - evidence 不足的节点以降级样式展示（如灰色、虚线边框）
  - 标注为 `unknown` 的节点在标签旁显示"?"标记
  - 用户悬停时显示"证据不足"提示
- **成功结果**：用户了解哪些部分缺乏证据
- **失败表现**：无
- **evidence 要求**：unknown 节点仍应尽可能关联 source path
- **MVP 必须**：是

### VS-009 冲突证据处理

- **系统动作**：当存在冲突 evidence 时处理
- **系统必须**：
  - 冲突节点/边以警示样式展示（如红色边框、感叹号标记）
  - 用户点击时显示冲突详情（矛盾的 evidence 列表）
  - 不自动裁决哪一方正确
- **成功结果**：用户了解存在冲突的证据
- **失败表现**：无
- **evidence 要求**：冲突详情应列出冲突的 evidence_ids 和 source paths
- **MVP 必须**：是

### VS-010 节点点击交互

- **用户动作**：用户点击结构图中的节点
- **系统必须**：
  - 高亮选中节点
  - 显示节点详情面板：名称、类型、描述、evidence 列表
  - 提供"查看证据"按钮，跳转到 evidence 追溯
  - 提供"追问"按钮，围绕该节点提问
- **成功结果**：用户可深入了解节点的 evidence 和上下文
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
| 结构图可视化 | UI 渲染 | 节点、边、层级、流向的图形展示 |
| 节点详情面板 | UI 状态 | 选中节点的详细信息 |

## 异常 / 空状态

| 场景 | 处理 |
|------|------|
| structure_view 为空 | 显示"未识别到模块结构"，提供重新分析入口 |
| 渲染失败 | 显示"无法渲染结构图"，提供重试按钮 |
| 所有节点均为 unknown | 显示"证据不足，无法确定模块结构" |
| 存在冲突节点 | 以警示样式展示，点击显示冲突详情 |

## 证据与追溯要求

- 每个节点必须携带 evidence_refs（evidence_id 列表）
- 每条边必须携带 evidence_refs
- 用户点击节点/边后，必须能查看到对应的 source_path 和 line_range
- evidence_refs 必须是 evidence_index 中真实存在的 id

## 不确定性表达要求

- `unknown` 节点以降级样式展示（灰色、虚线边框、"?"标记）
- `inferred` 边以虚线展示
- `conflicting` 节点/边以警示样式展示（红色边框、感叹号）
- 所有不确定项必须在图中可见，不隐藏

## MVP 验收标准

- [ ] 结构图展示输入 → 模块 → 输出的整体流向
- [ ] 节点类型可区分（输入/模块/子模块/输出/辅助）
- [ ] 边类型可区分（包含/调用/数据/依赖）
- [ ] 每个节点携带 evidence_refs，可点击查看详情
- [ ] 每条边携带 evidence_refs
- [ ] 空视图有恰当提示
- [ ] 证据不足的节点以降级样式展示
- [ ] 冲突节点以警示样式展示
- [ ] 不允许将图做成自由文本拼接或 JSON viewer

## 非目标

- 不追求 UML 类图级别的细节
- 不做交互式拖拽编辑（只读展示）
- 不做 3D 可视化
- 不做动画效果
- 不把结构图做成静态 Markdown 报告

## 关联文档

- [`../mvp-functional-contract.md`](../mvp-functional-contract.md) — 跨 story 对象契约与验收场景
- [`story-generate-understanding.md`](story-generate-understanding.md) — 前置：生成结构化理解
- [`story-trace-evidence.md`](story-trace-evidence.md) — 相关：追溯证据
- [`story-ask-node-question.md`](story-ask-node-question.md) — 相关：围绕节点追问
- [`../mvp-requirements.md`](../mvp-requirements.md) — MVP 结构视图要求
