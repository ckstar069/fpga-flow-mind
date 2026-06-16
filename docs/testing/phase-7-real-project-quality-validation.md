# Phase 7 真实项目质量评估验证设计

---
status: active
updated: 2026-06-16
---

> 本文档定义 Phase 7（真实项目评估与 evidence/understanding 质量补强）的验证与验收设计：Rust 单元/集成测试方向、前端构建与组件验证方向、真实桌面验收步骤、真实样本验收策略、checksum 只读验证、rg 安全回归、以及 Phase 7 完成标准。
>
> 验证目标是"工具在真实项目上的分析质量是否可信且未越界"，**不是**验证目标项目是否"正确"。
>
> 本文档 status 为 `active`，是 Phase 7 验证与验收依据。**Batch A/B/C 已完成；Batch D P0/P1 已完成；当前允许进入 Batch D P2；Phase 8/9/10/11 仍未开始**。

## 1. 验证策略总览

Phase 7 验证分四层，层层递进：

| 层 | 方式 | 目的 |
|----|------|------|
| 后端单测/集成测试 | Rust `cargo test --lib` | 验证评估模型、evaluator、reporter 的正确性与追溯性 |
| 前端构建与组件验证 | `npm run build` + 必要组件检查 | 验证 Quality Review 视图类型与渲染、不破坏既有构建 |
| 真实桌面验收 | 在真实样本上跑完整 Phase 1~6 + 评估链路 | 验证真实项目下分析质量与安全边界 |
| 安全回归 | checksum + rg | 验证只读、不接 LLM、不运行工具链、不输出审计结论 |

## 2. Rust 单元 / 集成测试方向

计划中的测试模块（具体测试数在实施时收敛，本文给出方向）：

| 模块 | 测试方向 |
|------|----------|
| 评估模型（`quality/models`） | 各对象 serde round-trip；`QualityIssue` 必填追溯字段（`stage_id` + `artifact_kind` + 可选 `source_path`/`line_range`）校验；`QualityIssueKind`/`QualitySeverity`/`QualityIssuePolarity` 枚举完备；`QaEvaluationQuestionSet` 结构 |
| 阶段识别评估 | 阶段识别与 `expected_stages` 比对；空阶段/缺失阶段/命名异常阶段识别；误判记为 `stage_identification_mismatch`（polarity=problem） |
| evidence 质量评估 | 覆盖率计算、`line_range` 越界检测、`source_kind`/`language` 不匹配检测、未覆盖文件归类（issue 携带 `source_path`/`line_range`） |
| understanding 质量评估 | existence check（claim 引用不存在 `evidence_id` → `unsupported_claim`）、hallucination guard 拦截记录（`hallucinated_claim_blocked`，polarity=positive_guardrail，不计入负向 backlog）、unknown/gap 表达检测、`weak_summary` 检测 |
| 视图质量评估 | `trace_refs` 可解析性、孤立节点计数、退化视图检测；**Python L0/L1 无 cycle/latency/clock 证据时 timing 必须为空且有 empty_reason** |
| Q&A 质量评估 | 基于 `QaEvaluationQuestionSet`（人工准备的评估问题集）逐题比对 MockProvider 实际回答：citation 有效性、有证据未回答检测、无证据诚实返回检测 |
| reporter | `QualityRunSummary` 聚合正确（负向问题与正向 guardrail 分计）、`QualityAcceptanceStatus` 门槛判定（仅看 polarity=problem） |
| 追溯性 | 每条 `QualityIssue` 可解析回 `stage_id` + `artifact_kind` + 可选 `evidence_id`/`claim_id`/`node_id`/`source_path`/`line_range`；`polarity=positive_guardrail` 记录不计入负向 backlog |

> 测试夹具使用真实形态的样本结构（含空阶段、命名异常、噪声、跨语言），但不读取真实业务项目源码副本进仓库；测试样本为构造的等价本地只读夹具。

## 3. 前端构建与组件验证方向

