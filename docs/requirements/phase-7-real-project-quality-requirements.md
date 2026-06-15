# Phase 7 真实项目评估与 evidence/understanding 质量补强需求

---
status: active
updated: 2026-06-15
---

> 本文档定义 Phase 7（真实项目评估与 evidence/understanding 质量补强）的产品需求。
>
> **Phase 7 的目标不是新增炫酷功能，而是验证并提升真实 `ai_project_template` 项目上的分析质量。**
> MVP（Phase 0–6 / tag `v0.1.0-mvp`）已在 `/tmp` 手工构造的临时小型样例上证明单阶段理解闭环技术成立，但尚未在真实业务项目上验证分析质量是否可信。Phase 7 把"技术上成立"推进为"在真实项目上可信"。
>
> 本文档 status 为 `active`，是 Phase 7 编码的需求依据。**Phase 7 Batch A/B 已实现并进入审核收口，Batch C 尚未授权**。

## 1. 用户目标

- 用户能在**自己真实**的 `ai_project_template` 业务项目上打开 `fpga-flow-mind`，得到可信、可追溯、诚实表达不确定性的理解产物，而不是只在 toy 样例上有效。
- 用户能感知"哪些结论由证据直接支持、哪些是推断、哪些是 evidence gap"，且这些标注在真实噪声（TODO、被注释代码、跨文件 import、命名异常阶段）下仍然诚实。
- 用户能看到工具对自身分析质量的诚实自评（覆盖率、命中率、缺口），而不是把分析能力当成黑盒。
- 用户感知到工具"在哪些地方没看懂"，比"看起来都看懂了"更有价值——Phase 7 的核心价值是**让理解质量可度量、可补强**。

## 2. 业务背景

MVP 的真实桌面验收使用的是 `/tmp` 下手工构造的临时小型样例（`L0/L1/L2/rtl_final/docs`，每阶段 1–3 个文件）。这套样例验证了**单阶段理解闭环在技术上成立**，但真实 `ai_project_template` 业务项目与临时样例存在系统性差距：

| 维度 | 临时样例 | 真实业务项目 |
|------|----------|--------------|
| 文件规模 | 每阶段 1–3 个文件 | 单阶段可能数十个 Python/Verilog 文件、嵌套目录 |
| 语言混用 | 单一、清晰 | 前期 Python、后期 Verilog/SystemVerilog，并存约束文件、生成脚本 |
| 外部依赖 | 无 | 从 `urban_wireless` 导入模块、相对/绝对 import、跨阶段接口约定 |
| 噪声 | 几乎无 | TODO 注释、实验性代码、被注释块、文档与代码不一致 |
| 阶段形态 | L0/L1/RTL 各一个 | 空阶段、命名异常阶段、并行多模块阶段 |

这些差距会在 evidence 覆盖率、提取质量、`ImplementationUnderstanding` 可信度、视图可用性、Q&A 命中率上集中暴露。MVP 没有对这些维度做过量化评估，因此**当前无法回答"在真实项目上，理解产物到底有多可信"**。Phase 7 的核心命题是：**在真实项目上，量化并提升分析质量，让理解产物可信。**

## 3. 功能点

### RQ-001 真实项目样本集定义与输入要求

| 维度 | 说明 |
|------|------|
| **目标** | 定义一组有代表性的真实 `ai_project_template` 业务项目作为评估语料（非 `/tmp` 临时样例），明确每个样本的登记字段（来源、阶段构成、文件类型、规模、已知特性）与只读输入约束 |
| **输入** | 用户/评估者选定的真实业务项目根路径，或其等价本地只读副本 |
| **输出** | `RealProjectSample` 登记记录，含 `sample_id`、`root_path`、阶段清单、文件类型分布、规模度量、特性标签（空阶段 / 命名异常 / 多语言等） |
| **验收标准** | 至少 2 个真实（或等价本地）样本被登记并进入评估；每个样本的登记字段完整可追溯 |
| **非目标** | 不修改样本项目；不把样本源码外发；不评估样本项目本身是否"正确" |

### RQ-002 workspace / stage 识别质量评估

| 维度 | 说明 |
|------|------|
| **目标** | 在真实样本上量化 Phase 1 workspace 扫描与阶段识别（`StageStatus`：`available`/`empty`/`missing`/`naming_anomaly`/`unreadable`）的准确性，刻画空阶段、缺失阶段、命名异常阶段的识别表现 |
| **输入** | `WorkspaceProfile`、各阶段 `StageStatus`、warnings |
| **输出** | `StageEvaluationTarget` 与对应的质量度量：阶段命中、误判（如把命名异常阶段判为 missing，记为 `stage_identification_mismatch`）、warning 合理性 |
| **验收标准** | 真实样本上的阶段识别结果与人工登记的阶段清单一致（空阶段标 empty、命名异常可识别、缺失标 missing）；不一致处被记录为 `QualityIssue`（kind=`stage_identification_mismatch`）而非被掩盖 |
| **非目标** | 不替用户裁决阶段实现是否正确；不修改目标项目阶段目录 |

