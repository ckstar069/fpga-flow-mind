# Phase 1 Workspace 扫描与阶段识别验证设计

---
status: draft
updated: 2026-06-11
---

> 本文档是 Phase 1 的测试与验证设计，约束后续编码阶段如何验证功能、边界、安全和 UI 状态。
> Phase 1 指 `fpga-flow-mind` **本项目**的开发推进阶段，不是业务项目的 `L0` / `L1` / `RTL` 实现阶段。
> 不写测试代码、不创建测试夹具、不运行测试。

## 1. 测试目标

### 功能验证

- `open_workspace` 能正确接收路径字符串，返回 `CommandResult<WorkspaceProfile>`
- 路径校验（存在性、目录性、可读性、非 symlink 根路径）正确映射错误码
- `workspace_profile.json` 字段完整：`stages[]`、`file_type_stats`、`validity`、`warnings[]`、`error_codes[]`
- 阶段识别覆盖标准阶段（`L0`~`L6`、`RTL`）、变体（`rtl`、`rtl_final` 等）、命名异常、空阶段、不可读阶段
- `missing` 阶段不插入 `stages[]`，通过 `warnings[]` 和 `validity_reasons[]` 展示
- `select_stage` 能正确返回 `CommandResult<StageContext>`，`files[]` 允许为空
- `stage_empty` 返回 `success=true` 携带空 `files[]`，不触发 evidence 收集
- `stage_unreadable` 返回 `success=false`，仅阻断该阶段，不阻断 workspace

### 边界验证

- 大文件（>5MB）触发 `file_too_large` warning，只读前 100 行
- 扫描超时（>30 秒）触发 `scan_timeout` warning，返回已收集结果
- 单目录文件数 >1000 截断并记录 warning
- 根路径为 symlink 时拒绝，映射为 `permission_denied`
- 扫描中遇 symlink 跳过，不跟随到 workspace 外部

### 安全验证

- 目标项目目录**不被写入**
- 不创建 / 修改 / 删除目标项目文件
- 不运行目标项目脚本（`.py`、`.sh`、`.tcl` 等）
- 不运行 Vivado / synthesis / implementation / bitstream
- 所有输出仅进入 app-owned 目录或测试临时目录

### UI 验证

- 不展示原始 JSON 或 JSON viewer
- 不生成 Markdown 格式分析报告
- 不暗示已完成 evidence 收集或语义分析
- "开始分析"按钮保持禁用或占位状态
- 空阶段展示空状态说明，不激活分析流程

## 2. 测试来源文档

本文档的验证点来源于以下文档，后续实施中若来源文档发生变更，本验证设计应同步更新：

| 来源文档 | 定位 | 验证点覆盖 |
|---------|------|-----------|
| [`docs/requirements/mvp-functional-contract.md`](../requirements/mvp-functional-contract.md) | MVP 功能契约（`active`） | 枚举值、错误码、字段语义、validity 判定规则 |
| [`docs/requirements/stories/story-open-workspace.md`](../requirements/stories/story-open-workspace.md) | WS-001~007 用户故事 | workspace 打开流程、路径校验、概览展示 |
| [`docs/requirements/stories/story-select-stage.md`](../requirements/stories/story-select-stage.md) | ST-001~008 用户故事 | 阶段列表、单阶段选择、空阶段、命名异常 |
| [`docs/design/workspace-scanning-and-stage-detection.md`](../design/workspace-scanning-and-stage-detection.md) | Phase 1 技术入口 | 扫描范围、阶段识别边界、安全约束 |
| [`docs/design/phase-1-architecture.md`](../design/phase-1-architecture.md) | Phase 1 概要设计 | 模块职责、数据流、前后端边界 |
| [`docs/design/phase-1-data-and-api-contract.md`](../design/phase-1-data-and-api-contract.md) | Phase 1 数据/API 契约 | `CommandResult` 语义、`WorkspaceProfile` / `StageContext` 字段完整性、UI 状态映射 |
| [`docs/design/phase-1-scanner-detail-design.md`](../design/phase-1-scanner-detail-design.md) | Phase 1 扫描详细设计 | DFS 算法、文件分类、外部引用识别、validity 判定算法、边界条件 |
| [`docs/ui-ux/phase-1-workspace-and-stage-flow.md`](../ui-ux/phase-1-workspace-and-stage-flow.md) | Phase 1 UI/UX 轻量设计 | 组件定义、状态展示规则、空状态处理、不展示 JSON 规则 |

## 3. 样例 workspace 矩阵

以下测试夹具设计为后续编码阶段构造，本轮不创建真实目录。每个夹具应使用临时目录构造，验证后清理。

### 3.1 标准完整项目

**场景**：`L0/`、`L1/`、`L2/`、`L3/`、`RTL/` 均存在，含 `.py` 和 `.v` 文件。

