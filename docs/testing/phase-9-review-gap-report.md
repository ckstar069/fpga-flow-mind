# Phase 9 详细文档正式审核记录

---
status: active
updated: 2026-06-18
---

> 本文档是 Phase 9 6 份详细文档的**正式审核记录**。`status: active`，与 6 份详细文档一同审核通过并转 `active`。**Phase 9 Batch A 编码已完成并审核收口**（Provider 抽象、配置模型、Fake/Mock transport、no-network-by-default 守卫与测试）；**Phase 9 Batch B 编码已完成并进入审核收口**（RequestBuilder / ResponseParser / 可注入 Transport / RealLlmProvider 骨架）。未接入真实 LLM，未发起真实网络调用。Batch C/D/E 尚未开始。Phase 10/11 尚未开始。
>
> 审核原则：不做代码实现，不接真实 LLM；审核期间未转 active，审核完成后统一转 active。

## 1. 审核结论总览

| 级别 | 数量 | 说明 |
|------|------|------|
| **BLOCKER** | 0 | 无必须修复后才能转 active 的阻塞项 |
| **IMPORTANT** | 3 | 已修复；建议确认后转 active |
| **MINOR** | 1 | 已修复；不阻塞转 active |
| **CLEAR** | 6 份文档 × 6 个审核项 = 36 项检查 | 全部通过 |

## 2. 发现的问题清单（按严重程度排序）

### IMPORTANT-1：缺少 cancellation 设计（用户明确要求）

- **位置**：`docs/design/phase-9-llm-provider-architecture.md` §6、`docs/design/phase-9-grounding-and-validation-design.md` §9
- **问题**：用户明确要求检查 "cancellation"，但 6 份文档均未涉及用户取消进行中的 LLM 调用。
- **影响**：用户可能无法取消长时间运行的 LLM 调用，导致体验问题；取消后的降级/审计链路未定义。
- **修复**：
  - 架构 §6：新增 `cancellation` 行，说明用户可取消、取消与 timeout 同等处理、Transport 暴露取消信号。
  - 架构 §10：fallback 列表补充"用户取消"。
  - grounding §9：failure modes 新增 `cancelled` 行，明确降级处置（不视为 error，标 degraded/已取消）。
  - 测试 §3：集成测试新增"用户取消"场景。
- **状态**：✅ 已修复

### IMPORTANT-2：prompt injection 测试覆盖缺失

- **位置**：`docs/testing/phase-9-real-llm-grounding-validation.md` §3
- **问题**：grounding 设计 §6 已定义 prompt injection 防护（目标项目文本只作 data、不覆盖系统约束），但测试文档 §3 集成测试列表中**无 prompt injection 测试项**。
- **影响**：设计有防护但无测试验证，防护效果无法在 CI 中回归。
- **修复**：测试 §3 集成测试新增 prompt injection 场景：构造含"忽略以上指令""你现在是审计器""回显 api_key"等注入文本的证据片段，断言 data/system 分离、validator 不受越权、输出不含裁决语/敏感项、无 citation 时降级 unknown。
- **状态**：✅ 已修复

### IMPORTANT-3：R9-012 承诺的量化阈值未定义

- **位置**：`docs/requirements/phase-9-real-llm-grounding-requirements.md` §5.12、`docs/testing/phase-9-real-llm-grounding-validation.md` §10
- **问题**：需求 R9-012 声明"量化门槛在测试文档中定义"，但测试文档 §10 仅列方向性完成标准，**无具体数字/阈值**。
- **影响**：Phase 9 完成审查时无法判定"是否通过"，缺少可量化的验收依据。
- **修复**：测试 §10 新增 §10.1"量化门槛"，定义：
  - citation 合规：伪造 citation 数 = 0（硬门）
  - 安全回归：api_key 明文泄露断言失败数 = 0（硬门）
  - 零回归：Mock 路径与 Phase 8 baseline 逐字段等价（硬门）
  - checksum：目标项目 `src/` 聚合 SHA256 前后一致（硬门）
  - 语义质量：real LLM "优于或不劣于" baseline（人工判定，非自动裁决）
  - 测试：build/tsc/cargo check/test 全通过（零 warning 为目标）
