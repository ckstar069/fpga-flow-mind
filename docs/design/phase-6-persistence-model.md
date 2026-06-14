# Phase 6 持久化数据模型

---
status: active
updated: 2026-06-14
---

> 本文档定义 Phase 6 持久化层的数据模型：SessionManifest、PersistedWorkspace、PersistedStageArtifacts、ArtifactIndex、StorageVersion 等 Rust/TypeScript 草案。
>
> 本文档 status 为 active，是 Phase 6 编码的实施依据之一。

## 1. 设计目标

- 将 Phase 1~5 的系统内产物持久化到 app-owned storage。
- 产物与目标项目分离存储，不污染目标 workspace。
- 通过 manifest 描述一次分析会话（session）的完整上下文。
- 支持版本号校验、目标项目 fingerprint 校验、artifact 按需加载。
- 不保存完整源码副本；如需缓存 source excerpt，必须限定范围并提供用户可清理策略。

## 2. 存储目录布局

```text
<app_data_dir>/
└── sessions/
    ├── <session_id_1>/
    │   ├── manifest.json
    │   ├── workspace_profile.json
    │   ├── stage_contexts/
    │   │   └── <stage_id>.json
    │   ├── evidence_collections/
    │   │   └── <stage_id>.json
    │   ├── understandings/
    │   │   └── <stage_id>.json
    │   ├── view_graphs/
    │   │   └── <stage_id>.json
    │   ├── qa_histories/
    │   │   └── <stage_id>.json
    │   └── ui_states/
    │       └── <stage_id>.json
    └── <session_id_2>/
        └── ...
```

**说明**：
- `session_id` 使用 UUID v4 或类似随机标识，不含目标项目路径信息。
- 每个 stage 的 artifact 独立成文件，避免单文件过大。
- manifest 记录 artifact 路径、版本、fingerprint、时间戳，不重复存储大对象内容。

## 3. Rust 数据模型草案

### 3.1 StorageVersion

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageVersion {
    /// 持久化存储格式主版本号
    pub major: u32,
    /// 持久化存储格式次版本号
    pub minor: u32,
    /// 持久化存储格式补丁版本号
    pub patch: u32,
}

impl StorageVersion {
    pub const CURRENT: StorageVersion = StorageVersion {
        major: 1,
        minor: 0,
        patch: 0,
    };
}
```

**兼容性规则**：
- `major` 一致且 `minor >=` 记录值 → 向后兼容，允许加载。
- `major` 不一致或 `minor <` 记录值 → 不兼容，拒绝加载。

### 3.2 SessionManifest

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionManifest {
    pub session_id: String,
    pub storage_version: StorageVersion,
    pub created_at: String,
    pub updated_at: String,
    /// 应用版本号（Tauri app version）
    pub app_version: String,
    pub persisted_workspace: PersistedWorkspace,
    /// 本次会话分析过的阶段列表
    pub stages: Vec<PersistedStageSummary>,
    /// 当前选中的阶段，加载后恢复
    pub selected_stage_id: Option<String>,
    /// 可选：全局 UI 状态（最近使用视图等）
    pub global_ui_state: Option<GlobalUiState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedWorkspace {
    pub workspace_name: String,
    /// 目标项目根路径（绝对路径）
    pub root_path: String,
    /// 目标项目 canonical 路径
    pub canonical_root_path: String,
    /// 目标项目 fingerprint（关键文件 checksum 集合的哈希）
    pub fingerprint: String,
    /// fingerprint 生成时使用的算法
    pub fingerprint_algorithm: String,
    /// 保存时 WorkspaceProfile 的 artifact 路径
    pub workspace_profile_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedStageSummary {
    pub stage_id: String,
    pub stage_name: String,
    pub artifacts: ArtifactIndex,
    pub last_analyzed_at: String,
}
```

### 3.3 ArtifactIndex

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactIndex {
    /// stage_context.json 的相对路径
    pub stage_context_path: Option<String>,
    /// evidence_collection.json 的相对路径
    pub evidence_collection_path: Option<String>,
    /// implementation_understanding.json 的相对路径
    pub understanding_path: Option<String>,
    /// view_graphs.json 的相对路径
    pub view_graphs_path: Option<String>,
    /// qa_history.json 的相对路径
    pub qa_history_path: Option<String>,
    /// ui_state.json 的相对路径
    pub ui_state_path: Option<String>,
}
```

### 3.4 PersistedStageArtifacts

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedStageArtifacts {
    pub stage_id: String,
    pub stage_context: Option<StageContext>,
    pub evidence_collection: Option<EvidenceCollection>,
    pub understanding: Option<ImplementationUnderstanding>,
    pub view_graphs: Option<Vec<ViewGraph>>,
    pub qa_history: Option<QaHistory>,
    pub ui_state: Option<PersistedUiState>,
}
```

