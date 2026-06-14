# Phase 5 编码实施计划

---
status: draft
updated: 2026-06-13
---

> 本文档定义 Phase 5（证据回链与 grounded Q&A）的编码实施计划，包含任务拆解、依赖关系、Batch 划分、进入/退出条件、验收标准和安全边界。需待本文档及配套需求/设计/UI/测试文档全部 active 后才允许编码。
>
> 本文档为 draft，需审核收口后方可进入编码。

## 1. 进入条件

| 条件 | 状态 |
|------|------|
| Phase 4 completion review status 为 active | ✅ |
| Phase 4 真实 Tauri 桌面验收通过 | ✅ 12/12 |
| Phase 5 需求文档已创建 | ✅ `phase-5-trace-and-qa-requirements.md`（draft） |
| Phase 5 数据模型设计已创建 | ✅ `phase-5-trace-model.md`（draft） |
| Phase 5 后端设计已创建 | ✅ `phase-5-trace-and-qa-design.md`（draft） |
| Phase 5 UI/UX 文档已创建 | ✅ `phase-5-trace-and-qa-view.md`（draft） |
| Phase 5 测试文档已创建 | ✅ `phase-5-trace-and-qa-validation.md`（draft） |
| Phase 5 实施计划已创建 | ✅ 本文档（draft） |
| **以上文档全部转为 active** | ⏳ 当前未满足，不允许编码 |

## 2. 任务拆分

### P5-T01 定义 Phase 5 Rust 数据模型

| 维度 | 说明 |
|------|------|
| **目标** | 在 `trace/models.rs` 中定义 SelectedTraceTarget、TraceRefResolved、SourceLocation、SourceExcerpt、TracePanelState、GroundedQuestion、GroundedAnswer 等 |
| **输入文档** | `phase-5-trace-model.md` |
| **预计修改文件** | `src-tauri/src/trace/mod.rs`（新增）、`src-tauri/src/trace/models.rs`（新增） |
| **验收命令** | `cargo test --lib trace::models` |
| **不做什么** | 不实现 resolver/command/UI |

### P5-T02 实现 TraceResolver

| 维度 | 说明 |
|------|------|
| **目标** | 根据 SelectedTraceTarget 解析出 TraceRefResolved[] |
| **输入文档** | `phase-5-trace-and-qa-design.md` §2 |
| **预计修改文件** | `src-tauri/src/trace/resolver.rs`（新增） |
| **验收命令** | `cargo test --lib trace::resolver` |
| **不做什么** | 不读取目标项目文件 |

### P5-T03 实现 SourceExcerptResolver

| 维度 | 说明 |
|------|------|
| **目标** | 安全读取目标项目文件，返回 SourceExcerpt |
| **输入文档** | `phase-5-trace-and-qa-design.md` §4 |
| **预计修改文件** | `src-tauri/src/trace/source_resolver.rs`（新增） |
| **验收命令** | `cargo test --lib trace::source_resolver` |
| **验收点** | 1. 正常 source_path 返回 `SourceExcerpt`；2. root_path 本身为 symlink 时拒绝；3. source_path 或任意父目录为 symlink 时拒绝；4. 字符串前缀路径（如 `/tmp/root2/...` 针对 `/tmp/root`）被拒绝；5. canonicalize 后跳出 root_path 的路径（`..` 拼接等）被拒绝；6. 超大/二进制/非 UTF-8 文件返回可读错误；7. 越界 `line_range` 返回错误；8. 不写入目标项目 |
| **不做什么** | 不写入文件；不读取 root_path 外文件 |

### P5-T04 实现 GroundedQaContextBuilder

| 维度 | 说明 |
|------|------|
| **目标** | 从 understanding + evidence + question 构建 Q&A 上下文 |
| **输入文档** | `phase-5-trace-and-qa-design.md` §5.2 |
| **预计修改文件** | `src-tauri/src/trace/qa/context_builder.rs`（新增） |
| **验收命令** | `cargo test --lib trace::qa::context_builder` |
| **不做什么** | 不调用 LLM |

### P5-T05 实现 GroundedQaProvider trait + MockProvider

