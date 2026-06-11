use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::models::enums::{ErrorCode, StageStatus};
use crate::models::error::WorkspaceWarning;
use crate::workspace::scanner::ScannedFile;

/// 标准阶段集合（按期望顺序）
const STANDARD_STAGES: &[&str] = &["L0", "L1", "L2", "L3", "L4", "L5", "L6", "RTL"];

/// 命名异常变体映射：变体名称 -> 映射到的标准阶段（None 表示保留原名）
fn variant_mapping(name: &str) -> Option<&'static str> {
    match name.to_ascii_uppercase().as_str() {
        "RTL" | "RTL_FINAL" | "HARDWARE" | "FPGA" => Some("RTL"),
        "L0" | "LEVEL0" | "STAGE0" => Some("L0"),
        "L1" | "LEVEL1" | "STAGE1" => Some("L1"),
        "L2" | "LEVEL2" | "STAGE2" => Some("L2"),
        "L3" | "LEVEL3" | "STAGE3" => Some("L3"),
        "L4" | "LEVEL4" | "STAGE4" => Some("L4"),
        "L5" | "LEVEL5" | "STAGE5" => Some("L5"),
        "L6" | "LEVEL6" | "STAGE6" => Some("L6"),
        _ => None,
    }
}

/// 判断目录名是否为阶段（标准或变体）
fn is_stage_name(name: &str) -> bool {
    variant_mapping(name).is_some()
}

/// 检测是否命名异常（匹配变体但非标准名）
fn is_naming_anomaly(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    if STANDARD_STAGES.contains(&upper.as_str()) {
        return false;
    }
    variant_mapping(name).is_some()
}

/// 阶段识别结果
#[derive(Debug, Clone)]
pub struct StageDetectionResult {
    pub stages: Vec<StageInfo>,
    pub missing: Vec<String>,
    pub warnings: Vec<WorkspaceWarning>,
    pub validity_reasons: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StageInfo {
    pub stage_id: String,
    pub source_path: String,
    pub status: StageStatus,
    pub file_count: u64,
}

/// 基于扫描结果识别阶段目录。
pub fn detect_stages(root: &Path, scanned: &[ScannedFile]) -> StageDetectionResult {
    let mut result = StageDetectionResult {
        stages: Vec::new(),
        missing: Vec::new(),
        warnings: Vec::new(),
        validity_reasons: Vec::new(),
    };

    let _root_str = root.to_string_lossy();
    let mut found_stages: HashMap<String, StageInfo> = HashMap::new();

    // 扫描根目录下的子目录
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_type = match entry.file_type() {
                Ok(t) => t,
                Err(_) => continue,
            };

            // 跳过 symlink
            if file_type.is_symlink() {
                continue;
            }

            if !file_type.is_dir() {
                continue;
            }

            let dir_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            if !is_stage_name(&dir_name) {
                continue;
            }

            // 可读性检查
            if std::fs::read_dir(&path).is_err() {
                let mapped = variant_mapping(&dir_name).unwrap_or(&dir_name);
                found_stages.insert(
                    mapped.to_string(),
                    StageInfo {
                        stage_id: mapped.to_string(),
                        source_path: path.display().to_string(),
                        status: StageStatus::Unreadable,
                        file_count: 0,
                    },
                );
                continue;
            }

            // 计算阶段内文件数
            let prefix = format!("{}/", dir_name);
            let count = scanned
                .iter()
                .filter(|f| f.rel_path.starts_with(&prefix))
                .count() as u64;

            let mapped = variant_mapping(&dir_name).unwrap_or(&dir_name);
            let is_anomaly = is_naming_anomaly(&dir_name);

            let status = if count == 0 {
                StageStatus::Empty
            } else if is_anomaly {
                StageStatus::NamingAnomaly
            } else {
                StageStatus::Available
            };

            found_stages.insert(
                mapped.to_string(),
                StageInfo {
                    stage_id: mapped.to_string(),
                    source_path: path.display().to_string(),
                    status,
                    file_count: count,
                },
            );
        }
    }

    // 排序：标准阶段在前，命名异常按字典序
    let mut standard: Vec<StageInfo> = Vec::new();
    let mut anomalies: Vec<StageInfo> = Vec::new();

    for id in STANDARD_STAGES {
        if let Some(info) = found_stages.remove(*id) {
            standard.push(info);
        }
    }

    let mut remaining: Vec<StageInfo> = found_stages.into_values().collect();
    remaining.sort_by(|a, b| a.stage_id.cmp(&b.stage_id));
    anomalies.extend(remaining);

    result.stages = standard;
    result.stages.extend(anomalies);

    // 检测缺失阶段
    let found_ids: HashSet<String> = result.stages.iter().map(|s| s.stage_id.clone()).collect();
    for expected in STANDARD_STAGES {
        if !found_ids.contains(*expected) {
            result.missing.push((*expected).to_string());
            result.warnings.push(WorkspaceWarning {
                error_code: ErrorCode::NoStageFound,
                message: format!("预期阶段 {} 未找到", expected),
                source_path: None,
                related_stage_id: Some((*expected).to_string()),
                recoverable: true,
            });
            result.validity_reasons.push(format!("缺失阶段: {}", expected));
        }
    }

    result
}

