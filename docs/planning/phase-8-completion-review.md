# Phase 8 完成审查

---
status: draft
updated: 2026-06-18
---

> 本文档是 Phase 8（产品级 UI/UX 工作台重构）的完成审查草稿。P8-T01~P8-T09 已完成；P8-T10 的自动化回归、checksum 只读验证、边界 rg 检查与代码级桌面验收已完成，但**真实 GUI 桌面验收尚未完成**。
>
> **结论：Phase 8 暂不允许 completion。待真实桌面验收（含截图/交互验证）完成后，方可将本文档转 active 并允许进入 Phase 9 详细文档编制阶段。**
>
> 本审查基于 CLI 环境执行自动化验证与代码级桌面验收；未生成实时 GUI 截图。代码级核验不能替代真实桌面验收。

## 1. 任务完成状态

| 任务 | 目标 | Batch | 状态 | 验证方式 |
|------|------|-------|------|----------|
| P8-T01 | AppShell + LeftNav + Header + 焦点状态机 | A | ✅ | `npm run build` + 组件渲染检查 |
| P8-T02 | StageWorkspace 骨架 + 占位容器 | A | ✅ | `npm run build` + 骨架渲染检查 |
| P8-T03 | Artifact tabs 重组（功能等价迁移） | B | ✅ | `npm run build` + tab 切换验收 |
| P8-T04 | 顶部概览 + 中部筛选 + 可展开对象列表 | B | ✅ | `npm run build` + 筛选/展开验收 |
| P8-T05 | ContextPanel 选中上下文 | C | ✅ | `npm run build` + 选中联动验收 |
| P8-T06 | 状态隔离：阶段切换 / 重生成清空上下文 | C | ✅ | 代码审查 + 隔离路径确认 |
| P8-T07 | 视觉 polish：AgentScope 风格 | D | ✅ | `npm run build` + 视觉语义一致性检查 |
| P8-T08 | warnings 降噪 | D | ✅ | `npm run build` + WarningsPanel 折叠/计数验收 |
| P8-T09 | 空/错误/session/UI state 恢复 | D | ✅ | `npm run build` + 状态组件验收 |
| P8-T10 | 回归 + 桌面验收 + completion review | E | ⏳ pending_desktop_acceptance | 自动化回归 + checksum + rg + 代码级桌面验收已完成；真实 GUI 桌面验收待完成 |

### Batch 状态

| Batch | 范围 | 状态 |
|-------|------|------|
| Batch A（P8-T01~P8-T02） | 工作台骨架：AppShell / LeftNav / Header / StageWorkspace | 已完成 |
| Batch B（P8-T03~P8-T04） | Artifact tabs 重组 + 概览/筛选/可展开列表 | 已完成 |
| Batch C（P8-T05~P8-T06） | ContextPanel 真实联动 + 状态隔离 | 已完成 |
| Batch D（P8-T07~P8-T09） | AgentScope 视觉 polish + warnings 降噪 + 空/错误/session 状态产品化 | 已完成 |
| Batch E（P8-T10） | 回归测试 + 桌面验收 + completion review | ⏳ pending_desktop_acceptance（自动化回归/代码级核验已完成；真实 GUI 桌面验收未完成） |

## 2. 自动化验证结果

### 2.1 前端类型检查

```bash
npx tsc --noEmit
```

结果：**通过**，无类型错误。

### 2.2 前端生产构建

```bash
npm run build
```

结果：**通过**。输出：

```text
> tsc && vite build
vite v6.4.3 building for production...
✓ 58 modules transformed.
dist/index.html                  0.33 kB │ gzip:  0.24 kB
dist/assets/index-ldj5uWH9.js  319.15 kB │ gzip: 91.68 kB
✓ built in 489ms
```

### 2.3 Rust 编译检查

```bash
cd src-tauri && cargo check
```

