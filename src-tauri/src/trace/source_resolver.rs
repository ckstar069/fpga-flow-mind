use std::path::{Path, PathBuf};

use crate::evidence::models::EvidenceCollection;
use crate::models::enums::ErrorCode;
use crate::trace::models::{ExcerptWarning, SourceExcerpt, SourceLine, SourceLocation};
use crate::workspace::file_classifier::{classify_file, is_binary};

/// 单个文件大小上限：5 MB
const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024;
/// 单次 excerpt 最大行数
const MAX_EXCERPT_LINES: usize = 100;
/// 单次 excerpt 最大字符数
const MAX_EXCERPT_CHARS: usize = 8192;

/// SourceExcerptResolver 内部错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceExcerptError {
    pub error_code: ErrorCode,
    pub message: String,
}

impl SourceExcerptError {
    fn root_not_allowed(message: impl Into<String>) -> Self {
        Self {
            error_code: ErrorCode::SourcePathNotAllowed,
            message: message.into(),
        }
    }

    fn source_not_allowed(message: impl Into<String>) -> Self {
        Self {
            error_code: ErrorCode::SourcePathNotAllowed,
            message: message.into(),
        }
    }

    fn file_unreadable(message: impl Into<String>) -> Self {
        Self {
            error_code: ErrorCode::SourceFileUnreadable,
            message: message.into(),
        }
    }

    fn line_range_invalid(message: impl Into<String>) -> Self {
        Self {
            error_code: ErrorCode::LineRangeInvalid,
            message: message.into(),
        }
    }
}

/// 安全读取目标项目源码片段
pub struct SourceExcerptResolver;

impl SourceExcerptResolver {
    /// 从 evidence_id 解析：使用 evidence item 的 source_path / line_range / language
    pub fn resolve_from_evidence(
        evidence_id: &str,
        evidence_collection: &EvidenceCollection,
        root_path: &Path,
    ) -> Result<SourceExcerpt, SourceExcerptError> {
        let item = evidence_collection
            .evidence_items
            .iter()
            .find(|e| e.evidence_id == evidence_id)
            .ok_or_else(|| SourceExcerptError::source_not_allowed(format!("evidence_id {} not found", evidence_id)))?;

        let location = SourceLocation {
            source_path: item.source_path.clone(),
            line_range: item.line_range,
            evidence_id: Some(evidence_id.to_string()),
        };

        let mut excerpt = Self::resolve_from_location(&location, root_path)?;
        excerpt.language = item.language;
        Ok(excerpt)
    }

    /// 从 SourceLocation 解析：source_path 必须已属于 root_path
    pub fn resolve_from_location(
        location: &SourceLocation,
        root_path: &Path,
    ) -> Result<SourceExcerpt, SourceExcerptError> {
        let canonical_root = Self::validate_root_path(root_path)?;
        let source_path = Path::new(&location.source_path);
        let canonical_source = Self::validate_source_path(source_path, &canonical_root)?;

        let language = classify_file(&canonical_source).0;

        // 二进制检查
        if is_binary(&canonical_source) {
            return Err(SourceExcerptError::file_unreadable("binary file skipped"));
        }

        // 读取并验证 UTF-8
        let content = std::fs::read(&canonical_source).map_err(|e| {
            SourceExcerptError::file_unreadable(format!("failed to read file: {}", e))
        })?;

        // 大小二次检查（read 后）
        if content.len() as u64 > MAX_FILE_SIZE {
            return Err(SourceExcerptError::file_unreadable("file too large"));
        }

        let text = String::from_utf8(content).map_err(|_| {
            SourceExcerptError::file_unreadable("non-utf8 file skipped")
        })?;

        let all_lines: Vec<&str> = text.lines().collect();
        let total_lines = all_lines.len();

        // line_range 校验
        if location.line_range.start < 1 {
            return Err(SourceExcerptError::line_range_invalid("start line must be >= 1"));
        }
        if location.line_range.start > location.line_range.end {
            return Err(SourceExcerptError::line_range_invalid("start line must be <= end line"));
        }
        let end_usize = location.line_range.end as usize;
        let start_usize = location.line_range.start as usize;
        if end_usize > total_lines {
            return Err(SourceExcerptError::line_range_invalid(format!(
                "end line {} exceeds total lines {}",
                location.line_range.end, total_lines
            )));
        }

        // 提取请求行
        let requested: Vec<SourceLine> = all_lines[start_usize - 1..end_usize]
            .iter()
            .enumerate()
            .map(|(idx, &line)| SourceLine {
                line_number: location.line_range.start + idx as u32,
                content: line.to_string(),
            })
            .collect();

        // 截断逻辑
        let (lines, is_truncated, truncation_reason) = Self::apply_limits(requested);

        let mut warnings = Vec::new();
        if is_truncated {
            warnings.push(ExcerptWarning {
                error_code: "source_excerpt_truncated".to_string(),
                message: truncation_reason.clone().unwrap_or_default(),
            });
        }

        Ok(SourceExcerpt {
            location: location.clone(),
            language,
            lines,
            is_truncated,
            truncation_reason,
            warnings,
        })
    }

