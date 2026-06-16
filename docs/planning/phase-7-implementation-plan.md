# Phase 7 编码实施计划

---
status: active
updated: 2026-06-16
---

> 本文档定义 Phase 7（真实项目评估与 evidence/understanding 质量补强）的编码实施计划：任务拆解（P7-T01~P7-T10）、依赖关系、Batch 划分（A~E）、进入/退出条件、安全边界。
>
> 本文档 status 为 `active`，是 Phase 7 编码的实施依据。6 份 Phase 7 详细文档已全部审核转 `active`；**Batch A/B/C 已完成；Batch D P0/P1 已完成；当前允许进入 Batch D P2；Phase 8/9/10/11 仍未开始**。
>
> Phase 7 是质量补强阶段，目标是在真实 `ai_project_template` 项目上验证并提升分析质量，而非新增功能。范围严格收敛于评估与补强，不做 Phase 8/9/10 能力。

## 1. 进入条件

| 条件 | 当前状态 |
|------|----------|
| Phase 6 completion review 已完成 | ✅（active，tag `v0.1.0-mvp`） |
| Phase 7 需求文档 active | ✅ `phase-7-real-project-quality-requirements.md`（active） |
| Phase 7 评估模型设计 active | ✅ `phase-7-real-project-evaluation-model.md`（active） |
| Phase 7 评估/补强设计 active | ✅ `phase-7-evidence-understanding-quality-design.md`（active） |
| Phase 7 UI/UX 文档 active | ✅ `phase-7-quality-review-view.md`（active） |
| Phase 7 测试文档 active | ✅ `phase-7-real-project-quality-validation.md`（active） |
| Phase 7 实施计划 active | ✅ 本文档（active） |
| **以上 Phase 7 详细文档全部转为 active** | ✅ 已满足，**Batch A/B/C 已完成；Batch D P0/P1 已完成；当前允许进入 Batch D P2** |

> 纪律：**Batch D P2 阶段**：在 P0/P1 收口基础上进行质量信号校准和阶段状态隔离修复。不改 evidence/understanding/view/qa 既有核心逻辑、不接真实 LLM、不写目标项目。**P2 完成后可进入 P3（completion review）或 Phase 7 completion review**。Phase 8/9/10/11 仍未开始。

## 2. 任务拆分

### P7-T01 真实项目样本与 evaluation model

| 维度 | 说明 |
|------|------|
| **目标** | 定义 Phase 7 评估数据模型（`RealProjectSample`/`StageEvaluationTarget`/4 类 `QualityReport`/`QualityIssue`+Kind+Severity+Polarity/`QualityRunSummary`/`QualityAcceptanceStatus`，含 `source_path`/`line_range` 追溯字段、`stage_identification_mismatch`、`QaEvaluationQuestionSet`）；登记真实样本结构 |
| **输入文档** | `phase-7-real-project-evaluation-model.md`、`phase-7-real-project-quality-requirements.md` §4 |
| **预计修改文件** | `src-tauri/src/quality/models.rs`（新增）、`src-tauri/src/quality/mod.rs`（新增） |
| **验收命令** | `cargo test --lib quality::models` |
| **不做什么** | 不实现 evaluator、reporter、UI；不改既有 evidence/understanding/view/qa 模型 |

### P7-T02 quality issue model 与 reporter

| 维度 | 说明 |
|------|------|
| **目标** | 实现 `QualityIssue` 记录（含 `polarity`/`source_path`/`line_range` 追溯字段校验）、`QualityRunSummary` 聚合（负向问题与正向 guardrail 分计）、`QualityAcceptanceStatus` 门槛判定（仅看 polarity=problem）、reporter 输出 |
| **输入文档** | `phase-7-real-project-evaluation-model.md` §5~§6、`phase-7-evidence-understanding-quality-design.md` §4 |
| **预计修改文件** | `src-tauri/src/quality/reporter.rs`（新增）、`src-tauri/src/quality/issue.rs`（新增） |
| **验收命令** | `cargo test --lib quality::reporter` |
| **不做什么** | 不接 LLM；不做 PASS/HOLD；issue 文案禁用审计用语 |

### P7-T03 evidence / stage 识别 quality evaluator