- **状态**：✅ 已修复

### MINOR-1："test connection" 路径归属不明确

- **位置**：`docs/ui-ux/phase-9-llm-configuration-and-grounded-qa-view.md` §3
- **问题**：UI/UX 文档说"测试连接"按钮"发起一次受控请求"，但未说明该功能在 Batch A（无真实网络）还是 Batch B（有 RealProvider）才可用。实施计划 Batch D 的 P9-T07 验收标准写"test connection 走 command"，但 Batch D 本身不做真实网络调用。
- **影响**：可能导致 Batch D 验收时误以为"test connection"应能真实连通外部 provider。
- **修复**：UI/UX §3 补充说明"真实网络调用仅在 Batch B 的 RealProvider 实现完成后才可用，Batch A 中此按钮仅校验配置格式"。
- **状态**：✅ 已修复

## 3. 6 个重点审核项逐项结论

### 3.1 Provider 架构 ✅ CLEAR

| 检查项 | 结论 | 依据 |
|--------|------|------|
| MockProvider 仍为默认 | ✅ | 需求 R9-001、架构 §2.2、UI/UX §2、实施计划 §4.1 |
| RealProvider 必须显式配置 | ✅ | 需求 R9-002、架构 §5、实施计划 §4.2 |
| API key 安全读取 | ✅ | 需求 R9-003、架构 §4、UI/UX §3/§8、实施计划 P9-T02 |
| timeout/retry/rate limit | ✅ | 架构 §6（已补充 cancellation） |
| degraded fallback | ✅ | 需求 R9-007、架构 §10、grounding §9、测试 §3 |
| capability/status/error 一致性 | ✅ | 架构 §2.1、UI/UX §2、测试 §2/§3 |

### 3.2 Grounding 与 citation ✅ CLEAR

| 检查项 | 结论 | 依据 |
|--------|------|------|
| 输入只来自受控 evidence | ✅ | 需求 R9-004、架构 §7、grounding §2/§5 |
| 输出必须经过 citation validation | ✅ | 需求 R9-005、架构 §8、grounding §4 |
| citation 追溯到 evidence_id/source_path/line_range | ✅ | grounding §3、需求 §6 |
| 无 citation 降级 unknown | ✅ | 需求 R9-005/R9-006、grounding §4/§7 |
| 禁止 LLM 输出写入目标项目 | ✅ | 需求 R9-010 §8、架构 §12 |

### 3.3 Prompt / context 安全 ✅ CLEAR

| 检查项 | 结论 | 依据 |
|--------|------|------|
| 上下文大小限制 | ✅ | 架构 §7（"片段受 context 预算与裁剪策略约束"） |
| 敏感数据过滤 | ✅ | grounding §5、需求 R9-004、测试 §5 |
| prompt injection 防护 | ✅ 设计 / ✅ 测试（已补充） | grounding §6、测试 §3（新增） |
| 禁止记录完整 prompt/response/api_key | ✅ | 需求 R9-008、架构 §9、测试 §5 |

### 3.4 UI/UX ✅ CLEAR

| 检查项 | 结论 | 依据 |
|--------|------|------|
| Provider 状态可见（mock/real/degraded） | ✅ | UI/UX §2（已修正为"未来运行态"） |
| 不写成当前已接入 | ✅ | UI/UX §2（"未来运行态"措辞） |
| 用户能看出 mock/real | ✅ | UI/UX §2/§4/§5 |
| unknown/evidence_gap 清晰表达 | ✅ | UI/UX §4/§6 |
| 延续 Phase 8 工作台 | ✅ | UI/UX §1/§9 |
| 不引入 Phase 10/11 UI | ✅ | 需求 §7 非目标、UI/UX §1 |

