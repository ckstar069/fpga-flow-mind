# Phase 1 扫描与阶段识别详细设计

---
status: draft
updated: 2026-06-11
---

> 本文档是 Phase 1 扫描与阶段识别的详细设计，细化 `workspace-scanning-and-stage-detection.md` 中的扫描、识别、分类、判定算法。
> 必须覆盖 [`story-open-workspace.md`](../requirements/stories/story-open-workspace.md) WS-001~WS-007 和 [`story-select-stage.md`](../requirements/stories/story-select-stage.md) ST-001~ST-008 的功能点。

## 1. 设计目标

将 story 功能点映射为可实施的具体算法和流程：

- WS-001~WS-007：目录选择 → 路径校验 → 扫描 → 合法性判断 → profile 生成 → 概览展示 → 安全边界
- ST-001~ST-008：阶段列表展示 → 单阶段选择 → 阶段验证 → 上下文准备 → 概览展示 → 缺失处理 → 命名异常处理 → 空阶段处理

## 2. 功能点映射表

| Story 功能点 | 设计章节 | 输入 | 输出 | MVP 必须 |
|-------------|---------|------|------|---------|
| WS-001 目录选择 | §6 Tauri command | 用户选择的目录路径 | 路径字符串 | 是 |
| WS-002 路径可读性检查 | §4 路径校验算法 | 路径字符串 | 通过/错误码 | 是 |
| WS-003 目录结构扫描 | §5 扫描算法 | 验证通过的路径 | 文件元数据列表 | 是 |
| WS-004 合法性判断 | §9 validity 判定算法 | 扫描结果 | `WorkspaceValidity` | 是 |
| WS-005 生成 workspace profile | §3 open_workspace 流程 | 扫描结果 + 阶段识别结果 | `WorkspaceProfile` | 是 |
| WS-006 展示 workspace 概览 | §11 UI 映射 | `WorkspaceProfile` JSON | UI 状态更新 | 是 |
| WS-007 安全边界执行 | §4、§5、§6 | 扫描上下文 | 只读保证 | 是 |
| ST-001 阶段列表展示 | §8 阶段识别算法 | `WorkspaceProfile.stages[]` | 排序后的阶段列表 | 是 |
| ST-002 单阶段选择 | §10 select_stage 流程 | `stage_id` + `root_path` | 阶段选中状态 | 是 |
| ST-003 阶段有效性验证 | §10 select_stage 流程 | `stage_id` | 存在/可读/空判断 | 是 |
| ST-004 阶段上下文准备 | §10 select_stage 流程 | 阶段目录路径 | `StageContext` | 是 |
| ST-005 阶段概览展示 | §11 UI 映射 | `StageContext` JSON | UI 状态更新 | 是 |
| ST-006 阶段缺失处理 | §8 阶段识别算法 | 预期阶段 vs 实际阶段 | warnings + validity_reasons | 否 |
| ST-007 阶段命名异常处理 | §8 阶段识别算法 | 实际目录名 | `naming_anomaly` | 否 |
| ST-008 阶段为空处理 | §8 阶段识别算法 | 空目录 | `empty` | 是 |

## 3. open_workspace 详细流程

```text
1. 接收 path: String（来自 Tauri command）
2. Normalize path（消除相对路径、符号链接、尾部斜杠）
3. 路径校验（§4）：
   3.1 存在性检查 → path_not_found（阻塞）
   3.2 目录性检查 → not_directory（阻塞）
   3.3 可读性检查 → permission_denied（阻塞）
4. 初始化扫描上下文：
   - root_path
   - depth = 0
   - file_count = 0
   - total_scanned = 0
   - start_time = now()
   - warnings = []
5. 执行扫描（§5）：
   - DFS 遍历，受深度/数量/超时约束
   - 收集文件元数据（路径、扩展名、大小）
   - 跳过 symlink、二进制文件
   - 大文件仅读前 100 行
6. 分类文件（§6）：
   - 扩展名 → language / source_kind
   - 测试文件模式优先匹配
7. 识别外部模块引用（§7）：
   - 在 .py 文件中匹配 urban_wireless import 模式
   - 在配置文件中匹配路径包含 urban_wireless
8. 识别阶段（§8）：
   - 匹配标准/变体目录名
   - 判断 naming_anomaly / empty / unreadable
   - 判断 missing（不插入 stages[]）
9. 计算 file_type_stats
10. 计算 validity（§9）
11. 组装 WorkspaceProfile
12. 返回 CommandResult<WorkspaceProfile>
```

## 4. 路径校验算法

**输入**：用户选择的目录路径字符串
**输出**：校验通过 或 返回错误码