| 维度 | 说明 |
|------|------|
| **目标** | Provider trait + 确定性 MockProvider |
| **输入文档** | `phase-5-trace-and-qa-design.md` §5 |
| **预计修改文件** | `src-tauri/src/trace/qa/provider.rs`、`src-tauri/src/trace/qa/mock_provider.rs` |
| **验收命令** | `cargo test --lib trace::qa::mock_provider` |
| **不做什么** | 不接入真实云端 LLM |

### P5-T06 实现 GroundedQaValidator

| 维度 | 说明 |
|------|------|
| **目标** | 检查 answer 的 citations、confidence、禁用词汇 |
| **输入文档** | `phase-5-trace-and-qa-design.md` §6 |
| **预计修改文件** | `src-tauri/src/trace/qa/validator.rs`（新增） |
| **验收命令** | `cargo test --lib trace::qa::validator` |
| **不做什么** | 不修改 answer 语义，只验证 |

### P5-T07 实现 Phase 5 Tauri commands

| 维度 | 说明 |
|------|------|
| **目标** | `resolve_trace_target`、`get_source_excerpt`、`ask_grounded_question` |
| **输入文档** | `phase-5-trace-and-qa-design.md` §3 |
| **预计修改文件** | `src-tauri/src/commands/resolve_trace_target.rs`、`get_source_excerpt.rs`、`ask_grounded_question.rs` |
| **验收命令** | `cargo test --lib commands` |
| **不做什么** | `resolve_trace_target` 和 `ask_grounded_question` 不访问目标项目文件 |

### P5-T08 前端 TypeScript 类型 + command 调用

| 维度 | 说明 |
|------|------|
| **目标** | 扩展 `src/types/workspace.ts` + `src/lib/tauriCommands.ts` |
| **输入文档** | `phase-5-trace-model.md` |
| **预计修改文件** | `src/types/workspace.ts`、`src/lib/tauriCommands.ts` |
| **验收命令** | `npm run build` |
| **不做什么** | 不实现 UI 组件 |

### P5-T09 前端 selection + TracePanel + SourceExcerptPanel + EvidencePanel 高亮

| 维度 | 说明 |
|------|------|
| **目标** | MultiViewPanel 可选中、TracePanel、SourceExcerptPanel、EvidencePanel 高亮 |
| **输入文档** | `phase-5-trace-and-qa-view.md` |
| **预计修改文件** | `src/features/workspace/components/MultiViewPanel.tsx`（修改）、新增 `TracePanel.tsx`、`SourceExcerptPanel.tsx`，修改 `EvidencePanel.tsx` |
| **验收命令** | `npm run build` + 桌面验收 |
| **不做什么** | 不做 GroundedQAPanel |

### P5-T10 前端 GroundedQAPanel

| 维度 | 说明 |
|------|------|
| **目标** | GroundedQAPanel 组件 + 问答历史展示 |
| **输入文档** | `phase-5-trace-and-qa-view.md` |
| **预计修改文件** | 新增 `GroundedQAPanel.tsx`，修改 `WorkspacePage.tsx`/`StageDetail.tsx` 状态机 |
| **验收命令** | `npm run build` + 桌面验收 |
| **不做什么** | 不做持久化 |

### P5-T11 Phase 5 验收与文档同步

| 维度 | 说明 |
|------|------|
| **目标** | 全量验证、文档状态更新、完成审查 |
| **输入文档** | `phase-5-trace-and-qa-validation.md` |
| **预计修改文件** | `docs/planning/phase-5-completion-review.md`（新增）、各 index 更新 |
| **验收命令** | 全量测试 + rg 检查 + 桌面验收 |
| **不做什么** | 不进入 Phase 6 编码 |

## 3. 依赖关系

