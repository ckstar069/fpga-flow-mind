# Post-MVP 总体路线图（Phase 7 ~ Phase 11）

---
status: draft
updated: 2026-06-15
---

> 本文档是 `fpga-flow-mind` 在 MVP（Phase 0–6 / tag `v0.1.0-mvp`）之后的总体路线图。
> 它只描述"整体方向、阶段关系、依赖顺序、进入纪律"，不承载任何阶段的详细需求、设计或编码计划。
> 所有 Phase 7 ~ Phase 11 的 overview 文档当前状态为 `draft`，**均未进入编码**。

## 1. 背景与问题

### 1.1 当前位置：技术闭环 MVP，不等于产品可用

`fpga-flow-mind` 已完成 Phase 0–6，发布了 tag `v0.1.0-mvp`（2026-06-15）。这一里程碑验证了**单阶段理解闭环在技术上成立**：

- 能打开项目、识别阶段、收集证据、生成 `ImplementationUnderstanding`；
- 能产出结构图 / 数据流图 / 时序流水图，节点与边可追溯到证据；
- 能做基于证据的 Grounded Q&A（当前为 MockProvider）；
- 能持久化 session 并再次打开，目标项目只读。

但这是**技术闭环 MVP**，不等于产品可用性完成。它存在几个本质缺口：

| 维度 | MVP 现状 | 缺口 |
|------|----------|------|
| 分析对象 | 临时小型样例项目（`/tmp/...` 手工构造） | 未在**真实** `ai_project_template` 业务项目上验证分析质量 |
| 语义引擎 | MockProvider（关键词匹配） | 未接入**真实 LLM**，grounding 在真实语义压力下未验证 |
| 理解范围 | 单阶段孤立理解 | 无法跨阶段、无法做 **Python → RTL** 语义映射 |
| 知识沉淀 | 单次 session 产物 | 无**多阶段语义记忆**，历史问答无法复用 |
| UI 形态 | 工程调试式界面 | 尚非真实可用的**理解工作台** |
| 联动 | 独立运行 | 未与 `agent-scope` / 测试覆盖理解 / 外部可视化工具联动 |

### 1.2 为什么必须先补总体文档

Phase 0–6 是按 Phase-by-Phase 临时推进的，每个阶段的详细文档在该阶段实施前才补齐。这种节奏在 MVP 阶段可行，但进入 Post-MVP 后风险上升：候选方向多（真实质量、产品 UI、真实 LLM、跨阶段、语义记忆、外部联动），若不先明确**阶段关系和依赖顺序**，极易出现顺序错乱（例如在真实项目质量未验证前就接真实 LLM，或在单阶段理解不稳定前就做跨阶段映射）。

因此本轮只补**总体路线图与各阶段 overview**，作为后续进入任一阶段详细文档的前提。

## 2. Post-MVP 总体目标

把"技术上成立的单阶段理解闭环"推进为"在真实 `ai_project_template` 业务项目上可靠、可用、可积累的理解工作台"，同时严格守住本项目始终不变的产品定位：

- 仍是**理解与可视化工具**，不是审计器 / PASS-HOLD 工具 / 正确性裁决器；
- 仍以**大模型为主分析者、静态分析为证据基础设施**；
- 仍是**本地桌面优先**，不是 server-first / cloud-first 架构；
- 仍保持**目标项目只读**，不运行 Vivado / synthesis / implementation / bitstream。

## 3. Phase 7 ~ Phase 11 阶段总览

> 下表的 **Phase** 指 `fpga-flow-mind` **本项目**的开发推进阶段；与业务项目的 `L0` / `L1` / `RTL` 实现阶段无关，二者不能混用。

| 阶段 | 主题 | 一句话目标 | 对应 overview 文档 |
|------|------|-----------|--------------------|
| Phase 7 | 真实项目评估与 evidence/understanding 质量补强 | 在真实 `ai_project_template` 项目上验证并提升分析能力，让理解产物可信 | [`phase-7-overview-real-project-quality.md`](phase-7-overview-real-project-quality.md) |
| Phase 8 | 产品级 UI/UX 工作台重构 | 把工程调试式界面重构为真实可用的理解工作台 | [`phase-8-overview-product-ui-workbench.md`](phase-8-overview-product-ui-workbench.md) |
| Phase 9 | 真实 LLM Provider 与 grounding 生产化 | 在显式配置、可关闭、可验证前提下接入真实 LLM，并守住 grounding 与 citation | [`phase-9-overview-real-llm-grounding.md`](phase-9-overview-real-llm-grounding.md) |
| Phase 10 | 跨阶段理解与 Python-to-RTL 映射 | 把 L0/L1/.../RTL 的实现关系组织起来，支持跨阶段对比与 Python 到 RTL 的语义映射 | [`phase-10-overview-cross-stage-python-rtl.md`](phase-10-overview-cross-stage-python-rtl.md) |
| Phase 11 | 多阶段语义记忆、测试覆盖图与 agent-scope 联动 | 把阶段理解沉淀为可复用语义记忆，并探索联动边界 | [`phase-11-overview-semantic-memory-and-integration.md`](phase-11-overview-semantic-memory-and-integration.md) |

