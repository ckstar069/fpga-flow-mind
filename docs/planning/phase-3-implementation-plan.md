# Phase 3 编码实施计划

---
status: active
updated: 2026-06-12
---

> 本文档定义 Phase 3（单阶段结构化理解产物）的编码实施计划，包含任务拆解、依赖关系、Batch 划分、进入/退出条件和验收标准。
>
> **本文档已审核收口，作为 Phase 3 编码依据。**

## 1. 进入条件

| 条件 | 状态 |
|------|------|
| Phase 2 completion review 状态为 active | ✅ `docs/planning/phase-2-completion-review.md` status=active |
| Phase 2 全量测试通过 | ✅ 160 passed |
| Phase 2 Tauri 桌面验收通过 | ✅ 10 步全部通过 |
| Phase 3 需求文档已创建 | ✅ `docs/requirements/phase-3-understanding-requirements.md` |
| Phase 3 设计文档已创建 | ✅ model + generator design |
| Phase 3 UI/UX 文档已创建 | ✅ `docs/ui-ux/phase-3-understanding-view.md` |
| Phase 3 测试文档已创建 | ✅ `docs/testing/phase-3-understanding-validation.md` |
| Phase 3 实施计划已创建 | ✅ 本文档 |

## 2. 任务拆解

### P3-T01 定义 Rust 数据模型与枚举

| 维度 | 说明 |
|------|------|
| **目标** | 在 `understanding/models.rs` 中定义 ImplementationUnderstanding 及所有子类型 |
| **文件** | `src-tauri/src/understanding/mod.rs`（新增）、`src-tauri/src/understanding/models.rs`（新增）、`src-tauri/src/models/enums.rs`（修改，扩展 ErrorCode） |
| **内容** | ImplementationUnderstanding、ImplementationClaim、ClaimConfidence、ClaimCategory、EvidenceRef、UnknownItem、EvidenceGap、ModuleSummary、SignalSummary、InterfaceSummary、ProcessingStepSummary、GenerationMeta、UnderstandingStats 的 Rust struct/enum 定义 |
| **测试** | serde round-trip（4 个） |
| **依赖** | 无 |
| **安全约束** | 无写入/执行 API |

### P3-T02 实现 ContextBuilder

| 维度 | 说明 |
|------|------|
| **目标** | 将 EvidenceCollection 转换为结构化的 LLM 输入上下文 |
| **文件** | `src-tauri/src/understanding/context_builder.rs`（新增） |
| **内容** | GeneratorInput/GeneratorOutput 结构、ContextBuilder 实现（build_prompt、build_output_schema）、evidence items 精简摘要 |
| **测试** | context builder 输出正确性（5 个） |
| **依赖** | P3-T01 |

### P3-T03 实现 SchemaValidator

| 维度 | 说明 |
|------|------|
| **目标** | 对 generator 输出进行 JSON schema 验证 + evidence_id existence check + 业务规则检查 |
| **文件** | `src-tauri/src/understanding/schema_validator.rs`（新增） |
| **内容** | ValidationResult、ValidationError、ValidationWarning 枚举、SchemaValidator 实现（validate、check_evidence_ids、check_business_rules） |
| **测试** | schema 验证 + evidence_id check（8 个） |
| **依赖** | P3-T01 |

### P3-T04 实现 Provider trait 和 MockProvider

| 维度 | 说明 |
|------|------|
| **目标** | 定义 provider 抽象和 mock 实现 |
| **文件** | `src-tauri/src/understanding/generator.rs`（新增） |
| **内容** | UnderstandingProvider trait、ProviderError、MockProvider、ManualProvider、UnderstandingGenerator 主流程 |
| **测试** | generator pipeline mock 测试（4 个） |
| **依赖** | P3-T01、P3-T02、P3-T03 |

### P3-T05 实现 generate_understanding Tauri command

| 维度 | 说明 |
|------|------|
| **目标** | 暴露理解生成为 Tauri command |
| **文件** | `src-tauri/src/commands/generate_understanding.rs`（新增）、`src-tauri/src/commands/mod.rs`（修改）、`src-tauri/src/lib.rs`（修改） |
| **内容** | generate_understanding command、参数校验、集成 EvidenceCollector + UnderstandingGenerator、CommandResult 返回 |
| **测试** | command 层测试（5 个） |
| **依赖** | P3-T04 |

### P3-T06 实现前端 TypeScript 类型定义

| 维度 | 说明 |
|------|------|
| **目标** | 在前端定义 ImplementationUnderstanding 及所有子类型 |
| **文件** | `src/types/workspace.ts`（修改） |
| **内容** | ImplementationUnderstanding、ImplementationClaim、ClaimConfidence、ClaimCategory、EvidenceRef、UnknownItem、EvidenceGap、ModuleSummary、SignalSummary、InterfaceSummary、ProcessingStepSummary、GenerationMeta、UnderstandingStats 的 TypeScript interface/type 定义 |
| **测试** | TypeScript 编译通过 |
| **依赖** | 无（可与后端并行） |

### P3-T07 实现前端 Tauri command 调用

| 维度 | 说明 |
|------|------|
| **目标** | 新增 generateUnderstanding Tauri command 调用 |
| **文件** | `src/lib/tauriCommands.ts`（修改） |
| **内容** | generateUnderstanding(rootPath, stageId) 函数 |
| **测试** | TypeScript 编译通过 |
| **依赖** | P3-T06 |

