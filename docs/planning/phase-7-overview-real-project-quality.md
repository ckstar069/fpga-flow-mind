# Phase 7 Overview：真实项目评估与 evidence/understanding 质量补强

---
status: draft
updated: 2026-06-15
---

> 本文档是 Phase 7 的**方向性 overview**，描述主题、目标、范围、非目标、阶段关系与验收方向。
> 它**不**包含详细需求、设计、UI/UX、测试设计或编码任务清单——那些在进入 Phase 7 编码前另行编制并审核为 active。
> Phase 7 当前**未开始编码**。

## 1. 背景与问题

MVP（Phase 0–6）的真实桌面验收使用的是 `/tmp` 下手工构造的**临时小型样例项目**（`L0/L1/L2/rtl_final/docs`，每阶段仅 1–3 个文件）。这套样例足以证明**单阶段理解闭环在技术上成立**，但不足以证明分析能力在**真实** `ai_project_template` 业务项目上可信。

真实业务项目与临时样例之间存在系统性差距：

- **文件规模**：真实单阶段可能有数十个 Python/Verilog 文件、大量 import、嵌套目录；
- **语言混用**：前期 Python、后期 Verilog/RTL，且存在 SystemVerilog、约束文件、生成脚本等；
- **外部依赖**：从 `urban_wireless` 导入的模块、相对/绝对 import、跨阶段接口约定文件；
- **噪声**：TODO 注释、实验性代码、被注释掉的块、文档与代码不一致；
- **阶段形态多样**：空阶段、命名异常阶段（非标准 `L0..RTL`）、并行多模块阶段。

这些差距会在 evidence 覆盖率、提取质量、`ImplementationUnderstanding` 可信度、视图可用性、Q&A 命中率上集中暴露。MVP 没有对这些维度做过量化评估，因此**当前无法回答"在真实项目上，理解产物到底有多可信"**。

Phase 7 的核心命题是：**在真实项目上，量化并提升分析质量，让理解产物可信。**

## 2. 阶段目标

1. **建立真实项目评估基准**：选择有代表性的真实 `ai_project_template` 业务项目作为评估语料（非 `/tmp` 临时样例），明确每个维度的度量方法。
2. **量化 evidence 覆盖率与提取质量**：度量真实项目下 Python/Verilog 证据的文件覆盖率、符号命中率、行号准确性、`strength` 标注合理性。
3. **量化并补强 `ImplementationUnderstanding` 质量**：评估 claim 的 `evidence_refs` 真实性、`confidence` 标注合理性、`unknown`/`evidence_gap` 是否被正确表达而非强行解释。
4. **评估视图可信度**：真实项目下三类视图（结构/数据流/时序）是否仍可读、节点/边 `trace_refs` 是否准确回链、是否出现"孤立方块"或错连。
5. **评估 Q&A 行为与 unknown 质量**：在仍使用 MockProvider 的前提下，刻画 Q&A 在真实项目上的命中率、unknown/gap 的合理性，**为 Phase 9 真实 LLM 接入提供"基线缺口清单"**。

> Phase 7 **不接入真实 LLM**。Q&A 仍由 MockProvider 承担；Phase 7 的价值之一是**用真实项目暴露 MockProvider 的能力边界**，作为 Phase 9 的输入。

## 3. 用户价值

- 用户在**自己真实的项目**上打开 `fpga-flow-mind`，看到的是可信的理解产物，而不是只在 toy 样例上有效；
- 用户能感知到"哪些结论是 confirmed、哪些是 inferred、哪些是 unknown"，且这些标注在真实噪声下仍然诚实；
- 为后续 Phase 8（产品 UI）、Phase 9（真实 LLM）提供质量地基——理解产物不可信，再好的 UI 和 LLM 也只是把不可信的东西包装得更漂亮。

## 4. 允许范围

Phase 7 允许做（具体范围在详细文档中收敛）：

- 选择并登记真实评估项目语料；
- 定义并实现分析质量度量（覆盖率、命中率、回链准确性等）；
- 补强 Phase 2 证据提取规则（Python/Verilog）以提升真实项目覆盖率；
- 补强 Phase 3 understanding 生成规则（仍为确定性/Mock，不调 LLM）以提升 claim/evidence/unknown 质量；
- 修正 Phase 4 视图派生在真实项目下的退化（孤立方块、错连、trace 回链失效）；
- 调整/补强 evidence/understanding 的验证测试与回归测试。

