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
        &[],
        &mut output,
        deadline,
        total_limit,
        per_dir_limit,
        big_file_threshold,
    );

    output
}

/// 判断 ai_project_template 中的"阶段根目录"（深层有效源码目录的入口）。
///
/// 仅匹配直接位于 `src/python_model/` 下的 `L*_xxx` 目录，
/// 以及 `src/verilog_model/rtl`。这些是允许更深递归的入口目录；
/// 一旦进入这些入口，其全部子孙目录都应继续递归（见 `is_inside_deep_source_tree`）。
fn is_deep_source_root(name: &str, parent_chain: &[String]) -> bool {
    let lower = name.to_lowercase();
    // 直接位于 src/python_model/ 下的 L*_xxx 目录
    let is_lstar = lower.starts_with("l0_") || lower.starts_with("l1_") || lower.starts_with("l2_")
        || lower.starts_with("l3_") || lower.starts_with("l4_") || lower.starts_with("l5_")
        || lower.starts_with("l6_");
    if is_lstar {
        if parent_chain.len() >= 2 {
            let parent = parent_chain[parent_chain.len() - 1].to_lowercase();
            let grandparent = parent_chain[parent_chain.len() - 2].to_lowercase();
            if parent == "python_model" && grandparent == "src" {
                return true;
            }
        }
    }
    // rtl 目录位于 src/verilog_model/ 下
    if lower == "rtl" {
        if parent_chain.len() >= 2 {
            let parent = parent_chain[parent_chain.len() - 1].to_lowercase();
            let grandparent = parent_chain[parent_chain.len() - 2].to_lowercase();
            if parent == "verilog_model" && grandparent == "src" {
                return true;
            }
        }
    }
    false
}

/// 判断当前目录是否处于 ai_project_template 深层源码树之内。
///
/// 如果当前目录自身是阶段根（`L*_xxx` / `rtl`），或者其祖先链中存在阶段根，
/// 则视为"深层源码树内部"，允许超过默认深度 3 的进一步递归。
/// 这样 `src/python_model/L0_external/rx_02_coarse_sync/sub/...` 这类深层子包
/// 也能被完整扫描，而噪声目录（`.git` / `.claude` / `__pycache__` 等）仍受
/// `should_skip_dir` 拦截，不会无限制递归。
fn is_inside_deep_source_tree(name: &str, parent_chain: &[String]) -> bool {
    if is_deep_source_root(name, parent_chain) {
        return true;
    }
    // 沿祖先链查找阶段根：阶段根的路径模式为 .../python_model/L*_xxx 或 .../verilog_model/rtl
    // 即祖先链中存在某个元素是阶段根（其父为 python_model/verilog_model 且祖父为 src）
    let n = parent_chain.len();
    for i in 0..n {
        let ancestor = parent_chain[i].to_lowercase();
        if i >= 2 {
            let parent = parent_chain[i - 1].to_lowercase();
            let grandparent = parent_chain[i - 2].to_lowercase();
            if is_deep_source_root(&ancestor, &[grandparent.clone(), parent]) {
                return true;
            }
        } else {
            // 链过短，直接构造空父链判断（阶段根需 src/python_model，i>=2 才满足）
            if is_deep_source_root(&ancestor, &[]) {
                return true;
            }
        }
    }
    false
}

fn should_skip_dir(name: &str) -> bool {
    let lower = name.to_lowercase();
    matches!(
        lower.as_str(),
        ".git"
            | ".claude"
            | "__pycache__"
            | ".pytest_cache"
            | ".mypy_cache"
            | ".ruff_cache"
            | ".egg-info"
            | "reports"
            | "vivado"
            | "build"
            | "dist"
            | "node_modules"
            | "target"
            | ".idea"
            | ".vscode"
            | ".venv"
            | "venv"
            | "sim_build"
            | ".tox"
            | "htmlcov"
    )
}

fn should_skip_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    matches!(
        lower.as_str(),
        ".ds_store" | ".coverage" | ".gitignore" | ".editorconfig"
    )
}

