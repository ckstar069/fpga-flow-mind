use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{de::DeserializeOwned, Serialize};

use crate::evidence::models::EvidenceCollection;
use crate::models::stage_context::StageContext;
use crate::models::workspace_profile::WorkspaceProfile;
use crate::persistence::models::{
    ArtifactIndex, GlobalUiState, PersistedStageArtifacts, PersistedUiState, QaHistory,
};
use crate::understanding::models::ImplementationUnderstanding;
use crate::views::models::ViewGraph;

/// ArtifactRepository 操作错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactRepositoryError {
    PathTraversal { path: String },
    AbsolutePath { path: String },
    InvalidPath { path: String },
    SymlinkNotAllowed { path: String },
    IoError { message: String },
    SerializationError { message: String },
    DeserializationError { message: String },
}

impl std::fmt::Display for ArtifactRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArtifactRepositoryError::PathTraversal { path } => {
                write!(f, "路径越界: {}", path)
            }
            ArtifactRepositoryError::AbsolutePath { path } => {
                write!(f, "不接受绝对路径: {}", path)
            }
            ArtifactRepositoryError::InvalidPath { path } => {
                write!(f, "非法路径: {}", path)
            }
            ArtifactRepositoryError::SymlinkNotAllowed { path } => {
                write!(f, "不允许符号链接: {}", path)
            }
            ArtifactRepositoryError::IoError { message } => write!(f, "IO 错误: {}", message),
            ArtifactRepositoryError::SerializationError { message } => write!(f, "序列化失败: {}", message),
            ArtifactRepositoryError::DeserializationError { message } => {
                write!(f, "反序列化失败: {}", message)
            }
        }
    }
}

impl std::error::Error for ArtifactRepositoryError {}

/// Artifact 原子写入与读取仓库。
pub struct ArtifactRepository;

impl ArtifactRepository {
    /// 写入 WorkspaceProfile artifact。
    pub fn write_workspace_profile(
        session_dir: &Path,
        path: &str,
        profile: &WorkspaceProfile,
    ) -> Result<(), ArtifactRepositoryError> {
        Self::write_json(session_dir, path, profile)
    }

    /// 读取 WorkspaceProfile artifact。
    pub fn read_workspace_profile(
        session_dir: &Path,
        path: &str,
    ) -> Result<WorkspaceProfile, ArtifactRepositoryError> {
        Self::read_json(session_dir, path)
    }

    /// 写入 StageContext artifact。
    pub fn write_stage_context(
        session_dir: &Path,
        path: &str,
        stage_context: &StageContext,
    ) -> Result<(), ArtifactRepositoryError> {
        Self::write_json(session_dir, path, stage_context)
    }

    /// 读取 StageContext artifact。
    pub fn read_stage_context(
        session_dir: &Path,
        path: &str,
    ) -> Result<StageContext, ArtifactRepositoryError> {
        Self::read_json(session_dir, path)
    }

    /// 写入 EvidenceCollection artifact。
    pub fn write_evidence_collection(
        session_dir: &Path,
        path: &str,
        collection: &EvidenceCollection,
    ) -> Result<(), ArtifactRepositoryError> {
        Self::write_json(session_dir, path, collection)
    }

    /// 读取 EvidenceCollection artifact。
    pub fn read_evidence_collection(
        session_dir: &Path,
        path: &str,
    ) -> Result<EvidenceCollection, ArtifactRepositoryError> {
        Self::read_json(session_dir, path)
    }

    /// 写入 ImplementationUnderstanding artifact。
    pub fn write_understanding(
        session_dir: &Path,
        path: &str,
        understanding: &ImplementationUnderstanding,
    ) -> Result<(), ArtifactRepositoryError> {
        Self::write_json(session_dir, path, understanding)
    }

    /// 读取 ImplementationUnderstanding artifact。
    pub fn read_understanding(
        session_dir: &Path,
        path: &str,
    ) -> Result<ImplementationUnderstanding, ArtifactRepositoryError> {
        Self::read_json(session_dir, path)
    }

    /// 写入 ViewGraph[] artifact。
    pub fn write_view_graphs(
        session_dir: &Path,
        path: &str,
        graphs: &[ViewGraph],
    ) -> Result<(), ArtifactRepositoryError> {
        Self::write_json(session_dir, path, graphs)
    }

