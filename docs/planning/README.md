# 计划文档索引

---
status: active
updated: 2026-06-16
---

> 注：本索引的 `status: active` 表示**索引文件本身**为生效文档；下表中 Phase 7 详细文档已 active，**Phase 7 已全部完成（completion review active，详见 [`phase-7-completion-review.md`](phase-7-completion-review.md)）**；Phase 8~11 overview 仍为 `draft`。

## Planning 目录用途

本目录存放 `fpga-flow-mind` 的实施计划文档。计划文档描述"在什么时间完成什么目标、按什么顺序推进、如何验证"，不描述具体技术方案或产品需求。

## 建议文档类型

| 文档类型 | 说明 | 示例 |
|----------|------|------|
| `implementation roadmap` | 实施路线图 | 从当前状态到 MVP 的完整路径 |
| `milestone plan` | 里程碑计划 | 关键节点、交付物、判定标准 |
| `phase checklist` | 阶段清单 | 每个阶段的具体任务列表 |
| `release criteria` | 发布标准 | MVP 可发布的判定条件 |

## 推荐阶段

> 本目录中的 **Phase** 指 `fpga-flow-mind` **本项目**的开发推进阶段，不是业务项目中的 `L0` / `L1` / `RTL` 实现阶段。两者不能混用。

| 阶段 | 名称 | 核心目标 |
|------|------|----------|
| Phase 0 | 文档体系与需求契约 | 建立稳定文档体系，明确需求边界和验收标准 |
| Phase 1 | Workspace 扫描与阶段识别 | 能够打开业务项目，识别阶段目录结构 |
| Phase 2 | 证据索引与 evidence model | 建立证据抽取、索引和存储能力 |
| Phase 3 | 单阶段结构化理解产物 | 生成结构化的 `ImplementationUnderstanding` |
| Phase 4 | 三类视图展示 | 在前端展示结构图、数据流图、时序/流水图 |
| Phase 5 | 证据回链与 grounded Q&A | 支持节点点击追溯证据、用户追问 |
| Phase 6 | 持久化、回放与 MVP 验收 | 产物可持久化加载，完成 MVP 闭环验收 |

### Post-MVP 阶段（Phase 7 已完成；**Phase 8 编码已完成**，completion review draft / pending_desktop_acceptance；Phase 9~11 overview 仍为 draft）

> MVP（Phase 0–6 / tag `v0.1.0-mvp`）是技术闭环 MVP，不等于产品可用性完成。Post-MVP 总体路线图见 [`post-mvp-roadmap.md`](post-mvp-roadmap.md)。**Phase 7 详细文档（requirements/design/ui-ux/testing/implementation-plan）已全部 active，Phase 7 全部完成（Batch A/B/C/D，completion review active）**。Phase 8 详细文档（requirements/design/ui-ux/testing/implementation-plan）**已全部 active**；**Phase 8 编码已完成（Batch A/B/C/D 已完成，Batch E 中 P8-T10 待真实桌面验收，[`completion review`](phase-8-completion-review.md) 仍为 draft / pending_desktop_acceptance）。Phase 9 仅可在 Phase 8 completion 完成后进入；Phase 9~11 overview 仍为 `draft**。

| 阶段 | 名称 | 核心目标 | overview 文档 |
|------|------|----------|---------------|
| Phase 7 | 真实项目评估与 evidence/understanding 质量补强 | 在真实 `ai_project_template` 项目上验证并提升分析能力，让理解产物可信 | [`phase-7-overview-real-project-quality.md`](phase-7-overview-real-project-quality.md) |
| Phase 8 | 产品级 UI/UX 工作台重构 | 把工程调试式界面重构为真实可用的理解工作台 | [`phase-8-overview-product-ui-workbench.md`](phase-8-overview-product-ui-workbench.md) |
| Phase 9 | 真实 LLM Provider 与 grounding 生产化 | 在显式配置、可关闭、可验证前提下接入真实 LLM，守住 grounding 与 citation | [`phase-9-overview-real-llm-grounding.md`](phase-9-overview-real-llm-grounding.md) |
| Phase 10 | 跨阶段理解与 Python-to-RTL 映射 | 组织 L0/L1/.../RTL 实现关系，支持跨阶段对比与 Python 到 RTL 语义映射 | [`phase-10-overview-cross-stage-python-rtl.md`](phase-10-overview-cross-stage-python-rtl.md) |
| Phase 11 | 多阶段语义记忆、测试覆盖图与 agent-scope 联动 | 沉淀可复用语义记忆，探索联动边界 | [`phase-11-overview-semantic-memory-and-integration.md`](phase-11-overview-semantic-memory-and-integration.md) |

推荐主干顺序：Phase 7 → Phase 9 → Phase 10 → Phase 11；Phase 8 可与 Phase 7 部分并行。依赖细节见 [`post-mvp-roadmap.md`](post-mvp-roadmap.md) §4。

## 每阶段必须写清

每个阶段计划文档必须包含：

```markdown
# Phase X: <阶段名称>

