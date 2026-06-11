# Phase 1 文档收口说明

---
status: active
updated: 2026-06-11
---

> 本文档记录 Phase 1 文档收口范围、active 文档清单和进入编码条件。
> 收口完成后，所有 Phase 1 编码依据文档状态为 `active`，可作为实施唯一权威来源。

## 1. 收口范围

本次收口将以下 7 份 Phase 1 文档从 `draft` 更新为 `active`：

| 文档 | 角色 | 状态变更 |
|------|------|---------|
| [`docs/design/workspace-scanning-and-stage-detection.md`](../design/workspace-scanning-and-stage-detection.md) | Phase 1 技术入口 | `draft` → `active` |
| [`docs/design/phase-1-architecture.md`](../design/phase-1-architecture.md) | Phase 1 概要设计 | `draft` → `active` |
| [`docs/design/phase-1-data-and-api-contract.md`](../design/phase-1-data-and-api-contract.md) | Phase 1 数据/API 契约 | `draft` → `active` |
| [`docs/design/phase-1-scanner-detail-design.md`](../design/phase-1-scanner-detail-design.md) | Phase 1 扫描详细设计 | `draft` → `active` |
| [`docs/ui-ux/phase-1-workspace-and-stage-flow.md`](../ui-ux/phase-1-workspace-and-stage-flow.md) | Phase 1 UI/UX 设计 | `draft` → `active` |
| [`docs/testing/phase-1-workspace-scanning-validation.md`](../testing/phase-1-workspace-scanning-validation.md) | Phase 1 验证设计 | `draft` → `active` |
| [`docs/planning/phase-1-implementation-plan.md`](phase-1-implementation-plan.md) | Phase 1 实施计划 | `draft` → `active` |

## 2. 未收口文档

以下文档**不**在本次收口范围内，保持原状态：

| 文档 | 当前状态 | 原因 |
|------|---------|------|
| `docs/requirements/stories/story-*.md`（10 个 story） | `draft` | 后续实施中可局部细化交互细节，主链路和跨 story 契约已稳定 |
| `docs/initial-requirements-draft/*` | `archived` | 历史草案，仅作参考 |

## 3. 进入编码条件

Phase 1 编码实施开始前必须满足：

1. **所有 active 文档已阅读**：实施者已阅读全部 7 份 active 文档，理解契约、设计、UI/UX、验证和实施顺序。
2. **权威优先级已确认**：当文档冲突时，按 `phase-1-implementation-plan.md` §2 的 7 层优先级裁决。
3. **任务顺序已明确**：按 `phase-1-implementation-plan.md` §5 的 P1-T01~T13 顺序执行，不得跳过类型契约直接编码 scanner。
4. **验证标准已确认**：编码必须满足 `phase-1-workspace-scanning-validation.md` §9 验收 checklist。
5. **安全约束已确认**：目标项目只读、不运行 Vivado/脚本、symlink 安全。

## 4. 编码中变更文档的规则

- 编码阶段发现契约漏洞时，**暂停编码**，更新 active 文档，经审核后再继续。
- 当 active 文档之间冲突时，按 `phase-1-implementation-plan.md` §2 的权威优先级裁决。
- 当同级 active 文档冲突时，**暂停编码**，修正文档并经审核后再继续。
- 任何对 active 文档的修改应更新 `updated` 字段，并在变更处添加说明。
