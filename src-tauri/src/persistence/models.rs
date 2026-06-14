use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::evidence::models::EvidenceCollection;
use crate::models::stage_context::StageContext;
use crate::models::workspace_profile::WorkspaceProfile;
use crate::trace::models::{GroundedAnswer, SelectedTraceTarget, SourceExcerpt, TraceRefResolved};
use crate::understanding::models::ImplementationUnderstanding;
use crate::views::models::{ViewGraph, ViewType};

/// 持久化存储格式版本号。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageVersion {
    /// 主版本号
    pub major: u32,
    /// 次版本号
    pub minor: u32,
    /// 补丁版本号
    pub patch: u32,
}

impl StorageVersion {
    /// Phase 6 MVP 支持的存储格式版本。
    pub const CURRENT: Self = Self {
        major: 1,
        minor: 0,
        patch: 0,
    };
}

/// 一次分析会话的完整清单。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionManifest {
    pub session_id: String,
    pub storage_version: StorageVersion,
    pub created_at: String,
    pub updated_at: String,
    /// 应用版本号（Tauri app version）。
    pub app_version: String,
    pub persisted_workspace: PersistedWorkspace,
    /// 本次会话分析过的阶段列表。
    pub stages: Vec<PersistedStageSummary>,
    /// 当前选中的阶段，加载后恢复。
    pub selected_stage_id: Option<String>,
    /// 可选：全局 UI 状态（最近使用视图等）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_ui_state: Option<GlobalUiState>,
}

/// 持久化的目标项目信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedWorkspace {
    pub workspace_name: String,
    /// 目标项目根路径（绝对路径，保存时原始输入）。
    pub root_path: String,
    /// 目标项目 canonical 路径。
    pub canonical_root_path: String,
    /// 目标项目 fingerprint（关键文件 checksum 集合的哈希）。
    pub fingerprint: String,
    /// fingerprint 生成时使用的算法。
    pub fingerprint_algorithm: String,
    /// 保存时 WorkspaceProfile 的 artifact 相对路径。
    pub workspace_profile_path: String,
}

/// 单个阶段在 manifest 中的摘要。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedStageSummary {
    pub stage_id: String,
    pub stage_name: String,
    pub artifacts: ArtifactIndex,
    pub last_analyzed_at: String,
}

/// 单个阶段各 artifact 的相对路径索引。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArtifactIndex {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage_context_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_collection_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub understanding_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_graphs_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qa_history_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_state_path: Option<String>,
}

/// 单个阶段的完整产物集合（反序列化后的内存形态）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersistedStageArtifacts {
    pub stage_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage_context: Option<StageContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_collection: Option<EvidenceCollection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub understanding: Option<ImplementationUnderstanding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_graphs: Option<Vec<ViewGraph>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qa_history: Option<QaHistory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_state: Option<PersistedUiState>,
}

/// 单阶段 Q&A 历史。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QaHistory {
    pub stage_id: String,
    pub entries: Vec<QaHistoryEntry>,
    pub version: String,
}

/// 一条 Q&A 记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QaHistoryEntry {
    pub entry_id: String,
    pub timestamp: String,
    pub question: String,
    pub answer: GroundedAnswer,
    /// 提问时是否关联了 selected_target。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_target_kind: Option<String>,
}

/// 单阶段 UI 状态。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersistedUiState {
    pub stage_id: String,
    /// 当前选中的 trace target。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_trace_target: Option<SelectedTraceTarget>,
    /// 已解析的 trace 列表。
    pub resolved_traces: Vec<TraceRefResolved>,
    /// 当前打开的 source excerpt。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_source_excerpt: Option<SourceExcerpt>,
    /// 当前高亮的 evidence_id。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlighted_evidence_id: Option<String>,
    /// 当前激活的视图 tab（structure/dataflow/timing）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_view_type: Option<ViewType>,
}

/// 跨会话全局 UI 状态。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GlobalUiState {
    /// 最后选中的 session_id。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_session_id: Option<String>,
    /// 最后打开的路径（仅用于展示，加载前需重新校验）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_root_path: Option<String>,
}

/// load_session 成功执行后的目标项目状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadSessionStatus {
    SourceUnchanged,
    SourceChanged,
    SourceMissing,
    SourcePathNotAllowed,
}

/// load_session 命令的业务结果。
///
/// 阻塞性错误（session 不存在、manifest 损坏、版本不兼容等）通过
/// `CommandResult::Err` 返回，不会进入此结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadSessionResult {
    pub success: bool,
    pub status: LoadSessionStatus,
    pub session_state: SessionState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mismatch_reason: Option<String>,
    pub warnings: Vec<String>,
}

/// 保存会话命令的结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSessionResult {
    pub session_id: String,
    pub saved_at: String,
    pub success: bool,
}

/// 会话列表中的摘要信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub workspace_name: String,
    pub root_path: String,
    pub updated_at: String,
    pub stage_count: usize,
}