### RQ-003 evidence 覆盖率与缺口识别

| 维度 | 说明 |
|------|------|
| **目标** | 在真实样本上量化 Phase 2 evidence 收集的文件覆盖率、符号命中率、`line_range` 准确性、`source_kind` / `language` 标注合理性、`strength` 标注合理性，并显式识别 evidence 缺口 |
| **输入** | `EvidenceCollection`（`EvidenceItem[]`、warnings、stats） |
| **输出** | `EvidenceQualityReport`：覆盖的源文件比例、未覆盖文件及原因、`missing_evidence` / `noisy_evidence` / `wrong_source_kind` 缺口列表 |
| **验收标准** | 覆盖率达到量化门槛（待详细设计收敛）；未覆盖或可疑证据被记录为 `QualityIssue` 并可追溯到 `stage_id` + `source_path`，而非静默丢弃 |
| **非目标** | 不做 AST 级深度语义提取（保持 Phase 2 的正则/行级匹配定位）；不修改目标项目文件 |

### RQ-004 ImplementationUnderstanding 质量评估

| 维度 | 说明 |
|------|------|
| **目标** | 在真实样本上量化 Phase 3 `ImplementationUnderstanding` 的质量：claim 的 `evidence_refs` 真实性、`confidence`（`confirmed`/`supported`/`inferred`/`unknown`/`conflicting`）标注合理性、`unknown` / `evidence_gap` 是否被诚实表达而非强行解释、hallucination guard 是否有效拦截无证据 claim |
| **输入** | `ImplementationUnderstanding`（`ImplementationClaim[]`、`UnknownItem[]`、`EvidenceGap[]`、`StageSummary`） |
| **输出** | `UnderstandingQualityReport`：claim 通过 existence check 的比例、`unsupported_claim` / `hallucinated_claim_blocked` / `weak_summary` 记录、unknown/gap 表达合理性评估 |
| **验收标准** | 所有 claim 通过 existence check（无引用不存在的 `evidence_id`）；证据不足处被表达为 `unknown`/`evidence_gap` 而非伪造支持；问题被记录为 `QualityIssue` 并可追溯到 `stage_id` + `claim_id` |
| **非目标** | 不评判 claim 在工程上是否"正确"；不接入真实 LLM 改写 claim（保持 MockProvider / 确定性生成） |

### RQ-005 ViewGraph 可解释性评估

| 维度 | 说明 |
|------|------|
| **目标** | 在真实样本上评估 Phase 4 三类视图（结构 / 数据流 / 时序流水）的可解释性：节点/边 `trace_refs` 是否准确回链、是否出现"孤立方块"或错连、视图在真实噪声下是否仍可读 |
| **输入** | `ViewGraph`（`ViewNode[]`、`ViewEdge[]`、`ViewTraceRef[]`） |
| **输出** | `ViewQualityReport`：节点/边回链可解析比例、孤立节点数、错连嫌疑、`empty_or_unhelpful_view` 记录 |
| **验收标准** | 节点/边的 `trace_refs` 在真实样本上可解析回 claim/evidence；退化视图（孤立方块、空图）被记录为 `QualityIssue` 并可追溯到 `stage_id` + `node_id` |
| **非目标** | 不做自动布局引擎、不引入图形库；不重做 Phase 8 工作台级视图重构 |

### RQ-006 Grounded Q&A 可用性评估

| 维度 | 说明 |
|------|------|
| **目标** | 在仍使用 MockProvider 的前提下，刻画 Q&A 在真实样本上的表现：对有证据支持问题的命中率、对无证据问题是否诚实返回 `unknown`/`evidence_gap`、`citations` 是否真实有效，产出"基线缺口清单"供 Phase 9 真实 LLM |
| **输入** | `GroundedAnswer`（`text`、`claims`、`citations`、`confidence`、`warnings`、`is_degraded`） |
| **输出** | `QaQualityReport`：citation 有效比例、`qa_unanswered_when_evidence_exists` / `qa_answer_without_valid_citation` 记录、unknown/gap 表现 |
| **验收标准** | 有证据的问题不应当静默返回无解；回答的 citation 必须指向真实存在的 evidence/claim；问题被记录为 `QualityIssue` 并可追溯到 `stage_id` + `claim_id`/`evidence_id` |
| **非目标** | 不接入真实 LLM；不评判回答"对错"；Q&A 行为刻画用于暴露 MockProvider 边界，作为 Phase 9 输入 |