```text
P5-T01 (models)
  │
  ├── P5-T02 (TraceResolver) ──┐
  ├── P5-T03 (SourceExcerptResolver) ──┤
  ├── P5-T04 (ContextBuilder) ─────────┤
  │                                    ▼
  │                             P5-T05 (MockProvider)
  │                                    │
  │                             P5-T06 (Validator)
  │                                    │
  │                             P5-T07 (commands)
  │                                    │
  ├────────────────────────────────────┤
  ▼                                    ▼
P5-T08 (TS types/commands)      P5-T09 (selection + TracePanel)
  │                                    │
  └────────────────────────────────────┘
         │
         ▼
    P5-T10 (GroundedQAPanel)
         │
         ▼
    P5-T11 (验收与 completion review)
```

## 4. Batch 划分

### 4.1 Batch A：Trace model + resolver + source excerpt resolver

| 任务 | 内容 |
|------|------|
| P5-T01 | Phase 5 Rust 数据模型 |
| P5-T02 | TraceResolver |
| P5-T03 | SourceExcerptResolver |

**预估测试**：26 个（model 4 + resolver 10 + source_resolver 12）。

### 4.2 Batch B：Tauri commands + Rust 测试

| 任务 | 内容 |
|------|------|
| P5-T07 | resolve_trace_target / get_source_excerpt / ask_grounded_question |

**预估测试**：14 个（command 层 4+6+4）。
**约束**：`resolve_trace_target` 和 `ask_grounded_question` 不访问目标项目文件。

### 4.3 Batch C：前端 selection + TracePanel + EvidencePanel 高亮

| 任务 | 内容 |
|------|------|
| P5-T08 | TypeScript 类型 + command 调用 |
| P5-T09 | MultiViewPanel 选中态 + TracePanel + SourceExcerptPanel + EvidencePanel 高亮 |

**验证**：`npm run build` + 桌面验收（步骤 1~5）。

### 4.4 Batch D：Grounded Q&A mock provider + UI

| 任务 | 内容 |
|------|------|
| P5-T04 | GroundedQaContextBuilder |
| P5-T05 | GroundedQaProvider trait + MockProvider |
| P5-T06 | GroundedQaValidator |
| P5-T10 | GroundedQAPanel |

**验证**：`npm run build` + 桌面验收（步骤 6~8）。

### 4.5 Batch E：桌面验收 + completion review

| 任务 | 内容 |
|------|------|
| P5-T11 | 全量验证、文档同步、完成审查 |

**验证**：全量测试 + rg + 桌面验收 + checksum。

## 5. 退出条件

| 条件 | 验证方式 |
|------|----------|
| Rust 全量测试通过 | `cargo test --lib` |
| 前端构建通过 | `npm run build` |
| TraceResolver 解析正确 | 单元测试 |
| SourceExcerptResolver 安全只读 | 单元测试 + rg |
| MockProvider 回答带 citations | 单元测试 |
| MultiViewPanel 节点/边可选中 | 桌面验收 |
| TracePanel 展示 trace_refs | 桌面验收 |
| EvidencePanel 高亮 | 桌面验收 |
| GroundedQAPanel 回答带 citations | 桌面验收 |
| 目标项目只读 | rg + checksum |
| 无真实 LLM 默认调用 | rg |
| 无 PASS/HOLD 审计用语 | rg |
| Phase 5 completion review 完成 | 文档 |

## 6. 安全边界

- 不修改 `fpga_project_*`
- 不运行 Vivado / synthesis / implementation / bitstream
- 不调用真实 LLM API（Phase 5 使用 MockProvider）
- `get_source_excerpt` 只允许读取当前 workspace root 下的文件
- 拒绝 symlink / path traversal / 超大文件 / 二进制 / 非 UTF-8
- 不实现 evidence 点击打开外部编辑器
- 不实现 EvidencePanel 高亮以外的任何写操作
- 不输出 PASS/HOLD/正确/错误等审计结论

## 7. 进入 Phase 6 的条件

- Phase 5 completion review status 为 active。
- Phase 5 真实 Tauri 桌面验收通过。
- 全量测试通过。
- 安全约束满足。
- **Phase 6 需求/设计/计划文档 active 后**方可进入 Phase 6 编码。

## 8. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-13 | 初始创建（draft）：定义 P5-T01~P5-T11、5 个 Batch、退出条件、安全边界、进入 Phase 6 条件 | Claude |
