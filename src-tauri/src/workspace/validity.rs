use crate::models::enums::{ErrorCode, Language, WorkspaceValidity};
use crate::workspace::scanner::ScannedFile;
use crate::workspace::stage_detector::StageInfo;

/// 基于扫描结果和阶段识别结果计算 workspace validity。
///
/// 判定规则（与文档契约一致）：
/// - 至少 1 个标准/变体阶段 且 存在 Python 或 Verilog/SystemVerilog → likely_valid
/// - 有阶段但无可分析代码 → uncertain
/// - 无可识别阶段 但 存在可分析代码 → uncertain
/// - 无阶段 且 无可分析代码 → unlikely
/// - 仅文档（无代码）→ uncertain（有阶段）/ unlikely（无阶段）
pub fn calculate_validity(stages: &[StageInfo], scanned: &[ScannedFile]) -> (WorkspaceValidity, Vec<String>) {
    let mut reasons = Vec::new();

    let has_stages = !stages.is_empty();
    let has_code = scanned.iter().any(|f| {
        matches!(
            f.language,
            Language::Python | Language::Verilog | Language::SystemVerilog
        )
    });

    let has_any_files = !scanned.is_empty();
    let has_only_docs = has_any_files
        && !has_code
        && scanned.iter().all(|f| {
            matches!(
                f.language,
                Language::Markdown | Language::Text | Language::Json | Language::Yaml | Language::Toml
            )
        });

    if has_stages && has_code {
        return (WorkspaceValidity::LikelyValid, reasons);
    }

    if has_stages && !has_code {
        reasons.push("存在阶段目录但无核心代码文件".to_string());
        return (WorkspaceValidity::Uncertain, reasons);
    }

    if !has_stages && has_code {
        reasons.push("存在代码文件但未识别到标准阶段".to_string());
        return (WorkspaceValidity::Uncertain, reasons);
    }

    if !has_stages && has_only_docs {
        reasons.push("仅发现文档文件，无阶段和核心代码".to_string());
        return (WorkspaceValidity::Unlikely, reasons);
    }

    // 空目录
    reasons.push("未识别到阶段且无可分析文件".to_string());
    (WorkspaceValidity::Unlikely, reasons)
}

/// 收集 error_codes（workspace 级）。
pub fn collect_error_codes(stages: &[StageInfo]) -> Vec<ErrorCode> {
    let mut codes = Vec::new();

    if stages.is_empty() {
        codes.push(ErrorCode::NoStageFound);
    }

    codes
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::models::enums::{SourceKind, StageStatus};
    use crate::workspace::scanner::ScannedFile;

    use super::*;

    fn make_file(lang: Language) -> ScannedFile {
        ScannedFile {
            path: Path::new("f.py").to_path_buf(),
            rel_path: "f.py".to_string(),
            language: lang,
            source_kind: SourceKind::PythonStage,
            size_bytes: 100,
        }
    }

    #[test]
    fn stages_and_code_is_likely_valid() {
        let stages = vec![StageInfo {
            stage_id: "L0".to_string(),
            source_path: "/p/L0".to_string(),
            status: StageStatus::Available,
            file_count: 1,
        }];
        let scanned = vec![make_file(Language::Python)];
        let (v, _) = calculate_validity(&stages, &scanned);
        assert_eq!(v, WorkspaceValidity::LikelyValid);
    }

    #[test]
    fn no_stages_but_code_is_uncertain() {
        let stages: Vec<StageInfo> = vec![];
        let scanned = vec![make_file(Language::Python)];
        let (v, reasons) = calculate_validity(&stages, &scanned);
        assert_eq!(v, WorkspaceValidity::Uncertain);
        assert!(!reasons.is_empty());
    }

    #[test]
    fn empty_dir_is_unlikely() {
        let stages: Vec<StageInfo> = vec![];
        let scanned: Vec<ScannedFile> = vec![];
        let (v, _) = calculate_validity(&stages, &scanned);
        assert_eq!(v, WorkspaceValidity::Unlikely);
    }

    #[test]
    fn stages_no_code_is_uncertain() {
        let stages = vec![StageInfo {
            stage_id: "L0".to_string(),
            source_path: "/p/L0".to_string(),
            status: StageStatus::Empty,
            file_count: 0,
        }];
        let scanned: Vec<ScannedFile> = vec![];
        let (v, _) = calculate_validity(&stages, &scanned);
        assert_eq!(v, WorkspaceValidity::Uncertain);
    }

    #[test]
    fn only_rtl_is_likely_valid() {
        let stages = vec![StageInfo {
            stage_id: "RTL".to_string(),
            source_path: "/p/RTL".to_string(),
            status: StageStatus::Available,
            file_count: 1,
        }];
        let scanned = vec![ScannedFile {
            path: Path::new("RTL/top.sv").to_path_buf(),
            rel_path: "RTL/top.sv".to_string(),
            language: Language::SystemVerilog,
            source_kind: SourceKind::Rtl,
            size_bytes: 100,
        }];
        let (v, _) = calculate_validity(&stages, &scanned);
        assert_eq!(v, WorkspaceValidity::LikelyValid);
    }
}
