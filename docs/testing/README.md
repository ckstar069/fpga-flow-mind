# 测试与验收文档索引

---
status: active
updated: 2026-06-11
---

## Testing 目录用途

本目录存放 `fpga-flow-mind` 的测试策略、验收标准和 QA 清单。测试文档描述"如何验证产品满足需求、如何发现回归问题、如何判定可发布"，不描述具体功能需求。

**核心理念**：测试不是为了证明实现正确，而是验证工具是否帮助用户理解项目且没有越界。

## 建议文档类型

| 文档类型 | 说明 | 示例 |
|----------|------|------|
| `validation strategy` | 验证策略 | 测试层级、方法、覆盖范围 |
| `MVP acceptance` | MVP 验收标准 | MVP 完整闭环的验收条件 |
| `manual QA checklist` | 手工 QA 清单 | 每次发布前必须手工验证的项目 |
| `safety regression checklist` | 安全回归清单 | 确保目标项目只读等安全约束未被破坏 |

## 测试关注点

### 1. 是否保持目标项目只读

- 测试过程中不应修改任何 `fpga_project_*` 目录下的文件
- 不应在目标项目目录中创建新文件
- 所有写入应发生在 app-owned 或临时目录

### 2. 是否避免 Vivado / synthesis / implementation / bitstream

- 验证产品不会自动或隐式调用 Vivado
- 验证不会触发 synthesis、implementation 或 bitstream 生成

### 3. Evidence 是否可追溯

- 图中节点和解释是否绑定了 evidence id
- 是否可追溯到源码文件和行号范围
- evidence 是否真实存在（非伪造）

### 4. unknown / inferred / conflicting 是否正确展示

- 不确定性是否被显式标注
- 用户是否能区分 confirmed、inferred 和 unknown
- conflicting 证据是否被提示而非自动裁决

### 5. 三类视图是否来自结构化理解对象

- 结构图、数据流图、时序/流水图是否基于 `ImplementationUnderstanding` 生成
- 不是自由文本拼接或静态模板填充

### 6. 用户是否能通过图和证据更好理解阶段实现

- 用户是否能在不通读全部代码的情况下理解阶段做了什么
- 图中是否有明确的输入、处理、中间结果、输出
- 图不是一堆孤立方块

## 当前文档列表

| 文档 | 状态 | 说明 | 推荐阅读时机 |
|------|------|------|-------------|
| [`phase-1-workspace-scanning-validation.md`](phase-1-workspace-scanning-validation.md) | `active` | Phase 1 workspace 扫描与阶段识别验证设计：样例矩阵、后端/前端/安全验证点、手工验证清单、自动化测试规划、验收标准 | Phase 1 验证与验收依据 |
| [`phase-2-evidence-validation.md`](phase-2-evidence-validation.md) | `active` | Phase 2 evidence collection 验证设计：测试夹具、Rust 单元/集成测试矩阵、前端组件测试、安全回归测试、手工验收 10 步骤、Phase 1 样例复用 | Phase 2 验证与验收依据 |
| [`phase-3-understanding-validation.md`](phase-3-understanding-validation.md) | `active` | Phase 3 理解生成验证设计：数据模型 serde、context builder、schema validator、evidence_id 检查、claim 约束、unknown/gap 处理、mock provider pipeline、前端渲染、安全回归、手工验收 10 步骤 | Phase 3 验证与验收依据 |
| [`phase-4-view-validation.md`](phase-4-view-validation.md) | `active` | Phase 4 视图验证设计：后端 ViewGraph 生成测试（~33 个）、前端渲染验证、hover tooltip、空状态/degraded、安全回归、桌面验收 8 步骤 | Phase 4 验证与验收依据 |
| [`phase-5-trace-and-qa-validation.md`](phase-5-trace-and-qa-validation.md) | `active` | Phase 5 证据回链与 Grounded Q&A 验证设计：TraceResolver/SourceExcerptResolver/Q&A 测试矩阵、安全回归、桌面验收 10 步骤、完成标准 | Phase 5 验证与验收依据 |
