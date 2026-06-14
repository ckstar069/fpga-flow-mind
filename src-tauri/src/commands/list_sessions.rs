use tauri::Manager;

use crate::models::enums::ErrorCode;
use crate::models::error::{CommandError, CommandResult};
use crate::persistence::models::SessionSummary;
use crate::persistence::session_store::SessionStore;

/// Tauri command：列出最近保存的 session。
///
/// `limit` 为可选参数，默认不限制，由调用方控制最大数量。
#[tauri::command]
pub fn list_sessions(
    limit: Option<u32>,
    app_handle: tauri::AppHandle,
) -> CommandResult<Vec<SessionSummary>> {
    let store = match build_session_store(&app_handle) {
        Ok(s) => s,
        Err(e) => return e,
    };

    let limit = limit.map(|n| n as usize);
    match store.list_sessions(limit) {
        Ok(summaries) => CommandResult {
            success: true,
            data: Some(summaries),
            error: None,
            warnings: vec![],
        },
        Err(e) => CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::LoadFailed,
                message: format!("列出 session 失败: {}", e),
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
) -> Result<SessionStore, CommandResult<Vec<SessionSummary>>> {
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
