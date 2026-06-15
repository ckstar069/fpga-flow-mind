# Phase 7 evidence/understanding 质量评估与补强设计

---
status: active
updated: 2026-06-15
---

> 本文档定义 Phase 7（真实项目评估与 evidence/understanding 质量补强）的**评估与补强设计**：如何用真实项目样本驱动质量评估、如何复用 Phase 1~6 既有能力、如何记录质量问题、如何形成补强 backlog，以及 Phase 7 允许修改的边界。
>
> 本文档描述的是"工具理解质量"的评估与补强，**不是目标项目的正确性审计**。所有 issue 描述"工具是否理解到位"，不描述"目标项目对不对"。
>
> 本文档 status 为 `active`，是 Phase 7 编码的设计依据。**Phase 7 编码尚未开始**；允许进入 Phase 7 Batch A（范围仅限 P7-T01~P7-T02）。
>
> 数据模型见 [`phase-7-real-project-evaluation-model.md`](phase-7-real-project-evaluation-model.md)；既有能力/类型以 Phase 1~6 active 设计文档为准。

## 1. 设计目标

1. **用真实样本驱动评估**：以登记的真实 `ai_project_template` 样本为输入，复现 Phase 1~6 主链路（scan → select stage → collect evidence → generate understanding → generate views → trace → Q&A → session），在真实噪声下度量分析质量。
2. **复用而非重写**：Phase 7 不重建理解链路，而是把"评估探针"挂在既有 Phase 1~6 产物之上，读取既有产物并产出质量报告。
3. **诚实暴露问题**：把覆盖缺口、噪声证据、无证据 claim、退化视图、Q&A 失误统一记录为 `QualityIssue`，可追溯、可分类、可分级。
4. **形成可执行的补强 backlog**：评估产出直接驱动"补什么"，补强范围被严格限定（见 §6）。
5. **守住边界**：不接真实 LLM、不做跨阶段映射、不做 UI 大重构、不输出审计结论。

## 2. 复用 Phase 1~6 既有能力

Phase 7 评估流程**直接调用**下列既有能力，不重新实现：

| 既有能力 | 来源 | 在评估中的作用 |
|----------|------|----------------|
| 扫描 workspace | Phase 1 `open_workspace` | 产出 `WorkspaceProfile` / `StageStatus`，用于 RQ-002 阶段识别评估 |
| 选择阶段 | Phase 1 `select_stage` | 产出 `StageContext`，作为逐阶段评估入口 |
| 收集证据 | Phase 2 `collect_evidence` | 产出 `EvidenceCollection`，用于 RQ-003 evidence 覆盖率评估 |
| 生成理解 | Phase 3 `generate_understanding`（MockProvider/确定性） | 产出 `ImplementationUnderstanding`，用于 RQ-004 评估 |
| 生成视图 | Phase 4 `generate_views` | 产出 `ViewGraph[]`，用于 RQ-005 评估 |
| 追溯 | Phase 5 `resolve_trace_target` / `get_source_excerpt` | 验证 `trace_refs` 可解析、source excerpt 可读 |
| Grounded Q&A | Phase 5 `ask_grounded_question`（MockProvider） | 产出 `GroundedAnswer`，用于 RQ-006 评估 |
| session 持久化 | Phase 6 `save_session` / `load_session` | 评估产物与回放状态持久化（仅 app-owned storage） |

评估流程的伪结构：

```text
for sample in RealProjectSample[]:
    open_workspace(sample.root_path)             # 复用 Phase 1
    for stage in expected_stages:
        select_stage(stage)                       # 复用 Phase 1
        evidence   = collect_evidence(stage)      # 复用 Phase 2
        understand = generate_understanding(...)  # 复用 Phase 3（Mock）
        views      = generate_views(understand)   # 复用 Phase 4
        # 挂评估探针（Phase 7 新增）
        evaluate_stage_identification(...)
        evaluate_evidence_quality(...)
        evaluate_understanding_quality(...)
        evaluate_view_quality(...)
        evaluate_qa_quality(...)                  # 复用 Phase 5 Q&A 作输入
    summarize_quality_run(...)
```

