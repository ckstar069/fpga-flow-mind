# 文档体系总索引

本文档是 `fpga-flow-mind` 正式文档体系的入口。所有后续需求、设计、计划、测试文档应在此体系内有序存放，避免信息碎片化。

## 文档体系说明

```text
docs/
├── README.md                        ← 本文档（总索引）
├── requirements/
│   ├── README.md                    ← 需求文档索引
│   └── stories/
│       └── README.md                ← Story 拆分索引
├── ui-ux/
│   └── README.md                    ← UI/UX 设计索引
├── design/
│   └── README.md                    ← 技术设计索引
├── planning/
│   └── README.md                    ← 实施计划索引
├── testing/
│   └── README.md                    ← 测试与验收索引
└── initial-requirements-draft/      ← 初始需求草案（归档参考）
    ├── PROJECT_BRIEF.md
    ├── MVP_ARCHITECTURE.md
    └── DEVELOPMENT_WORKFLOW.md
```

## 各目录用途

| 目录 | 用途 | 读者 |
|------|------|------|
| `requirements/` | 产品范围、用户故事、MVP 需求、验收标准、非目标 | 产品经理、设计师、实施者、审核者 |
| `requirements/stories/` | 按 story 拆分的需求明细，每个 story 聚焦一个用户目标 | 实施者、设计师 |
| `ui-ux/` | 用户流程、信息架构、视图定义、交互状态、视觉语义 | 设计师、前端开发者 |
| `design/` | 数据契约、Agent 工作流、证据模型、可视化规范、持久化设计、架构决策 | 后端开发者、架构师 |
| `planning/` | 路线图、里程碑、阶段清单、发布标准 | 项目负责人、实施者 |
| `testing/` | 验证策略、MVP 验收标准、手工 QA 清单、安全回归清单 | QA、实施者、审核者 |
| `initial-requirements-draft/` | 初始草案归档，当前仍是重要约束来源 | 所有角色 |

## 推荐阅读路径

### 新任务 / 接手项目时

1. 项目根目录 `README.md`
2. `AGENTS.md`
3. `docs/README.md`（本文档）
4. `docs/initial-requirements-draft/PROJECT_BRIEF.md`
5. 根据任务类型，继续读对应子目录索引

### 需求任务

1. `docs/requirements/README.md`
2. `docs/initial-requirements-draft/PROJECT_BRIEF.md`
3. `docs/requirements/stories/README.md`
4. 相关 story 文件

### UI/UX 任务

1. `docs/ui-ux/README.md`
2. `docs/requirements/README.md`
3. 相关 story 文件（了解用户目标）
4. `docs/design/README.md`（了解技术约束和数据契约）
5. `docs/ui-ux/phase-4-multi-view-panel.md`（Phase 4 三视图面板设计，active）
6. `docs/ui-ux/phase-5-trace-and-qa-view.md`（Phase 5 证据回链与 Grounded Q&A 视图设计，active）
7. `docs/ui-ux/phase-6-session-and-mvp-view.md`（Phase 6 Session 管理与 MVP 验收 UI/UX 设计，active）

### 架构 / 技术设计任务

1. `docs/design/README.md`
2. `docs/requirements/README.md`
3. `docs/initial-requirements-draft/MVP_ARCHITECTURE.md`
4. 相关 story 文件
5. `docs/design/phase-4-view-model.md`（Phase 4 视图数据模型，active）
6. `docs/design/phase-4-view-generator-design.md`（Phase 4 视图生成器设计，active）
7. `docs/design/phase-5-trace-model.md`（Phase 5 证据回链与 Grounded Q&A 数据模型，active）
8. `docs/design/phase-5-trace-and-qa-design.md`（Phase 5 证据回链与 Grounded Q&A 后端设计，active）
9. `docs/design/phase-6-persistence-model.md`（Phase 6 持久化数据模型，active）
10. `docs/design/phase-6-persistence-and-replay-design.md`（Phase 6 持久化与回放后端设计，active）

### 实施计划任务