这五个主题对应 `PROJECT_BRIEF` §10 与 `mvp-requirements.md` "MVP 暂不做能力"中列出的后续扩展方向（跨阶段对比、Python→RTL 映射、测试覆盖图、多阶段语义记忆、外部可视化工具接入、agent-scope 联动），加上 MVP 已暴露的两个真实化缺口（真实项目质量、真实 LLM grounding）和产品化缺口（UI 工作台）。

## 4. 阶段关系与依赖顺序

```text
Phase 7（真实项目质量）
   │   分析能力的可信度基线。没有它，后续阶段的产出都建立在未验证的理解之上。
   │
   ├──> Phase 8（产品 UI 工作台）        可与 Phase 7 部分并行；但工作台的信息架构依赖
   │                                       Phase 7 暴露的真实理解形态来定型。
   │
   └──> Phase 9（真实 LLM grounding）     依赖 Phase 7 的 evidence/understanding 质量基线：
                                           grounding 要在真实、有噪声的证据上验证。
                  │
                  └──> Phase 10（跨阶段 + Python→RTL）
                          跨语言语义映射在实质上需要真实 LLM，依赖 Phase 9；
                          也依赖 Phase 7 对真实多阶段项目的理解能力。
                                 │
                                 └──> Phase 11（语义记忆 + agent-scope 联动）
                                         语义记忆需要跨阶段理解产物作为沉淀来源，
                                         依赖 Phase 10。
```

推荐串行主干与允许的并行：

- **主干顺序（强约束）**：Phase 7 → Phase 9 → Phase 10 → Phase 11。
  - Phase 7 是质量基线，必须先做；
  - Phase 10 的 Python→RTL 语义映射在实质上依赖真实 LLM（Phase 9）；
  - Phase 11 的多阶段语义记忆需要跨阶段产物（Phase 10）作为来源。
- **可并行**：Phase 8（UI 工作台）可与 Phase 7 部分并行；它的信息架构宜在 Phase 7 暴露真实理解形态后定型，因此更稳妥是 Phase 7 完成或接近完成时启动。
- **非强制锁定顺序**：本路线图给出的是依赖关系，不是日历排期。任一阶段可在其前置阶段完成且详细文档 active 后启动；也可视资源调整并行度，但**不得跳过依赖**（例如不得在 Phase 7 未完成时启动 Phase 10 编码）。

## 5. 进入任一阶段的纪律（文档先于编码）

延续 `AGENTS.md` §4 的开发节奏与 `docs/README.md` 的文档状态约定，进入 Phase 7 ~ Phase 11 中**任一阶段**的编码前，必须满足：

```text
1. 该阶段的 overview 文档（本文档体系中的 phase-N-overview-*.md）存在并已审阅；
2. 针对该阶段，编制详细文档并审核为 active：
     - docs/requirements/phase-N-*-requirements.md
     - docs/design/phase-N-*-design.md（必要时含 phase-N-*-model.md）
     - docs/ui-ux/phase-N-*-view.md（涉及 UI 变更时）
     - docs/testing/phase-N-*-validation.md
     - docs/planning/phase-N-implementation-plan.md
3. 详细文档 active 后，才允许进入该阶段编码；
4. 编码完成后，编写 docs/planning/phase-N-completion-review.md，真实桌面验收通过后标记完成。
```

本轮**只完成第 1 步**：总体路线图 + Phase 7 ~ Phase 11 的 overview 文档，全部 `draft`。第 2 步及之后的详细文档、编码、验收均**未开始**。

## 6. 未来详细文档清单（每阶段编码前必须补齐）

§5 给出了进入纪律的流程。本节列出 Phase 7 ~ Phase 11 **任一阶段**进入编码前必须补齐的详细文档清单。这些文档当前**均不存在**，需在各阶段启动时编制，并逐份审核转为 `active` 后，该阶段才允许编码。