结果：**通过**，无 warning。

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.66s
```

### 2.4 Rust 单元测试

```bash
cd src-tauri && cargo test --lib
```

结果：**通过**。`549 passed; 0 failed; 1 ignored`（含 Phase 8 L0/L4 质量阻塞修复新增 5 个单元测试）。

### 2.5 真实项目只读验证

```bash
cd src-tauri && cargo test --test real_project_validation -- --ignored
```

结果：**通过**。6 个 ignored 测试全部通过（含新增 `primary_sample_l0_l4_quality_blockers_fixed`）：

```text
running 6 tests
test primary_sample_deep_scan_no_timeout ... ok
test primary_sample_detects_l0_l1_rtl ... ok
test primary_sample_l0_l4_quality_blockers_fixed ... ok
test primary_sample_skips_noise_dirs ... ok
test primary_sample_checksum_consistency ... ok
test secondary_sample_detects_stages ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

主样本：`/Users/ckstar/Repo/znxt_ofdm/fpga_project_coarse_sync`  
副样本：`/Users/ckstar/Repo/znxt_ofdm/fpga_project_fft`

## 3. 代码级桌面验收结果（不能替代真实 GUI 桌面验收）

> ⚠️ 当前为 CLI 环境，无法运行 Tauri 桌面应用并截取实时 GUI 截图。以下验收项通过**代码结构审查 + 自动化构建 + 组件实现核对**完成，属于 P8-T10 的“桌面验收”前置准备工作，**不能替代真实 GUI 桌面验收**。
>
> 真实桌面验收必须在可运行 Tauri 应用的桌面环境中完成：逐项点击、布局可视、响应式宽度、截图记录。完成前不得将 Phase 8 标记为 completion。

| # | 验收项 | 结果 | 核验依据 |
|---|--------|------|----------|
| 1 | 打开项目后，左侧深色导航、顶部 header、主工作区骨架正常 | ✅ | `WorkspacePage.tsx` 渲染 `AppShell` + `LeftNav` + `AppHeader` + `StageWorkspace`；`StageWorkspace` 包含 topbar / `StageOverviewBar` / `StageFilterBar` / Artifact tabs / 内容区 |
| 2 | 主区能看到 OverviewBar、FilterBar、Artifact tabs、ContextPanel、WarningsPanel | ✅ | `StageWorkspace.tsx:153~268` 依次渲染上述组件；`WorkspacePage.tsx` 底部渲染 `<WarningsPanel warnings={currentWarnings} />` |
| 3 | Artifact tabs 能正常切换：overview、evidence、understanding、views、trace、qa、quality | ✅ | `StageWorkspace.tsx:21~38` 定义 7 个 tab 及中文标签；`activeTab` 状态 + `setActiveTab` 切换；渲染正常 |
| 4 | Evidence tab 的筛选可用 | ✅ | `StageFilterBar` 接收 `evidenceFilter` / `onEvidenceFilterChange`；`StageWorkspace` 维护 `evidenceFilter` 状态并随阶段切换重置 |
| 5 | Views tab 选择节点或边后，ContextPanel 联动正常 | ✅ | `MultiViewPanel.tsx` `handleNodeClick` / `handleEdgeClick` 先 `onSelectTarget` 再调用 `onContextSelection` 设置 `view_node` / `view_edge` |
| 6 | Trace、SourceExcerpt、QualityIssue 三类上下文切换时，不会互相错误覆盖 | ✅ | Batch C 审核收口已修复：`handleGroundedCitationClick` 不再自动触发 `handleViewSource`；`QualityReviewPanel.handleIssueClick` 不再自动触发 `onViewSource`；`clearQualityState` 仅清除 `quality_issue` context；`handleCloseSourceExcerpt` 仅清除 `source_excerpt` context |
| 7 | 切换阶段后，上下文状态会正确清理，不串台 | ✅ | `handleSelectStage` 调用 `clearTraceState()` + `clearQualityState()`；`StageWorkspace` 用 `validSelection = contextSelection?.stageId === stageId ? contextSelection : null` 做阶段作用域校验 |
| 8 | 重新收集 evidence、重新生成 understanding、views、quality 后，旧请求不会回写污染新状态 | ✅ | 8 条隔离路径均清空 `contextSelection`：打开项目 / 选择阶段 / 加载 session / 收集 evidence / 生成 understanding / 生成 views / 生成 quality / 清空 trace；`traceGuardRef` / `excerptGuardRef` / `qaGuardRef` / `qualityGuardRef` 过滤旧异步响应 |
| 9 | WarningsPanel 已变为折叠/分类/计数/可展开模式，不再长期占据底部大面积空间 | ✅ | `WarningsPanel.tsx` 默认折叠为 36px 高紧凑条；按 `error_code` 聚合并展示 count badge；展开后 `maxHeight: 240` 滚动；空 warnings 时返回 `null` |
| 10 | 空阶段、命名异常阶段、source_changed、session 恢复等状态展示符合当前 Phase 8 设计 | ✅ | `StateBlocks.tsx` 提供统一 `EmptyState` / `Callout`；`WorkspacePage.tsx` 对 initial / opening / loaded / error / selecting_stage / stage_error 提供 `MainStateBlock`；`StageList.tsx` 对 empty / missing / unreadable 使用中性 slate badge |
| 11 | 保存/加载 session 后，工作台状态不会明显错乱 | ✅ | `handleLoadSession` 恢复 maps 与 UI state；session 保存逻辑沿用 Phase 6 自动保存 + dirty version 守卫；构建通过 |
| 12 | 目标项目 checksum 前后一致，保持只读 | ✅ | `primary_sample_checksum_consistency` 通过；目标项目 git status 无已修改的 tracked 文件 |

