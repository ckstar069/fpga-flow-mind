# Phase 9 编码实施计划

---
status: active
updated: 2026-06-18
---

> 本文档是 Phase 9（真实 LLM Provider 与 grounding 生产化）的**编码实施计划**。`status: active`，已审核通过。Phase 9 **Batch A 编码已完成**（Provider 抽象、配置模型、Fake/Mock transport、no-network-by-default 守卫与测试），**未接入任何真实 LLM**，**未发起真实网络调用**。Batch B/C/D/E 尚未开始。
>
> **进入 Phase 9 编码的硬前置**：本文档与需求 / 架构 / grounding 设计 / UI/UX / 测试 5 份详细文档**全部审核通过并转 `active`** 后，方允许进入 Batch A 编码。当前该前置已满足。

## 0. 核心方向（先读）

1. **默认不发真实网络请求**：全程默认 disabled；真实调用仅出现在 Batch E 的可选 `#[ignore]` smoke / 手工验收。
2. **先 grounding/降级骨架，后真实 provider**：Batch A/B 先把 Provider 抽象、凭据安全、redaction、降级骨架做实（用 fake provider / mock transport 验证），Batch C 才把 understanding/Q&A 接入，Batch D 做 UI 与状态可见，Batch E 做真实项目验收。
3. **可注入 Transport**：真实 HTTP 调用封装为可注入 Transport，测试用 fake，避免任何默认路径发真实请求。
4. **heuristic 保留**：Phase 8 确定性派生保留为 fallback / 辅助，不删除。

## 1. 进入条件

| 条件 | 当前状态 |
|------|----------|
| Phase 8 完成审查 active | ✅（Phase 8 已完成） |
| Phase 9 overview 已编制 | ✅ `phase-9-overview-real-llm-grounding.md`（draft） |
| Phase 9 需求文档 active | ⏳ `phase-9-real-llm-grounding-requirements.md`（draft，待审核） |
| Phase 9 架构设计 active | ⏳ `phase-9-llm-provider-architecture.md`（draft，待审核） |
| Phase 9 grounding 设计 active | ⏳ `phase-9-grounding-and-validation-design.md`（draft，待审核） |
| Phase 9 UI/UX 设计 active | ⏳ `phase-9-llm-configuration-and-grounded-qa-view.md`（draft，待审核） |
| Phase 9 验证文档 active | ⏳ `phase-9-real-llm-grounding-validation.md`（draft，待审核） |
| Phase 9 实施计划 active | ⏳ 本文档（draft，待审核） |
| **以上 Phase 9 详细文档全部转 active** | ⏳ 待审核通过 |

> 纪律：Phase 9 详细文档全部审核转 `active` 后，方允许进入 Batch A 编码。审核前不得编写 Phase 9 产品代码。

## 2. 任务拆分

### P9-T01 Provider 抽象扩展与配置模型（Batch A）✅ 已完成

| 项 | 内容 |
|----|------|
| 目标 | 扩展 Provider 抽象：新增 `ProviderConfig` / `ProviderCapability` / `ProviderError`；`RealLlmProvider` 类型骨架（不含真实网络） |
| 输入文档 | 架构 §2~§5、需求 R9-001/R9-002 |
| 允许范围 | `src-tauri/src` Provider/config 模型、单元测试 |
| 禁止范围 | 真实网络调用、写入目标项目、读取 `api_key` 到日志 |
| 输出 | Provider/config/error 类型 + config 校验 |
| 验收标准 | config 合法性校验通过；默认 disabled 断言通过 |
| 必跑测试 | `cargo test --lib`（config 校验、default-disabled） |
| 真实网络调用 | 否 |
| 退出条件 | 模型 + 校验单测通过，0 warning |
| 完成记录 | 实现 `src-tauri/src/llm/{models,provider,mod}.rs`；`ProviderConfig` 默认 `kind=Mock`、`enabled=false`、`network_mode=Disabled`；单测覆盖序列化/校验/default-disabled。 |

### P9-T02 凭据安全包装与 no-network-by-default 守卫（Batch A）✅ 已完成

