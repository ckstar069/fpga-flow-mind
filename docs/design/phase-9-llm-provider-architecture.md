# Phase 9 LLM Provider 架构设计

---
status: active
updated: 2026-06-18
---

> 本文档是 Phase 9 的 **Provider 架构设计**。`status: active`，已审核通过。Phase 9 **Batch A 编码已完成**（Provider 抽象、配置模型、Fake/Mock transport、no-network-by-default 守卫与测试），**未接入任何真实 LLM**，**未发起真实网络调用**。Batch B/C/D/E 尚未开始。
>
> 上游：[`phase-9-overview-real-llm-grounding.md`](../planning/phase-9-overview-real-llm-grounding.md)、需求 [`phase-9-real-llm-grounding-requirements.md`](../requirements/phase-9-real-llm-grounding-requirements.md)。

## 1. 设计原则

1. **抽象不变**：真实 LLM 实现为既有 `UnderstandingProvider` / Q&A provider 的新具体类型，不破坏既有契约。
2. **默认关闭**：真实网络调用默认 disabled，仅用户显式启用后允许。
3. **可回退**：真实 LLM 失败时回退 MockProvider / heuristic，标 degraded。
4. **不发完整源码**：上下文只含证据片段 + 摘要。
5. **凭据最小暴露**：不落日志 / session / 目标项目。
6. **本地桌面优先**：LLM 仅作为本地发起的可控外部调用，不引入 cloud-first 依赖。

## 2. Provider 抽象

### 2.1 类型与职责

| 类型 | 职责 | 是否真实网络 |
|------|------|--------------|
| `MockProvider` | 确定性 heuristic 生成（既有） | 否 |
| `RealLlmProvider` | 真实 LLM 调用 + 结构化输出 | 是（显式启用后） |
| `ProviderConfig` | enable / provider 类别 / model / base_url / api_key / timeout / retry 预算 | — |
| `ProviderCapability` | 声明能力：支持 understanding / Q&A / 结构化输出 / 最大 context | — |
| `ProviderError` | `NotConfigured` / `NetworkError` / `Timeout` / `RateLimited` / `InvalidResponse` / `ProviderCallFailed` | — |

### 2.2 选择逻辑

- `enable_real_llm == false` 或配置不完整 → 使用 `MockProvider`。
- `enable_real_llm == true` 且配置完整 → 使用 `RealLlmProvider`；调用失败时按 §7 降级。

## 3. 真实 LLM 调用链

```text
UI 配置 (ProviderConfig)
  -> Tauri command (set_provider_config / generate_understanding / ask_grounded_question)
  -> Rust Provider 选择（Mock | Real）
  -> RequestBuilder（redaction + context packing + schema 约束）
  -> [可注入的 Transport：真实 HTTP / mock transport / fake provider]
  -> ResponseParser（结构化解析）
  -> SchemaValidator（hallucination guard：evidence_id 存在性）
  -> GroundedQaValidator / understanding grounding 校验（citation 存在性）
  -> 结果（成功 / degraded / unknown）
  -> AuditRecord（脱敏留痕）
```

- Transport 在 Batch B 设计为**可注入**：单元/集成测试用 mock transport / fake provider，不发真实网络请求。

## 4. 配置存储边界

- `ProviderConfig` 中 `api_key`：
  - **不写入目标项目**；
  - 持久化（如需）仅写 **app-owned storage**，且 UI 可一键清除；
  - 优先级：显式输入 > 环境变量（如 `FPGA_FLOW_LLM_API_KEY`）> 不启用；
  - 内存中不通过日志/Debug 输出泄露。
- 其他配置（provider/model/base_url/timeout）可持久化，但仍 app-owned。

## 5. 网络调用开关

- `enable_real_llm` 默认 `false`。
- 仅当显式 `true` 且配置完整时，`RealLlmProvider` 才发起真实请求。
- 全局存在单一可审计的"真实调用入口"，便于门禁与测试断言"默认不发请求"。

## 6. timeout / retry / rate limit / error mapping

