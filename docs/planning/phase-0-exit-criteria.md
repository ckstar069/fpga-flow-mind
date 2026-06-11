# Phase 0 退出标准

---
status: draft
updated: 2026-06-11
---

## 文档目的

判断 Phase 0 何时可以结束，防止需求未定就进入编码，也防止文档无度扩张。

## Phase 0 目标

Phase 0 只负责：

- 建立稳定文档体系与索引规则
- 明确产品范围、MVP 范围和非目标
- 编写 10 个 MVP story 的功能点、输入输出、异常处理和验收标准
- 建立跨 story 的 MVP 功能契约（对象、枚举、错误码、依赖、端到端验收）

Phase 0 不负责技术实现。

## 必须完成的文档

Phase 0 退出前，以下文档必须存在且通过审核：

| 文档 | 退出时状态要求 |
|------|---------------|
| `docs/README.md` | `active` |
| `docs/requirements/product-scope.md` | 需从 `draft` → `active` |
| `docs/requirements/mvp-requirements.md` | 需从 `draft` → `active` |
| `docs/requirements/mvp-functional-contract.md` | 需从 `draft` → `active` |
| `docs/requirements/stories/README.md` | `active` |
| `docs/requirements/stories/story-*.md`（10 个） | `draft` 或 `active`，但须已审核 |
| `docs/planning/phase-0-exit-criteria.md` | `active`（本文档） |

## 文档状态规则

- `product-scope.md`、`mvp-requirements.md`、`mvp-functional-contract.md` 在 Phase 0 退出前应从 `draft` 转为 `active`。它们定义了产品边界、MVP 范围和跨 story 契约，必须稳定后才能进入技术设计。
- 10 个 story 可保持 `draft` 或 `active`，但必须已经审核，且不能存在阻塞 Phase 1 的缺口。
- 索引类 README 可为 `active`。
- 如果某文档仍为 `draft`，必须在退出检查表中说明为什么不阻塞 Phase 1。

## Phase 0 退出检查表

- [ ] 产品边界清楚（做什么、不做什么）
- [ ] MVP 范围清楚（必须能力、暂不做能力）
- [ ] 非目标清楚（审计器、PASS/HOLD、JSON viewer 等明确排除）
- [ ] 安全边界清楚（目标项目只读、不运行 Vivado 等）
- [ ] 10 个 story 覆盖 MVP 主链路完整闭环
- [ ] 每个 story 有功能点清单、输入、输出、异常/空状态、验收标准
- [ ] MVP 功能契约统一了对象字段、枚举值、错误码和跨 story 数据流
- [ ] 从 `docs/README.md` 可以索引到需求、story、契约和计划文档
- [ ] 没有孤立关键文档
- [ ] 没有进入技术实现或依赖引入
- [ ] `product-scope.md`、`mvp-requirements.md`、`mvp-functional-contract.md` 已转为 `active` 或有明确不阻塞理由

## Phase 0 不继续扩张的边界

Phase 0 到此为止，不再：

- 新增大量低价值需求描述文档
- 写 Rust / Tauri / React 实现
- 写完整技术设计文档
- 写 UI 视觉稿或高保真原型
- 做跨阶段对比、Python→RTL 映射等高级能力
- 访问或修改 `fpga_project_*`
- 运行 Vivado / synthesis / implementation / bitstream

## 进入 Phase 1 的入口

Phase 1 从 **workspace 扫描与阶段识别**开始，优先基于：

- `story-open-workspace.md`（WS-001~007）
- `story-select-stage.md`（ST-001~008）
- `mvp-functional-contract.md` 中的 `workspace_profile.json` 和 `stage_context.json`

Phase 1 的核心目标是：能够打开业务项目，识别阶段目录结构，生成 `workspace_profile.json`。

## Phase 1 前仍可保留的待办

以下待办不阻塞 Phase 1 技术设计，进入对应后续阶段处理：

| 待办 | 原因 | 处理阶段 |
|------|------|----------|
| UI/UX 详细视图设计 | 视图渲染细节可在 Phase 4 前补充 | Phase 3~4 |
| evidence-model 详细技术设计 | 契约已定义对象字段，技术 schema 在 Phase 2 细化 | Phase 2 |
| 真实业务项目样例接入 | 用于手工验证，不影响技术设计启动 | Phase 1~2 |
| 大模型/Agent 调用策略 | grounding 检查机制在 Phase 3 实施前细化 | Phase 3 |
| 持久化存储格式版本策略 | MVP 初期只有一个版本，可在 Phase 6 前细化 | Phase 6 |

---

> 本文档自身在 Phase 0 退出时应转为 `active`。