> 评估过程对既有能力是**只读消费**：不修改 evidence_id/claim_id/source_path/line_range 绑定，不写回目标项目。

## 3. 评估探针设计（按维度）

### 3.1 阶段识别探针（RQ-002）

- 输入：`WorkspaceProfile`、各阶段 `StageStatus`、人工登记的 `expected_stages`。
- 检查：
  - 识别阶段集合与 `expected_stages` 是否一致；
  - 空阶段是否被正确标 `empty`、缺失阶段标 `missing`、命名异常阶段标 `naming_anomaly` 且可参与后续流程；
  - warning 是否合理（非阻塞、可读）。
- 产出：`StageEvaluationTarget` 与对应 `QualityIssue`；阶段识别误判（命名异常被判 missing、空阶段误判、阶段漏识别等）记录为 `stage_identification_mismatch`（polarity=problem）。

### 3.2 evidence 质量探针（RQ-003）

- 输入：`EvidenceCollection`、`file_type_distribution`（分母来源）。
- 检查：
  - 文件覆盖率：已收集文件 / 应收集文件；
  - `line_range` 是否落在真实文件行范围；
  - `source_kind` / `language` 是否与文件实际匹配；
  - `strength`（`direct`/`indirect`/...）标注是否合理；
  - warnings 中的可疑项是否被降级而非静默丢弃。
- 产出：`EvidenceQualityReport` + `QualityIssue`（`missing_evidence` / `noisy_evidence` / `wrong_source_kind`，均携带 `source_path` / 可选 `line_range` 源码级追溯，polarity=problem）。

### 3.3 understanding 质量探针（RQ-004）

- 输入：`ImplementationUnderstanding`、对应 `EvidenceCollection`。
- 检查：
  - 每个 `ImplementationClaim` 的 `evidence_refs` 是否引用真实存在的 `evidence_id`（existence check）；
  - claim 无 `evidence_refs` 且未标注 `evidence_gap` → `unsupported_claim`；
  - 无证据 claim 被 hallucination guard 拦截 → 记录 `hallucinated_claim_blocked`（**正向 guardrail**，polarity=positive_guardrail，不计入负向 backlog）；
  - `confidence` 标注与 supporting evidence 强度是否一致（`confirmed`/`supported`/`inferred`/`unknown`/`conflicting`）；
  - 证据不足处是否被表达为 `unknown`/`evidence_gap` 而非强行解释；
  - `StageSummary`（short/detailed）是否空洞 → `weak_summary`。
- 产出：`UnderstandingQualityReport` + `QualityIssue`。

### 3.4 视图质量探针（RQ-005）

- 输入：`ViewGraph[]`（structure/dataflow/timing）、`ImplementationUnderstanding`。
- 检查：
  - 每个节点/边的 `ViewTraceRef` 是否可解析回真实 claim/evidence（复用 Phase 5 trace resolve）；
  - 孤立节点（无连边）计数；
  - 错连嫌疑（边两端节点语义不相关）启发式计数；
  - 视图是否退化为空图/无信息 → `empty_or_unhelpful_view`。
- 产出：`ViewQualityReport` + `QualityIssue`（可追溯到 `node_id`）。

### 3.5 Q&A 质量探针（RQ-006）

- 输入：`GroundedAnswer`（含 `claims`/`citations`/`confidence`/`warnings`）、该阶段 evidence/understanding。
- 检查：
  - `citations` 是否指向真实存在的 evidence/claim（复用 existence check）；
  - 对"有证据支持"的问题是否给出回答，否则 `qa_unanswered_when_evidence_exists`；
  - 对"无证据"的问题是否诚实返回 unknown/gap；
  - 回答引用无效 citation → `qa_answer_without_valid_citation`。
