# Story: 生成结构化理解

---
status: draft
updated: 2026-06-11
---

## 用户目标

作为 FPGA 开发人员，我希望系统基于收集到的证据生成对该阶段实现的结构化理解，以便后续通过视图和问答快速掌握实现意图。

## 业务背景

原始 evidence item 是离散的符号和代码片段，用户无法直接从这些片段理解整体实现。系统需要通过大模型/Agent 基于 evidence 进行语义归纳，生成结构化的理解产物。

## 前置条件

- evidence 收集已完成并建立索引
- evidence_index.json 可用

## 触发入口

用户在证据收集结果面板中点击"生成理解"或"开始分析"按钮。

## 主流程

1. 用户触发生成理解
2. 系统构建调查上下文
3. 系统调用大模型/Agent 生成结构化理解
4. 系统进行 grounding 检查
5. 系统生成并保存 ImplementationUnderstanding
6. 系统展示理解生成结果

## 功能点清单

### IU-001 触发理解生成

- **用户动作**：用户点击"生成理解"按钮
- **系统必须**：
  - 显示生成进度（上下文构建、模型调用、结果解析、grounding 检查）
  - 允许用户取消
  - 显示预计耗时提示
- **成功结果**：理解生成流程启动
- **失败表现**：无 evidence 时禁用按钮并提示"请先收集证据"
- **evidence 要求**：不需要
- **MVP 必须**：是

### IU-002 构建调查上下文

- **系统动作**：从 evidence 索引中构建供大模型理解的上下文
- **系统必须**：
  - 按 source_kind 分组组织 evidence（Python / RTL / Test / Doc / Config）
  - 为每个 evidence item 提供：evidence_id、source_path、line_range、summary
  - 构建阶段目标描述（基于文档 evidence 或目录名推断）
  - 构建关键符号列表（模块名、函数名、信号名）
  - 控制上下文长度，避免超出模型输入限制
- **成功结果**：生成结构化的调查上下文
- **失败表现**：上下文构建失败时提示"无法构建分析上下文"
- **evidence 要求**：上下文中的每个引用必须包含 evidence_id
- **MVP 必须**：是

### IU-003 调用大模型/Agent 生成理解

- **系统动作**：将调查上下文提交给大模型/Agent，请求生成结构化理解
- **系统必须**：
  - 提交上下文和结构化输出要求
  - 要求模型输出：阶段摘要、结构视图描述、数据流视图描述、时序视图描述、概念列表、公式列表、信号列表、证据引用、不确定项
  - 要求模型对每个 claim 标注置信度（confirmed / supported / inferred / unknown / conflicting）
  - 要求模型对每个 claim 引用 evidence_ids
- **成功结果**：模型返回结构化的理解结果
- **失败表现**：
  - 模型调用失败：提示"分析服务暂不可用，请重试"
  - 返回格式错误：提示"分析结果格式异常，请重试"
- **evidence 要求**：模型输出中的每个 claim 必须关联 evidence_ids
- **MVP 必须**：是

### IU-004 结构化理解产物定义

- **系统动作**：将模型输出转换为统一的 `ImplementationUnderstanding` 结构
- **系统必须**：生成的产物包含以下部分：
  - `stage_id`：阶段标识
  - `summary`：阶段实现的一句话摘要 + 详细描述
  - `structure_view`：模块列表、接口列表、层级关系
  - `dataflow_view`：数据来源、变换步骤、流向输出
  - `timing_view`：latency 估计、握手信号流、流水线行为
  - `concepts[]`：关键概念列表（名称、描述、evidence_refs）
  - `formulas[]`：关键公式/表达式列表（表达式、描述、evidence_refs）
  - `signals[]`：关键信号列表（名称、类型、来源、evidence_refs）
  - `evidence_refs[]`：所有引用的 evidence_id 列表
  - `uncertainties[]`：不确定项列表（描述、类型、相关 evidence_refs）
