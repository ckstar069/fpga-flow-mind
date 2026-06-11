# Story 文档索引

---
status: active
updated: 2026-06-11
---

## Story 拆分原则

Story 按**用户目标**拆分，而不是按技术模块拆分。每个 story 应满足：

- 有明确的用户角色和使用场景
- 描述"用户想做什么"，而非"系统要实现什么功能"
- 可在一个 MVP 迭代内完成或验证
- 有明确的验收标准
- 不脱离 evidence、stage、dataflow、timing、traceability 和 Q&A 等核心能力

## 建议 Story 文件命名

| 文件名 | 对应用户目标 |
|--------|-------------|
| `story-open-workspace.md` | 作为用户，我想打开一个业务项目，以便开始理解它 |
| `story-select-stage.md` | 作为用户，我想选择并聚焦于一个阶段，以便理解该阶段的实现 |
| `story-view-structure.md` | 作为用户，我想看到该阶段的结构图，以便理解模块组成和接口关系 |
| `story-view-dataflow.md` | 作为用户，我想看到数据流图，以便理解数据从哪里来、经过什么变换、流向哪里 |
| `story-view-timing.md` | 作为用户，我想看到时序/流水图，以便理解 latency、握手信号和流水线行为 |
| `story-trace-evidence.md` | 作为用户，我想点击图中节点查看对应的源码证据，以便验证结论 |
| `story-ask-node-question.md` | 作为用户，我想围绕某个节点继续追问，以便深入理解具体实现细节 |

## Story 文档模板

```markdown
# Story: <标题>

---
status: draft | active | superseded | archived
updated: YYYY-MM-DD
---

## 用户角色

## 用户目标

作为 <角色>，我希望 <目标>，以便 <价值>。

## 业务背景

## 主流程

1. 步骤一
2. 步骤二
3. 步骤三

## 异常与边界

## 验收标准

- [ ] 标准一
- [ ] 标准二

## 关联需求

## 关联设计文档

## 备注
```

## Story 与 MVP 闭环的关系

MVP 的成功标准是"用户能否更快读懂项目"。所有 story 应共同构成以下闭环：

```text
打开项目 → 选择阶段 → 查看结构图 → 查看数据流图 → 查看时序图
  → 点击节点追溯证据 → 围绕节点追问 → 获得基于证据的回答
```

单个 story 可以不覆盖完整闭环，但不应与闭环方向偏离。

## Story 与 MVP 功能契约的关系

- Story 文档定义**单个用户目标**和**具体功能点**。
- **跨 story 的统一对象、字段、枚举值、依赖关系和端到端验收场景**以 [`../mvp-functional-contract.md`](../mvp-functional-contract.md) 为准。
- 当 story 文档中的输入/输出对象名称、字段或枚举与契约文档冲突时，以契约文档为准。
- 技术设计应从契约文档派生，不得重新定义需求对象。

## Story 核心约束

每个 story 不应脱离以下核心关注点：

- **evidence** — 结论必须有源码证据支撑
- **stage** — 理解应围绕具体阶段展开
- **dataflow** — 数据流向和变换必须可理解
- **timing** — 时序/流水关系必须可理解
- **traceability** — 所有结论可追溯至源码
- **Q&A** — 支持用户继续追问，而非一次性报告

## 当前文档列表

| 文档 | 状态 | 说明 | 推荐阅读时机 |
|------|------|------|-------------|
| [`story-open-workspace.md`](story-open-workspace.md) | `draft` | 打开业务项目、扫描目录、识别阶段、生成 workspace profile | Phase 1 实施前必读 |
| [`story-select-stage.md`](story-select-stage.md) | `draft` | 阶段列表展示、单阶段选择、阶段上下文准备 | Phase 1 实施前必读 |
| [`story-collect-evidence.md`](story-collect-evidence.md) | `draft` | 从 Python/Verilog/docs/tests/config 提取 evidence item、建立索引 | Phase 2 实施前必读 |
| [`story-generate-understanding.md`](story-generate-understanding.md) | `draft` | 基于 evidence 生成结构化 ImplementationUnderstanding、grounding 检查 | Phase 3 实施前必读 |
| [`story-view-structure.md`](story-view-structure.md) | `draft` | 结构图：模块、接口、层级、节点/边 evidence 绑定 | Phase 4 实施前必读 |
| [`story-view-dataflow.md`](story-view-dataflow.md) | `draft` | 数据流图：数据来源、变换、流向、节点/边 evidence 绑定 | Phase 4 实施前必读 |
| [`story-view-timing.md`](story-view-timing.md) | `draft` | 时序/流水图：latency、握手信号、流水线、状态机 | Phase 4 实施前必读 |
| [`story-trace-evidence.md`](story-trace-evidence.md) | `draft` | 点击节点追溯 evidence、源码路径、行号范围、代码片段 | Phase 5 实施前必读 |
| [`story-ask-node-question.md`](story-ask-node-question.md) | `draft` | 围绕节点追问、基于 evidence 回答、问答历史 | Phase 5 实施前必读 |
| [`story-persist-and-reopen.md`](story-persist-and-reopen.md) | `draft` | 产物持久化、再次打开加载、源码变更检测 | Phase 6 实施前必读 |
