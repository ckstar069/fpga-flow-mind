use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use chrono::Utc;

use crate::persistence::artifact_repository::ArtifactRepository;
use crate::persistence::fingerprint_service::{
    FingerprintComparison, WorkspaceFingerprintService,
};
use crate::persistence::manifest_repository::{
    ManifestRepositoryError, SessionManifestRepository,
};
use crate::persistence::models::{
    ArtifactIndex, LoadSessionResult, LoadSessionStatus, PersistedStageArtifacts,
    PersistedStageSummary, PersistedWorkspace, SaveSessionResult, SessionManifest, SessionState,
    SessionSummary,
};

/// SessionStore 操作错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionStoreError {
    /// session_id 格式非法。
    InvalidSessionId { session_id: String },
    /// 会话不存在。
    SessionNotFound { session_id: String },
    /// manifest 损坏。
    ManifestCorrupted { message: String },
    /// storage_version 不兼容。
    StorageVersionIncompatible { version: String },
    /// 保存失败。
    SaveError { message: String },
    /// 加载失败。
    LoadError { message: String },
    /// IO 错误。
    IoError { message: String },
}

impl std::fmt::Display for SessionStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionStoreError::InvalidSessionId { session_id } => {
                write!(f, "非法 session_id: {}", session_id)
            }
            SessionStoreError::SessionNotFound { session_id } => {
                write!(f, "会话不存在: {}", session_id)
            }
            SessionStoreError::ManifestCorrupted { message } => {
                write!(f, "manifest 损坏: {}", message)
            }
            SessionStoreError::StorageVersionIncompatible { version } => {
                write!(f, "不兼容的 storage_version: {}", version)
            }
            SessionStoreError::SaveError { message } => write!(f, "保存失败: {}", message),
            SessionStoreError::LoadError { message } => write!(f, "加载失败: {}", message),
            SessionStoreError::IoError { message } => write!(f, "IO 错误: {}", message),
        }
    }
}

impl std::error::Error for SessionStoreError {}

/// 会话持久化与回放的高层入口。
///
/// 职责：
/// - 协调 `ArtifactRepository`、`SessionManifestRepository`、`WorkspaceFingerprintService`
/// - 提供 `save_session`、`load_session`、`list_sessions`、`delete_session`
/// - `load_session` 对 fingerprint mismatch / 路径缺失 / 路径不安全返回
///   `success=true` 的可恢复状态，仍携带 `session_state`
pub struct SessionStore {
    app_data_dir: PathBuf,
    app_version: String,
}

