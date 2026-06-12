# Phase 3 单阶段结构化理解需求

---
status: draft
updated: 2026-06-12
---

> 本文档定义 Phase 3（单阶段结构化理解产物）的产品需求。Phase 3 基于 Phase 2 的 `EvidenceCollection`，生成结构化的 `ImplementationUnderstanding`，使用户能在不通读全部代码的情况下理解单个阶段实现了什么。
>
> **约束**：Phase 3 仍然不做图视图、不做 Q&A、不做持久化、不做跨阶段对比。Phase 3 的核心产出是结构化理解对象及其前端展示。

## 1. Phase 3 目标

Phase 3 编码完成后，产品应能：

1. 从 Phase 2 的 `EvidenceCollection` 出发，生成结构化的 `ImplementationUnderstanding` 对象
2. 每个 ImplementationClaim 必须绑定 evidence_id，可追溯到源码文件和行号
3. 明确区分 confirmed / inferred / unknown / conflicting 的置信度
4. 对 evidence 不足或无法理解的项标注 unknown 和 evidence gap
5. 生成模块、信号、接口、数据处理的结构化摘要
6. 前端展示单阶段理解摘要，用户能看到"这个阶段做了什么"
7. 保持目标项目只读、不运行工具、不调用 Vivado

## 2. Phase 3 用户价值

| 用户痛点 | Phase 3 解决方式 |
|----------|-----------------|
| 代码量大，无法逐行阅读 | 生成结构化摘要，快速了解阶段实现全貌 |
| 不知道代码中哪些是核心、哪些是辅助 | 通过 claim category 区分模块/信号/接口/处理步骤 |
| 无法判断理解是否可靠 | confidence 标签让用户区分确认/推断/未知 |
| AI 生成代码可能与预期不符 | evidence_id 回链让用户追溯每个 claim 到源码 |
| 工具给出结论但用户不敢信 | unknown / evidence gap 显式标注，不强行解释 |

## 3. Phase 3 做什么 / 不做什么

### 3.1 做

- 基于 `EvidenceCollection` 生成 `ImplementationUnderstanding`
- 每条 claim 绑定 evidence_id，支持 evidence 追溯
- 区分 claim confidence（confirmed / inferred / unknown / conflicting）
- 标注 unknown 和 evidence gap
- 生成模块/信号/接口/数据处理摘要
- 前端展示理解摘要面板
- 保持目标项目只读

### 3.2 不做

- **不做跨阶段对比**：Phase 3 只处理单个阶段
- **不做图视图**：结构图、数据流图、时序/流水图属于 Phase 4
- **不做 Q&A**：用户追问能力属于 Phase 5
- **不做持久化**：页面刷新后需重新生成，属于 Phase 6
- **不做 Vivado / synthesis / implementation / bitstream**：不运行任何 EDA 工具
- **不替用户判断正确/错误**：产品是理解工具，不是审计器
- **不做 PASS/HOLD 判定**：不做审计结论
- **不把 unknown 强行解释成结论**：evidence 不足时必须标注 unknown
- **不运行目标项目脚本**：只做静态分析 + 语义理解
- **不把输出写回目标项目目录**：所有写入在 app-owned 或临时目录

## 4. 功能点拆解

### IU-001 从 EvidenceCollection 生成 ImplementationUnderstanding

**输入**：
- `EvidenceCollection`（来自 Phase 2 `collect_evidence` 命令）

**输出**：
- `ImplementationUnderstanding`（结构化理解对象）

**后端责任**：
- 接收 `EvidenceCollection` 作为输入
- 调用理解生成流程（context builder → generator → validator）
- 返回 `CommandResult<ImplementationUnderstanding>`

**前端责任**：
- 在证据收集完成后，提供"生成理解"按钮
- 触发 `generateUnderstanding` 命令
- 处理成功/失败/degraded 状态

**验收标准**：
- 给定有效 `EvidenceCollection`，能生成结构完整的 `ImplementationUnderstanding`
- 生成的对象通过 schema validation
- 所有 evidence_refs 中的 evidence_id 在输入 EvidenceCollection 中存在

