# Phase 7 真实项目质量评估数据模型

---
status: draft
updated: 2026-06-15
---

> 本文档定义 Phase 7（真实项目评估与 evidence/understanding 质量补强）的**质量评估数据模型**：用于在真实 `ai_project_template` 样本上刻画工具自身分析质量的产物结构。
>
> **关键定位**：本文档定义的所有对象都是 **Phase 7 质量评估产物**，描述"工具理解得怎么样"；**它们不是用户业务项目的审计结论**，不描述"目标项目正确/错误"。质量评分（若出现）只用于内部质量门槛，不对目标项目做评价。
>
> 本文档 status 为 `draft`，待审核通过后转为 `active`。Phase 7 当前**未开始编码**，本文模型为计划中的数据契约，实现以 P7-T01~P7-T05 为准。
>
> 既有类型引用（保持稳定，不重定义）：`EvidenceItem`/`EvidenceCollection`/`EvidenceStrength`、`ImplementationUnderstanding`/`ImplementationClaim`/`ClaimConfidence`/`UnknownItem`/`EvidenceGap`、`ViewGraph`/`ViewNode`/`ViewEdge`/`ViewTraceRef`、`GroundedAnswer`/`GroundedAnswerCitation`，以及 `StageStatus`/`SourceKind`/`Language`。

## 1. 设计原则

### 1.1 评估产物，非审计结论

- 所有 `QualityReport` / `QualityIssue` 描述的都是**工具自身的理解质量问题**（如"这条 claim 没有引用真实证据"），**不是**目标项目本身的正确性判断。
- issue 文案禁止使用"正确/错误""PASS/HOLD""审计结论"等用语；只描述"工具理解质量"与"不确定性"。

### 1.2 可追溯

- 每条 `QualityIssue` 必须可追溯到：
  - `stage_id`（必填，业务项目阶段）；
  - `artifact_kind`（必填，被评估产物类型）；
  - 可选 `evidence_id` / `claim_id` / `node_id`（指向具体证据/声明/视图节点）。
- 评估过程不伪造、不修改既有 `evidence_id` / `claim_id` / `source_path` / `line_range` 绑定。

### 1.3 评分仅用于内部门槛

- 质量评分（如覆盖率、命中率、回链可解析率）只用于 Phase 7 内部质量门槛判定，**不对目标项目做评价**，不对外输出 PASS/HOLD。

### 1.4 契约稳定

- 既有 evidence/understanding/view/qa 模型字段语义保持稳定；本文档新增的是**评估层**对象，挂在既有产物之上，不改写既有契约（必要时按 §6 的扩展原则，不破坏 `mvp-functional-contract.md` 与持久化兼容性）。

## 2. 模型总览

```text
RealProjectSample                     # 评估语料登记
  └─ StageEvaluationTarget[]          # 逐阶段评估目标
        ├─ EvidenceQualityReport      # evidence 覆盖率与缺口
        ├─ UnderstandingQualityReport # understanding claim/unknown 质量
        ├─ ViewQualityReport          # 视图可解释性
        └─ QaQualityReport            # Q&A 可用性
              └─ QualityIssue[]       # 统一质量问题记录（带 kind/severity/trace）

QualityRunSummary                     # 一次评估运行的汇总
QualityAcceptanceStatus               # Phase 7 门槛判定
```

## 3. 真实项目样本登记

### 3.1 `RealProjectSample`

```rust
/// 一个被纳入 Phase 7 评估的真实（或等价本地只读）样本登记记录。
///
/// 仅登记"评估输入是什么"，不携带任何对样本项目正确性的判断。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealProjectSample {
    /// 样本唯一标识，如 "sample-urban-ofdm-001"
    pub sample_id: String,
    /// 样本项目根路径（只读输入）
    pub root_path: String,
    /// 来源描述（如 "ai_project_template 生成项目" 或 "等价本地只读副本"）
    pub source_description: String,
    /// 阶段清单（人工登记的 ground-truth 阶段构成，用于比对识别准确性）
    pub expected_stages: Vec<ExpectedStageEntry>,
    /// 文件类型分布（语言/source_kind 计数，用于覆盖率分母）
    pub file_type_distribution: FileTypeDistribution,
    /// 规模度量（阶段数、文件数、总行数等）
    pub scale_metrics: SampleScaleMetrics,
    /// 特性标签：empty_stage / missing_stage / naming_anomaly / multi_language 等
    pub trait_tags: Vec<String>,
    /// 登记时间（ISO8601，由调用方传入，评估运行不自行取系统时间）
    pub registered_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedStageEntry {
    pub stage_id: String,
    /// 人工期望的 StageStatus：available / empty / missing / naming_anomaly
    pub expected_status: String,
    pub expected_languages: Vec<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeDistribution {
    pub by_language: std::collections::HashMap<String, u32>,
    pub by_source_kind: std::collections::HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleScaleMetrics {
    pub stage_count: u32,
    pub file_count: u32,
    pub total_lines: u32,
}
```