| 验证项 | 预期结果 |
|--------|---------|
| `validity` | `likely_valid` |
| `stages[]` 数量 | 5 |
| `stages[].status` | 均为 `available`（非空时） |
| `file_type_stats` | 含 `.py`、`.v` 计数 |
| `external_refs` | 若代码含 `urban_wireless` 引用则非空 |
| `warnings[]` | 空（理想情况下） |

### 3.2 早期阶段项目（仅 Python）

**场景**：`L0/`、`L1/`、`L2/` 存在，仅含 `.py`，无 `.v`/`.sv`。

| 验证项 | 预期结果 |
|--------|---------|
| `validity` | `likely_valid` |
| `stages[].status` | 均为 `available` |
| `file_type_stats` | 仅 `.py` |

### 3.3 硬件阶段项目（仅 RTL）

**场景**：`RTL/` 存在，含 `.v`/`.sv`，无 `.py`。

| 验证项 | 预期结果 |
|--------|---------|
| `validity` | `likely_valid` |
| `stages[].status` | `available` |

### 3.4 无阶段但有代码

**场景**：根目录下有 `.py`、`.v` 文件，无标准阶段目录。

| 验证项 | 预期结果 |
|--------|---------|
| `validity` | `uncertain` |
| `stages[]` | 空 |
| `error_codes[]` | 含 `no_stage_found` |
| `warnings[]` | 含 "未识别到标准阶段" |
| `recoverable` | `true`（可继续浏览 workspace） |

### 3.5 空目录

**场景**：无任何文件和子目录。

| 验证项 | 预期结果 |
|--------|---------|
| `validity` | `unlikely` |
| `stages[]` | 空 |
| `error_codes[]` | 含 `no_stage_found` |
| `file_type_stats` | 空 |

### 3.6 仅文档无代码

**场景**：根目录下仅有 `.md`、`.txt`，无 `.py`/`.v`/`.sv`，可能有阶段目录（空）。

| 验证项 | 预期结果 |
|--------|---------|
| 有阶段目录（空） | `validity = uncertain`，UI 提示"未发现可分析源码" |
| 无阶段目录 | `validity = unlikely`，UI 提示"未发现可分析源码" |
| `stages[]` | 视阶段目录存在性而定（空阶段也插入 `stages[]`，`status = empty`） |
| `file_type_stats` | 仅 `.md`/`.txt` |
| 不展示 JSON dump | UI 层验证 |

### 3.7 部分阶段缺失

**场景**：`L0/`、`RTL/` 存在，`L1`~`L3` 缺失。

| 验证项 | 预期结果 |
|--------|---------|
| `stages[]` | 仅含 `L0`、`RTL` |
| `warnings[]` | 含 `L1`、`L2`、`L3` 缺失记录 |
| `validity` | `uncertain`（视代码存在性） |
| `validity_reasons[]` | 含缺失阶段说明 |

### 3.8 命名异常阶段

**场景**：存在 `rtl_final/`、`hardware/` 等非标准目录名。

| 验证项 | 预期结果 |
|--------|---------|
| `stages[].status` | `naming_anomaly` |
| `stages[].stage_id` | 按变体映射（如 `rtl_final` → `rtl`）或保留原值 |
| 可点击选择 | 是 |
| 排序 | 排在标准阶段之后 |

### 3.9 空阶段

**场景**：`L0/` 存在但无文件，`L1/` 有文件。

| 验证项 | 预期结果 |
|--------|---------|
| `L0.status` | `empty` |
| `L1.status` | `available` |
| `select_stage(L0)` | 返回 `CommandResult<StageContext>`，`success=true`，`files[] = []`，`error_code = stage_empty` |
| UI | 展示空阶段说明，不激活"开始分析"，不显示"强制继续分析"按钮 |
| `select_stage(L1)` | 正常返回，`files[]` 非空 |

### 3.10 不可读阶段

**场景**：`L0/` 权限不可读，`L1/` 可读。

| 验证项 | 预期结果 |
|--------|---------|
| `L0.status` | `unreadable` |
| `L1.status` | `available` |
| `select_stage(L0)` | `success=false`，`error_code = stage_unreadable` |
| workspace 概览 | 正常展示，不阻断 |
| `L1` 可选择 | 是 |

### 3.11 大文件

**场景**：单个 `.v` 文件 >5MB。

| 验证项 | 预期结果 |
|--------|---------|
| `warnings[]` | 含 `file_too_large`，记录文件路径 |
| `file_type_stats` | 仍计入该文件（按扩展名） |
| 文件内容读取 | 仅读前 100 行用于类型识别 |

### 3.12 不可读文件

**场景**：某个文件权限不可读。

| 验证项 | 预期结果 |
|--------|---------|
| `warnings[]` | 含 `file_unreadable`，记录路径 |
| 不影响整体扫描 | `success = true` |

### 3.13 扫描超时

**场景**：构造大目录使扫描超过 30 秒。

| 验证项 | 预期结果 |
|--------|---------|
| `warnings[]` | 含 `scan_timeout` |
| `success` | `true` |
| 返回结果 | 已收集的部分结果 |

