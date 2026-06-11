use std::io;
use std::path::{Path, PathBuf};

use crate::models::enums::ErrorCode;
use crate::models::error::{CommandError, CommandResult};

/// 校验 workspace 根路径是否适合打开。
///
/// 校验顺序（与文档契约一致）：
/// 1. 空路径 → `not_directory`
/// 2. 不存在 → `path_not_found`（通过 `symlink_metadata` 检测，不跟随 symlink）
/// 3. 根路径是 symlink → `permission_denied`
/// 4. 非目录 → `not_directory`
/// 5. 不可读 → `permission_denied`
/// 6. 通过后返回规范化绝对路径
///
/// 全程不写入目标目录、不创建文件、不跟随 symlink root。
pub fn validate_workspace_root(path: &Path) -> CommandResult<PathBuf> {
    // 1. 空路径
    if path.as_os_str().is_empty() {
        return CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::NotDirectory,
                message: "路径为空".to_string(),
                recoverable: false,
                details: Some("提供的路径字符串为空".to_string()),
                source_path: Some("".to_string()),
            }),
            warnings: Vec::new(),
        };
    }

    // 2. 存在性检查（不跟随 symlink）
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return CommandResult {
                success: false,
                data: None,
                error: Some(CommandError {
                    error_code: ErrorCode::PathNotFound,
                    message: "路径不存在".to_string(),
                    recoverable: false,
                    details: Some(format!("{}", e)),
                    source_path: Some(path.display().to_string()),
                }),
                warnings: Vec::new(),
            };
        }
        Err(e) => {
            return CommandResult {
                success: false,
                data: None,
                error: Some(CommandError {
                    error_code: ErrorCode::PermissionDenied,
                    message: "无法访问路径".to_string(),
                    recoverable: false,
                    details: Some(format!("{}", e)),
                    source_path: Some(path.display().to_string()),
                }),
                warnings: Vec::new(),
            };
        }
    };

    // 3. symlink root 拒绝
    if metadata.file_type().is_symlink() {
        return CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::PermissionDenied,
                message: "根路径不能是符号链接".to_string(),
                recoverable: false,
                details: Some("安全策略：不允许选择符号链接作为项目根路径".to_string()),
                source_path: Some(path.display().to_string()),
            }),
            warnings: Vec::new(),
        };
    }

    // 4. 必须是目录
    if !metadata.is_dir() {
        return CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::NotDirectory,
                message: "路径不是目录".to_string(),
                recoverable: false,
                details: Some("选择的路径是一个文件而非目录".to_string()),
                source_path: Some(path.display().to_string()),
            }),
            warnings: Vec::new(),
        };
    }

    // 5. 可读性检查（尝试读取目录内容）
    if std::fs::read_dir(path).is_err() {
        return CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::PermissionDenied,
                message: "目录不可读".to_string(),
                recoverable: false,
                details: Some("无法读取目录内容".to_string()),
                source_path: Some(path.display().to_string()),
            }),
            warnings: Vec::new(),
        };
    }

    // 6. 规范化绝对路径（此时已确认不是 symlink，canonicalize 安全）
    let canonical = match path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            return CommandResult {
                success: false,
                data: None,
                error: Some(CommandError {
                    error_code: ErrorCode::PermissionDenied,
                    message: "无法规范化路径".to_string(),
                    recoverable: false,
                    details: Some(format!("{}", e)),
                    source_path: Some(path.display().to_string()),
                }),
                warnings: Vec::new(),
            };
        }
    };

    CommandResult {
        success: true,
        data: Some(canonical),
        error: None,
        warnings: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn unwrap_code(result: &CommandResult<PathBuf>) -> Option<ErrorCode> {
        result.error.as_ref().map(|e| e.error_code)
    }

    #[test]
    fn existing_directory_passes() {
        let tmp = tempfile::tempdir().unwrap();
        let result = validate_workspace_root(tmp.path());
        assert!(result.success, "现有目录应通过校验");
        assert!(result.data.is_some(), "应返回规范化路径");
        assert_eq!(unwrap_code(&result), None);
    }

    #[test]
    fn nonexistent_path_returns_not_found() {
        let path = Path::new("/this/path/does/not/exist/12345");
        let result = validate_workspace_root(path);
        assert!(!result.success);
        assert_eq!(unwrap_code(&result), Some(ErrorCode::PathNotFound));
    }

    #[test]
    fn regular_file_returns_not_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = tmp.path().join("test.txt");
        fs::write(&file_path, b"hello").unwrap();

        let result = validate_workspace_root(&file_path);
        assert!(!result.success);
        assert_eq!(unwrap_code(&result), Some(ErrorCode::NotDirectory));
    }

    #[test]
    fn symlink_root_returns_permission_denied() {
        let tmp = tempfile::tempdir().unwrap();
        let real_dir = tmp.path().join("real");
        fs::create_dir(&real_dir).unwrap();
        let link_path = tmp.path().join("link");

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(&real_dir, &link_path).unwrap();
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_dir;
            symlink_dir(&real_dir, &link_path).unwrap();
        }

        let result = validate_workspace_root(&link_path);
        assert!(!result.success);
        assert_eq!(unwrap_code(&result), Some(ErrorCode::PermissionDenied));
    }

    #[test]
    fn empty_path_returns_not_directory() {
        let path = Path::new("");
        let result = validate_workspace_root(path);
        assert!(!result.success);
        assert_eq!(unwrap_code(&result), Some(ErrorCode::NotDirectory));
    }
}
