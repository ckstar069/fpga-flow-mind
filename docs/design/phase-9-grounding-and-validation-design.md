# Phase 9 grounding 与校验设计

---
status: active
updated: 2026-06-18
---

> 本文档是 Phase 9 的 **grounding 与校验安全设计**。`status: active`，已审核通过。Phase 9 **Batch A 编码已完成并审核收口**；**Phase 9 Batch B 编码已完成并进入审核收口**（RequestBuilder / ResponseParser / 可注入 Transport / RealLlmProvider 骨架），**未接入真实 LLM**，**未发起真实网络调用**；Batch C/D/E 尚未开始。Phase 10/11 尚未开始。
>
> 本文档是 Phase 9 **安全敏感度最高**的部分：它定义"如何让真实 LLM 不胡说、不伪造 citation、不泄露敏感数据"。

## 1. 设计目标

1. **grounding 守门**：所有非 unknown 的 LLM 产出必须回链真实 evidence。
2. **citation 一致性**：citation 的 `evidence_id` / `source_location` / `line_range` 与已收集证据一致。
3. **敏感数据不出本地**：`api_key`/env/secrets/`.git`/大二进制不进入 prompt。
4. **prompt 注入防护**：目标项目文本只作 data，不得覆盖系统约束。
5. **confidence 语义清晰**：confirmed/supported/inferred/unknown/conflicting 由 LLM 输出 + validator 共同约束。

## 2. Grounding pipeline

```text
EvidenceSelection（按相关性/强度选取证据片段）
  -> ContextPacking（摘要 + 片段 + schema 约束 + redaction）
  -> PromptConstraints（系统约束：必须 citation、不足返 unknown、禁裁决语）
  -> StructuredResponse（LLM 输出结构化 JSON）
  -> SchemaValidation（结构 + evidence_id 存在性）
  -> CitationValidation（citation 合法性 + line_range 边界）
  -> UIPresentation（citation 高亮 / unknown / degraded 可见）
```

任一校验失败 → 拒绝该产出或降级 unknown，并记录 `error_code`。

## 3. citation 模型与一致性

- citation 字段：`citation_index`（输出内编号）、`evidence_id`、`source_location`（`source_path`）、`line_range`（`{start,end}`）。
- 一致性要求：
  - `evidence_id` 必须存在于当前 `EvidenceCollection`；
  - `source_location` 必须与该 evidence 的 `source_path` 一致；
  - `line_range` 必须落在该 evidence 实际行范围内（不得越界/伪造）。
- 输出内多个 citation 各自独立校验。

## 4. hallucination guard

- **不存在 evidence_id**：校验失败 → 拒绝/降级 unknown。
- **越界 line_range**：校验失败 → 拒绝/降级 unknown。
- **无 citation 的非 unknown 回答**：拒绝（强制 unknown）。
- understanding 端：claim 的 `evidence_refs` 全部走 `SchemaValidator` 既有 hallucination guard（evidence_id 存在性）。
- Q&A 端：answer 若非 unknown，必须携带至少一条合法 citation；否则 `GroundedQaValidator` 拒绝。

## 5. sensitive data guard

- 在 `ContextPacking` / `RequestBuilder` 阶段过滤：
  - `api_key` / 环境变量值 / secrets；
  - `.git` 目录内容；
  - 大体积二进制（按 size 阈值跳过）；
  - 与本问题无关的文件。
- prompt 中**只保留**证据片段 + 摘要 + 约束。
- 测试断言：构造请求的 redacted payload 不含上述敏感项（详见测试文档）。

## 6. prompt injection 防护

- 目标项目文本（证据片段、注释、文档）**只能作为 data**，置于 data 区，不得进入 system 约束区。
- system 约束（必须 citation、不足返 unknown、禁裁决语、禁执行指令）置于 prompt 顶层，不被 data 覆盖。
- 设计上不把"目标项目文本里出现的指令"当作系统指令执行；LLM 输出仍经 grounding 校验兜底。

