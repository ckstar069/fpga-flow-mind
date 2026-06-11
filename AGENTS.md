# AGENTS.md

本文件是 `fpga-flow-mind` 的项目记忆与工作约束入口。

开发/审核协作策略单独写在：

- `docs/initial-requirements-draft/DEVELOPMENT_WORKFLOW.md`

## 1. 项目身份

`fpga-flow-mind` 是一个本地桌面端 FPGA 阶段实现理解与可视化 Agent。

它面向通过 `ai_project_template` 创建、并由 AI Agent 从 L0 到 RTL 分阶段推进
的业务项目。当前核心是帮助用户理解和可视化实现，而不是替用户判断对错。

## 2. 必读文件

任何设计、规划或实施前，必须阅读：

1. **`docs/README.md`** ← 正式文档体系入口，含各目录用途与推荐阅读路径
2. `docs/initial-requirements-draft/PROJECT_BRIEF.md`
3. `docs/initial-requirements-draft/MVP_ARCHITECTURE.md`
4. `AGENTS.md`
5. `docs/initial-requirements-draft/DEVELOPMENT_WORKFLOW.md`
6. 用户指定的补充任务说明

## 3. 语言要求

默认使用简体中文。

适用范围：

- 项目文档内容；
- 需求讨论；
- 实施说明；
- 审核意见；
- 最终回报；
- 面向用户的默认界面文案。

代码标识符、文件名、命令、第三方 API 名称和必要英文术语可以保留英文，但解释性内容
应使用简体中文。

## 4. 开发节奏

必须遵循：

```text
需求澄清
  -> 设计
  -> 规划
  -> 实施
  -> 验证
  -> 回报
```

不要从模糊想法直接跳到大量实现。

## 5. 实施方向规则

每个任务至少应推进以下之一：

- workspace understanding
- source evidence
- stage understanding
- dataflow understanding
- timing understanding
- semantic claims
- evidence traceability
- uncertainty expression
- grounded Q&A
- local desktop usability

如果一个任务只是在增加 UI 面板、抽象层或内部结构，但没有提升项目理解能力，应先提
出质疑。

## 6. 技术约束

- 主产品 UI：Tauri + React/TypeScript
- Backend/runtime/safety：Rust
- 大模型/Agent：主语义理解引擎
- 静态分析：证据抽取、索引、约束和 grounding 辅助

明确禁止：

- 不使用 Python 实现产品核心
- 不使用 PySide6
- 不做 Web GUI 主路线
- 不做 server-first / cloud-first 架构
- 不把产品做成 JSON artifact viewer

## 7. 目标项目安全边界

分析业务项目时：

- 目标项目只读
- 不修改 `fpga_project_*`
- 不运行 Vivado
- 不运行 synthesis / implementation / bitstream
- 不默认运行目标项目脚本
- 不把输出写回目标项目目录
- 优先使用临时目录或 app-owned 目录

## 8. 证据规则

用户可见的主要结论必须：

- 绑定 evidence id；
- 可追溯到源码文件和行号范围；
- 区分 confirmed / inferred / unknown；
- 在证据不足时明确标注不确定，而不是强行解释。

图应来自结构化理解对象，不应只是自由文本拼接。

## 9. 审核关注点

每次回报都应自查：

### 方向

- 是否仍然面向 `ai_project_template` 生态业务项目？
- 是否提升了阶段实现理解能力？
- 是否保持“大模型主导、静态分析辅助”？
- 是否避免变成审计器、检查器或 viewer？

### 架构

- UI 是否仍在 Tauri + React/TypeScript？
- Backend 是否仍在 Rust？
- 是否引入了不必要的大依赖或错误主路线？

### 安全

- 是否保持目标项目只读？
- 是否避免 Vivado / synthesis / implementation / bitstream？
- 是否避免隐式外部调用和敏感信息泄露？

### 用户价值

- 用户是否能真正多理解一点东西？
- 是否新增了结构图、数据流图、时序图、证据追踪或追问能力？
- 是否减少困惑，而不是只增加更多内部面板？

## 10. 实施纪律

- 小步、可验证、可回滚
- 不做无关改动
- 不提前实现未来复杂能力
- 先把单阶段 MVP 闭环做实
- 不脱离文档边界扩张需求

## 11. 项目推进阶段定义

`fpga-flow-mind` 使用 **Phase 0 ~ Phase 6** 作为自身的开发推进阶段。每个 Phase 有明确的目标和验收标准，与 FPGA 业务项目的阶段命名无关。

| 阶段 | 名称 | 核心目标 |
|------|------|----------|
| Phase 0 | 文档体系与需求契约 | 建立稳定文档体系，明确需求边界和验收标准 |
| Phase 1 | Workspace 扫描与阶段识别 | 能够打开业务项目，识别阶段目录结构 |
| Phase 2 | 证据索引与 evidence model | 建立证据抽取、索引和存储能力 |
| Phase 3 | 单阶段结构化理解产物 | 生成结构化的 `ImplementationUnderstanding` |
| Phase 4 | 三类视图展示 | 在前端展示结构图、数据流图、时序/流水图 |
| Phase 5 | 证据回链与 grounded Q&A | 支持节点点击追溯证据、用户追问 |
| Phase 6 | 持久化、回放与 MVP 验收 | 产物可持久化加载，完成 MVP 闭环验收 |

**必须区分两套概念**：

- **Phase 0 / Phase 1 / ... / Phase 6** — `fpga-flow-mind` **本项目**的开发推进阶段。
- **L0 / L1 / L2 / ... / RTL** — 被分析的 **FPGA 业务项目**的实现阶段（由 `ai_project_template` 创建）。

两者不是同一套概念，不能混用。当文档或任务中出现 `Phase`，应默认理解为 `fpga-flow-mind` 的开发阶段；当出现 `L0` / `L1` / `RTL`，应默认理解为目标业务项目的实现阶段。

详细阶段计划和验收标准以以下文档为准：

- `docs/planning/README.md`
- `docs/planning/phase-0-exit-criteria.md`

AGENTS.md 只保存稳定记忆，不承载详细计划。

## 12. GitHub 远程与提交要求

- GitHub 远程名固定为 `github`
- 远程地址为 `git@github.com:ckstar069/fpga-flow-mind.git`
- 每次修改完成并验证通过后，必须执行：

  ```text
  git status
  git add ...
  git commit -m "..."
  git push github main
  ```

- 如果当前任务明确要求不提交或不推送，则以用户当轮指令为准

## 12. 最终回报格式

每次任务结束后，至少回报：

1. 修改文件列表
2. 实现摘要
3. 测试命令和结果
4. 手工验证结果
5. 安全确认
6. 已知限制
7. 下一步建议
