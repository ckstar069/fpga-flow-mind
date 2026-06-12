# Phase 4 编码实施计划

---
status: active
updated: 2026-06-12
---

> 本文档定义 Phase 4（三类视图展示）的编码实施计划，包含任务拆解、依赖关系、Batch 划分、进入/退出条件和验收标准。

## 1. 进入条件

| 条件 | 状态 |
|------|------|
| Phase 3 completion review status 为 active | ✅ |
| Phase 3 真实 Tauri 桌面验收通过 | ✅ 11/11 |
| Phase 3 全量测试通过 | ✅ 219 passed |
| Phase 4 需求文档已创建 | ✅ `phase-4-view-requirements.md` |
| Phase 4 数据模型设计已创建 | ✅ `phase-4-view-model.md` |
| Phase 4 生成器设计已创建 | ✅ `phase-4-view-generator-design.md` |
| Phase 4 UI/UX 文档已创建 | ✅ `phase-4-multi-view-panel.md` |
| Phase 4 测试文档已创建 | ✅ `phase-4-view-validation.md` |
| Phase 4 实施计划已创建 | ✅ 本文档 |

## 2. 任务拆解

### P4-T01 定义 Rust 数据模型

| 维度 | 说明 |
|------|------|
| **目标** | 在 `views/models.rs` 中定义 ViewGraph/ViewNode/ViewEdge/ViewTraceRef/ViewLayoutHint/ViewMeta + 枚举 |
| **文件** | `src-tauri/src/views/mod.rs`（新增）、`src-tauri/src/views/models.rs`（新增） |
| **测试** | serde round-trip 4 个 |
| **依赖** | 无（纯数据结构） |

### P4-T02 实现 StructureBuilder

| 维度 | 说明 |
|------|------|
| **目标** | IU → ViewGraph(structure) 确定性转换 |
| **文件** | `src-tauri/src/views/structure_builder.rs`（新增） |
| **测试** | 6 个（正常 IU / 仅 modules / 空 IU / degraded / claims 匹配 / 无匹配） |
| **依赖** | P4-T01 |

### P4-T03 实现 DataflowBuilder

| 维度 | 说明 |
|------|------|
| **目标** | IU → ViewGraph(dataflow) 确定性转换 |
| **文件** | `src-tauri/src/views/dataflow_builder.rs`（新增） |
| **测试** | 6 个（正常 / 无 steps / 单 step / order 排序 / 空 / degraded） |
| **依赖** | P4-T01 |

### P4-T04 实现 TimingBuilder

| 维度 | 说明 |
|------|------|
| **目标** | IU → ViewGraph(timing) 确定性转换 |
| **文件** | `src-tauri/src/views/timing_builder.rs`（新增） |
| **测试** | 6 个（正常 / 无 timing / clock only / reset only / 空 / degraded） |
| **依赖** | P4-T01 |

### P4-T05 实现 ViewGraphGenerator + generate_views command

| 维度 | 说明 |
|------|------|
| **目标** | 总调度器 + Tauri command（纯转换：`generate_views(understanding) → Vec<ViewGraph>`） |
| **文件** | `src-tauri/src/views/generator.rs`（新增）、`src-tauri/src/commands/generate_views.rs`（新增） |
| **测试** | generator 6 个 + command 5 个（纯转换 + degraded + 空 IU） |
| **依赖** | P4-T02～P4-T04 |
| **约束** | command 不接收 root_path/stage_id，不访问目标项目文件，不调用 generate_understanding |

### P4-T06 前端 TypeScript 类型 + command 调用

| 维度 | 说明 |
|------|------|
| **目标** | ViewGraph 等 TS 类型 + `generateViews()` Tauri 调用 |
| **文件** | `src/types/workspace.ts`（修改）、`src/lib/tauriCommands.ts`（修改） |
| **测试** | TypeScript 编译通过 |
| **依赖** | P4-T01 |

### P4-T07 实现 MultiViewPanel 组件

| 维度 | 说明 |
|------|------|
| **目标** | 三 tab + SVG 渲染 + hover tooltip |
| **文件** | `src/features/workspace/components/MultiViewPanel.tsx`（新增） |
| **测试** | 桌面验收 |
| **依赖** | P4-T06 |

