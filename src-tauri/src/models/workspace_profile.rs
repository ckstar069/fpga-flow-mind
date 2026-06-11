use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::enums::{ErrorCode, StageStatus, WorkspaceValidity};
use crate::models::error::WorkspaceWarning;

/// workspace 扫描后的完整概览
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceProfile {
    pub workspace_name: String,
    pub root_path: String,
    pub stages: Vec<StageSummary>,
    pub file_type_stats: HashMap<String, u64>,
    pub external_refs: Vec<String>,
    pub validity: WorkspaceValidity,
    pub validity_reasons: Vec<String>,
    pub warnings: Vec<WorkspaceWarning>,
    pub error_codes: Vec<ErrorCode>,
    pub scan_timestamp: String,
    pub version: String,
}

/// 单个阶段的摘要信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageSummary {
    pub stage_id: String,
    pub source_path: String,
    pub file_count: u64,
    pub status: StageStatus,
}