### 3.14 根路径为 symlink

**场景**：用户选择的路径是符号链接。

| 验证项 | 预期结果 |
|--------|---------|
| `open_workspace` | `success=false` |
| `error_code` | `permission_denied`（不映射为 `path_not_found`） |
| 不跟随 symlink | 安全边界验证 |

### 3.15 扫描中遇 symlink

**场景**：workspace 内部某个子目录或文件是符号链接。

| 验证项 | 预期结果 |
|--------|---------|
| 扫描行为 | 跳过，不跟随到外部目录 |
| `warnings[]` | 可能含跳过记录（视设计而定，非强制） |
| `success` | `true` |

## 4. 后端验证点

### 4.1 Tauri Command 输入/输出

| Command | 输入验证 | 输出验证 |
|---------|---------|---------|
| `open_workspace(path: String)` | 空字符串、含尾部斜杠、相对路径、含 symlink 的路径 | `CommandResult<WorkspaceProfile>` 的 `success`/`data`/`error`/`warnings` 字段完整 |
| `select_stage(root_path: String, stage_id: String)` | 空 `stage_id`、`stage_id` 不在 `stages[]` 中 | `CommandResult<StageContext>` 的 `files[]`、`external_deps[]`、`upstream_refs[]`、`error_code` 字段完整 |

### 4.2 CommandResult 语义

| 场景 | `success` | `data` | `error` | 验证点 |
|------|-----------|--------|---------|--------|
| 正常 workspace | `true` | `Some` | `None` | 所有字段非空 |
| 路径不存在 | `false` | `None` | `Some` | `error_code = path_not_found` |
| 非目录 | `false` | `None` | `Some` | `error_code = not_directory` |
| 权限不足 | `false` | `None` | `Some` | `error_code = permission_denied` |
| 无阶段 | `true` | `Some` | `None` | `stages[]` 为空，`error_codes[]` 含 `no_stage_found` |
| 空阶段 | `true` | `Some(StageContext)` | `None` | `files[] = []`，`error_code = stage_empty` |
| 阶段不可读 | `false` | `None` | `Some` | `error_code = stage_unreadable` |

### 4.3 WorkspaceProfile 字段完整性

| 字段 | 验证内容 |
|------|---------|
| `workspace_name` | 从路径提取的目录名 |
| `root_path` | 规范化后的绝对路径（不含尾部斜杠，非 symlink） |
| `validity` | 枚举值之一，与阶段/代码存在性匹配 |
| `validity_reasons[]` | 非空时说明降级原因 |
| `stages[]` | 排序正确，标准阶段在前，命名异常在后 |
| `stages[].stage_id` | 映射正确 |
| `stages[].status` | `available`/`empty`/`naming_anomaly`/`unreadable` 之一 |
| `stages[].file_count` | 阶段目录下（深度 1）可识别文件数，不含二进制 |
| `stages[].source_path` | 阶段目录绝对路径 |
| `file_type_stats` | 按扩展名统计，测试文件正确归类 |
| `external_refs[]` | 识别 `urban_wireless` 引用 |
| `warnings[]` | workspace 扫描过程中的非致命问题，每个含 `error_code`、`message`、`source_path`；不含 `select_stage` 产生的阶段级问题 |
| `error_codes[]` | 仅在 `open_workspace` 返回 `WorkspaceProfile`（`success=true`）时记录 workspace 级异常码（如 `no_stage_found`）；路径校验失败通过 `CommandError.error_code` 返回，无 `WorkspaceProfile`，因此不进入 `WorkspaceProfile.error_codes[]`；不含 `select_stage` 产生的 `stage_empty`/`stage_unreadable` |

### 4.4 StageSummary / StageContext 字段完整性

| 字段 | 验证内容 |
|------|---------|
| `stage_id` | 与 `WorkspaceProfile.stages[].stage_id` 一致 |
| `source_path` | 绝对路径 |
| `files[]` | 允许为空；非空时含 `source_path`、`language`、`source_kind`、`size_bytes` |
| `files[].source_kind` | `python_stage`/`rtl`/`test`/`doc`/`config` 等 |
| `external_deps[]` | 阶段级别的外部模块引用 |
| `upstream_refs[]` | 含 `stage_id`、`interface_file_path`、`inferred` |
| `error_code` | 空阶段时为 `stage_empty`，不可读时为 `stage_unreadable` |

### 4.5 Error Code / Warning 映射

> 下表区分 **workspace 级**（归属 `WorkspaceProfile`）与 **stage/select_stage 级**（归属 `StageContext` 或 `CommandError`）。
> `WorkspaceProfile.error_codes[]` 与 `WorkspaceProfile.warnings[]` 仅收录 workspace 扫描过程中的问题；`select_stage` 产生的阶段级结果不在其中重复。