- 产出：`QaQualityReport` + `QualityIssue`（可追溯到 `evidence_id`/`claim_id`）。
- **定位**：本探针刻画 MockProvider 在真实样本上的能力边界，产出"基线缺口清单"作为 Phase 9 真实 LLM 的输入；Phase 7 **不**接入真实 LLM。

## 4. 质量问题分类（10 类负向 + 1 类正向 guardrail）

评估探针产出的 `QualityIssue` 按 `QualityIssueKind` 分类，全部围绕"工具理解质量"。`polarity=problem` 为负向问题（进入 backlog），`polarity=positive_guardrail` 为正向守卫生效记录（不计入 backlog）：

| `QualityIssueKind` | 极性 | 含义 | 典型触发 | 追溯键 |
|--------------------|------|------|----------|--------|
| `missing_evidence` | problem | 应被覆盖的证据未被收集 | 真实文件未进入 `EvidenceCollection` | `source_path`/`line_range`（evidence 缺） |
| `noisy_evidence` | problem | 证据含噪声被当主证据 | TODO/注释块/实验代码被提取为 direct | `evidence_id`/`source_path`/`line_range` |
| `wrong_source_kind` | problem | evidence 标注与实际不符 | Python 文件被标 `rtl`，反之亦然 | `evidence_id`/`source_path` |
| `stage_identification_mismatch` | problem | 阶段识别与人工期望不符 | 命名异常被判 missing、空阶段误判、阶段漏识别 | `stage_id` |
| `weak_summary` | problem | StageSummary 空洞 | summary 未抓住阶段核心 | `stage_id` |
| `unsupported_claim` | problem | claim 缺真实 evidence | `evidence_refs` 缺失或未过 existence check | `claim_id` |
| `hallucinated_claim_blocked` | **positive_guardrail** | 无证据 claim 被拦截（守卫生效） | hallucination guard 生效 | `claim_id` |
| `empty_or_unhelpful_view` | problem | 视图退化 | 孤立方块/空图 | `node_id` |
| `qa_unanswered_when_evidence_exists` | problem | 有证据却未回答 | MockProvider 未命中已有证据 | `evidence_id`/`claim_id` |
| `qa_answer_without_valid_citation` | problem | 回答引用无效 citation | citation 指向不存在 evidence | `evidence_id`/`claim_id` |
| `confusing_ui_state` | problem | UI 状态令人困惑（仅 UI 状态表达问题） | 空/加载/降级提示不清 | — |

> 所有 issue 文案禁用"正确/错误""PASS/HOLD""审计结论"；只描述客观事实与不确定性。
> `hallucinated_claim_blocked` 为正向 guardrail，仅作为"守卫工作正常"的证据，不进入补强 backlog、不参与门槛判定。

## 5. 检查方式分层

Phase 7 明确区分三种检查方式，避免把主观判断伪装成自动化结论：

| 方式 | 说明 | 产物字段 |
|------|------|----------|
| 自动化检查 | 评估探针在真实样本上可程序化判定的项（覆盖率、existence check、trace 可解析率、citation 有效性） | `DetectionMethod::Automated` |
| 人工检查 | 需人工判读的项（summary 是否抓住核心、confidence 校准合理性、错连嫌疑确认） | `DetectionMethod::Manual` |
| 真实桌面验收 | 在真实样本上跑完整 Phase 1~6 链路并人工核对（见验证文档） | `DetectionMethod::DesktopAcceptance` |

- 自动化检查只输出**可程序化度量**的客观结果，不下主观结论。
- 主观维度必须由人工/桌面验收标注，不得由探针自动"裁决"。

## 6. Phase 7 补强 backlog 形成与允许修改边界

### 6.1 backlog 形成

- 仅 `polarity=problem` 的 `QualityIssue`（`status=open`）构成补强 backlog 项；`polarity=positive_guardrail`（如 `hallucinated_claim_blocked`）不进入 backlog。
- 负向问题按 `QualitySeverity` 排序：`High` > `Medium` > `Low`。
- backlog 驱动 P7-T08 / P7-T09 的"基于实际发现的补强修复 Batch"。

