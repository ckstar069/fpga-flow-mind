# Story: 收集证据

---
status: draft
updated: 2026-06-11
---

## 用户目标

作为 FPGA 开发人员，我希望系统自动从选中阶段的源码、测试、文档和配置中抽取证据，以便后续理解有可靠的源码支撑。

## 业务背景

所有语义结论必须有源码证据支撑。证据收集是从原始文件到结构化 evidence item 的转换过程，是后续大模型理解和证据回链的基础。

## 前置条件

- 用户已选中一个阶段
- 阶段上下文已准备完毕（文件列表已生成）

## 触发入口

用户在阶段概览面板中点击"开始分析"或"收集证据"按钮。

## 主流程

1. 用户触发证据收集
2. 系统读取阶段相关文件
3. 系统提取 evidence item
4. 系统建立 evidence 索引
5. 系统展示收集结果

## 功能点清单

### EV-001 触发证据收集

- **用户动作**：用户点击"开始分析"或"收集证据"按钮
- **系统必须**：
  - 显示收集进度（文件读取进度、证据提取进度）
  - 允许用户取消（中断收集过程）
  - 收集过程中保持界面响应
- **成功结果**：证据收集流程启动
- **失败表现**：按钮点击无响应时提示"请稍后再试"
- **evidence 要求**：不需要
- **MVP 必须**：是

### EV-002 读取允许的文件类型

- **系统动作**：读取阶段上下文中列出的文件
- **系统必须**：
  - 读取 Python 文件（`.py`）
  - 读取 Verilog/SystemVerilog 文件（`.v`、`.sv`、`.vh`）
  - 读取文档文件（`.md`、`.rst`、`.txt`）
  - 读取测试文件（`test_*.py`、`*_tb.v`、`*_test.v`）
  - 读取配置文件（`.json`、`.yaml`、`.yml`、`.toml`）
  - 跳过非文本文件（二进制文件、图片、编译产物等）
- **成功结果**：文件内容被读取到内存
- **失败表现**：单个文件读取失败时记录错误，继续处理其他文件
- **evidence 要求**：每个被读取的文件记录 source path
- **MVP 必须**：是

### EV-003 文件过大处理

- **系统动作**：当遇到超过大小阈值的文件时处理
- **系统必须**：
  - 设定文件大小上限（建议 5MB）
  - 超过上限的文件仅读取前 N 行（如前 1000 行），并标注"文件过大，已截断"
  - 不超过上限的文件完整读取
- **成功结果**：大文件被截断读取，标注截断状态
- **失败表现**：无法读取时记录错误
- **evidence 要求**：截断标注应包含 source path 和实际行数
- **MVP 必须**：是

### EV-004 二进制文件处理

- **系统动作**：识别并跳过二进制文件
- **系统必须**：
  - 通过文件扩展名或内容检测识别二进制文件
  - 跳过二进制文件，不尝试读取其内容
  - 在收集结果中标注"跳过 X 个二进制文件"
- **成功结果**：二进制文件被安全跳过
- **失败表现**：误识别为二进制时可能导致有效文件被跳过（可接受的风险，MVP 不追求完美）
- **evidence 要求**：不需要
- **MVP 必须**：是

### EV-005 不可读文件处理

- **系统动作**：处理因权限或其他原因无法读取的文件
- **系统必须**：
  - 记录不可读文件的路径和原因
  - 继续处理其他可读文件
  - 在收集结果中标注"X 个文件无法读取"
- **成功结果**：不可读文件被记录并跳过，不影响整体收集
- **失败表现**：无
- **evidence 要求**：不可读文件记录应包含 source path
- **MVP 必须**：是

### EV-006 提取 evidence item（Python）

- **系统动作**：从 Python 文件中提取 evidence item
- **系统必须**：
  - 识别函数定义（`def` 语句）及行号范围
  - 识别类定义（`class` 语句）及行号范围
  - 识别关键变量赋值和类型注解
  - 识别文档字符串（docstring）
  - 识别导入语句（特别是 `urban_wireless` 相关导入）
  - 每个 evidence item 包含：
    - `evidence_id`（唯一标识）
    - `source_path`（文件绝对路径）
    - `language`（`python`）
    - `source_kind`（`python_stage`）
    - `line_range`（起始行号 - 结束行号）
    - `symbol`（函数名/类名/变量名）
    - `summary`（代码片段或描述）
- **成功结果**：生成 Python 相关的 evidence item 列表
- **失败表现**：语法解析失败时记录错误，不影响其他文件
- **evidence 要求**：每个 item 必须包含 evidence_id、source_path、line_range
- **MVP 必须**：是

### EV-007 提取 evidence item（Verilog）

- **系统动作**：从 Verilog/SystemVerilog 文件中提取 evidence item
- **系统必须**：
  - 识别 module 定义及行号范围
  - 识别 port 声明及行号范围
  - 识别信号/寄存器声明及行号范围
  - 识别 always 块和 assign 语句及行号范围
  - 识别参数和局部参数定义
  - 每个 evidence item 包含：
    - `evidence_id`（唯一标识）
    - `source_path`（文件绝对路径）
    - `language`（`verilog` 或 `systemverilog`）
    - `source_kind`（`rtl`）
    - `line_range`（起始行号 - 结束行号）
    - `symbol`（module 名/信号名/port 名）
    - `summary`（代码片段或描述）
- **成功结果**：生成 Verilog 相关的 evidence item 列表
- **失败表现**：语法解析失败时记录错误，不影响其他文件
- **evidence 要求**：每个 item 必须包含 evidence_id、source_path、line_range
- **MVP 必须**：是