| error_code | 作用域 | 进入 `WorkspaceProfile.warnings[]` | 进入 `WorkspaceProfile.error_codes[]` | `CommandResult.success` | 验证点 |
|-----------|--------|-----------------------------------|--------------------------------------|------------------------|--------|
| `path_not_found` | `open_workspace` | 否 | 否 | `false` | 阻塞 workspace 打开；错误码在 `CommandError.error_code`，无 `WorkspaceProfile` |
| `not_directory` | `open_workspace` | 否 | 否 | `false` | 阻塞 workspace 打开；错误码在 `CommandError.error_code`，无 `WorkspaceProfile` |
| `permission_denied` | `open_workspace` | 否 | 否 | `false` | 阻塞 workspace 打开；错误码在 `CommandError.error_code`，无 `WorkspaceProfile` |
| `stage_unreadable` | `select_stage` | 否 | 否 | `false` | 仅阻断该阶段；`CommandError.error_code` |
| `no_stage_found` | `open_workspace` | 是 | 是 | `true` | workspace 级降级浏览 |
| `stage_empty` | `select_stage` | 否 | 否 | `true` | 展示空阶段说明；`StageContext.error_code` |
| `file_unreadable` | `open_workspace` | 是 | 否 | `true` | 仅展示 |
| `file_too_large` | `open_workspace` | 是 | 否 | `true` | 仅展示 |
| `scan_timeout` | `open_workspace` | 是 | 否 | `true` | 仅展示 |

**说明**：
- 路径校验类错误（`path_not_found`/`not_directory`/`permission_denied`）：`open_workspace` 返回 `success=false`，错误码在 `CommandError.error_code`；无 `WorkspaceProfile`，因此**不进入** `WorkspaceProfile.warnings[]` 或 `WorkspaceProfile.error_codes[]`。
- `stage_empty` 仅通过 `StageContext.error_code` 表达，不进入 `WorkspaceProfile.warnings[]`，也不进入 `WorkspaceProfile.error_codes[]`。
- `stage_unreadable` 仅通过 `CommandError.error_code` 表达，不进入 `WorkspaceProfile.warnings[]`，也不进入 `WorkspaceProfile.error_codes[]`。
- `no_stage_found` 同时进入 `WorkspaceProfile.warnings[]` 和 `WorkspaceProfile.error_codes[]`，因为既是扫描结果也是 workspace 级异常码。

### 4.6 Validity 判定

| 条件 | `validity` | 验证点 |
|------|-----------|--------|
| 标准阶段 + Python 或 Verilog | `likely_valid` | 不要求同时存在两种语言 |
| 无可识别阶段 + 有可分析代码 | `uncertain` | 非标准结构或不完整项目 |
| 有阶段但无核心代码 | `uncertain` | 阶段存在但内容异常 |
| 无阶段 + 无代码 + 无文档 | `unlikely` | 空目录 |
| 仅文档 + 有阶段目录（空） | `uncertain` | 有项目痕迹但无核心代码 |
| 仅文档 + 无阶段目录 | `unlikely` | 无项目结构 |

### 4.7 Stage 列表完整性

- `stages[]` **包含**真实存在且**可识别为阶段**的目录，不要求可读
- 可读阶段：`status` 可为 `available`、`empty`、`naming_anomaly`
- 不可读阶段：`status` 可为 `unreadable`（保留在列表中，用户可知该阶段存在但不可访问）
- `missing` 阶段**不插入** `stages[]`
- `missing` 信息通过 `WorkspaceProfile.warnings[]` 和 `validity_reasons[]` 传递
- 排序：标准阶段（`L0`→`L6`→`RTL`）→ 命名异常阶段（字典序）→ `unreadable` / `empty` 阶段保留在对应位置，不单独分组

### 4.8 Stage Empty 不触发 Evidence 收集

- `select_stage` 返回 `stage_empty` 时，`CommandResult.success = true`
- `files[]` 为空数组
- **不进入 evidence 收集流程**
- UI 展示空阶段说明（`stage_empty_view`），不激活"开始分析"按钮

### 4.9 Recoverable 不等于强制继续分析

- `recoverable = true` 仅表示问题非崩溃级，用户可通过其他操作恢复流程
- `no_stage_found` + `recoverable = true` → 前端展示"继续浏览 workspace"按钮（workspace 级降级浏览）
- `stage_empty` + `recoverable = true` → 前端展示空阶段说明，**不显示"强制继续分析"按钮**
- `recoverable = false` → 当前操作阻塞，必须重新选择或修正输入

## 5. 前端验证点

### 5.1 初始状态

| 验证项 | 预期 |
|--------|------|
| 页面加载 | 显示欢迎引导，无 workspace 数据 |
| "打开项目"按钮 | 可见且可点击 |
| 阶段列表 | 不展示 |
| 底部 warnings 面板 | 折叠或不展示 |

### 5.2 打开项目 Loading

| 验证项 | 预期 |
|--------|------|
| 点击"打开项目" | 弹出系统文件选择器 |
| 选择目录后 | 显示 loading 状态，按钮禁用 |
| 加载中取消 | 保持当前状态不变（若支持取消） |