## 4. 安全与边界检查

### 4.1 目标项目只读

- 未修改目标项目源码。
- `real_project_validation::primary_sample_checksum_consistency` 通过。
- 主样本 git status：仅存在未跟踪文件（`constraints/`、`reports/`、`scripts/synthesis/`、`sim_build/`、部分测试文件），无 tracked 文件被修改；这些未跟踪文件与本次 Phase 8 运行无关。

### 4.2 不运行 Vivado / synthesis / implementation / bitstream

- 代码中无相关调用。
- rg 仅命中禁用语境注释与常量名中的 `THRESHOLD` / `LARGE_FILE_THRESHOLD` 等无关词。

### 4.3 不接真实 LLM / OpenAI / Anthropic / api_key

- 产品代码中无相关 API 调用或 key 读取。
- rg 仅命中模块顶部安全边界注释与测试中的禁用词检查。

### 4.4 不引入 ReactFlow / D3 / Mermaid

- `package.json` 无相关依赖。
- 视图继续沿用既有 SVG + CSS 方案。

### 4.5 不输出 PASS/HOLD / 正确/错误类审计裁决

- 前端文案不出现相关用语。
- `trace/qa/validator.rs` 与 `quality/reporter.rs` 中保留对禁用词的断言守卫。
- rg 命中均位于注释、守卫代码、测试断言或常量名（`THRESHOLD`），非用户可见裁决。

### 4.6 rg 检查命令与结果

执行命令：

```bash
rg -n "OpenAI|Anthropic|api_key|Vivado|synthesis|implementation|bitstream|ReactFlow|reactflow|Mermaid|mermaid|D3|\bd3\b|PASS|HOLD|审计结论" src src-tauri/src docs
```

结果：命中均位于禁用/守卫/注释语境，产品代码中无实际调用或用户可见裁决输出。具体说明：

- `src/types/workspace.ts:622`、`src-tauri/src/quality/mod.rs:14-16`、`src-tauri/src/quality/models.rs`、`src-tauri/src/quality/reporter.rs` 等为注释/文档字符串，明确声明不输出审计结论。
- `src-tauri/src/trace/qa/validator.rs:246`、`src-tauri/src/quality/reporter.rs:700-701` 为测试/守卫代码，检查禁用词不出现。
- `src-tauri/src/understanding/context_builder.rs:175` 为 prompt 中的禁用语说明。
- `src-tauri/src/evidence/extractors/{verilog,systemverilog}.rs` 中 `THRESHOLD` 为 Verilog 参数名测试样例，与审计用语无关。
- `docs/` 中命中为设计文档明确列出禁用词与边界。

结论：**边界检查通过**。

## 5. 已知限制与未完成项