### 3.2 `StageEvaluationTarget`

```rust
/// 对单个阶段执行质量评估的目标对象。
///
/// 绑定该阶段的既有 Phase 1~6 产物（只读引用），评估结果写入对应 QualityReport。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageEvaluationTarget {
    pub sample_id: String,
    pub stage_id: String,
    /// 实际识别到的 StageStatus（来自 WorkspaceProfile）
    pub recognized_status: String,
    /// 该阶段的 EvidenceCollection（Phase 2 产物）
    pub evidence_collection_present: bool,
    /// 该阶段的 ImplementationUnderstanding（Phase 3 产物）
    pub understanding_present: bool,
    /// 该阶段的 ViewGraph 数量与类型（Phase 4 产物）
    pub view_graph_types: Vec<String>,
    /// 该阶段是否产生过 Q&A（Phase 5 产物）
    pub qa_history_present: bool,
}
```

## 4. 分维度质量报告

### 4.1 `EvidenceQualityReport`

```rust
/// evidence 覆盖率与缺口评估结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceQualityReport {
    pub sample_id: String,
    pub stage_id: String,
    /// 被覆盖的源文件数 / 该阶段应覆盖源文件数（内部门槛指标，不评价目标项目）
    pub file_coverage_ratio: f32,
    /// line_range 准确性比例（落在真实行范围内的比例）
    pub line_range_accuracy: f32,
    /// strength / source_kind / language 标注合理性比例
    pub label_sanity_ratio: f32,
    /// 未覆盖文件及原因
    pub uncovered_files: Vec<UncoveredFile>,
    /// 该阶段证据相关 issue 的引用（kind ∈ missing_evidence/noisy_evidence/wrong_source_kind）
    pub issue_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncoveredFile {
    pub source_path: String,
    pub reason: String,
}
```

### 4.2 `UnderstandingQualityReport`

```rust
/// ImplementationUnderstanding 质量评估结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnderstandingQualityReport {
    pub sample_id: String,
    pub stage_id: String,
    /// claim 通过 existence check 的比例（evidence_id 真实存在）
    pub claim_existence_check_ratio: f32,
    /// unknown / evidence_gap 表达合理性比例（证据不足处被诚实表达）
    pub uncertainty_expression_ratio: f32,
    /// claim 中 confidence 标注合理性比例（与 supporting evidence 是否一致）
    pub confidence_calibration_ratio: f32,
    /// StageSummary 质量评估（weak_summary 计数等）
    pub summary_quality: SummaryQuality,
    pub issue_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryQuality {
    pub total_summaries: u32,
    pub weak_summary_count: u32,
}
```

### 4.3 `ViewQualityReport`

```rust
/// 三类视图可解释性评估结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewQualityReport {
    pub sample_id: String,
    pub stage_id: String,
    pub view_type: String, // structure / dataflow / timing
    /// 节点/边 trace_refs 可解析回 claim/evidence 的比例
    pub trace_resolvable_ratio: f32,
    /// 孤立节点数（无连边）
    pub isolated_node_count: u32,
    /// 错连嫌疑计数
    pub suspected_misconnection_count: u32,
    pub issue_refs: Vec<String>,
}
```

### 4.4 `QaQualityReport`

```rust
/// Grounded Q&A 可用性评估结果（基于 MockProvider）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QaQualityReport {
    pub sample_id: String,
    pub stage_id: String,
    /// citation 指向真实 evidence/claim 的比例
    pub citation_validity_ratio: f32,
    /// 对"有证据支持问题"的回答命中率
    pub answerable_hit_ratio: f32,
    /// 对"无证据问题"诚实返回 unknown/gap 的比例
    pub unknown_honesty_ratio: f32,
    pub issue_refs: Vec<String>,
}
```

## 5. 统一质量问题记录

### 5.1 `QualityIssue`

```rust
/// 一条工具理解质量问题记录。
///
/// 仅描述"工具理解质量"，不描述"目标项目正确/错误"。
/// 每条必须可追溯到 stage_id + artifact_kind + 可选 evidence_id/claim_id/node_id。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    /// issue 唯一标识
    pub issue_id: String,
    pub sample_id: String,
    pub stage_id: String,
    /// 被评估产物类型
    pub artifact_kind: ArtifactKind,
    /// 问题分类（见 QualityIssueKind）
    pub kind: QualityIssueKind,
    /// 严重程度（见 QualitySeverity）
    pub severity: QualitySeverity,
    /// 可选追溯到具体证据/声明/视图节点
    pub evidence_id: Option<String>,
    pub claim_id: Option<String>,
    pub node_id: Option<String>,
    /// 问题描述（客观、避免审计用语）
    pub description: String,
    /// 发现方式：automated / manual / desktop_acceptance
    pub detected_by: DetectionMethod,
    /// 处置状态：open / fixed / accepted_as_known_limitation
    pub status: IssueStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    Workspace,
    Stage,
    Evidence,
    Understanding,
    View,
    Qa,
    Ui,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionMethod {
    Automated,
    Manual,
    DesktopAcceptance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueStatus {
    Open,
    Fixed,
    AcceptedAsKnownLimitation,
}
```

