use tauri::Manager;

use crate::models::enums::ErrorCode;
use crate::models::error::{CommandError, CommandResult};
use crate::persistence::session_store::{SessionStore, SessionStoreError};

/// Tauri command：删除指定 session 及其所有 artifact。
///
/// 仅删除 app-owned storage 下的 session 目录，不触碰目标项目。
#[tauri::command]
pub fn delete_session(
    session_id: String,
    app_handle: tauri::AppHandle,
) -> CommandResult<()> {
    let store = match build_session_store(&app_handle) {
        Ok(s) => s,
        Err(e) => return e,
    };
    execute_delete_session(store, &session_id)
}

fn execute_delete_session(store: SessionStore, session_id: &str) -> CommandResult<()> {
    match store.delete_session(session_id) {
        Ok(()) => CommandResult {
            success: true,
            data: Some(()),
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
        Err(e) => CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::SessionDeleteFailed,
                message: format!("删除 session 失败: {}", e),
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
) -> Result<SessionStore, CommandResult<()>> {
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
    fn delete_session_not_found_returns_command_error() {
        let store = make_store();
        let result = execute_delete_session(store, "sess-missing");

        assert!(!result.success);
        let error = result.error.expect("应有错误");
        assert_eq!(error.error_code, ErrorCode::SessionNotFound);
        assert_eq!(error.message, "session 不存在: sess-missing");
    }
}
