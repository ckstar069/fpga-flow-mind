# Phase 9 真实 LLM 与 grounding 验证设计

---
status: draft
updated: 2026-06-18
---

> 本文档是 Phase 9 的 **验证与验收设计草案**。`status: draft`，尚未审核生效。Phase 9 **编码尚未开始**，**未接入任何真实 LLM**。需与需求 / 架构 / grounding 设计 / UI/UX / 实施计划一同审核转 `active` 后方允许编码。
>
> 核心原则：**默认测试路径不发真实网络请求**；真实 LLM 仅作为 `#[ignore]` 可选 smoke test，需显式 env/config 才运行，不进 CI 默认路径。

## 1. 验证策略总览

| 层 | 方式 | 目的 |
|----|------|------|
| 单元 | config 校验、request redaction、error mapping、schema/citation 校验 | 各组件契约正确 |
| 集成 | MockProvider / fake local provider / mock transport；invalid response / missing citation / unknown fallback | 调用链与降级正确 |
| 安全回归 | api_key 不进日志/目标项目/session 明文；prompt 不含 .git/env/secrets | 凭据与数据安全 |
| 真实项目验收 | `fpga_project_coarse_sync` L0/L4，Mock/heuristic vs real LLM 质量对比（可选） | 语义质量提升、grounding 守住 |
| 桌面验收 | 配置 provider、测试连接、生成理解、Q&A citation、unknown、断网/错误态 | 真实可用 + 状态可见 |
| 回归 | Phase 8 UI/workbench 零退化；目标项目 checksum 一致 | 不破坏既有 |

## 2. 单元测试

- **config 校验**：`ProviderConfig` 合法性（enable 时 provider/model/base_url/api_key 完备；不全则视为未启用）。
- **request builder redaction**：构造的 payload 不含 `api_key`/env/`.git`/大二进制；只含证据片段 + 摘要。
- **provider error mapping**：网络/超时/429/解析错误 → 对应 `ProviderError` 变体。
- **schema validation**：LLM 结构化输出缺字段 / 引用未知 `evidence_id` → 失败。
- **citation validation**：citation `evidence_id` 不存在 / `line_range` 越界 → 失败；非 unknown 回答无 citation → 失败。
- **default-disabled 断言**：未启用时选择 Mock，不进入真实调用入口。

## 3. 集成测试（mock transport / fake provider）

- **fake local provider**：注入返回固定结构化 JSON 的 fake provider，验证 understanding/Q&A 全链路成功 + grounding 通过。
- **invalid response**：fake 返回非法 JSON / 缺字段 → `InvalidResponse` → 降级或 unknown。
- **missing citation**：fake 返回非 unknown 但无/非法 citation → grounding 拒绝 → unknown。
- **unknown fallback**：fake 返回 unknown → 正确呈现 unknown + 原因。
- **network/timeout/rate-limited**：fake transport 注入失败 → 降级 Mock/heuristic + 标 degraded + 审计 error_code。
- **redaction 集成**：发往 fake transport 的 payload 断言无敏感项。

## 4. 可选真实 LLM smoke test（默认 ignored）

- 标记 `#[ignore]`，需显式 env（如 `FPGA_FLOW_LLM_SMOKE=1`）+ 完整 config 才运行。
- **不进 CI 默认路径**；本地手工运行。
- 用最小证据集验证一次真实 understanding + 一次 Q&A，断言 grounding 通过（citation 合法）。
- 失败不阻塞主测试套件。

## 5. 安全测试

- `api_key` 不进日志（断言日志/audit 记录无明文 key）；
- 不进目标项目（目标项目 git status 无改动）；
- 不进 persisted session 明文（session 序列化无明文 key）；
- prompt 不包含 `.git` / env / secrets / 大二进制（redacted payload 断言）。
- 审计记录字段断言：含 provider/model/timestamp/token_estimate/error_code；**不含** api_key/完整 prompt。

## 6. 真实项目验收（`fpga_project_coarse_sync`）

