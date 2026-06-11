use serde::{Deserialize, Serialize};

use crate::models::enums::{ErrorCode, Language, SourceKind};

/// 选中单个阶段后的上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageContext {
    pub stage_id: String,
    pub source_path: String,
    pub files: Vec<StageFile>,
    pub external_deps: Vec<String>,
    pub upstream_refs: Vec<UpstreamRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<ErrorCode>,
}

/// 阶段内的单个文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageFile {
    pub source_path: String,
    pub language: Language,
    pub source_kind: SourceKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
}

/// 推断出的上游阶段引用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamRef {
    pub stage_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface_file_path: Option<String>,
    pub inferred: bool,
}
