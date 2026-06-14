# 设计文档索引

---
status: active
updated: 2026-06-11
---

## 设计目录用途

本目录存放 `fpga-flow-mind` 的技术设计文档。设计文档描述"系统如何构建、数据如何流转、组件如何协作"，不描述产品需求或界面细节。

## 建议文档类型

| 文档类型 | 说明 | 示例 |
|----------|------|------|
| `semantic data contract` | 语义数据契约 | 结构化理解产物的 schema、字段定义、版本约定 |
| `agent workflow` | Agent 工作流设计 | 大模型/Agent 的调查计划、理解流程、grounding 检查 |
| `workspace scanning design` | Workspace 扫描设计 | 如何发现、识别和索引业务项目中的阶段和文件 |
| `evidence model` | 证据模型 | EvidenceItem 定义、证据抽取、索引、关联 |
| `visualization spec` | 可视化规范 | 图数据格式、渲染要求、交互协议 |
| `persistence design` | 持久化设计 | 产物存储格式、加载协议、版本迁移 |
| `provider boundary` | Provider 调用边界 | 大模型 API 调用、错误处理、配额控制、审计 |
| `safety boundary` | 安全边界 | 目标项目只读约束、文件访问白名单、禁止操作 |
| `Tauri/Rust/React architecture` | 技术架构 | 前后端职责划分、command 边界、状态管理 |

## 设计原则

### 1. 大模型/Agent 是主语义理解引擎

- 系统的核心理解能力来自大模型/Agent，不是静态分析
- 静态分析负责提供证据、索引和约束，不独立做出语义结论

### 2. 静态分析是辅助证据基础设施

- 静态分析负责：文件/符号/调用提取、RTL module/port/signal 提取、测试断言提取、行号和切片
- 静态分析不独立回答：主算法路径是什么、哪些逻辑是主路径、Python 概念如何映射到 RTL

### 3. 图来自结构化理解对象，不是自由文本拼接

- 可视化产物应基于结构化的 `ImplementationUnderstanding` 对象生成
- 不应将大模型返回的自由文本直接拼接成图

### 4. 用户可见主要结论必须绑定 evidence id、源码路径、行号范围

- 每个 claim 必须关联到具体的 evidence
- 用户应能追溯到源码的精确位置

### 5. 目标项目只读

- 系统设计必须确保对业务项目的访问是只读的
- 所有写入操作应发生在 app-owned 目录或临时目录

## 明确约束

- **Python 不能作为产品核心实现** — 后端必须用 Rust，前端用 React/TypeScript
- Python 可用于原型验证、工具脚本、辅助分析，但不应成为产品主路线

## 当前设计文档层级关系

Phase 1 技术设计由以下文档组成，按阅读顺序排列：

| 文档 | 定位 | 关系 |
|------|------|------|
| [`workspace-scanning-and-stage-detection.md`](workspace-scanning-and-stage-detection.md) | Phase 1 **入口与边界说明** | 阅读起点，概述做什么、不做什么 |
| [`phase-1-architecture.md`](phase-1-architecture.md) | Phase 1 **概要设计** | 模块划分、数据流、职责边界、安全设计 |
| [`phase-1-data-and-api-contract.md`](phase-1-data-and-api-contract.md) | Phase 1 **数据/API 契约** | Rust struct、TypeScript interface、Tauri command、错误格式 |
| [`phase-1-scanner-detail-design.md`](phase-1-scanner-detail-design.md) | Phase 1 **详细设计** | 扫描算法、阶段识别算法、validity 判定、边界条件 |

## 当前文档列表