**非目标**：
- 不保证语义理解 100% 正确
- 不生成图视图

---

### IU-002 生成 ImplementationClaim

**输入**：
- `EvidenceCollection.evidence_items`
- `EvidenceCollection.index_by_path` / `index_by_kind` / `index_by_symbol`

**输出**：
- `Vec<ImplementationClaim>`：结构化的实现声明列表

**后端责任**：
- 基于证据生成 claim
- 每个 claim 包含：claim_id、category、description、confidence、evidence_refs
- claim 内容覆盖模块、信号、接口、数据处理等维度

**前端责任**：
- 展示 claim 列表
- 每个 claim 显示 category 标签、confidence 标签、evidence refs 链接

**验收标准**：
- 每个 claim 有唯一 claim_id
- 每个 claim 有明确的 category
- 每个 claim 的 confidence 为 ClaimConfidence 枚举值之一

**非目标**：
- 不保证 claim 数量或完整性

---

### IU-003 每个 claim 必须绑定 evidence_id

**输入**：
- `ImplementationClaim`
- `EvidenceCollection.evidence_items`（用于校验）

**输出**：
- `ImplementationClaim.evidence_refs: Vec<EvidenceRef>`

**后端责任**：
- 每个 claim 必须有 `evidence_refs` 或明确标注 `evidence_gap`
- evidence_refs 中的 evidence_id 必须在输入 EvidenceCollection 中真实存在
- hallucination guard：拒绝输出中包含不存在的 evidence_id

**前端责任**：
- 每个 claim 展示关联的 evidence_id 列表
- evidence_id 可点击，展示对应的 evidence item 详情

**验收标准**：
- 所有 evidence_refs 中的 evidence_id 在输入 EvidenceCollection 中可查到
- 无 evidence_refs 的 claim 必须有 evidence_gap 标注
- schema validator 拒绝包含虚假 evidence_id 的输出

**非目标**：
- 不要求 evidence_refs 完全覆盖 claim 的所有方面

---

### IU-004 claim confidence：confirmed / inferred / unknown / conflicting

**输入**：
- evidence strength（direct / indirect）
- evidence 数量和一致性

**输出**：
- `ImplementationClaim.confidence: ClaimConfidence`

**后端责任**：
- 定义 ClaimConfidence 枚举：confirmed / inferred / unknown / conflicting
- confidence 判定规则：
  - `confirmed`：有多条 direct strength evidence 支持
  - `inferred`：有 indirect evidence 或仅有单条 direct evidence 支持
  - `unknown`：evidence 不足或无法从 evidence 推断
  - `conflicting`：evidence 之间存在矛盾

**前端责任**：
- 每个 claim 展示 confidence 标签
- 颜色映射：confirmed=绿、inferred=蓝、unknown=灰、conflicting=红
- 不使用"正确/错误"、"PASS/HOLD"、"审计结论"等用语

**验收标准**：
- 每个 claim 的 confidence 是 ClaimConfidence 枚举值之一
- confidence 标签在前端正确展示

**非目标**：
- confidence 不是对代码正确性的判断
- Phase 3 不要求 confidence 判定算法完美

---

### IU-005 标注 unknown / evidence gap

**输入**：
- 无法从现有 evidence 推断出的信息需求

**输出**：
- `ImplementationUnderstanding.unknowns: Vec<UnknownItem>`
- `ImplementationUnderstanding.evidence_gaps: Vec<EvidenceGap>`

**后端责任**：
- 记录无法从 evidence 推断的信息项为 UnknownItem
- 记录期望存在但缺失的 evidence 为 EvidenceGap
- unknown 不允许绑定伪造的 evidence_id
- 每个 unknown/gap 有描述和可选的 related_evidence_refs

**前端责任**：
- 在理解面板中展示 unknown 和 evidence gap 区域
- 不隐藏或淡化不确定项
- 区分 unknown（无法理解）和 evidence gap（证据缺失）