| 维度 | 说明 |
|------|------|
| **目标** | evidence 覆盖率、`line_range` 准确性、`source_kind`/`language`/`strength` 合理性检查，产出 `EvidenceQualityReport` + `missing_evidence`/`noisy_evidence`/`wrong_source_kind`（携带 `source_path`/`line_range`）；并执行阶段识别比对，产出 `stage_identification_mismatch`（polarity=problem） |
| **输入文档** | `phase-7-evidence-understanding-quality-design.md` §3.1~§3.2 |
| **预计修改文件** | `src-tauri/src/quality/evidence_evaluator.rs`、`src-tauri/src/quality/stage_evaluator.rs`（新增） |
| **验收命令** | `cargo test --lib quality::evidence_evaluator quality::stage_evaluator` |
| **不做什么** | 不做 AST 级深度提取；不改 Phase 1/2 既有实现（补强在 P7-T08/T09 按实际发现进行） |

### P7-T04 understanding quality evaluator

| 维度 | 说明 |
|------|------|
| **目标** | claim existence check、`unsupported_claim`/`hallucinated_claim_blocked`/`weak_summary` 检测、confidence 校准、unknown/gap 表达检测；产出 `UnderstandingQualityReport` |
| **输入文档** | `phase-7-evidence-understanding-quality-design.md` §3.3 |
| **预计修改文件** | `src-tauri/src/quality/understanding_evaluator.rs`（新增） |
| **验收命令** | `cargo test --lib quality::understanding_evaluator` |
| **不做什么** | 不接 LLM 改写 claim；不评价 claim 工程正确性 |

### P7-T05 view / trace / Q&A quality evaluator

| 维度 | 说明 |
|------|------|
| **目标** | 视图 `trace_refs` 可解析、孤立节点、退化视图检测（`empty_or_unhelpful_view`）；基于 `QaEvaluationQuestionSet` 做 Q&A citation 有效性、有证据未回答、无证据诚实返回检测；产出 `ViewQualityReport` + `QaQualityReport` |
| **输入文档** | `phase-7-evidence-understanding-quality-design.md` §3.4~§3.5 |
| **预计修改文件** | `src-tauri/src/quality/view_evaluator.rs`、`src-tauri/src/quality/qa_evaluator.rs`（新增） |
| **验收命令** | `cargo test --lib quality::view_evaluator quality::qa_evaluator` |
| **不做什么** | 不改 Phase 4 视图生成（补强在 P7-T08/T09）；Q&A 仍用 MockProvider |

### P7-T06 Quality Review UI 最小视图

| 维度 | 说明 |
|------|------|
| **目标** | Quality Review 面板、issue list、stage quality summary、各面板最小质量提示、真实项目验收清单视图；新增读取评估产物的 Tauri command |
| **输入文档** | `phase-7-quality-review-view.md` |
| **预计修改文件** | `src/features/workspace/components/QualityReviewPanel.tsx`（新增）、`src/features/workspace/WorkspacePage.tsx`、`src/features/workspace/components/StageDetail.tsx`、`src/lib/tauriCommands.ts`、`src/types/workspace.ts`、`src-tauri/src/commands/generate_quality_report.rs`（新增只读 command） |
| **验收命令** | `npm run build` + 桌面验收 |
| **不做什么** | 不重写整体布局；不引入图形库；不做工作台重构；文案禁用审计用语 |

### P7-T07 真实项目桌面验收样本与 checklist

| 维度 | 说明 |
|------|------|
| **目标** | 登记至少 2 个真实/等价本地只读样本，覆盖需求文档 §4 全部形态；执行桌面验收步骤；checksum 只读验证；rg 安全回归 |
| **输入文档** | `phase-7-real-project-quality-validation.md` |
| **预计修改文件** | `docs/planning/phase-7-completion-review.md`（新增，初始 draft）、验收夹具说明 |
| **验收命令** | 桌面验收 + checksum + rg |
| **不做什么** | 不修改样本项目；不接 LLM；不运行 Vivado 等 |

### P7-T08 质量补强修复 Batch A（基于实际发现）

| 维度 | 说明 |
|------|------|
| **目标** | 基于桌面验收与 evaluator 实际发现，补强 evidence extractor（Phase 2 规则）与 summary 生成（Phase 3 规则，仍确定性/Mock），修复高优先级 `QualityIssue` |
| **输入文档** | `phase-7-evidence-understanding-quality-design.md` §6 |
| **预计修改文件** | `src-tauri/src/evidence/`（提取规则）、`src-tauri/src/understanding/`（summary 规则） |
| **验收命令** | `cargo test --lib` + 回归测试（多样本交叉验证） |
| **不做什么** | 不接 LLM；不破坏核心契约；不过拟合单一项目 |

### P7-T09 质量补强修复 Batch B（如需要）

