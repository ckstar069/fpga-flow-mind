# Workspace 扫描与阶段识别设计

---
status: draft
updated: 2026-06-11
---

> 本文档是 Phase 1 技术设计**入口与边界说明**，不是完整详细设计。
> Phase 1 指 `fpga-flow-mind` **本项目**的开发推进阶段，不是业务项目的 `L0` / `L1` / `RTL` 实现阶段。
>
> 详细设计拆分为：
> - [`phase-1-architecture.md`](phase-1-architecture.md) — 概要设计（模块划分、数据流、职责边界）
> - [`phase-1-data-and-api-contract.md`](phase-1-data-and-api-contract.md) — 数据结构与前后端接口契约
> - [`phase-1-scanner-detail-design.md`](phase-1-scanner-detail-design.md) — 扫描与阶段识别详细算法设计

## 1. 设计目标

Phase 1 只解决：

- 打开业务项目目录（Tauri 文件选择器）
- 只读扫描 workspace
- 识别是否可能是 `ai_project_template` 创建的项目
- 识别候选阶段目录
- 生成 `workspace_profile.json`
- 用户选择单阶段后生成 `stage_context.json`

Phase 1 **不解决**：evidence 提取（Phase 2）、大模型调用（Phase 3）、语义理解（Phase 3）、视图生成（Phase 4）、追问（Phase 5）、持久化回放（Phase 6）、跨阶段对比、Python→RTL 映射。

## 2. 需求来源

本设计基于以下已生效的需求文档，不得重新定义需求对象，只能细化实现边界：

- [`story-open-workspace.md`](../requirements/stories/story-open-workspace.md) — WS-001~007
- [`story-select-stage.md`](../requirements/stories/story-select-stage.md) — ST-001~008
- [`mvp-functional-contract.md`](../requirements/mvp-functional-contract.md) — 对象契约与枚举定义
- [`mvp-requirements.md`](../requirements/mvp-requirements.md) — Workspace 识别、阶段聚焦要求

## 3. 职责边界

| 层级 | Phase 1 内具体工作 |
|------|-------------------|
| **Tauri** | 提供文件选择器；暴露 Rust command（`open_workspace`、`select_stage`）；前后端桥接 |
| **Rust backend** | 验证路径；递归扫描；识别阶段目录和文件类型；生成 `workspace_profile.json` 和 `stage_context.json`；**不调用大模型** |
| **React frontend** | 展示 workspace 概览、阶段列表、文件统计、warning/error；处理单阶段选择；"开始分析"按钮禁用或占位 |

约束：Python 不能作为产品核心实现；Phase 1 不调用大模型或外部 provider；不运行目标项目脚本。

## 4. 只读安全策略

Rust backend 对目标项目仅允许 `read_dir`、`metadata`、`read_file` 类读取操作。禁止调用 `write`、`create`、`remove`、`rename`。所有输出产物仅作为内存对象或写入 app-owned 目录。不运行 Vivado、synthesis、implementation、bitstream。不默认执行目标项目中的 `.py`、`.sh`、`.tcl` 等脚本。扫描前检查路径可读性，遇 `permission_denied` 立即返回错误。

Phase 1 输出对象在本轮技术设计中**只作为内存/系统对象**，不落地到目标项目目录。持久化设计留到 Phase 6。

## 5. Workspace 扫描设计

**输入**：用户通过 Tauri 选择的目录绝对路径。

**路径校验**（顺序执行，任一失败即返回对应 error_code）：
1. 路径是否存在 → `path_not_found`
2. 路径是否为目录 → `not_directory`
3. 是否有读权限 → `permission_denied`

**扫描范围策略**：
- 递归深度上限：3 层
- 单目录文件数上限：1000（超限记录 warning，跳过剩余）
- 总扫描文件数上限：5000
- 符号链接：不跟随
- 隐藏目录（`.` 开头）：扫描但不做阶段识别（除非明确匹配阶段模式）

**文件类型识别**：

| 扩展名/模式 | language | source_kind |
|-----------|----------|-------------|
| `.py` | `python` | `python_stage` |
| `.v`, `.sv`, `.vh` | `verilog`/`systemverilog` | `rtl` |
| `.md`, `.rst`, `.txt` | `markdown`/`text` | `doc` |
| `test_*.py`, `*_tb.v`, `*_test.v` | 同对应语言 | `test` |
| `.json`, `.yaml`, `.yml`, `.toml` | `json`/`yaml`/`toml` | `config` |