- **成功结果**：结构化的 ImplementationUnderstanding 已生成
- **失败表现**：转换失败时提示"无法解析理解结果"
- **evidence 要求**：每个 claim 必须绑定 evidence_ids 或 uncertainty_ids
- **MVP 必须**：是

### IU-005 claim 绑定 evidence_ids

- **系统动作**：确保每个语义 claim 正确绑定到 evidence
- **系统必须**：
  - 每个 claim（模块描述、数据流步骤、时序关系、概念、公式、信号）必须关联至少一个 evidence_id
  - claim 的 evidence_refs 必须是 evidence_index 中实际存在的 evidence_id
  - 如果 claim 无直接 evidence 支撑，必须关联 uncertainty_id 并标注为 `unknown`
- **成功结果**：所有 claim 都有 evidence 或 uncertainty 绑定
- **失败表现**：无 evidence 绑定的 claim 被标记为 `unknown` 并加入 uncertainties 列表
- **evidence 要求**：每个 claim 的 evidence_refs 必须可验证存在于 evidence_index
- **MVP 必须**：是

### IU-006 置信度判定规则

- **系统动作**：根据 evidence 强度判定每个 claim 的置信度
- **系统必须**：
  - `confirmed`：有强源码证据直接支撑（如模块定义、明确的端口声明）
  - `supported`：有证据支撑但需辅助推断（如从多个信号推断出数据通路）
  - `inferred`：基于间接证据或上下文推断（如从文档描述推断实现意图）
  - `unknown`：证据不足，无法确定（如缺少关键文件）
  - `conflicting`：存在矛盾的证据（如代码与文档描述不一致）
- **成功结果**：每个 claim 都有明确的置信度标注
- **失败表现**：无法判定时默认标注为 `unknown`
- **evidence 要求**：置信度判定应基于 evidence 的可验证属性
- **MVP 必须**：是

### IU-007 禁止生成 confirmed 结论的情况

- **系统动作**：在以下情况下，不得将 claim 标注为 `confirmed`
- **系统必须**：
  - 无直接源码证据支撑时，不得标注 `confirmed`
  - 基于文档描述 alone 时，最高标注为 `supported` 或 `inferred`
  - 证据之间存在矛盾时，必须标注为 `conflicting`
  - 模型基于训练知识而非当前 evidence 得出的结论，不得标注为 `confirmed`
- **成功结果**：所有 `confirmed` 结论都有严格的 evidence 支撑
- **失败表现**：审核发现无 evidence 支撑的 `confirmed` 结论时，应降级为 `inferred` 或 `unknown`
- **evidence 要求**：每个 `confirmed` claim 必须可被 evidence_index 中的 item 直接验证
- **MVP 必须**：是

### IU-008 grounding 检查

- **系统动作**：验证模型生成的结论是否与 evidence 一致
- **系统必须**：
  - 检查 claim 引用的 evidence_id 是否存在于 evidence_index
  - 检查 claim 描述是否与 evidence 内容一致（如 claim 说"模块 A 有 3 个输入端口"，但 evidence 显示有 5 个）
  - 对不一致的 claim 标注为 `conflicting` 或降级置信度
  - 对无法 ground 的 claim 标注为 `unknown`
- **成功结果**：grounding 检查通过，claims 与 evidence 一致
- **失败表现**：
  - grounding 失败时，将失败的 claim 降级或标记为 uncertain
  - 不删除失败的 claim，而是标注其问题
- **evidence 要求**：grounding 检查必须基于 evidence_index 中的实际内容
- **MVP 必须**：是

### IU-009 grounding 失败处理

- **系统动作**：当 grounding 检查发现 claim 与 evidence 不一致时处理
- **系统必须**：
  - 将不一致的 claim 加入 uncertainties 列表
  - 在 claim 中标注 `conflicting` 或降级为 `inferred`/`unknown`
  - 在 uncertainties 中说明不一致的原因
  - 不自动修正 claim 以匹配 evidence（由用户判断）