| 项 | 方向 |
|----|------|
| 构建 | `npm run build` 通过（TypeScript 编译 + Vite 生产构建） |
| 类型 | Quality Review 相关 TypeScript 类型与评估模型对齐 |
| 组件 | Quality Review 面板、issue list、stage quality summary、各面板最小质量提示在空/加载/有数据态下渲染正常 |
| 文案 | rg 验证前端文案不含"正确/错误""PASS/HOLD""审计结论" |
| 回归 | 既有 Phase 1~6 面板功能不受 Quality Review 引入影响 |

## 4. 真实桌面验收步骤

在真实样本上执行（步骤编号供 completion review 引用，具体在实施时按实际补强结果细化）：

| 步骤 | 验收内容 | 预期 |
|------|----------|------|
| 1 | 打开真实样本项目，workspace 概览、阶段识别（含空/缺失/命名异常）正常 | 识别结果与人工登记一致或差异被记录为 issue |
| 2 | 对 L0/L1/RTL 阶段执行收集证据 → 生成理解 → 生成视图 → 追踪 → Q&A | 主链路在真实噪声下不崩溃 |
| 3 | 运行质量评估，Quality Review 面板展示 summary + issue list | issue 可追溯、可分类、可分级 |
| 4 | evidence 质量提示（覆盖缺口/噪声/source_kind 不符）可见且可定位 | 点击追溯定位到 evidence/源码 |
| 5 | understanding 质量提示（unsupported_claim/weak_summary）可见 | 标注诚实，不掩盖 |
| 6 | 视图质量提示（退化视图/孤立节点）可见 | 退化被如实暴露 |
| 7 | Q&A 质量提示（无效 citation/有证据未回答）可见 | MockProvider 边界被刻画 |
| 8 | 真实项目验收清单视图可用（评估/验收场景） | checklist 勾选状态仅本地/评估用 |
| 9 | 评估产物可持久化与再次加载（仅 app-owned storage） | 不写回目标项目 |
| 10 | 目标项目 checksum 验收前后一致 | 只读验证通过 |

> **严格只读约束：** 验收全程不得向目标项目目录写入任何文件或创建临时目录（包括但不限于 `L0`、`L1`、`RTL` 等阶段目录）。若当前工具无法识别目标项目结构，必须先将项目复制到 `/tmp` 或 app-owned 临时目录形成 normalized mirror，在镜像上操作；禁止为适配工具而向目标项目根目录写入临时文件。

> 步骤数量与具体细节在实施阶段按实际补强发现最终确定；上表为方向性步骤。

## 5. 真实样本验收策略

- **至少 2 个真实 `ai_project_template` 项目样本或等价本地只读样本**进入验收（满足需求文档 §4）。
- 每个样本须覆盖：完整项目形态、L0/L1/RTL 三类阶段、Python/Verilog/SystemVerilog 与 doc/config/test 若干类型、至少 1 个空/缺失阶段、至少 1 个命名异常阶段。
- 真实业务项目样本以**只读输入**参与验收；若不便直接读取真实项目，可用结构等价的本地只读副本，但须在 completion review 中说明等价性。
- 多样本交叉验证补强规则，避免过拟合单一项目。

### 5.1 硬只读约束（Batch D 起必须遵守）

- **目标项目目录必须保持完全只读。** 验收前、验收后均需对 `src/`（或全部源码目录）计算 SHA-256 并比对；任何差异均视为安全边界破坏。
- **禁止在目标项目根目录或源码树内创建临时目录/文件。** 包括但不限于为适配 `stage_detector` 而临时创建的 `L0` / `L1` / ... / `RTL` 顶层目录；此类操作即使事后删除，也构成对目标项目的写入偏差。
- **若工具无法直接识别真实项目结构，必须使用 normalized mirror：**
  - 将目标项目完整复制到 `/tmp` 或 app-owned 临时目录；
  - 在 mirror 上执行所有识别、分析、视图生成与质量评估；
  - 记录 mirror 的 source 路径与 mirror 自身的 checksum，便于追溯。
- **不得将评估产物写回目标项目。** 持久化仅限 app-owned storage 或 `/tmp` 临时产物。
- **历史偏差不复用：** Batch D 基线验收中曾出现“向目标项目根目录写入临时阶段副本后删除”的操作，已在基线报告中标记为验收方法偏差；后续验收必须避免。

