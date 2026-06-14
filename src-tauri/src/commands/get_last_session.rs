use tauri::Manager;

use crate::models::enums::ErrorCode;
use crate::models::error::{CommandError, CommandResult};
use crate::persistence::models::SessionSummary;
use crate::persistence::session_store::SessionStore;

/// Tauri command：返回最近更新的 session 摘要。
///
/// 没有任何 session 时返回 `success=true, data=None`。
#[tauri::command]
pub fn get_last_session(app_handle: tauri::AppHandle) -> CommandResult<Option<SessionSummary>> {
    let store = match build_session_store(&app_handle) {
        Ok(s) => s,
        Err(e) => return e,
    };
    execute_get_last_session(store)
}

fn execute_get_last_session(store: SessionStore) -> CommandResult<Option<SessionSummary>> {
    match store.list_sessions(Some(1)) {
        Ok(mut summaries) => CommandResult {
            success: true,
            data: Some(summaries.pop()),
            error: None,
            warnings: vec![],
        },
        Err(e) => CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::LoadFailed,
                message: format!("获取最近 session 失败: {}", e),
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
) -> Result<SessionStore, CommandResult<Option<SessionSummary>>> {
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
    fn get_last_session_empty_returns_success_with_none() {
        let store = make_store();
        let result = execute_get_last_session(store);

        assert!(result.success);
        assert!(result.data.is_none() || result.data.as_ref().unwrap().is_none());
    }
}