- **成功结果**：不一致项被显式标注，用户了解问题所在
- **失败表现**：无
- **evidence 要求**：不一致说明应引用涉及的 evidence_ids
- **MVP 必须**：是

### IU-010 理解结果展示

- **系统动作**：展示结构化理解结果概览
- **系统必须**：
  - 显示阶段摘要
  - 显示 claim 总数和置信度分布（confirmed / supported / inferred / unknown / conflicting 数量）
  - 显示不确定项列表（标题和简要描述）
  - 提供"查看结构图"、"查看数据流图"、"查看时序图"的入口按钮
- **成功结果**：用户了解阶段实现的总体情况
- **失败表现**：生成失败时提示"理解生成失败"，允许用户重试
- **evidence 要求**：概览中的 claim 统计应关联到 evidence 分布
- **MVP 必须**：是

## 输入

| 输入项 | 来源 | 类型 |
|--------|------|------|
| evidence_index.json | story-collect-evidence 输出 | 结构化数据 |
| 大模型/Agent | 外部服务 | 语义理解引擎 |

## 输出

| 输出项 | 类型 | 说明 |
|--------|------|------|
| implementation_understanding.json | 结构化数据 | 结构化理解产物 |
| 理解结果概览面板 | UI 状态 | 摘要、置信度分布、不确定项、视图入口 |

## 异常 / 空状态

| 场景 | 处理 |
|------|------|
| evidence_index 为空 | 禁用"生成理解"按钮，提示"请先收集证据" |
| 模型调用失败 | 提示"分析服务暂不可用，请稍后重试" |
| 模型返回格式错误 | 提示"分析结果异常，请重试"，记录原始响应用于调试 |
| grounding 检查大量失败 | 提示"部分结论与证据不一致，已标注为不确定" |
| 生成过程被取消 | 保留部分结果（如有），提示"生成已中断" |

## 证据与追溯要求

- 每个 claim 必须绑定至少一个 evidence_id 或 uncertainty_id
- evidence_refs 必须是 evidence_index 中真实存在的 evidence_id
- 用户应能通过 evidence_id 追溯到 source_path 和 line_range

## 不确定性表达要求

- `unknown` claims 必须加入 uncertainties 列表并说明原因
- `conflicting` claims 必须列出冲突的 evidence_ids
- `inferred` claims 必须说明推断依据
- 系统不应隐藏任何 uncertain 或 conflicting 的 claim

## MVP 验收标准

- [ ] 能基于 evidence_index 生成结构化理解产物
- [ ] 每个 claim 绑定 evidence_ids 或 uncertainty_ids
- [ ] 置信度分级正确（confirmed/supported/inferred/unknown/conflicting）
- [ ] grounding 检查验证 claim 与 evidence 的一致性
- [ ] grounding 失败时不删除 claim，而是标注为 conflicting/unknown
- [ ] 无 evidence 支撑的 claim 不得标注为 confirmed
- [ ] 理解结果概览展示摘要、置信度分布和不确定项
- [ ] 提供三类视图的入口按钮

## 非目标

- 不做跨阶段理解（单阶段分析）
- 不做自动生成测试用例
- 不做代码修复建议
- 不做性能优化建议
- 不做形式化验证

## 关联文档

- [`story-collect-evidence.md`](story-collect-evidence.md) — 前置：收集证据
- [`story-view-structure.md`](story-view-structure.md) — 下一步：查看结构图
- [`story-view-dataflow.md`](story-view-dataflow.md) — 下一步：查看数据流图
- [`story-view-timing.md`](story-view-timing.md) — 下一步：查看时序图
- [`../mvp-requirements.md`](../mvp-requirements.md) — MVP 结构化理解要求
- [`../../design/evidence-model.md`](../../design/evidence-model.md)（待创建）— evidence 模型设计
