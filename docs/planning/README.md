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

> **Phase 3 已完成，允许进入 Phase 4。**
>
> 验收确认：
> - ✅ P3-T01~P3-T10 全部完成
> - ✅ `npm run build` 通过，`cargo test` 219 passed，`cargo check` 通过
> - ✅ 真实 Tauri 桌面验收 11/11 通过（样例项目：`/tmp/fpga-flow-mind-phase3-acceptance-20260612-144026`）
> - ✅ checksum 只读验证通过（6 文件前后一致）
> - ✅ 安全约束满足（目标目录只读、无 LLM API、无 Phase 4 图视图）
> - ✅ **允许进入 Phase 4**
>
> Phase 3 编码依据文档已收口（status=active）：`phase-3-understanding-requirements.md` + `phase-3-understanding-model.md` + `phase-3-understanding-generator-design.md` + `phase-3-understanding-view.md` + `phase-3-understanding-validation.md` + `phase-3-implementation-plan.md`。
>
> **Phase 4 文档已收口（status=active），允许进入 Phase 4 Batch A 编码。**
>
> Batch A 范围限定为 P4-T01~P4-T04：ViewGraph 数据模型 + structure/dataflow/timing builder。
> 不允许在 Batch A 中实现前端 UI、Tauri command、Phase 5 回链、真实 LLM、目标项目文件访问。
