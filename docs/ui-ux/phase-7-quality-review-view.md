# Phase 7 Quality Review 视图设计

---
status: active
updated: 2026-06-15
---

> 本文档定义 Phase 7 的 UI/UX 范围：**只做质量评估视图，不做整体工作台重构**。
>
> Phase 7 的 UI 目标是让用户/评估者看到"工具对自身分析质量的诚实自评"，而不是重构整个界面。工作台级 UI 重构属于 Phase 8，不在 Phase 7 范围。
>
> 所有 UI 文案表达"工具理解质量"与"不确定性"，**不评价目标项目**，避免"正确/错误""PASS/HOLD""审计结论"等用语。
>
> 本文档 status 为 `active`，是 Phase 7 前端编码依据（Phase 7 UI 属 Batch C，Batch A/B 不含 UI）。**Phase 7 已全部完成（Batch A/B/C/D，completion review 已 active）；Phase 8/9/10/11 尚未开始**。

## 1. 设计目标

1. **质量可见**：让用户看到每个阶段的 evidence/understanding/view/Q&A 质量概要与缺口，而不是把分析能力当黑盒。
2. **不确定性可见**：显式呈现覆盖缺口、无证据 claim、退化视图、Q&A 失误，与既有 unknown/gap 视觉语义一致。
3. **最小改动**：在既有 Phase 1~6 面板基础上加最小质量提示，不重写布局。
4. **守住定位**：UI 表达"工具理解得怎么样"，不表达"目标项目对不对"。

## 2. 允许新增 / 调整的视图元素

Phase 7 UI 仅限以下最小集合，挂在既有布局之上：

| 元素 | 定位 | 内容 |
|------|------|------|
| Quality Review 面板 | 新增（可折叠） | 当前样本/阶段的 `QualityRunSummary` 概览、issue 数按维度/严重程度分布 |
| quality issue list | Quality Review 面板内 | `QualityIssue[]` 列表：kind、severity、stage_id、可追溯链接（evidence/claim/node）、状态 |
| stage quality summary | 阶段导航旁/阶段标题区 | 每阶段 4 维（evidence/understanding/view/qa）质量提示徽标（如覆盖率档位） |
| evidence 质量提示 | 既有 EvidencePanel 内（小幅） | 在证据项或概要区提示 `missing_evidence`/`noisy_evidence`/`wrong_source_kind` |
| understanding 质量提示 | 既有 UnderstandingPanel 内（小幅） | 在 claim 行提示 `unsupported_claim`/`hallucinated_claim_blocked`、在 summary 区提示 `weak_summary` |
| view 质量提示 | 既有 MultiViewPanel 内（小幅） | 在视图区提示 `empty_or_unhelpful_view`、孤立节点标记 |
| Q&A 质量提示 | 既有 GroundedQAPanel 内（小幅） | 在回答区提示 `qa_answer_without_valid_citation`/`qa_unanswered_when_evidence_exists` |
| 真实项目验收清单视图 | 新增（仅评估/验收场景） | 桌面验收 checklist 展示与勾选状态（验收用，非日常主界面） |

> "小幅质量提示"指：在既有面板已有信息旁加一行提示或一个徽标，不改变面板主体结构与交互。

## 3. 明确禁止（不属于 Phase 7）

- **不重写整体布局为新工作台**——整体信息架构/导航重构属于 Phase 8。
- **不引入复杂图形库**——沿用既有 SVG + CSS 方案，不引入 React Flow / D3 / Mermaid。
- **不做 Phase 8 的导航/信息架构重构**。
- **不做工作台级 dashboard**——Quality Review 是面板，不是全屏 dashboard。
- **不在 UI 中输出 PASS/HOLD/正确性裁决/审计结论**。

## 4. 视觉语义（复用既有 + 最小扩展）

### 4.1 复用既有置信度/不确定性视觉语义

- 继续沿用 Phase 3/4/5 已有的 confidence 颜色映射（confirmed/supported/inferred/unknown/conflicting）与 unknown/gap 视觉表达，保持一致性。

### 4.2 质量提示视觉（最小扩展）

