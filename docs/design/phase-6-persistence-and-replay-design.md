# Phase 6 持久化与回放后端设计

---
status: draft
updated: 2026-06-14
---

> 本文档定义 Phase 6 持久化与回放的后端概要设计、核心流程、Tauri commands、安全边界和原子性策略。
>
> 本文档为 draft，仅供评审与讨论，不得作为 Phase 6 编码唯一依据。本轮修复后仍需审核并转为 active，方可进入 Phase 6 编码。

## 1. 概要设计

### 1.1 组件分层

```text
前端 WorkspacePage
  │
  ▼
SessionStore 模块（Rust）
  ├── SessionManifestRepository ──→ manifest.json 读写
  ├── ArtifactRepository ─────────→ artifact JSON 文件读写
  ├── WorkspaceFingerprintService ─→ 目标项目 fingerprint 计算
  └── StorageVersionService ──────→ 版本校验与迁移
  │
  ▼
Tauri commands（save_session / load_session / list_sessions / delete_session / get_last_session）
```

### 1.2 组件职责

| 组件 | 职责 |
|------|------|
| `SessionStore` | 暴露持久化与回放的高层 API，协调各 repository 与服务 |
| `SessionManifestRepository` | 负责 `manifest.json` 的读取、写入、更新 |
| `ArtifactRepository` | 负责 artifact JSON 文件的原子写入与读取 |
| `WorkspaceFingerprintService` | 计算目标项目关键文件 checksum 的 fingerprint，用于变更检测 |
| `StorageVersionService` | 校验 `StorageVersion`，处理版本兼容与不兼容 |
| `SessionListService` | 扫描 app_data/sessions/ 目录，生成最近 session 列表 |

## 2. 数据流

### 2.1 保存 session

1. 前端在用户触发或关键状态变更后调用 `save_session(session_state)`。
2. 后端 `SessionStore` 接收当前完整状态：
   - `WorkspaceProfile`
   - 已分析阶段的 `StageContext`、`EvidenceCollection`、`ImplementationUnderstanding`、`ViewGraph[]`
   - 各阶段 `QaHistory` 和 `PersistedUiState`
   - 当前 `selected_stage_id`、`GlobalUiState`
3. `WorkspaceFingerprintService` 计算目标项目 fingerprint（关键文件集合的 SHA-256 哈希）。
4. `StorageVersionService` 确定当前 `StorageVersion`。
5. `ArtifactRepository` 将每个 artifact 写入临时文件，再 `rename` 到目标路径（原子写入）。
6. `SessionManifestRepository` 更新 `manifest.json`（同样使用临时文件 + rename）。
7. 返回 `SaveSessionResult { session_id, success, error? }`。

### 2.2 加载 session

1. 用户从最近列表选择 session，或应用启动时自动加载 `global_ui_state.last_session_id`。
2. 后端读取 `manifest.json`，校验 `StorageVersion`。
3. 校验目标项目路径是否存在、是否为目录、是否为 symlink 或越界路径。
4. 重新计算目标项目 fingerprint，与 `manifest.persisted_workspace.fingerprint` 比对，确定 `LoadSessionStatus`：
   - 未变更：`source_unchanged`。
   - 已变更：`source_changed`，附带 `mismatch_reason` 说明变更。
   - 路径不存在：`source_missing`。
   - 目标路径变为 symlink 或超出允许范围：`source_path_not_allowed`。
   
   以上四种状态均视为命令执行成功，返回 `CommandResult<LoadSessionResult>`（`LoadSessionResult.success = true`），并携带 `session_state`。前端根据 `status` 决定：正常恢复、仅查看历史产物、重新选择路径、或删除记录。
5. 按 `ArtifactIndex` 读取各 artifact 文件。
6. 校验 artifact 版本号与内部 ID 一致性。
7. 返回 `LoadSessionResult { success: true, status, session_state, mismatch_reason?, warnings }`。

### 2.3 列出 session

1. 后端扫描 `app_data/sessions/` 目录。
2. 对每个子目录尝试读取 `manifest.json`。
3. 过滤不可读或版本严重损坏的 session（仍可展示占位信息）。
4. 按 `updated_at` 倒序排列。
5. 返回 `SessionSummary[]`。

### 2.4 删除 session