| 校验项 | 方法 | 失败错误码 | 阻塞性 |
|--------|------|-----------|--------|
| 路径非空 | `!path.is_empty()` | `not_directory`（视为无效） | 是 |
| 路径不是符号链接 | `std::fs::symlink_metadata(path)`，检查 `file_type.is_symlink()` | `permission_denied` | 是 |
| 路径存在 | `std::fs::metadata(path)` | `path_not_found` | 是 |
| 是目录 | `metadata.is_dir()` | `not_directory` | 是 |
| 可读 | `std::fs::read_dir(path)` 试探 | `permission_denied` | 是 |

**路径规范化**：
- 先用 `symlink_metadata` 检查根路径是否为符号链接；若是，按 `permission_denied` 拒绝（防止穿越到未授权目录）
- 对非 symlink 的路径，使用 `std::path::Path::canonicalize` 消除 `.` 和 `..`
- **不调用 `canonicalize` 处理符号链接本身**，避免 `canonicalize` 隐式跟随 symlink 导致路径穿越
- 去除尾部路径分隔符

**目标目录写入检查**：
- Rust 代码只使用 `read_dir`、`metadata`、`read_file` 操作
- 禁止调用 `write`、`create`、`remove`、`rename`
- 所有输出仅作为内存对象或写入 app-owned 目录

## 5. 扫描算法

**策略**：深度优先搜索（DFS），递归实现。

**边界约束**：

| 约束 | 值 | 超限处理 |
|------|-----|---------|
| 最大递归深度 | 3 | 跳过更深层次，记录 warning |
| 单目录文件上限 | 1000 | 记录 warning，跳过剩余文件（不抛错） |
| 总扫描文件上限 | 5000 | 记录 warning，停止扫描，返回已收集结果 |
| 扫描超时 | 30 秒 | 记录 `scan_timeout` warning，返回已收集结果 |
| 符号链接 | 不跟随 | 遇到 symlink 直接跳过 |
| 隐藏目录 | 扫描但不识别阶段 | 纳入文件统计，但阶段识别时忽略（除非明确匹配阶段模式） |

**单文件处理**：

```text
对于每个目录项：
  如果是目录：
    如果深度 >= 3：跳过，记录 warning
    否则：递归扫描
  如果是文件：
    如果 total_scanned >= 5000：停止扫描，记录 warning
    如果不可读：记录 warning，跳过
    如果是二进制（无文本特征）：跳过，不计入统计
    如果大小 > 5MB：记录 warning，仅读前 100 行用于类型识别
    否则：读取前 N 行（用于外部引用识别），记录元数据
```

**二进制文件判断**：
- 读前 8KB，检查是否包含大量 NUL 字节（> 10%）
- 或检查文件是否以已知二进制魔数开头
- 二进制文件跳过，不计入 `file_type_stats`

## 6. 文件分类算法

**扩展名到 language / source_kind 映射**（优先级从高到低）：

| 文件名模式 | 扩展名 | language | source_kind |
|-----------|--------|----------|-------------|
| `test_*.py` | `.py` | `python` | `test` |
| `*_tb.v` | `.v` | `verilog` | `test` |
| `*_test.v` | `.v` | `verilog` | `test` |
| `test_*.sv` | `.sv` | `systemverilog` | `test` |
| 普通 `.py` | `.py` | `python` | `python_stage` |
| 普通 `.v` | `.v` | `verilog` | `rtl` |
| 普通 `.sv` | `.sv` | `systemverilog` | `rtl` |
| `.vh` | `.vh` | `verilog` | `rtl` |
| `.md` | `.md` | `markdown` | `doc` |
| `.rst` | `.rst` | `text` | `doc` |
| `.txt` | `.txt` | `text` | `doc` |
| `.json` | `.json` | `json` | `config` |
| `.yaml` / `.yml` | `.yaml`/`.yml` | `yaml` | `config` |
| `.toml` | `.toml` | `toml` | `config` |
| 其他 | — | `unknown` | — |

**优先级规则**：
- 测试文件名模式优先于普通扩展名匹配。例如 `test_l0.py` 应标记为 `test`，不是 `python_stage`。
- `.v`、`.sv`、`.vh` 统一归入 RTL 相关 `source_kind`，`language` 按实际扩展名区分。

## 7. 外部模块引用识别

**策略**：正则/字符串匹配，不做 AST 解析。

**Python 文件中的匹配模式**：

```text
匹配以下字符串模式：
  - "from urban_wireless import"
  - "import urban_wireless"
  - "urban_wireless."
  - 路径字符串中包含 "urban_wireless"
```

**配置文件中的匹配**：
- `.json`、`.yaml`、`.toml` 中字符串值包含 `urban_wireless`
- 模块路径字符串中包含 `urban_wireless`

**结果处理**：
- 匹配到的模块名进入 `external_refs[]`（workspace 级别）
- 阶段级别的外部依赖进入 `stage_context.external_deps[]`
- 结果按文件路径归类，不展开深度依赖解析