1. `docs/planning/README.md`
2. `docs/planning/phase-1-documents-closure.md`（Phase 1 文档收口说明）
3. `docs/planning/phase-1-implementation-plan.md`（Phase 1 编码实施计划）
4. `docs/planning/phase-2-implementation-plan.md`（Phase 2 编码实施计划）
5. `docs/planning/phase-0-exit-criteria.md`（确认当前阶段退出标准）
6. `docs/planning/phase-3-completion-review.md`（Phase 3 完成状态）
7. `docs/planning/phase-4-implementation-plan.md`（Phase 4 编码实施计划，active）
8. `docs/planning/phase-4-completion-review.md`（Phase 4 完成状态，active）
9. `docs/planning/phase-5-implementation-plan.md`（Phase 5 编码实施计划，active）
10. `docs/planning/phase-5-completion-review.md`（Phase 5 完成状态，active）
11. `docs/planning/phase-6-implementation-plan.md`（Phase 6 编码实施计划，active）
12. `docs/design/README.md`（设计文档索引）
13. `docs/requirements/README.md`
14. 当前阶段应完成的 story 列表

### Post-MVP 路线图 / 后续阶段规划任务

> MVP（Phase 0–6 / tag `v0.1.0-mvp`）是技术闭环 MVP，不等于产品可用性完成。下列 Post-MVP 文档中，**Phase 7 详细文档已 `active`**；**Phase 7 已全部完成（Batch A/B/C/D，[completion review](planning/phase-7-completion-review.md) active）**；Phase 7 overview 与 Phase 9~11 overview 仍为 `draft`。**Phase 8 详细文档均已 `active`；Phase 8 Batch A（P8-T01~P8-T02）、Batch B（P8-T03~P8-T04）与 Batch C（P8-T05~P8-T06）已实现/进入审核收口；Batch D/E 与 Phase 9/10/11 未开始。**

1. `docs/planning/post-mvp-roadmap.md`（Post-MVP 总体路线图：Phase 7~11 阶段关系、依赖顺序、进入纪律，draft）
2. `docs/planning/phase-7-overview-real-project-quality.md`（Phase 7 overview：真实项目质量补强，draft）
   - Phase 7 详细文档（**均已 active；Phase 7 全部完成（[completion review](planning/phase-7-completion-review.md) active）**）：
     - `docs/requirements/phase-7-real-project-quality-requirements.md`（需求 RQ-001~RQ-008，active）
     - `docs/design/phase-7-real-project-evaluation-model.md`（评估数据模型，active）
     - `docs/design/phase-7-evidence-understanding-quality-design.md`（评估与补强设计，active）
     - `docs/ui-ux/phase-7-quality-review-view.md`（Quality Review 视图，active）
     - `docs/testing/phase-7-real-project-quality-validation.md`（验证与验收，active）
     - `docs/planning/phase-7-implementation-plan.md`（编码实施计划，active）
3. `docs/planning/phase-8-overview-product-ui-workbench.md`（Phase 8 overview：产品级 UI 工作台重构，draft）
   - Phase 8 详细文档（**均已 `active`；Phase 8 Batch A（P8-T01~P8-T02）、Batch B（P8-T03~P8-T04）与 Batch C（P8-T05~P8-T06）已实现/进入审核收口；Batch D/E 与 Phase 9/10/11 未开始**）：
     - `docs/requirements/phase-8-product-workbench-requirements.md`（需求 R8-001~R8-010，active）
     - `docs/design/phase-8-workbench-architecture.md`（工作台架构，active）
     - `docs/design/phase-8-ui-state-and-navigation-design.md`（UI 状态与导航，active）
     - `docs/ui-ux/phase-8-product-workbench-view.md`（工作台 UI/UX，active）
     - `docs/testing/phase-8-product-workbench-validation.md`（验证与验收，active）
     - `docs/planning/phase-8-implementation-plan.md`（编码实施计划，active）