| 提示类型 | 视觉处理（建议，最终以实现为准） |
|----------|----------------------------------|
| 缺口/问题提示（`missing_evidence` 等） | 中性提示色 + 图标，非"错误/报警"红 |
| 严重程度 | `High` 加粗强调、`Medium` 普通、`Low` 弱化；不使用 PASS/HOLD 风格的红绿裁决色 |
| 正向记录（`hallucinated_claim_blocked`） | 中性"守卫生效"标记，非通过/失败语义 |

> 颜色只表达"质量提示强度"与"不确定性"，不表达"目标项目通过/失败"。

## 5. 交互状态

| 状态 | 处理 |
|------|------|
| 无评估数据（未跑评估） | Quality Review 面板显示"尚未运行质量评估"空状态，不报错 |
| 评估运行中 | 显示加载态，不阻塞既有 Phase 1~6 主链路 |
| 评估完成 | 显示 summary + issue list |
| issue 可追溯链接点击 | 复用 Phase 5 trace/EvidencePanel 高亮机制，定位到对应 evidence/claim/node |
| 真实项目验收清单 | 勾选状态仅本地/评估用，不持久化为"目标项目结论" |

## 6. 文案规范

- **禁用**："正确""错误""PASS""HOLD""审计结论""通过/失败裁决"等用语。
- **使用**：
  - "工具在该阶段未覆盖的证据：…"
  - "此声明缺少引用真实证据"
  - "工具未能基于已有证据回答此问题"
  - "该视图退化为孤立节点，可解释性低"
  - "覆盖率：x%（内部质量指标，不代表目标项目质量）"
- 任何带评分/比例的文案必须标注其为**内部质量指标**，不代表目标项目质量。

## 7. 前后端边界

- Quality Review 面板数据来自 Phase 7 后端评估产物（`QualityReport`/`QualityIssue`/`QualityRunSummary`），通过新增 Tauri command（计划中，见实施计划 P7-T06）读取。该 command 属 Batch C（最小 UI），**不属 Batch A/B**（Batch A/B 含 P7-T01~P7-T05 模型、reporter 与后端 evaluators），避免越界。
- 前端只读展示评估产物，不重新计算质量结论；主观维度不在前端自动裁决。
- TypeScript 类型从评估模型派生（与 `phase-7-real-project-evaluation-model.md` 对齐）。

## 8. 安全边界

- UI 不触发对目标项目的写入；所有可追溯点击只读定位。
- 不调用真实 LLM；不读取 `api_key`。
- 不运行 Vivado / synthesis / implementation / bitstream。
- 文案不输出审计结论。

## 9. 关联文档

- [`../design/phase-7-real-project-evaluation-model.md`](../design/phase-7-real-project-evaluation-model.md) — 评估数据模型
- [`../design/phase-7-evidence-understanding-quality-design.md`](../design/phase-7-evidence-understanding-quality-design.md) — 评估与补强设计
- [`../requirements/phase-7-real-project-quality-requirements.md`](../requirements/phase-7-real-project-quality-requirements.md) — 需求
- [`../testing/phase-7-real-project-quality-validation.md`](../testing/phase-7-real-project-quality-validation.md) — 验证与验收
- [`phase-5-trace-and-qa-view.md`](phase-5-trace-and-qa-view.md) — 既有 trace/Q&A 视图（复用其高亮机制）
- [`phase-4-multi-view-panel.md`](phase-4-multi-view-panel.md) — 既有三视图面板（在其内加最小质量提示）

## 10. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-15 | 初始 draft：定义 Phase 7 仅做质量评估视图、允许/禁止项、视觉语义、文案规范、前后端边界。明确不做工作台重构。Batch A/B 后续已实现，当前进入审核收口。 | Claude |
| 2026-06-15 | 审核收口修复（status 保持 draft）：修正 §7 Tauri command 归属，指向 P7-T06 并明确不属 Batch A，避免越界。Batch A/B 后续已实现，当前进入审核收口。 | Claude |
| 2026-06-15 | 审核通过，status 从 draft 转为 active，作为 Phase 7 编码依据；Phase 7 Batch A/B 已实现并进入审核收口，Batch C 未授权。 | Claude |
| 2026-06-15 | Batch C 实现：`QualityReviewPanel` 完成加载/空/报错/报告态、汇总、分维度概览、可点击 issue 列表；`WorkspacePage` 接入状态机并支持重新收集/切换/生成视图/Q&A 时过期质量报告；文案使用"达到/低于当前质量门槛"，禁用 PASS/HOLD。 | Claude |