| 文档 | 状态 | 说明 | 推荐阅读时机 |
|------|------|------|-------------|
| [`workspace-scanning-and-stage-detection.md`](workspace-scanning-and-stage-detection.md) | `active` | Phase 1 技术设计入口与边界说明：Workspace 扫描范围、阶段识别边界、输出对象概述 | Phase 1 实施前必读 |
| [`phase-1-architecture.md`](phase-1-architecture.md) | `active` | Phase 1 概要设计：模块划分、数据流、前后端职责边界、安全设计、目录结构建议 | 了解整体架构时阅读 |
| [`phase-1-data-and-api-contract.md`](phase-1-data-and-api-contract.md) | `active` | Phase 1 数据结构与 API 契约：Rust/TypeScript 类型、Tauri command 签名、错误/warning 格式 | 编码前后端接口前必读 |
| [`phase-1-scanner-detail-design.md`](phase-1-scanner-detail-design.md) | `active` | Phase 1 扫描与阶段识别详细设计：功能点映射、算法流程、边界条件、测试映射 | 编码扫描模块前必读 |
| [`phase-2-evidence-model.md`](phase-2-evidence-model.md) | `active` | Phase 2 evidence model 数据结构设计：EvidenceItem / EvidenceCollection / EvidenceStrength、ID 生成规则、line_range 规则、summary 规则、strength 语义、错误结构 | Phase 2 编码前必读 |
| [`phase-2-evidence-collector-design.md`](phase-2-evidence-collector-design.md) | `active` | Phase 2 evidence collector 后端设计：模块布局、collect_evidence command、文件读取策略、代码分块策略、提取器 trait、错误处理、单元测试设计 | Phase 2 后端编码前必读 |
| [`phase-3-understanding-model.md`](phase-3-understanding-model.md) | `active` | Phase 3 ImplementationUnderstanding 数据结构设计：Rust/TypeScript 字段定义、StageSummary、ImplementationClaim、ClaimConfidence（5 值含 supported）、ClaimCategory、EvidenceRef、UnknownItem、EvidenceGap、摘要对象、GenerationMeta、UnderstandingStats | Phase 3 编码前必读 |
| [`phase-3-understanding-generator-design.md`](phase-3-understanding-generator-design.md) | `active` | Phase 3 理解生成器后端设计：ContextBuilder、Provider trait、SchemaValidator、hallucination guard、Generator 主流程、generate_understanding command、degraded mode | Phase 3 后端编码前必读 |
| [`phase-4-view-model.md`](phase-4-view-model.md) | `active` | Phase 4 视图数据模型：ViewGraph/ViewNode/ViewEdge/ViewTraceRef/ViewLayoutHint/ViewMeta + NodeType/EdgeType/ViewType 枚举 + Rust/TypeScript 完整定义 | Phase 4 编码前必读 |
| [`phase-4-view-generator-design.md`](phase-4-view-generator-design.md) | `active` | Phase 4 视图生成器后端设计：ViewGraphGenerator + structure/dataflow/timing builder 转换规则 + generate_views command + 错误处理 | Phase 4 后端编码前必读 |
| [`phase-5-trace-model.md`](phase-5-trace-model.md) | `active` | Phase 5 证据回链与 Grounded Q&A 数据模型：SelectedTraceTarget、TraceRefResolved、SourceExcerpt、GroundedQuestion/Answer 等 Rust/TypeScript 定义 | Phase 5 编码前必读 |
| [`phase-5-trace-and-qa-design.md`](phase-5-trace-and-qa-design.md) | `active` | Phase 5 证据回链与 Grounded Q&A 后端设计：TraceResolver、SourceExcerptResolver、Provider trait、Tauri commands、安全边界 | Phase 5 后端编码前必读 |

> Phase 1 编码依据文档已收口。Phase 1 的 4 份设计文档与 `mvp-functional-contract.md` 共同构成 Phase 1 编码的权威依据。
> Phase 2 编码依据文档已收口（status=active）：`phase-2-evidence-model.md` + `phase-2-evidence-collector-design.md` + `phase-2-evidence-requirements.md`。
> Phase 3 编码依据文档已收口（status=active）：`phase-3-understanding-model.md` + `phase-3-understanding-generator-design.md`。
> Phase 4 设计文档已收口（status=active）：`phase-4-view-model.md` + `phase-4-view-generator-design.md`。
> **Phase 5 设计文档已收口（status=active）：`phase-5-trace-model.md` + `phase-5-trace-and-qa-design.md`，允许进入 Phase 5 Batch A 编码（范围限定 P5-T01~P5-T03）。**

> 注：当前设计应优先依据 active 的 `../requirements/` 需求文档和 `../requirements/mvp-functional-contract.md`；`../initial-requirements-draft/` 仅作为历史草案参考，当与正式文档冲突时以 active 文档为准。
