# 需求文档索引

---
status: active
updated: 2026-06-11
---

## 需求目录用途

本目录存放 `fpga-flow-mind` 的所有产品需求文档。需求文档描述"产品要解决什么问题、为谁解决、解决到什么程度"，不描述具体技术实现。

需求文档服务于以下核心能力：

- workspace understanding — 理解业务项目整体结构
- source evidence — 源码证据的收集与展示
- stage understanding — 阶段实现的理解与可视化
- dataflow understanding — 数据流向与变换的理解
- timing understanding — 时序/流水关系的理解
- semantic claims — 语义结论的生成与表达
- evidence traceability — 证据可追溯性
- uncertainty expression — 不确定性的显式表达
- grounded Q&A — 基于证据的问答
- local desktop usability — 本地桌面端可用性

**明确不要写成**：审计工具需求、PASS-HOLD 判定需求、通用静态分析器需求。

## 建议文档类型

| 文档类型 | 说明 | 示例 |
|----------|------|------|
| `product scope` | 产品范围与边界 | 本项目做什么、不做什么 |
| `user stories` | 用户故事 | 具体使用场景与用户目标 |
| `MVP requirements` | MVP 功能需求 | 首个闭环必须支持的能力 |
| `non-goals` | 非目标与排除项 | 当前明确不做的方向 |
| `acceptance criteria` | 验收标准 | 每个需求如何判定已完成 |

## `stories/` 子目录

`stories/` 用于存放按用户目标拆分的 story 文档。每个 story 聚焦一个具体的、可验证的用户价值。

详见 [`stories/README.md`](stories/README.md)。

## 需求文档最小模板

每个需求文档建议包含以下章节：

```markdown
# 文档标题

---
status: draft | active | superseded | archived
updated: YYYY-MM-DD
---

## 1. 用户目标

这个需求为谁解决什么问题？

## 2. 业务背景

为什么现在需要这个功能？与现有流程的关系？

## 3. 主流程

正常情况下的使用步骤。

## 4. 异常 / 空状态

边界情况、无数据情况、错误情况如何处理。

## 5. 证据与追溯要求

该需求产生的结论如何绑定 evidence id、源码路径、行号范围。

## 6. MVP 验收标准

最小可用版本下，如何判定此需求已满足。

## 7. 非目标

此需求明确不包含的内容。

## 8. 关联设计文档

对应 UI/UX 文档、技术设计文档的链接。
```

## 当前文档列表

| 文档 | 状态 | 说明 | 推荐阅读时机 |
|------|------|------|-------------|
| [`product-scope.md`](product-scope.md) | `active` | 产品范围、目标用户、核心痛点、非目标、成功标准、安全边界 | 任何需求讨论、规划或审核前必读，确认产品边界 |
| [`mvp-requirements.md`](mvp-requirements.md) | `active` | MVP 主流程、必须能力、暂不做能力、视图边界、evidence 追溯、置信度要求、验收标准 | 进入 Phase 1+ 实施前必读，确认 MVP 范围 |
| [`mvp-functional-contract.md`](mvp-functional-contract.md) | `active` | 跨 story 统一对象、字段约束、枚举值、依赖关系、端到端验收场景 | **进入设计和实施前必读**，对象契约与验收标准来源 |
| [`phase-2-evidence-requirements.md`](phase-2-evidence-requirements.md) | `active` | Phase 2 证据索引与 evidence model 需求：功能点 EV-001~EV-008、前后端边界、Phase 1/3 接口 | Phase 2 设计和实施前必读 |
| [`phase-3-understanding-requirements.md`](phase-3-understanding-requirements.md) | `draft` | Phase 3 单阶段结构化理解需求：功能点 IU-001~IU-008、ImplementationUnderstanding、claim/evidence binding、confidence、unknown/gap | Phase 3 设计和实施前必读 |

> **需求文档职责分层**：
>
> | 文档 | 职责 | 冲突裁决 |
> |------|------|----------|
> | `product-scope.md` | 决定产品边界、非目标、成功标准 | 产品边界冲突以此为准 |
> | `mvp-requirements.md` | 决定 MVP 范围、必须/暂不做能力 | MVP 范围冲突以此为准 |
> | `mvp-functional-contract.md` | 决定跨 story 对象、字段、枚举、错误码、依赖、端到端验收 | 对象字段、枚举、跨 story 数据流冲突以此为准 |
> | `stories/*.md` | 决定单个用户目标、局部功能点、局部异常处理 | 单 story 内交互细节以此为准，但不得违反上层范围和契约 |
>
> **规则**：
> - 后续技术设计不得重新定义需求对象，应从 `mvp-functional-contract.md` 派生。
> - story 文档中的功能点不得扩大 `mvp-requirements.md` 定义的 MVP 范围。
> - story 文档中的对象字段和枚举必须与 `mvp-functional-contract.md` 保持一致。
>
> ---
>
> **范围文档与功能点文档的关系**（历史说明，仍有效）：
> - `product-scope.md` 和 `mvp-requirements.md` 描述产品范围和 MVP 能力边界，是方向性文档。
> - **具体可实施的功能点和可验收标准**以 `stories/` 下的 story 文档为准。
> - 实施前应阅读对应阶段的 story 文档，范围讨论应以 `product-scope.md` 为准。
>
> 各 story 按 MVP 主流程顺序排列：
> ```text
> story-open-workspace → story-select-stage → story-collect-evidence
>   → story-generate-understanding
>   → story-view-structure / story-view-dataflow / story-view-timing
>   → story-trace-evidence → story-ask-node-question
>   → story-persist-and-reopen
> ```
