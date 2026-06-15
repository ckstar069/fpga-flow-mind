# Phase 9 Overview：真实 LLM Provider 与 grounding 生产化

---
status: draft
updated: 2026-06-15
---

> 本文档是 Phase 9 的**方向性 overview**。详细需求、设计、安全设计、测试设计与编码计划在进入 Phase 9 编码前另行编制并审核为 active。Phase 9 当前**未开始编码**，也**未接入任何真实 LLM**。

## 1. 背景与问题

`fpga-flow-mind` 的产品定位要求"大模型为主分析者、静态分析为证据基础设施"（`PROJECT_BRIEF` §4、`AGENTS.md` §6）。但截至 MVP，语义引擎一直是 **MockProvider**：

- Phase 3 understanding 生成使用 MockProvider（确定性规则），不调用 LLM；
- Phase 5 Grounded Q&A 使用 MockProvider（关键词匹配）。

MockProvider 验证了**数据流与 grounding 框架**成立，但它不是真正的语义理解引擎——它无法做主算法路径识别、Python 概念到 RTL 的语义归纳、文档与代码不一致时的解释（这些都是 `PROJECT_BRIEF` §4 列出"静态分析很难独立回答"的问题）。

Phase 7 会用真实项目暴露 MockProvider 的能力边界，产出"基线缺口清单"。Phase 9 的命题是：**在显式配置、可关闭、可验证、可审计的前提下，接入真实 LLM Provider 作为语义引擎，并用 grounding 守住"不胡说、不伪造 citation"。**

这是整个 Post-MVP 中**安全敏感度最高**的阶段：它首次引入外部网络调用、API key、源码片段外发。因此安全边界必须在详细设计阶段前置，而非事后补救。

## 2. 阶段目标

1. **可配置的 Provider 层**：在 MockProvider 之外引入真实 LLM Provider，作为可插拔、**默认关闭**、**显式启用**的选项，保持 Provider 抽象不变。
2. **安全的 provider 配置**：显式配置入口（provider / model / endpoint / 凭据），凭据存储与传输安全，不写入目标项目、不落入日志、不随 session 泄露。
3. **受控的上下文构造**：只发送为回答当前问题/生成当前 understanding 所需的**证据片段**，**不上传完整源码**；遵守 context 限制与裁剪策略。
4. **Grounding 生产化**：强化 `GroundedQaValidator` 与 understanding 生成后的 grounding 校验——所有 LLM 产出的 claim 必须能回链到已收集 evidence，citation 不可伪造，证据不足时强制 `unknown`。
5. **失败/超时/限流可恢复**：真实 LLM 调用会失败、超时、被限流；系统必须优雅降级（回退到 MockProvider 或返回 unknown），不崩溃、不残留半成品。
6. **可审计**：真实 LLM 调用（provider、是否启用、调用次数、是否被 grounding 拒绝）可在 app-owned 存储中留痕，供用户自查，但不记录敏感请求体。

## 3. 用户价值

- 用户获得真正的语义理解能力（主算法路径、跨概念归纳、文档/代码不一致解释），而非关键词匹配；
- 用户可以信任结论——因为 grounding 强制每个 LLM 结论回链证据，伪造 citation 的回答会被拒绝或降级为 unknown；
- 用户对"是否启用真实 LLM、用哪个 provider、调用了多少次"有完全的控制与可见性（显式配置、可关闭、可审计）；
- 用户的源码不会被打包外发，只有必要证据片段离开本地。

## 4. 允许范围

Phase 9 允许做（具体范围在详细文档中收敛）：

- 新增真实 LLM Provider 实现（符合既有 Provider 抽象）；
- 新增 provider 配置入口、凭据安全存储与传输；
- 新增受控上下文构造（证据片段选取、context 裁剪）；
- 强化 `GroundedQaValidator` 与 understanding 的 grounding 校验；
- 新增失败/超时/限流的降级与重试策略；
- 新增调用审计留痕（app-owned，脱敏）；
- 配套测试（含 Mock 化的外部调用测试，真实调用仅在显式配置下进行）。

## 5. 明确非目标

Phase 9 **不做**：

- **不默认调用真实 LLM**：默认仍为 MockProvider 或关闭，真实 LLM 仅在用户显式配置启用时生效；
- **不上传完整源码**：只发送证据片段，不做"整项目丢给 LLM"；
- **不绕过 grounding**：任何 LLM 产出都必须经过 grounding 校验，不存在"信任 LLM 原文输出"的旁路；
- **不做 server-first / cloud-first 架构**：仍是本地桌面优先，LLM 仅作为本地发起的可控外部调用；
- **不引入云端优先的数据存储 / 账号体系 / 计费**；
- **不做跨阶段映射**（Phase 10）与语义记忆（Phase 11）——Phase 9 只把单阶段语义引擎做实；
- **不修改目标业务项目**，不运行工具链；
- **不把 API key 或敏感请求体写入日志 / session / 目标项目**。