## 5. 明确非目标

Phase 7 **不做**：

- **不接入真实 LLM**（属于 Phase 9）；
- **不做 UI 大重构 / 工作台化**（属于 Phase 8），UI 改动仅限为评估必要的最小展示；
- **不做跨阶段映射 / Python-to-RTL 映射**（属于 Phase 10），仍聚焦单阶段理解质量；
- **不做多阶段语义记忆 / agent-scope 联动**（属于 Phase 11）；
- **不修改目标业务项目**，不运行 Vivado / synthesis / implementation / bitstream；
- **不引入新的外部依赖**（无新 LLM SDK、无网络库）；
- **不改变核心语义模型**（`ImplementationUnderstanding`、`confidence` 枚举、evidence model 的字段语义保持稳定，必要时扩展但不破坏既有契约）。

## 6. 与前后阶段关系

- **前置**：Phase 6（MVP completion）。Phase 7 是 MVP 之后的第一步，无其他前置。
- **后置依赖方**：
  - Phase 8（UI 工作台）受益于 Phase 7 暴露的"真实理解形态"来定型信息架构；
  - Phase 9（真实 LLM）以 Phase 7 产出的"基线缺口清单 + evidence/understanding 质量基线"为输入，grounding 在真实、有噪声的证据上验证；
  - Phase 10（跨阶段）依赖 Phase 7 对真实多阶段项目的单阶段理解能力。
- 详见 [`post-mvp-roadmap.md`](post-mvp-roadmap.md) §4 依赖顺序：Phase 7 是质量主干的起点。

## 7. 未来详细文档清单（进入 Phase 7 编码前编制）

| 文档 | 目录 | 内容方向 |
|------|------|----------|
| 真实项目质量需求 | `docs/requirements/` | 评估维度定义、度量标准、质量门槛、真实项目语料选择标准 |
| 评估与质量设计 | `docs/design/` | 度量实现方式、提取/生成规则补强设计、回归测试设计、（不破坏契约下的）模型扩展 |
| 验证设计 | `docs/testing/` | 真实项目评估流程、量化度量脚本、回归基线、安全回归 |
| 编码实施计划 | `docs/planning/` | 任务拆解、依赖、Batch 划分、退出条件 |

UI/UX 文档：Phase 7 原则上**不需要**单独的 UI/UX 设计文档（非 UI 阶段）；如评估需要最小展示，在实施计划中说明即可。

## 8. 验收方向（方向性，具体门槛在需求/测试文档中量化）

- 真实项目语料上，evidence 覆盖率与提取质量达到量化门槛；
- `ImplementationUnderstanding` 的 claim 全部通过 existence check，`unknown`/`evidence_gap` 在证据不足处被诚实表达；
- 三类视图在真实项目下可读、`trace_refs` 准确回链；
- Q&A（MockProvider）行为被刻画，产出"基线缺口清单"供 Phase 9；
- 真实桌面验收在真实业务项目上通过（不只 `/tmp` 样例）；
- 目标项目只读、不运行 Vivado 等安全边界保持；
- 全量 `npm run build` / `cargo test --lib` / `cargo check` 通过。

## 9. 风险与边界

- **评估主观性风险**：理解质量难以纯客观度量，需在需求文档中定义可操作的量化指标，避免"看起来更好"式验收。
- **规则补强过拟合风险**：针对真实项目补强提取/生成规则时，易过拟合到具体项目；需用多项目语料交叉验证。
- **模型扩展破坏契约风险**：若为质量补强扩展 evidence/understanding 字段，必须不破坏 `mvp-functional-contract.md` 既有契约与持久化兼容性。
- **范围蔓延风险**：补强过程中容易顺手做 UI 调整或跨阶段尝试，必须严格守住 §5 非目标。
- **安全边界**：真实项目可能含敏感代码，Phase 7 不外发任何内容（不接 LLM）、不修改目标项目、不运行工具链。

## 10. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-15 | 初始 draft：Phase 7 overview，主题为真实项目评估与 evidence/understanding 质量补强，明确目标/范围/非目标/阶段关系/验收方向。未进入编码。 | Claude |