4. `docs/planning/phase-9-overview-real-llm-grounding.md`（Phase 9 overview：真实 LLM 与 grounding 生产化，draft）
5. `docs/planning/phase-10-overview-cross-stage-python-rtl.md`（Phase 10 overview：跨阶段与 Python-to-RTL 映射，draft）
6. `docs/planning/phase-11-overview-semantic-memory-and-integration.md`（Phase 11 overview：语义记忆、测试覆盖图与 agent-scope 联动，draft）

### 测试 / 验收任务

1. `docs/testing/README.md`
2. `docs/testing/phase-1-workspace-scanning-validation.md`（Phase 1 验证设计与验收标准）
3. `docs/testing/phase-2-evidence-validation.md`（Phase 2 验证设计与验收标准）
4. `docs/planning/phase-1-completion-review.md`（Phase 1 收尾验收与完成审查）
5. `docs/planning/phase-2-completion-review.md`（Phase 2 收尾验收与完成审查）
6. `docs/planning/phase-3-completion-review.md`（Phase 3 收尾验收与完成审查）
7. `docs/testing/phase-3-understanding-validation.md`（Phase 3 验证设计与验收标准）
8. `docs/testing/phase-4-view-validation.md`（Phase 4 视图验证设计与验收标准，active）
9. `docs/planning/phase-4-completion-review.md`（Phase 4 收尾验收与完成审查，active）
10. `docs/testing/phase-5-trace-and-qa-validation.md`（Phase 5 证据回链与 Grounded Q&A 验证设计，active）
11. `docs/testing/phase-6-mvp-validation.md`（Phase 6 持久化与 MVP 总体验收验证设计，active）
12. `docs/planning/phase-6-completion-review.md`（Phase 6 收尾验收与完成审查，active）
13. `docs/planning/mvp-release-notes.md`（MVP Release Notes，Phase 0–6 completion 发布说明，active）
14. `docs/requirements/README.md`（验收标准来源）
15. `docs/planning/README.md`（当前阶段验收要求）
16. 相关 story 文件中的验收标准

### 审核任务

1. `AGENTS.md`
2. `docs/README.md`
3. `docs/planning/README.md`（确认当前阶段目标）
4. `docs/requirements/README.md`（确认需求边界）
5. `docs/design/README.md`（确认架构边界）
6. 被审核文档本身

---

## 文档状态约定

每个文档文件应在开头注明状态，使用以下标签之一：

| 状态 | 含义 | 处理规则 |
|------|------|----------|
| `draft` | 草案，尚未定稿 | 可读、可评论、可修改，但不应作为实施唯一依据 |
| `active` | 当前生效文档 | 实施和审核应以此为依据 |
| `superseded` | 已被新文档取代 | 保留供追溯，但不应继续引用；应在顶部注明替代文档 |
| `archived` | 归档，仅作历史参考 | 不再更新，如 `initial-requirements-draft/` 整体属于归档 |

状态标注示例（放在文档 frontmatter 或标题下方）：

```markdown
---
status: active
updated: 2026-06-11
---
```

## 文档拆分规则

- **复杂需求**应按 story、模块或能力拆分为多个文档，不把所有内容堆进一个大文档。
- **每个 story**可以包含多个功能点，但应围绕一个明确的用户目标展开。
- **单个文档**应聚焦一个明确主题，便于定位、修改和版本控制。
- **当文档超过 200 行**或覆盖多个主题时，应考虑拆分。

## 章节级索引规则

- **每个子目录 README**应索引该目录下的所有文档。
- **索引应说明**：每个文档解决什么问题、当前状态是什么、什么角色应在何时阅读。
- **避免只列文件名**，应附上一句话说明文档价值。

## `initial-requirements-draft/` 定位

- 该目录是**初始草案归档**，其内容在正式文档建立前是项目的重要约束来源。
- **后续正式文档应从中提炼**，而不是直接丢弃或原样搬运。
- 当正式文档与草案冲突时，以正式文档为准，但应在变更记录中说明理由。
- 当前该目录下的文件状态统一视为 `archived`，但内容仍具参考价值。
