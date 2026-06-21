# Phase 9 完成审查

---
status: draft
updated: 2026-06-21
---

> 本文档是 Phase 9（真实 LLM Provider 与 grounding 生产化）的完成审查草案。
>
> **当前结论**：Phase 9 Batch A/B/C/D 已完成并审核收口；Batch E 的自动化回归、真实项目只读 grounding 验收、安全 rg 与 checksum 验证已完成。真实 GUI 桌面验收与可选真实 LLM smoke 尚未完成，因此本文档保持 `status: draft`，Phase 9 completion 暂不激活。Phase 10/11 尚未开始。

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

当前结果：`686 passed; 0 failed; 2 ignored`。

### 2.5 LLM 模块回归

```bash
cd src-tauri && cargo test --lib llm::
```

当前结果：`121 passed; 0 failed; 1 ignored`。ignored 项为显式真实 LLM smoke，未设置 env 时安全跳过。

### 2.6 Provider status command 回归

```bash
cd src-tauri && cargo test --lib commands::provider_status
```

当前结果：`11 passed; 0 failed`。

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

1. **真实 GUI 桌面验收尚未完成**：需要在桌面应用中完成 provider 配置面板、状态条、配置校验、test connection、理解/Q&A grounding 状态、错误态与断网态的截图验收。
2. **可选真实 LLM smoke 尚未执行**：如用户提供显式配置和 API key，可运行 ignored smoke；若不执行，应在最终 completion 中记录为“未执行可选 real smoke，不阻塞默认安全路径”。
3. **真实 LLM 语义质量仍需人工判断**：R9-012 中的“优于或不劣于 heuristic baseline”需要真实 provider 输出后人工确认，不做自动裁决。

## 6. 当前结论

Phase 9 Batch E 的自动化/只读部分已完成：P9-T09 可视为自动化验收通过；P9-T10 仍待真实 GUI 桌面验收与最终 completion 激活。当前不允许进入 Phase 10 编码。

## 7. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-21 | 新增 Phase 9 completion review 草案；记录 Batch E 自动化回归、真实项目只读 grounding 验收、安全边界、待真实 GUI 桌面验收项。 | Codex |