**限制**：
- 不做 pip/依赖树解析
- 不做 Python AST import 分析
- 仅做文本层面的标志识别

## 8. 阶段识别算法

**候选阶段目录识别**（不区分大小写）：

| 类别 | 匹配模式 | 结果 status |
|------|---------|------------|
| 标准阶段 | `L0`~`L6`、`RTL` | `available`（若目录非空）/ `empty`（若目录为空） |
| 常见变体 | `rtl`、`rtl_final`、`hardware`、`fpga` | `naming_anomaly` |
| 命名异常 | 含 `rtl` 但不等于 `RTL`；含 `level` 加数字 | `naming_anomaly` |

**目录状态判定**：

```text
对于每个候选目录：
  如果不可读：status = unreadable
  否则如果为空（无任何文件/子目录）：status = empty
  否则如果命名匹配标准模式：status = available
  否则：status = naming_anomaly
```

**阶段缺失处理**：

```text
预期标准阶段集合 = {L0, L1, L2, L3, L4, L5, L6, RTL}
实际发现阶段集合 = 扫描中识别到的阶段目录
缺失阶段 = 预期集合 - 实际集合

对于每个缺失阶段：
  - 不插入 stages[]
  - 在 warnings[] 中记录：error_code = no_stage_found（或自定义 warning code），message = "预期阶段 X 未找到"
  - 在 validity_reasons[] 中追加说明
```

**排序规则**：

```text
stages[] 中的条目按以下顺序排列：
  1. L0 → L1 → L2 → L3 → L4 → L5 → L6 → RTL（标准阶段按自然顺序）
  2. 命名异常阶段按字典序排列，排在标准阶段之后
  3. unreadable / empty 阶段保留在对应位置，不单独分组
```

**file_count 计算**：
- 统计阶段目录下（递归深度 1）所有可识别文件的总量
- 不含子目录计数
- 二进制文件不计入

## 9. validity 判定算法

**输入**：扫描结果（stages[]、file_type_stats、external_refs）
**输出**：`WorkspaceValidity`

| 条件 | validity | 说明 |
|------|----------|------|
| 至少 1 个标准/变体阶段 且 存在 Python 或 Verilog/SystemVerilog 文件 | `likely_valid` | 符合 ai_project_template 特征 |
| 无可识别阶段 但 存在可分析代码文件（Python/Verilog/文档/配置） | `uncertain` | 可能是非标准结构或不完整项目 |
| 有阶段但无可分析代码文件 | `uncertain` | 阶段存在但内容异常 |
| 无阶段特征 且 无可分析代码 | `unlikely` | 可能不是 ai_project_template 项目 |

**规则细节**：
- `likely_valid` **不要求**同时存在 Python 和 Verilog。早期 L0/L1/L2 项目可能只有 Python。
- 同时存在 Python + Verilog 是**增强信号**，不是必要条件。
- 仅有文档/配置而无代码 = `uncertain`（有项目痕迹但无核心代码）。
- 空目录（无阶段、无代码、无文档）= `unlikely`。
- `uncertain` / `unlikely` 均允许用户**强制继续**。

## 10. select_stage 详细流程

```text
1. 接收 root_path: String, stage_id: String
2. 构造阶段完整路径：root_path + stage_id
3. 验证阶段目录：
   3.1 存在性 → 若不存在，返回 error（理论上不应发生，因 stages[] 只含已验证目录）
   3.2 可读性 → 若不可读，返回 stage_unreadable（阻塞）
4. 扫描阶段文件（深度 2）：
   - 收集文件列表
   - 分类每个文件的 language / source_kind
   - 识别外部依赖
   - 识别上游引用（推断模式：检查相邻阶段目录中的接口定义文件）
5. 如果 files[] 为空：
   - error_code = stage_empty（可恢复）
6. 组装 StageContext
7. 返回 CommandResult<StageContext>
```

**上游引用推断**（Phase 1 最小实现）：
- 检查前一阶段目录（如 L3 的前一阶段是 L2）是否存在 `interface_*.py`、`*_interface.v` 等文件
- 若存在，生成 `upstream_refs[]` 条目，`inferred = true`
- 不做精确接口匹配，仅做文件名模式推断

## 11. warnings / error_codes 生成规则

