use std::collections::HashSet;
use std::path::Path;

use crate::models::enums::{ErrorCode, StageStatus};
use crate::models::error::{CommandError, CommandResult};
use crate::models::stage_context::{StageContext, StageFile, UpstreamRef};
use crate::workspace::external_refs::detect_urban_wireless;
use crate::workspace::safety_guard::validate_workspace_root;
use crate::workspace::scanner::scan_workspace_files;
use crate::workspace::stage_detector::detect_stages;

/// 共享校验 + StageContext 构建（供 select_stage 和 collect_evidence 复用）。
///
/// 处理流程：
/// 1. 校验 root_path（复用 safety_guard）
/// 2. 空 stage_id → success=false + CommandError
/// 3. 扫描 workspace + 识别阶段
/// 4. stage_id 不存在 → success=false + CommandError
/// 5. 阶段不可读 → success=false + stage_unreadable
/// 6. 收集阶段文件、外部依赖、上游引用推断
/// 7. 空阶段 → success=true + error_code=stage_empty
pub fn resolve_stage_context(root_path: &str, stage_id: &str) -> CommandResult<StageContext> {
    // 1. 校验 root_path
    let validated = validate_workspace_root(Path::new(root_path));
    if !validated.success {
        return CommandResult {
            success: false,
            data: None,
            error: validated.error,
            warnings: validated.warnings,
        };
    }
    let root = validated.data.unwrap();

    // 2. 空 stage_id
    if stage_id.trim().is_empty() {
        return CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::NotDirectory,
                message: "阶段标识不能为空".to_string(),
                recoverable: false,
                details: None,
                source_path: None,
            }),
            warnings: Vec::new(),
        };
    }

    // 3. 扫描 workspace 获取阶段信息
    let scan = scan_workspace_files(&root);
    let detection = detect_stages(&root, &scan.files);

    // 4. 查找 stage_id
    let stage = detection.stages.iter().find(|s| s.stage_id == stage_id);
    if stage.is_none() {
        return CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::NotDirectory,
                message: format!("阶段 {} 不存在", stage_id),
                recoverable: false,
                details: None,
                source_path: Some(root.display().to_string()),
            }),
            warnings: Vec::new(),
        };
    }
    let stage = stage.unwrap();

    // 5. 阶段不可读
    if stage.status == StageStatus::Unreadable {
        return CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::StageUnreadable,
                message: format!("阶段 {} 不可读", stage_id),
                recoverable: false,
                details: None,
                source_path: Some(stage.source_path.clone()),
            }),
            warnings: Vec::new(),
        };
    }

    // 6. 收集阶段文件（基于已扫描的全 workspace 文件过滤）
    // 使用 stage.source_path（真实阶段目录路径）过滤，而非 stage_id
    // 这样命名异常阶段（如 rtl_final/ -> stage_id = "RTL"）也能正确过滤
    let stage_path = Path::new(&stage.source_path);
    let stage_scanned_files: Vec<_> = scan
        .files
        .iter()
        .filter(|f| f.path.starts_with(stage_path))
        .collect();

    let files: Vec<StageFile> = stage_scanned_files
        .iter()
        .map(|f| StageFile {
            source_path: f.path.display().to_string(),
            language: f.language,
            source_kind: f.source_kind,
            size_bytes: Some(f.size_bytes),
        })
        .collect();

    // 7. 外部依赖（去重）
    let mut external_deps = Vec::new();
    let mut ext_set = HashSet::new();
    for f in &stage_scanned_files {
        for dep in detect_urban_wireless(&f.path) {
            if ext_set.insert(dep.clone()) {
                external_deps.push(dep);
            }
        }
    }

    // 8. 上游引用推断
    let upstream_refs = infer_upstream_refs(&detection.stages, &stage_id);

    // 9. 空阶段判断
    let error_code = if files.is_empty() {
        Some(ErrorCode::StageEmpty)
    } else {
        None
    };

    let context = StageContext {
        stage_id: stage_id.to_string(),
        source_path: stage.source_path.clone(),
        files,
        external_deps,
        upstream_refs,
        error_code,
    };

    CommandResult {
        success: true,
        data: Some(context),
        error: None,
        warnings: Vec::new(),
    }
}

/// Tauri command：选择单个阶段并返回 `StageContext`。
///
/// 委托 `resolve_stage_context` 执行校验与构建。
#[tauri::command]
pub fn select_stage(root_path: String, stage_id: String) -> CommandResult<StageContext> {
    resolve_stage_context(&root_path, &stage_id)
}

