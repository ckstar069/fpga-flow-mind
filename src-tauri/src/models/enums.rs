use serde::{Deserialize, Serialize};

/// workspace 整体有效性评估
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceValidity {
    LikelyValid,
    Uncertain,
    Unlikely,
}

/// 单个阶段的识别状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageStatus {
    Available,
    Empty,
    Missing,
    NamingAnomaly,
    Unreadable,
}

/// Phase 1 使用的错误码子集
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    PathNotFound,
    NotDirectory,
    PermissionDenied,
    NoStageFound,
    StageEmpty,
    StageUnreadable,
    FileUnreadable,
    FileTooLarge,
    ScanTimeout,
}

/// 源码文件的语义分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    PythonStage,
    Rtl,
    Test,
    Doc,
    Config,
    ExternalModule,
}

/// 编程语言识别结果
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Python,
    Verilog,
    SystemVerilog,
    Markdown,
    Text,
    Json,
    Yaml,
    Toml,
    Unknown,
}