| 维度 | 说明 |
|------|------|
| **目标** | 处理 P7-T08 之后剩余的中低优先级 `QualityIssue` 或视图/Q&A 补强（视图派生退化、Q&A 命中） |
| **输入文档** | backlog（`QualityRunSummary`） |
| **预计修改文件** | `src-tauri/src/views/`、`src-tauri/src/trace/qa/`（按需） |
| **验收命令** | `cargo test --lib` + 桌面验收 |
| **不做什么** | 不做 Phase 8/9/10 能力；剩余项可 `accepted_as_known_limitation` |

### P7-T10 Phase 7 completion review

| 维度 | 说明 |
|------|------|
| **目标** | 全量验证、文档状态更新、完成审查、明确进入 Phase 8/9 条件 |
| **输入文档** | `phase-7-real-project-quality-validation.md` §8~§9 |
| **预计修改文件** | `docs/planning/phase-7-completion-review.md`、各 index 更新 |
| **验收命令** | 全量测试 + rg + 桌面验收 + checksum |
| **不做什么** | 不进入 Phase 8/9/10 编码 |

## 3. 依赖关系

```text
P7-T01 (evaluation model)
  │
  ├── P7-T02 (quality issue + reporter)
  │     │
  │     ├── P7-T03 (evidence evaluator)
  │     ├── P7-T04 (understanding evaluator)
  │     └── P7-T05 (view/trace/Q&A evaluator)
  │           │
  │           ▼
  ├── P7-T06 (Quality Review UI)
  │           │
  │           ▼
  ├── P7-T07 (真实样本验收 + checklist)
  │           │
  │           ▼
  ├── P7-T08 (补强 Batch A)
  │           │
  │           ▼
  ├── P7-T09 (补强 Batch B，如需要)
  │           │
  │           ▼
  └── P7-T10 (completion review)
```

## 4. Batch 划分（保守）

### 4.1 Batch A：模型与文档化评估框架

| 任务 | 内容 |
|------|------|
| P7-T01 | evaluation model |
| P7-T02 | quality issue + reporter |

**允许范围**：仅新增 `src-tauri/src/quality/` 评估层模型与 reporter，只读消费既有产物。
**禁止越界**：Batch A 不实现 evaluator 逻辑（已交给 Batch B 实现）；不改既有 evidence/understanding/view/qa 模型；不接 LLM。

> 边界澄清：P7-T02 的 `QualityReporter` 可在内部实现最小 **baseline reporter checks**（如 trace_refs 存在性、空视图、错误 citation 等），用于在 Batch A 产出最小确定性 `QualityReport`。Batch B 已在此基础上拆分为正式 `evidence_evaluator` / `stage_evaluator` / `understanding_evaluator` / `view_evaluator` / `qa_evaluator` 模块。**Batch D P2 进一步校准 view evaluator 的分类和严重度**。

### 4.2 Batch B：后端 evaluator

| 任务 | 内容 |
|------|------|
| P7-T03 | evidence / stage 识别 evaluator |
| P7-T04 | understanding evaluator |
| P7-T05 | view/trace/Q&A evaluator |

**允许范围**：新增各维度 evaluator，只读评估既有产物，产出 `QualityReport` + `QualityIssue`。
**禁止越界**：不改 Phase 2/3/4/5 既有实现（补强在 Batch D）；不下主观裁决；不接 LLM。

### 4.3 Batch C：最小 UI（已实现，待审核收口）

| 任务 | 内容 |
|------|------|
| P7-T06 | Quality Review UI 最小视图 |

**允许范围**：新增 Quality Review 面板（`QualityReviewPanel`）+ 只读 Tauri command（`generate_quality_report`）+ `WorkspacePage` 状态机接入；各面板小幅质量提示为后续 Batch D 可选补强。
**禁止越界**：不重写整体布局；不引入图形库；不做工作台重构；文案禁用审计用语；不接真实 LLM；不写目标项目。

### 4.4 Batch D：真实项目验收与质量补强

| 任务 | 内容 |
|------|------|
| P7-T07 | 真实样本验收 + checklist |
| P7-T08 | 补强 Batch A（evidence/summary 规则） |
| P7-T09 | 补强 Batch B（视图/Q&A，如需要） |

**允许范围**：基于实际发现补强 Phase 2 提取规则、Phase 3 summary 规则、必要时 Phase 4/5 补强；多样本交叉验证；允许使用 `/tmp` 或 app-owned normalized mirror 来适配当前工具对阶段目录的识别需求。
**禁止越界**：不接真实 LLM；不做跨阶段映射；不破坏核心契约；**不修改目标项目，不在目标项目根目录或源码树内创建临时阶段目录/文件**；不运行 Vivado / synthesis / implementation / bitstream。