    /// 读取 ViewGraph[] artifact。
    pub fn read_view_graphs(
        session_dir: &Path,
        path: &str,
    ) -> Result<Vec<ViewGraph>, ArtifactRepositoryError> {
        Self::read_json(session_dir, path)
    }

    /// 写入 QaHistory artifact。
    pub fn write_qa_history(
        session_dir: &Path,
        path: &str,
        history: &QaHistory,
    ) -> Result<(), ArtifactRepositoryError> {
        Self::write_json(session_dir, path, history)
    }

    /// 读取 QaHistory artifact。
    pub fn read_qa_history(
        session_dir: &Path,
        path: &str,
    ) -> Result<QaHistory, ArtifactRepositoryError> {
        Self::read_json(session_dir, path)
    }

    /// 写入 PersistedUiState artifact。
    pub fn write_ui_state(
        session_dir: &Path,
        path: &str,
        state: &PersistedUiState,
    ) -> Result<(), ArtifactRepositoryError> {
        Self::write_json(session_dir, path, state)
    }

    /// 读取 PersistedUiState artifact。
    pub fn read_ui_state(
        session_dir: &Path,
        path: &str,
    ) -> Result<PersistedUiState, ArtifactRepositoryError> {
        Self::read_json(session_dir, path)
    }

    /// 写入 GlobalUiState artifact（位于 session 根目录，通常不由 ArtifactIndex 引用）。
    pub fn write_global_ui_state(
        session_dir: &Path,
        path: &str,
        state: &GlobalUiState,
    ) -> Result<(), ArtifactRepositoryError> {
        Self::write_json(session_dir, path, state)
    }

    /// 读取 GlobalUiState artifact。
    pub fn read_global_ui_state(
        session_dir: &Path,
        path: &str,
    ) -> Result<GlobalUiState, ArtifactRepositoryError> {
        Self::read_json(session_dir, path)
    }