**外部模块引用识别**：在 `.py` 中匹配 `from urban_wireless import`、`import urban_wireless`、路径含 `urban_wireless` 的模式，提取模块名作为 `external_refs[]`。不做 AST 解析，用正则/字符串匹配。

**异常文件处理**：不可读文件记录 warning 并跳过；二进制文件跳过；大文件（>5MB）记录 warning，仅读前 100 行识别类型；全局扫描超时 30 秒，超时后返回已收集结果 + `scan_timeout` warning。

## 6. 阶段识别设计

**候选阶段目录识别**（不区分大小写）：
- 标准模式：`L0`~`L6`、`RTL`
- 常见变体：`rtl`、`rtl_final`、`hardware`、`fpga`

**命名异常**：目录名含 `rtl` 但不等于 `RTL`（如 `rtl_final`），或含 `level` 加数字（如 `level3`）→ `status = naming_anomaly`。仍作为候选展示，允许选择。

**阶段缺失**：预期标准阶段未找到 → **不插入 `stages[]`**，仅在 `warnings[]` / `error_codes[]` / `validity_reasons[]` 中记录。原因：`stages[].source_path` 约束为阶段目录绝对路径，若插入缺失阶段则 `source_path` 指向不存在的目录，容易误导后续实现（如误将预期路径当作可读目录处理）。`stages[]` 中只保留真实存在且可读的目录条目。缺失信息通过 warnings 和 validity_reasons 向用户展示。不阻塞其他可用阶段。

**空阶段与不可读**：目录存在但无文件 → `status = empty`；不可读 → `status = unreadable`。

**阶段排序**：`stages[]` 中只包含真实存在的目录。按 `L0`→`L1`→...→`L6`→`RTL` → 其他命名异常阶段按字典序排最后。缺失阶段不进入 `stages[]`，其信息通过 `warnings[]` 和 `validity_reasons[]` 展示。

**阶段选择后生成 stage_context**：验证阶段目录仍存在且可读；收集阶段目录下文件列表（递归深度 2 层）；识别每个文件的 `language` 和 `source_kind`；识别 `external_deps[]` 和 `upstream_refs[]`；生成 `stage_context.json`。

注意：`files[]` 允许为空（概览上下文），但进入 evidence 收集前必须非空。

## 7. 输出对象映射

基于 [`mvp-functional-contract.md`](../requirements/mvp-functional-contract.md)，所有字段必须从契约派生，不得新增冲突字段。

**workspace_profile.json 字段来源**：`workspace_name`（根目录名）、`root_path`（用户选择路径）、`stages[]`（阶段识别结果，含 `stage_id`、`source_path`、`file_count`、`status`）、`file_type_stats`（扩展名统计）、`external_refs[]`（外部模块引用）、`validity`（见下方判定规则）、`validity_reasons[]`（判定理由）、`warnings[]`（非致命警告）、`error_codes[]`（错误码）、`scan_timestamp`（ISO 8601）、`version`（`"1.0.0"`）。

**validity 判定规则**：

`likely_valid` 不要求同时存在 Python 和 Verilog。早期 L0/L1/L2 项目可能只有 Python，不应被误判为 `unlikely`。

| 条件 | validity | 说明 |
|------|----------|------|
| 符合 `ai_project_template` 阶段目录特征（至少 1 个标准/变体阶段）且存在可分析代码（Python 或 Verilog/SystemVerilog） | `likely_valid` | 同时存在 Python 和 Verilog 可作为增强信号，不是必要条件 |
| 只有少量特征匹配（如仅 1 个标准阶段但无可分析代码，或无可识别阶段但存在代码文件） | `uncertain` | 可能是不完整项目或非标准结构 |
| 无阶段特征且无可分析代码 | `unlikely` | 可能不是 `ai_project_template` 项目，但仍允许用户强制继续 |

`uncertain` / `unlikely` 允许用户强制继续。