| 文档类型 | 目录 | 用途 | 是否每阶段必需 |
|----------|------|------|----------------|
| 阶段需求 | `docs/requirements/phase-N-*-requirements.md` | 该阶段要解决什么问题、量化验收门槛、非目标 | 是 |
| 技术设计 | `docs/design/phase-N-*-design.md`（必要时含 `phase-N-*-model.md`） | 数据模型、架构、接口、安全设计 | 是 |
| UI/UX 设计 | `docs/ui-ux/phase-N-*-view.md` | 信息架构、视图、交互 | 涉及 UI 变更时必需 |
| 验证设计 | `docs/testing/phase-N-*-validation.md` | 验证策略、量化度量、回归、安全回归 | 是 |
| 编码实施计划 | `docs/planning/phase-N-implementation-plan.md` | 任务拆解、依赖、Batch 划分、退出条件 | 是 |
| 完成审查 | `docs/planning/phase-N-completion-review.md` | 编码完成后真实桌面验收与完成审查 | 编码完成后 |

> 约束：上述任一"必须"文档未达到 `active` 前，对应阶段不得进入编码；每个"详细文档 active"决策都应可追溯到对对应 overview draft 的审阅。

## 7. 验收方向（roadmap 层面）

本路线图本轮（draft 阶段）的验收不涉及任何编码，只验收"方向与纪律是否就位"：

- 6 份 overview 文档（本路线图 + Phase 7 ~ Phase 11 各一份）已建立，状态均为 `draft`；
- 索引已同步（`docs/planning/README.md`、`docs/README.md`、根 `README.md` 均已引用，且不暗示 Phase 7 已开始）；
- Phase 7 及后续阶段**未开始编码**，真实 LLM / 跨阶段映射 / 语义记忆等能力**未实现**；
- 硬约束：上述能力**不得**在任何阶段的详细文档审核转为 `active` 之前被实现。

各阶段的具体量化验收标准，在该阶段的需求文档（`docs/requirements/phase-N-*-requirements.md`）与验证设计（`docs/testing/phase-N-*-validation.md`）中定义，不属于本路线图范畴。

## 8. 整体非目标（适用于整个 Post-MVP）

即便进入 Post-MVP，以下边界在整个 Phase 7 ~ Phase 11 期间始终不变：

- 不做正确性裁决、不做 PASS/HOLD、不做自动审计；
- 不修改目标业务项目，不运行 Vivado / synthesis / implementation / bitstream；
- 不做 server-first / cloud-first 架构；
- 不把产品做成 JSON artifact viewer 或一次性报告生成器；
- 不默认调用真实 LLM，不上传完整源码到外部，不绕过 grounding；
- 不做团队协作平台、审批流、多人分享。

## 9. 版本与标签策略

- 本轮**不打新 tag**。tag `v0.1.0-mvp` 仍是当前唯一发布点。
- 后续每个阶段完成真实桌面验收后，可由该阶段 completion review 提议打 tag（如 Phase 7 完成后提议 `v0.2.0-phase7`），但标签决策由用户在该阶段 completion review 时确定，不在 overview 阶段承诺。

## 10. 风险与边界（整体）

- **顺序风险**：最大风险是在 Phase 7 质量基线未立前就推进 Phase 9/10/11，导致真实 LLM 与跨阶段映射建立在不可信的理解上。本路线图以依赖顺序约束规避。
- **范围扩张风险**：每个阶段都有明确非目标（见各 overview §5），实施时不得脱离文档边界扩张需求（`AGENTS.md` §10）。
- **定位漂移风险**：Post-MVP 容易向"审计器 / dashboard / 大而全可视化平台"漂移；每阶段验收都应复查 `AGENTS.md` §9 审核关注点。
- **安全边界**：真实 LLM 接入会引入新的敏感信息面（API key、源码片段外发），Phase 9 必须显式设计安全边界，其他阶段不得在 overview 之外私自引入外部调用。

## 11. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-15 | 初始 draft：建立 Post-MVP 总体路线图，明确 Phase 7 ~ Phase 11 阶段关系、依赖顺序、进入纪律、整体非目标与版本/标签策略。本文档为总体方向，不含任何阶段详细需求/设计/编码。 | Claude |
| 2026-06-15 | 小修收口：新增"未来详细文档清单（§6）"与"验收方向 roadmap 层面（§7）"小节，原 §6~§9 顺延为 §8~§11；status 保持 draft。 | Claude |