### 5.3 Workspace Summary

| 验证项 | 预期 |
|--------|------|
| 名称 | 显示目录名 |
| 根路径 | 显示规范化后的绝对路径 |
| validity 标识 | 颜色/图标与 `likely_valid`/`uncertain`/`unlikely` 对应 |
| 外部引用数量 | 若 `external_refs[]` 非空则展示数量 |

### 5.4 Validity 展示

| `validity` | 文案 | 颜色 | 强制继续入口 |
|-----------|------|------|-------------|
| `likely_valid` | "项目结构符合预期" | 绿色/正常 | 不需要 |
| `uncertain` | "项目结构部分匹配" | 黄色/橙色 | 可提供"继续"入口（浏览 workspace） |
| `unlikely` | "项目结构不符合预期" | 红色/警告 | 可提供"强制继续"入口（浏览 workspace） |

> `stage_empty` **不**触发强制继续入口。强制继续仅用于 workspace 级 `no_stage_found` 或 validity 降级场景。

### 5.5 Stage List 状态展示

| `status` | 视觉标识 | 可点击 | 可选中 |
|---------|---------|--------|--------|
| `available` | 正常样式 | 是 | 是 |
| `empty` | 灰色降级 + 警告图标 | 是（提示原因） | 否 |
| `naming_anomaly` | 黄色/橙色标签 | 是 | 是 |
| `unreadable` | 灰色禁用 + 锁定图标 | 是（提示原因） | 否 |

> 图标使用项目图标库，不使用 emoji。

### 5.6 Warning List

| 验证项 | 预期 |
|--------|------|
| 展示位置 | 底部可折叠面板 |
| 每条 warning | 图标 + `error_code` + 中文 message + `source_path` |
| `no_stage_found` | "未识别到阶段" + 强制继续浏览按钮 |
| `stage_empty` | "该阶段为空"（阶段列表中展示，不在全局 warning 中重复） |
| `file_too_large` | 文件路径 + "超过 5MB" |
| `scan_timeout` | "扫描超时，已返回部分结果" |

### 5.7 Error Banner / 阶段级错误提示

| 场景 | 展示组件 | 阻断范围 |
|------|---------|---------|
| `path_not_found`/`not_directory`/`permission_denied` | ErrorBanner | 全局（workspace 级） |
| `stage_unreadable` | 阶段级错误提示 | 仅该阶段 |

> `stage_unreadable` 不作为全局 ErrorBanner 展示，仅在该阶段详情区域或阶段列表状态中提示。

### 5.8 Stage Overview

| 验证项 | 预期 |
|--------|------|
| 未选中阶段 | 显示"请从左侧选择一个阶段" |
| 选中可用阶段 | 展示阶段名称、路径、文件列表（按 `source_kind` 分组）、外部依赖、上游引用 |
| 选中空阶段 | 展示空阶段说明（`stage_empty_view`），`files[]` 为空提示 |
| 选中不可读阶段 | 展示阶段级错误提示，建议"选择其他阶段" |

### 5.9 Stage Empty View

| 验证项 | 预期 |
|--------|------|
| 触发条件 | `select_stage` 返回 `stage_empty` |
| 展示内容 | "该阶段无文件"或等价空状态文案 |
| "开始分析"按钮 | 禁用或隐藏，不激活 |
| 用户操作 | 可选择其他阶段，或查看空状态说明 |
| 不触发 | evidence 收集、语义分析 |

### 5.10 "开始分析"按钮

| 验证项 | 预期 |
|--------|------|
| Phase 1 状态 | 禁用或灰显，标注"Phase 2 后可用" |
| 点击行为 | 无响应或提示"功能开发中" |
| 空阶段 | 不因此按钮激活 |
| 未选中阶段 | 不展示 |

### 5.11 不展示原始 JSON

| 验证项 | 预期 |
|--------|------|
| `workspace_profile.json` | 不直接作为 UI 内容展示 |
| `stage_context.json` | 不直接作为 UI 内容展示 |
| 无 JSON viewer | 不嵌入 JSON 查看器 |
| 无 Markdown report | 不生成静态 Markdown 报告 |
| 文件统计 | 以可读列表或简单图表展示，不展示 JSON key-value |

### 5.12 不暗示 Evidence/Semantic Analysis

| 验证项 | 预期 |
|--------|------|
| 阶段概览 | 仅展示文件列表，不展示证据面板 |
| 无结构图 | 不展示模块结构图 |
| 无数据流图 | 不展示数据流图 |
| 无时序图 | 不展示时序/流水图 |
| 无 Q&A 面板 | 不展示问答交互 |
| 文案 | 不出现"已分析"、"已理解"、"已收集证据"等暗示性表述 |

## 6. 安全验证点

### 6.1 目标 Workspace 不被写入

- Rust backend 仅调用 `read_dir`、`metadata`、`read_file`
- **禁止**调用 `write`、`create`、`remove`、`rename`
- 所有输出仅写入 app-owned 目录或测试临时目录