### P4-T08 集成到 WorkspacePage + StageDetail

| 维度 | 说明 |
|------|------|
| **目标** | AppState 新增 views_* 阶段 + StageDetail 新增三视图区域 |
| **文件** | `src/features/workspace/WorkspacePage.tsx`（修改）、`src/features/workspace/components/StageDetail.tsx`（修改） |
| **测试** | 桌面验收 |
| **依赖** | P4-T05, P4-T07 |

### P4-T09 执行 Phase 4 验收与文档同步

| 维度 | 说明 |
|------|------|
| **目标** | 全量验证、文档状态更新 |
| **文件** | `docs/planning/phase-4-completion-review.md`（新增）、各 index 文件更新 |
| **测试** | 全量测试 + rg 检查 + 桌面验收 |
| **依赖** | P4-T08 |

## 3. 依赖关系

```text
P4-T01 (models)
  ├── P4-T02 (structure_builder) ──┐
  ├── P4-T03 (dataflow_builder) ──┤
  └── P4-T04 (timing_builder) ────┤
                                    ▼
                             P4-T05 (generator + command)
                                    │
              ┌─────────────────────┤
              ▼                     ▼
       P4-T06 (TS types)    P4-T08 (frontend integration)
              │                     ▲
              ▼                     │
       P4-T07 (MultiViewPanel) ─────┘
                                    ▼
                             P4-T09 (验收)
```

## 4. Batch 划分

### 4.1 Batch A: Rust 数据模型 + 三个 Builder（后端数据层）

| 任务 | 内容 |
|------|------|
| P4-T01 | ViewGraph/ViewNode/ViewEdge 等数据模型 |
| P4-T02 | StructureBuilder |
| P4-T03 | DataflowBuilder |
| P4-T04 | TimingBuilder |

**预估测试**：22 个（model 4 + builder 6×3）。含 node_id 唯一 + edge endpoint 存在 + empty_reason 验证。

### 4.2 Batch B: Generator + Command（后端链路）

| 任务 | 内容 |
|------|------|
| P4-T05 | ViewGraphGenerator + generate_views command（纯 IU→ViewGraph 转换） |

**预估测试**：11 个（generator 6 + command 5）
**约束**：command 不接收 root_path/stage_id，前端必须先有 understanding

### 4.3 Batch C: 前端类型 + MultiViewPanel（前端）

| 任务 | 内容 |
|------|------|
| P4-T06 | TypeScript 类型 + generateViews() |
| P4-T07 | MultiViewPanel 组件 |

**验证**：`npm run build` + 桌面验收

### 4.4 Batch D: 集成 + 验收（端到端）

| 任务 | 内容 |
|------|------|
| P4-T08 | WorkspacePage + StageDetail 集成 |
| P4-T09 | 验收与文档同步 |

**验证**：全量测试 + rg + 桌面验收

## 5. 退出条件

| 条件 | 验证方式 |
|------|----------|
| Rust 全量测试通过 | `cargo test` |
| 前端构建通过 | `npm run build` |
| 三类 ViewGraph 生成正确 | 单元测试 |
| MultiViewPanel 三 tab 渲染 | 桌面验收 |
| Node hover tooltip 展示 trace_refs | 桌面验收 |
| 空状态/degraded 正确 | 单元测试 + 桌面验收 |
| 目标项目只读 | rg + checksum |
| 无 Phase 5 回链功能 | rg |
| 无真实 LLM API | rg |
| Phase 4 completion review 完成 | 文档 |

## 6. 安全边界

- 不修改 `fpga_project_*`
- 不运行 Vivado / synthesis / implementation / bitstream
- 不调用真实 LLM API
- 不引入 React Flow / D3 / Mermaid 等图形库
- 不实现 evidence_id 点击跳转源码（Phase 5）
- 不实现 EvidencePanel 高亮回链（Phase 5）
- 不新增 Rust crate 依赖（除非绝对必要且文档充分论证）

## 7. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-12 | 初始创建（draft）：定义 P4-T01~P4-T09、4 个 Batch、退出条件、安全边界 | Claude |