1. 前端调用 `delete_session(session_id)`。
2. 后端校验目标路径位于 `app_data/sessions/<session_id>` 下。
3. 递归删除该目录。
4. 返回删除结果。

## 3. Tauri Commands 草案

### 3.1 save_session

```rust
#[tauri::command]
pub fn save_session(
    session_id: Option<String>,
    session_state: SessionState,
) -> CommandResult<SaveSessionResult> {
    // 保存 session 到 app-owned storage
}
```

| 维度 | 说明 |
|------|------|
| 输入 | 可选 `session_id`（首次保存时为 None）、当前完整 `SessionState` |
| 输出 | `SaveSessionResult { session_id, saved_at, success }` |
| 错误分支 | 存储空间不足、写入失败、路径安全校验失败 |
| 访问目标项目 | 只读（仅计算 fingerprint） |

### 3.2 load_session

```rust
#[tauri::command]
pub fn load_session(
    session_id: String,
) -> CommandResult<LoadSessionResult> {
    // 从 app-owned storage 加载 session
}
```

| 维度 | 说明 |
|------|------|
| 输入 | `session_id` |
| 输出 | `LoadSessionResult { success: true, status: LoadSessionStatus, session_state: SessionState, mismatch_reason?: String, warnings: Vec<String> }` |
| 错误分支 | session 不存在、manifest 损坏、版本不兼容、artifact 文件缺失/损坏、session 路径不在 app-owned storage 下 |
| 可恢复加载状态 | `status = source_unchanged / source_changed / source_missing / source_path_not_allowed`，均返回 `success=true` 并携带 `session_state` |

### 3.3 list_sessions

```rust
#[tauri::command]
pub fn list_sessions(
    limit: Option<u32>,
) -> CommandResult<Vec<SessionSummary>> {
    // 列出最近 session
}
```

| 维度 | 说明 |
|------|------|
| 输入 | 可选 `limit`（默认 10，最大 50） |
| 输出 | `SessionSummary[]` |
| 错误分支 | storage 目录不可读 |
| 访问目标项目 | 否 |

### 3.4 delete_session

```rust
#[tauri::command]
pub fn delete_session(
    session_id: String,
) -> CommandResult<DeleteSessionResult> {
    // 删除 session 目录
}
```

| 维度 | 说明 |
|------|------|
| 输入 | `session_id` |
| 输出 | `DeleteSessionResult { success }` |
| 错误分支 | session 不存在、路径不在 app-owned storage 下、删除失败 |
| 访问目标项目 | 否 |

### 3.5 get_last_session

```rust
#[tauri::command]
pub fn get_last_session() -> CommandResult<Option<SessionSummary>> {
    // 返回 global_ui_state 中记录的最近 session
}
```