| 代码 | 来源 | `CommandResult.success` | 进入 warnings[] | 进入 error_codes[] | 阻塞 | UI 表现 |
|------|------|------------------------|----------------|-------------------|------|---------|
| `path_not_found` | 路径校验 | `false` | 否 | 是 | 是 | 弹窗"路径不存在" |
| `not_directory` | 路径校验 | `false` | 否 | 是 | 是 | 弹窗"请选择一个目录" |
| `permission_denied` | 路径校验 | `false` | 否 | 是 | 是 | 弹窗"无读权限" |
| `stage_unreadable` | select_stage | `false` | 否 | 是 | 是 | 禁用该阶段 |
| `no_stage_found` | 阶段识别 | `true` | 是（同时） | 是 | 否 | "未识别到阶段"提示 + 强制继续按钮 |
| `stage_empty` | select_stage | `true` | 是 | 是 | 否 | 阶段灰色展示，提示"该阶段为空" |
| `file_unreadable` | 扫描过程 | `true` | 是 | 否 | 否 | warning 列表中展示 |
| `file_too_large` | 扫描过程 | `true` | 是 | 否 | 否 | warning 列表中展示 |
| `scan_timeout` | 扫描过程 | `true` | 是 | 否 | 否 | warning 列表中展示 |

**说明**：
- 路径校验类错误（`path_not_found`/`not_directory`/`permission_denied`）和 `stage_unreadable` 返回 `success=false`，前端走 error 分支。
- 业务结果类（`no_stage_found`/`stage_empty`）返回 `success=true` 携带 data，前端正常展示但需处理空状态。
- 扫描过程中的非致命问题（`file_unreadable`/`file_too_large`/`scan_timeout`）返回 `success=true`，仅出现在 `warnings[]`。
- `no_stage_found` 同时进入 `warnings[]` 和 `error_codes[]`，因为既是扫描结果也是系统级异常码。
- `file_unreadable`、`file_too_large`、`scan_timeout` 只进入 `warnings[]`，不进入 `error_codes[]`（非系统级错误，仅影响单个文件）。

## 12. 边界条件

| 场景 | 处理策略 | validity | 特殊行为 |
|------|---------|----------|---------|
| 空目录（无阶段、无代码、无文档） | `stages[]` 为空，`no_stage_found` | `unlikely` | 允许强制继续 |
| 无阶段但有代码（仅 .py/.v） | `stages[]` 为空，`no_stage_found` | `uncertain` | 允许强制继续 |
| 阶段缺失（仅 L0/L3/RTL） | 缺失阶段不插入 `stages[]` | `uncertain` | warnings + validity_reasons 展示 |
| 命名异常（rtl_final/） | 插入 `stages[]`，`status = naming_anomaly` | 不影响 | 可点击选择 |
| 空阶段（L0/ 为空、L1/ 有文件） | L0 `status = empty`，L1 `status = available` | 视整体判定 | L0 灰色展示 |
| 不可读阶段 | `status = unreadable` | 视整体判定 | 禁用 |
| 大目录（单目录 2000+ 文件） | 截断至 1000，记录 warning | 不影响 | 不卡死 |
| symlink | 不跟随，跳过 | 不影响 | 不穿越 |
| 二进制文件 | 跳过，不计入统计 | 不影响 | 无提示（静默跳过） |
| 仅文档无代码（.md/.txt） | 有文件但无核心代码 | `uncertain` | 允许继续 |
| 仅 RTL 无 Python | 有标准阶段 + Verilog | `likely_valid` | 正常处理 |
| 仅 Python 无 RTL | 有标准阶段 + Python | `likely_valid` | 正常处理（早期 L0/L1/L2 常见） |

## 13. 测试映射

以下边界条件对应后续 `docs/testing/` 中应覆盖的测试样例，本轮不编写完整测试文档：

| 测试样例 | 对应边界条件 | 验证重点 |
|---------|------------|---------|
| 标准业务项目（L0~RTL，含 .py/.v） | 正常路径 | validity=likely_valid、排序正确、file_count 准确 |
| 无阶段但有代码 | 空目录 + 代码 | validity=uncertain、`no_stage_found`、允许强制继续 |
| 阶段缺失 | 部分预期阶段缺失 | 缺失阶段不插入 stages[]、validity=uncertain |
| 命名异常 | `rtl_final` / `hardware` | `status = naming_anomaly`、可点击选择 |
| 空阶段 | L0/ 为空 | `status = empty`、灰色展示 |
| 不可读路径 | 权限不足 | `permission_denied`、弹窗提示 |
| 大目录 | 单目录 2000+ 文件 | 不卡死、文件数上限 warning |
| 完全空目录 | 无任何内容 | validity=unlikely、`no_stage_found` |
| 仅 Python 项目 | L0/L1 只有 .py | validity=likely_valid、不误判 |
| 仅文档项目 | 只有 .md/.txt | validity=uncertain |
| symlink 目录 | 含符号链接 | 不跟随、不穿越 |
| 超大文件 | 单文件 > 5MB | `file_too_large` warning、仅读前 100 行 |

**测试数据来源**：使用临时目录构造样例，无需真实 Vivado 或业务项目。真实 `ai_project_template` 业务项目样例在 Phase 1~2 接入后补充回归测试。