/// 为给定阶段计算文件数（基于扫描结果）。
pub fn count_stage_files(stage_id: &str, scanned: &[ScannedFile]) -> u64 {
    scanned
        .iter()
        .filter(|f| f.rel_path.starts_with(&format!("{}/", stage_id)))
        .count() as u64
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::models::enums::Language;
    use super::*;

    fn make_file(rel: &str, lang: Language, kind: crate::models::enums::SourceKind) -> ScannedFile {
        ScannedFile {
            path: Path::new(rel).to_path_buf(),
            rel_path: rel.to_string(),
            language: lang,
            source_kind: kind,
            size_bytes: 100,
        }
    }

    #[test]
    fn standard_stages_detected() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir(root.join("L0")).unwrap();
        fs::create_dir(root.join("L1")).unwrap();
        fs::create_dir(root.join("RTL")).unwrap();

        let scanned = vec![
            make_file("L0/top.py", Language::Python, crate::models::enums::SourceKind::PythonStage),
            make_file("RTL/top.v", Language::Verilog, crate::models::enums::SourceKind::Rtl),
        ];

        let result = detect_stages(root, &scanned);
        assert_eq!(result.stages.len(), 3);
        assert_eq!(result.stages[0].stage_id, "L0");
        assert_eq!(result.stages[1].stage_id, "L1");
        assert_eq!(result.stages[2].stage_id, "RTL");
        assert_eq!(result.stages[0].status, StageStatus::Available);
        assert_eq!(result.missing, vec!["L2", "L3", "L4", "L5", "L6"]);
    }

    #[test]
    fn naming_anomaly_detected() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir(root.join("rtl_final")).unwrap();

        let scanned = vec![make_file(
            "rtl_final/top.v",
            Language::Verilog,
            crate::models::enums::SourceKind::Rtl,
        )];

        let result = detect_stages(root, &scanned);
        assert_eq!(result.stages.len(), 1);
        assert_eq!(result.stages[0].stage_id, "RTL");
        assert_eq!(result.stages[0].status, StageStatus::NamingAnomaly);
    }

    #[test]
    fn empty_stage_detected() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir(root.join("L0")).unwrap();

        let scanned: Vec<ScannedFile> = vec![];

        let result = detect_stages(root, &scanned);
        assert_eq!(result.stages.len(), 1);
        assert_eq!(result.stages[0].status, StageStatus::Empty);
    }

    #[test]
    fn missing_stages_not_inserted() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir(root.join("L0")).unwrap();

        let scanned = vec![make_file(
            "L0/top.py",
            Language::Python,
            crate::models::enums::SourceKind::PythonStage,
        )];

        let result = detect_stages(root, &scanned);
        assert_eq!(result.stages.len(), 1);
        assert!(!result.missing.is_empty());
        assert!(result.missing.contains(&"L1".to_string()));
        assert!(result.missing.contains(&"RTL".to_string()));
        assert!(
            result.warnings.iter().any(|w| w.message.contains("L1")),
            "应有 L1 缺失 warning"
        );
    }

    #[test]
    fn stage_status_missing_not_in_stages() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir(root.join("L0")).unwrap();

        let scanned = vec![make_file(
            "L0/top.py",
            Language::Python,
            crate::models::enums::SourceKind::PythonStage,
        )];

        let result = detect_stages(root, &scanned);
        for s in &result.stages {
            assert_ne!(s.status, StageStatus::Missing, "stages[] 不应包含 Missing");
        }
    }

    #[test]
    fn rtl_lowercase_mapped() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir(root.join("rtl")).unwrap();

        let scanned = vec![make_file(
            "rtl/top.v",
            Language::Verilog,
            crate::models::enums::SourceKind::Rtl,
        )];

        let result = detect_stages(root, &scanned);
        assert_eq!(result.stages.len(), 1);
        assert_eq!(result.stages[0].stage_id, "RTL");
        assert_eq!(result.stages[0].status, StageStatus::Available);
    }
}
