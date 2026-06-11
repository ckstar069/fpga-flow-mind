# Phase 1 数据结构与 API 契约

---
status: draft
updated: 2026-06-11
---

> 本文档是 Phase 1 数据结构与 API 契约，把 [`mvp-functional-contract.md`](../requirements/mvp-functional-contract.md) 中的需求对象落到 Rust/TypeScript/Tauri command 边界。
> 不写完整可编译代码，用伪 Rust struct / TypeScript interface 形式定义字段。

## 1. 设计目标

将需求契约中的 `workspace_profile.json` 和 `stage_context.json` 转化为：

- Rust 后端的数据结构（serde 序列化）
- TypeScript 前端的数据类型
- Tauri command 的输入输出签名
- 统一的错误和 warning 返回格式

## 2. 命名与序列化规则

| 规则 | 说明 |
|------|------|
| Rust 字段命名 | `snake_case`，通过 serde 输出 `snake_case` JSON |
| TypeScript 字段 | 与 JSON 字段保持一致（`snake_case`），不转 camelCase |
| 枚举值 | 小写 `snake_case`，如 `"likely_valid"`、`"naming_anomaly"` |
| 路径（Rust 内部）| `std::path::PathBuf` |
| 路径（JSON 输出）| 字符串绝对路径（UTF-8） |
| 时间戳 | ISO 8601 字符串 |
| 版本 | 字符串，MVP 为 `"1.0.0"` |

## 3. 枚举定义

所有枚举值与 [`mvp-functional-contract.md`](../requirements/mvp-functional-contract.md) 保持一致。

| 枚举 | Rust 变体 | JSON 值 |
|------|----------|---------|
| **WorkspaceValidity** | `LikelyValid` / `Uncertain` / `Unlikely` | `"likely_valid"` / `"uncertain"` / `"unlikely"` |
| **StageStatus** | `Available` / `Empty` / `Missing` / `NamingAnomaly` / `Unreadable` | `"available"` / `"empty"` / `"missing"` / `"naming_anomaly"` / `"unreadable"` |
| **SourceKind** | `PythonStage` / `Rtl` / `Test` / `Doc` / `Config` / `ExternalModule` | `"python_stage"` / `"rtl"` / `"test"` / `"doc"` / `"config"` / `"external_module"` |
| **Language** | `Python` / `Verilog` / `SystemVerilog` / `Markdown` / `Text` / `Json` / `Yaml` / `Toml` / `Unknown` | `"python"` / `"verilog"` / `"systemverilog"` / `"markdown"` / `"text"` / `"json"` / `"yaml"` / `"toml"` / `"unknown"` |
| **ErrorCode** (Phase 1 子集) | `PathNotFound` / `NotDirectory` / `PermissionDenied` / `NoStageFound` / `StageEmpty` / `StageUnreadable` / `FileUnreadable` / `FileTooLarge` / `ScanTimeout` | 对应 snake_case 字符串 |

> `mvp-functional-contract.md` 定义了 15 个错误码（含 `scan_timeout`），Phase 1 只使用上述 9 个，其余留到后续阶段。
> `StageStatus::Missing` 保留在枚举定义中（与功能契约的 `stage_status` 枚举一致），但 Phase 1 默认不在 `workspace_profile.stages[]` 中发出 `status=missing`；缺失阶段通过 `warnings[]` / `validity_reasons[]` 表示。

## 4. Rust 数据结构草案

```rust
struct WorkspaceProfile {
    workspace_name: String,
    root_path: String,              // 绝对路径字符串
    stages: Vec<StageSummary>,
    file_type_stats: HashMap<String, u64>,  // 扩展名 -> 数量
    external_refs: Vec<String>,
    validity: WorkspaceValidity,
    validity_reasons: Vec<String>,
    warnings: Vec<WorkspaceWarning>,
    error_codes: Vec<ErrorCode>,
    scan_timestamp: String,         // ISO 8601
    version: String,                // "1.0.0"
}

struct StageSummary {
    stage_id: String,
    source_path: String,            // 阶段目录绝对路径
    file_count: u64,
    status: StageStatus,
}

struct WorkspaceWarning {
    error_code: ErrorCode,
    message: String,
    source_path: Option<String>,
    related_stage_id: Option<String>,
    recoverable: bool,              // warning 始终为 true
}

struct StageContext {
    stage_id: String,
    source_path: String,
    files: Vec<StageFile>,
    external_deps: Vec<String>,
    upstream_refs: Vec<UpstreamRef>,
    error_code: Option<ErrorCode>,  // stage_empty / stage_unreadable
}

struct UpstreamRef {
    stage_id: String,
    interface_file_path: Option<String>,
    inferred: bool,
}

struct StageFile {
    source_path: String,
    language: Language,
    source_kind: SourceKind,
    size_bytes: Option<u64>,
}

struct CommandError {
    error_code: ErrorCode,
    message: String,
    recoverable: bool,
    details: Option<String>,
    source_path: Option<String>,
}

struct CommandResult<T> {
    success: bool,
    data: Option<T>,
    error: Option<CommandError>,
    warnings: Vec<WorkspaceWarning>,
}
```

