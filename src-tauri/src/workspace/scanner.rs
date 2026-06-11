use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::models::enums::{ErrorCode, Language, SourceKind};
use crate::models::error::WorkspaceWarning;
use crate::workspace::file_classifier::{classify_file, is_binary};

/// 扫描结果中的单个文件条目
#[derive(Debug, Clone)]
pub struct ScannedFile {
    pub path: PathBuf,
    pub rel_path: String,
    pub language: Language,
    pub source_kind: SourceKind,
    pub size_bytes: u64,
}

/// 扫描输出：文件列表 + 统计 + 警告
#[derive(Debug, Clone)]
pub struct ScanOutput {
    pub files: Vec<ScannedFile>,
    pub file_type_stats: HashMap<String, u64>,
    pub warnings: Vec<WorkspaceWarning>,
}

/// 只读 DFS 扫描 workspace 文件。
///
/// 约束：
/// - 递归深度 ≤ 3（根目录深度 = 0）
/// - 单目录文件数 ≤ 1000，超限记录 warning
/// - 总文件数 ≤ 5000，超限停止扫描
/// - 超时 30 秒，记录 `scan_timeout` warning，返回已收集结果
/// - 跳过 symlink、二进制文件
/// - 大文件（> 5MB）记录 `file_too_large` warning，但不跳过
/// - 不可读文件记录 `file_unreadable` warning
pub fn scan_workspace_files(root: &Path) -> ScanOutput {
    let mut output = ScanOutput {
        files: Vec::with_capacity(256),
        file_type_stats: HashMap::new(),
        warnings: Vec::new(),
    };

    let deadline = Instant::now() + Duration::from_secs(30);
    let total_limit = 5000;
    let per_dir_limit = 1000;
    let big_file_threshold = 5 * 1024 * 1024;

    scan_dir(
        root,
        root,
        0,
        &mut output,
        deadline,
        total_limit,
        per_dir_limit,
        big_file_threshold,
    );

    output
}

