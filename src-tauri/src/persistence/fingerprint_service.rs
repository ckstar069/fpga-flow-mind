use std::collections::HashSet;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::workspace::file_classifier::is_binary;

/// fingerprint 算法标识。
pub const FINGERPRINT_ALGORITHM: &str = "sha256:file-list:v1";

/// 大文件阈值：超过此大小的文件不参与 fingerprint（5MB）。
const LARGE_FILE_THRESHOLD: u64 = 5 * 1024 * 1024;

/// 参与 fingerprint 计算的文件扩展名集合。
const INCLUDED_EXTENSIONS: &[&str] = &["py", "v", "sv", "md", "json", "yaml", "yml", "toml"];

/// fingerprint 计算过程中遇到的错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FingerprintError {
    /// 目标路径不存在。
    SourceMissing,
    /// 目标路径为 symlink / 非目录 / 不可读 / 空路径。
    SourcePathNotAllowed { reason: String },
    /// IO 错误导致无法完成计算。
    IoError { message: String },
}

impl fmt::Display for FingerprintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FingerprintError::SourceMissing => write!(f, "目标项目路径不存在"),
            FingerprintError::SourcePathNotAllowed { reason } => {
                write!(f, "目标路径不允许访问: {}", reason)
            }
            FingerprintError::IoError { message } => write!(f, "读取文件失败: {}", message),
        }
    }
}

impl std::error::Error for FingerprintError {}

/// fingerprint 比较结果，直接对应 `LoadSessionStatus` 的业务语义。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FingerprintComparison {
    Unchanged,
    Changed { reason: String },
    Missing,
    NotAllowed { reason: String },
}

/// 目标项目 fingerprint 计算服务。
pub struct WorkspaceFingerprintService;

impl WorkspaceFingerprintService {
    /// 计算目标项目的 workspace fingerprint。
    ///
    /// 算法：
    /// 1. 收集扩展名在 {py, v, sv, md, json, yaml, yml, toml} 范围内的普通文件。
    /// 2. 跳过 symlink、二进制文件、超大文件（>5MB）、不可读文件。
    /// 3. 按相对路径排序。
    /// 4. 对每个文件计算 SHA-256。
    /// 5. 将 `相对路径:文件哈希\n` 拼接为字符串。
    /// 6. 对拼接结果再计算一次 SHA-256，作为最终 fingerprint。
    pub fn compute_fingerprint(root: &Path) -> Result<String, FingerprintError> {
        let root = Self::validate_root(root)?;
        let files = Self::collect_files(&root)?;
        let manifest = Self::build_manifest(&root, &files)?;
        Ok(Self::hash_text(&manifest))
    }

    /// 重新计算目标项目 fingerprint 并与记录值比较。
    pub fn compare(root: &Path, recorded: &str) -> FingerprintComparison {
        match Self::compute_fingerprint(root) {
            Ok(current) => {
                if current == recorded {
                    FingerprintComparison::Unchanged
                } else {
                    FingerprintComparison::Changed {
                        reason: "目标项目关键文件已变更".to_string(),
                    }
                }
            }
            Err(FingerprintError::SourceMissing) => FingerprintComparison::Missing,
            Err(FingerprintError::SourcePathNotAllowed { reason }) => {
                FingerprintComparison::NotAllowed { reason }
            }
            Err(FingerprintError::IoError { message }) => {
                FingerprintComparison::Changed { reason: message }
            }
        }
    }

