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
    #[serde(rename = "systemverilog")]
    SystemVerilog,
    Markdown,
    Text,
    Json,
    Yaml,
    Toml,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn systemverilog_serializes_correctly() {
        assert_eq!(
            serde_json::to_string(&Language::SystemVerilog).unwrap(),
            "\"systemverilog\""
        );
    }

    #[test]
    fn systemverilog_deserializes_correctly() {
        let lang: Language = serde_json::from_str("\"systemverilog\"").unwrap();
        assert_eq!(lang, Language::SystemVerilog);
    }

    #[test]
    fn naming_anomaly_snake_case_is_stable() {
        assert_eq!(
            serde_json::to_string(&StageStatus::NamingAnomaly).unwrap(),
            "\"naming_anomaly\""
        );
    }

    #[test]
    fn error_code_roundtrip() {
        for code in [
            ErrorCode::PathNotFound,
            ErrorCode::NotDirectory,
            ErrorCode::PermissionDenied,
            ErrorCode::NoStageFound,
            ErrorCode::StageEmpty,
            ErrorCode::StageUnreadable,
            ErrorCode::FileUnreadable,
            ErrorCode::FileTooLarge,
            ErrorCode::ScanTimeout,
        ] {
            let json = serde_json::to_string(&code).unwrap();
            let back: ErrorCode = serde_json::from_str(&json).unwrap();
            assert_eq!(code, back, "roundtrip failed for {}", json);
        }
    }
}
