# Phase 9 真实 LLM 与 grounding 需求

---
status: draft
updated: 2026-06-18
---

> 本文档是 Phase 9（真实 LLM Provider 与 grounding 生产化）的**详细需求草案**。`status: draft`，尚未审核生效。Phase 9 **编码尚未开始**，**未接入任何真实 LLM**。本文需与设计、grounding 安全设计、UI/UX、测试、实施计划一同审核转 `active` 后，方允许进入 Phase 9 编码。
>
> 上游方向性文档：[`phase-9-overview-real-llm-grounding.md`](../planning/phase-9-overview-real-llm-grounding.md)（draft）。

## 1. 阶段目标（需求层）

Phase 9 在 **MockProvider / 确定性 heuristic** 之上引入**真实 LLM Provider**，作为语义引擎，用于：

- 提升 **semantic claims**（主算法路径、模块/接口语义）质量；
- 提升 **stage summary**（阶段结构化理解）质量；
- 提升 **grounded Q&A**（基于证据回答）质量；
- 支持 **复杂代码语义解释**（文档/代码不一致、跨概念归纳）。

但必须满足四项硬约束：**显式配置、可关闭、可审计、可回退**。heuristic 仍保留为 grounding 辅助与 fallback。

## 2. 业务背景

- MVP / Phase 7 的语义引擎为 MockProvider（确定性规则 + 关键词匹配），验证了数据流与 grounding 框架，但不是真正语义引擎（详见 overview §1）。
- Phase 8 工作台已具备承载更多 inferred / unknown / citation 的视觉层级。
- Phase 9 是 Post-MVP 中**安全敏感度最高**的阶段：首次引入外部网络调用、凭据、源码片段外发。

## 3. 功能性需求

### R9-001 可配置 Provider 层（默认关闭、显式启用）

- 在既有 Provider 抽象（`UnderstandingProvider` / Q&A provider）之上新增 **RealLlmProvider**，与 MockProvider 并存。
- **默认不启用真实 LLM**：未配置/未显式开启时，系统行为与 MVP 一致（MockProvider + heuristic），无回归。
- 真实 LLM 仅在用户**显式启用**后生效。

### R9-002 显式配置入口

- 配置项：`enable_real_llm`（开关）、`provider`（如 openai-compatible / anthropic-compatible 等抽象类别，不硬编码具体厂商密钥）、`model`、`base_url`、`api_key`。
- 配置必须**显式**：无默认开启、无隐式 provider、无内置密钥。
- 优先级：显式输入 > 环境变量 > 不启用。

### R9-003 凭据安全

- `api_key` **不写入目标项目**；如需持久化，仅写 **app-owned storage** 且可一键清除。
- `api_key` **不进日志、不进 session 明文、不进审计记录**。
- UI 不显示明文 `api_key`。

### R9-004 受控上下文构造（输入边界）

- 仅发送为回答当前问题 / 生成当前 understanding 所需的**证据片段（evidence excerpt）+ 必要 stage context 摘要 + 当前已选 trace/context**。
- **不发送**：完整仓库、完整源码副本、环境变量、密钥、`.git`、大体积二进制、与本问题无关的文件。

### R9-005 grounding / citation 要求

- 所有**非 unknown** 回答（Q&A）与 claim（understanding）**必须有 evidence citation**，且回链到已收集的 `evidence_id`。
- **不得伪造 citation**：引用不存在的 `evidence_id`、越界 `line_range`、无 citation 的非 unknown 回答必须被 validator 拒绝或降级为 unknown。

### R9-006 unknown 行为

- 证据不足时必须返回 `unknown` / `evidence_gap`，**不允许编造**。
- Q&A 在无可用上下文时返回 unknown 并说明原因；understanding 在证据不足字段保持空（沿用既有 empty_reason 机制）。

### R9-007 失败 / 超时 / 限流降级

- 真实 LLM 调用失败、超时、限流时，**优雅降级**：回退 MockProvider / heuristic，或返回 unknown；明确标记 provider 与 `degraded` 状态。
- 不崩溃、不残留半成品、不污染既有产物。

### R9-008 可审计

- 记录：`provider` / `model` / `timestamp` / `token 估算` / `error_code` / 是否被 grounding 拒绝。
- **不记录**：`api_key`、完整 prompt 中敏感内容、完整源码片段。
- 审计记录写 app-owned storage，脱敏，可由用户自查。

### R9-009 heuristic 作为 grounding 辅助与 fallback

- 确定性 heuristic（Phase 8 的 claim/流水线/摘要派生）保留：
  - 作为真实 LLM **失败时的 fallback**；
  - 作为 grounding **辅助证据/校验来源**（如候选 evidence、关键词提示）。
- 不得用 heuristic 伪造 LLM 产物；二者职责清晰。

### R9-010 安全边界（需求层声明）