- L0 / L4：
  - Mock/heuristic baseline 与 real LLM（可选启用）产出对比；
  - 语义丰富度（claim/摘要质量）**优于或至少不劣于** baseline；
  - grounding 守住：real LLM 产出经校验，无伪造 citation；证据不足字段保持 unknown/empty_reason。
- 不启用真实 LLM 时，行为与 Phase 8 baseline 一致（零回归）。
- 目标项目 checksum 前后一致（`src/` 聚合 SHA256 不变）。

## 7. 桌面验收

1. 默认 provider=mock，配置入口提示未启用；
2. 启用真实 LLM（显式配置），"测试连接"成功；
3. L0/L4 生成理解，显示 provider/model，claims 走 grounding + trace；
4. Q&A 提问，显示 citation（可回链）、unknown（证据不足）；
5. 断网 / 错误响应 / 限流场景 → degraded 标记 + 回退/unknown；
6. 清除 api_key → 状态回到未配置；
7. 全程目标项目只读，无 PASS/HOLD/正确/错误裁决文案。

## 8. 既有能力回归（零退化）

- Phase 8 工作台 UI/状态隔离/视觉不退化；
- MockProvider 路径产出不变；
- L0 timing 空 + empty_reason、L4 timing 周期精确流水等 Phase 8 质量修复行为不变；
- `cargo test --lib` / `cargo test --test real_project_validation -- --ignored` 通过。

## 9. 禁止项 rg

```bash
# OpenAI/Anthropic/api_key 只能出现在 provider/config 安全路径与测试中；不得硬编码真实 key
rg -n "OpenAI|Anthropic|api_key" src src-tauri/src docs
# Vivado/synthesis/implementation/bitstream 仅禁用语境
rg -n "Vivado|synthesis|implementation|bitstream" src src-tauri/src
# PASS/HOLD/正确/错误/审计结论 仅守卫/注释/测试语境
rg -n "PASS|HOLD|正确|错误|审计结论" src src-tauri/src
```

- 期望：上述词仅出现在 Provider 抽象/配置安全路径/禁用语境守卫/测试断言中；**无硬编码真实 key**、无实际工具链调用、无用户可见裁决。

## 10. Phase 9 完成标准（方向性）

- 真实 LLM 默认关闭，显式启用可用；
- grounding 守住不胡说 / 不伪造 citation（单测 + 集成 + 可选真实 smoke）；
- 安全回归通过（凭据不泄露、无完整源码外发、redacted payload 断言）；
- 不启用真实 LLM 时零回归；
- 真实项目验收 + 桌面验收（含可选真实 LLM 路径）通过；
- 全量构建/测试通过。

## 11. 安全边界汇总

- 目标项目只读，checksum 一致；
- 不运行 Vivado/synthesis/implementation/bitstream；
- 不调用真实 LLM 除非显式启用；凭据不落日志/session/目标项目；
- 默认测试路径不发真实网络请求；真实调用仅 `#[ignore]` smoke；
- 不输出 PASS/HOLD/正确/错误/审计结论裁决。

## 12. 关联文档

- [`../requirements/phase-9-real-llm-grounding-requirements.md`](../requirements/phase-9-real-llm-grounding-requirements.md) — 需求（draft）
- [`../design/phase-9-llm-provider-architecture.md`](../design/phase-9-llm-provider-architecture.md) — Provider 架构（draft）
- [`../design/phase-9-grounding-and-validation-design.md`](../design/phase-9-grounding-and-validation-design.md) — grounding 设计（draft）
- [`../ui-ux/phase-9-llm-configuration-and-grounded-qa-view.md`](../ui-ux/phase-9-llm-configuration-and-grounded-qa-view.md) — UI/UX（draft）
- [`../planning/phase-9-implementation-plan.md`](../planning/phase-9-implementation-plan.md) — 实施计划（draft）

## 13. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-18 | 初始 draft：单元/集成/安全/真实项目/桌面/回归验证设计，可选真实 smoke 默认 ignored，禁止项 rg。`status: draft`，未接入真实 LLM，编码未开始。 | Claude |
