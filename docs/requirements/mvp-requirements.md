# MVP 需求

---
status: draft
updated: 2026-06-11
---

## MVP 目标

建立首个可运行的理解闭环：用户打开一个业务项目，选择一个阶段，系统通过大模型主导的语义理解与静态证据辅助，生成结构化的理解产物，并以三类核心视图呈现，支持证据追溯和追问。

## MVP 主流程

```text
打开业务项目
  -> 识别阶段目录结构
  -> 用户选择一个阶段
  -> 收集该阶段相关源码/RTL/tests/docs/config 证据
  -> 生成调查上下文
  -> 大模型/Agent 生成结构化理解结果
  -> 进行 grounding 检查
  -> 持久化理解产物
  -> UI 展示三类核心视图
  -> 用户点击节点查看证据
  -> 用户围绕节点继续追问
  -> 系统基于证据回答
```

## MVP 必须能力

### 1. Workspace 识别

- 能够打开由 `ai_project_template` 创建的业务项目目录
- 识别阶段目录结构（如 L0、L1、...、L6、RTL 等约定）
- 识别项目中的 Python、Verilog、文档、测试、配置文件
- 识别外部模块引用（如来自 `urban_wireless` 的导入）

### 2. 阶段聚焦

- 用户可选择单个阶段进行深入理解
- 系统收集该阶段的相关上下文（不限于该阶段目录，可能包含上游阶段的接口约定）

### 3. 证据收集与索引

- 从源码、RTL、测试、文档、配置中提取 evidence item
- 每个 evidence item 至少包含：来源路径、语言、行号范围、符号、摘要
- 建立 evidence 索引，支持按阶段、按文件类型、按符号检索

### 4. 结构化理解生成

- 生成 `ImplementationUnderstanding` 结构化对象
- 包含：阶段摘要、结构视图、数据流视图、时序视图、概念列表、公式列表、信号列表、证据引用、不确定项列表
- 所有语义结论必须绑定 evidence id，区分置信度

### 5. 三类核心视图

#### 结构视图

回答：
- 输入接口是什么？
- 主模块有哪些？
- pipeline/stage 如何分布？
- 输出接口是什么？

#### 数据流视图

回答：
- 数据从哪里来？
- 中间经历哪些变换？
- 每个 stage 的核心运算是什么？
- 结果如何流向输出？

#### 时序/流水视图

回答：
- stage latency 是多少？
- register/valid/ready 如何流动？
- pipeline overlap 如何发生？
- 是否存在状态机切换？

### 6. 证据回链

- 图中每个节点和边应可点击
- 点击后展示对应的源码 evidence
- evidence 展示应包含：来源文件路径、行号范围、代码片段、evidence id

### 7. Grounded Q&A

- 用户可围绕某个节点或结论继续提问
- 系统回答必须基于已收集的 evidence
- 回答应标注所依赖的 evidence id
- 对证据不足的问题，明确标注 `unknown`

### 8. 持久化与再次打开

- 理解产物可持久化存储
- 再次打开同一项目时，可加载已生成的理解产物
- 支持重新生成（当源码发生变化时）

## MVP 暂不做能力

- 跨阶段对比
- Python 到 RTL 的映射图
- 测试覆盖图
- 多阶段语义记忆
- 外部开源可视化工具接入
- 与 `agent-scope` 的上下文联动
- 自动检测源码变更并增量更新
- 多人协作或分享

## 三类核心视图的需求边界

| 视图 | 必须回答的问题 | 不追求的内容 |
|------|---------------|-------------|
| 结构视图 | 模块有哪些、接口是什么、层级如何分布 | 不追求 UML 类图级别的细节 |
| 数据流视图 | 数据来源、变换步骤、流向输出 | 不追求逐信号级别的完整数据通路 |
| 时序/流水视图 | latency、握手信号、流水线行为 | 不追求 cycle-accurate 仿真波形 |

## Evidence 追溯要求

用户可见的每个主要结论必须满足：

- 绑定**唯一 evidence id**
- 可追溯到**源码文件路径**
- 可追溯到**行号范围**（起始行 - 结束行）
- evidence 展示应包含代码片段和上下文

## 置信度用户可见要求

所有语义结论必须标注置信度，用户应能一眼区分：

