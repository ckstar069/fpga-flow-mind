# 计划文档索引

---
status: active
updated: 2026-06-11
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

> 当前处于 **Phase 1 技术设计补强完成：Workspace 扫描与阶段识别**。
> Phase 0 文档体系与需求契约已完成。
> Phase 1 技术设计已拆分为：概要设计（`phase-1-architecture.md`）、数据/API 契约（`phase-1-data-and-api-contract.md`）、详细设计（`phase-1-scanner-detail-design.md`），均处于 `draft` 状态，需审核后收口。
> 下一步应补 Phase 1 UI/UX 轻量设计、Phase 1 testing/validation 设计、Phase 1 implementation plan。在这些完成前，不进入编码实施。
>
> Phase 0 已完成：
> - ✅ `docs/` 文档体系与索引规则
> - ✅ `docs/requirements/product-scope.md` — 产品范围草案
> - ✅ `docs/requirements/mvp-requirements.md` — MVP 需求草案
> - ✅ `docs/requirements/mvp-functional-contract.md` — MVP 功能契约草案
> - ✅ `docs/requirements/stories/` — 10 个 story 文档
> - ✅ `docs/planning/phase-0-exit-criteria.md` — Phase 0 退出标准
>
> Phase 0 已完成（全部）：
> - ✅ `docs/requirements/product-scope.md` — 从 `draft` → `active`
> - ✅ `docs/requirements/mvp-requirements.md` — 从 `draft` → `active`
> - ✅ `docs/requirements/mvp-functional-contract.md` — 从 `draft` → `active`
> - ✅ `docs/planning/phase-0-exit-criteria.md` — 从 `draft` → `active`
>
> 10 个 story 保持 `draft`，原因：后续实施中可局部细化交互细节，但主链路和跨 story 契约已稳定，不阻塞 Phase 1。
