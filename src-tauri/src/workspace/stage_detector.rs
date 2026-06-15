use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::models::enums::{ErrorCode, StageStatus};
use crate::models::error::WorkspaceWarning;
use crate::workspace::scanner::ScannedFile;

/// ai_project_template 布局映射：将深层目录名映射到标准阶段
fn ai_project_template_mapping(name: &str) -> Option<&'static str> {
    match name.to_ascii_lowercase().as_str() {
        "l0_external" => Some("L0"),
        "l1_prototype" => Some("L1"),
        "l2_structured" => Some("L2"),
        "l3_pipeline" => Some("L3"),
        "l4_cycle_acc" => Some("L4"),
        "l5_fixedpoint" => Some("L5"),
        "l6_resource_opt" => Some("L6"),
        _ => None,
    }
}

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
///
/// 支持两种布局：
/// 1. 传统顶层布局：根目录下直接存在 L0/L1/.../RTL 目录
/// 2. ai_project_template 布局：src/python_model/L0_external 等深层目录
///
/// 规则：
/// - 如果顶层与深层同时存在同一阶段，优先使用顶层，并生成重复候选 warning
/// - 不将 src/python_model 本身当作单个阶段
/// - 深层识别到的阶段 status 为 Available（有文件时）或 Empty（无文件时）
pub fn detect_stages(root: &Path, scanned: &[ScannedFile]) -> StageDetectionResult {
    let mut result = StageDetectionResult {
        stages: Vec::new(),
        missing: Vec::new(),
        warnings: Vec::new(),
        validity_reasons: Vec::new(),
    };

    let _root_str = root.to_string_lossy().to_string();
    let mut found_stages: HashMap<String, StageInfo> = HashMap::new();
    let mut top_level_stages: HashSet<String> = HashSet::new();

    // === Pass 1: 扫描根目录下的子目录（传统顶层布局）===
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
                top_level_stages.insert(mapped.to_string());
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
            top_level_stages.insert(mapped.to_string());
        }
    }

    // === Pass 2: 扫描 ai_project_template 深层布局 ===
    // 检查 src/python_model/L0_external 等路径
    let src_path = root.join("src");
    if let Ok(src_entries) = std::fs::read_dir(&src_path) {
        for src_entry in src_entries.flatten() {
            let src_sub_path = src_entry.path();
            let file_type = match src_entry.file_type() {
                Ok(t) => t,
                Err(_) => continue,
            };

            if !file_type.is_dir() || file_type.is_symlink() {
                continue;
            }

            let src_sub_name = src_sub_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            // 处理 src/python_model/ 下的 L*_xxx 目录
            if src_sub_name.eq_ignore_ascii_case("python_model") {
                if let Ok(py_entries) = std::fs::read_dir(&src_sub_path) {
                    for py_entry in py_entries.flatten() {
                        let py_path = py_entry.path();
                        let py_type = match py_entry.file_type() {
                            Ok(t) => t,
                            Err(_) => continue,
                        };

                        if !py_type.is_dir() || py_type.is_symlink() {
                            continue;
                        }

                        let py_dir_name = py_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();

                        if let Some(stage_id) = ai_project_template_mapping(&py_dir_name) {
                            // 如果顶层已存在该阶段，生成 warning 并跳过
                            if top_level_stages.contains(stage_id) {
                                result.warnings.push(WorkspaceWarning {
                                    error_code: ErrorCode::NoStageFound,
                                    message: format!(
                                        "阶段 {} 同时存在顶层目录与 ai_project_template 深层目录 ({})，优先使用顶层",
                                        stage_id, py_path.display()
                                    ),
                                    source_path: Some(py_path.display().to_string()),
                                    related_stage_id: Some(stage_id.to_string()),
                                    recoverable: true,
                                });
                                continue;
                            }

                            // 计算文件数
                            let prefix = format!("src/python_model/{}/", py_dir_name);
                            let count = scanned
                                .iter()
                                .filter(|f| f.rel_path.starts_with(&prefix))
                                .count() as u64;

                            let status = if count == 0 {
                                StageStatus::Empty
                            } else {
                                StageStatus::Available
                            };

                            found_stages.insert(
                                stage_id.to_string(),
                                StageInfo {
                                    stage_id: stage_id.to_string(),
                                    source_path: py_path.display().to_string(),
                                    status,
                                    file_count: count,
                                },
                            );
                        }
                    }
                }
            }

            // 处理 src/verilog_model/rtl 目录
            if src_sub_name.eq_ignore_ascii_case("verilog_model") {
                let rtl_path = src_sub_path.join("rtl");
                if rtl_path.is_dir() {
                    // 如果顶层已存在 RTL，生成 warning 并跳过
                    if top_level_stages.contains("RTL") {
                        result.warnings.push(WorkspaceWarning {
                            error_code: ErrorCode::NoStageFound,
                            message: format!(
                                "阶段 RTL 同时存在顶层目录与 ai_project_template 深层目录 ({})，优先使用顶层",
                                rtl_path.display()
                            ),
                            source_path: Some(rtl_path.display().to_string()),
                            related_stage_id: Some("RTL".to_string()),
                            recoverable: true,
                        });
                    } else {
                        let prefix = "src/verilog_model/rtl/";
                        let count = scanned
                            .iter()
                            .filter(|f| f.rel_path.starts_with(prefix))
                            .count() as u64;

                        let status = if count == 0 {
                            StageStatus::Empty
                        } else {
                            StageStatus::Available
                        };

                        found_stages.insert(
                            "RTL".to_string(),
                            StageInfo {
                                stage_id: "RTL".to_string(),
                                source_path: rtl_path.display().to_string(),
                                status,
                                file_count: count,
                            },
                        );
                    }
                }
            }
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
        assert_eq!(result.stages[0].file_count, 1, "命名异常阶段的 file_count 应基于真实目录 rtl_final 统计");
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

    #[test]
    fn ai_project_template_layout_detected() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src/python_model/L0_external")).unwrap();
        fs::create_dir_all(root.join("src/python_model/L1_prototype")).unwrap();
        fs::create_dir_all(root.join("src/python_model/L2_structured")).unwrap();
        fs::create_dir_all(root.join("src/verilog_model/rtl")).unwrap();

        let scanned = vec![
            make_file("src/python_model/L0_external/a.py", Language::Python, crate::models::enums::SourceKind::PythonStage),
            make_file("src/python_model/L1_prototype/b.py", Language::Python, crate::models::enums::SourceKind::PythonStage),
            make_file("src/python_model/L2_structured/c.py", Language::Python, crate::models::enums::SourceKind::PythonStage),
            make_file("src/verilog_model/rtl/top.v", Language::Verilog, crate::models::enums::SourceKind::Rtl),
        ];

        let result = detect_stages(root, &scanned);
        assert_eq!(result.stages.len(), 4, "应识别 L0, L1, L2, RTL");
        assert_eq!(result.stages[0].stage_id, "L0");
        assert_eq!(result.stages[0].status, StageStatus::Available);
        assert_eq!(result.stages[0].file_count, 1);
        assert!(result.stages[0].source_path.contains("L0_external"), "source_path 应指向真实目录 L0_external");

        assert_eq!(result.stages[1].stage_id, "L1");
        assert_eq!(result.stages[1].status, StageStatus::Available);
        assert_eq!(result.stages[1].file_count, 1);
        assert!(result.stages[1].source_path.contains("L1_prototype"));

        assert_eq!(result.stages[2].stage_id, "L2");
        assert_eq!(result.stages[2].status, StageStatus::Available);
        assert_eq!(result.stages[2].file_count, 1);

        assert_eq!(result.stages[3].stage_id, "RTL");
        assert_eq!(result.stages[3].status, StageStatus::Available);
        assert_eq!(result.stages[3].file_count, 1);
        assert!(result.stages[3].source_path.contains("verilog_model/rtl"));

        // 缺失 L3~L6
        assert!(result.missing.contains(&"L3".to_string()));
        assert!(result.missing.contains(&"L6".to_string()));
    }

    #[test]
    fn ai_project_template_empty_stage() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src/python_model/L0_external")).unwrap();

        let scanned: Vec<ScannedFile> = vec![];

        let result = detect_stages(root, &scanned);
        assert_eq!(result.stages.len(), 1);
        assert_eq!(result.stages[0].stage_id, "L0");
        assert_eq!(result.stages[0].status, StageStatus::Empty);
        assert_eq!(result.stages[0].file_count, 0);
    }

    #[test]
    fn top_level_priority_over_ai_project_template() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // 顶层 L1
        fs::create_dir(root.join("L1")).unwrap();
        // 深层 L1_prototype
        fs::create_dir_all(root.join("src/python_model/L1_prototype")).unwrap();

        let scanned = vec![
            make_file("L1/top.py", Language::Python, crate::models::enums::SourceKind::PythonStage),
            make_file("src/python_model/L1_prototype/other.py", Language::Python, crate::models::enums::SourceKind::PythonStage),
        ];

        let result = detect_stages(root, &scanned);
        assert_eq!(result.stages.len(), 1, "应只保留顶层 L1");
        assert_eq!(result.stages[0].stage_id, "L1");
        assert_eq!(result.stages[0].file_count, 1, "应只统计顶层 L1 文件");
        assert!(result.stages[0].source_path.contains("L1"));
        assert!(!result.stages[0].source_path.contains("L1_prototype"), "不应指向深层目录");

        // 应有重复候选 warning
        assert!(result.warnings.iter().any(|w| {
            w.message.contains("L1") && w.message.contains("ai_project_template") && w.message.contains("优先使用顶层")
        }), "应有重复阶段候选 warning");
    }

    #[test]
    fn top_level_rtl_priority_over_ai_project_template() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir(root.join("RTL")).unwrap();
        fs::create_dir_all(root.join("src/verilog_model/rtl")).unwrap();

        let scanned = vec![
            make_file("RTL/top.v", Language::Verilog, crate::models::enums::SourceKind::Rtl),
            make_file("src/verilog_model/rtl/other.v", Language::Verilog, crate::models::enums::SourceKind::Rtl),
        ];

        let result = detect_stages(root, &scanned);
        assert_eq!(result.stages.len(), 1);
        assert_eq!(result.stages[0].stage_id, "RTL");
        assert_eq!(result.stages[0].file_count, 1);
        assert!(result.stages[0].source_path.contains("RTL"));
        assert!(!result.stages[0].source_path.contains("verilog_model"));

        assert!(result.warnings.iter().any(|w| {
            w.message.contains("RTL") && w.message.contains("ai_project_template") && w.message.contains("优先使用顶层")
        }));
    }

    #[test]
    fn mixed_layout_both_top_and_deep() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // 顶层 L0
        fs::create_dir(root.join("L0")).unwrap();
        // 深层 L1_prototype, L2_structured
        fs::create_dir_all(root.join("src/python_model/L1_prototype")).unwrap();
        fs::create_dir_all(root.join("src/python_model/L2_structured")).unwrap();
        // 深层 RTL
        fs::create_dir_all(root.join("src/verilog_model/rtl")).unwrap();

        let scanned = vec![
            make_file("L0/top.py", Language::Python, crate::models::enums::SourceKind::PythonStage),
            make_file("src/python_model/L1_prototype/b.py", Language::Python, crate::models::enums::SourceKind::PythonStage),
            make_file("src/python_model/L2_structured/c.py", Language::Python, crate::models::enums::SourceKind::PythonStage),
            make_file("src/verilog_model/rtl/top.v", Language::Verilog, crate::models::enums::SourceKind::Rtl),
        ];

        let result = detect_stages(root, &scanned);
        assert_eq!(result.stages.len(), 4, "L0(顶层), L1(深层), L2(深层), RTL(深层)");

        let l0 = result.stages.iter().find(|s| s.stage_id == "L0").unwrap();
        assert_eq!(l0.status, StageStatus::Available);
        assert!(l0.source_path.contains("L0"));
        assert!(!l0.source_path.contains("python_model"));

        let l1 = result.stages.iter().find(|s| s.stage_id == "L1").unwrap();
        assert_eq!(l1.status, StageStatus::Available);
        assert!(l1.source_path.contains("L1_prototype"));

        let l2 = result.stages.iter().find(|s| s.stage_id == "L2").unwrap();
        assert_eq!(l2.status, StageStatus::Available);
        assert!(l2.source_path.contains("L2_structured"));

        let rtl = result.stages.iter().find(|s| s.stage_id == "RTL").unwrap();
        assert_eq!(rtl.status, StageStatus::Available);
        assert!(rtl.source_path.contains("verilog_model/rtl"));
    }

    #[test]
    fn python_model_not_a_stage() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src/python_model")).unwrap();

        let scanned = vec![make_file(
            "src/python_model/helper.py",
            Language::Python,
            crate::models::enums::SourceKind::PythonStage,
        )];

        let result = detect_stages(root, &scanned);
        // python_model 本身不是阶段，其子目录 L0_external 等才是
        assert!(!result.stages.iter().any(|s| s.stage_id == "python_model" || s.stage_id == "PYTHON_MODEL"));
    }
}