**验收标准**：
- unknown 和 evidence gap 在前端可见
- 无伪造 evidence_id
- 用户能区分 confirmed / inferred / unknown / conflicting 内容

**非目标**：
- 不自动解决 unknown 或填补 evidence gap

---

### IU-006 生成模块/信号/接口/数据处理摘要

**输入**：
- `EvidenceCollection` 中的 RTL module、Python class/def、信号声明、接口定义等

**输出**：
- `ImplementationUnderstanding.module_summaries: Vec<ModuleSummary>`
- `ImplementationUnderstanding.signal_summaries: Vec<SignalSummary>`
- `ImplementationUnderstanding.interface_summaries: Vec<InterfaceSummary>`
- `ImplementationUnderstanding.processing_steps: Vec<ProcessingStepSummary>`

**后端责任**：
- 从 evidence 中提取模块、信号、接口、处理步骤的结构化摘要
- 每个摘要绑定 evidence_refs
- 摘要内容基于 evidence 事实，不做主观判断

**前端责任**：
- 分区展示模块摘要、信号摘要、接口摘要、处理步骤
- 每个摘要项显示关联的 evidence_id

**验收标准**：
- 摘要覆盖 evidence 中已识别的主要结构
- 每个摘要项有 evidence_refs

**非目标**：
- 不要求摘要 100% 完整
- 不做跨阶段接口追踪

---

### IU-007 前端展示单阶段理解摘要

**输入**：
- `ImplementationUnderstanding`（从后端返回）

**输出**：
- UnderstandingPanel 前端组件

**前端责任**：
- 新增 UnderstandingPanel 组件，嵌入 StageDetail
- 展示：阶段摘要、claim 列表、confidence 标签、evidence 回链、unknown/gap 区域、模块/信号/接口/处理步骤摘要
- 支持生成中/成功/失败/degraded/unknown-heavy 状态
- 不展示原始 JSON

**后端责任**：
- 返回结构化的 `ImplementationUnderstanding`
- 状态信息通过 CommandResult 传递

**验收标准**：
- 用户能看到"这个阶段做了什么"的结构化摘要
- 每个 claim 可追溯到 evidence
- 不确定项显式展示
- 无"正确/错误"、"PASS/HOLD"、"审计结论"用语

**非目标**：
- 不做图视图
- 不做 Q&A
- 不做 Markdown 报告导出

---

### IU-008 保持目标项目只读与不运行工具

**输入**：
- 全部 Phase 3 操作

**输出**：
- 目标项目无任何变化

**后端责任**：
- Phase 3 新增代码中不使用 `std::fs::write`、`std::fs::create_dir`、`std::fs::remove_file`、`std::fs::rename`、`std::fs::copy`
- 不使用 `std::process::Command` 或 `Command::new`
- 不调用 Vivado / synthesis / implementation / bitstream
- 不调用目标项目中的脚本
- 不把输出写回目标项目目录

**前端责任**：
- 不向用户展示任何可能暗示修改目标项目的操作

**验收标准**：
- `rg` 扫描 Phase 3 新增代码无写入/执行 API
- 目标项目目录在理解生成前后无变化

**非目标**：
- 无额外非目标

## 5. 与 Phase 2 的接口

Phase 3 消费 Phase 2 的 `EvidenceCollection`：

```text
Phase 2: collect_evidence(root_path, stage_id) → EvidenceCollection
Phase 3: generate_understanding(EvidenceCollection) → ImplementationUnderstanding
```

关键接口契约：
- Phase 3 通过 `evidence_id` 引用 Phase 2 的证据
- Phase 2 保证 `evidence_id` 在一次收集内全局唯一
- Phase 3 的 `source_path` 和 `line_range` 通过 `evidence_id` 回链，不在 claim 中重复

## 6. 与 Phase 4 的输出关系

Phase 3 的 `ImplementationUnderstanding` 是 Phase 4 的输入：

```text
Phase 3: ImplementationUnderstanding
Phase 4: 基于 ImplementationUnderstanding 生成结构图、数据流图、时序/流水图
```

## 7. 文档变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-12 | 初始创建（draft） | Claude |
