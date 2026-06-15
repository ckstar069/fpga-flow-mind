# fpga-flow-mind

`fpga-flow-mind` 是一个本地桌面端 FPGA 阶段实现理解与可视化 Agent。

它面向通过 `/Users/ckstar/Repo/znxt_ofdm/ai_project_template` 创建、并从
`/Users/ckstar/Repo/znxt_ofdm/urban_wireless` 导入模块后，由 AI Agent 从
L0/L1/L2/... 一直推进到 RTL/硬件实现阶段的 FPGA 业务项目。

它的目标不是替用户判断代码对错，而是帮助用户快速理解某个阶段实际实现了什么，
并把源码证据约束的流程图、数据流图、模块关系、公式/信号说明、阶段关系和不确
定项展示出来。

## 本项目是什么

- 一个 Tauri + Rust + React/TypeScript 的本地桌面应用。
- 一个面向 AI Agent 开发流程的 FPGA 项目理解工具。
- 一个以大模型为主分析者、静态分析为辅助证据工具的 Agent 产品。
- 一个以“读懂、建图、解释、可追溯、可追问”为第一阶段目标的系统。

## 本项目不是什么

- 不是通用静态分析器。
- 不是审计 dashboard。
- 不是 PASS/HOLD 决策工具。
- 不是 Markdown 报告生成器。
- 不是 JSON artifact viewer。
- 不是会修改目标 FPGA 项目的自动化工具。
- 不是替用户裁决正确/错误的审查系统。

## 首批必读文档

1. **`docs/README.md`** ← 正式文档体系入口，含各目录用途与推荐阅读路径
2. `docs/initial-requirements-draft/PROJECT_BRIEF.md`
3. `docs/initial-requirements-draft/MVP_ARCHITECTURE.md`
4. `AGENTS.md`
5. `docs/initial-requirements-draft/DEVELOPMENT_WORKFLOW.md`

> `docs/README.md` 是正式文档体系的总索引，涵盖需求、UI/UX、设计、计划、测试等子目录的导航与阅读指引。新增任务应先查阅该文档确定上下文。

## 当前阶段

**MVP / Phase 0–6 已完成（tag `v0.1.0-mvp`，2026-06-15）。**

首个可用闭环已实现并完成桌面验收（基于自包含验收样例项目）：

- 打开并分析自包含验收样例项目，完成桌面闭环；
- 选择一个阶段；
- 读取 Python/Verilog/docs/tests/config 相关上下文；
- 生成结构图、数据流图、时序/流水图等理解视图；
- 所有主要结论可回链到源码证据；
- 支持用户继续追问具体节点、公式、信号和映射来源；
- session 可保存、加载、从最近项目恢复，目标项目只读。

> 说明：桌面验收基于自包含样例项目；**真实 `ai_project_template` 业务项目的可用性验证留给 Phase 7**，本阶段不声称已在真实复杂业务项目上验证通过。

完整发布说明见 [`docs/planning/mvp-release-notes.md`](docs/planning/mvp-release-notes.md)，Phase 6 完成审查见 [`docs/planning/phase-6-completion-review.md`](docs/planning/phase-6-completion-review.md)。

## 下一步（Post-MVP）

MVP 是**技术闭环** MVP，不等于产品可用性完成。后续围绕真实项目分析质量、产品级 UI、真实 LLM grounding、跨阶段理解与语义记忆的方向，已建立总体路线图与各阶段 overview。

- 总体路线图：[`docs/planning/post-mvp-roadmap.md`](docs/planning/post-mvp-roadmap.md)

**当前状态**：Phase 7 详细文档已 `active`，Phase 7 Batch A（models + reporter）与 Batch B（后端 evaluator）已进入实现与审核收口；Phase 7 Batch C/D/E 尚未授权/尚未开始。Phase 8~11 overview 仍为 `draft`。进入任一阶段编码前，需先编制并审核该阶段详细文档为 active。本阶段及后续阶段仍禁止真实 LLM 默认接入、禁止目标项目写入、禁止输出 PASS/HOLD 等审计裁决。

## 本地运行

```bash
npm install
npm run tauri dev
```

构建与测试：

```bash
npm run build
cd src-tauri && cargo test --lib
cd src-tauri && cargo check
```

## 安全边界

- 目标项目只读：不修改目标 FPGA 项目源码。
- 不运行 Vivado / synthesis / implementation / bitstream。
- 不调用真实 LLM API（当前为 MockProvider）。
- 持久化只写 app-owned storage。
- 不输出 PASS/HOLD/正确/错误等审计结论。