> `CommandResult` 保证每个 command 返回格式统一：前端总是先检查 `success`，再读取 `data` 或 `error`。

## 5. TypeScript 类型草案

```typescript
type WorkspaceValidity = 'likely_valid' | 'uncertain' | 'unlikely';
type StageStatus = 'available' | 'empty' | 'missing' | 'naming_anomaly' | 'unreadable';
type SourceKind = 'python_stage' | 'rtl' | 'test' | 'doc' | 'config' | 'external_module';
type Language = 'python' | 'verilog' | 'systemverilog' | 'markdown' | 'text' | 'json' | 'yaml' | 'toml' | 'unknown';
type ErrorCode = 'path_not_found' | 'not_directory' | 'permission_denied' | 'no_stage_found' | 'stage_empty' | 'stage_unreadable' | 'file_unreadable' | 'file_too_large' | 'scan_timeout';

interface WorkspaceProfile {
  workspace_name: string;
  root_path: string;
  stages: StageSummary[];
  file_type_stats: Record<string, number>;
  external_refs: string[];
  validity: WorkspaceValidity;
  validity_reasons: string[];
  warnings: WorkspaceWarning[];
  error_codes: ErrorCode[];
  scan_timestamp: string;
  version: string;
}

interface StageSummary {
  stage_id: string;
  source_path: string;
  file_count: number;
  status: StageStatus;
}

interface WorkspaceWarning {
  error_code: ErrorCode;
  message: string;
  source_path?: string;
  related_stage_id?: string;
  recoverable: boolean;
}

interface StageContext {
  stage_id: string;
  source_path: string;
  files: StageFile[];
  external_deps: string[];
  upstream_refs: UpstreamRef[];
  error_code?: ErrorCode;
}

interface StageFile {
  source_path: string;
  language: Language;
  source_kind: SourceKind;
  size_bytes?: number;
}

interface UpstreamRef {
  stage_id: string;
  interface_file_path?: string;
  inferred: boolean;
}

interface CommandError {
  error_code: ErrorCode;
  message: string;
  recoverable: boolean;
  details?: string;
  source_path?: string;
}

interface CommandResult<T> {
  success: boolean;
  data?: T;
  error?: CommandError;
  warnings: WorkspaceWarning[];
}
```

## 6. Tauri command 设计

### `open_workspace`

```rust
#[tauri::command]
fn open_workspace(path: String) -> CommandResult<WorkspaceProfile>
```

| 项 | 说明 |
|----|------|
| **输入** | `path: String` — 用户选择的目录绝对路径 |
| **成功输出** | `CommandResult { success: true, data: Some(workspace_profile), warnings: [...] }` |
| **失败输出** | `CommandResult { success: false, error: Some(command_error), warnings: [...] }` |
| **warnings 返回** | 扫描中的非致命问题（文件过大、数量超限等）随 `warnings` 返回，不阻断成功 |
| **前端处理** | 检查 `success`；true 则更新 workspace 状态展示概览；false 则弹窗展示 `error.message`；无论成败都展示 `warnings` |

### `select_stage`

```rust
#[tauri::command]
fn select_stage(root_path: String, stage_id: String) -> CommandResult<StageContext>
```

| 项 | 说明 |
|----|------|
| **输入** | `root_path: String` — 项目根路径；`stage_id: String` — 选中阶段 |
| **成功输出** | `CommandResult { success: true, data: Some(stage_context), warnings: [...] }` |
| **失败输出** | `CommandResult { success: false, error: Some(command_error), warnings: [...] }` |
| **前端处理** | true 则展示阶段概览；false（如阶段不可读）提示选择其他阶段；`stage_empty` 时降级展示 |

## 7. 错误返回格式

`CommandError` 字段语义：

| 字段 | 必填 | 说明 |
|------|------|------|
| `error_code` | 是 | 机器可读的错误标识 |
| `message` | 是 | 用户可读的错误描述 |
| `recoverable` | 是 | `true` = 可强制继续；`false` = 必须重新选择 |
| `details` | 否 | 技术细节或调试信息 |
| `source_path` | 否 | 触发错误的文件或目录路径 |

**阻塞性 vs 可恢复错误**：