impl SessionStore {
    /// 使用当前 crate 版本作为 app_version。
    pub fn new(app_data_dir: PathBuf) -> Self {
        Self {
            app_data_dir,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    #[cfg(test)]
    pub fn with_version(app_data_dir: PathBuf, app_version: String) -> Self {
        Self {
            app_data_dir,
            app_version,
        }
    }

    /// 保存会话。`session_id` 为 `None` 时自动生成新 id。
    pub fn save_session(
        &self,
        session_id: Option<String>,
        session_state: &SessionState,
    ) -> Result<SaveSessionResult, SessionStoreError> {
        let session_id = session_id.unwrap_or_else(|| Self::generate_session_id());
        Self::validate_session_id(&session_id)?;

        let sessions_dir = self.sessions_dir();
        std::fs::create_dir_all(&sessions_dir).map_err(|e| SessionStoreError::IoError {
            message: format!("无法创建 sessions 目录: {}", e),
        })?;
        let session_dir =
            SessionManifestRepository::session_dir(&sessions_dir, &session_id).map_err(|e| {
                match e {
                    ManifestRepositoryError::InvalidSessionId { .. } => {
                        SessionStoreError::InvalidSessionId {
                            session_id: session_id.clone(),
                        }
                    }
                    _ => SessionStoreError::SaveError {
                        message: format!("无法解析 session 目录: {}", e),
                    },
                }
            })?;

        std::fs::create_dir_all(&session_dir).map_err(|e| SessionStoreError::IoError {
            message: format!("无法创建 session 目录: {}", e),
        })?;

        let root_path = Path::new(&session_state.workspace_profile.root_path);
        let (fingerprint, fingerprint_algorithm) =
            self.compute_fingerprint(root_path).map_err(|e| {
                SessionStoreError::SaveError {
                    message: format!("无法计算 fingerprint: {}", e),
                }
            })?;

        let canonical_root_path = root_path
            .canonicalize()
            .map_err(|e| SessionStoreError::SaveError {
                message: format!("无法规范化目标路径: {}", e),
            })?
            .to_string_lossy()
            .to_string();

        let now = Utc::now().to_rfc3339();

        // 写入 workspace_profile artifact。
        let workspace_profile_path = "workspace_profile.json".to_string();
        ArtifactRepository::write_workspace_profile(
            &session_dir,
            &workspace_profile_path,
            &session_state.workspace_profile,
        )
        .map_err(|e| SessionStoreError::SaveError {
            message: format!("无法写入 workspace_profile: {}", e),
        })?;

        // 收集所有 stage_id。
        let mut stage_ids: HashSet<String> = HashSet::new();
        stage_ids.extend(session_state.stage_contexts.keys().cloned());
        stage_ids.extend(session_state.evidence_collections.keys().cloned());
        stage_ids.extend(session_state.understandings.keys().cloned());
        stage_ids.extend(session_state.view_graphs.keys().cloned());
        stage_ids.extend(session_state.qa_histories.keys().cloned());
        stage_ids.extend(session_state.ui_states.keys().cloned());
        if let Some(id) = &session_state.selected_stage_id {
            stage_ids.insert(id.clone());
        }

        let mut stages: Vec<PersistedStageSummary> = Vec::with_capacity(stage_ids.len());
        for stage_id in stage_ids {
            let (index, artifacts) = self.write_stage_artifacts(
                &session_dir,
                &stage_id,
                session_state,
            )?;
            stages.push(PersistedStageSummary {
                stage_id: stage_id.clone(),
                stage_name: stage_id.clone(),
                artifacts: index,
                last_analyzed_at: artifacts
                    .as_ref()
                    .map(|a| &a.stage_id)
                    .map_or_else(|| now.clone(), |_| now.clone()),
            });
        }

        let manifest = SessionManifest {
            session_id: session_id.clone(),
            storage_version: crate::persistence::models::StorageVersion::CURRENT,
            created_at: now.clone(),
            updated_at: now.clone(),
            app_version: self.app_version.clone(),
            persisted_workspace: PersistedWorkspace {
                workspace_name: session_state.workspace_profile.workspace_name.clone(),
                root_path: session_state.workspace_profile.root_path.clone(),
                canonical_root_path,
                fingerprint,
                fingerprint_algorithm,
                workspace_profile_path,
            },
            stages,
            selected_stage_id: session_state.selected_stage_id.clone(),
            global_ui_state: session_state.global_ui_state.clone(),
        };

        SessionManifestRepository::write(&sessions_dir, &session_id, &manifest,
        )
        .map_err(|e| SessionStoreError::SaveError {
            message: format!("无法写入 manifest: {}", e),
        })?;

        Ok(SaveSessionResult {
            session_id,
            saved_at: now,
            success: true,
        })
    }

    /// 加载会话。
    ///
    /// 阻塞性错误（不存在、损坏、版本不兼容）返回 `Err`；
    /// 目标项目变更 / 缺失 / 不安全返回 `Ok(LoadSessionResult { success: true, status, ... })`。
    pub fn load_session(
        &self,
        session_id: &str,
    ) -> Result<LoadSessionResult, SessionStoreError> {
        Self::validate_session_id(session_id)?;
        let sessions_dir = self.sessions_dir();
        let manifest = SessionManifestRepository::read(&sessions_dir, session_id,
        )
        .map_err(|e| match e {
            ManifestRepositoryError::DeserializationError { message } => {
                SessionStoreError::ManifestCorrupted { message }
            }
            ManifestRepositoryError::StorageVersionIncompatible { version } => {
                SessionStoreError::StorageVersionIncompatible { version }
            }
            ManifestRepositoryError::IoError { .. } => {
                SessionStoreError::SessionNotFound {
                    session_id: session_id.to_string(),
                }
            }
            _ => SessionStoreError::LoadError {
                message: e.to_string(),
            },
        })?;

        let session_dir =
            SessionManifestRepository::session_dir(&sessions_dir, session_id).map_err(|e| {
                SessionStoreError::LoadError {
                    message: e.to_string(),
                }
            })?;

        let workspace_profile = ArtifactRepository::read_workspace_profile(
            &session_dir,
            &manifest.persisted_workspace.workspace_profile_path,
        )
        .map_err(|e| SessionStoreError::LoadError {
            message: format!("无法读取 workspace_profile: {}", e),
        })?;

        let (status, mismatch_reason) = self.compare_fingerprint(
            &manifest.persisted_workspace.root_path,
            &manifest.persisted_workspace.fingerprint,
        );

        let mut stage_contexts = HashMap::new();
        let mut evidence_collections = HashMap::new();
        let mut understandings = HashMap::new();
        let mut view_graphs = HashMap::new();
        let mut qa_histories = HashMap::new();
        let mut ui_states = HashMap::new();

        for stage in &manifest.stages {
            let artifacts = ArtifactRepository::read_stage_artifacts(
                &session_dir,
                &stage.artifacts,
                &stage.stage_id,
            )
            .map_err(|e| SessionStoreError::LoadError {
                message: format!(
                    "无法读取 stage {} 的 artifact: {}",
                    stage.stage_id, e
                ),
            })?;

            if let Some(v) = artifacts.stage_context {
                stage_contexts.insert(stage.stage_id.clone(), v);
            }
            if let Some(v) = artifacts.evidence_collection {
                evidence_collections.insert(stage.stage_id.clone(), v);
            }
            if let Some(v) = artifacts.understanding {
                understandings.insert(stage.stage_id.clone(), v);
            }
            if let Some(v) = artifacts.view_graphs {
                view_graphs.insert(stage.stage_id.clone(), v);
            }
            if let Some(v) = artifacts.qa_history {
                qa_histories.insert(stage.stage_id.clone(), v);
            }
            if let Some(v) = artifacts.ui_state {
                ui_states.insert(stage.stage_id.clone(), v);
            }
        }

        let session_state = SessionState {
            workspace_profile,
            selected_stage_id: manifest.selected_stage_id,
            stage_contexts,
            evidence_collections,
            understandings,
            view_graphs,
            qa_histories,
            ui_states,
            global_ui_state: manifest.global_ui_state,
        };

        let mut warnings = Vec::new();
        if let Some(reason) = &mismatch_reason {
            warnings.push(reason.clone());
        }

        Ok(LoadSessionResult {
            success: true,
            status,
            session_state,
            mismatch_reason,
            warnings,
        })
    }

    /// 列出最近保存的会话，按 `updated_at` 倒序。
    pub fn list_sessions(
        &self,
        limit: Option<usize>,
    ) -> Result<Vec<SessionSummary>, SessionStoreError> {
        let sessions_dir = self.sessions_dir();
        if !sessions_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = std::fs::read_dir(&sessions_dir).map_err(|e| SessionStoreError::IoError {
            message: format!("无法读取 sessions 目录: {}", e),
        })?;

        let mut summaries = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| SessionStoreError::IoError {
                message: format!("无法读取目录项: {}", e),
            })?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let session_id = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            if session_id.is_empty() {
                continue;
            }

            match SessionManifestRepository::read(&sessions_dir, &session_id,
            ) {
                Ok(manifest) => {
                    summaries.push(SessionSummary {
                        session_id: manifest.session_id,
                        workspace_name: manifest.persisted_workspace.workspace_name,
                        root_path: manifest.persisted_workspace.root_path,
                        updated_at: manifest.updated_at,
                        stage_count: manifest.stages.len(),
                    });
                }
                Err(_) => {
                    // 损坏或版本不兼容的 session 过滤掉，不中断列表。
                }
            }
        }

        summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        if let Some(limit) = limit {
            summaries.truncate(limit);
        }
        Ok(summaries)
    }