    /// root_path 安全校验（与 Phase 1 safety_guard 等价）
    fn validate_root_path(root_path: &Path) -> Result<PathBuf, SourceExcerptError> {
        if root_path.as_os_str().is_empty() {
            return Err(SourceExcerptError::root_not_allowed("root path is empty"));
        }

        let metadata = std::fs::symlink_metadata(root_path).map_err(|e| {
            SourceExcerptError::root_not_allowed(format!("failed to stat root path: {}", e))
        })?;

        if metadata.file_type().is_symlink() {
            return Err(SourceExcerptError::root_not_allowed("root path must not be a symlink"));
        }

        if !metadata.is_dir() {
            return Err(SourceExcerptError::root_not_allowed("root path must be a directory"));
        }

        if std::fs::read_dir(root_path).is_err() {
            return Err(SourceExcerptError::root_not_allowed("root path is not readable"));
        }

        root_path.canonicalize().map_err(|e| {
            SourceExcerptError::root_not_allowed(format!("failed to canonicalize root path: {}", e))
        })
    }

    /// source_path 安全校验
    fn validate_source_path(
        source_path: &Path,
        canonical_root: &Path,
    ) -> Result<PathBuf, SourceExcerptError> {
        if source_path.as_os_str().is_empty() {
            return Err(SourceExcerptError::source_not_allowed("source path is empty"));
        }

        if !source_path.is_absolute() {
            return Err(SourceExcerptError::source_not_allowed("source path must be absolute"));
        }

        // source_path 本身不能是 symlink
        let source_meta = std::fs::symlink_metadata(source_path).map_err(|e| {
            SourceExcerptError::source_not_allowed(format!("failed to stat source path: {}", e))
        })?;
        if source_meta.file_type().is_symlink() {
            return Err(SourceExcerptError::source_not_allowed(
                "source path must not be a symlink",
            ));
        }

        // 检查每一级父路径直到 canonical_root（不含），不允许 symlink。
        // 顺序：先检查当前 parent 是否为 symlink，再 canonicalize 比较，避免 symlink 指向 root 的绕过。
        let mut current = source_path.parent();
        while let Some(parent) = current {
            let meta = std::fs::symlink_metadata(parent).map_err(|e| {
                SourceExcerptError::source_not_allowed(format!("failed to stat parent path: {}", e))
            })?;
            if meta.file_type().is_symlink() {
                return Err(SourceExcerptError::source_not_allowed(format!(
                    "parent path {} is a symlink",
                    parent.display()
                )));
            }

            // canonicalize 后比较，兼容 macOS /var -> /private/var 等系统路径差异
            let canonical_parent = parent.canonicalize().map_err(|e| {
                SourceExcerptError::source_not_allowed(format!(
                    "failed to canonicalize parent path: {}",
                    e
                ))
            })?;
            if canonical_parent == *canonical_root {
                break;
            }

            current = parent.parent();
        }

        // canonicalize source_path
        let canonical_source = source_path.canonicalize().map_err(|e| {
            SourceExcerptError::source_not_allowed(format!("failed to canonicalize source path: {}", e))
        })?;

        // 使用 canonical 路径的组件前缀判断
        if !canonical_source.starts_with(canonical_root) {
            return Err(SourceExcerptError::source_not_allowed(
                "source path is outside workspace root",
            ));
        }

        // 必须是普通文件
        let canonical_meta = std::fs::symlink_metadata(&canonical_source,
        ).map_err(|e| SourceExcerptError::file_unreadable(format!("failed to stat canonical source: {}", e)))?;
        if !canonical_meta.is_file() {
            return Err(SourceExcerptError::file_unreadable("source path is not a regular file"));
        }

        // 大小检查
        if canonical_meta.len() > MAX_FILE_SIZE {
            return Err(SourceExcerptError::file_unreadable("file too large (> 5MB)"));
        }

        Ok(canonical_source)
    }