### 6.2 允许修改范围（Phase 7 内）

为补强质量问题，Phase 7 **允许**修改：

- **evidence extractor**：Phase 2 提取规则（Python/Verilog），提升真实项目覆盖率、降低噪声、修正 `source_kind` 标注；
- **summary 生成**：Phase 3 `StageSummary` 生成规则（仍为确定性/Mock，不调 LLM），减少 `weak_summary`；
- **quality reporting**：新增 Phase 7 评估层（本设计与评估模型文档定义的对象），含 reporter/evaluator；
- **既有 UI 小幅质量提示**：在既有 evidence/understanding/view/Q&A 面板上加最小质量提示（见 UI/UX 文档），不做工作台重构。

### 6.3 禁止越界项（不属于 Phase 7）

- **不做 Phase 8 UI 大重构**：不重写整体布局、不引入图形库、不做导航/信息架构重构。
- **不做 Phase 9 真实 LLM**：不接入真实 LLM Provider、不读取 `api_key`、不调用 OpenAI / Anthropic。
- **不做 Phase 10 跨阶段语义映射**：不做 Python→RTL 映射、不做跨阶段对比。
- **不做正确性裁决**：不输出 PASS/HOLD/正确性裁决/审计结论。
- **不修改目标项目**，不运行 Vivado / synthesis / implementation / bitstream。
- **不破坏核心语义契约**：`ImplementationUnderstanding`/`confidence` 枚举/evidence model 字段语义保持稳定；必要时扩展但不破坏 `mvp-functional-contract.md` 与持久化兼容性。

### 6.4 补强不过拟合

- 针对真实项目补强提取/生成规则时，须用**至少 2 个样本**交叉验证，避免过拟合到单一项目（见需求文档 §4 与验证文档）。
- 规则补强须配套回归测试，防止在简单样例上退化。

## 7. 安全边界

- 评估与补强全程目标项目只读；验收前后 checksum 一致。
- 不调用真实 LLM（不读取 `api_key`、不调用 OpenAI / Anthropic）。
- 不运行 Vivado / synthesis / implementation / bitstream。
- 持久化只写 app-owned storage；评估产物不写回目标项目。
- issue 文案禁用审计用语。

## 8. 关联文档

- [`phase-7-real-project-evaluation-model.md`](phase-7-real-project-evaluation-model.md) — 评估数据模型
- [`../requirements/phase-7-real-project-quality-requirements.md`](../requirements/phase-7-real-project-quality-requirements.md) — 需求（RQ-001~RQ-008）
- [`../ui-ux/phase-7-quality-review-view.md`](../ui-ux/phase-7-quality-review-view.md) — Quality Review 视图
- [`../testing/phase-7-real-project-quality-validation.md`](../testing/phase-7-real-project-quality-validation.md) — 验证与验收
- [`../planning/phase-7-implementation-plan.md`](../planning/phase-7-implementation-plan.md) — 编码实施计划

## 9. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-15 | 初始 draft：复用 Phase 1~6 能力、5 维度评估探针、10 类 issue、检查方式分层、补强 backlog 与允许/禁止边界。Phase 7 未进入编码。 | Claude |
| 2026-06-15 | 审核收口修复（status 保持 draft）：阶段识别误判改用 `stage_identification_mismatch`（不再归入 `confusing_ui_state`）；evidence 类 issue 追溯键补 `source_path`/`line_range`；issue 分类表新增"极性"列并明确 `hallucinated_claim_blocked` 为正向 guardrail；§6.1 backlog 明确仅 polarity=problem 入列、正向 guardrail 不入列。Phase 7 未进入编码。 | Claude |
| 2026-06-15 | 审核通过，status 从 draft 转为 active，作为 Phase 7 编码依据；Phase 7 编码尚未开始。 | Claude |