## 6. 与前后阶段关系

- **前置**：Phase 7（真实项目质量基线 + MockProvider 缺口清单）。grounding 必须在**真实、有噪声**的证据上验证，而非 toy 样例；Phase 7 的质量基线是 Phase 9 的前提。详见 [`post-mvp-roadmap.md`](post-mvp-roadmap.md) §4。
- **后置依赖方**：
  - Phase 10（跨阶段 + Python→RTL）在**实质上依赖真实 LLM**——Python 到 RTL 的语义映射是静态分析无法独立完成的，需要 Phase 9 的真实语义引擎；
  - Phase 8（UI 工作台）的视觉层级会承载 Phase 9 带来的更多 inferred/unknown/citation，二者协同。
- Phase 9 是 Post-MVP 主干（Phase 7 → 9 → 10 → 11）的第二环。

## 7. 未来详细文档清单（进入 Phase 9 编码前编制）

| 文档 | 目录 | 内容方向 |
|------|------|----------|
| 真实 LLM 与 grounding 需求 | `docs/requirements/` | provider 配置、可关闭/显式启用、context 限制、grounding 规则、citation 约束、unknown 规则、失败/超时/限流、审计可见性 |
| 安全与架构设计 | `docs/design/` | provider 抽象、凭据存储/传输安全、证据片段选取与 context 裁剪、GroundedQaValidator 强化、降级/重试、审计留痕脱敏、（必要时）独立安全设计文档 |
| 验证设计 | `docs/testing/` | Mock 化外部调用测试、grounding 拒绝/降级测试、超时/限流测试、安全回归（无 key 泄露、无完整源码外发）、真实调用可选验收 |
| 编码实施计划 | `docs/planning/` | 任务拆解、依赖、Batch 划分（建议先 grounding/降级骨架后真实 provider）、退出条件、安全门禁 |

UI/UX 文档：若需 provider 配置 UI，单独编制 `docs/ui-ux/phase-9-provider-config-view.md`；审计可见性可纳入该文档或实施计划。

## 8. 验收方向（方向性，具体门槛在需求/测试文档中量化）

- 真实 LLM **默认关闭**，仅在显式配置启用时生效；
- 启用后，understanding 与 Q&A 的 claim 经 grounding 校验，伪造 citation 的回答被拒绝或降级为 unknown；
- 证据不足问题返回 unknown，不编造；
- 调用失败/超时/限流时优雅降级，不崩溃、不残留半成品；
- 安全回归通过：API key 不落日志/session/目标项目，无完整源码外发（可由请求构造测试佐证）；
- 调用可审计（次数、provider、是否被 grounding 拒绝），且审计记录脱敏；
- 不启用真实 LLM 时，系统行为与 MVP 一致（MockProvider），无回归；
- 真实桌面验收（含显式启用真实 LLM 的可选路径）通过，安全边界保持，全量构建/测试通过。

## 9. 风险与边界

- **凭据泄露风险**：API key 是最高敏感项。必须前置设计存储与传输安全，禁止任何形式的日志/session 落盘。
- **源码外发风险**：必须只发证据片段，且片段选取受控、可审计；详细设计需论证"不发什么"。
- **grounding 旁路风险**：任何为"让 LLM 答案通过"而放松 grounding 的倾向都必须拒绝——宁可多 unknown，不可伪造 citation。
- **成本/限流风险**：真实 LLM 有成本与速率限制；需有调用预算、裁剪与缓存策略，避免无界调用。
- **可用性风险**：外部依赖不可控，必须保证"LLM 不可用时产品仍可用（降级到 Mock/unknown）"。
- **合规风险**：源码片段外发涉及数据合规，需在详细文档中明确用户知情与同意（显式启用即隐含同意，但需在 UI 文案中说明）。
- **依赖风险**：不引入将架构引向 cloud-first 的依赖；provider SDK 应被 Provider 抽象隔离。

## 10. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-15 | 初始 draft：Phase 9 overview，主题为真实 LLM Provider 与 grounding 生产化，强调显式配置/可关闭/可验证/可审计与安全边界，明确目标/范围/非目标/阶段关系/验收方向。未接入任何真实 LLM，未进入编码。 | Claude |