    /// 删除指定会话及其所有 artifact。
    pub fn delete_session(
        &self,
        session_id: &str,
    ) -> Result<(), SessionStoreError> {
        Self::validate_session_id(session_id)?;
        let sessions_dir = self.sessions_dir();
        if !sessions_dir.exists() {
            return Err(SessionStoreError::SessionNotFound {
                session_id: session_id.to_string(),
            });
        }
        let session_dir = SessionManifestRepository::session_dir(
            &sessions_dir, session_id,
        )
        .map_err(|e| match e {
            ManifestRepositoryError::InvalidSessionId { .. } => {
                SessionStoreError::InvalidSessionId {
                    session_id: session_id.to_string(),
                }
            }
            _ => SessionStoreError::IoError {
                message: format!("无法解析 session 目录: {}", e),
            },
        })?;

        if !session_dir.exists() {
            return Err(SessionStoreError::SessionNotFound {
                session_id: session_id.to_string(),
            });
        }

        std::fs::remove_dir_all(&session_dir).map_err(|e| SessionStoreError::IoError {
            message: format!("无法删除 session 目录: {}", e),
        })?;
        Ok(())
    }

    fn sessions_dir(&self) -> PathBuf {
        self.app_data_dir.join("sessions")
    }

    fn generate_session_id() -> String {
        let nanos = Utc::now().timestamp_nanos_opt().unwrap_or(0);
        format!("sess-{}", nanos)
    }

