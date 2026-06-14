# 计划文档索引

---
status: active
updated: 2026-06-12
---

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
| [`phase-6-completion-review.md`](phase-6-completion-review.md) | `draft` | Phase 6 收尾验收与完成审查草稿：P6-T01~P6-T10 完成，P6-T11 真实桌面验收待完成，暂不允许 MVP completion | 真实桌面验收完成后必读 |

> **Phase 6 编码与非交互式验证完成，真实桌面验收待完成；暂不允许 MVP completion。**
>
> 当前 active 文档：`phase-6-persistence-and-mvp-requirements.md`、`phase-6-persistence-model.md`、`phase-6-persistence-and-replay-design.md`、`phase-6-session-and-mvp-view.md`、`phase-6-mvp-validation.md`、`phase-6-implementation-plan.md`。
>
> 待真实桌面验收通过后，将 `phase-6-completion-review.md` 更新为 `status: active`，方可允许 Phase 6 / MVP completion。
>
> Batch A 范围限制：
> - ✅ 允许：P6-T01（Phase 6 Rust/TS 持久化数据模型）、P6-T02（StorageVersionService）、P6-T03（WorkspaceFingerprintService）
> - ❌ 不允许：Tauri commands（P6-T07）、前端 UI（P6-T08~P6-T09）、save_session / load_session 完整流程（P6-T04~P6-T07）、delete_session（P6-T07）、自动保存（P6-T01/P6-T09 后续）、Phase 6 completion review（P6-T10）、真实 LLM、写目标 workspace、Phase 7+
>
> 进入 Phase 6 Batch B 及后续前需确认 Batch A 验收通过。