### 6.2 不修改目标项目文件

- 验证目标目录的修改时间（`mtime`）在扫描前后**不变**
- 验证目标目录的 `git status`（如有）在扫描前后**无新增/修改/删除**
- 验证不创建 `.fpga-flow-mind` 等隐藏文件或目录

### 6.3 不运行目标项目脚本

- **禁止**自动运行 `.py`、`.sh`、`.tcl` 等脚本
- **禁止**解析并执行代码中的 import 或函数调用
- 外部模块引用仅做文本层面字符串匹配，不做 AST 执行

### 6.4 不运行 Vivado / Synthesis / Implementation / Bitstream

- **禁止**调用 `vivado` 可执行文件
- **禁止**触发 synthesis、implementation、bitstream 生成
- 不解析 `.xpr`、`.tcl` 中的 Vivado 命令并执行

### 6.5 输出隔离

- `workspace_profile.json` / `stage_context.json` 输出到 app-owned 目录
- 不写入目标业务项目的任何子目录
- 临时目录（如有）在测试后清理

### 6.6 Symlink 安全

- 根路径为 symlink → 拒绝，映射为 `permission_denied`
- 扫描中遇 symlink → 跳过，不跟随
- 不跟随 symlink 到 workspace 外部目录
- `canonicalize` 仅用于非 symlink 的普通路径规范化

## 7. 手工验证脚本清单

以下步骤为人工验证流程，非自动化脚本。每个步骤需记录环境、输入、预期和实际结果。

### 7.1 选择标准项目

**环境**：构造含 `L0/`~`L3/`、`RTL/`、`.py`、`.v` 的临时目录。

| 步骤 | 操作 | 预期 |
|------|------|------|
| 1 | 点击"打开项目"，选择标准项目目录 | 扫描完成，展示 workspace 概览 |
| 2 | 检查 workspace 名称 | 显示目录名 |
| 3 | 检查阶段列表 | 显示 `L0`、`L1`、`L2`、`L3`、`RTL`，状态均为 `available` |
| 4 | 检查文件统计 | `.py`、`.v` 数量正确 |
| 5 | 点击 `L0` | 展示阶段概览，文件列表非空 |
| 6 | 检查"开始分析"按钮 | 禁用或灰显 |
| 7 | 检查 warnings 面板 | 空或折叠 |

### 7.2 选择空目录

**环境**：创建无任何内容的临时目录。

| 步骤 | 操作 | 预期 |
|------|------|------|
| 1 | 选择空目录 | 展示 workspace 概览，`validity = unlikely` |
| 2 | 检查阶段列表 | 空或显示"未识别到阶段" |
| 3 | 检查 warnings | 含 `no_stage_found` |
| 4 | 检查"强制继续"入口 | 可强制继续浏览（workspace 级） |
| 5 | 检查目标目录 mtime | 未变化 |

### 7.3 选择无阶段但有代码目录

**环境**：临时目录下放 `.py` 和 `.v` 文件，无阶段子目录。

| 步骤 | 操作 | 预期 |
|------|------|------|
| 1 | 选择该目录 | `validity = uncertain` |
| 2 | 检查阶段列表 | 空 |
| 3 | 检查文件统计 | `.py`、`.v` 数量正确 |
| 4 | 检查 warnings | 含 `no_stage_found` + "存在代码文件但未识别到标准阶段" |
| 5 | 检查"强制继续"入口 | 可强制继续浏览 |

### 7.4 选择包含命名异常阶段目录

**环境**：临时目录含 `rtl_final/`（含 `.v` 文件）。

| 步骤 | 操作 | 预期 |
|------|------|------|
| 1 | 选择该目录 | 阶段列表含 `rtl_final`，状态为 `naming_anomaly` |
| 2 | 点击 `rtl_final` | 可进入阶段概览，文件列表展示 |
| 3 | 检查排序 | 命名异常阶段排在标准阶段之后 |

### 7.5 选择包含空阶段目录

**环境**：临时目录含 `L0/`（空）和 `L1/`（有文件）。

| 步骤 | 操作 | 预期 |
|------|------|----|
| 1 | 选择该目录 | `L0` 灰色展示，状态 `empty`；`L1` 正常展示 |
| 2 | 点击 `L0` | 展示空阶段说明，不进入分析 |
| 3 | 检查"开始分析"按钮 | 禁用或隐藏 |
| 4 | 检查是否出现"强制继续分析"按钮 | **不出现** |
| 5 | 点击 `L1` | 正常展示文件列表 |

### 7.6 选择包含大文件目录

**环境**：临时目录含单个 >5MB 的 `.v` 文件。

| 步骤 | 操作 | 预期 |
|------|------|------|
| 1 | 选择该目录 | 扫描完成，`success = true` |
| 2 | 检查 warnings | 含 `file_too_large` + 文件路径 |
| 3 | 检查文件统计 | 该文件计入 `.v` 数量 |
| 4 | 检查阶段概览 | 正常展示，不报错 |

