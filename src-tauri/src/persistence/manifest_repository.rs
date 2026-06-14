use std::path::{Path, PathBuf};

use crate::persistence::artifact_repository::{ArtifactRepository, ArtifactRepositoryError};
use crate::persistence::models::SessionManifest;
use crate::persistence::storage_version::{StorageVersionService, VersionCompatibility};

const MANIFEST_FILE: &str = "manifest.json";

/// SessionManifestRepository 操作错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestRepositoryError {
    /// session_id 格式非法。
    InvalidSessionId { session_id: String },
    /// 路径超出 app-owned storage 范围。
    PathNotAllowed { path: String },
    /// IO 错误。
    IoError { message: String },
    /// 序列化失败。
    SerializationError { message: String },
    /// 反序列化失败（manifest 损坏）。
    DeserializationError { message: String },
    /// storage_version 不兼容。
    StorageVersionIncompatible { version: String },
}

impl std::fmt::Display for ManifestRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestRepositoryError::InvalidSessionId { session_id } => {
                write!(f, "非法 session_id: {}", session_id)
            }
            ManifestRepositoryError::PathNotAllowed { path } => {
                write!(f, "路径不在允许范围内: {}", path)
            }
            ManifestRepositoryError::IoError { message } => write!(f, "IO 错误: {}", message),
            ManifestRepositoryError::SerializationError { message } => {
                write!(f, "序列化失败: {}", message)
            }
            ManifestRepositoryError::DeserializationError { message } => {
                write!(f, "manifest 损坏或无法解析: {}", message)
            }
            ManifestRepositoryError::StorageVersionIncompatible { version } => {
                write!(f, "不兼容的 storage_version: {}", version)
            }
        }
    }
}

impl std::error::Error for ManifestRepositoryError {}

/// Session manifest 读写仓库。
///
/// 负责：
/// - session_id 安全校验（仅允许 `[a-zA-Z0-9_-]+`）
/// - `manifest.json` 的原子写入与读取
/// - 读取时校验 `storage_version` 兼容性
/// - 保证 session 目录位于 `base_dir` 之下
pub struct SessionManifestRepository;

impl SessionManifestRepository {
    /// 解析并校验 session 目录。
    pub fn session_dir(
        base_dir: &Path,
        session_id: &str,
    ) -> Result<PathBuf, ManifestRepositoryError> {
        Self::validate_session_id(session_id)?;
        let canonical_base = base_dir.canonicalize().map_err(|e| {
            ManifestRepositoryError::IoError {
                message: format!("无法规范化基础目录 {}: {}", base_dir.display(), e),
            }
        })?;
        let session_dir = canonical_base.join(session_id);

        // session 目录本身不能是 symlink（含断链 symlink）。
        match std::fs::symlink_metadata(&session_dir) {
            Ok(m) if m.file_type().is_symlink() => {
                return Err(ManifestRepositoryError::PathNotAllowed {
                    path: session_dir.display().to_string(),
                });
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(ManifestRepositoryError::IoError {
                    message: format!(
                        "无法读取 session 目录元数据 {}: {}",
                        session_dir.display(),
                        e
                    ),
                });
            }
        }

        let canonical_session = session_dir.canonicalize().unwrap_or(session_dir.clone());
        if !canonical_session.starts_with(&canonical_base) {
            return Err(ManifestRepositoryError::PathNotAllowed {
                path: session_dir.display().to_string(),
            });
        }
        Ok(session_dir)
    }

    /// 写入 manifest.json。
    pub fn write(
        base_dir: &Path,
        session_id: &str,
        manifest: &SessionManifest,
    ) -> Result<(), ManifestRepositoryError> {
        let session_dir = Self::session_dir(base_dir, session_id)?;
        std::fs::create_dir_all(&session_dir).map_err(|e| ManifestRepositoryError::IoError {
            message: format!("无法创建 session 目录 {}: {}", session_dir.display(), e),
        })?;
        ArtifactRepository::write_json(&session_dir, MANIFEST_FILE, manifest)
            .map_err(Self::map_artifact_error)?;
        Ok(())
    }

    /// 读取 manifest.json 并校验 storage_version。
    pub fn read(
        base_dir: &Path,
        session_id: &str,
    ) -> Result<SessionManifest, ManifestRepositoryError> {
        let session_dir = Self::session_dir(base_dir, session_id)?;
        let manifest: SessionManifest =
            ArtifactRepository::read_json(&session_dir, MANIFEST_FILE)
                .map_err(Self::map_artifact_error)?;

        if StorageVersionService::check_compatibility(&manifest.storage_version)
            == VersionCompatibility::Incompatible
        {
            return Err(ManifestRepositoryError::StorageVersionIncompatible {
                version: format!(
                    "{}.{}.{}",
                    manifest.storage_version.major,
                    manifest.storage_version.minor,
                    manifest.storage_version.patch
                ),
            });
        }

        Ok(manifest)
    }

    /// 判断指定 session 的 manifest.json 是否存在。
    pub fn exists(
        base_dir: &Path,
        session_id: &str,
    ) -> Result<bool, ManifestRepositoryError> {
        let session_dir = Self::session_dir(base_dir, session_id)?;
        Ok(session_dir.join(MANIFEST_FILE).exists())
    }