| 参数/场景 | 设计方向 |
|-----------|----------|
| `timeout` | 可配置，默认有界（如 30~60s），到点判 `Timeout` |
| `retry` | 有界重试（如 1~2 次），仅对幂等可重试错误（网络瞬断/5xx）重试；4xx/限流不盲目重试 |
| `rate limit` | 尊重 429/Retry-After；本地轻量节流，避免无界调用 |
| `cancellation` | 用户可取消进行中的 LLM 调用；取消与 timeout 同等处理（终止调用、不残留半成品、按 §10 降级到 Mock/unknown、标 degraded）。Transport 抽象暴露取消信号，便于注入测试断言。 |
| `error mapping` | 网络→`NetworkError`；超时→`Timeout`；429→`RateLimited`；解析失败→`InvalidResponse`；用户取消→作为降级处理（不视为 error，但标 degraded/已取消）；其他→`ProviderCallFailed` |

> 说明：`ProviderError` 是 **Provider 调用层** 的错误枚举（见 §2.1）。grounding 校验层另有 failure mode（`citation_invalid` / `grounding_failed`，snake_case），属不同层枚举，定义见 grounding 设计 §9；两层不混用。

## 7. prompt / context 构造

- 输入只基于：`EvidenceCollection`（证据片段）、`StageContext` 摘要、当前已选 trace/context、schema 约束。
- **不发送完整源码副本**；片段受 context 预算与裁剪策略约束（按相关性/强度选取，超限截断并标注）。
- redaction：在 RequestBuilder 阶段过滤 `api_key`/env/`.git`/大二进制等敏感项（详见 grounding 设计文档）。

## 8. response validation

- 复用既有 `SchemaValidator`（understanding 结构 + evidence_id 存在性 hallucination guard）。
- 复用 `GroundedQaValidator`（Q&A citation 存在性）。
- 新增/强化：citation 越界校验（`line_range` 落在 evidence 实际范围）、非 unknown 回答必须有合法 citation。
- 校验失败：拒绝该产出或降级 unknown，并记录 `error_code`。

## 9. audit / logging

- `AuditRecord` 字段：`provider` / `model` / `timestamp` / `token_estimate` / `success` / `error_code` / `grounding_rejected`（是否被 grounding 拒绝）/ `degraded`。
- **不记录**：`api_key`、完整 prompt 正文、完整源码片段、用户私密内容。
- 写 app-owned storage，可由用户自查；不外发。

## 10. fallback

- 真实 LLM 失败（网络/超时/限流/校验失败/用户取消）→ 回退 `MockProvider` / heuristic，或返回 unknown。
- 回退产物明确标记 `provider` 与 `is_degraded`，UI 可见（详见 UI/UX 文档）。
- 用户取消按降级处理（不视为 error，但标 degraded/已取消，避免用户取消被误报为调用失败）。
- 不存在"信任 LLM 原文输出而绕过 grounding"的旁路。

## 11. 与既有 Phase 3/5/8 对接

- 既有 `UnderstandingProvider` trait 不变；新增 `RealLlmProvider` 实现。
- 既有 `GroundedQaValidator` / `SchemaValidator` 复用并强化。
- 既有 `MockProvider` 与 Phase 8 heuristic 派生保留为 fallback / 辅助。
- Phase 8 工作台视觉层级（provider/degraded/citation/unknown）承载 Phase 9 新状态。

## 12. 安全边界

- 不引入 cloud-first 依赖；provider SDK 被 `RealLlmProvider` 隔离，不向上层泄漏。
- `api_key` 不落日志/session/目标项目/审计。
- 默认不发真实网络请求；真实调用仅单一可审计入口。
- 不修改目标项目，不运行工具链。

## 13. 关联文档

- [`../requirements/phase-9-real-llm-grounding-requirements.md`](../requirements/phase-9-real-llm-grounding-requirements.md) — 需求（draft）
- [`phase-9-grounding-and-validation-design.md`](phase-9-grounding-and-validation-design.md) — grounding 与校验设计（draft）
- [`../ui-ux/phase-9-llm-configuration-and-grounded-qa-view.md`](../ui-ux/phase-9-llm-configuration-and-grounded-qa-view.md) — UI/UX（draft）
- [`../testing/phase-9-real-llm-grounding-validation.md`](../testing/phase-9-real-llm-grounding-validation.md) — 验证设计（draft）
- [`../planning/phase-9-implementation-plan.md`](../planning/phase-9-implementation-plan.md) — 实施计划（draft）

## 14. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-18 | 初始 draft：Provider 架构设计，覆盖 Provider 抽象、调用链、配置存储、网络开关、timeout/retry、context 构造、response validation、审计、fallback。`status: draft`，未接入真实 LLM，编码未开始。 | Claude |