| 项 | 内容 |
|----|------|
| 目标 | `api_key` 抽象状态包装（不 Display/不 Serialize/Debug 脱敏）；no-network-by-default 守卫；不落日志/session/target |
| 输入文档 | 架构 §4、grounding §5、需求 R9-003 |
| 允许范围 | `ApiKey` 包装、`ProviderConfig` 脱敏字段、network guard、单元测试 |
| 禁止范围 | 写目标项目、日志/session 明文、外发、真实网络调用 |
| 输出 | `ApiKey` 安全包装 + `check_network_allowed` + `network_policy_summary` |
| 验收标准 | `api_key` 不出现在 Debug/序列化/策略摘要中；真实 provider 默认被 `NetworkDisabled` 拦截；Mock/Fake 可本地运行 |
| 必跑测试 | `cargo test --lib`（redaction、network guard、create_provider 默认拦截） |
| 真实网络调用 | 否 |
| 退出条件 | 安全/守卫单测通过 |
| 完成记录 | 实现 `ApiKey`（`Debug` 输出 `[REDACTED]`，不实现 `Display`/`Serialize`，序列化时 `skip_serializing`）；`no_network_guard.rs` 拦截 `NetworkMode::Disabled`；`create_provider` 对 OpenAi/Anthropic 默认返回 `LlmError::NetworkDisabled`，`Allow` 时返回 `LlmError::NotImplemented`。 |
| 说明 | app-owned 持久化存储与可清除接口、完整 redaction 过滤引擎（env/.git/大二进制）留在 Batch B/C 按需扩展。 |

### P9-T03 RequestBuilder / ResponseParser / 可注入 Transport（Batch B）

| 项 | 内容 |
|----|------|
| 目标 | context packing（证据片段 + 摘要 + schema 约束 + redaction）、结构化响应解析、可注入 Transport（fake 实现） |
| 输入文档 | 架构 §3/§7、grounding §2/§6 |
| 允许范围 | request/response 构建、transport 抽象、fake provider、单元/集成测试 |
| 禁止范围 | 真实网络调用、发送完整源码 |
| 输出 | RequestBuilder + ResponseParser + Transport trait + fake transport |
| 验收标准 | fake 全链路成功；payload redacted；context 只含证据片段 |
| 必跑测试 | `cargo test --lib`（builder redaction、parser、fake transport） |
| 真实网络调用 | 否 |
| 退出条件 | 全链路 fake 测试通过 |

### P9-T04 RealLlmProvider 接入 + timeout/retry/限流 + error mapping（Batch B）

| 项 | 内容 |
|----|------|
| 目标 | `RealLlmProvider` 实现（经 Transport），timeout/retry/rate-limit、`ProviderError` 映射；**默认 disabled** |
| 输入文档 | 架构 §5/§6/§10、需求 R9-007 |
| 允许范围 | provider 实现、错误映射、fake transport 注入测试 |
| 禁止范围 | 默认发起真实请求、硬编码真实 key |
| 输出 | RealLlmProvider + 降级/fallback 骨架 |
| 验收标准 | network/timeout/rate-limited/invalid 均正确映射 + 降级 Mock/unknown + 标 degraded |
| 必跑测试 | `cargo test --lib`（error mapping、降级、default-disabled） |
| 真实网络调用 | 否（仅 fake transport） |
| 退出条件 | 错误/降级单测通过 |

### P9-T05 understanding 接入 provider 选择 + 校验 + fallback（Batch C）

| 项 | 内容 |
|----|------|
| 目标 | understanding 生成按 config 选择 Mock/Real；schema validation + grounding 校验；失败 fallback |
| 输入文档 | 架构 §8/§10、grounding §4/§8、需求 R9-005/R9-009 |
| 允许范围 | understanding provider 接线、validator 复用强化、fake provider 测试 |
| 禁止范围 | 绕过 grounding、伪造 citation、默认真实调用 |
| 输出 | understanding 走 provider 选择 + 校验 + fallback |
| 验收标准 | fake 成功产物 grounding 通过；伪造 evidence_id 被拒；失败 fallback 标 degraded |
| 必跑测试 | `cargo test --lib`（understanding grounding、fallback） |
| 真实网络调用 | 否（fake provider） |
| 退出条件 | understanding 校验/fallback 测试通过 |