| error_code | recoverable | 前端行为 |
|-----------|-------------|---------|
| `path_not_found` | `false` | 弹窗提示，允许重新选择 |
| `not_directory` | `false` | 弹窗提示，允许重新选择 |
| `permission_denied` | `false` | 弹窗提示，允许重新选择 |
| `no_stage_found` | `true` | 显示"未识别到阶段"并提供"强制继续"按钮 |
| `stage_empty` | `true` | 阶段列表中灰色展示，点击提示"该阶段为空" |
| `stage_unreadable` | `false` | 禁用该阶段，提示选择其他阶段 |

**CommandResult success / failure 语义**：

`CommandResult.success` 决定前端走 `data` 分支还是 `error` 分支。以下按 error_code 明确语义：

| error_code | `success` | `data` | `error` | 说明 |
|-----------|-----------|--------|---------|------|
| `path_not_found` | `false` | `None` | `Some(CommandError)` | 路径校验失败，阻塞 |
| `not_directory` | `false` | `None` | `Some(CommandError)` | 路径校验失败，阻塞 |
| `permission_denied` | `false` | `None` | `Some(CommandError)` | 权限校验失败，阻塞 |
| `stage_unreadable` | `false` | `None` | `Some(CommandError)` | 阶段不可读，阻塞该阶段 |
| `no_stage_found` | `true` | `Some(WorkspaceProfile)` | `None` | 正常返回，`stages[]` 为空，`error_codes[]` 含 `no_stage_found` |
| `stage_empty` | `true` | `Some(StageContext)` | `None` | 正常返回，`files[]` 为空，`error_code` 字段为 `stage_empty` |
| `file_unreadable` | `true` | `Some(...)` | `None` | 文件不可读，仅进入 `warnings[]` |
| `file_too_large` | `true` | `Some(...)` | `None` | 文件过大，仅进入 `warnings[]` |
| `scan_timeout` | `true` | `Some(WorkspaceProfile)` | `None` | 扫描超时返回已收集结果，仅进入 `warnings[]` |

> 规则总结：路径校验类错误（`path_not_found`/`not_directory`/`permission_denied`/`stage_unreadable`）→ `success=false`；业务结果类（`no_stage_found`/`stage_empty`）→ `success=true` 携带 data；扫描过程中的非致命问题（`file_unreadable`/`file_too_large`/`scan_timeout`）→ `success=true`，仅出现在 `warnings[]`。

## 8. warnings[] 格式

`WorkspaceWarning` 与 `CommandError` 的区别：

| 维度 | Warning | Command Error |
|------|---------|---------------|
| 致命性 | 非致命，不阻断主流程 | 可能致命，阻断当前操作 |
| 出现场景 | 扫描过程中边扫描边记录 | 路径校验失败或阶段验证失败 |
| 前端展示 | 列表/图标形式展示在概览面板 | 弹窗或禁用状态 |
| `recoverable` | 始终 `true` | 视 `error_code` 而定 |

**典型 warning 示例**：

```json
{
  "error_code": "file_too_large",
  "message": "文件超过 5MB，仅读取前 100 行进行类型识别",
  "source_path": "/path/to/large_file.v",
  "related_stage_id": "L3",
  "recoverable": true
}
```

## 9. 前后端状态映射

前端 UI 状态由 `CommandResult` 驱动：

| UI 状态 | 触发条件 |
|---------|---------|
| `loading` | 调用 Tauri command 后、收到响应前 |
| `success` | `CommandResult.success = true` |
| `error` | `CommandResult.success = false` |
| `empty` | `success = true` 但 `stages[]` 为空 |
| `forced_continue_available` | `success = true` 且 `error_codes` 包含 `no_stage_found`，或 `validity` 为 `uncertain`/`unlikely` |
| `selected_stage` | 用户点击阶段后，等待 `select_stage` 响应 |

状态机：

```text
idle -> loading -> success / error / empty
success -> selected_stage -> stage_loaded / stage_error
empty -> forced_continue_available（用户可强制继续）
```

## 10. 与 active 功能契约的一致性

| 本文档定义 | 对应 `mvp-functional-contract.md` |
|-----------|-----------------------------------|
| `WorkspaceProfile` | `workspace_profile.json` 对象 |
| `StageSummary` | `stages[]` 中的条目 |
| `StageContext` | `stage_context.json` 对象 |
| `StageFile` | `stage_context.json` 的 `files[]` 条目 |
| `WorkspaceValidity` | `workspace_validity` 枚举 |
| `StageStatus` | `stage_status` 枚举（含 `missing`；Phase 1 不在 stages[] 中发出） |
| `SourceKind` | `source_kind` 枚举 |
| `Language` | `language` 枚举 |
| `ErrorCode` | `error_code` 枚举的 Phase 1 子集 |
| `file_type_stats` | `file_type_stats` 字段 |
| `external_refs` | `external_refs[]` 字段 |
| `upstream_refs` | `upstream_refs[]` 字段 |

**无冲突字段**：本文档所有字段均从 `mvp-functional-contract.md` 派生，未新增与需求契约冲突的字段。