### RQ-007 质量问题记录与分类

| 维度 | 说明 |
|------|------|
| **目标** | 把 Phase 7 评估中发现的工具理解质量问题统一记录为 `QualityIssue`，按 `QualityIssueKind` 分类、按 `QualitySeverity` 分级，全部可追溯到 `stage_id` + artifact kind + 可选 `evidence_id` / `claim_id` / `node_id` |
| **输入** | 各 evaluator 的检查结果 + 人工桌面验收发现 |
| **输出** | `QualityIssue[]` 与 `QualityRunSummary`，形成 Phase 7 质量补强 backlog 的输入 |
| **验收标准** | 每条 issue 可追溯（`stage_id` + artifact kind + 可选 `evidence_id`/`claim_id`/`node_id`/`source_path`/`line_range`）、可分类、可分级；issue 描述的是"工具理解质量问题"，不描述"目标项目正确/错误"；正向 guardrail（如 `hallucinated_claim_blocked`）不计入负向 backlog |
| **非目标** | 不输出 PASS/HOLD/正确性裁决/审计结论；不评价目标项目 |

### RQ-008 Phase 7 质量补强退出标准

| 维度 | 说明 |
|------|------|
| **目标** | 定义 Phase 7 何时可视为质量补强完成：量化门槛达成、质量问题 backlog 闭环（已修复或显式接受为已知限制）、真实桌面验收在真实样本上通过、completion review 转 active |
| **输入** | `QualityRunSummary`、`QualityAcceptanceStatus`、桌面验收结果 |
| **输出** | Phase 7 完成判定与允许进入 Phase 8 / Phase 9 的条件 |
| **验收标准** | 见 §7 与验证文档 `phase-7-real-project-quality-validation.md` |
| **非目标** | 不承诺根除所有质量问题（理解质量有固有的主观性，遗留项以已知限制形式记录） |

## 4. 真实项目样本集要求

RQ-001 的样本集至少应覆盖以下形态，确保评估覆盖真实项目的系统性差距，而非只在简单样例上自洽：

| 覆盖维度 | 最低要求 |
|----------|----------|
| 项目形态 | 至少 1 个**完整** `ai_project_template` 生成项目（非 `/tmp` 手工片段） |
| 阶段类型 | 至少覆盖 `L0` / `L1` / `RTL` 三类阶段（业务项目实现阶段） |
| 文件语言类型 | 至少覆盖 Python、Verilog/SystemVerilog、以及 Markdown/doc/config/test 类文件中的若干类型（对应 `SourceKind`：`python_stage` / `rtl` / `test` / `doc` / `config` / `external_module`） |
| 边界场景（空/缺失） | 至少 1 个空阶段（`empty`）或缺失阶段（`missing`）场景 |
| 边界场景（命名异常） | 至少 1 个命名异常但可识别阶段（`naming_anomaly`，如 `rtl_final` 等非标准 `L0..RTL` 命名）场景 |
| 样本数量 | 至少 2 个真实样本或等价本地只读样本（验证文档要求） |

> 样本项目必须是**只读输入**：Phase 7 不修改样本项目，验收前后 checksum 一致。

## 5. 异常 / 空状态

| 场景 | 处理 |
|------|------|
| 真实样本路径不存在 | 评估流程拒绝该样本，提示重新选择，不产生质量报告 |
| 真实样本路径为 symlink | 安全拒绝，记录为不可评估，不读取 |
| 阶段为空（`empty`） | 不产生强证据缺口误报；空阶段被如实记录为空，对应 `QualityIssue` 仅在工具对空状态处理不当时产生 |
| 阶段缺失（`missing`） | 如实记录缺失，不伪造证据；评估其对依赖该阶段的下游视图/Q&A 的影响 |
| 命名异常阶段（`naming_anomaly`） | 应能被识别并参与评估；若被误判为 `missing`，记录为 `QualityIssue` |
| evidence 收集产生大量 warnings | warnings 进入 `EvidenceQualityReport`，可疑项降级处理而非静默丢弃 |
| 视图退化为孤立方块/空图 | 记录为 `empty_or_unhelpful_view`，不掩盖 |
| Q&A 对有证据问题返回无解 | 记录为 `qa_unanswered_when_evidence_exists` |

## 6. 证据与追溯要求