    /// 应用行数/字符数截断
    fn apply_limits(
        requested: Vec<SourceLine>,
    ) -> (Vec<SourceLine>, bool, Option<String>) {
        let mut lines = Vec::new();
        let mut char_count = 0usize;
        let mut truncated = false;
        let mut reason = None;

        for (idx, line) in requested.into_iter().enumerate() {
            if idx >= MAX_EXCERPT_LINES {
                truncated = true;
                reason = Some(format!(
                    "已截断，仅展示前 {} 行（共请求 {} 行）",
                    MAX_EXCERPT_LINES,
                    idx
                ));
                break;
            }

            let line_chars = line.content.chars().count();
            if char_count + line_chars > MAX_EXCERPT_CHARS {
                // 尝试截断当前行以塞满剩余额度
                let remaining = MAX_EXCERPT_CHARS.saturating_sub(char_count);
                if remaining > 0 {
                    let mut truncated_content = String::new();
                    let mut used = 0usize;
                    for ch in line.content.chars() {
                        if used >= remaining {
                            break;
                        }
                        truncated_content.push(ch);
                        used += 1;
                    }
                    lines.push(SourceLine {
                        line_number: line.line_number,
                        content: truncated_content,
                    });
                }
                truncated = true;
                reason = Some(format!(
                    "已截断，超过 {} 字符上限",
                    MAX_EXCERPT_CHARS
                ));
                break;
            }

            char_count += line_chars;
            lines.push(line);
        }

        (lines, truncated, reason)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;

    use super::*;
    use crate::evidence::models::{
        EvidenceCollection, EvidenceItem, EvidenceStats, EvidenceStrength, LineRange,
    };
    use crate::models::enums::{Language, SourceKind};

    fn make_evidence_collection(items: Vec<EvidenceItem>) -> EvidenceCollection {
        EvidenceCollection {
            stage_id: "L0".to_string(),
            evidence_items: items,
            index_by_path: HashMap::new(),
            index_by_kind: HashMap::new(),
            index_by_symbol: HashMap::new(),
            warnings: vec![],
            stats: EvidenceStats {
                files_processed: 0,
                files_skipped: 0,
                total_items: 0,
                items_by_kind: HashMap::new(),
                items_by_strength: HashMap::new(),
            },
            version: "1.0.0".to_string(),
        }
    }

    fn make_evidence_item(id: &str, path: String, start: u32, end: u32) -> EvidenceItem {
        EvidenceItem {
            evidence_id: id.to_string(),
            source_path: path,
            language: Language::Verilog,
            source_kind: SourceKind::Rtl,
            line_range: LineRange { start, end },
            symbol: None,
            summary: "test".to_string(),
            strength: EvidenceStrength::Direct,
        }
    }

    fn write_lines(dir: &Path, name: &str, count: usize) -> PathBuf {
        let path = dir.join(name);
        let content: String = (1..=count)
            .map(|i| format!("line {}\n", i))
            .collect();
        fs::write(&path, content).unwrap();
        path
    }

    // ─── source resolver 测试 ─────────────────────────────────────────

    #[test]
    fn sr_01_valid_evidence_id_resolves() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = write_lines(tmp.path(), "test.v", 20);
        let item = make_evidence_item("EV-L0-000001", file_path.to_string_lossy().to_string(), 1, 5);
        let collection = make_evidence_collection(vec![item]);

        let excerpt = SourceExcerptResolver::resolve_from_evidence(
            "EV-L0-000001",
            &collection,
            tmp.path(),
        )
        .unwrap();

        assert_eq!(excerpt.lines.len(), 5);
        assert_eq!(excerpt.lines[0].line_number, 1);
        assert_eq!(excerpt.lines[0].content, "line 1");
        assert!(!excerpt.is_truncated);
    }

    #[test]
    fn sr_02_direct_source_location_resolves() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = write_lines(tmp.path(), "test.v", 20);
        let location = SourceLocation {
            source_path: file_path.to_string_lossy().to_string(),
            line_range: LineRange { start: 3, end: 6 },
            evidence_id: None,
        };