### 3.5 Testing ✅ CLEAR

| 检查项 | 结论 | 依据 |
|--------|------|------|
| fake provider / mock transport 覆盖 | ✅ | 测试 §2/§3 |
| 成功/失败/超时/无 citation/伪 citation | ✅ | 测试 §2/§3 |
| 敏感数据拦截 | ✅ | 测试 §3/§5 |
| 真实 LLM smoke 为 `#[ignore]` / opt-in | ✅ | 测试 §4 |
| no-network-by-default 测试 | ✅（已补充） | 测试 §2（新增） |
| api_key redaction 测试 | ✅（已补充） | 测试 §2（新增） |
| 真实项目 L0/L4 回归 | ✅ | 测试 §6/§8 |
| rg 禁止项检查 | ✅ | 测试 §9 |

### 3.6 Implementation plan ✅ CLEAR

| 检查项 | 结论 | 依据 |
|--------|------|------|
| Batch A 只做抽象/配置/fake/守卫 | ✅ | 实施计划 §4.1、P9-T01/P9-T02 |
| Batch B 接 RealProvider + 显式 opt-in | ✅ | 实施计划 §4.2、P9-T03/P9-T04 |
| Batch C 接 grounding validator + citation | ✅ | 实施计划 §4.3、P9-T05/P9-T06 |
| Batch D 接 UI + degraded/unknown | ✅ | 实施计划 §4.4、P9-T07/P9-T08 |
| Batch E 验收 + 可选 smoke + completion | ✅ | 实施计划 §4.5、P9-T09/P9-T10 |
| 每个 Batch 有允许/禁止/测试/退出 | ✅ | 实施计划 §2 各任务表 |

## 4. 修改文件列表

| 文件 | 修改内容 | 严重程度 |
|------|----------|----------|
| `docs/design/phase-9-llm-provider-architecture.md` | §6 新增 `cancellation` 行；§10 fallback 补充"用户取消" | IMPORTANT |
| `docs/design/phase-9-grounding-and-validation-design.md` | §9 failure modes 新增 `cancelled` 行；补充两层枚举不混用说明 | IMPORTANT |
| `docs/testing/phase-9-real-llm-grounding-validation.md` | §2 新增 no-network-by-default + api_key redaction；§3 新增用户取消 + prompt injection；§10 新增 §10.1 量化门槛 | IMPORTANT |
| `docs/ui-ux/phase-9-llm-configuration-and-grounded-qa-view.md` | §3 "测试连接"补充 Batch B 才可用说明 | MINOR |
| `docs/requirements/phase-9-real-llm-grounding-requirements.md` | R9-003 补充 audit 脱敏字段清单引用 | MINOR |
| `docs/testing/phase-9-real-llm-grounding-validation.md` | 本次审核记录文件（新增） | — |

## 5. 关键修复摘要

1. **cancellation**：用户取消进行中的 LLM 调用现在在设计中完整定义（架构 §6/§10、grounding §9、测试 §3），按降级处理（不视为 error，标 degraded/已取消）。
2. **prompt injection 测试**：测试 §3 新增 4 条断言，覆盖注入文本在 data 区隔离、validator 不受越权、输出不含裁决语/敏感项、无 citation 降级 unknown。
3. **量化门槛**：测试 §10.1 定义 6 条可量化门槛，其中 citation 合规、安全回归、零回归、checksum 为硬门（0 容忍），语义质量为人工判定（非自动裁决）。
4. **test connection 归属**：UI/UX §3 明确 Batch A 中仅校验配置格式，真实网络连通性在 Batch B 才可用。
5. **audit 脱敏一致性**：需求 R9-003 补充引用架构 §9 的完整不记录清单（`api_key`、完整 prompt 正文、完整源码片段、用户私密内容）。

