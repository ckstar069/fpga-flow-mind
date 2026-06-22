# Phase 9 完成审查

---
status: draft
updated: 2026-06-22
---

> 本文档是 Phase 9（真实 LLM Provider 与 grounding 生产化）的完成审查草案。
>
> **当前结论**：Phase 9 Batch A/B/C/D 已完成并审核收口；Batch E 的自动化回归、真实项目只读 grounding 验收、安全 rg、checksum 验证、DeepSeek OpenAI-compatible 真实 LLM smoke、命令级连接测试 smoke 与生成理解主链路真实 provider smoke 已完成。真实 GUI 桌面验收尚未完成，因此本文档保持 `status: draft`，Phase 9 completion 暂不激活。Phase 10/11 尚未开始。

## 1. 任务完成状态

| 任务 | 目标 | Batch | 当前状态 | 验证方式 |
|------|------|-------|----------|----------|
| P9-T01 | Provider 抽象扩展与配置模型 | A | 已完成 | `cargo test --lib llm::` |
| P9-T02 | 凭据安全包装与 no-network-by-default 守卫 | A | 已完成 | redaction/no-network 单测 |
| P9-T03 | RequestBuilder / ResponseParser / 可注入 Transport | B | 已完成 | fake transport / parser 单测 |
| P9-T04 | RealLlmProvider 接入 + timeout/retry/限流 + error mapping | B | 已完成 | fake transport 单测；真实 smoke ignored |
| P9-T05 | understanding 接入 provider 选择 + 校验 + fallback | C | 已完成 | grounding validator 单测 |
| P9-T06 | Q&A 接入 provider + GroundedQaValidator 强化 + unknown | C | 已完成 | citation / unknown / safety 单测 |
| P9-T07 | Provider 配置入口 UI + 状态可见 + 安全文案 | D | 已完成 | provider status command + type/build |
| P9-T08 | 错误/degraded 状态 UI + 审计可见性 + desktop smoke（fake） | D | 已完成 | provider status command + UI build |
| P9-T09 | 真实项目验收 + 安全回归 + checksum | E | 自动化/只读验收完成 | `real_project_validation --ignored` 6 项 |
| P9-T10 | 回归 + 桌面验收 + Phase 9 完成审查 | E | 待真实 GUI 桌面验收 | 本文档 draft；需截图/人工验收后转 active |

## 2. 自动化验证结果

### 2.1 前端类型检查

```bash
npx tsc --noEmit
```

结果：通过，无类型错误。

### 2.2 前端生产构建

```bash
npm run build
```

结果：通过。

### 2.3 Rust 编译检查

```bash
cd src-tauri && cargo check --tests
```

结果：通过，0 warning。

### 2.4 Rust 单元测试

```bash
cd src-tauri && cargo test --lib
```

当前结果：`686 passed; 0 failed; 3 ignored`。

### 2.5 LLM 模块回归

```bash
cd src-tauri && cargo test --lib llm::
```

当前结果：`121 passed; 0 failed; 2 ignored`。ignored 项为显式真实 LLM smoke，默认测试路径安全跳过。

### 2.6 可选真实 LLM smoke

```bash
cd src-tauri && cargo test --lib real_smoke_deepseek_openai_compatible -- --ignored
```

当前结果：`1 passed; 0 failed`。

说明：

- Provider：DeepSeek OpenAI-compatible endpoint（`https://api.deepseek.com`，model `deepseek-chat`）。
- API key 仅通过环境变量传入，不写入仓库、文档、session 或日志。
- 测试不打印响应正文。
- 默认分析路径仍使用 no-network guard；默认测试路径仍不联网。

同时补充验证配置面板使用的 `test_provider_connection` 命令路径：

```bash
cd src-tauri && cargo test --lib test_connection_deepseek_real_smoke -- --ignored
```

当前结果：`1 passed; 0 failed`。

边界：仅在显式 env/config 下发送最小 ping，不发送项目源码、evidence、Q&A、session 或截图。

### 2.7 Provider status command 回归

```bash
cd src-tauri && cargo test --lib commands::provider_status
```

当前结果：`15 passed; 0 failed`。

## 3. 真实项目只读验收

### 3.1 样本

| 样本 | 路径 | 验收内容 |
|------|------|----------|
| 主样本 | `/Users/ckstar/Repo/znxt_ofdm/fpga_project_coarse_sync` | L0/L4 evidence/understanding/view/quality + Phase 9 grounding safety |
| 副样本 | `/Users/ckstar/Repo/znxt_ofdm/fpga_project_fft` | 阶段识别与深层布局适配 |

### 3.2 命令

```bash
cd src-tauri && cargo test --test real_project_validation -- --ignored
```

当前结果：`6 passed; 0 failed`。

新增 Phase 9 验收项：

- `primary_sample_phase9_grounding_safety_on_real_evidence`

该测试在真实项目 L0/L4 evidence 上验证：

- Fake provider 返回合法 L0 citation 时，`GroundingValidator` 保持 grounded；
- L0 回答引用 L4 evidence 时，stage mismatch 触发降级；
- 引用不存在的 evidence id 时触发降级；
- 验证前后目标项目 `src/` checksum 一致。

### 3.3 真实 GUI 部分验收记录（2026-06-21）

本轮在真实 Tauri 桌面窗口中完成了以下可交互路径，并将截图保存到
`docs/screenshots/phase-9-completion/`：