- 评估产物中的每条 `QualityIssue` 必须可追溯到 `stage_id`、artifact kind，并尽可能带上 `evidence_id` / `claim_id` / `node_id`，以及 `missing_evidence` / `noisy_evidence` / `wrong_source_kind` / source excerpt 相关问题所需的 `source_path` / `line_range`。
- 仅负向问题（`polarity=problem`）进入补强 backlog；正向 guardrail 记录（如 `hallucinated_claim_blocked`）不计入 backlog、不参与门槛判定。
- 评估过程不得伪造或修改既有 `evidence_id` / `claim_id` / `source_path` / `line_range` 绑定。
- `QualityReport` 本身是 Phase 7 质量评估产物，**不是用户业务项目的审计结论**；它描述的是"工具理解得怎么样"，不描述"目标项目对不对"。
- 质量评分（若出现）只用于内部质量门槛，不对目标项目做评价。

## 7. Phase 7 质量补强退出标准

- RQ-001~RQ-008 全部定义并实现对应 evaluator / 报告。
- 真实样本（至少 2 个）完成评估，量化门槛达成（具体阈值在设计与验证文档中收敛）。
- `QualityIssue` backlog 闭环：已修复或显式接受为已知限制并记录。
- 真实桌面验收在真实样本上通过，目标项目 checksum 验收前后一致。
- 全量 `npm run build` / `cargo test --lib` / `cargo check` 通过。
- Phase 7 completion review 转 `active`，并明确进入 Phase 8 / Phase 9 的条件。

## 8. 非目标

Phase 7 明确**不做**：

- **不接入真实 LLM**（属于 Phase 9）；Q&A 仍由 MockProvider 承担，Phase 7 用真实样本暴露其能力边界。
- **不做跨阶段 Python-to-RTL 映射**（属于 Phase 10）；仍聚焦单阶段理解质量。
- **不做 UI 大重构 / 工作台化**（属于 Phase 8）；UI 仅限为评估必要的最小质量展示。
- **不做 PASS/HOLD / 正确性裁决 / 审计结论**；不评价目标项目"正确/错误"。
- **不运行 Vivado / synthesis / implementation / bitstream**，不运行目标项目脚本。
- **不修改目标项目**，不写回目标项目目录，不外发任何内容。
- **不引入新的外部依赖**（无新 LLM SDK、无网络库、无图形库）。
- **不破坏核心语义契约**（`ImplementationUnderstanding`、`confidence` 枚举、evidence model 字段语义保持稳定；必要时扩展但不破坏 `mvp-functional-contract.md` 既有契约与持久化兼容性）。
- 不做多阶段语义记忆 / agent-scope 联动（属于 Phase 11）。

## 9. 安全边界

- 目标项目只读：不修改样本项目源码，验收前后 checksum 一致。
- 不运行 Vivado / synthesis / implementation / bitstream。
- 不调用真实 LLM（不读取 `api_key`、不调用 OpenAI / Anthropic 或任何外部模型服务）。
- 持久化只写 app-owned storage；评估产物不写回目标项目。
- 不外发任何样本内容；不保存敏感环境变量。

## 10. 关联设计文档

- [`../design/phase-7-real-project-evaluation-model.md`](../design/phase-7-real-project-evaluation-model.md) — 质量评估数据模型
- [`../design/phase-7-evidence-understanding-quality-design.md`](../design/phase-7-evidence-understanding-quality-design.md) — evidence/understanding 质量评估与补强设计
- [`../ui-ux/phase-7-quality-review-view.md`](../ui-ux/phase-7-quality-review-view.md) — Quality Review 视图设计
- [`../testing/phase-7-real-project-quality-validation.md`](../testing/phase-7-real-project-quality-validation.md) — 验证与验收
- [`../planning/phase-7-implementation-plan.md`](../planning/phase-7-implementation-plan.md) — 编码实施计划
- [`../planning/phase-7-overview-real-project-quality.md`](../planning/phase-7-overview-real-project-quality.md) — Phase 7 overview（draft）
- [`../planning/post-mvp-roadmap.md`](../planning/post-mvp-roadmap.md) — Post-MVP 总体路线图（draft）

## 11. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-15 | 初始 draft：定义 RQ-001~RQ-008、真实项目样本覆盖要求、非目标、安全边界、退出标准。Batch A/B 后续已实现，当前进入审核收口。 | Claude |
| 2026-06-15 | 审核收口修复（status 保持 draft）：RQ-002 阶段识别误判改用 `stage_identification_mismatch`；RQ-007/§6 追溯字段补 `source_path`/`line_range`，并明确正向 guardrail 不计入 backlog。Batch A/B 后续已实现，当前进入审核收口。 | Claude |
| 2026-06-15 | 审核通过，status 从 draft 转为 active，作为 Phase 7 编码依据；Phase 7 Batch A/B 已实现并进入审核收口，Batch C 未授权。 | Claude |