- 不运行 Vivado / synthesis / implementation / bitstream；
- 不修改目标项目（`fpga_project_*` 只读）；
- 不默认运行目标项目脚本；
- 持久化只写 app-owned storage；
- 不输出 PASS/HOLD / 正确/错误 / 审计结论等用户可见裁决（真实 LLM 输出不包装为裁决）。

### R9-011 语义质量提升（用户价值，验收方向）

- 在真实项目（`fpga_project_coarse_sync`）L0/L4 上，真实 LLM 的 understanding/Q&A 在语义丰富度上**优于或至少不劣于** MockProvider/heuristic baseline；
- 优势以 grounding 校验通过为前提：质量提升不能以伪造 citation 为代价。

### R9-012 Phase 9 退出标准（需求层，方向性）

- 真实 LLM 默认关闭，显式启用可用；
- grounding 守住不胡说 / 不伪造 citation；
- 安全回归通过（凭据不泄露、无完整源码外发）；
- 不启用真实 LLM 时零回归；
- 全量构建/测试通过，真实桌面验收（含可选真实 LLM 路径）通过。
- 量化门槛在测试文档中定义。

## 4. 输入 / 输出边界（汇总）

| 维度 | 允许 | 禁止 |
|------|------|------|
| **发送给 LLM** | 证据片段、stage context 摘要、已选 trace/context、schema 约束 | 完整仓库、完整源码、env、密钥、`.git`、大二进制、无关文件 |
| **凭据** | app-owned 存储 / 环境变量 / 显式输入，可清除 | 写目标项目、日志、session 明文、审计记录 |
| **LLM 产出** | 经 grounding 校验的 claim/answer、unknown/gap | 伪造 citation、无 citation 的非 unknown 回答、裁决性结论 |
| **降级** | 回退 Mock/heuristic、返回 unknown | 崩溃、半成品、污染既有产物 |

## 5. 异常 / 空状态

- `provider_not_configured`：未配置 → 使用 Mock，UI 标注 provider=mock。
- `network_error` / `timeout` / `rate_limited`：降级 + 标 degraded + 审计 error_code。
- `invalid_response`：schema 校验失败 → 降级或 unknown。
- `citation_invalid` / `grounding_failed`：拒绝或降级 unknown。
- 无可用上下文的 Q&A：返回 unknown + 原因。

## 6. 证据与追溯要求（延续既有契约）

- LLM 产出的 claim/answer 必须绑定 `evidence_id` + 可追溯 `source_path` / `line_range`；
- 区分 confirmed / supported / inferred / unknown / conflicting；
- 证据不足明确标注 unknown，不强行解释。

## 7. 非目标

Phase 9 **不做**：

- 不做云端账号体系 / 计费 / server-first 架构；
- 不做自动代码修改（不写目标项目）；
- 不做目标项目正确性审计裁决；
- 不做跨阶段映射 / Python→RTL 等价验证（**留给 Phase 10**）；
- 不做长期语义记忆（**留给 Phase 11**）；
- 不上传完整源码；
- 不默认调用真实 LLM。

## 8. 安全边界（权威）

- 目标项目只读，checksum 前后一致；
- 不运行 Vivado / synthesis / implementation / bitstream，不默认运行目标项目脚本；
- 不调用真实 LLM（不读取 `api_key`、不外发）除非用户**显式启用**；
- 凭据不落日志 / session / 目标项目；
- 持久化只写 app-owned storage；
- 不输出 PASS/HOLD/正确/错误/审计结论裁决；
- 不引入 cloud-first 依赖；provider SDK 被 Provider 抽象隔离。

## 9. 关联文档

- [`../planning/phase-9-overview-real-llm-grounding.md`](../planning/phase-9-overview-real-llm-grounding.md) — Phase 9 overview（draft）
- [`../design/phase-9-llm-provider-architecture.md`](../design/phase-9-llm-provider-architecture.md) — Provider 架构设计（draft）
- [`../design/phase-9-grounding-and-validation-design.md`](../design/phase-9-grounding-and-validation-design.md) — grounding 与校验设计（draft）
- [`../ui-ux/phase-9-llm-configuration-and-grounded-qa-view.md`](../ui-ux/phase-9-llm-configuration-and-grounded-qa-view.md) — 配置与 Q&A UI/UX（draft）
- [`../testing/phase-9-real-llm-grounding-validation.md`](../testing/phase-9-real-llm-grounding-validation.md) — 验证设计（draft）
- [`../planning/phase-9-implementation-plan.md`](../planning/phase-9-implementation-plan.md) — 编码实施计划（draft）

## 10. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-18 | 初始 draft：Phase 9 真实 LLM 与 grounding 需求，覆盖可配置 Provider、显式配置、凭据安全、受控上下文、grounding/citation、unknown 行为、降级、可审计、heuristic 辅助、安全边界、非目标。`status: draft`，Phase 9 编码尚未开始，未接入真实 LLM。 | Claude |