### EV-008 提取 evidence item（测试与文档）

- **系统动作**：从测试和文档文件中提取 evidence item
- **系统必须**：
  - 测试文件：识别测试函数/测试模块、断言语句及行号范围
  - 文档文件：识别章节标题、关键段落及行号范围
  - 每个 evidence item 包含：
    - `evidence_id`（唯一标识）
    - `source_path`（文件绝对路径）
    - `language`（文件类型）
    - `source_kind`（`test` 或 `doc`）
    - `line_range`（起始行号 - 结束行号）
    - `symbol`（测试名/章节标题）
    - `summary`（内容片段）
- **成功结果**：生成测试和文档相关的 evidence item 列表
- **失败表现**：解析失败时记录错误
- **evidence 要求**：每个 item 必须包含 evidence_id、source_path、line_range
- **MVP 必须**：是

### EV-009 建立 evidence 索引

- **系统动作**：将所有 evidence item 组织成可检索的索引
- **系统必须**：
  - 按 source_kind 分组索引
  - 按文件路径分组索引
  - 按 symbol 名称建立反向索引
  - 支持按 evidence_id 快速查找
- **成功结果**：evidence 索引建立完毕，支持后续检索和关联
- **失败表现**：索引建立失败时提示"证据索引建立失败"
- **evidence 要求**：索引应完整覆盖所有 evidence item
- **MVP 必须**：是

### EV-010 收集结果展示

- **系统动作**：在界面展示证据收集结果
- **系统必须**：
  - 显示收集到的 evidence item 总数
  - 按 source_kind 分组统计（Python / RTL / Test / Doc / Config）
  - 显示处理失败的文件数量和原因
  - 显示"下一步：生成理解"按钮
- **成功结果**：用户了解证据收集情况并准备进入理解生成
- **失败表现**：无证据时提示"未收集到有效证据，请检查阶段内容"
- **evidence 要求**：统计信息应关联到具体 source path
- **MVP 必须**：是

### EV-011 未知证据标记

- **系统动作**：对无法明确分类或解析的内容进行标记
- **系统必须**：
  - 对解析失败的文件内容标记为 `unknown`
  - 对无法确定 symbol 类型的代码片段标记为 `unknown`
  - 在 evidence item 的 `confidence` 字段中标注 `unknown`
- **成功结果**：不可解析的内容被显式标记，不隐藏
- **失败表现**：无
- **evidence 要求**：unknown 标记的 item 仍应包含 source_path 和 line_range（如可获取）
- **MVP 必须**：是

## 输入

| 输入项 | 来源 | 类型 |
|--------|------|------|
| stage_context.json | story-select-stage 输出 | 结构化数据（文件列表） |
| 文件系统内容 | 目标项目目录 | 文本文件 |

## 输出

| 输出项 | 类型 | 说明 |
|--------|------|------|
| evidence_index.json | 结构化数据 | evidence item 列表和索引 |
| 收集结果面板 | UI 状态 | 统计信息、失败记录、下一步按钮 |

## 异常 / 空状态

| 场景 | 处理 |
|------|------|
| 所有文件都无法读取 | 提示"无法读取任何文件，请检查权限" |
| 未收集到任何 evidence | 提示"未找到可分析的内容"，建议检查阶段目录 |
| 文件过大被截断 | 标注"文件过大已截断"，展示截断行数 |
| 解析失败 | 记录失败文件和原因，继续处理其他文件 |
| 收集过程被取消 | 保留已收集的 evidence，标注"收集已中断，部分结果可用" |

## 证据与追溯要求

- 每个 evidence item 必须包含唯一的 `evidence_id`
- 每个 evidence item 必须包含 `source_path`（绝对路径）
- 每个 evidence item 必须包含 `line_range`（起始行号 - 结束行号）
- evidence 索引应支持通过 `evidence_id` 快速定位到具体的 source_path 和 line_range

## 不确定性表达要求

- 解析失败的文件内容标记为 `unknown`
- 无法确定类型的代码片段标记为 `unknown`
- 文件过大被截断的部分标注为 `inferred`（基于部分内容的推断）
- 所有 `unknown` 和 `inferred` 标记必须在收集结果中可见

## MVP 验收标准

- [ ] 能读取 Python、Verilog、文档、测试、配置文件
- [ ] 能跳过二进制文件和不可读文件
- [ ] 能处理大文件（截断并标注）
- [ ] 每个 evidence item 包含 evidence_id、source_path、line_range
- [ ] 能按 source_kind 建立索引
- [ ] 收集结果展示统计信息和失败记录
- [ ] 解析失败的内容标记为 unknown
- [ ] 收集过程中不写入目标项目目录

## 非目标

- 不做完整的 AST（抽象语法树）解析（MVP 使用轻量级提取）
- 不做跨文件符号解析（留到理解生成阶段）
- 不做类型推断（Python 的动态类型）
- 不做综合或仿真（明确禁止）

## 关联文档

- [`../mvp-functional-contract.md`](../mvp-functional-contract.md) — 跨 story 对象契约与验收场景
- [`story-select-stage.md`](story-select-stage.md) — 前置：选择阶段
- [`story-generate-understanding.md`](story-generate-understanding.md) — 下一步：生成结构化理解
- [`../mvp-requirements.md`](../mvp-requirements.md) — MVP 证据收集要求
- [`../../design/evidence-model.md`](../../design/evidence-model.md)（待创建）— evidence 模型设计
