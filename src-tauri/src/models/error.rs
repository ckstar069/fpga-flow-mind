use serde::{Deserialize, Serialize};

use crate::models::enums::ErrorCode;

/// 阻塞性错误，导致当前 command 返回 success=false
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandError {
    pub error_code: ErrorCode,
    pub message: String,
    pub recoverable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
}

/// 扫描过程中的非致命警告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceWarning {
    pub error_code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_stage_id: Option<String>,
    pub recoverable: bool,
}

/// 统一的 command 返回结构
///
/// 前端总是先检查 `success`，再读取 `data` 或 `error`。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<CommandError>,
    pub warnings: Vec<WorkspaceWarning>,
}