    /// 通用 JSON 写入：临时文件 + fsync + rename，保证原子性。
    pub fn write_json<T: Serialize + ?Sized>(
        session_dir: &Path,
        path: &str,
        value: &T,
    ) -> Result<(), ArtifactRepositoryError> {
        let target = Self::resolve_safe_path(session_dir, path)?;
        let tmp_path = Self::temp_path(&target).ok_or_else(|| ArtifactRepositoryError::InvalidPath {
            path: path.to_string(),
        })?;

        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ArtifactRepositoryError::IoError {
                message: format!("无法创建目录 {}: {}", parent.display(), e),
            })?;
        }

        let json = serde_json::to_string_pretty(value).map_err(|e| {
            ArtifactRepositoryError::SerializationError {
                message: format!("{}", e),
            }
        })?;

        {
            let mut file = std::fs::File::create(&tmp_path).map_err(|e| {
                ArtifactRepositoryError::IoError {
                    message: format!("无法创建临时文件 {}: {}", tmp_path.display(), e),
                }
            })?;
            file.write_all(json.as_bytes()).map_err(|e| {
                ArtifactRepositoryError::IoError {
                    message: format!("写入临时文件失败: {}", e),
                }
            })?;
            file.flush().map_err(|e| ArtifactRepositoryError::IoError {
                message: format!("刷新临时文件失败: {}", e),
            })?;
            #[cfg(not(target_os = "windows"))]
            {
                let _ = std::fs::File::open(&tmp_path)
                    .and_then(|f| f.sync_all());
            }
        }

        std::fs::rename(&tmp_path, &target).map_err(|e| ArtifactRepositoryError::IoError {
            message: format!("重命名失败 {} -> {}: {}", tmp_path.display(), target.display(), e),
        })?;

        // 尝试清理临时文件；rename 成功后通常不存在，失败不影响最终结果。
        if tmp_path.exists() {
            let _ = std::fs::remove_file(&tmp_path);
        }

        Ok(())
    }

    /// 通用 JSON 读取。
    pub fn read_json<T: DeserializeOwned>(
        session_dir: &Path,
        path: &str,
    ) -> Result<T, ArtifactRepositoryError> {
        let target = Self::resolve_safe_path(session_dir, path)?;
        Self::ensure_not_symlink(&target)?;

        let content = std::fs::read_to_string(&target).map_err(|e| {
            ArtifactRepositoryError::IoError {
                message: format!("无法读取 {}: {}", target.display(), e),
            }
        })?;

        serde_json::from_str(&content).map_err(|e| ArtifactRepositoryError::DeserializationError {
            message: format!("{}", e),
        })
    }

    /// 读取 stage 的全部 artifact 到一个 PersistedStageArtifacts。
    pub fn read_stage_artifacts(
        session_dir: &Path,
        index: &ArtifactIndex,
        stage_id: &str,
    ) -> Result<PersistedStageArtifacts, ArtifactRepositoryError> {
        let mut artifacts = PersistedStageArtifacts {
            stage_id: stage_id.to_string(),
            ..Default::default()
        };

        if let Some(p) = index.stage_context_path.as_deref() {
            artifacts.stage_context = Some(Self::read_stage_context(session_dir, p)?);
        }
        if let Some(p) = index.evidence_collection_path.as_deref() {
            artifacts.evidence_collection = Some(Self::read_evidence_collection(session_dir, p)?);
        }
        if let Some(p) = index.understanding_path.as_deref() {
            artifacts.understanding = Some(Self::read_understanding(session_dir, p)?);
        }
        if let Some(p) = index.view_graphs_path.as_deref() {
            artifacts.view_graphs = Some(Self::read_view_graphs(session_dir, p)?);
        }
        if let Some(p) = index.qa_history_path.as_deref() {
            artifacts.qa_history = Some(Self::read_qa_history(session_dir, p)?);
        }
        if let Some(p) = index.ui_state_path.as_deref() {
            artifacts.ui_state = Some(Self::read_ui_state(session_dir, p)?);
        }

        Ok(artifacts)
    }

    /// 校验 session_dir / path 组合后的安全路径。
    ///
    /// 规则：
    /// - path 必须是相对路径
    /// - path 不得包含 `..`、`.` 开头、空段、绝对路径前缀
    /// - 最终 canonical 路径必须位于 session_dir 内
    fn resolve_safe_path(session_dir: &Path, path: &str) -> Result<PathBuf, ArtifactRepositoryError> {
        if path.is_empty() {
            return Err(ArtifactRepositoryError::InvalidPath {
                path: path.to_string(),
            });
        }

        let rel = Path::new(path);
        if rel.is_absolute() {
            return Err(ArtifactRepositoryError::AbsolutePath {
                path: path.to_string(),
            });
        }

        // 显式拒绝 `.` 开头、`..`、空段。
        for component in rel.components() {
            match component {
                std::path::Component::Normal(s) => {
                    let seg = s.to_str().unwrap_or("");
                    if seg.is_empty() || seg == "." || seg.starts_with(".") {
                        return Err(ArtifactRepositoryError::InvalidPath {
                            path: path.to_string(),
                        });
                    }
                }
                std::path::Component::ParentDir => {
                    return Err(ArtifactRepositoryError::PathTraversal {
                        path: path.to_string(),
                    });
                }
                _ => {
                    return Err(ArtifactRepositoryError::InvalidPath {
                        path: path.to_string(),
                    });
                }
            }
        }

        // 在 canonicalize 之前检查 session_dir 本身是否为 symlink。
        // 调用方若传入 symlink，canonicalize 会解析为真实目录，导致原始 symlink 信息丢失。
        match std::fs::symlink_metadata(session_dir) {
            Ok(m) if m.file_type().is_symlink() => {
                return Err(ArtifactRepositoryError::SymlinkNotAllowed {
                    path: session_dir.display().to_string(),
                });
            }
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(ArtifactRepositoryError::IoError {
                    message: format!("无法读取 session 目录元数据 {}: {}", session_dir.display(), e),
                });
            }
        }

        let canonical_session = session_dir.canonicalize().map_err(|e| {
            ArtifactRepositoryError::IoError {
                message: format!("无法规范化 session 目录: {}", e),
            }
        })?;

        // 先检查 session_dir 本身及其路径上是否有 symlink。
        Self::ensure_path_chain_not_symlink(&canonical_session)?;

        let joined = canonical_session.join(rel);

        // 在 canonicalize 之前检查 joined 及其父路径链上是否存在 symlink。
        // 必须早于 canonicalize，因为 canonicalize 会解析 symlink。
        Self::ensure_chain_no_symlink_up_to(&joined, &canonical_session)?;

        let canonical_target = match joined.canonicalize() {
            Ok(p) => p,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Self::check_parent_within(
                    &canonical_session,
                    joined.parent().unwrap_or(Path::new("")),
                    path,
                );
            }
            Err(e) => {
                return Err(ArtifactRepositoryError::IoError {
                    message: format!("无法校验路径: {}", e),
                });
            }
        };

        if !canonical_target.starts_with(&canonical_session) {
            return Err(ArtifactRepositoryError::PathTraversal {
                path: path.to_string(),
            });
        }

        Ok(canonical_target)
    }

    fn check_parent_within(
        canonical_session: &Path,
        parent: &Path,
        original: &str,
    ) -> Result<PathBuf, ArtifactRepositoryError> {
        let canonical_parent = match parent.canonicalize() {
            Ok(p) => p,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // parent 不存在时，递归检查其 parent，直到 session_dir。
                if parent.as_os_str().is_empty() || parent == canonical_session {
                    return Ok(canonical_session.to_path_buf());
                }
                return Self::check_parent_within(
                    canonical_session,
                    parent.parent().unwrap_or(Path::new("")),
                    original,
                );
            }
            Err(e) => {
                return Err(ArtifactRepositoryError::IoError {
                    message: format!("无法校验父目录: {}", e),
                });
            }
        };
        if !canonical_parent.starts_with(canonical_session) {
            return Err(ArtifactRepositoryError::PathTraversal {
                path: original.to_string(),
            });
        }
        Ok(canonical_session.join(original))
    }

    fn ensure_path_chain_not_symlink(path: &Path) -> Result<(), ArtifactRepositoryError> {
        let mut current = Some(path);
        while let Some(p) = current {
            if p.as_os_str().is_empty() {
                break;
            }
            match std::fs::symlink_metadata(p) {
                Ok(m) if m.file_type().is_symlink() => {
                    return Err(ArtifactRepositoryError::SymlinkNotAllowed {
                        path: p.display().to_string(),
                    });
                }
                _ => {}
            }
            current = p.parent();
        }
        Ok(())
    }

    /// 检查 `path` 到 `stop_at`（含）之间的路径链上是否存在 symlink。
    /// 不存在的中间路径会被忽略，因为写入前允许父目录不存在。
    fn ensure_chain_no_symlink_up_to(
        path: &Path,
        stop_at: &Path,
    ) -> Result<(), ArtifactRepositoryError> {
        let mut current = Some(path);
        while let Some(p) = current {
            if p.as_os_str().is_empty() {
                break;
            }
            match std::fs::symlink_metadata(p) {
                Ok(m) if m.file_type().is_symlink() => {
                    return Err(ArtifactRepositoryError::SymlinkNotAllowed {
                        path: p.display().to_string(),
                    });
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => {
                    return Err(ArtifactRepositoryError::IoError {
                        message: format!("无法读取路径元数据 {}: {}", p.display(), e),
                    });
                }
                _ => {}
            }
            if p == stop_at {
                break;
            }
            current = p.parent();
        }
        Ok(())
    }

    fn ensure_not_symlink(path: &Path) -> Result<(), ArtifactRepositoryError> {
        let meta = std::fs::symlink_metadata(path).map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                ArtifactRepositoryError::IoError {
                    message: format!("文件不存在: {}", path.display()),
                }
            } else {
                ArtifactRepositoryError::IoError {
                    message: format!("无法读取元数据 {}: {}", path.display(), e),
                }
            }
        })?;

        if meta.file_type().is_symlink() {
            return Err(ArtifactRepositoryError::SymlinkNotAllowed {
                path: path.display().to_string(),
            });
        }

        if let Some(parent) = path.parent() {
            let parent_meta = std::fs::symlink_metadata(parent).map_err(|e| {
                ArtifactRepositoryError::IoError {
                    message: format!("无法读取父目录元数据 {}: {}", parent.display(), e),
                }
            })?;
            if parent_meta.file_type().is_symlink() {
                return Err(ArtifactRepositoryError::SymlinkNotAllowed {
                    path: parent.display().to_string(),
                });
            }
        }

        Ok(())
    }

    fn temp_path(target: &Path) -> Option<PathBuf> {
        let file_name = target.file_name()?.to_str()?;
        let parent = target.parent()?;
        Some(parent.join(format!(".{}.tmp", file_name)))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;

    use super::*;
    use crate::models::enums::{Language, SourceKind, StageStatus, WorkspaceValidity};

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

    fn sample_view_graphs() -> Vec<ViewGraph> {
        vec![]
    }

    #[test]
    fn write_and_read_workspace_profile() {
        let tmp = tempfile::tempdir().unwrap();
        let profile = sample_workspace_profile();
        ArtifactRepository::write_workspace_profile(tmp.path(), "workspace_profile.json", &profile)
            .unwrap();
        let read =
            ArtifactRepository::read_workspace_profile(tmp.path(), "workspace_profile.json")
                .unwrap();
        assert_eq!(profile.workspace_name, read.workspace_name);
        assert_eq!(profile.root_path, read.root_path);
    }

    #[test]
    fn write_and_read_stage_context() {
        let tmp = tempfile::tempdir().unwrap();
        let ctx = sample_stage_context();
        ArtifactRepository::write_stage_context(tmp.path(), "stage_contexts/L0.json", &ctx)
            .unwrap();
        let read = ArtifactRepository::read_stage_context(tmp.path(), "stage_contexts/L0.json")
            .unwrap();
        assert_eq!(ctx.stage_id, read.stage_id);
        assert_eq!(ctx.files.len(), read.files.len());
    }

    #[test]
    fn write_and_read_evidence_collection() {
        let tmp = tempfile::tempdir().unwrap();
        let collection = sample_evidence_collection();
        ArtifactRepository::write_evidence_collection(
            tmp.path(),
            "evidence_collections/L0.json",
            &collection,
        )
        .unwrap();
        let read = ArtifactRepository::read_evidence_collection(
            tmp.path(),
            "evidence_collections/L0.json",
        )
        .unwrap();
        assert_eq!(collection.stage_id, read.stage_id);
        assert_eq!(collection.version, read.version);
    }

    #[test]
    fn write_and_read_view_graphs() {
        let tmp = tempfile::tempdir().unwrap();
        let graphs = sample_view_graphs();
        ArtifactRepository::write_view_graphs(tmp.path(), "view_graphs/L0.json", &graphs)
            .unwrap();
        let read =
            ArtifactRepository::read_view_graphs(tmp.path(), "view_graphs/L0.json").unwrap();
        assert_eq!(graphs.len(), read.len());
    }

    #[test]
    fn write_and_read_qa_history_and_ui_state() {
        let tmp = tempfile::tempdir().unwrap();
        let history = QaHistory {
            stage_id: "L0".to_string(),
            entries: vec![],
            version: "1.0.0".to_string(),
        };
        ArtifactRepository::write_qa_history(tmp.path(), "qa_histories/L0.json", &history)
            .unwrap();
        let read =
            ArtifactRepository::read_qa_history(tmp.path(), "qa_histories/L0.json").unwrap();
        assert_eq!(history.stage_id, read.stage_id);

        let ui = PersistedUiState {
            stage_id: "L0".to_string(),
            selected_trace_target: None,
            resolved_traces: vec![],
            current_source_excerpt: None,
            highlighted_evidence_id: None,
            active_view_type: None,
        };
        ArtifactRepository::write_ui_state(tmp.path(), "ui_states/L0.json", &ui)
            .unwrap();
        let read_ui = ArtifactRepository::read_ui_state(tmp.path(), "ui_states/L0.json").unwrap();
        assert_eq!(ui.stage_id, read_ui.stage_id);
    }

    #[test]
    fn absolute_path_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let profile = sample_workspace_profile();
        let result = ArtifactRepository::write_workspace_profile(
            tmp.path(),
            "/etc/passwd",
            &profile,
        );
        assert!(
            matches!(result.unwrap_err(), ArtifactRepositoryError::AbsolutePath { .. }),
            "绝对路径应被拒绝"
        );
    }

    #[test]
    fn dotdot_path_traversal_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let profile = sample_workspace_profile();
        let result = ArtifactRepository::write_workspace_profile(
            tmp.path(),
            "../evil.json",
            &profile,
        );
        assert!(
            matches!(result.unwrap_err(), ArtifactRepositoryError::PathTraversal { .. }),
            ".. 应被拒绝"
        );
    }

    #[test]
    fn dot_prefixed_path_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let profile = sample_workspace_profile();
        let result = ArtifactRepository::write_workspace_profile(
            tmp.path(),
            ".hidden.json",
            &profile,
        );
        assert!(
            matches!(result.unwrap_err(), ArtifactRepositoryError::InvalidPath { .. }),
            "以 . 开头的路径应被拒绝"
        );
    }

    #[test]
    fn symlink_artifact_file_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let real = tmp.path().join("real.json");
        fs::write(&real, "{}").unwrap();
        let link = tmp.path().join("link.json");
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(&real, &link).unwrap();
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_file;
            symlink_file(&real, &link).unwrap();
        }

        let result = ArtifactRepository::read_workspace_profile(tmp.path(), "link.json");
        let err = result.unwrap_err();
        assert!(
            matches!(err, ArtifactRepositoryError::SymlinkNotAllowed { .. }),
            "artifact symlink 应被拒绝，实际得到 {:?}",
            err
        );
    }

    #[test]
    fn symlink_parent_directory_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let real_dir = tmp.path().join("real_dir");
        fs::create_dir(&real_dir).unwrap();
        let link_dir = tmp.path().join("link_dir");
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(&real_dir, &link_dir).unwrap();
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_dir;
            symlink_dir(&real_dir, &link_dir).unwrap();
        }

        // link_dir/evil.json 的父目录是 symlink。
        let profile = sample_workspace_profile();
        let result = ArtifactRepository::write_workspace_profile(
            tmp.path(),
            "link_dir/evil.json",
            &profile,
        );
        assert!(
            matches!(result.unwrap_err(), ArtifactRepositoryError::SymlinkNotAllowed { .. }),
            "父目录为 symlink 应被拒绝"
        );
    }

    #[test]
    fn session_dir_symlink_is_rejected_for_write() {
        let tmp = tempfile::tempdir().unwrap();
        let real_session = tmp.path().join("real_session");
        fs::create_dir(&real_session).unwrap();
        let link_session = tmp.path().join("link_session");
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(&real_session, &link_session).unwrap();
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_dir;
            symlink_dir(&real_session, &link_session).unwrap();
        }

        let profile = sample_workspace_profile();
        let result = ArtifactRepository::write_workspace_profile(
            &link_session,
            "workspace_profile.json",
            &profile,
        );
        assert!(
            matches!(result.unwrap_err(), ArtifactRepositoryError::SymlinkNotAllowed { .. }),
            "session_dir 为 symlink 时写入应被拒绝"
        );
    }

    #[test]
    fn session_dir_symlink_is_rejected_for_read() {
        let tmp = tempfile::tempdir().unwrap();
        let real_session = tmp.path().join("real_session");
        fs::create_dir(&real_session).unwrap();
        let link_session = tmp.path().join("link_session");
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(&real_session, &link_session).unwrap();
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_dir;
            symlink_dir(&real_session, &link_session).unwrap();
        }

        ArtifactRepository::write_workspace_profile(
            &real_session,
            "workspace_profile.json",
            &sample_workspace_profile(),
        )
        .unwrap();

        let result = ArtifactRepository::read_workspace_profile(
            &link_session,
            "workspace_profile.json",
        );
        assert!(
            matches!(result.unwrap_err(), ArtifactRepositoryError::SymlinkNotAllowed { .. }),
            "session_dir 为 symlink 时读取应被拒绝"
        );
    }

    #[test]
    fn atomic_write_produces_complete_target() {
        let tmp = tempfile::tempdir().unwrap();
        let profile = sample_workspace_profile();
        ArtifactRepository::write_workspace_profile(tmp.path(), "workspace_profile.json", &profile)
            .unwrap();

        let target = tmp.path().join("workspace_profile.json");
        assert!(target.exists());
        let content = fs::read_to_string(&target).unwrap();
        assert!(content.contains("demo"));
        // 临时文件不应残留。
        assert!(!tmp.path().join(".workspace_profile.json.tmp").exists());
    }

    #[test]
    fn temp_path_does_not_escape_session_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let profile = sample_workspace_profile();
        let result = ArtifactRepository::write_workspace_profile(
            tmp.path(),
            "subdir/../../outside.json",
            &profile,
        );
        assert!(
            matches!(result.unwrap_err(), ArtifactRepositoryError::PathTraversal { .. }),
            ".. 越界应被拒绝"
        );
    }
}