## 6. checksum 只读验证

```bash
# 验收前对样本项目源文件计算 checksum 基线
# 验收后重新计算并比对
diff checksums.md checksums-recomputed.md   # 期望：Source files checksums MATCH
```

- 目标项目文件验收前后必须一致。
- 若使用 normalized mirror，须同时记录：
  - 原始目标项目 checksum（证明源未被修改）；
  - mirror 的 source 路径与 mirror checksum（证明镜像来源与一致性）。
- 评估产物只写 app-owned storage，不写回目标项目。

## 7. rg 安全回归

```bash
# 不运行 Vivado / synthesis / implementation / bitstream
rg "Vivado|synthesis|implementation|bitstream" src src-tauri/src
# 期望：仅出现在文档/禁用语境，产品代码无实际调用

# 不调用真实 LLM（不读取 api_key、不调用 OpenAI / Anthropic）
rg "OpenAI|Anthropic|api_key" src src-tauri/src
# 期望：无产品代码匹配（仅文档禁用语境）

# 不写目标项目（无对样本项目的写入调用）

# 不输出 PASS/HOLD / 正确性裁决
rg "PASS|HOLD|正确性裁决|正确/错误|审计结论" src src-tauri/src
# 期望：仅出现在禁用列表/历史结论/错误码文案，不作为当前用户可见结论

# 无对外部进程的隐式调用
rg "Command::new|std::process::Command" src src-tauri/src
# 期望：无匹配（Phase 7 不引入进程调用）
```

> 安全回归命中点需逐条核对：Vivado/synthesis/implementation/bitstream、OpenAI/Anthropic/api_key、PASS/HOLD 等词只允许出现在**禁止/安全边界语境**，不得出现在产品代码的实际调用或当前用户可见结论中。

## 8. Phase 7 完成标准

Phase 7 视为完成，当且仅当：

| 条件 | 验证方式 |
|------|----------|
| P7-T01~P7-T10 全部完成 | 实施计划任务表 |
| 真实项目验收通过（至少 2 样本，桌面验收步骤通过） | 桌面验收 |
| 质量问题记录闭环（`polarity=problem` 的 `QualityIssue` 已 `fixed` 或 `accepted_as_known_limitation`；正向 guardrail 不计入） | `QualityRunSummary` + backlog |
| `QualityAcceptanceStatus` 达 `meets_gate` | 评估产物 |
| 目标项目只读 | checksum + rg |
| 无真实 LLM 默认调用 / 无 PASS-HOLD 审计用语 | rg |
| 全量 `npm run build` / `cargo test --lib` / `cargo check` 通过 | 命令 |
| Phase 7 completion review 转 `active` | 文档 |

## 9. 进入 Phase 8 / Phase 9 的条件

Phase 7 完成后，方允许考虑进入：

- **Phase 8（产品 UI 工作台）**：可基于 Phase 7 暴露的"真实理解形态"定型信息架构；需 Phase 8 详细文档 active。
- **Phase 9（真实 LLM grounding）**：以 Phase 7 产出的"基线缺口清单 + evidence/understanding 质量基线"为输入；需 Phase 9 详细文档 active。
- **不得**在 Phase 7 未完成（completion review 未 active）时启动 Phase 9 / Phase 10 编码（见 Post-MVP 路线图依赖顺序）。

## 10. 安全边界汇总

- 目标项目只读，checksum 一致；若无法直接识别结构，使用 `/tmp` 或 app-owned normalized mirror，并记录 source 与 mirror checksum。
- 不运行 Vivado / synthesis / implementation / bitstream。
- 不调用真实 LLM（不读取 `api_key`、不调用 OpenAI / Anthropic）。
- 持久化只写 app-owned storage。
- 不输出 PASS/HOLD/正确性裁决/审计结论。

## 11. 关联文档

