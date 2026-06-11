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

当前阶段先建立项目需求、边界和 MVP 技术路线，再进入实现。

首个可用闭环不追求“大而全”，而是优先支持：

- 打开一个真实业务项目；
- 选择一个阶段；
- 读取 Python/Verilog/docs/tests/config 相关上下文；
- 生成结构图、数据流图、时序/流水图等理解视图；
- 所有主要结论可回链到源码证据；
- 支持用户继续追问具体节点、公式、信号和映射来源。