/// 一次会话的完整运行时状态。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub workspace_profile: WorkspaceProfile,
    pub selected_stage_id: Option<String>,
    pub stage_contexts: HashMap<String, StageContext>,
    pub evidence_collections: HashMap<String, EvidenceCollection>,
    pub understandings: HashMap<String, ImplementationUnderstanding>,
    pub view_graphs: HashMap<String, Vec<ViewGraph>>,
    pub qa_histories: HashMap<String, QaHistory>,
    pub ui_states: HashMap<String, PersistedUiState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub global_ui_state: Option<GlobalUiState>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::enums::{StageStatus, WorkspaceValidity};

    fn sample_workspace_profile() -> WorkspaceProfile {
        WorkspaceProfile {
            workspace_name: "demo".to_string(),
            root_path: "/project".to_string(),
            stages: vec![crate::models::workspace_profile::StageSummary {
                stage_id: "L0".to_string(),
                source_path: "/project/L0".to_string(),
                file_count: 3,
                status: StageStatus::Available,
            }],
            file_type_stats: HashMap::new(),
            external_refs: vec![],
            validity: WorkspaceValidity::LikelyValid,
            validity_reasons: vec![],
            warnings: vec![],
            error_codes: vec![],
            scan_timestamp: "2026-06-14T00:00:00Z".to_string(),
            version: "1.0.0".to_string(),
        }
    }

    fn sample_manifest() -> SessionManifest {
        SessionManifest {
            session_id: "sess-001".to_string(),
            storage_version: StorageVersion::CURRENT,
            created_at: "2026-06-14T00:00:00Z".to_string(),
            updated_at: "2026-06-14T00:00:00Z".to_string(),
            app_version: "0.1.0".to_string(),
            persisted_workspace: PersistedWorkspace {
                workspace_name: "demo".to_string(),
                root_path: "/project".to_string(),
                canonical_root_path: "/project".to_string(),
                fingerprint: "abc123".to_string(),
                fingerprint_algorithm: "sha256:file-list:v1".to_string(),
                workspace_profile_path: "workspace_profile.json".to_string(),
            },
            stages: vec![PersistedStageSummary {
                stage_id: "L0".to_string(),
                stage_name: "L0 阶段".to_string(),
                artifacts: ArtifactIndex {
                    stage_context_path: Some("stage_contexts/L0.json".to_string()),
                    evidence_collection_path: Some("evidence_collections/L0.json".to_string()),
                    understanding_path: None,
                    view_graphs_path: None,
                    qa_history_path: None,
                    ui_state_path: None,
                },
                last_analyzed_at: "2026-06-14T00:00:00Z".to_string(),
            }],
            selected_stage_id: Some("L0".to_string()),
            global_ui_state: Some(GlobalUiState {
                last_session_id: Some("sess-001".to_string()),
                last_root_path: Some("/project".to_string()),
            }),
        }
    }

    #[test]
    fn storage_version_current_is_100() {
        let v = StorageVersion::CURRENT;
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
    }

    #[test]
    fn storage_version_roundtrip() {
        let v = StorageVersion {
            major: 1,
            minor: 2,
            patch: 3,
        };
        let json = serde_json::to_string(&v).unwrap();
        let back: StorageVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(v, back);
    }

    #[test]
    fn session_manifest_roundtrip() {
        let manifest = sample_manifest();
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let back: SessionManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(manifest.session_id, back.session_id);
        assert_eq!(manifest.storage_version, back.storage_version);
        assert_eq!(manifest.stages.len(), back.stages.len());
        assert_eq!(
            manifest.stages[0].artifacts.evidence_collection_path,
            back.stages[0].artifacts.evidence_collection_path
        );
        assert_eq!(manifest.global_ui_state, back.global_ui_state);
    }

    #[test]
    fn artifact_index_omits_none_fields() {
        let index = ArtifactIndex {
            stage_context_path: Some("a.json".to_string()),
            evidence_collection_path: None,
            understanding_path: None,
            view_graphs_path: None,
            qa_history_path: None,
            ui_state_path: None,
        };
        let json = serde_json::to_string(&index).unwrap();
        assert!(json.contains("stage_context_path"));
        assert!(!json.contains("evidence_collection_path"));
    }

    #[test]
    fn load_session_status_serde_as_snake_case() {
        let status = LoadSessionStatus::SourceChanged;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"source_changed\"");

        let back: LoadSessionStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back, LoadSessionStatus::SourceChanged);
    }

    #[test]
    fn session_state_roundtrip_with_empty_collections() {
        let state = SessionState {
            workspace_profile: sample_workspace_profile(),
            selected_stage_id: Some("L0".to_string()),
            stage_contexts: HashMap::new(),
            evidence_collections: HashMap::new(),
            understandings: HashMap::new(),
            view_graphs: HashMap::new(),
            qa_histories: HashMap::new(),
            ui_states: HashMap::new(),
            global_ui_state: None,
        };
        let json = serde_json::to_string(&state).unwrap();
        let back: SessionState = serde_json::from_str(&json).unwrap();
        assert_eq!(state.selected_stage_id, back.selected_stage_id);
        assert_eq!(state.workspace_profile.root_path, back.workspace_profile.root_path);
        assert!(back.stage_contexts.is_empty());
    }

    #[test]
    fn save_session_result_roundtrip() {
        let result = SaveSessionResult {
            session_id: "sess-002".to_string(),
            saved_at: "2026-06-14T01:00:00Z".to_string(),
            success: true,
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: SaveSessionResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.session_id, back.session_id);
        assert!(back.success);
    }

    #[test]
    fn session_summary_roundtrip() {
        let summary = SessionSummary {
            session_id: "sess-003".to_string(),
            workspace_name: "demo".to_string(),
            root_path: "/project".to_string(),
            updated_at: "2026-06-14T02:00:00Z".to_string(),
            stage_count: 2,
        };
        let json = serde_json::to_string(&summary).unwrap();
        let back: SessionSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(summary.stage_count, back.stage_count);
    }
}