### 3.5 QaHistory

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QaHistory {
    pub stage_id: String,
    pub entries: Vec<QaHistoryEntry>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QaHistoryEntry {
    pub entry_id: String,
    pub timestamp: String,
    pub question: String,
    pub answer: GroundedAnswer,
    /// 提问时是否关联了 selected_target
    pub selected_target_kind: Option<String>,
}
```

### 3.6 PersistedUiState

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedUiState {
    pub stage_id: String,
    /// 当前选中的 trace target
    pub selected_trace_target: Option<SelectedTraceTarget>,
    /// 已解析的 trace 列表
    pub resolved_traces: Vec<TraceRefResolved>,
    /// 当前打开的 source excerpt
    pub current_source_excerpt: Option<SourceExcerpt>,
    /// 当前高亮的 evidence_id
    pub highlighted_evidence_id: Option<String>,
    /// 当前激活的视图 tab（structure/dataflow/timing）
    pub active_view_type: Option<ViewType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalUiState {
    /// 最后选中的 session_id
    pub last_session_id: Option<String>,
    /// 最后打开的路径（仅用于展示，加载前需重新校验）
    pub last_root_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadSessionStatus {
    SourceUnchanged,
    SourceChanged,
    SourceMissing,
    SourcePathNotAllowed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadSessionResult {
    /// 命令是否成功执行；阻塞错误通过 CommandResult::Err 返回，不会进入此结构
    pub success: bool,
    pub status: LoadSessionStatus,
    /// 恢复后的完整会话状态；status 为 source_changed/source_missing/source_path_not_allowed 时仍存在
    pub session_state: SessionState,
    pub mismatch_reason: Option<String>,
    pub warnings: Vec<String>,
}
```

## 4. TypeScript 类型草案

```typescript
interface StorageVersion {
  major: number;
  minor: number;
  patch: number;
}

interface SessionManifest {
  session_id: string;
  storage_version: StorageVersion;
  created_at: string;
  updated_at: string;
  app_version: string;
  persisted_workspace: PersistedWorkspace;
  stages: PersistedStageSummary[];
  selected_stage_id?: string;
  global_ui_state?: GlobalUiState;
}

interface PersistedWorkspace {
  workspace_name: string;
  root_path: string;
  canonical_root_path: string;
  fingerprint: string;
  fingerprint_algorithm: string;
  workspace_profile_path: string;
}

interface PersistedStageSummary {
  stage_id: string;
  stage_name: string;
  artifacts: ArtifactIndex;
  last_analyzed_at: string;
}

interface ArtifactIndex {
  stage_context_path?: string;
  evidence_collection_path?: string;
  understanding_path?: string;
  view_graphs_path?: string;
  qa_history_path?: string;
  ui_state_path?: string;
}

interface PersistedStageArtifacts {
  stage_id: string;
  stage_context?: StageContext;
  evidence_collection?: EvidenceCollection;
  understanding?: ImplementationUnderstanding;
  view_graphs?: ViewGraph[];
  qa_history?: QaHistory;
  ui_state?: PersistedUiState;
}

interface QaHistory {
  stage_id: string;
  entries: QaHistoryEntry[];
  version: string;
}

interface QaHistoryEntry {
  entry_id: string;
  timestamp: string;
  question: string;
  answer: GroundedAnswer;
  selected_target_kind?: string;
}

interface PersistedUiState {
  stage_id: string;
  selected_trace_target?: SelectedTraceTarget;
  resolved_traces: TraceRefResolved[];
  current_source_excerpt?: SourceExcerpt;
  highlighted_evidence_id?: string;
  active_view_type?: ViewType;
}

interface GlobalUiState {
  last_session_id?: string;
  last_root_path?: string;
}

/**
 * load_session 命令成功执行后的业务状态。
 * 注意：阻塞错误（session 不存在、manifest 损坏、版本不兼容等）通过 CommandResult 的 Err 返回，不会进入此结构。
 */
interface LoadSessionResult {
  success: boolean;
  status: LoadSessionStatus;
  session_state: SessionState;
  mismatch_reason?: string;
  warnings: string[];
}

type LoadSessionStatus =
  | "source_unchanged"
  | "source_changed"
  | "source_missing"
  | "source_path_not_allowed";
```

## 5. 字段稳定性说明

| 字段 | 是否必须稳定 | 说明 |
|------|-------------|------|
| `session_id` | 是 | 用于唯一标识 session，不可变更 |
| `storage_version` | 是 | 用于版本校验 |
| `root_path` / `canonical_root_path` | 是 | 用于定位目标项目，但允许加载时提示变更 |
| `fingerprint` | 是 | 用于变更检测 |
| `workspace_profile_path` | 是 | manifest 中记录相对路径 |
| `stage_context` / `evidence_collection` / `understanding` | 否 | 可重新生成；缺失时提示用户重新分析 |
| `view_graphs` | 否 | 可由 understanding 重新生成 |
| `qa_history` | 否 | 纯会话状态，可丢失 |
| `ui_state` | 否 | 纯 UI 状态，可丢失 |

## 6. 文件路径约束

- manifest 与 artifact 路径均使用相对于 `session_id` 目录的相对路径。
- 实际读取时由后端拼接为绝对路径，并校验其位于 app_data/sessions/ 下。
- 拒绝任何尝试跳出 app_data 目录的相对路径（如 `../`、`symlink`）。

## 7. Source excerpt cache（可选）

如需缓存 source excerpt 以加速加载：
- 必须限定每个 excerpt 最大 100 行 / 8192 字符。
- cache 文件必须保存在 app-owned storage 的 `excerpts/` 子目录下。
- cache 必须记录原始 source_path、line_range、evidence_id 和生成时间戳。
- 用户可通过“清理缓存”入口一键删除所有 excerpt cache。
- 默认不开启 excerpt cache；如开启，必须在 UI 中明示。

## 8. 版本号约定

- `StorageVersion` 描述整个持久化存储格式。
- 各 artifact（如 WorkspaceProfile、EvidenceCollection）保留自身版本号（如 `"1.0.0"`），用于 artifact 级校验。
- Phase 6 MVP 仅支持 `StorageVersion { major: 1, minor: 0, patch: 0 }`。

## 9. 安全边界

- 目标项目仍然只读。
- 持久化只写 app-owned storage，不写目标 workspace。
- 不把截图或临时验收项目写入仓库。
- 不保存敏感环境变量。
- 不默认保存完整源码副本。
- path traversal / symlink / root mismatch 必须拒绝。

## 10. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-14 | 初始 draft：定义 SessionManifest、PersistedWorkspace、ArtifactIndex、QaHistory、PersistedUiState、目录布局、版本规则、安全边界 | Claude |