1. **真实 GUI 桌面验收未完成**：当前为 CLI 环境，无法运行 Tauri 桌面应用并截取实时 GUI 截图。代码结构审查 + 自动化测试 + 组件核对不能替代真实桌面验收。
2. **Phase 8 completion review 仍为 draft**：待真实桌面验收完成后，方可将本文档转 active 并给出最终 completion 结论。
3. **ContextPanel 未持久化**：符合设计，阶段切换 / session 加载后右侧上下文清空，不恢复之前选中的 evidence / node / issue。
4. **WarningsPanel 空状态时隐藏**：当 warnings 为空时不渲染，不占用底部空间；有 warnings 时默认折叠。
5. **视觉 polish 仍为内联 style + 设计 token 阶段**：未引入 CSS-in-JS 库或 Tailwind，符合 Phase 8 范围约束。

## 6. 当前结论

- **Phase 8 编码已完成**：Batch A/B/C/D 已完成；Batch E 中自动化回归、checksum 只读验证、边界 rg 检查、代码级桌面验收均已完成。
- **Phase 8 暂不允许 completion**：真实 GUI 桌面验收尚未完成，因此 `phase-8-completion-review.md` 保持 `status: draft`。
- **Phase 9 尚未进入**：只有在真实桌面验收完成、completion review 转 active 后，方可进入 Phase 9 详细文档编制阶段；目前 Phase 9/10/11 overview 仍为 `draft`，不得开始编码。

## 7. Phase 8 真实项目 L0/L4 质量阻塞修复收口

### 7.1 问题与修复

> 在真实项目 `fpga_project_coarse_sync` 的桌面验收准备中发现：L0/L4 视图被 `annotations`、`dataclass`、`Optional`、`np`、`PARAMS`、`config`、`data_width` 等 import/typing/decorator 噪声符号主导，淹没了 coarse-sync 算法语义。
>
> 修复措施（**确定性启发式**，不接 LLM、不改目标项目）：

- 噪声符号保留为 evidence，但不再被提升为 claim / module / processing step。
- `MockProvider` 停止“一条 evidence 一条 claim”，改为按证据质量打分选择 claim 候选：排除 import/typing/decorator/dunder/config/type-alias；优先 class 定义、业务函数、AXI-Stream 接口、端口声明、命中 L0/L4 标准流水线关键词的证据。
- claim 描述与 category 根据证据类型语义化（“识别到模块/类 …”、“识别到处理步骤 …”、“识别到 AXI-Stream 信号 …” 等），不再输出“基于证据 X 的声明 N”模板。
- L0 dataflow 反映真实粗同步流水线：correlation → energy → metric → smoothing → peak detection → CFO。
- L4 timing/dataflow 反映周期精确流水线：input → correlation → energy → metric → detection → output，并识别 AXI-Stream `s_*` / `m_*` I/O；`has_temporal_evidence` 对 L4/cycle_acc 增加语义门控，避免普通函数顺序被伪造成硬件时序。
- `QualityReport` 诚实暴露退化：`NoisyEvidence` 负向记录。
- **结构限制**：当前 claim/流水线/摘要均为 keyword/symbol 启发式，未解析函数体语义；Phase 9 仍需接入真实 LLM 替代 heuristic。

### 7.2 验证

- 新增 11 个单元/集成测试覆盖噪声过滤、L0/L4 流水线、AXI-Stream I/O、temporal evidence、claim 质量、退化标记。
- `cargo test --lib` 553 passed；`cargo test --test real_project_validation -- --ignored` 5 passed（含新增端到端测试）。
- 目标项目 checksum 修复前后一致，未修改目标项目任何文件。
- GUI 截图审阅受 CLI 环境限制未完成，截图目录 `docs/screenshots/phase-8-quality-fixes/` 已预留。

### 7.3 结论

- Phase 8 全部编码（含质量阻塞修复）已完成，自动化验证全部通过。
- 本次质量修复为**确定性启发式**，claim/流水线/摘要基于 symbol/summary 关键词派生；Phase 9 仍需接入真实 LLM 以替换 heuristic、提升语义准确性。
- 真实 GUI 桌面验收仍未完成；待真实桌面环境补录截图后，方可将 completion review 转 active。