| 截图 | 内容 |
|------|------|
| `01-app-open.png` | 应用启动，provider footer 默认显示 `Mock · 本地模式 · 未配置` |
| `09-open-project-accessibility.png` | 打开真实项目 `/Users/ckstar/Repo/znxt_ofdm/fpga_project_coarse_sync`，识别 L0~L6/RTL 8 个阶段 |
| `10-l0-selected.png` | 选择 L0，工作台显示文件、证据、声明、视图、Provider 概览 |
| `11-l0-evidence-collected.png` | L0 收集证据完成，显示 244 项 evidence |
| `13-l0-understanding-generated.png` | L0 生成理解完成，显示 provider=`mock` / `Mock` 与 8 条声明 |
| `14-l0-views-generated.png` | L0 生成视图完成，显示 3 个视图，时序流水 tab 标记为空 |

已完成的 GUI 验收点：

- 真实项目打开与阶段识别；
- 默认 provider 状态可见，且未配置真实 provider；
- L0 evidence → understanding → view 工作流可由桌面 UI 触发；
- L0 产物中 provider/model 可见；
- L0 视图生成后，结构/数据流/时序 tab 可见，其中 timing 保持空态标记，不把算法步骤伪造成硬件时序。

### 2.7 生成理解主链路真实 LLM smoke

```bash
cd src-tauri && cargo test --lib commands::generate_understanding::tests::und_11_real_llm_generate_understanding_smoke -- --ignored
```

当前结果：`1 passed; 0 failed`。

说明：

- Provider：DeepSeek OpenAI-compatible endpoint（`https://api.deepseek.com`，model `deepseek-chat`）。
- API key 仅通过环境变量传入，不写入仓库、文档、session、localStorage、目标项目或日志。
- 测试覆盖 `generate_understanding` 主链路：前置 evidence collection → `RealLlmProvider<HttpTransport>` → JSON 提取 → 本地 ID/meta/stats 归一化 → schema validator → `ImplementationUnderstanding`。
- 若真实 provider 失败或输出不满足 schema，产品路径会降级为 mock fallback 并产生 warning；本次 smoke 未发生降级。

仍未完成的 GUI 验收点：

- Provider 配置面板的真实配置、配置校验、清除 api_key；
- UI 层“测试连接”的成功/失败状态截图；
- L4 周期精确 timing 的桌面截图；
- Q&A 真实/Mock grounding 展示、citation 回链、unknown 展示；
- 错误态、断网态、限流态的 degraded UI 截图。

## 4. 安全与边界确认

- 默认配置仍为 no-network-by-default：未启用真实 LLM 时不发起真实网络调用。
- 真实网络调用只允许通过 `#[ignore]` smoke 或人工显式配置触发；默认测试路径不联网。
- API key 不写入 session、localStorage、目标项目、日志、Debug 或序列化输出。
- 目标项目只读；真实项目验收前后 checksum 一致。
- 未运行 Vivado / synthesis / implementation / bitstream。
- 不输出 PASS/HOLD/正确/错误等用户可见审计裁决。
- Phase 10/11 未开始。

## 5. 待完成项

Phase 9 completion 暂不转 active，原因如下：

1. **真实 GUI 桌面验收仅完成部分路径**：已完成应用启动、真实项目打开、L0 evidence/understanding/view 与默认 provider 状态截图；仍需完成 provider 配置面板、配置校验、test connection、L4 timing、Q&A grounding、错误态与断网态截图验收。
2. **真实 LLM 语义质量仍需人工判断**：R9-012 中的“优于或不劣于 heuristic baseline”需要真实 provider 输出后人工确认，不做自动裁决。

## 6. 当前结论

Phase 9 Batch E 的自动化/只读部分、可选 DeepSeek 真实 LLM smoke、命令级连接测试 smoke 与生成理解主链路真实 provider smoke 已完成：P9-T09 可视为自动化验收通过；P9-T10 仍待真实 GUI 桌面验收与最终 completion 激活。当前不允许进入 Phase 10 编码。

## 7. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-21 | 新增 Phase 9 completion review 草案；记录 Batch E 自动化回归、真实项目只读 grounding 验收、安全边界、待真实 GUI 桌面验收项。 | Codex |
| 2026-06-21 | 补充 DeepSeek OpenAI-compatible 真实 LLM smoke 结果：`real_smoke_deepseek_openai_compatible` 在显式 env/config 下通过；completion 仍因真实 GUI 桌面验收待完成而保持 draft。 | Codex |
| 2026-06-21 | 补充真实 GUI 部分验收截图：应用启动、真实项目打开、L0 evidence/understanding/view 工作流已完成；provider 配置、test connection、L4 timing、Q&A grounding、错误/断网态仍待补验。 | Codex |
| 2026-06-21 | 补充配置面板连接测试真实 ping 收口：`test_provider_connection` 在显式 DeepSeek env/config 下通过；默认分析路径仍不联网，真实 GUI 桌面验收仍待完成。 | Codex |
| 2026-06-22 | 补充生成理解主链路真实 provider 接线：`generate_understanding` 在显式 provider config 下可调用 DeepSeek/OpenAI-compatible provider，输出通过本地 schema validator；默认仍 mock/no-network，api_key 仅运行态内存，不持久化；Q&A 主链路仍待后续接真实 provider。 | Codex |