    /// 返回当前 fingerprint 算法标识。
    pub fn algorithm() -> &'static str {
        FINGERPRINT_ALGORITHM
    }

    fn validate_root(path: &Path) -> Result<PathBuf, FingerprintError> {
        if path.as_os_str().is_empty() {
            return Err(FingerprintError::SourcePathNotAllowed {
                reason: "路径为空".to_string(),
            });
        }

        let metadata = match std::fs::symlink_metadata(path) {
            Ok(m) => m,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Err(FingerprintError::SourceMissing);
            }
            Err(e) => {
                return Err(FingerprintError::SourcePathNotAllowed {
                    reason: format!("无法访问路径: {}", e),
                });
            }
        };

        if metadata.file_type().is_symlink() {
            return Err(FingerprintError::SourcePathNotAllowed {
                reason: "根路径不能是符号链接".to_string(),
            });
        }

        if !metadata.is_dir() {
            return Err(FingerprintError::SourcePathNotAllowed {
                reason: "路径不是目录".to_string(),
            });
        }

        if std::fs::read_dir(path).is_err() {
            return Err(FingerprintError::SourcePathNotAllowed {
                reason: "目录不可读".to_string(),
            });
        }

        path.canonicalize().map_err(|e| FingerprintError::SourcePathNotAllowed {
            reason: format!("无法规范化路径: {}", e),
        })
    }

    fn collect_files(root: &Path) -> Result<Vec<PathBuf>, FingerprintError> {
        let mut files = Vec::new();
        let included: HashSet<String> =
            INCLUDED_EXTENSIONS.iter().map(|e| e.to_string()).collect();
        Self::walk(root, root, &included, &mut files)?;
        files.sort_by(|a, b| {
            let ra = Self::relative_path(root, a);
            let rb = Self::relative_path(root, b);
            ra.cmp(&rb)
        });
        Ok(files)
    }

    fn walk(
        root: &Path,
        dir: &Path,
        included: &HashSet<String>,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), FingerprintError> {
        let entries = std::fs::read_dir(dir).map_err(|e| FingerprintError::IoError {
            message: format!("无法读取目录 {}: {}", dir.display(), e),
        })?;

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let file_type = match entry.file_type() {
                Ok(t) => t,
                Err(_) => continue,
            };

            if file_type.is_symlink() {
                continue;
            }

            let path = entry.path();

            if file_type.is_dir() {
                Self::walk(root, &path, included, files)?;
                continue;
            }

            if !file_type.is_file() {
                continue;
            }

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            if metadata.len() > LARGE_FILE_THRESHOLD {
                continue;
            }

            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if !included.contains(ext.to_lowercase().as_str()) {
                    continue;
                }
            } else {
                continue;
            }

            if is_binary(&path) {
                continue;
            }

            files.push(path);
        }

        Ok(())
    }

    fn build_manifest(root: &Path, files: &[PathBuf]) -> Result<String, FingerprintError> {
        let mut lines: Vec<String> = Vec::with_capacity(files.len());
        for path in files {
            let rel = Self::relative_path(root, path);
            let content = match std::fs::read(path) {
                Ok(c) => c,
                Err(e) => {
                    return Err(FingerprintError::IoError {
                        message: format!("无法读取 {}: {}", path.display(), e),
                    });
                }
            };
            let file_hash = Self::hash_bytes(&content);
            lines.push(format!("{}:{}", rel, file_hash));
        }
        Ok(lines.join("\n"))
    }

    fn relative_path(root: &Path, path: &Path) -> String {
        path.strip_prefix(root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"))
    }

    fn hash_bytes(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        Self::hex(&hasher.finalize())
    }

    fn hash_text(text: &str) -> String {
        Self::hash_bytes(text.as_bytes())
    }

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn empty_directory_yields_stable_fingerprint() {
        let tmp = tempfile::tempdir().unwrap();
        let fp1 = WorkspaceFingerprintService::compute_fingerprint(tmp.path()).unwrap();
        let fp2 = WorkspaceFingerprintService::compute_fingerprint(tmp.path()).unwrap();
        assert_eq!(fp1, fp2);
        assert_eq!(fp1.len(), 64);
        assert_eq!(
            fp1,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn fingerprint_ignores_unrelated_extensions() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("readme.txt"), "hello").unwrap();
        fs::write(tmp.path().join("data.bin"), vec![0u8; 1024]).unwrap();
        let fp = WorkspaceFingerprintService::compute_fingerprint(tmp.path()).unwrap();
        assert_eq!(
            fp,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn included_files_change_fingerprint() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("top.py"), "def add(): pass").unwrap();
        let fp1 = WorkspaceFingerprintService::compute_fingerprint(tmp.path()).unwrap();

        fs::write(tmp.path().join("top.py"), "def add(): return 1").unwrap();
        let fp2 = WorkspaceFingerprintService::compute_fingerprint(tmp.path()).unwrap();
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn adding_included_file_changes_fingerprint() {
        let tmp = tempfile::tempdir().unwrap();
        let fp1 = WorkspaceFingerprintService::compute_fingerprint(tmp.path()).unwrap();
        fs::write(tmp.path().join("new.md"), "# doc").unwrap();
        let fp2 = WorkspaceFingerprintService::compute_fingerprint(tmp.path()).unwrap();
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn order_independent_for_multiple_files() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("a.py"), "a").unwrap();
        fs::write(tmp.path().join("b.py"), "b").unwrap();
        let fp1 = WorkspaceFingerprintService::compute_fingerprint(tmp.path()).unwrap();

        // 重新创建同名文件，顺序不影响结果（因为已排序）。
        let tmp2 = tempfile::tempdir().unwrap();
        fs::write(tmp2.path().join("b.py"), "b").unwrap();
        fs::write(tmp2.path().join("a.py"), "a").unwrap();
        let fp2 = WorkspaceFingerprintService::compute_fingerprint(tmp2.path()).unwrap();
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn binary_file_is_skipped() {
        let tmp = tempfile::tempdir().unwrap();
        let mut binary = vec![0u8; 256];
        binary[0] = 0x89;
        binary[1] = 0x50;
        fs::write(tmp.path().join("image.py"), &binary).unwrap();
        let fp = WorkspaceFingerprintService::compute_fingerprint(tmp.path()).unwrap();
        assert_eq!(
            fp,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn large_file_is_skipped() {
        let tmp = tempfile::tempdir().unwrap();
        let big = vec![b' '; (LARGE_FILE_THRESHOLD + 1) as usize];
        fs::write(tmp.path().join("big.py"), big).unwrap();
        let fp = WorkspaceFingerprintService::compute_fingerprint(tmp.path()).unwrap();
        assert_eq!(
            fp,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn missing_path_returns_source_missing() {
        let path = Path::new("/this/path/does/not/exist/12345");
        let result = WorkspaceFingerprintService::compute_fingerprint(path);
        assert_eq!(result.unwrap_err(), FingerprintError::SourceMissing);
    }

    #[test]
    fn symlink_root_returns_not_allowed() {
        let tmp = tempfile::tempdir().unwrap();
        let real = tmp.path().join("real");
        fs::create_dir(&real).unwrap();
        let link = tmp.path().join("link");
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
        let result = WorkspaceFingerprintService::compute_fingerprint(&link);
        match result.unwrap_err() {
            FingerprintError::SourcePathNotAllowed { .. } => {}
            other => panic!("应为 SourcePathNotAllowed，得到 {:?}", other),
        }
    }

    #[test]
    fn file_as_root_returns_not_allowed() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("file.txt");
        fs::write(&file, "hello").unwrap();
        let result = WorkspaceFingerprintService::compute_fingerprint(&file);
        match result.unwrap_err() {
            FingerprintError::SourcePathNotAllowed { .. } => {}
            other => panic!("应为 SourcePathNotAllowed，得到 {:?}", other),
        }
    }

    #[test]
    fn compare_detects_unchanged() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("top.py"), "x").unwrap();
        let fp = WorkspaceFingerprintService::compute_fingerprint(tmp.path()).unwrap();
        let result = WorkspaceFingerprintService::compare(tmp.path(), &fp);
        assert_eq!(result, FingerprintComparison::Unchanged);
    }

    #[test]
    fn compare_detects_changed() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("top.py"), "x").unwrap();
        let result = WorkspaceFingerprintService::compare(tmp.path(), "old-hash");
        match result {
            FingerprintComparison::Changed { .. } => {}
            other => panic!("应为 Changed，得到 {:?}", other),
        }
    }

    #[test]
    fn compare_detects_missing() {
        let path = Path::new("/this/path/does/not/exist/12345");
        let result = WorkspaceFingerprintService::compare(path, "any");
        assert_eq!(result, FingerprintComparison::Missing);
    }

    #[test]
    fn algorithm_label_is_correct() {
        assert_eq!(WorkspaceFingerprintService::algorithm(), "sha256:file-list:v1");
    }
}
