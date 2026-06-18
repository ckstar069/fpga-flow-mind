# Phase 9 LLM 配置与 Grounded Q&A 视图 UI/UX 设计

---
status: draft
updated: 2026-06-18
---

> 本文档是 Phase 9 的 **UI/UX 设计草案**（Provider 配置入口 + Grounded Q&A / understanding 体验增强）。`status: draft`，尚未审核生效。Phase 9 **编码尚未开始**，**未接入任何真实 LLM**。需与需求 / 架构 / grounding 设计 / 测试 / 实施计划一同审核转 `active` 后方允许编码。
>
> 设计基线：延续 Phase 8 工作台（三段式骨架、Artifact tabs、卡片化、蓝色强调、confidence 视觉语义），**不回退成长页面堆叠**，不膨胀为复杂设置页。

## 1. 设计目标

1. **最小入口**：在现有工作台增加最少的 Provider 配置入口，不做独立复杂设置站。
2. **provider 可见**：始终明确当前 provider = `mock` / `real` / `degraded`。
3. **安全可见**：凭据不明文、可清除、不写目标项目，文案说明。
4. **grounding 可见**：Q&A 显示 citation / unknown / evidence_gap / provider / degraded。
5. **不裁决化**：不把 LLM 输出包装成正确/错误裁决。

## 2. Provider 状态可见性

- 在工作台 header / 阶段概览条 / Q&A 与 understanding 产物上，统一显示 provider 标记：
  - `mock`：未启用真实 LLM（默认）。
  - `real`：真实 LLM 已启用且当前产物来自真实调用。
  - `degraded`：真实 LLM 已启用但本次降级（失败/超时/限流/校验失败，回退 Mock 或 unknown）。
- 视觉沿用 Phase 8 中性色（非红绿裁决色）；`degraded` 用中性琥珀/灰提示，不用红色报错感。

## 3. 配置入口（最小）

- 位置：工作台 header 右侧或设置抽屉（Drawer/Popover），**不新增顶级长页面**。
- 字段：
  - 启用真实 LLM（toggle，默认关）；
  - provider 类别（抽象选项，不内置厂商密钥）；
  - `model`；
  - `base_url`；
  - `api_key`（密码型输入，不明文显示，带"清除"按钮）；
  - "测试连接"按钮（仅在此动作发起一次受控请求）。
- 文案（安全）：
  - "api_key 仅存储于本机 app 数据，不写入目标项目，不上传，可随时清除。"
  - "启用后，仅必要的证据片段会发送给所选 provider。"
  - "默认关闭；关闭时使用本地确定性引擎。"

## 4. Q&A 体验

- 沿用 Phase 5/8 Q&A 分区，增强呈现：
  - 答案区显示 **citations**（可点击回链 evidence + 高亮 `line_range`）；
  - **unknown / evidence_gap**：明确说明"证据不足"，不强行作答；
  - 顶部标记 **provider**（mock/real/degraded）与（real 时）model；
  - degraded 时提示"真实 LLM 不可用，已回退/返回 unknown"。
- 提问框保留；无可用上下文时返回 unknown + 原因。

## 5. understanding 体验

- 沿用 Phase 8 understanding/视图分区，增强元信息：
  - 显示 **provider / model**（real 时）；
  - 显示 **生成时间** 与 **是否 fallback**（degraded 标记）；
  - claims/steps 仍走既有 grounding + trace 回链；
  - L4/L0 流水线与 timing 行为不变（真实 LLM 仅提升 claim/摘要语义，不改变 stage-aware timing 门控）。

## 6. 错误态

| 错误 | UI 表现 |
|------|---------|
| 未配置 | provider=mock，配置入口提示"未启用真实 LLM" |
| 网络失败 | degraded + "网络错误，已回退" |
| 限流 | degraded + "请求被限流，已回退" |
| 超时 | degraded + "请求超时，已回退" |
| 响应格式错误 | 该产物降级 unknown / "响应校验失败" |
| citation 校验失败 | 该回答降级 unknown / "引用校验失败" |

- 错误态用中性表达，不渲染为"正确/错误"裁决；不暴露原始 LLM 原文作为结论。

## 7. 不裁决化文案规范

- 不出现 PASS/HOLD / 正确/错误 / 审计结论作为用户可见结论。
- 用"识别到 / 推断 / 证据不足（unknown）/ 证据冲突（conflicting）"等中性表述。
- LLM 原文不直接作为裁决；必须经 grounding 校验 + 结构化呈现。

## 8. 前后端边界

- 配置读写、测试连接、understanding/Q&A 调用走 Tauri command（沿用 Phase 8 command 结构）。
- `api_key` 由 Rust 侧持有/脱敏，前端只显示掩码（如 `••••••`）与"已设置/未设置"。
- 真实网络调用仅在 Rust 单一可审计入口发起；前端不直接 fetch 外部。

## 9. 视觉与交互一致性（延续 Phase 8）

- 沿用 workbenchTheme token、三段式骨架、Artifact tabs、卡片化、蓝色强调。
- 配置入口为抽屉/Popover，非顶级页面；不增加页面层级。
- 状态色中性（mock 灰 / real 蓝 / degraded 琥珀），非红绿裁决。

## 10. 安全边界

- `api_key` 不明文显示、不写目标项目、不进日志/session 明文，可清除。
- 测试连接为显式动作，不自动发起；默认 disabled。
- 不引入 cloud-first 依赖；外部调用经 Rust Provider 抽象。

## 11. 关联文档

- [`../requirements/phase-9-real-llm-grounding-requirements.md`](../requirements/phase-9-real-llm-grounding-requirements.md) — 需求（draft）
- [`../design/phase-9-llm-provider-architecture.md`](../design/phase-9-llm-provider-architecture.md) — Provider 架构（draft）
- [`../design/phase-9-grounding-and-validation-design.md`](../design/phase-9-grounding-and-validation-design.md) — grounding 设计（draft）
- [`../testing/phase-9-real-llm-grounding-validation.md`](../testing/phase-9-real-llm-grounding-validation.md) — 验证设计（draft）
- [`../planning/phase-9-implementation-plan.md`](../planning/phase-9-implementation-plan.md) — 实施计划（draft）

## 12. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-18 | 初始 draft：Provider 配置最小入口、provider 状态可见、配置字段与安全文案、Q&A/understanding 体验、错误态、不裁决化、延续 Phase 8 视觉。`status: draft`，未接入真实 LLM，编码未开始。 | Claude |