**stage_context.json 字段来源**：`stage_id`（用户选中阶段）、`source_path`（阶段目录绝对路径）、`files[]`（阶段下文件列表，含 `source_path`、`language`、`source_kind`、`size_bytes`）、`external_deps[]`（阶段内外部引用）、`upstream_refs[]`（推断的上游引用，可标注 `inferred`）、`error_code`（`stage_empty`/`stage_unreadable`）。

## 8. 错误与空状态

| error_code | 触发条件 | 是否允许继续 |
|-----------|---------|-------------|
| `path_not_found` | 路径不存在 | 否（需重新选择） |
| `not_directory` | 路径不是目录 | 否（需重新选择） |
| `permission_denied` | 无读权限 | 否（需重新选择） |
| `no_stage_found` | 未识别到阶段目录 | 是（用户强制继续） |
| `stage_empty` | 阶段目录为空 | 是（但无可分析内容） |
| `stage_unreadable` | 阶段目录不可读 | 否（选择其他阶段） |

Rust backend 返回错误结果 + error_code。React frontend 对 `path_not_found`/`not_directory`/`permission_denied` 弹窗提示并允许重新选择；对 `no_stage_found` 显示"未识别到阶段"并提供"强制继续"按钮；对 `stage_empty` 灰色展示该阶段；对 `stage_unreadable` 禁用该阶段。

## 9. UI 状态与交互

Phase 1 最小 UI：

| UI 元素 | Phase 1 行为 |
|--------|-------------|
| 选择项目目录入口 | 打开 Tauri 文件选择器 |
| workspace 概览 | 显示名称、根路径、阶段列表、文件类型统计 |
| 阶段列表 | 按排序规则展示 `stages[]` 中的阶段，标注状态（available/empty/naming_anomaly/unreadable）。缺失阶段不在列表中，仅在 warnings 区域展示 |
| warning/error 展示 | 在概览面板以列表或图标展示 |
| 单阶段选择 | 高亮选中，触发 stage_context 生成 |
| 阶段概览 | 显示阶段名称、路径、文件列表分组、外部依赖 |
| "开始分析"按钮 | **禁用或占位**，不进入 evidence 收集 |

不设计高保真 UI 或视觉稿。

## 10. 验证计划

使用临时目录构造样例，无需真实 Vivado 或业务项目：

| 样例 | 验证点 |
|------|--------|
| 标准业务项目（L0~RTL，含 .py/.v） | workspace_profile 正确、validity=likely_valid、排序正确 |
| 无阶段目录但存在代码（仅 .py/.v） | validity=uncertain、`no_stage_found`、warnings 提示未识别到阶段、允许强制继续 |
| 命名异常阶段（rtl_final/、hardware/） | naming_anomaly 标注、仍可作为候选 |
| 阶段缺失（仅 L0/L3/RTL） | 缺失阶段不进入 `stages[]`，通过 warnings 和 validity_reasons 展示，validity=uncertain |
| 空阶段（L0/ 为空、L1/ 有文件） | L0 status=empty、L1 status=available |
| 不可读路径 | permission_denied、友好提示 |
| 大目录（单目录 2000+ 文件） | 不卡死、触发文件数上限 warning |
| 空目录（无阶段且无代码） | validity=unlikely、no_stage_found、允许强制继续 |
| 安全验证 | 目标目录无新增/修改/删除、不运行脚本 |

## 11. Phase 1 验收标准

- [ ] 用户能通过 Tauri 选择业务项目目录
- [ ] Rust backend 能生成 `workspace_profile.json`
- [ ] 能识别标准阶段（L0~RTL）和命名异常阶段
- [ ] 能处理阶段缺失、空阶段、不可读阶段
- [ ] 用户能选择单个阶段
- [ ] 能生成 `stage_context.json`
- [ ] 能展示 warning 和 error
- [ ] 扫描过程中不写目标项目目录
- [ ] 不运行 Vivado / synthesis / implementation / bitstream
- [ ] 不调用大模型
- [ ] 不执行目标项目脚本

## 12. 已知限制

- 不做 evidence 提取（Phase 2）、语义理解（Phase 3）、视图生成（Phase 4）、追问（Phase 5）
- 不做深度依赖解析（仅表层正则匹配外部引用）
- 阶段识别规则基于目录名模式，可能随真实 `ai_project_template` 业务项目样例接入后调整
- `upstream_refs[]` 在 Phase 1 中可能仅做简单推断，精确关联留到 Phase 2