    fn validate_session_id(session_id: &str) -> Result<(), SessionStoreError> {
        if session_id.is_empty() {
            return Err(SessionStoreError::InvalidSessionId {
                session_id: session_id.to_string(),
            });
        }
        for ch in session_id.chars() {
            if !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-' {
                return Err(SessionStoreError::InvalidSessionId {
                    session_id: session_id.to_string(),
                });
            }
        }
        Ok(())
    }

    fn compute_fingerprint(
        &self,
        root_path: &Path,
    ) -> Result<(String, String), crate::persistence::fingerprint_service::FingerprintError> {
        let fingerprint = WorkspaceFingerprintService::compute_fingerprint(root_path)?;
        Ok((
            fingerprint,
            WorkspaceFingerprintService::algorithm().to_string(),
        ))
    }

    fn compare_fingerprint(
        &self,
        root_path_str: &str,
        recorded: &str,
    ) -> (LoadSessionStatus, Option<String>) {
        let root_path = Path::new(root_path_str);
        match WorkspaceFingerprintService::compare(root_path, recorded) {
            FingerprintComparison::Unchanged => (LoadSessionStatus::SourceUnchanged, None),
            FingerprintComparison::Changed { reason } => {
                (LoadSessionStatus::SourceChanged, Some(reason))
            }
            FingerprintComparison::Missing => (
                LoadSessionStatus::SourceMissing,
                Some("目标项目路径不存在".to_string()),
            ),
            FingerprintComparison::NotAllowed { reason } => {
                (
                    LoadSessionStatus::SourcePathNotAllowed,
                    Some(reason),
                )
            }
        }
    }

    fn write_stage_artifacts(
        &self,
        session_dir: &Path,
        stage_id: &str,
        session_state: &SessionState,
    ) -> Result<(ArtifactIndex, Option<PersistedStageArtifacts>), SessionStoreError> {
        let mut index = ArtifactIndex::default();
        let mut artifacts = PersistedStageArtifacts {
            stage_id: stage_id.to_string(),
            ..Default::default()
        };

        if let Some(ctx) = session_state.stage_contexts.get(stage_id) {
            let path = format!("stage_contexts/{}.json", stage_id);
            ArtifactRepository::write_stage_context(session_dir, &path, ctx)
                .map_err(|e| SessionStoreError::SaveError {
                    message: format!("无法写入 stage_context: {}", e),
                })?;
            index.stage_context_path = Some(path);
            artifacts.stage_context = Some(ctx.clone());
        }

        if let Some(collection) = session_state.evidence_collections.get(stage_id) {
            let path = format!("evidence_collections/{}.json", stage_id);
            ArtifactRepository::write_evidence_collection(
                session_dir, &path, collection,
            )
            .map_err(|e| SessionStoreError::SaveError {
                message: format!("无法写入 evidence_collection: {}", e),
            })?;
            index.evidence_collection_path = Some(path);
            artifacts.evidence_collection = Some(collection.clone());
        }

        if let Some(u) = session_state.understandings.get(stage_id) {
            let path = format!("understandings/{}.json", stage_id);
            ArtifactRepository::write_understanding(session_dir, &path, u)
                .map_err(|e| SessionStoreError::SaveError {
                    message: format!("无法写入 understanding: {}", e),
                })?;
            index.understanding_path = Some(path);
            artifacts.understanding = Some(u.clone());
        }

        if let Some(graphs) = session_state.view_graphs.get(stage_id) {
            let path = format!("view_graphs/{}.json", stage_id);
            ArtifactRepository::write_view_graphs(session_dir, &path, graphs)
                .map_err(|e| SessionStoreError::SaveError {
                    message: format!("无法写入 view_graphs: {}", e),
                })?;
            index.view_graphs_path = Some(path);
            artifacts.view_graphs = Some(graphs.clone());
        }

        if let Some(history) = session_state.qa_histories.get(stage_id) {
            let path = format!("qa_histories/{}.json", stage_id);
            ArtifactRepository::write_qa_history(session_dir, &path, history)
                .map_err(|e| SessionStoreError::SaveError {
                    message: format!("无法写入 qa_history: {}", e),
                })?;
            index.qa_history_path = Some(path);
            artifacts.qa_history = Some(history.clone());
        }

        if let Some(state) = session_state.ui_states.get(stage_id) {
            let path = format!("ui_states/{}.json", stage_id);
            ArtifactRepository::write_ui_state(session_dir, &path, state)
                .map_err(|e| SessionStoreError::SaveError {
                    message: format!("无法写入 ui_state: {}", e),
                })?;
            index.ui_state_path = Some(path);
            artifacts.ui_state = Some(state.clone());
        }

        let has_any = index.stage_context_path.is_some()
            || index.evidence_collection_path.is_some()
            || index.understanding_path.is_some()
            || index.view_graphs_path.is_some()
            || index.qa_history_path.is_some()
            || index.ui_state_path.is_some();

        Ok((index, if has_any { Some(artifacts) } else { None }))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;

    use super::*;
    use crate::evidence::models::EvidenceCollection;
    use crate::models::enums::{Language, SourceKind, StageStatus, WorkspaceValidity};
    use crate::models::stage_context::StageContext;
    use crate::models::workspace_profile::WorkspaceProfile;
    use crate::persistence::models::GlobalUiState;
    use crate::understanding::models::ImplementationUnderstanding;

    fn sample_workspace_profile(root_path: &Path) -> WorkspaceProfile {
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

    fn sample_stage_context() -> StageContext {
        StageContext {
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

    fn sample_evidence_collection() -> EvidenceCollection {
        EvidenceCollection {
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

    fn sample_understanding() -> ImplementationUnderstanding {
        ImplementationUnderstanding {
            stage_id: "L0".to_string(),
            version: "3.0.0".to_string(),
            summary: crate::understanding::models::StageSummary {
                short: "short".to_string(),
                detailed: "detailed".to_string(),
            },
            claims: vec![],
            module_summaries: vec![],
            signal_summaries: vec![],
            interface_summaries: vec![],
            processing_steps: vec![],
            unknowns: vec![],
            evidence_gaps: vec![],
            generation_meta: crate::understanding::models::GenerationMeta {
                provider: "mock".to_string(),
                generated_at: "2026-06-14T00:00:00Z".to_string(),
                input_evidence_count: 0,
                generation_time_ms: 0,
                is_degraded: false,
            },
            stats: crate::understanding::models::UnderstandingStats {
                total_claims: 0,
                claims_by_confidence: HashMap::new(),
                claims_by_category: HashMap::new(),
                module_count: 0,
                signal_count: 0,
                interface_count: 0,
                processing_step_count: 0,
                unknown_count: 0,
                evidence_gap_count: 0,
            },
        }
    }

    fn sample_session_state(root_path: &Path) -> SessionState {
        let mut stage_contexts = HashMap::new();
        stage_contexts.insert("L0".to_string(), sample_stage_context());
        let mut evidence_collections = HashMap::new();
        evidence_collections.insert("L0".to_string(), sample_evidence_collection());
        let mut understandings = HashMap::new();
        understandings.insert("L0".to_string(), sample_understanding());

        SessionState {
            workspace_profile: sample_workspace_profile(root_path),
            selected_stage_id: Some("L0".to_string()),
            stage_contexts,
            evidence_collections,
            understandings,
            view_graphs: HashMap::new(),
            qa_histories: HashMap::new(),
            ui_states: HashMap::new(),
            global_ui_state: Some(GlobalUiState {
                last_session_id: Some("sess-001".to_string()),
                last_root_path: Some(root_path.to_string_lossy().to_string()),
            }),
        }
    }

    fn make_store() -> (tempfile::TempDir, SessionStore) {
        let tmp = tempfile::tempdir().unwrap();
        let store = SessionStore::with_version(
            tmp.path().to_path_buf(),
            "0.1.0".to_string(),
        );
        (tmp, store)
    }

    fn create_workspace() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("top.py"), "def add(): pass").unwrap();
        tmp
    }

    #[test]
    fn save_then_load_roundtrip() {
        let workspace = create_workspace();
        let (_tmp, store) = make_store();
        let state = sample_session_state(workspace.path());

        let saved = store.save_session(Some("sess-001".to_string()), &state).unwrap();
        assert_eq!(saved.session_id, "sess-001");
        assert!(saved.success);

        let loaded = store.load_session("sess-001").unwrap();
        assert!(loaded.success);
        assert_eq!(
            loaded.status,
            LoadSessionStatus::SourceUnchanged,
            "源项目未变更时应为 source_unchanged"
        );
        assert_eq!(
            loaded.session_state.selected_stage_id,
            Some("L0".to_string())
        );
        assert!(loaded.session_state.stage_contexts.contains_key("L0"));
        assert!(loaded.session_state.evidence_collections.contains_key("L0"));
        assert!(loaded.session_state.understandings.contains_key("L0"));
    }

    #[test]
    fn list_sessions_sorted_by_updated_at() {
        let workspace = create_workspace();
        let (_tmp, store) = make_store();
        let state = sample_session_state(workspace.path());

        store.save_session(Some("sess-a".to_string()), &state).unwrap();
        store.save_session(Some("sess-b".to_string()), &state).unwrap();
        store.save_session(Some("sess-c".to_string()), &state).unwrap();

        let list = store.list_sessions(None).unwrap();
        assert_eq!(list.len(), 3);
        // 按 updated_at 倒序。
        for i in 0..list.len() - 1 {
            assert!(list[i].updated_at >= list[i + 1].updated_at);
        }
    }

    #[test]
    fn delete_session_removes_storage() {
        let workspace = create_workspace();
        let (tmp, store) = make_store();
        let state = sample_session_state(workspace.path());

        store.save_session(Some("sess-001".to_string()), &state).unwrap();
        let session_dir = tmp.path().join("sessions").join("sess-001");
        assert!(session_dir.exists());

        store.delete_session("sess-001").unwrap();
        assert!(!session_dir.exists());
    }

    #[test]
    fn delete_session_not_found() {
        let (_tmp, store) = make_store();
        let result = store.delete_session("sess-missing");
        assert!(
            matches!(result.unwrap_err(), SessionStoreError::SessionNotFound { .. }),
            "不存在的 session 应返回 SessionNotFound"
        );
    }

    #[test]
    fn load_nonexistent_session() {
        let (_tmp, store) = make_store();
        let result = store.load_session("sess-missing");
        assert!(
            matches!(result.unwrap_err(), SessionStoreError::SessionNotFound { .. }),
            "不存在的 session 应返回 SessionNotFound"
        );
    }

    #[test]
    fn load_corrupted_manifest() {
        let (_tmp, store) = make_store();
        let sessions_dir = store.sessions_dir();
        let session_dir = sessions_dir.join("sess-001");
        fs::create_dir_all(&session_dir).unwrap();
        fs::write(session_dir.join("manifest.json"), "not json").unwrap();

        let result = store.load_session("sess-001");
        assert!(
            matches!(result.unwrap_err(), SessionStoreError::ManifestCorrupted { .. }),
            "损坏的 manifest 应返回 ManifestCorrupted"
        );
    }

    #[test]
    fn load_changed_source_returns_recoverable_status() {
        let workspace = create_workspace();
        let (_tmp, store) = make_store();
        let state = sample_session_state(workspace.path());

        store.save_session(Some("sess-001".to_string()), &state).unwrap();
        fs::write(workspace.path().join("top.py"), "def add(): return 1").unwrap();

        let loaded = store.load_session("sess-001").unwrap();
        assert!(loaded.success);
        assert_eq!(
            loaded.status,
            LoadSessionStatus::SourceChanged,
            "源项目变更时应为 source_changed"
        );
        assert!(
            loaded.mismatch_reason.is_some(),
            "应提供 mismatch_reason"
        );
        assert!(loaded.session_state.stage_contexts.contains_key("L0"));
    }

    #[test]
    fn load_missing_source_returns_recoverable_status() {
        let workspace = create_workspace();
        let (_tmp, store) = make_store();
        let state = sample_session_state(workspace.path());

        store.save_session(Some("sess-001".to_string()), &state).unwrap();
        fs::remove_dir_all(workspace.path()).unwrap();

        let loaded = store.load_session("sess-001").unwrap();
        assert!(loaded.success);
        assert_eq!(
            loaded.status,
            LoadSessionStatus::SourceMissing,
            "源项目缺失时应为 source_missing"
        );
        assert!(loaded.session_state.stage_contexts.contains_key("L0"));
    }

    #[test]
    fn load_unsafe_source_returns_recoverable_status() {
        let workspace = create_workspace();
        let (_tmp, store) = make_store();
        let state = sample_session_state(workspace.path());

        store.save_session(Some("sess-001".to_string()), &state).unwrap();

        // 将 workspace 替换为指向其他目录的 symlink。
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

        let loaded = store.load_session("sess-001").unwrap();
        assert!(loaded.success);
        assert_eq!(
            loaded.status,
            LoadSessionStatus::SourcePathNotAllowed,
            "源项目变为 symlink 时应为 source_path_not_allowed"
        );
        assert!(loaded.session_state.stage_contexts.contains_key("L0"));
    }

    #[test]
    fn load_incompatible_version() {
        let workspace = create_workspace();
        let (_tmp, store) = make_store();
        let state = sample_session_state(workspace.path());

        store.save_session(Some("sess-001".to_string()), &state).unwrap();

        let sessions_dir = store.sessions_dir();
        let session_dir = sessions_dir.join("sess-001");
        let content = fs::read_to_string(session_dir.join("manifest.json")).unwrap();
        let mut manifest: serde_json::Value = serde_json::from_str(&content).unwrap();
        manifest["storage_version"] = serde_json::json!({
            "major": 9,
            "minor": 0,
            "patch": 0,
        });
        fs::write(
            session_dir.join("manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let result = store.load_session("sess-001");
        assert!(
            matches!(
                result.unwrap_err(),
                SessionStoreError::StorageVersionIncompatible { .. }
            ),
            "不兼容版本应返回 StorageVersionIncompatible"
        );
    }
}
