use std::path::Path;

use chrono::Utc;

use crate::models::enums::ErrorCode;
use crate::models::error::CommandResult;
use crate::models::workspace_profile::{StageSummary, WorkspaceProfile};
use crate::workspace::external_refs::detect_urban_wireless;
use crate::workspace::safety_guard::validate_workspace_root;
use crate::workspace::scanner::scan_workspace_files;
use crate::workspace::stage_detector::detect_stages;
use crate::workspace::validity::{calculate_validity, collect_error_codes};

/// 组装完整的 WorkspaceProfile。
///
/// 流程：
/// 1. 路径校验（P1-T03）
/// 2. 扫描文件（P1-T04）
/// 3. 识别阶段（P1-T05）
/// 4. 识别外部引用（P1-T05）
/// 5. 计算 validity（P1-T05）
/// 6. 组装 WorkspaceProfile
///
/// 路径校验失败 → success=false + CommandError，无 WorkspaceProfile。
/// no_stage_found → success=true + WorkspaceProfile，error_codes 含 no_stage_found。
pub fn build_workspace_profile(path_str: &str) -> CommandResult<WorkspaceProfile> {
    // 1. 路径校验
    let path = Path::new(path_str);
    let validated = validate_workspace_root(path);
    if !validated.success {
        return CommandResult {
            success: false,
            data: None,
            error: validated.error,
            warnings: validated.warnings,
        };
    }

    let root = validated.data.unwrap();
    let root_str = root.display().to_string();

    // 2. 扫描文件
    let scan = scan_workspace_files(&root);

    // 3. 识别阶段
    let detection = detect_stages(&root, &scan.files);

    // 4. 识别外部引用
    let mut external_refs = Vec::new();
    let mut ext_set = std::collections::HashSet::new();
    for file in &scan.files {
        for r in detect_urban_wireless(&file.path) {
            if ext_set.insert(r.clone()) {
                external_refs.push(r);
            }
        }
    }

    // 5. 计算 validity
    let (validity, mut validity_reasons) = calculate_validity(&detection.stages, &scan.files);
    validity_reasons.extend(detection.validity_reasons);

    // 6. 组装 StageSummary
    let stages: Vec<StageSummary> = detection
        .stages
        .iter()
        .map(|s| StageSummary {
            stage_id: s.stage_id.clone(),
            source_path: s.source_path.clone(),
            file_count: s.file_count,
            status: s.status,
        })
        .collect();

    // error_codes（workspace 级）
    let mut error_codes = collect_error_codes(&detection.stages, &scan.files);
    if detection.stages.is_empty() {
        error_codes.push(ErrorCode::NoStageFound);
    }

    // 合并 warnings
    let mut warnings = scan.warnings;
    warnings.extend(detection.warnings);

    let workspace_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let profile = WorkspaceProfile {
        workspace_name,
        root_path: root_str,
        stages,
        file_type_stats: scan.file_type_stats,
        external_refs,
        validity,
        validity_reasons,
        warnings,
        error_codes,
        scan_timestamp: Utc::now().to_rfc3339(),
        version: "1.0.0".to_string(),
    };

    CommandResult {
        success: true,
        data: Some(profile),
        error: None,
        warnings: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn touch(root: &Path, rel: &str) {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, "content\n").unwrap();
    }

    #[test]
    fn standard_project() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top.py");
        touch(root, "L1/adder.py");
        touch(root, "RTL/top.v");
        touch(root, "README.md");

        let result = build_workspace_profile(root.to_str().unwrap());
        assert!(result.success, "应成功");
        let profile = result.data.unwrap();
        assert_eq!(profile.stages.len(), 3);
        assert_eq!(profile.validity, crate::models::enums::WorkspaceValidity::LikelyValid);
        assert!(profile.error_codes.is_empty());
    }

    #[test]
    fn no_stage_with_code_is_uncertain() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "script.py");
        touch(root, "design.v");

        let result = build_workspace_profile(root.to_str().unwrap());
        assert!(result.success);
        let profile = result.data.unwrap();
        assert!(profile.stages.is_empty());
        assert_eq!(profile.validity, crate::models::enums::WorkspaceValidity::Uncertain);
        assert!(profile.error_codes.contains(&ErrorCode::NoStageFound));
    }

    #[test]
    fn empty_dir_is_unlikely() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        let result = build_workspace_profile(root.to_str().unwrap());
        assert!(result.success);
        let profile = result.data.unwrap();
        assert!(profile.stages.is_empty());
        assert_eq!(profile.validity, crate::models::enums::WorkspaceValidity::Unlikely);
        assert!(profile.error_codes.contains(&ErrorCode::NoStageFound));
    }

    #[test]
    fn early_python_only() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top.py");
        touch(root, "L1/adder.py");
        touch(root, "L2/test.py");

        let result = build_workspace_profile(root.to_str().unwrap());
        assert!(result.success);
        let profile = result.data.unwrap();
        assert_eq!(profile.stages.len(), 3);
        assert_eq!(profile.validity, crate::models::enums::WorkspaceValidity::LikelyValid);
    }

    #[test]
    fn only_rtl_sv() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "RTL/top.sv");

        let result = build_workspace_profile(root.to_str().unwrap());
        assert!(result.success);
        let profile = result.data.unwrap();
        assert_eq!(profile.stages.len(), 1);
        assert_eq!(profile.validity, crate::models::enums::WorkspaceValidity::LikelyValid);
    }

    #[test]
    fn partial_stages_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top.py");
        touch(root, "RTL/top.v");

        let result = build_workspace_profile(root.to_str().unwrap());
        assert!(result.success);
        let profile = result.data.unwrap();
        assert_eq!(profile.stages.len(), 2);
        assert!(profile.warnings.iter().any(|w| w.message.contains("L1")));
        assert_eq!(profile.validity, crate::models::enums::WorkspaceValidity::LikelyValid);
    }

    #[test]
    fn naming_anomaly_project() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "rtl_final/top.v");

        let result = build_workspace_profile(root.to_str().unwrap());
        assert!(result.success);
        let profile = result.data.unwrap();
        assert_eq!(profile.stages.len(), 1);
        assert_eq!(profile.stages[0].stage_id, "RTL");
        assert_eq!(profile.stages[0].status, crate::models::enums::StageStatus::NamingAnomaly);
    }

    #[test]
    fn empty_stage_project() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir(root.join("L0")).unwrap();
        touch(root, "L1/top.py");

        let result = build_workspace_profile(root.to_str().unwrap());
        assert!(result.success);
        let profile = result.data.unwrap();
        let l0 = profile.stages.iter().find(|s| s.stage_id == "L0").unwrap();
        assert_eq!(l0.status, crate::models::enums::StageStatus::Empty);
    }

    #[test]
    fn symlink_in_scan_skipped() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let real = root.join("real");
        fs::create_dir(&real).unwrap();
        touch(&real, "file.py");
        let link = root.join("link");
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(&real, &link).unwrap();
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_dir;
            symlink_dir(&real, &link).unwrap();
        }

        let result = build_workspace_profile(root.to_str().unwrap());
        assert!(result.success);
        let profile = result.data.unwrap();
        // symlink 被跳过，不进入 file_type_stats
        assert_eq!(profile.file_type_stats.get("py"), Some(&1));
    }

    #[test]
    fn external_refs_detected() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top.py");
        let ext_path = root.join("L0/external.py");
        fs::write(&ext_path, "from urban_wireless import channel_model\n").unwrap();

        let result = build_workspace_profile(root.to_str().unwrap());
        assert!(result.success);
        let profile = result.data.unwrap();
        assert!(profile.external_refs.contains(&"urban_wireless".to_string()));
    }

    #[test]
    fn path_not_found_fails() {
        let result = build_workspace_profile("/does/not/exist/xyz");
        assert!(!result.success);
        assert_eq!(
            result.error.as_ref().unwrap().error_code,
            ErrorCode::PathNotFound
        );
    }

    #[test]
    fn docs_only_no_stages_uncertain() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "README.md");

        let result = build_workspace_profile(root.to_str().unwrap());
        assert!(result.success);
        let profile = result.data.unwrap();
        assert_eq!(profile.validity, crate::models::enums::WorkspaceValidity::Unlikely);
    }

    #[test]
    fn docs_only_with_stages_uncertain() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir(root.join("L0")).unwrap();
        touch(root, "L0/README.md");

        let result = build_workspace_profile(root.to_str().unwrap());
        assert!(result.success);
        let profile = result.data.unwrap();
        assert_eq!(profile.validity, crate::models::enums::WorkspaceValidity::Uncertain);
    }
}