### P3-T08 实现 UnderstandingPanel 组件

| 维度 | 说明 |
|------|------|
| **目标** | 新增 UnderstandingPanel 前端组件 |
| **文件** | `src/features/workspace/components/UnderstandingPanel.tsx`（新增） |
| **内容** | 状态栏、阶段摘要、统计概览、claim 列表（ClaimCard）、模块/信号/接口/处理步骤摘要区域、unknown 区域、evidence gap 区域、confidence 颜色映射、evidence 回链交互、禁止用语检查 |
| **测试** | 手工验收（桌面端） |
| **依赖** | P3-T06、P3-T07 |

### P3-T09 集成到 WorkspacePage 状态机

| 维度 | 说明 |
|------|------|
| **目标** | 将 UnderstandingPanel 集成到 StageDetail 和 WorkspacePage 状态机 |
| **文件** | `src/features/workspace/WorkspacePage.tsx`（修改）、`src/features/workspace/components/StageDetail.tsx`（修改） |
| **内容** | AppState 新增 generating_understanding / understanding_loaded / understanding_error 阶段、handleGenerateUnderstanding handler、StageDetail 新增理解生成区域、UnderstandingPanel 嵌入 |
| **测试** | 手工验收 |
| **依赖** | P3-T05、P3-T08 |

### P3-T10 执行 Phase 3 验收与文档同步

| 维度 | 说明 |
|------|------|
| **目标** | 全量验证、文档状态更新 |
| **文件** | `docs/planning/phase-3-implementation-plan.md`（状态更新）、各 index 文件更新 |
| **内容** | 执行验证命令、Tauri 桌面验收、文档状态 draft → active、Phase 3 completion review |
| **测试** | 全量测试 + rg 检查 + 桌面验收 |
| **依赖** | P3-T09 |

## 3. 依赖关系

```text
P3-T01 (models)
  ├── P3-T02 (context builder) ──┐
  └── P3-T03 (schema validator) ─┤
                                 ▼
                          P3-T04 (generator + provider)
                                 ▼
                          P3-T05 (tauri command)
                                 ▼
                          P3-T09 (frontend integration) ──▶ P3-T10 (验收)
                                 ▲
P3-T06 (TS types) ──▶ P3-T07 (tauri command call) ──▶ P3-T08 (UnderstandingPanel)
```

## 4. Batch 划分

### 4.1 Batch 划分原则

- 每个 Batch 2-3 个任务
- 不跨越设计边界（后端 / 前端 / 集成 / 验收）
- 编码前需文档审核收口

### 4.2 Batch A: Rust 数据模型 + ContextBuilder + SchemaValidator（后端基础）

| 任务 | 内容 |
|------|------|
| P3-T01 | Rust 数据模型与枚举 |
| P3-T02 | ContextBuilder |
| P3-T03 | SchemaValidator |

**预估测试**：17 个

### 4.3 Batch B: Generator + Tauri Command + Provider（后端完整）

| 任务 | 内容 |
|------|------|
| P3-T04 | Provider trait + MockProvider + Generator |
| P3-T05 | generate_understanding Tauri command |

**预估测试**：9 个

### 4.4 Batch C: 前端类型 + Command + UnderstandingPanel（前端）

| 任务 | 内容 |
|------|------|
| P3-T06 | TypeScript 类型定义 |
| P3-T07 | Tauri command 调用 |
| P3-T08 | UnderstandingPanel 组件 |

**预估测试**：TypeScript 编译 + 手工验证

### 4.5 Batch D: 集成 + 验收（端到端）

| 任务 | 内容 |
|------|------|
| P3-T09 | WorkspacePage 状态机集成 |
| P3-T10 | 验收与文档同步 |

**预估测试**：全量测试 + rg 检查 + 桌面验收

## 5. 退出条件

| 条件 | 验证方式 |
|------|----------|
| Rust 全量测试通过（含 Phase 1 + Phase 2 + Phase 3） | `cd src-tauri && cargo test` |
| 前端构建通过 | `npm run build` |
| 所有 evidence_refs 中的 evidence_id 存在 | 单元测试 |
| 无伪造 evidence_id | 单元测试 |
| 无写入/执行 API（rg 检查） | `rg "std::fs::write\|..." src-tauri/src/understanding/` |
| 无越界功能（图/Q&A/持久化） | `rg "GraphView\|Dataflow\|Q&A\|LLM" src src-tauri/src` |
| Tauri 桌面验收通过（10 步） | 手工验证 |
| Phase 1/Phase 2 功能无回归 | 全量测试 + UI 验证 |
| 文档状态更新为 active | 各文档 status 字段 |
| Phase 3 completion review 完成 | `docs/planning/phase-3-completion-review.md` |

## 6. 安全边界

Phase 3 编码阶段与 Phase 2 保持相同的安全约束：

- **不创建或修改** `src/`、`src-tauri/` 以外的产品代码
- **不访问或修改** `fpga_project_*`
- **不运行** Vivado / synthesis / implementation / bitstream
- **不调用** LLM API（Phase 3 使用 mock provider）
- **不引入** 新的 crate 依赖（Phase 3 使用 mock，不需要 HTTP/LLM SDK）
- **不修改** Phase 1/Phase 2 的已有功能代码

## 7. 文档变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-12 | 收口修复：status draft → active；文档审核通过，允许进入 Phase 3 编码实施 | Claude |
| 2026-06-12 | 初始创建（draft） | Claude |