fn scan_dir(
    root: &Path,
    dir: &Path,
    depth: usize,
    output: &mut ScanOutput,
    deadline: Instant,
    total_limit: usize,
    per_dir_limit: usize,
    big_file_threshold: u64,
) {
    if Instant::now() > deadline {
        output.warnings.push(WorkspaceWarning {
            error_code: ErrorCode::ScanTimeout,
            message: "扫描超时，已返回部分结果".to_string(),
            source_path: Some(dir.display().to_string()),
            related_stage_id: None,
            recoverable: true,
        });
        return;
    }

    if depth > 3 {
        output.warnings.push(WorkspaceWarning {
            error_code: ErrorCode::ScanTimeout,
            message: format!("目录深度超过 3，跳过: {}", dir.display()),
            source_path: Some(dir.display().to_string()),
            related_stage_id: None,
            recoverable: true,
        });
        return;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => {
            output.warnings.push(WorkspaceWarning {
                error_code: ErrorCode::FileUnreadable,
                message: format!("无法读取目录: {}", dir.display()),
                source_path: Some(dir.display().to_string()),
                related_stage_id: None,
                recoverable: true,
            });
            return;
        }
    };

    let mut dir_file_count = 0usize;
    let mut dirs_to_recurse: Vec<PathBuf> = Vec::new();

    for entry in entries {
        if Instant::now() > deadline {
            output.warnings.push(WorkspaceWarning {
                error_code: ErrorCode::ScanTimeout,
                message: "扫描超时，已返回部分结果".to_string(),
                source_path: Some(dir.display().to_string()),
                related_stage_id: None,
                recoverable: true,
            });
            return;
        }

        if output.files.len() >= total_limit {
            output.warnings.push(WorkspaceWarning {
                error_code: ErrorCode::ScanTimeout,
                message: format!("总文件数超过 {}，停止扫描", total_limit),
                source_path: Some(dir.display().to_string()),
                related_stage_id: None,
                recoverable: true,
            });
            return;
        }

        let entry = match entry {
            Ok(e) => e,
            Err(_) => {
                output.warnings.push(WorkspaceWarning {
                    error_code: ErrorCode::FileUnreadable,
                    message: "无法读取目录项".to_string(),
                    source_path: Some(dir.display().to_string()),
                    related_stage_id: None,
                    recoverable: true,
                });
                continue;
            }
        };

        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(t) => t,
            Err(_) => continue,
        };

        // 跳过 symlink
        if file_type.is_symlink() {
            continue;
        }

        if file_type.is_dir() {
            dirs_to_recurse.push(path);
            continue;
        }

        if !file_type.is_file() {
            continue;
        }

        // 单目录文件数限制
        dir_file_count += 1;
        if dir_file_count > per_dir_limit {
            if dir_file_count == per_dir_limit + 1 {
                output.warnings.push(WorkspaceWarning {
                    error_code: ErrorCode::ScanTimeout,
                    message: format!("目录文件数超过 {}，跳过剩余文件", per_dir_limit),
                    source_path: Some(dir.display().to_string()),
                    related_stage_id: None,
                    recoverable: true,
                });
            }
            continue;
        }

        // 文件元数据
        let size = match entry.metadata() {
            Ok(m) => m.len(),
            Err(_) => {
                output.warnings.push(WorkspaceWarning {
                    error_code: ErrorCode::FileUnreadable,
                    message: format!("无法读取文件元数据: {}", path.display()),
                    source_path: Some(path.display().to_string()),
                    related_stage_id: None,
                    recoverable: true,
                });
                continue;
            }
        };

        // 大文件 warning
        if size > big_file_threshold {
            output.warnings.push(WorkspaceWarning {
                error_code: ErrorCode::FileTooLarge,
                message: format!(
                    "文件超过 5MB，仅读取前 100 行进行类型识别: {}",
                    path.display()
                ),
                source_path: Some(path.display().to_string()),
                related_stage_id: None,
                recoverable: true,
            });
        }

        // 二进制跳过
        if is_binary(&path) {
            continue;
        }

        let rel = match path.strip_prefix(root) {
            Ok(p) => p.display().to_string(),
            Err(_) => path.display().to_string(),
        };

        let (language, source_kind) = classify_file(&path);

        output.files.push(ScannedFile {
            path: path.clone(),
            rel_path: rel,
            language,
            source_kind,
            size_bytes: size,
        });

        // 更新文件类型统计
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown")
            .to_lowercase();
        *output.file_type_stats.entry(ext).or_insert(0) += 1;
    }

    // 递归子目录
    for sub in dirs_to_recurse {
        scan_dir(
            root,
            &sub,
            depth + 1,
            output,
            deadline,
            total_limit,
            per_dir_limit,
            big_file_threshold,
        );
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn touch(root: &Path, rel: &str) -> PathBuf {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, "content\n").unwrap();
        path
    }

    #[test]
    fn standard_project_scan() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top.py");
        touch(root, "L1/adder.py");
        touch(root, "RTL/top.v");
        touch(root, "README.md");

        let out = scan_workspace_files(root);
        assert_eq!(out.files.len(), 4);
        assert!(out.file_type_stats.contains_key("py"));
        assert!(out.file_type_stats.contains_key("v"));
        assert!(out.file_type_stats.contains_key("md"));
        assert!(out.warnings.is_empty());
    }

    #[test]
    fn skips_symlink() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let real = root.join("real_dir");
        fs::create_dir(&real).unwrap();
        touch(&real, "file.py");

        let link = root.join("link_dir");
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

        let out = scan_workspace_files(root);
        assert_eq!(out.files.len(), 1);
    }

    #[test]
    fn big_file_warning() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let big = root.join("big.v");
        fs::write(&big, vec![b'x'; 6 * 1024 * 1024]).unwrap();

        let out = scan_workspace_files(root);
        assert_eq!(out.files.len(), 1);
        assert_eq!(out.warnings.len(), 1);
        assert_eq!(out.warnings[0].error_code, ErrorCode::FileTooLarge);
    }

    #[test]
    fn depth_limit() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "a/b/c/d/deep.py");

        let out = scan_workspace_files(root);
        // depth > 3 被跳过
        assert_eq!(out.files.len(), 0);
        assert_eq!(out.warnings.len(), 1);
    }
}
