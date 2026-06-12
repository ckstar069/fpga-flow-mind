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

### 架构 / 技术设计任务

1. `docs/design/README.md`
2. `docs/requirements/README.md`
3. `docs/initial-requirements-draft/MVP_ARCHITECTURE.md`
4. 相关 story 文件
5. `docs/design/phase-4-view-model.md`（Phase 4 视图数据模型，active）
6. `docs/design/phase-4-view-generator-design.md`（Phase 4 视图生成器设计，active）

### 实施计划任务

1. `docs/planning/README.md`
2. `docs/planning/phase-1-documents-closure.md`（Phase 1 文档收口说明）
3. `docs/planning/phase-1-implementation-plan.md`（Phase 1 编码实施计划）
4. `docs/planning/phase-2-implementation-plan.md`（Phase 2 编码实施计划）
5. `docs/planning/phase-0-exit-criteria.md`（确认当前阶段退出标准）
6. `docs/planning/phase-3-completion-review.md`（Phase 3 完成状态）
7. `docs/planning/phase-4-implementation-plan.md`（Phase 4 编码实施计划，active）
8. `docs/design/README.md`（设计文档索引）
9. `docs/requirements/README.md`
10. 当前阶段应完成的 story 列表

### 测试 / 验收任务

1. `docs/testing/README.md`
2. `docs/testing/phase-1-workspace-scanning-validation.md`（Phase 1 验证设计与验收标准）
3. `docs/testing/phase-2-evidence-validation.md`（Phase 2 验证设计与验收标准）
4. `docs/planning/phase-1-completion-review.md`（Phase 1 收尾验收与完成审查）
5. `docs/planning/phase-2-completion-review.md`（Phase 2 收尾验收与完成审查）
6. `docs/planning/phase-3-completion-review.md`（Phase 3 收尾验收与完成审查）
7. `docs/testing/phase-3-understanding-validation.md`（Phase 3 验证设计与验收标准）
8. `docs/testing/phase-4-view-validation.md`（Phase 4 视图验证设计与验收标准，active）
9. `docs/requirements/README.md`（验收标准来源）
10. `docs/planning/README.md`（当前阶段验收要求）
11. 相关 story 文件中的验收标准

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
