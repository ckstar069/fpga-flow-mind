use crate::models::error::CommandResult;
use crate::models::workspace_profile::WorkspaceProfile;
use crate::workspace::workspace_builder::build_workspace_profile;

/// Tauri command：打开 workspace 并返回 `WorkspaceProfile`。
///
/// 内部复用 `build_workspace_profile`，不新增业务逻辑分支。
#[tauri::command]
pub fn open_workspace(path: String) -> CommandResult<WorkspaceProfile> {
    build_workspace_profile(&path)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use crate::models::enums::{ErrorCode, WorkspaceValidity};

    use super::*;

    fn touch(root: &Path, rel: &str) {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, "content\n").unwrap();
    }

    #[test]
    fn standard_project_success() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top.py");
        touch(root, "L1/adder.py");
        touch(root, "RTL/top.v");
        touch(root, "README.md");

        let result = open_workspace(root.to_str().unwrap().to_string());
        assert!(result.success, "标准项目应成功");
        let profile = result.data.unwrap();
        assert_eq!(profile.validity, WorkspaceValidity::LikelyValid);
        assert_eq!(profile.stages.len(), 3);
    }

    #[test]
    fn path_not_found_fails() {
        let result = open_workspace("/does/not/exist/xyz".to_string());
        assert!(!result.success);
        assert_eq!(
            result.error.as_ref().unwrap().error_code,
            ErrorCode::PathNotFound
        );
    }

    #[test]
    fn empty_dir_returns_unlikely() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        let result = open_workspace(root.to_str().unwrap().to_string());
        assert!(result.success, "空目录应返回 success=true");
        let profile = result.data.unwrap();
        assert_eq!(profile.validity, WorkspaceValidity::Unlikely);
        assert_eq!(
            profile.error_codes.iter().filter(|c| **c == ErrorCode::NoStageFound).count(),
            1,
            "no_stage_found 应只出现一次"
        );
    }

    #[test]
    fn no_stage_but_code_uncertain() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "script.py");
        touch(root, "design.v");

        let result = open_workspace(root.to_str().unwrap().to_string());
        assert!(result.success);
        let profile = result.data.unwrap();
        assert_eq!(profile.validity, WorkspaceValidity::Uncertain);
        assert!(profile.stages.is_empty());
        assert_eq!(
            profile.error_codes.iter().filter(|c| **c == ErrorCode::NoStageFound).count(),
            1,
            "no_stage_found 应只出现一次"
        );
    }
}