### 7.7 选择 Symlink 根路径

**环境**：创建一个指向真实项目目录的符号链接。

| 步骤 | 操作 | 预期 |
|------|------|------|
| 1 | 选择 symlink 路径 | ErrorBanner 展示"无读权限"或等效提示 |
| 2 | 检查 `error_code` | `permission_denied`（不是 `path_not_found`） |
| 3 | 检查是否跟随 symlink | **未跟随**，未展示真实目录内容 |

### 7.8 检查 UI 不出现原始 JSON

| 步骤 | 操作 | 预期 |
|------|------|------|
| 1 | 打开任意项目 | 检查整个界面 |
| 2 | 检查是否有 JSON 面板 | **无** |
| 3 | 检查是否有 JSON viewer | **无** |
| 4 | 检查是否有 Markdown 报告 | **无** |
| 5 | 检查文件统计展示形式 | 可读列表或简单图表，非 JSON key-value |

### 7.9 检查目标目录未变化

| 步骤 | 操作 | 预期 |
|------|------|------|
| 1 | 记录目标目录 `git status` | 扫描前状态 |
| 2 | 执行 `open_workspace` | 扫描完成 |
| 3 | 再次检查 `git status` | **无变化** |
| 4 | 检查目录 `mtime` | **未变化** |
| 5 | 检查是否有新增文件 | **无** |

## 8. 自动化测试规划

本轮不写测试代码，仅规划后续应实现的测试类型、覆盖范围和不覆盖范围。

### 8.1 Rust Unit Tests

**覆盖范围**：
- `workspace_scanner`：DFS 遍历、深度限制、文件计数
- `stage_detector`：阶段目录识别、命名异常检测、排序规则
- `file_classifier`：扩展名映射、测试文件模式匹配
- `external_ref_detector`：字符串匹配模式
- `validity_calculator`：validity 判定规则
- `safety_guard`：路径校验、symlink 检测、只读检查

**不覆盖**：
- Tauri command 层（由 integration tests 覆盖）
- React 前端（由 frontend tests 覆盖）
- 真实文件系统（使用 mock 或临时目录）

### 8.2 Rust Integration Tests

**覆盖范围**：
- `open_workspace` command 完整流程（输入 → Rust 扫描 → JSON 输出）
- `select_stage` command 完整流程
- `CommandResult` 序列化/反序列化
- 边界条件：空目录、大文件、超时、symlink、权限不足
- 安全约束：只读验证（扫描前后目录状态对比）

**不覆盖**：
- React 前端渲染
- 真实 Tauri 窗口（使用 headless 测试）
- 跨平台差异（Linux/macOS/Windows 分别运行）

### 8.3 Frontend Component Tests

**覆盖范围**：
- `WorkspacePage`：初始状态、loading、成功展示
- `StagePanel`：阶段列表渲染、排序、状态标识
- `StageListItem`：点击行为、禁用状态
- `StageOverview`：空状态、文件列表分组
- `WarningList`：warning 渲染、折叠展开
- `ErrorBanner`：错误展示、重新选择按钮
- `AnalysisPlaceholderButton`：禁用状态

**不覆盖**：
- Tauri command 调用（mock 数据）
- 真实文件系统操作
- 端到端用户流程

### 8.4 Tauri Command Boundary Tests

**覆盖范围**：
- `invoke("open_workspace", { path })` 输入校验
- `invoke("select_stage", { rootPath, stageId })` 输入校验
- JSON 响应解析
- 错误码映射到前端状态

**不覆盖**：
- Rust 内部算法（由 unit tests 覆盖）
- React 组件渲染（由 component tests 覆盖）

### 8.5 UI Smoke Tests（Playwright 或等价）

**覆盖范围**：
- 端到端流程：打开应用 → 选择目录 → 查看概览 → 选择阶段 → 查看阶段概览
- 空目录、空阶段、命名异常阶段等边界场景
- UI 不出现 JSON viewer、Markdown report
- "开始分析"按钮禁用

**不覆盖**：
- Rust 内部逻辑（由 Rust tests 覆盖）
- 性能测试（单测/集成测试覆盖）
- 安全测试（由 Rust integration tests + 手工验证覆盖）

### 8.6 测试优先级

| 优先级 | 测试类型 | 原因 |
|--------|---------|------|
| P0 | Rust Unit Tests | 核心算法正确性，开发阶段即可运行 |
| P0 | Rust Integration Tests | Command 边界，确保前后端契约 |
| P1 | Frontend Component Tests | UI 状态正确性， mock 数据即可运行 |
| P1 | UI Smoke Tests | 端到端流程，需要完整构建 |
| P2 | Tauri Command Boundary Tests | 若 integration tests 已覆盖可降级 |

## 9. Phase 1 验收标准

以下 checklist 全部通过后，Phase 1 方可认为实现完成。