/// Phase 1 最小上游引用推断。
///
/// 检查前一阶段目录中是否存在 `interface_*.py`、`*_interface.py`、`*_interface.v`、`*_interface.sv` 等文件名模式。
/// 所有推断结果标记 `inferred = true`。
fn infer_upstream_refs(
    stages: &[crate::workspace::stage_detector::StageInfo],
    stage_id: &str,
) -> Vec<UpstreamRef> {
    let standard_order = ["L0", "L1", "L2", "L3", "L4", "L5", "L6", "RTL"];

    let current_idx = match standard_order.iter().position(|&s| s == stage_id) {
        Some(i) if i > 0 => i,
        _ => return Vec::new(),
    };

    let prev_id = standard_order[current_idx - 1];

    // 在已识别阶段中查找前一阶段
    let prev_stage = match stages.iter().find(|s| s.stage_id == prev_id) {
        Some(s) => s,
        None => return Vec::new(),
    };

    let prev_path = Path::new(&prev_stage.source_path);
    let mut refs = Vec::new();

    let entries = match std::fs::read_dir(prev_path) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let is_file = match entry.file_type() {
            Ok(t) => t.is_file(),
            Err(_) => continue,
        };
        if !is_file {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        if name.starts_with("interface_")
            || name.ends_with("_interface.py")
            || name.ends_with("_interface.v")
            || name.ends_with("_interface.sv")
        {
            refs.push(UpstreamRef {
                stage_id: prev_id.to_string(),
                interface_file_path: Some(path.display().to_string()),
                inferred: true,
            });
        }
    }

    refs
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use crate::models::enums::ErrorCode;

    use super::*;

    fn touch(root: &Path, rel: &str, content: &str) {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, content).unwrap();
    }

    #[test]
    fn available_stage_returns_files() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top.py", "def top(): pass\n");
        touch(root, "L0/helper.py", "def helper(): pass\n");

        let result = select_stage(root.to_str().unwrap().to_string(), "L0".to_string());
        assert!(result.success, "可用阶段应成功");
        let ctx = result.data.unwrap();
        assert_eq!(ctx.stage_id, "L0");
        assert_eq!(ctx.files.len(), 2);
        assert!(ctx.error_code.is_none());
    }

    #[test]
    fn empty_stage_returns_stage_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir(root.join("L0")).unwrap();
        touch(root, "L1/top.py", "def top(): pass\n");

        let result = select_stage(root.to_str().unwrap().to_string(), "L0".to_string());
        assert!(result.success, "空阶段应返回 success=true");
        let ctx = result.data.unwrap();
        assert!(ctx.files.is_empty());
        assert_eq!(ctx.error_code, Some(ErrorCode::StageEmpty));
    }

    #[test]
    fn nonexistent_stage_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top.py", "def top(): pass\n");

        let result = select_stage(root.to_str().unwrap().to_string(), "L9".to_string());
        assert!(!result.success);
        assert_eq!(result.error.as_ref().unwrap().error_code, ErrorCode::NotDirectory);
    }

    #[test]
    fn empty_stage_id_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top.py", "def top(): pass\n");

        let result = select_stage(root.to_str().unwrap().to_string(), "".to_string());
        assert!(!result.success);
        assert_eq!(result.error.as_ref().unwrap().error_code, ErrorCode::NotDirectory);
    }

    #[test]
    fn detects_external_deps() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top.py", "from urban_wireless import channel_model\n");

        let result = select_stage(root.to_str().unwrap().to_string(), "L0".to_string());
        assert!(result.success);
        let ctx = result.data.unwrap();
        assert!(ctx.external_deps.contains(&"urban_wireless".to_string()));
    }

    #[test]
    fn upstream_refs_inferred() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/interface_top.py", "# interface\n");
        touch(root, "L1/top.py", "def top(): pass\n");

        let result = select_stage(root.to_str().unwrap().to_string(), "L1".to_string());
        assert!(result.success);
        let ctx = result.data.unwrap();
        assert_eq!(ctx.upstream_refs.len(), 1);
        assert_eq!(ctx.upstream_refs[0].stage_id, "L0");
        assert!(ctx.upstream_refs[0].inferred);
        assert!(ctx.upstream_refs[0].interface_file_path.as_ref().unwrap().contains("interface_top.py"));
    }

    #[test]
    fn upstream_refs_first_stage_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top.py", "def top(): pass\n");

        let result = select_stage(root.to_str().unwrap().to_string(), "L0".to_string());
        assert!(result.success);
        let ctx = result.data.unwrap();
        assert!(ctx.upstream_refs.is_empty(), "L0 无上游阶段");
    }

    #[test]
    fn root_path_not_found_fails() {
        let result = select_stage("/does/not/exist".to_string(), "L0".to_string());
        assert!(!result.success);
        assert_eq!(
            result.error.as_ref().unwrap().error_code,
            ErrorCode::PathNotFound
        );
    }

    #[test]
    fn naming_anomaly_stage_returns_files() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "rtl_final/top.v", "module top; endmodule\n");

        let result = select_stage(root.to_str().unwrap().to_string(), "RTL".to_string());
        assert!(result.success, "命名异常阶段应成功，不应误判为空阶段");
        let ctx = result.data.unwrap();
        assert_eq!(ctx.stage_id, "RTL");
        assert!(ctx.source_path.contains("rtl_final"), "source_path 应指向真实目录 rtl_final");
        assert_eq!(ctx.files.len(), 1, "应返回 rtl_final/top.v");
        assert_eq!(ctx.files[0].language, crate::models::enums::Language::Verilog);
        assert!(ctx.error_code.is_none(), "有文件时不应返回 stage_empty");
    }

    #[test]
    fn upstream_refs_interface_py_pattern() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top_interface.py", "# interface definition\n");
        touch(root, "L1/top.py", "def top(): pass\n");

        let result = select_stage(root.to_str().unwrap().to_string(), "L1".to_string());
        assert!(result.success);
        let ctx = result.data.unwrap();
        assert_eq!(ctx.upstream_refs.len(), 1, "应通过 *_interface.py 模式推断出上游引用");
        assert_eq!(ctx.upstream_refs[0].stage_id, "L0");
        assert!(ctx.upstream_refs[0].inferred);
        assert!(ctx.upstream_refs[0].interface_file_path.as_ref().unwrap().contains("top_interface.py"));
    }
}