> **验收方法约束：** P7-T07 真实项目验收必须记录目标项目 `src/` checksum 并保证验收前后一致。若工具无法直接识别真实项目结构，必须将项目复制到 `/tmp` 或 app-owned 临时目录形成 normalized mirror，在镜像上完成所有分析；mirror 须记录来源路径与自身 checksum。禁止以“临时创建顶层阶段目录后删除”的方式在目标项目目录内写入。

### 4.5 Batch E：completion review

| 任务 | 内容 |
|------|------|
| P7-T10 | 全量验证、文档同步、完成审查 |

**允许范围**：文档状态更新、completion review 编写、进入 Phase 8/9 条件说明。
**禁止越界**：不进入 Phase 8/9/10 编码。

## 5. 退出条件

| 条件 | 验证方式 |
|------|----------|
| Rust 全量测试通过 | `cargo test --lib` |
| 前端构建通过 | `npm run build` |
| 评估模型 serde + 追溯字段校验 | 单元测试 |
| 各 evaluator 产出可追溯 `QualityIssue` | 单元测试 + 桌面验收 |
| `QualityIssue` backlog 闭环 | 仅 `polarity=problem` 需 fixed / accepted_as_known_limitation；正向 guardrail（如 `hallucinated_claim_blocked`）不计入 |
| `QualityAcceptanceStatus` 达 meets_gate | 评估产物 |
| 真实样本（≥2）桌面验收通过 | 桌面验收 |
| 目标项目只读 | checksum + rg |
| 无真实 LLM 默认调用 | rg |
| 无 PASS/HOLD 审计用语 | rg |
| Phase 7 completion review 完成 | 文档（转 active） |

## 6. 安全边界

- 不修改 `fpga_project_*` / 目标样本项目；checksum 一致；若需 normalized mirror，须位于 `/tmp` 或 app-owned 目录，并记录 source 与 mirror checksum。
- **禁止在目标项目根目录或源码树内创建临时目录/文件**（包括但不限于为适配 `stage_detector` 而临时放置的 `L0` / `L1` / ... / `RTL` 目录）。
- 不运行 Vivado / synthesis / implementation / bitstream。
- 不调用真实 LLM API（不读取 `api_key`、不调用 OpenAI / Anthropic）。
- 持久化只写 app-owned storage。
- 不破坏核心语义契约（`ImplementationUnderstanding`/`confidence` 枚举/evidence model 字段语义稳定）。
- 不输出 PASS/HOLD/正确性裁决/审计结论。

## 7. 进入 Phase 8 / Phase 9 的条件（预留）

- Phase 7 completion review 转 active。
- 真实样本桌面验收通过。
- 质量问题 backlog 闭环。
- 全量测试通过，安全约束满足。
- Phase 8 / Phase 9 详细文档 active 后方可进入对应编码。
- 不得在 Phase 7 未完成时启动 Phase 9 / Phase 10 编码（依赖顺序见 Post-MVP 路线图）。

## 8. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-15 | 初始 draft：定义 P7-T01~P7-T10、5 个 Batch（A~E）划分与允许/禁止边界、依赖关系、进入/退出条件、安全边界、进入 Phase 8/9 条件。**Batch A/B 后续已实现，当前进入审核收口**，详细文档已 active。 | Claude |
| 2026-06-15 | 审核收口修复（status 保持 draft）：P7-T01/T02 模型范围补 `polarity`/`source_path`/`line_range`/`stage_identification_mismatch`/`QaEvaluationQuestionSet`；P7-T03 扩为 evidence/阶段识别 evaluator；P7-T05 Q&A 改用 `QaEvaluationQuestionSet`；退出条件 backlog 闭环明确仅看 polarity=problem。**Batch A/B 后续已实现，当前进入审核收口**。 | Claude |
| 2026-06-15 | 审核通过，status 从 draft 转为 active，作为 Phase 7 编码依据；Phase 7 Batch A/B 已实现并进入审核收口，Batch C 未授权。 | Claude |
| 2026-06-15 | Batch C 实现：新增 `QualityReviewPanel`、`generate_quality_report` Tauri command、`WorkspacePage` 状态机接入；同步更新 P7-T06 与 Batch C 范围；Batch C 进入审核收口，仍禁止 Batch D/E。 | Claude |
