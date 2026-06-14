use tauri::Manager;

use crate::models::enums::ErrorCode;
use crate::models::error::{CommandError, CommandResult};
use crate::persistence::models::{SaveSessionResult, SessionState};
use crate::persistence::session_store::{SessionStore, SessionStoreError};

/// Tauri command：保存 session。
///
/// `session_id` 为 `None` 时自动创建新 session。
/// 所有磁盘写入均委托 `SessionStore`，仅允许写入 app-owned storage。
#[tauri::command]
pub fn save_session(
    session_id: Option<String>,
    session_state: SessionState,
    app_handle: tauri::AppHandle,
) -> CommandResult<SaveSessionResult> {
    let store = match build_session_store(&app_handle) {
        Ok(s) => s,
        Err(e) => return e,
    };
    execute_save_session(store, session_id, &session_state)
}

fn execute_save_session(
    store: SessionStore,
    session_id: Option<String>,
    session_state: &SessionState,
) -> CommandResult<SaveSessionResult> {
    match store.save_session(session_id, session_state) {
        Ok(result) => CommandResult {
            success: true,
            data: Some(result),
            error: None,
            warnings: vec![],
        },
        Err(SessionStoreError::InvalidSessionId { session_id }) => CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::InvalidSessionId,
                message: format!("非法 session_id: {}", session_id),
                recoverable: false,
                details: None,
                source_path: None,
            }),
            warnings: vec![],
        },
        Err(e) => CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::PersistFailed,
                message: format!("保存 session 失败: {}", e),
                recoverable: false,
                details: None,
                source_path: None,
            }),
            warnings: vec![],
        },
    }
}