---
status: draft | active | superseded | archived
updated: YYYY-MM-DD
---

## 目标

本阶段要达成什么。

## 允许修改范围

本阶段可以修改哪些文件、新增哪些目录。

## 禁止事项

本阶段明确不做的事情。

## 验收标准

如何判定本阶段已完成。

## 测试 / 手工验证方式

如何验证本阶段产出。

## 偏离产品方向的风险检查

- 是否偏离"理解工具"定位？
- 是否引入不必要的复杂度？
- 是否保持目标项目只读？
```

## 当前文档列表

| 文档 | 状态 | 说明 | 推荐阅读时机 |
|------|------|------|-------------|
| [`phase-0-exit-criteria.md`](phase-0-exit-criteria.md) | `active` | Phase 0 退出标准：必须完成的文档、检查表、不扩张边界、进入 Phase 1 的入口 | Phase 0 结束审核前必读 |
| [`phase-1-implementation-plan.md`](phase-1-implementation-plan.md) | `active` | Phase 1 实施计划：任务拆解、编码顺序、验证顺序、退出标准、风险与回滚 | Phase 1 编码实施依据 |
| [`phase-1-documents-closure.md`](phase-1-documents-closure.md) | `active` | Phase 1 文档收口说明：收口范围、active 文档清单、进入编码条件 | Phase 1 编码前必读 |
| [`phase-1-completion-review.md`](phase-1-completion-review.md) | `active` | Phase 1 收尾验收与完成审查：P1-T01~P1-T13 完成状态、真实 Tauri 桌面验收结果、允许进入 Phase 2 | Phase 1 编码完成后必读 |
| [`phase-2-implementation-plan.md`](phase-2-implementation-plan.md) | `active` | Phase 2 实施计划：入口条件、P2-T01~P2-T10 任务拆解、编码顺序、验证顺序、退出标准、风险与回滚 | Phase 2 编码实施依据 |
| [`phase-2-completion-review.md`](phase-2-completion-review.md) | `active` | Phase 2 收尾验收与完成审查：P2-T01~P2-T10 完成状态、真实 Tauri 桌面验收结果、允许进入 Phase 3 | Phase 2 编码完成后必读 |
| [`phase-3-implementation-plan.md`](phase-3-implementation-plan.md) | `active` | Phase 3 编码实施计划：进入条件、P3-T01~P3-T10 任务拆解、依赖关系、4 个 Batch 划分、退出条件、安全边界 | Phase 3 编码实施依据 |
| [`phase-3-completion-review.md`](phase-3-completion-review.md) | `active` | Phase 3 收尾验收与完成审查：P3-T01~P3-T10 全部完成、后端/前端/桌面验收 11/11 通过、**允许进入 Phase 4** | Phase 3 完成后必读 |
| [`phase-4-implementation-plan.md`](phase-4-implementation-plan.md) | `active` | Phase 4 编码实施计划：进入条件、P4-T01~P4-T09 任务拆解、依赖关系、4 个 Batch 划分、退出条件、安全边界 | Phase 4 编码实施依据 |
| [`phase-4-completion-review.md`](phase-4-completion-review.md) | `active` | Phase 4 收尾验收与完成审查：P4-T01~P4-T09 完成状态、后端/前端/桌面验收结果、**允许进入 Phase 5** | Phase 4 完成后必读 |
| [`phase-5-implementation-plan.md`](phase-5-implementation-plan.md) | `active` | Phase 5 编码实施计划：进入条件、P5-T01~P5-T11 任务拆解、5 个 Batch 划分、退出条件、安全边界、进入 Phase 6 条件、Batch A 仅允许 P5-T01~P5-T03 | Phase 5 编码实施依据 |
| [`phase-5-completion-review.md`](phase-5-completion-review.md) | `active` | Phase 5 完成审查：P5-T01~P5-T11 完成状态、真实 Tauri 桌面验收结果、测试/安全回归结果、进入 Phase 6 条件 | Phase 5 完成后必读 |
| [`phase-6-implementation-plan.md`](phase-6-implementation-plan.md) | `active` | Phase 6 编码实施计划：进入条件、P6-T01~P6-T10 任务拆解、5 个 Batch 划分、退出条件、安全边界、进入 Phase 7 条件 | Phase 6 编码实施依据 |
| [`phase-6-completion-review.md`](phase-6-completion-review.md) | `active` | Phase 6 收尾验收与完成审查：P6-T01~P6-T11 全部完成、真实桌面验收通过、checksum 只读验证通过、**允许 Phase 6 / MVP completion** | Phase 6 完成后必读 |
| [`mvp-release-notes.md`](mvp-release-notes.md) | `active` | MVP Release Notes：Phase 0–6 completion 发布说明，已完成能力、验证结果、安全边界、已知限制，对应 tag `v0.1.0-mvp` | MVP 发布审核必读 |
| [`post-mvp-roadmap.md`](post-mvp-roadmap.md) | `draft` | Post-MVP 总体路线图：明确 v0.1.0-mvp 是技术闭环 MVP，给出 Phase 7~11 阶段关系、依赖顺序、进入纪律 | 进入任何 Post-MVP 阶段前必读 |
| [`phase-7-overview-real-project-quality.md`](phase-7-overview-real-project-quality.md) | `draft` | Phase 7 overview：真实项目评估与 evidence/understanding 质量补强 | Phase 7 详细文档编制前必读 |
| [`phase-7-real-project-gap-report.md`](phase-7-real-project-gap-report.md) | `active` | Phase 7 Batch D 真实项目质量基线报告：历史基线（修复前）vs 修复后状态、P0~P2 收口记录 | Phase 7 真实项目验收参考 |
| [`phase-7-implementation-plan.md`](phase-7-implementation-plan.md) | `active` | Phase 7 编码实施计划：P7-T01~P7-T10 任务拆解、5 个 Batch（A~E）划分与允许/禁止边界、依赖关系、进入/退出条件、安全边界。**Phase 7 Batch A/B/C/D 全部完成** | Phase 7 编码实施依据（active） |
| [`phase-7-completion-review.md`](phase-7-completion-review.md) | `active` | Phase 7 收尾验收与完成审查：P7-T01~P7-T10 完成状态、真实项目验收（主/副样本）、前后对比、安全边界、**允许进入 Phase 8 详细文档编制** | Phase 7 完成后必读 |
| [`phase-8-overview-product-ui-workbench.md`](phase-8-overview-product-ui-workbench.md) | `draft` | Phase 8 overview：产品级 UI/UX 工作台重构 | Phase 8 详细文档编制前必读 |
| [`phase-8-implementation-plan.md`](phase-8-implementation-plan.md) | `active` | Phase 8 编码实施计划：P8-T01~P8-T10 任务拆解、5 个 Batch（A~E）划分、进入/退出条件、安全边界。**Phase 8 编码已完成（Batch A/B/C/D 已完成，Batch E 中 P8-T10 待真实桌面验收）** | Phase 8 编码实施依据（active） |
| [`phase-8-completion-review.md`](phase-8-completion-review.md) | `draft` | Phase 8 完成审查草稿：P8-T01~P8-T09 完成，P8-T10 自动化回归与代码级核验已完成，真实桌面验收待完成 | Phase 8 真实桌面验收完成后必读 |
| [`phase-9-overview-real-llm-grounding.md`](phase-9-overview-real-llm-grounding.md) | `draft` | Phase 9 overview：真实 LLM Provider 与 grounding 生产化 | Phase 9 详细文档编制前必读 |
| [`phase-10-overview-cross-stage-python-rtl.md`](phase-10-overview-cross-stage-python-rtl.md) | `draft` | Phase 10 overview：跨阶段理解与 Python-to-RTL 映射 | Phase 10 详细文档编制前必读 |
| [`phase-11-overview-semantic-memory-and-integration.md`](phase-11-overview-semantic-memory-and-integration.md) | `draft` | Phase 11 overview：多阶段语义记忆、测试覆盖图与 agent-scope 联动 | Phase 11 详细文档编制前必读 |

> **MVP / Phase 0–6 已完成（status=active，tag `v0.1.0-mvp`），允许 Phase 6 / MVP completion。**
>
> 当前 active 文档：`phase-6-persistence-and-mvp-requirements.md`、`phase-6-persistence-model.md`、`phase-6-persistence-and-replay-design.md`、`phase-6-session-and-mvp-view.md`、`phase-6-mvp-validation.md`、`phase-6-implementation-plan.md`、`phase-6-completion-review.md`、`mvp-release-notes.md`，以及 Phase 7 详细文档（requirements/design/ui-ux/testing/implementation-plan，均 active）。
>
> **Post-MVP 阶段状态**：Phase 7 详细文档（requirements/design/ui-ux/testing/implementation-plan）**已全部审核转 `active`**，**Phase 7 全部完成（Batch A/B/C/D，[`completion review`](phase-7-completion-review.md) active）**。Phase 8 详细文档（requirements/design/ui-ux/testing/implementation-plan）**已全部 active**；**Phase 8 编码已完成（Batch A/B/C/D 已完成，Batch E 中 P8-T10 待真实桌面验收，[`completion review`](phase-8-completion-review.md) 仍为 draft / pending_desktop_acceptance）。Phase 9 仅可在 Phase 8 completion 完成后进入；Phase 9~11 overview 仍为 `draft**。
