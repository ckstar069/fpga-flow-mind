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

/// 错误码枚举
///
/// Phase 1 + Phase 2 错误码。Phase 2 新增标记在注释中。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    // Phase 1
    PathNotFound,
    NotDirectory,
    PermissionDenied,
    NoStageFound,
    StageEmpty,
    StageUnreadable,
    FileUnreadable,
    FileTooLarge,
    ScanTimeout,
    // Phase 2 新增
    EvidenceCollectionFailed,
    SourceExcerptTruncated,
    BinaryFileSkipped,
    NonUtf8FileSkipped,
    // Phase 3 新增
    UnderstandingGenerationFailed,
    // Phase 5 新增（定义见 docs/design/phase-5-trace-and-qa-design.md §8）
    TraceTargetNotFound,
    SourcePathNotAllowed,
    SourceFileUnreadable,
    LineRangeInvalid,
    QaGenerationFailed,
    QaValidationFailed,
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
            ErrorCode::EvidenceCollectionFailed,
            ErrorCode::SourceExcerptTruncated,
            ErrorCode::BinaryFileSkipped,
            ErrorCode::NonUtf8FileSkipped,
            ErrorCode::UnderstandingGenerationFailed,
            ErrorCode::TraceTargetNotFound,
            ErrorCode::SourcePathNotAllowed,
            ErrorCode::SourceFileUnreadable,
            ErrorCode::LineRangeInvalid,
            ErrorCode::QaGenerationFailed,
            ErrorCode::QaValidationFailed,
        ] {
            let json = serde_json::to_string(&code).unwrap();
            let back: ErrorCode = serde_json::from_str(&json).unwrap();
            assert_eq!(code, back, "roundtrip failed for {}", json);
        }
    }

    #[test]
    fn phase2_error_code_snake_case() {
        assert_eq!(
            serde_json::to_string(&ErrorCode::EvidenceCollectionFailed).unwrap(),
            "\"evidence_collection_failed\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorCode::SourceExcerptTruncated).unwrap(),
            "\"source_excerpt_truncated\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorCode::BinaryFileSkipped).unwrap(),
            "\"binary_file_skipped\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorCode::NonUtf8FileSkipped).unwrap(),
            "\"non_utf8_file_skipped\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorCode::SourcePathNotAllowed).unwrap(),
            "\"source_path_not_allowed\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorCode::LineRangeInvalid).unwrap(),
            "\"line_range_invalid\""
        );
    }
}