- [`../requirements/phase-7-real-project-quality-requirements.md`](../requirements/phase-7-real-project-quality-requirements.md) — 需求（RQ-001~RQ-008、退出标准）
- [`../design/phase-7-real-project-evaluation-model.md`](../design/phase-7-real-project-evaluation-model.md) — 评估数据模型
- [`../design/phase-7-evidence-understanding-quality-design.md`](../design/phase-7-evidence-understanding-quality-design.md) — 评估与补强设计
- [`../ui-ux/phase-7-quality-review-view.md`](../ui-ux/phase-7-quality-review-view.md) — Quality Review 视图
- [`../planning/phase-7-implementation-plan.md`](../planning/phase-7-implementation-plan.md) — 编码实施计划
- [`phase-6-mvp-validation.md`](phase-6-mvp-validation.md) — MVP 验证（基线）

## 12. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-15 | 初始 draft：定义后端/前端测试方向、桌面验收步骤、≥2 真实样本策略、checksum、rg 安全回归、Phase 7 完成标准与进入 Phase 8/9 条件。Batch A/B 后续已实现，当前进入审核收口。 | Claude |
| 2026-06-15 | 审核收口修复（status 保持 draft）：测试矩阵新增阶段识别评估行（`stage_identification_mismatch`）；追溯字段补 `source_path`/`line_range`；`hallucinated_claim_blocked` 标注为正向 guardrail 不计入负向 backlog；Q&A 评估改用 `QaEvaluationQuestionSet`；完成标准 backlog 闭环明确仅看 polarity=problem。Batch A/B 后续已实现，当前进入审核收口。 | Claude |
| 2026-06-15 | 审核通过，status 从 draft 转为 active，作为 Phase 7 编码依据；Phase 7 Batch A/B 已实现并进入审核收口，Batch C 未授权。 | Claude |
| 2026-06-15 | Batch C 验证：`cargo test --lib` 485 通过；`npm run build` 通过；`commands::generate_quality_report` 单测增至 7 项（新增空阶段错误、有文件无 evidence 诚实暴露 missing_evidence、禁止空 meets_gate 报告）；rg 边界检查无新增产品代码越界（PASS/HOLD/Vivado/LLM/目标项目写入）；前端 Quality Review 按钮 disabled 状态与原因、Issues 标签中文化。 | Claude |
| 2026-06-15 | Batch C 审核收口修复：空阶段无产物返回错误、有文件无 evidence 构造空 EvidenceCollection 暴露 missing_evidence、加载会话清空 qualityReport、质量报告按钮在加载/无评估产物时禁用并给出原因、"Issues" 标签改为"质量记录"；未进入 Batch D/E。 | Claude |
| 2026-06-15 | Batch C 桌面验收与轻量收口：修正 `generate_quality_report.rs` 顶部过期注释，使其与当前 StageEmpty 行为一致；准备 `/tmp` 自包含验收样例项目（L0 Python 3 文件、L1 Python 2 文件、L2 空目录、rtl_final Verilog 2 文件、docs/README.md），验收前后 checksum 一致；Tauri app `npm run tauri build` 成功并在本地二进制可启动；自动化验证 `cargo test --lib` 485 通过、`npm run build` 通过、`npx tsc --noEmit` 通过；rg 边界检查未在产品代码中发现新增 PASS/HOLD/审计用语、真实 LLM/外部进程/Vivado、Phase 8+ 或图形库越界；交互式 GUI 桌面验收清单需在可用 GUI 环境中由用户完成。 | Claude |
| 2026-06-15 | Batch C GUI 验收补充：用户在可用 GUI 环境中完成截图验收，确认 L0 收集 evidence / 生成 understanding / 生成 views / 生成质量报告可显示，Trace / evidence / quality 面板可用；同时观察到 Quality Review 报告暴露 structure view 仅少量节点、dataflow / timing view 为空、`empty_or_unhelpful_view` 等退化项。该结果说明 Batch C UI 可运行，但真实产品可用性与分析价值仍弱，尚不足以支撑 Phase 7 完成结论。 | Claude |
| 2026-06-15 | Batch D 安全收口修正：在 §4 真实桌面验收步骤中增加严格只读约束（禁止向目标项目写入临时目录/文件，必须使用 `/tmp` 或 app-owned normalized mirror）；§5 新增”硬只读约束”子节，明确原始项目与 mirror 双 checksum 记录要求；§6/§10 同步更新安全边界与 checksum 规则。 | Claude |
| 2026-06-16 | **Batch D P0-1/P0-2 验证记录**：`stage_detector` 新增 ai_project_template 深层布局识别（`src/python_model/L0_external` -> L0 等、`src/verilog_model/rtl` -> RTL），顶层目录优先，重复候选生成 warning；`scanner` 新增噪声目录跳过（`.git`、`.claude`、`__pycache__`、`.pytest_cache`、`.mypy_cache`、`.ruff_cache`、`.egg-info`、`reports`、`vivado`、`build`、`dist`、`node_modules`、`target`、`.DS_Store`、`.idea`、`.vscode`、`.venv`、`venv`、`sim_build`、`.tox`、`.coverage`、`htmlcov`）；新增 Rust 单元测试 12 项（stage_detector ai_project_template 布局）+ 7 项（scanner 噪声跳过）；真实项目 `fpga_project_coarse_sync` 只读验证通过（src/ 下 49 个 .py/.v/.sv/.md 文件 SHA-256 前后一致，未在项目根目录创建临时 L0/L1/RTL 目录）；`cargo test --lib` 494 通过、`cargo check` 通过、`npm run build` 通过、`npx tsc --noEmit` 通过；rg 边界检查无新增产品代码越界（PASS/HOLD/审计用语、Vivado/synthesis/implementation/bitstream、OpenAI/Anthropic/api_key、Command::new/外部进程、目标项目写入）。P0-3（dataflow/timing 非空生成）未进入，需单独授权。 | Claude |
| 2026-06-16 | **Batch D P0-1/P0-2 审核收口验证**：发现 P0-2 残留缺陷并修复——原 scanner 深层源码判定 `is_deep_source_dir` 仅匹配阶段根目录名，阶段根子孙目录仍被 `depth > 3` 拦截，导致真实深度 5 源码（`src/python_model/L0_external/rx_02_coarse_sync/coarse_block.py`、`src/python_model/L0_external/shared_04_preamble/preamble.py`）漏扫并产生 `scan_timeout`。重写为 `is_deep_source_root` + `is_inside_deep_source_tree`，阶段根及其全部子孙不再受固定深度限制，噪声目录深度跳过在源码树之外仍生效。新增测试：scanner 3 项（`deep_ai_template_source_files_scanned_without_timeout`、`deep_source_tree_all_descendants_scanned`、`noise_dirs_still_depth_limited_outside_deep_source`）、select_stage 3 项（`ai_project_template_python_stage_selectable`、`ai_project_template_rtl_stage_selectable`、`ai_project_template_deep_source_file_collected`），均用 tempdir，生产代码不写入目标项目。`cargo test --lib` 全量通过、`cargo check` 通过、`npm run build` 通过、`npx tsc --noEmit` 通过；rg 边界检查无新增产品代码越界。P0-3 未进入，需单独授权；未进入 Phase 8/9/10。 | Claude |
| 2026-06-16 | **Batch D P0-3 验证记录（dataflow/timing 最小非空保守生成）**：根因定位——MockProvider 此前硬编码 `signal_summaries/interface_summaries/processing_steps` 为空数组，丢弃了 evidence 已携带的 symbol/excerpt/source_kind，导致 dataflow/timing builder 无可派生输入而退化为空图。修复（保守、可追溯，不接真实 LLM）：(1) `understanding::generator::MockProvider` 新增 `derive_conservative_summaries`——从 evidence context items 的 symbol/summary/source_kind 派生 `processing_steps`（仅 Python 函数/类符号，按 evidence 顺序，confidence=inferred，绑定 evidence_id，跳过 dunder/下划线前缀；RTL 不派生 step 以免把硬件 module 当算法步骤）、`signal_summaries`（从 RTL evidence excerpt 识别 input/output 端口与 clk/clock 关键字，绑定 evidence_id）；interface 本轮保守不派生（需明确契约证据，不伪造）。(2) `views::dataflow_builder` 与 `views::timing_builder` 的顺序/时钟驱动边改为从端点节点的 trace_refs 合并派生（`merge_node_trace_refs`），消除原先 `trace_refs: vec![]` 的推断边（此类空 trace 边会触发 view_evaluator 的 `empty_or_unhelpful_view`）。(3) `views::timing_builder` 新增 RTL 时序保守回退：当 processing_steps 为空但 signal_summaries 含 clk/rst 信号（由 MockProvider 从 RTL evidence 派生）时生成 ClockDomain/ResetDomain 节点；Python 阶段无时序依据时保持空图并给出明确 empty_reason（不再伪造硬件时序）。新增 Rust 单测 14 项（generator 5 项 df_13/14/15 + timing tm_07~10 + view_evaluator 非空可追溯不判 Medium empty 2 项）。真实项目 `fpga_project_coarse_sync` 只读验证（harness 仅 `std::fs::read_dir`/`read`/`metadata`，不写目标项目）：L0 dataflow 12 节点/11 边（基线 0/0）、timing 12/11（基线 0/0）；L1 dataflow 9/8、timing 9/8；RTL dataflow 4 节点/0 边、timing 2/0（clk/rst 节点，无 pipeline 可连 → 孤立节点 Low 提示，非 Medium empty）。**该条为历史记录：其中 L0/L1 timing 12/11 与 9/8 中的 PipelineStage 节点在后续 P0-3 收口修复中被判定为违规产物（Python 函数顺序不应产生时序图），已修正为 timing 0 节点/0 边 + 明确 empty_reason。** 修复前各阶段 dataflow/timing 触发 Medium `empty_or_unhelpful_view`（每阶段 3 条）；修复后 dataflow/timing 的 Medium empty 计数全部归零（RTL 残留 1 条 Low 孤立节点提示为诚实信号）。src/ 下 48 个 .py/.v/.sv/.md 文件 SHA-256 前后一致（`090fd1f4...`），项目根目录无新增临时 L0/L1/RTL 目录。`cargo test --lib` 514 通过（1 ignored 真实项目 harness）、`cargo check` 通过、`npm run build` 通过、`npx tsc --noEmit` 通过；rg 边界检查无新增产品代码越界（PASS/HOLD/审计用语、Vivado/synthesis/implementation/bitstream、OpenAI/Anthropic/api_key、Command::new/外部进程、目标项目写入）。未进入 Phase 8/9/10。 | Claude |
| 2026-06-16 | **Batch D P0-3 收口修复（禁止 Python 函数顺序伪造成 timing 图）**：`timing_builder` 新增 `has_temporal_evidence` 门控函数，明确时序依据包括：step/claim/signal 中出现 cycle/latency/clock/pipeline/stage/clk/rst/posedge 等关键词；RTL 证据含 always_ff/clock/reset。对普通 L0/L1 Python 原型阶段，即使存在 `processing_steps`（由 MockProvider 从函数符号派生），无时序依据时 timing 必须为空并给出 `empty_reason`：“无 cycle/latency/clock/pipeline 等可追溯时序证据，未生成 timing 图（当前 processing_steps 为算法/函数顺序，非硬件时序）”。新增测试：tm_11（Python 多 processing_steps 无时序关键词 → 空图 + 明确 empty_reason）、tm_12（RTL 含 posedge/always_ff → 非空图 + trace_refs 完整）。更新测试：tm_01/tm_03/tm_06/tm_09 补充时序关键词以继续通过。真实项目验证：L0 dataflow 12/11（非空，正确），timing 0/0 + empty_reason（**历史违规产物已修正**）；L1 dataflow 9/8（正确），timing 0/0 + empty_reason（**历史违规产物已修正**）；RTL timing 2/0（clk/rst，正确）。dataflow 未破坏。`cargo test --lib` 516 通过（1 ignored）、`cargo check` 0 warning、`npm run build` 通过、`npx tsc --noEmit` 通过；rg 边界检查无新增越界。未进入 Phase 8/9/10。 | Claude |
| 2026-06-16 | **Batch D P0-4 综合回归验收**：新增集成测试 `tests/real_project_validation.rs`（5 项测试：主样本 L0/L1/RTL 阶段检测、副样本阶段检测、深层扫描无 timeout、噪声目录跳过、checksum 一致性）。主样本 `fpga_project_coarse_sync`：识别 8 阶段（L0~L6+RTL），扫描 152 文件，timeout 0，深层文件（depth 5）10 个全部找到，噪声目录（__pycache__/.git/.claude/vivado/node_modules/target）全部跳过，Python 101 个/Verilog 8 个，checksum 48 文件前后一致。副样本 `fpga_project_fft`：识别 7 阶段（L0~L5+RTL），扫描 60 文件，checksum 前后一致。全量测试：`cargo test --lib` 516 通过（1 ignored）、集成测试 5 通过（0 ignored）、`cargo check` 0 warning、`npm run build` 通过、`npx tsc --noEmit` 通过。rg 边界检查：产品代码无 `std::fs::write`/`create_dir`/`remove_file`/`remove_dir`/`rename`/`copy`/`Command::new`（测试代码除外）；无 Vivado/synthesis/implementation/bitstream（仅 quality/mod.rs 注释说明不做）；无 OpenAI/Anthropic/api_key；无 PASS/HOLD/审计用语（"正确"/"错误"仅出现在代码注释/错误处理语境，非审计裁决）。文档更新：`docs/testing/phase-7-real-project-quality-validation.md` 追加 P0-4 验收记录。本轮不改产品代码，仅新增集成测试与文档。未进入 Phase 8/9/10。 | Claude |
| 2026-06-16 | **Batch D P1 evidence/understanding 丰富度补强**：Python extractor 新增 imports/constants/dataclass-fields/return-types/self-fields/call-sites 提取（+7 测试）；Verilog/SV extractor 新增 ports/signals/assign/always/instances/parameters 提取（+13 测试）；MockProvider claims 从封顶 3 到每条 evidence 独立生成，module_summaries 从 1 扩展到多个，interface_summaries 从空到派生，unknowns/gaps 基于维度缺失；structure_builder 边生成从首模块 break 到 evidence_id 匹配多对多。全量 `cargo test --lib` **536 通过**（+20，0 failed，1 ignored）；`cargo check` 通过；`npm run build` 通过；`npx tsc --noEmit` 通过；real_project_validation 5 项通过（checksum 一致）；rg 边界检查无新增越界。未进入 Phase 8/9/10。 | Claude |
| 2026-06-16 | **Batch D P2 质量信号校准 + 状态隔离**：引入 4 个更细 QualityIssueKind（`ExpectedEmptyTiming`/`IsolatedOrUnconnectedView`/`TraceabilityGap`/`LowSemanticDiversity`）；`view_evaluator` 校准：Python timing 空图 → Low expected_empty_timing（非 Medium empty_or_unhelpful）、缺 trace_refs → TraceabilityGap、孤立节点 → IsolatedOrUnconnectedView、重复标签 → LowSemanticDiversity；`reporter` 新增 structure 降解检测（多 summary 单节点 → LowSemanticDiversity Medium）；前端状态隔离：evidence 重新收集时清除 downstream maps（understanding/views/QA）、understanding 再生时清除 views/QA、views 再生时清除 QA；新增 view_evaluator 测试 7 项（expected_empty_timing、non_timing_empty、traceability_gap、traceable_dataflow、duplicate_labels、varied_labels、missing_trace_refs）；更新既有测试 4 项（trace_refs 从 EmptyOrUnhelpfulView 改为 TraceabilityGap、isolated 从 EmptyOrUnhelpfulView 改为 IsolatedOrUnconnectedView、timing isolated 断言适配 Medium）。全量 `cargo test --lib` **544 通过**（+8，0 failed，1 ignored）；`cargo test --test real_project_validation -- --ignored` 5 项通过（checksum 一致）；`cargo check` 通过；`npm run build` 通过；`npx tsc --noEmit` 通过；rg 边界检查无新增越界。未进入 Phase 8/9/10。 | Claude |