### 3.6 LoadSessionResult 数据契约

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadSessionStatus {
    SourceUnchanged,
    SourceChanged,
    SourceMissing,
    SourcePathNotAllowed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadSessionResult {
    pub success: bool,
    pub status: LoadSessionStatus,
    pub session_state: SessionState,
    pub mismatch_reason: Option<String>,
    pub warnings: Vec<String>,
}
```

**语义约定**：

- `CommandResult<LoadSessionResult>` 返回 `success=true` 时，`LoadSessionResult.success` 恒为 `true`，`session_state` 必须存在。
- `status = source_changed / source_missing / source_path_not_allowed` 不表示命令失败，而是表示目标项目状态与保存时不一致；前端可据此展示“仅查看历史产物”“重新选择路径”“重新分析”等选项。
- 真正的阻塞错误（session 不存在、manifest 损坏、版本不兼容、artifact 缺失等）通过 `CommandResult` 的 `Err` 返回，前端无法获取 `session_state`。

## 4. 原子写入策略

所有写入操作遵循：

1. 在目标目录同级创建临时文件（如 `manifest.json.tmp`）。
2. 完整写入后 `fsync`（或等价操作）。
3. 通过 `rename` 覆盖目标文件。
4. 删除临时文件（如 rename 失败）。

此策略保证：
- 写入过程中崩溃不会留下半份 manifest。
- 读取方永远看到完整旧文件或完整新文件。

## 5. 路径安全

- `app_data_dir` 通过 Tauri `app_data_dir()` 获取。
- 所有 session 路径必须位于 `app_data_dir/sessions/` 下。
- `session_id` 只允许 `[a-zA-Z0-9_-]`，不得包含 `/`、`..`、`.` 开头。
- manifest 中的 artifact 相对路径必须解析到 `app_data_dir/sessions/<session_id>/` 下，不得通过 `../` 跳出。
- 任何路径 canonicalize 后若跳出 app_data，立即拒绝。
- 拒绝 symlink：session 目录本身、artifact 文件、父目录均不得为 symlink。

## 6. Schema 校验与版本处理

### 6.1 校验流程

1. 读取 `manifest.json`。
2. 反序列化为 `SessionManifest`。
3. 校验 `storage_version`：
   - `major` 一致且 `minor` 不高于当前 → 通过。
   - 否则返回 `storage_version_incompatible`。
4. 校验各 artifact 版本号（如 WorkspaceProfile.version、EvidenceCollection.version）。
5. 校验 artifact 内部 ID 与 manifest 中 `stage_id` 一致。

### 6.2 版本不兼容处理

- 不兼容版本：拒绝加载，前端提示用户删除旧 session 或升级应用。
- MVP 阶段不做自动迁移；仅记录最小迁移策略设计（如字段重命名映射表）。

## 7. 加载后状态恢复

### 7.1 恢复 WorkspacePage 状态

后端返回 `SessionState` 后，前端按以下顺序恢复：

1. 设置 `workspaceProfile`。
2. 设置 `selectedStageId`。
3. 恢复当前阶段的 `stageContext`、`evidence`、`understanding`、`views`。
4. 恢复 `selectedTraceTarget`、`resolvedTraces`、`sourceExcerpt`、`highlightedEvidenceId`。
5. 恢复 `qaHistory`（如保存了）。
6. 恢复 `activeViewType`。

### 7.2 不重新生成

- 加载成功后不自动重新扫描目标项目。
- 不自动重新调用 MockProvider。
- 用户如需刷新，必须显式点击“重新收集证据”或“重新生成理解”。

## 8. Fingerprint 策略

### 8.1 计算范围

fingerprint 计算目标项目中以下文件：
- 所有 `*.py`、`*.v`、`*.sv`、`*.md`、`*.json`、`*.yaml`、`*.toml` 文件。
- 排除二进制文件、超大文件、symlink。

### 8.2 算法

1. 收集范围内所有文件。
2. 对每个文件计算 SHA-256。
3. 按相对路径排序后拼接为字符串。
4. 对拼接结果再计算一次 SHA-256，作为 workspace fingerprint。

### 8.3 变更提示

- fingerprint 一致 → `source_unchanged`。
- fingerprint 不一致 → `source_changed`。
- 目标路径不存在 → `source_missing`。
- 目标路径变为 symlink 或越界 → `source_path_not_allowed`。

## 9. 错误码扩展

以下错误码导致 `CommandResult` 返回 `success=false`，前端无法获取 `session_state`：

| 错误码 | 场景 |
|--------|------|
| `persist_failed` | 保存时写入失败 |
| `load_failed` | 加载时读取失败（IO 错误） |
| `session_not_found` | session_id 不存在 |
| `manifest_corrupted` | manifest.json 损坏或无法解析 |
| `storage_version_incompatible` | 版本不兼容 |
| `artifact_version_incompatible` | artifact 版本不兼容 |
| `artifact_missing_or_corrupted` | artifact 文件缺失或无法解析 |
| `session_path_not_allowed` | session 路径不在 app-owned storage 下 |

以下状态属于 `LoadSessionResult.status`（`CommandResult.success=true`），前端可获取 `session_state`：

| 状态 | 场景 |
|------|------|
| `source_unchanged` | 目标项目未变更 |
| `source_changed` | 目标项目已变更 |
| `source_missing` | 目标项目路径不存在 |
| `source_path_not_allowed` | 目标路径为 symlink 或不在允许范围内 |

## 10. 安全边界

- 目标项目仍然只读。
- 持久化只写 app-owned storage，不写目标 workspace。
- 不保存敏感环境变量。
- 不默认保存完整源码副本。
- 拒绝 path traversal / symlink / root mismatch。

## 11. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-14 | 初始 draft：定义 SessionStore、commands、原子写入、路径安全、schema 校验、状态恢复、fingerprint 策略 | Claude |