### P9-T06 Q&A 接入 provider + GroundedQaValidator 强化 + unknown（Batch C）

| 项 | 内容 |
|----|------|
| 目标 | Q&A 按 config 选择 provider；citation 存在性/越界校验；无 citation 非 unknown 回答拒绝；unknown 回退 |
| 输入文档 | 架构 §8、grounding §3/§4/§8、需求 R9-005/R9-006 |
| 允许范围 | Q&A provider 接线、validator 强化、fake provider 测试 |
| 禁止范围 | 信任原文旁路、伪造 citation |
| 输出 | Q&A grounding 守门 + unknown 规则 |
| 验收标准 | 合法 citation 通过；缺/越界 citation → unknown；证据不足 → unknown |
| 必跑测试 | `cargo test --lib`（citation 校验、unknown fallback） |
| 真实网络调用 | 否（fake provider） |
| 退出条件 | Q&A grounding 测试通过 |

### P9-T07 Provider 配置入口 UI + 状态可见 + 安全文案（Batch D）

| 项 | 内容 |
|----|------|
| 目标 | 配置抽屉/Popover（enable/provider/model/base_url/api_key/test connection）、provider=mock/real/degraded 标记、安全文案、api_key 掩码 |
| 输入文档 | UI/UX §2~§5/§8、需求 R9-002/R9-003 |
| 允许范围 | 前端配置入口、状态标记、文案 |
| 禁止范围 | api_key 明文、新增长页面、cloud-first 依赖 |
| 输出 | 配置入口 + 状态可见 + 安全文案 |
| 验收标准 | 默认 mock；配置可存可清；api_key 掩码；test connection 走 command |
| 必跑测试 | `npx tsc --noEmit` / `npm run build` |
| 真实网络调用 | 否 |
| 退出条件 | build/tsc 通过，配置入口可用 |

### P9-T08 错误/degraded 状态 UI + 审计可见性 + desktop smoke（Batch D）

| 项 | 内容 |
|----|------|
| 目标 | network/timeout/rate-limited/invalid/citation_failed 错误态 UI；审计脱敏可见（provider/model/time/error_code，无 key）；desktop smoke（fake provider） |
| 输入文档 | UI/UX §6/§7、架构 §9、需求 R9-008 |
| 允许范围 | 错误态 UI、审计展示、fake provider smoke |
| 禁止范围 | 暴露原文裁决、明文 key、真实网络 |
| 输出 | 错误/degraded 态 + 审计可见 |
| 验收标准 | 各错误态中性呈现；审计无 key；smoke 走 fake |
| 必跑测试 | `npx tsc --noEmit` / `npm run build` / `cargo test --lib` |
| 真实网络调用 | 否（fake smoke） |
| 退出条件 | 错误态 + 审计 UI 通过 |

### P9-T09 真实项目验收 + 安全回归 + checksum（Batch E）

| 项 | 内容 |
|----|------|
| 目标 | `fpga_project_coarse_sync` L0/L4：Mock/heuristic vs real（可选）质量对比；安全回归；checksum 一致 |
| 输入文档 | 测试 §5/§6/§9、需求 R9-011/R9-012 |
| 允许范围 | 真实项目只读验收、`#[ignore]` 可选 real smoke（显式 config）、安全 rg |
| 禁止范围 | 修改目标项目、默认真实调用、伪造 citation |
| 输出 | 真实项目验收记录 + 安全回归 + checksum |
| 验收标准 | grounding 守住；不启用 real 时零回归；checksum 一致；redaction 断言通过 |
| 必跑测试 | `cargo test --lib` / `cargo test --test real_project_validation -- --ignored` |
| 真实网络调用 | 仅可选 `#[ignore]` smoke（显式 config） |
| 退出条件 | 验收 + 安全回归通过 |

### P9-T10 回归 + 桌面验收 + Phase 9 完成审查（Batch E）