| 置信度 | 含义 | 展示要求 |
|--------|------|----------|
| `confirmed` | 有强源码证据直接支撑 | 明确标注，可作为高可信度结论 |
| `supported` | 有证据支撑，但需辅助推断 | 明确标注，说明支撑证据 |
| `inferred` | 基于间接证据或上下文推断 | 明确标注，说明推断依据 |
| `unknown` | 证据不足，无法确定 | 明确标注，不强行解释 |
| `conflicting` | 存在矛盾的证据 | 明确标注，列出冲突点，不自动裁决 |

系统不应隐藏 `unknown` 或 `inferred`，也不应将 `inferred` 呈现为 `confirmed`。

## Grounded Q&A 的需求边界

- **输入**：用户围绕某个节点、信号、公式或结论的自然语言提问
- **输出**：基于已收集 evidence 的回答，附带所引用的 evidence id
- **约束**：
  - 不回答与当前阶段无关的问题
  - 不基于未收集的 evidence 进行猜测
  - 对证据不足的问题回答 `unknown`，不编造
  - 不替用户做正确性判断

## 持久化与再次打开的最小要求

MVP 持久化至少包含以下产物：

| 产物 | 说明 | 必须？ |
|------|------|--------|
| `workspace_profile.json` | 项目结构识别结果 | 是 |
| `evidence_index.json` | 证据索引 | 是 |
| `implementation_understanding.json` | 结构化理解产物 | 是 |
| `trace_index.json` | 追溯索引 | 否（可由 understanding 派生） |
| `qa_history.json` | 问答历史 | 是（如有问答记录） |

以下产物为**可选**，是否持久化由后续设计决定：

| 产物 | 说明 |
|------|------|
| `visualization_spec.json` | 可视化渲染规格。可由前端根据 `implementation_understanding.json` 动态生成，MVP 不强制持久化。 |

这些文件是系统内产物，不要求直接暴露给用户作为最终成果，但必须可再次加载。

持久化存储位置应为 app-owned 目录或临时目录，**不写入目标项目目录**。

## MVP 验收标准

- [ ] 能打开一个真实业务项目并识别阶段
- [ ] 能选择一个阶段并收集相关证据
- [ ] 能生成结构化理解产物
- [ ] 能展示结构图、数据流图、时序/流水图
- [ ] 图中节点可点击并追溯到源码证据
- [ ] 不确定项被显式标注
- [ ] 用户可围绕节点追问并获得基于证据的回答
- [ ] 产物可持久化并再次加载
- [ ] 目标项目始终保持只读
- [ ] 不运行 Vivado / synthesis / implementation / bitstream

## 功能点拆解索引

MVP 主流程中的每个环节已拆解为独立的 story 文档，包含具体功能点、验收标准和异常处理。

| MVP 环节 | Story 文档 | 功能点编号前缀 |
|----------|-----------|---------------|
| 打开业务项目 | [`stories/story-open-workspace.md`](stories/story-open-workspace.md) | WS-xxx |
| 选择阶段 | [`stories/story-select-stage.md`](stories/story-select-stage.md) | ST-xxx |
| 收集证据 | [`stories/story-collect-evidence.md`](stories/story-collect-evidence.md) | EV-xxx |
| 生成结构化理解 | [`stories/story-generate-understanding.md`](stories/story-generate-understanding.md) | IU-xxx |
| 展示结构图 | [`stories/story-view-structure.md`](stories/story-view-structure.md) | VS-xxx |
| 展示数据流图 | [`stories/story-view-dataflow.md`](stories/story-view-dataflow.md) | VD-xxx |
| 展示时序/流水图 | [`stories/story-view-timing.md`](stories/story-view-timing.md) | VT-xxx |
| 追溯证据 | [`stories/story-trace-evidence.md`](stories/story-trace-evidence.md) | TR-xxx |
| 继续追问 | [`stories/story-ask-node-question.md`](stories/story-ask-node-question.md) | QA-xxx |
| 持久化与再次打开 | [`stories/story-persist-and-reopen.md`](stories/story-persist-and-reopen.md) | PS-xxx |

> 实施时以 story 文档中的功能点清单和 MVP 验收标准为准。
>
> **跨 story 输入/输出对象和验收契约**以 [`mvp-functional-contract.md`](mvp-functional-contract.md) 为准。
> MVP 范围以本文档为准，对象字段和枚举定义以契约文档为准。