    fn validate_session_id(session_id: &str) -> Result<(), ManifestRepositoryError> {
        if session_id.is_empty() {
            return Err(ManifestRepositoryError::InvalidSessionId {
                session_id: session_id.to_string(),
            });
        }
        for ch in session_id.chars() {
            if !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-' {
                return Err(ManifestRepositoryError::InvalidSessionId {
                    session_id: session_id.to_string(),
                });
            }
        }
        Ok(())
    }

    fn map_artifact_error(err: ArtifactRepositoryError) -> ManifestRepositoryError {
        match err {
            ArtifactRepositoryError::PathTraversal { path }
            | ArtifactRepositoryError::AbsolutePath { path }
            | ArtifactRepositoryError::InvalidPath { path }
            | ArtifactRepositoryError::SymlinkNotAllowed { path } => {
                ManifestRepositoryError::PathNotAllowed { path }
            }
            ArtifactRepositoryError::IoError { message } => {
                ManifestRepositoryError::IoError { message }
            }
            ArtifactRepositoryError::SerializationError { message } => {
                ManifestRepositoryError::SerializationError { message }
            }
            ArtifactRepositoryError::DeserializationError { message } => {
                ManifestRepositoryError::DeserializationError { message }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::persistence::models::{
        ArtifactIndex, GlobalUiState, PersistedStageSummary, PersistedWorkspace, StorageVersion,
    };

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
    fn write_and_read_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        let manifest = sample_manifest();
        SessionManifestRepository::write(tmp.path(), "sess-001", &manifest).unwrap();
        let read = SessionManifestRepository::read(tmp.path(), "sess-001").unwrap();
        assert_eq!(read.session_id, manifest.session_id);
        assert_eq!(read.storage_version, manifest.storage_version);
        assert_eq!(read.stages.len(), manifest.stages.len());
    }

    #[test]
    fn exists_returns_correct_value() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(!SessionManifestRepository::exists(tmp.path(), "sess-001").unwrap());
        SessionManifestRepository::write(tmp.path(), "sess-001", &sample_manifest()).unwrap();
        assert!(SessionManifestRepository::exists(tmp.path(), "sess-001").unwrap());
    }

    #[test]
    fn invalid_session_id_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let result = SessionManifestRepository::write(tmp.path(), "../evil", &sample_manifest());
        assert!(
            matches!(result.unwrap_err(), ManifestRepositoryError::InvalidSessionId { .. }),
            "非法 session_id 应被拒绝"
        );
    }

    #[test]
    fn empty_session_id_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let result = SessionManifestRepository::write(tmp.path(), "", &sample_manifest());
        assert!(
            matches!(result.unwrap_err(), ManifestRepositoryError::InvalidSessionId { .. }),
            "空 session_id 应被拒绝"
        );
    }

    #[test]
    fn slash_session_id_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let result =
            SessionManifestRepository::write(tmp.path(), "abc/def", &sample_manifest());
        assert!(
            matches!(result.unwrap_err(), ManifestRepositoryError::InvalidSessionId { .. }),
            "含路径分隔符的 session_id 应被拒绝"
        );
    }

    #[test]
    fn leading_dot_session_id_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let result =
            SessionManifestRepository::write(tmp.path(), ".hidden", &sample_manifest());
        assert!(
            matches!(result.unwrap_err(), ManifestRepositoryError::InvalidSessionId { .. }),
            "以 . 开头的 session_id 应被拒绝"
        );
    }

    #[test]
    fn session_dir_symlink_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().join("sessions");
        fs::create_dir(&base).unwrap();

        let real_session = base.join("real_session");
        fs::create_dir(&real_session).unwrap();
        let link_session = base.join("sess-001");
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

        let write_result = SessionManifestRepository::write(&base, "sess-001", &sample_manifest(),
        );
        assert!(
            matches!(write_result.unwrap_err(), ManifestRepositoryError::PathNotAllowed { .. }),
            "session 目录为 symlink 时写入应被拒绝"
        );

        let read_result = SessionManifestRepository::read(&base, "sess-001",
        );
        assert!(
            matches!(read_result.unwrap_err(), ManifestRepositoryError::PathNotAllowed { .. }),
            "session 目录为 symlink 时读取应被拒绝"
        );
    }

    #[test]
    fn manifest_file_symlink_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().join("sessions");
        fs::create_dir(&base).unwrap();

        let session_dir = base.join("sess-001");
        fs::create_dir(&session_dir).unwrap();

        let real_manifest = session_dir.join("real_manifest.json");
        fs::write(
            &real_manifest,
            serde_json::to_string_pretty(&sample_manifest()).unwrap(),
        )
        .unwrap();
        let manifest_link = session_dir.join("manifest.json");
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(&real_manifest, &manifest_link).unwrap();
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_file;
            symlink_file(&real_manifest, &manifest_link).unwrap();
        }

        let result = SessionManifestRepository::read(&base, "sess-001",
        );
        assert!(
            matches!(result.unwrap_err(), ManifestRepositoryError::PathNotAllowed { .. }),
            "manifest.json 为 symlink 时应被拒绝"
        );
    }

    #[test]
    fn corrupted_manifest_returns_error() {
        let tmp = tempfile::tempdir().unwrap();
        let session_dir = tmp.path().join("sess-001");
        fs::create_dir(&session_dir).unwrap();
        fs::write(session_dir.join("manifest.json"), "not valid json").unwrap();
        let result = SessionManifestRepository::read(tmp.path(), "sess-001");
        assert!(
            matches!(result.unwrap_err(), ManifestRepositoryError::DeserializationError { .. }),
            "损坏的 manifest 应返回反序列化错误"
        );
    }

    #[test]
    fn incompatible_storage_version_returns_error() {
        let tmp = tempfile::tempdir().unwrap();
        let mut manifest = sample_manifest();
        manifest.storage_version = StorageVersion {
            major: 9,
            minor: 0,
            patch: 0,
        };
        SessionManifestRepository::write(tmp.path(), "sess-001", &manifest).unwrap();
        let result = SessionManifestRepository::read(tmp.path(), "sess-001");
        assert!(
            matches!(
                result.unwrap_err(),
                ManifestRepositoryError::StorageVersionIncompatible { .. }
            ),
            "不兼容版本应被拒绝"
        );
    }
}