| 项 | 内容 |
|----|------|
| 目标 | 全量回归 + 桌面验收（配置/test connection/理解/Q&A citation/unknown/断网错误态）+ 编制 `phase-9-completion-review.md` |
| 输入文档 | 测试 §7/§10、UI/UX §2~§7 |
| 允许范围 | 回归、桌面验收（fake + 可选 real）、完成审查文档 |
| 禁止范围 | 默认真实调用、裁决化文案、修改目标项目 |
| 输出 | `phase-9-completion-review.md`（草案） |
| 验收标准 | Phase 8 零退化；安全边界保持；全量构建/测试通过 |
| 必跑测试 | tsc/build/cargo check --tests/cargo test --lib/real_project_validation --ignored |
| 真实网络调用 | 仅桌面手工可选（显式 config） |
| 退出条件 | 桌面验收通过 → 完成审查转 active（由真实桌面验收决定） |

## 3. 依赖关系

```text
P9-T01 (Provider/config 模型)
  -> P9-T02 (凭据安全/redaction)
  -> P9-T03 (RequestBuilder/Transport/fake)
  -> P9-T04 (RealLlmProvider + 错误/降级)
  -> P9-T05 (understanding 接入) + P9-T06 (Q&A 接入)
  -> P9-T07 (配置 UI) -> P9-T08 (错误/审计 UI)
  -> P9-T09 (真实项目验收) -> P9-T10 (回归/桌面/完成审查)
```

## 4. Batch 划分

### 4.1 Batch A：Provider 模型 + 凭据安全/redaction 骨架

- 任务：P9-T01、P9-T02
- 真实网络调用：**否**
- 退出：config 校验 + redaction + 安全存储单测通过

### 4.2 Batch B：RealLlmProvider pipeline + fake provider 测试（默认 disabled）

- 任务：P9-T03、P9-T04
- 真实网络调用：**否**（仅 fake transport）
- 退出：全链路 fake + 错误/降级单测通过

### 4.3 Batch C：understanding/Q&A 接入真实 provider + validator/fallback（默认 mock）

- 任务：P9-T05、P9-T06
- 真实网络调用：**否**（fake provider）
- 退出：grounding 校验 + unknown/fallback 测试通过

### 4.4 Batch D：配置 UI + 错误/degraded 状态 + desktop smoke

- 任务：P9-T07、P9-T08
- 真实网络调用：**否**（fake smoke）
- 退出：配置入口 + 状态/审计 UI + build/tsc 通过

### 4.5 Batch E：真实项目验收 + 完成审查

- 任务：P9-T09、P9-T10
- 真实网络调用：**仅可选 `#[ignore]` smoke / 手工**（显式 config）
- 退出：真实项目验收 + 安全回归 + checksum + 桌面验收通过 → 完成审查转 active

## 5. 退出条件

- 真实 LLM 默认关闭，显式启用可用；
- grounding 守住不胡说 / 不伪造 citation（单测 + 集成 + 可选 smoke）；
- 安全回归通过（凭据不泄露、redacted payload、无完整源码外发）；
- 不启用真实 LLM 时零回归（Phase 8 行为不变）；
- 真实项目验收 + 桌面验收通过；
- 全量构建/测试通过；
- `phase-9-completion-review.md` 转 active（由真实桌面验收决定）。

## 6. 安全边界（权威，各 Batch 不重复列举）

- 目标项目只读，checksum 一致；
- 不运行 Vivado / synthesis / implementation / bitstream；
- 不默认调用真实 LLM；真实调用仅 Batch E 可选 smoke（显式 config + `#[ignore]`）；
- 凭据不落日志 / session / 目标项目 / 审计明文；
- 持久化只写 app-owned storage；
- 不输出 PASS/HOLD/正确/错误/审计结论裁决；
- 不引入 cloud-first 依赖；默认测试路径不发真实网络请求。

## 7. 进入 Phase 10 的条件（预留）

- Phase 9 完成审查转 active；
- 真实 LLM 已可作为语义引擎（显式启用）；
- Phase 10（跨阶段 + Python→RTL）overview 仍为 draft，需先编制详细文档并转 active。

## 8. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-18 | 初始 draft：Phase 9 编码实施计划，Batch A~E（P9-T01~P9-T10），含允许/禁止范围、输入输出、验收、必跑测试、真实网络调用开关、退出条件；明确 6 份详细文档转 active 前不得编码。`status: draft`，Phase 9 编码尚未开始，未接入真实 LLM。 | Claude |
