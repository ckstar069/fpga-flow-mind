# fpga-flow-mind MVP 技术路线

## 1. 技术栈

主产品技术栈固定为：

```text
Tauri v2
Rust backend
React + TypeScript frontend
```

目标平台优先：

- macOS
- Linux

## 2. 架构原则

本项目不是 static-only 工具，必须服务于“Agent-first 理解流程”。

核心链路：

```text
用户问题 / 选中阶段
  -> Agent 制定调查计划
  -> Rust 工具读取 workspace / 源码 / RTL / 测试 / 文档 / 配置
  -> 静态分析提取 evidence
  -> 大模型/Agent 基于 evidence 理解工程语义
  -> Grounding 检查
  -> 写入结构化语义产物
  -> 前端展示图、表、解释和证据
  -> 用户继续追问
```

## 3. 职责边界

### Tauri

- 桌面应用壳；
- 本地文件访问入口；
- 前后端 command 边界；
- 打包与系统集成。

### Rust backend

- workspace 扫描；
- 阶段识别；
- Python/Verilog/docs/tests/config 证据切片；
- 统一 evidence model；
- 本地只读安全边界；
- 语义产物持久化；
- provider 调用边界控制。

### React/TypeScript frontend

- 项目打开；
- 阶段导航；
- 图视图；
- 证据视图；
- 节点解释面板；
- Agent 问答视图；
- 对比和追问交互。

### 大模型/Agent

- 理解用户问题；
- 制定调查计划；
- 基于证据归纳工程语义；
- 生成图意图和解释；
- 区分 confirmed / inferred / unknown。

## 4. 统一证据模型

MVP 先建立语言无关的 EvidenceItem。

建议最小字段：

```text
EvidenceItem
  evidence_id
  source_path
  language
  source_kind
  line_range
  symbol
  summary
  extracted_terms[]
  strength
```

其中 `source_kind` 可包括：

- python_stage
- rtl
- test
- doc
- config
- external_module

## 5. 语义模型

MVP 的主要结果不是 Markdown，而是结构化理解对象。

建议最小对象：

```text
ImplementationUnderstanding
  stage_id
  summary
  structure_view
  dataflow_view
  timing_view
  concepts[]
  formulas[]
  signals[]
  evidence_refs[]
  uncertainties[]
```

主要语义结论都要满足：

```text
claim
  -> evidence_ids 或 uncertainty_ids
  -> confidence
```

`confidence` 最小集合：

- confirmed
- supported
- inferred
- unknown
- conflicting

## 6. 首发视图

MVP 首发不追求很多图，而是优先把最有用的三类做好：

### 结构视图

回答：

- 输入接口是什么？
- 主模块有哪些？
- pipeline/stage 如何分布？
- 输出接口是什么？

### 数据流视图

回答：

- 数据从哪里来？
- 中间经历哪些变换？
- 每个 stage 的核心运算是什么？
- 结果如何流向输出？

### 时序/流水视图

回答：

- stage latency 是多少？
- register/valid/ready 如何流动？
- pipeline overlap 如何发生？
- 是否存在状态机切换？

## 7. MVP 闭环

第一个可运行闭环建议如下：

```text
打开业务项目
  -> 识别阶段
  -> 选择一个阶段
  -> 收集相关源码/RTL/tests/docs/config 证据
  -> 生成调查上下文
  -> 大模型生成结构化理解结果
  -> 进行 grounding 检查
  -> 持久化理解产物
  -> UI 展示三类视图
  -> 用户点击节点查看证据并继续追问
```

## 8. 持久化产物

MVP 需要把“临场理解”变成可持久化对象。

建议最小产物集合：

- `workspace_profile.json`
- `evidence_index.json`
- `implementation_understanding.json`
- `visualization_spec.json`
- `trace_index.json`

这些文件是系统内产物，不要求直接暴露给用户作为最终成果，但必须可再次加载。

## 9. 安全边界

必须坚持：

- 目标业务项目只读；
- 不修改 `fpga_project_*`；
- 不运行 Vivado；
- 不运行 synthesis / implementation / bitstream；
- 不默认执行目标项目脚本；
- 默认输出写入 app-owned 或临时目录；
- 外部模型调用必须可控、显式、可审计。

## 10. 后续扩展方向

MVP 之后再考虑：

- 跨阶段对比；
- Python 到 RTL 映射图；
- 测试覆盖图；
- 多阶段语义记忆；
- 外部开源可视化工具接入；
- 与 `agent-scope` 的上下文联动。

但这些都应建立在“单阶段理解闭环稳定”之后。