## 6. 6 份 Phase 9 详细文档 status 检查结果

| 文档 | status | 结果 |
|------|--------|------|
| `docs/requirements/phase-9-real-llm-grounding-requirements.md` | `draft` | ✅ 未转 active |
| `docs/design/phase-9-llm-provider-architecture.md` | `draft` | ✅ 未转 active |
| `docs/design/phase-9-grounding-and-validation-design.md` | `draft` | ✅ 未转 active |
| `docs/ui-ux/phase-9-llm-configuration-and-grounded-qa-view.md` | `draft` | ✅ 未转 active |
| `docs/testing/phase-9-real-llm-grounding-validation.md` | `draft` | ✅ 未转 active |
| `docs/planning/phase-9-implementation-plan.md` | `draft` | ✅ 未转 active |
| `docs/testing/phase-9-review-gap-report.md`（本文件） | `draft` | ✅ 未转 active |

## 7. rg 检查结果

```bash
# 6 份 Phase 9 详细文档 status
rg -n "^status:" docs/requirements/phase-9-real-llm-grounding-requirements.md docs/design/phase-9-llm-provider-architecture.md docs/design/phase-9-grounding-and-validation-design.md docs/ui-ux/phase-9-llm-configuration-and-grounded-qa-view.md docs/testing/phase-9-real-llm-grounding-validation.md docs/planning/phase-9-implementation-plan.md
# 结果：全部 6 份为 status: draft ✅

# 误导性当前状态表述
rg -n "真实 LLM 已接入|真实 LLM 已启用|Phase 9 编码已开始|允许进入 Phase 9 Batch A|status: active" README.md docs
# 结果：无误导性"已接入/已开始"表述；所有"status: active"均为 Phase 0-8 文档 ✅

# 边界敏感术语
rg -n "api_key|API key|OpenAI|Anthropic|Vivado|synthesis|implementation|bitstream|PASS|HOLD|正确|错误|审计" docs/requirements/phase-9-real-llm-grounding-requirements.md docs/design/phase-9-llm-provider-architecture.md docs/design/phase-9-grounding-and-validation-design.md docs/ui-ux/phase-9-llm-configuration-and-grounded-qa-view.md docs/testing/phase-9-real-llm-grounding-validation.md docs/planning/phase-9-implementation-plan.md
# 结果：所有匹配仅出现在安全边界/禁止/脱敏/测试断言/禁用语境中，无硬编码真实 key、无实际工具链调用、无用户可见裁决 ✅
```

## 8. 安全确认

- 未接入任何真实 LLM / OpenAI / Anthropic / API key。
- 未修改目标项目文件。
- 未运行 Vivado / synthesis / implementation / bitstream。
- 仅修改文档措辞与新增审核记录，零代码变更。

## 9. 是否建议下一步转 active

**建议：本次审核发现的 3 个 IMPORTANT + 1 个 MINOR 已全部修复，6 份详细文档 + 本审核记录可一同进入转 active 流程。**

但转 active 需满足以下前置（由实施计划 §1 定义）：
- 6 份详细文档全部审核通过 → 本次审核已完成；
- 本审核记录（`phase-9-review-gap-report.md`）作为第 7 份配套文档，需一同转 active；
- 转 active 动作本身需由人工确认（非自动），确认后更新各文档 frontmatter `status: draft → active`。

**当前状态：Phase 9 Batch A 编码已完成并审核收口，Phase 9 Batch B 编码已完成并进入审核收口，真实 LLM 尚未接入，真实网络调用尚未启用。Batch C/D/E 尚未开始。Phase 10/11 尚未开始。** 转 active 后已由人工触发并完成 Batch A/B；Batch C 及后续需等待进一步授权。

## 10. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-18 | 初始审核记录：审核 6 份 Phase 9 详细文档，发现 3 IMPORTANT + 1 MINOR，全部修复。`status: draft`。 | Claude |