fn build_session_store(
    app_handle: &tauri::AppHandle,
) -> Result<SessionStore, CommandResult<SaveSessionResult>> {
    let app_data_dir = app_handle.path().app_data_dir().map_err(|e| CommandResult {
        success: false,
        data: None,
        error: Some(CommandError {
            error_code: ErrorCode::PersistFailed,
            message: format!("无法获取 app_data_dir: {}", e),
            recoverable: false,
            details: None,
            source_path: None,
        }),
        warnings: vec![],
    })?;
    Ok(SessionStore::new(app_data_dir))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;

    use super::*;
    use crate::models::enums::{Language, SourceKind, StageStatus, WorkspaceValidity};
    use crate::models::workspace_profile::WorkspaceProfile;
    use crate::persistence::models::{GlobalUiState, LoadSessionStatus};

    fn sample_workspace_profile(root_path: &std::path::Path) -> WorkspaceProfile {
        WorkspaceProfile {
            workspace_name: "demo".to_string(),
            root_path: root_path.to_string_lossy().to_string(),
            stages: vec![crate::models::workspace_profile::StageSummary {
                stage_id: "L0".to_string(),
                source_path: root_path.join("L0").to_string_lossy().to_string(),
                file_count: 1,
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

    fn sample_stage_context() -> crate::models::stage_context::StageContext {
        crate::models::stage_context::StageContext {
            stage_id: "L0".to_string(),
            source_path: "/project/L0".to_string(),
            files: vec![crate::models::stage_context::StageFile {
                source_path: "/project/L0/top.py".to_string(),
                language: Language::Python,
                source_kind: SourceKind::PythonStage,
                size_bytes: Some(100),
            }],
            external_deps: vec![],
            upstream_refs: vec![],
            error_code: None,
        }
    }

    fn sample_evidence_collection() -> crate::evidence::models::EvidenceCollection {
        crate::evidence::models::EvidenceCollection {
            stage_id: "L0".to_string(),
            evidence_items: vec![],
            index_by_path: HashMap::new(),
            index_by_kind: HashMap::new(),
            index_by_symbol: HashMap::new(),
            warnings: vec![],
            stats: crate::evidence::models::EvidenceStats {
                files_processed: 0,
                files_skipped: 0,
                total_items: 0,
                items_by_kind: HashMap::new(),
                items_by_strength: HashMap::new(),
            },
            version: "1.0.0".to_string(),
        }
    }

    fn sample_session_state(root_path: &std::path::Path) -> SessionState {
        let mut stage_contexts = HashMap::new();
        stage_contexts.insert("L0".to_string(), sample_stage_context());
        let mut evidence_collections = HashMap::new();
        evidence_collections.insert("L0".to_string(), sample_evidence_collection());

        SessionState {
            workspace_profile: sample_workspace_profile(root_path),
            selected_stage_id: Some("L0".to_string()),
            stage_contexts,
            evidence_collections,
            understandings: HashMap::new(),
            view_graphs: HashMap::new(),
            qa_histories: HashMap::new(),
            ui_states: HashMap::new(),
            global_ui_state: Some(GlobalUiState {
                last_session_id: Some("sess-001".to_string()),
                last_root_path: Some(root_path.to_string_lossy().to_string()),
            }),
        }
    }

    fn workspace_with_top_py() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("top.py"), "def add(): pass").unwrap();
        tmp
    }

    fn make_store() -> (tempfile::TempDir, SessionStore) {
        let tmp = tempfile::tempdir().unwrap();
        let store = SessionStore::with_version(
            tmp.path().to_path_buf(),
            "0.1.0".to_string(),
        );
        (tmp, store)
    }

    #[test]
    fn save_session_command_returns_session_id() {
        let workspace = workspace_with_top_py();
        let (_tmp_dir, store) = make_store();
        let state = sample_session_state(workspace.path());

        let result = store.save_session(Some("sess-001".to_string()), &state).unwrap();
        assert_eq!(result.session_id, "sess-001");
        assert!(result.success);
    }

    #[test]
    fn load_session_command_source_unchanged() {
        let workspace = workspace_with_top_py();
        let (_tmp_dir, store) = make_store();
        let state = sample_session_state(workspace.path());

        store.save_session(Some("sess-001".to_string()), &state).unwrap();
        let result = store.load_session("sess-001").unwrap();

        assert!(result.success);
        assert_eq!(result.status, LoadSessionStatus::SourceUnchanged);
        assert_eq!(result.session_state.selected_stage_id, Some("L0".to_string()));
    }

    #[test]
    fn load_session_command_source_changed() {
        let workspace = workspace_with_top_py();
        let (_tmp_dir, store) = make_store();
        let state = sample_session_state(workspace.path());

        store.save_session(Some("sess-001".to_string()), &state).unwrap();
        fs::write(workspace.path().join("top.py"), "def add(): return 1").unwrap();

        let result = store.load_session("sess-001").unwrap();
        assert!(result.success);
        assert_eq!(result.status, LoadSessionStatus::SourceChanged);
        assert!(result.mismatch_reason.is_some());
        assert!(result.session_state.stage_contexts.contains_key("L0"));
    }

    #[test]
    fn load_session_command_source_missing() {
        let workspace = workspace_with_top_py();
        let (_tmp_dir, store) = make_store();
        let state = sample_session_state(workspace.path());

        store.save_session(Some("sess-001".to_string()), &state).unwrap();
        fs::remove_dir_all(workspace.path()).unwrap();

        let result = store.load_session("sess-001").unwrap();
        assert!(result.success);
        assert_eq!(result.status, LoadSessionStatus::SourceMissing);
        assert!(result.session_state.stage_contexts.contains_key("L0"));
    }

    #[test]
    fn load_session_command_source_path_not_allowed() {
        let workspace = workspace_with_top_py();
        let (_tmp_dir, store) = make_store();
        let state = sample_session_state(workspace.path());

        store.save_session(Some("sess-001".to_string()), &state).unwrap();

        let other = tempfile::tempdir().unwrap();
        fs::remove_dir_all(workspace.path()).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(other.path(), workspace.path()).unwrap();
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_dir;
            symlink_dir(other.path(), workspace.path()).unwrap();
        }

        let result = store.load_session("sess-001").unwrap();
        assert!(result.success);
        assert_eq!(result.status, LoadSessionStatus::SourcePathNotAllowed);
        assert!(result.session_state.stage_contexts.contains_key("L0"));
    }

    #[test]
    fn load_session_command_not_found() {
        let (_tmp_dir, store) = make_store();
        let result = store.load_session("sess-missing").unwrap_err();
        assert!(
            matches!(result, SessionStoreError::SessionNotFound { .. }),
            "应为 SessionNotFound"
        );
    }

    #[test]
    fn save_session_invalid_id_returns_command_error() {
        let workspace = workspace_with_top_py();
        let (_tmp_dir, store) = make_store();
        let state = sample_session_state(workspace.path());

        let result = execute_save_session(store, Some("bad/id".to_string()), &state);

        assert!(!result.success);
        let error = result.error.expect("应有错误");
        assert_eq!(error.error_code, ErrorCode::InvalidSessionId);
        assert_eq!(error.message, "非法 session_id: bad/id");
    }
}