## 7. confidence 语义（LLM + validator 共同约束）

| confidence | 语义 | validator 角色 |
|------------|------|----------------|
| `confirmed` | 多条直接证据强支撑 | 校验 evidence_id/line_range 合法 |
| `supported` | 有证据支撑 | 同上 |
| `inferred` | 基于证据推断 | 同上，标注推断 |
| `unknown` | 证据不足 | 强制：无合法 citation 时不得高于 unknown |
| `conflicting` | 证据互相矛盾 | 校验多 citation 合法 |

- LLM 自报 confidence 经 validator 复核：若 evidence 不足却自报高 confidence，validator 降级为 unknown（宁可多 unknown，不可伪造）。

## 8. understanding 与 Q&A 的差异

| 维度 | understanding 生成 | Grounded Q&A |
|------|--------------------|--------------|
| 输入 | 整阶段 `EvidenceCollection` + `StageContext` 摘要 | 当前已选 trace/context + 相关 evidence 子集 |
| 输出 | 结构化 `ImplementationUnderstanding`（claims/modules/steps/...） | 单条 answer + citations / unknown / evidence_gap |
| 校验 | `SchemaValidator`（结构 + evidence_id 存在性） | `GroundedQaValidator`（citation 存在性 + unknown 规则） |
| 触发 | 显式"生成理解" | 用户提问 |

- understanding 只在用户显式生成时调用 LLM；Q&A 只基于当前上下文回答，不全局重扫。

## 9. failure modes 与处置

| failure mode | 触发 | 处置 |
|--------------|------|------|
| `provider_not_configured` | 未配置/未启用 | 使用 Mock，标 provider=mock |
| `network_error` | 网络失败 | 降级 + degraded + 审计 |
| `timeout` | 超时 | 降级 + degraded |
| `rate_limited` | 429/限流 | 降级 + degraded，尊重 Retry-After |
| `cancelled` | 用户取消进行中的调用 | 降级 + degraded（已取消，不视为 error，但走 fallback/unknown 链路，审计记录 `cancelled`） |
| `invalid_response` | 解析/schema 失败 | 拒绝该产出或降级 unknown |
| `citation_invalid` | citation 不存在/越界 | 拒绝或降级 unknown |
| `grounding_failed` | grounding 整体不通过 | 降级 unknown |

> 说明：本表 failure mode 是 **grounding/降级处置层** 的语义分类（snake_case）。Provider 调用层另有 `ProviderError` 枚举（PascalCase，定义见架构 §2.1/§6）。两层不混用：`ProviderError` 描述"调用本身为何失败"，本表描述"对最终产物的处置"。

## 10. 安全边界

- 任何路径不得绕过 grounding（无"信任原文"旁路）。
- 敏感数据不进 prompt；凭据不落日志/session/目标项目。
- 不修改目标项目，不运行工具链。
- LLM 输出不包装为 PASS/HOLD/正确/错误/审计裁决。

## 11. 关联文档

- [`phase-9-llm-provider-architecture.md`](phase-9-llm-provider-architecture.md) — Provider 架构（draft）
- [`../requirements/phase-9-real-llm-grounding-requirements.md`](../requirements/phase-9-real-llm-grounding-requirements.md) — 需求（draft）
- [`../ui-ux/phase-9-llm-configuration-and-grounded-qa-view.md`](../ui-ux/phase-9-llm-configuration-and-grounded-qa-view.md) — UI/UX（draft）
- [`../testing/phase-9-real-llm-grounding-validation.md`](../testing/phase-9-real-llm-grounding-validation.md) — 验证设计（draft）
- [`../planning/phase-9-implementation-plan.md`](../planning/phase-9-implementation-plan.md) — 实施计划（draft）

## 12. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-18 | 初始 draft：grounding pipeline、citation 模型、hallucination guard、敏感数据防护、prompt 注入防护、confidence 语义、understanding/Q&A 差异、failure modes。`status: draft`，未接入真实 LLM，编码未开始。 | Claude |