        let excerpt = SourceExcerptResolver::resolve_from_location(&location, tmp.path()).unwrap();

        assert_eq!(excerpt.lines.len(), 4);
        assert_eq!(excerpt.lines[0].line_number, 3);
        assert_eq!(excerpt.lines[0].content, "line 3");
    }

    #[test]
    fn sr_03_source_path_outside_root() {
        let tmp = tempfile::tempdir().unwrap();
        let location = SourceLocation {
            source_path: "/etc/passwd".to_string(),
            line_range: LineRange { start: 1, end: 1 },
            evidence_id: None,
        };

        let result = SourceExcerptResolver::resolve_from_location(&location, tmp.path());

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code, ErrorCode::SourcePathNotAllowed);
    }

    #[test]
    fn sr_04_source_path_is_symlink() {
        let tmp = tempfile::tempdir().unwrap();
        let real_file = write_lines(tmp.path(), "real.v", 5);
        let link_path = tmp.path().join("link.v");

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(&real_file, &link_path).unwrap();
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_file;
            symlink_file(&real_file, &link_path).unwrap();
        }

        let location = SourceLocation {
            source_path: link_path.to_string_lossy().to_string(),
            line_range: LineRange { start: 1, end: 1 },
            evidence_id: None,
        };

        let result = SourceExcerptResolver::resolve_from_location(&location, tmp.path());

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code, ErrorCode::SourcePathNotAllowed);
    }

    #[test]
    fn sr_05_root_path_is_symlink() {
        let tmp = tempfile::tempdir().unwrap();
        let real_root = tmp.path().join("real_root");
        fs::create_dir(&real_root).unwrap();
        let link_root = tmp.path().join("link_root");

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(&real_root, &link_root).unwrap();
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_dir;
            symlink_dir(&real_root, &link_root).unwrap();
        }

        let location = SourceLocation {
            source_path: real_root.join("test.v").to_string_lossy().to_string(),
            line_range: LineRange { start: 1, end: 1 },
            evidence_id: None,
        };

        let result = SourceExcerptResolver::resolve_from_location(&location, &link_root);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code, ErrorCode::SourcePathNotAllowed);
    }

    #[test]
    fn sr_06_parent_dir_is_symlink() {
        let tmp = tempfile::tempdir().unwrap();
        let real_dir = tmp.path().join("real_dir");
        fs::create_dir(&real_dir).unwrap();
        let link_dir = tmp.path().join("link_dir");

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(&real_dir, &link_dir).unwrap();
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_dir;
            symlink_dir(&real_dir, &link_dir).unwrap();
        }

        let _file_in_real = write_lines(&real_dir, "test.v", 5);
        let via_link = link_dir.join("test.v");

        let location = SourceLocation {
            source_path: via_link.to_string_lossy().to_string(),
            line_range: LineRange { start: 1, end: 1 },
            evidence_id: None,
        };

        let result = SourceExcerptResolver::resolve_from_location(&location, tmp.path());

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code, ErrorCode::SourcePathNotAllowed);
    }

    #[test]
    fn sr_07_string_prefix_trick() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("root");
        fs::create_dir(&root).unwrap();
        let root2 = tmp.path().join("root2");
        fs::create_dir(&root2).unwrap();

        let evil_file = write_lines(&root2, "evil.v", 5);
        let location = SourceLocation {
            source_path: evil_file.to_string_lossy().to_string(),
            line_range: LineRange { start: 1, end: 1 },
            evidence_id: None,
        };

        let result = SourceExcerptResolver::resolve_from_location(&location, &root);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code, ErrorCode::SourcePathNotAllowed);
    }

    #[test]
    fn sr_08_canonical_escape_with_dotdot() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("root");
        fs::create_dir(&root).unwrap();
        let sibling = tmp.path().join("sibling");
        fs::create_dir(&sibling).unwrap();
        write_lines(&sibling, "evil.v", 5);

        let location = SourceLocation {
            source_path: root.join("../sibling/evil.v").to_string_lossy().to_string(),
            line_range: LineRange { start: 1, end: 1 },
            evidence_id: None,
        };

        let result = SourceExcerptResolver::resolve_from_location(&location, &root);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code, ErrorCode::SourcePathNotAllowed);
    }

    #[test]
    fn sr_09_binary_file() {
        let tmp = tempfile::tempdir().unwrap();
        let binary_path = tmp.path().join("binary.bin");
        fs::write(&binary_path, vec![0u8; 1024]).unwrap();

        let location = SourceLocation {
            source_path: binary_path.to_string_lossy().to_string(),
            line_range: LineRange { start: 1, end: 1 },
            evidence_id: None,
        };

        let result = SourceExcerptResolver::resolve_from_location(&location, tmp.path());

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code, ErrorCode::SourceFileUnreadable);
    }

    #[test]
    fn sr_10_non_utf8_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("invalid.txt");
        fs::write(&path, vec![0xff, 0xfe, 0xfd]).unwrap();

        let location = SourceLocation {
            source_path: path.to_string_lossy().to_string(),
            line_range: LineRange { start: 1, end: 1 },
            evidence_id: None,
        };

        let result = SourceExcerptResolver::resolve_from_location(&location, tmp.path());

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code, ErrorCode::SourceFileUnreadable);
    }

    #[test]
    fn sr_11_too_large_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("large.v");
        fs::write(&path, "x".repeat((MAX_FILE_SIZE + 1) as usize)).unwrap();

        let location = SourceLocation {
            source_path: path.to_string_lossy().to_string(),
            line_range: LineRange { start: 1, end: 1 },
            evidence_id: None,
        };

        let result = SourceExcerptResolver::resolve_from_location(&location, tmp.path());

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code, ErrorCode::SourceFileUnreadable);
    }

    #[test]
    fn sr_12_line_range_out_of_bounds() {
        let tmp = tempfile::tempdir().unwrap();
        let path = write_lines(tmp.path(), "test.v", 5);
        let location = SourceLocation {
            source_path: path.to_string_lossy().to_string(),
            line_range: LineRange { start: 1, end: 10 },
            evidence_id: None,
        };

        let result = SourceExcerptResolver::resolve_from_location(&location, tmp.path());

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code, ErrorCode::LineRangeInvalid);
    }

    #[test]
    fn sr_13_truncation_by_line_count() {
        let tmp = tempfile::tempdir().unwrap();
        let path = write_lines(tmp.path(), "test.v", 150);
        let location = SourceLocation {
            source_path: path.to_string_lossy().to_string(),
            line_range: LineRange { start: 1, end: 150 },
            evidence_id: None,
        };

        let excerpt = SourceExcerptResolver::resolve_from_location(&location, tmp.path()
        ).unwrap();

        assert_eq!(excerpt.lines.len(), MAX_EXCERPT_LINES);
        assert!(excerpt.is_truncated);
        assert!(excerpt.truncation_reason.as_ref().unwrap().contains("100"));
    }

    #[test]
    fn sr_14_truncation_by_char_count() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("test.v");
        // 每行 100 个 "中"，10 行共 1000 字符，仍不够；需要约 82 行超过 8192
        let content: String = (1..=100)
            .map(|i| format!("{:04} {}\n", i, "中".repeat(100)))
            .collect();
        fs::write(&path, content).unwrap();

        let location = SourceLocation {
            source_path: path.to_string_lossy().to_string(),
            line_range: LineRange { start: 1, end: 100 },
            evidence_id: None,
        };

        let excerpt = SourceExcerptResolver::resolve_from_location(&location, tmp.path()
        ).unwrap();

        assert!(excerpt.is_truncated);
        assert!(excerpt
            .truncation_reason
            .as_ref()
            .unwrap()
            .contains("8192"));
    }

    #[test]
    fn sr_15_parent_symlink_to_root_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("root");
        fs::create_dir(&root).unwrap();

        // root 内创建真实文件
        write_lines(&root, "real.v", 5);

        // root 内创建指向 root 自身的 symlink
        let link_to_root = root.join("alias_to_root");
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(&root, &link_to_root).unwrap();
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::symlink_dir;
            symlink_dir(&root, &link_to_root).unwrap();
        }

        let location = SourceLocation {
            source_path: link_to_root.join("real.v").to_string_lossy().to_string(),
            line_range: LineRange { start: 1, end: 1 },
            evidence_id: None,
        };

        let result = SourceExcerptResolver::resolve_from_location(&location, &root);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code, ErrorCode::SourcePathNotAllowed);
    }
}
