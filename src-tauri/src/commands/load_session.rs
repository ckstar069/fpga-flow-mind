use tauri::Manager;

use crate::models::enums::ErrorCode;
use crate::models::error::{CommandError, CommandResult};
use crate::persistence::models::LoadSessionResult;
use crate::persistence::session_store::{SessionStore, SessionStoreError};

/// Tauri command：加载 session。
///
/// 目标项目变更/缺失/不安全均返回 `success=true`，由 `LoadSessionResult.status` 表达。
/// 真正的阻塞错误（不存在、损坏、版本不兼容）返回 `success=false`。
#[tauri::command]
pub fn load_session(
    session_id: String,
    app_handle: tauri::AppHandle,
) -> CommandResult<LoadSessionResult> {
    let store = match build_session_store(&app_handle) {
        Ok(s) => s,
        Err(e) => return e,
    };
    execute_load_session(store, &session_id)
}

fn execute_load_session(
    store: SessionStore,
    session_id: &str,
) -> CommandResult<LoadSessionResult> {
    match store.load_session(session_id) {
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
        Err(SessionStoreError::SessionNotFound { session_id }) => CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::SessionNotFound,
                message: format!("session 不存在: {}", session_id),
                recoverable: false,
                details: None,
                source_path: None,
            }),
            warnings: vec![],
        },
        Err(SessionStoreError::ManifestCorrupted { message }) => CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::LoadFailed,
                message: format!("manifest 损坏: {}", message),
                recoverable: false,
                details: None,
                source_path: None,
            }),
            warnings: vec![],
        },
        Err(SessionStoreError::StorageVersionIncompatible { version }) => CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::StorageVersionIncompatible,
                message: format!("不兼容的 storage_version: {}", version),
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
                error_code: ErrorCode::LoadFailed,
                message: format!("加载 session 失败: {}", e),
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
) -> Result<SessionStore, CommandResult<LoadSessionResult>> {
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
    use super::*;

    fn make_store() -> SessionStore {
        let tmp = tempfile::tempdir().unwrap();
        SessionStore::with_version(tmp.path().to_path_buf(), "0.1.0".to_string())
    }

    #[test]
    fn load_session_not_found_returns_command_error() {
        let store = make_store();
        let result = execute_load_session(store, "sess-missing");

        assert!(!result.success);
        let error = result.error.expect("应有错误");
        assert_eq!(error.error_code, ErrorCode::SessionNotFound);
        assert_eq!(error.message, "session 不存在: sess-missing");
    }
}