### 9.1 后端验收

- [ ] `open_workspace` 对标准项目返回正确的 `WorkspaceProfile`，`validity = likely_valid`
- [ ] `open_workspace` 对空目录返回 `validity = unlikely`，`error_codes[]` 含 `no_stage_found`
- [ ] `open_workspace` 对无阶段但有代码目录返回 `validity = uncertain`
- [ ] `open_workspace` 对路径不存在返回 `success = false`，`error_code = path_not_found`
- [ ] `open_workspace` 对非目录返回 `success = false`，`error_code = not_directory`
- [ ] `open_workspace` 对权限不足返回 `success = false`，`error_code = permission_denied`
- [ ] `open_workspace` 对 symlink 根路径返回 `success = false`，`error_code = permission_denied`
- [ ] `select_stage` 对可用阶段返回 `StageContext`，`files[]` 非空
- [ ] `select_stage` 对空阶段返回 `success = true`，`files[] = []`，`error_code = stage_empty`
- [ ] `select_stage` 对不可读阶段返回 `success = false`，`error_code = stage_unreadable`
- [ ] `stages[]` 排序正确，标准阶段在前，命名异常在后
- [ ] `missing` 阶段不插入 `stages[]`
- [ ] 大文件触发 `file_too_large` warning
- [ ] 扫描超时触发 `scan_timeout` warning
- [ ] 扫描中遇 symlink 跳过，不跟随
- [ ] 目标目录扫描前后 `mtime` 不变，`git status` 无变化
- [ ] Rust 代码不调用 write/create/remove/rename

### 9.2 前端验收

- [ ] 初始状态展示欢迎引导
- [ ] 打开项目后展示 workspace 名称、路径、validity 标识
- [ ] 阶段列表正确展示状态（`available`/`empty`/`naming_anomaly`/`unreadable`）
- [ ] `empty` 阶段可点击提示原因，不可选中
- [ ] `unreadable` 阶段可点击提示原因，不可选中
- [ ] `naming_anomaly` 阶段可选中并展示概览
- [ ] WarningList 正确展示非致命问题
- [ ] ErrorBanner 正确展示全局阻塞错误
- [ ] `stage_unreadable` 作为阶段级错误提示，不作为全局 ErrorBanner
- [ ] 阶段概览正确展示文件列表（按 `source_kind` 分组）
- [ ] 空阶段展示空状态说明（`stage_empty_view`）
- [ ] "开始分析"按钮禁用或占位，不触发 evidence 收集
- [ ] UI 不出现 JSON viewer、Markdown report、原始 JSON dump
- [ ] UI 不暗示已完成 evidence 收集或语义分析
- [ ] 所有面向用户文案为简体中文

### 9.3 安全验收

- [ ] 目标项目目录不被写入
- [ ] 不创建 / 修改 / 删除目标项目文件
- [ ] 不运行目标项目脚本
- [ ] 不运行 Vivado / synthesis / implementation / bitstream
- [ ] 根路径 symlink 被拒绝
- [ ] 扫描中 symlink 不跟随到外部

### 9.4 文档一致性验收

- [ ] 实现与 [`mvp-functional-contract.md`](../requirements/mvp-functional-contract.md) 一致
- [ ] 实现与 [`phase-1-data-and-api-contract.md`](../design/phase-1-data-and-api-contract.md) 一致
- [ ] 实现与 [`phase-1-scanner-detail-design.md`](../design/phase-1-scanner-detail-design.md) 一致
- [ ] 实现与 [`phase-1-workspace-and-stage-flow.md`](../ui-ux/phase-1-workspace-and-stage-flow.md) 一致

---

## 附录：快速引用

### Error Code 速查

| error_code | success | 阻塞 | recoverable | 前端行为 |
|-----------|---------|------|-------------|---------|
| `path_not_found` | false | 全局 | false | ErrorBanner，重新选择 |
| `not_directory` | false | 全局 | false | ErrorBanner，重新选择 |
| `permission_denied` | false | 全局 | false | ErrorBanner，重新选择 |
| `stage_unreadable` | false | 仅阶段 | false | 阶段级错误提示 |
| `no_stage_found` | true | 不阻塞 | true | 继续浏览 workspace |
| `stage_empty` | true | 仅阶段 | true | 展示空阶段说明，不进入分析 |
| `file_unreadable` | true | 不阻塞 | true | WarningList 展示 |
| `file_too_large` | true | 不阻塞 | true | WarningList 展示 |
| `scan_timeout` | true | 不阻塞 | true | WarningList 展示 |

### Validity 速查

| 条件 | validity |
|------|---------|
| 标准阶段 + Python/Verilog | `likely_valid` |
| 无阶段 + 有代码 | `uncertain` |
| 有阶段 + 无核心代码 | `uncertain` |
| 空目录 | `unlikely` |
| 仅文档 + 有阶段目录（空） | `uncertain` |
| 仅文档 + 无阶段目录 | `unlikely` |