fn scan_dir(
    root: &Path,
    dir: &Path,
    depth: usize,
    parent_chain: &[String],
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

    // ai_project_template 深层有效源码树（src/python_model/L*_xxx、src/verilog_model/rtl
    // 及其子孙目录）允许超过默认深度 3 的递归，避免漏扫真实深层源码；
    // 其他路径（含噪声目录的深层子目录）仍受深度限制，并由 should_skip_dir 提前拦截。
    let dir_name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    if depth > 3 && !is_inside_deep_source_tree(&dir_name, parent_chain) {
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
            let entry_dir_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if should_skip_dir(entry_dir_name) {
                continue;
            }
            dirs_to_recurse.push(path);
            continue;
        }

        if !file_type.is_file() {
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if should_skip_file(file_name) {
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
        let sub_name = sub
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let mut new_chain = parent_chain.to_vec();
        new_chain.push(sub_name);
        scan_dir(
            root,
            &sub,
            depth + 1,
            &new_chain,
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
    fn skips_noise_directories() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "src/top.py");
        touch(root, ".git/config");
        touch(root, ".claude/commands/deep/file.md");
        touch(root, "__pycache__/module.cpython-311.pyc");
        touch(root, ".pytest_cache/v/cache/nodeids");
        touch(root, "vivado/project.xpr");
        touch(root, "reports/summary.html");
        touch(root, "build/artifact.o");
        touch(root, "node_modules/pkg/index.js");
        touch(root, "target/debug/app");
        touch(root, "venv/bin/python");
        touch(root, ".coverage");

        let out = scan_workspace_files(root);
        // 只应收集 src/top.py（.DS_Store 不是目录，是文件，会被 is_binary 判定为二进制跳过）
        assert_eq!(out.files.len(), 1, "应跳过所有噪声目录");
        assert_eq!(out.files[0].rel_path, "src/top.py");
        assert!(out.warnings.is_empty(), "跳过噪声目录不应产生 warnings");
    }

    #[test]
    fn noise_skip_does_not_hide_valid_src() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "src/python_model/L0_external/a.py");
        touch(root, "src/python_model/L1_prototype/b.py");
        touch(root, "src/verilog_model/rtl/top.v");
        touch(root, "tests/test_l0.py");
        touch(root, ".claude/commands/deep/cmd.md");
        touch(root, "__pycache__/cache.pyc");
        touch(root, "vivado/build.tcl");

        let out = scan_workspace_files(root);
        // 应收集 4 个有效文件，跳过噪声
        let rel_paths: Vec<_> = out.files.iter().map(|f| f.rel_path.clone()).collect();
        assert!(rel_paths.contains(&"src/python_model/L0_external/a.py".to_string()));
        assert!(rel_paths.contains(&"src/python_model/L1_prototype/b.py".to_string()));
        assert!(rel_paths.contains(&"src/verilog_model/rtl/top.v".to_string()));
        assert!(rel_paths.contains(&"tests/test_l0.py".to_string()));

        assert!(!rel_paths.iter().any(|p| p.contains("__pycache__")), "不应包含 __pycache__ 文件");
        assert!(!rel_paths.iter().any(|p| p.contains(".claude")), "不应包含 .claude 文件");
        assert!(!rel_paths.iter().any(|p| p.contains("vivado")), "不应包含 vivado 文件");
    }

    #[test]
    fn noise_skip_preserves_tests_source_files() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "tests/test_stage.py");
        touch(root, "tests/__pycache__/test.cpython.pyc");
        touch(root, "tests/.pytest_cache/v/cache/lastfailed");

        let out = scan_workspace_files(root);
        let rel_paths: Vec<_> = out.files.iter().map(|f| f.rel_path.clone()).collect();
        assert!(rel_paths.contains(&"tests/test_stage.py".to_string()), "tests 下的源文件应保留");
        assert!(!rel_paths.iter().any(|p| p.contains("__pycache__")), "tests 下的 cache 应跳过");
        assert!(!rel_paths.iter().any(|p| p.contains(".pytest_cache")), "tests 下的 pytest_cache 应跳过");
    }

    #[test]
    fn deep_project_with_noise_no_scan_timeout() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // 创建深层有效源码
        touch(root, "src/python_model/L0_external/a.py");
        touch(root, "src/python_model/L1_prototype/b.py");
        touch(root, "src/python_model/L2_structured/c.py");
        touch(root, "src/verilog_model/rtl/top.v");
        // 创建大量噪声目录
        for i in 0..20 {
            touch(root, &format!("__pycache__/cache_{}.pyc", i));
            touch(root, &format!(".pytest_cache/v{}/nodeids", i));
            touch(root, &format!("vivado/run_{}.tcl", i));
            touch(root, &format!("reports/report_{}.html", i));
            touch(root, &format!("build/obj_{}.o", i));
        }

        let out = scan_workspace_files(root);
        // 应只收集 4 个有效文件
        assert_eq!(out.files.len(), 4, "应跳过所有噪声目录，只保留有效源码");
        // 不应产生大量 scan_timeout warnings
        let timeout_warnings = out.warnings.iter().filter(|w| w.error_code == ErrorCode::ScanTimeout).count();
        assert_eq!(timeout_warnings, 0, "噪声目录跳过不应产生 scan_timeout");
    }

    #[test]
    fn deep_ai_template_source_files_scanned_without_timeout() {
        // 回归：真实 ai_project_template 存在深度 5 的源码文件，
        // 旧逻辑在 depth > 3 时整目录跳过，导致深层子包源码漏扫并产生 scan_timeout。
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "src/python_model/L0_external/rx_02_coarse_sync/coarse_block.py");
        touch(root, "src/python_model/L0_external/shared_04_preamble/preamble.py");
        touch(root, "src/python_model/L0_external/top.py");
        touch(root, "src/verilog_model/rtl/top.v");

        let out = scan_workspace_files(root);
        let rel_paths: Vec<_> = out.files.iter().map(|f| f.rel_path.clone()).collect();
        assert!(
            rel_paths.contains(&"src/python_model/L0_external/rx_02_coarse_sync/coarse_block.py".to_string()),
            "深层子包源码 coarse_block.py 必须被扫描到"
        );
        assert!(
            rel_paths.contains(&"src/python_model/L0_external/shared_04_preamble/preamble.py".to_string()),
            "深层子包源码 preamble.py 必须被扫描到"
        );
        assert!(rel_paths.contains(&"src/python_model/L0_external/top.py".to_string()));
        assert!(rel_paths.contains(&"src/verilog_model/rtl/top.v".to_string()));

        // 深层有效源码扫描不得产生 scan_timeout（深度限制不应误伤 ai_project_template 源码）
        let timeout_warnings = out
            .warnings
            .iter()
            .filter(|w| w.error_code == ErrorCode::ScanTimeout)
            .count();
        assert_eq!(timeout_warnings, 0, "深层 ai_project_template 源码扫描不应产生 scan_timeout");
    }

    #[test]
    fn deep_source_tree_all_descendants_scanned() {
        // 阶段根之下的多层子孙目录都应被递归扫描（不限 depth 3）。
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "src/python_model/L0_external/rx_02_coarse_sync/filters/deep_filter.py");
        touch(root, "src/python_model/L0_external/shared_04_preamble/sub/inner/innermost.py");
        touch(root, "src/verilog_model/rtl/ip/cores/core_top.v");

        let out = scan_workspace_files(root);
        let rel_paths: Vec<_> = out.files.iter().map(|f| f.rel_path.clone()).collect();
        assert!(
            rel_paths.contains(&"src/python_model/L0_external/rx_02_coarse_sync/filters/deep_filter.py".to_string()),
            "阶段根下多层子目录的源码应被扫描"
        );
        assert!(
            rel_paths.contains(&"src/python_model/L0_external/shared_04_preamble/sub/inner/innermost.py".to_string()),
            "阶段根下嵌套子包源码应被扫描"
        );
        assert!(
            rel_paths.contains(&"src/verilog_model/rtl/ip/cores/core_top.v".to_string()),
            "rtl 阶段根下多层子目录源码应被扫描"
        );
    }

    #[test]
    fn noise_dirs_still_depth_limited_outside_deep_source() {
        // 深度限制在 ai_project_template 深层源码树之外仍然生效：
        // 非 src/python_model、src/verilog_model 路径下的深层目录被噪声跳过或深度跳过，
        // 不得无限制递归进 .git / .claude 等噪声目录。
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // .claude 深层文件不应被扫描（should_skip_dir 提前拦截）
        touch(root, ".claude/commands/a/b/c/d/deep_cmd.md");
        // __pycache__ 深层不应被扫描
        touch(root, "__pycache__/x/y/z/deep_cache.pyc");
        // 非深层源码树的普通深层目录：深度 > 3 应被跳过（不视为有效源码树）
        touch(root, "docs/nested/very/deep/file.md");

        let out = scan_workspace_files(root);
        let rel_paths: Vec<_> = out.files.iter().map(|f| f.rel_path.clone()).collect();
        assert!(
            !rel_paths.iter().any(|p| p.contains(".claude")),
            ".claude 噪声目录不得被扫描"
        );
        assert!(
            !rel_paths.iter().any(|p| p.contains("__pycache__")),
            "__pycache__ 噪声目录不得被扫描"
        );
    }
}