### 5.2 `QualityIssueKind`

```rust
/// 工具理解质量问题分类。
///
/// 全部围绕"工具是否理解到位"，不涉及目标项目正确性。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityIssueKind {
    /// 应被覆盖的证据未被收集（文件/符号级缺口）
    MissingEvidence,
    /// 证据被收集但含噪声（TODO/注释块/实验代码被当主证据）
    NoisyEvidence,
    /// evidence 的 source_kind / language 标注与实际不符
    WrongSourceKind,
    /// StageSummary 过于空洞或未抓住阶段核心
    WeakSummary,
    /// claim 缺少 evidence_refs 或未通过 existence check
    UnsupportedClaim,
    /// 无证据 claim 被 hallucination guard 拦截（正向记录：守卫生效）
    HallucinatedClaimBlocked,
    /// 视图退化为孤立方块/空图/无信息
    EmptyOrUnhelpfulView,
    /// 有证据支持的问题，Q&A 未能给出回答
    QaUnansweredWhenEvidenceExists,
    /// Q&A 回答的 citation 指向不存在/不相关的 evidence
    QaAnswerWithoutValidCitation,
    /// UI 状态令人困惑（空状态/加载/降级提示不清）
    ConfusingUiState,
}
```

### 5.3 `QualitySeverity`

```rust
/// 问题严重程度（仅用于补强优先级排序，不用于目标项目评价）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualitySeverity {
    /// 轻微：不影响追溯，体验瑕疵
    Low,
    /// 中等：局部理解质量受损，但可追溯
    Medium,
    /// 重要：理解质量系统性受损或追溯链断裂
    High,
}
```

## 6. 运行汇总与门槛判定

### 6.1 `QualityRunSummary`

```rust
/// 一次 Phase 7 评估运行的汇总。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRunSummary {
    pub run_id: String,
    pub sample_ids: Vec<String>,
    pub total_issues: u32,
    pub issues_by_kind: std::collections::HashMap<String, u32>,
    pub issues_by_severity: std::collections::HashMap<String, u32>,
    pub issues_by_status: std::collections::HashMap<String, u32>,
    /// 各维度汇总指标（覆盖率/命中率等，内部门槛用）
    pub metric_snapshots: Vec<MetricSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshot {
    pub metric_name: String,
    pub stage_id: Option<String>,
    pub value: f32,
}
```

### 6.2 `QualityAcceptanceStatus`

```rust
/// Phase 7 质量门槛判定结果。
///
/// 仅表达"质量补强是否达到 Phase 7 退出门槛"，不输出 PASS/HOLD，不评价目标项目。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityAcceptanceStatus {
    /// 尚未达到门槛，仍需补强
    BelowGate,
    /// 达到门槛，满足 Phase 7 退出条件
    MeetsGate,
}
```

## 7. 持久化与版本

- 评估产物（`QualityReport` / `QualityIssue` / `QualityRunSummary`）作为 **Phase 7 系统内产物**，可持久化到 app-owned storage，**不写回目标项目**。
- 评估产物 schema 独立于既有 evidence/understanding/view/qa 模型，**不破坏既有持久化兼容性**。
- 若后续需扩展既有模型字段（如为 support 补强在 evidence 上加标注），必须遵循不破坏 `mvp-functional-contract.md` 的扩展原则，并在该阶段设计文档单独说明。

## 8. 安全边界

- 模型实现只读访问目标项目；`source_path` 仅用于追溯，不触发写入。
- 不调用真实 LLM（不读取 `api_key`、不调用 OpenAI / Anthropic）。
- 不运行 Vivado / synthesis / implementation / bitstream。
- issue 文案严禁出现"正确/错误""PASS/HOLD""审计结论"。

## 9. 关联文档

- [`../requirements/phase-7-real-project-quality-requirements.md`](../requirements/phase-7-real-project-quality-requirements.md) — Phase 7 需求（RQ-001~RQ-008）
- [`phase-7-evidence-understanding-quality-design.md`](phase-7-evidence-understanding-quality-design.md) — 评估与补强设计（如何用本模型）
- [`../ui-ux/phase-7-quality-review-view.md`](../ui-ux/phase-7-quality-review-view.md) — Quality Review 视图
- [`../testing/phase-7-real-project-quality-validation.md`](../testing/phase-7-real-project-quality-validation.md) — 验证与验收
- [`../planning/phase-7-implementation-plan.md`](../planning/phase-7-implementation-plan.md) — 编码实施计划

## 10. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-15 | 初始 draft：定义 RealProjectSample / StageEvaluationTarget / 4 类 QualityReport / QualityIssue(+Kind+Severity) / QualityRunSummary / QualityAcceptanceStatus。强调评估产物非审计结论、可追溯、评分仅内部门槛。Phase 7 未进入编码。 | Claude |
