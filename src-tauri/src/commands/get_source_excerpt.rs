/// `get_source_excerpt` Tauri command
///
/// 根据 SourceLocation 安全读取目标项目源码片段。SourceLocation 可由前端从
/// evidence item 的 source_path / line_range / evidence_id 组装。
/// 所有路径安全校验由 SourceExcerptResolver 完成。

use std::path::Path;

use crate::models::enums::ErrorCode;
use crate::models::error::{CommandError, CommandResult};
use crate::trace::models::{SourceExcerpt, SourceLocation};
use crate::trace::source_resolver::{SourceExcerptError, SourceExcerptResolver};

#[tauri::command]
pub fn get_source_excerpt(
    location: SourceLocation,
    root_path: String,
) -> CommandResult<SourceExcerpt> {
    let root = Path::new(&root_path);
    match SourceExcerptResolver::resolve_from_location(&location, root) {
        Ok(excerpt) => CommandResult {
            success: true,
            data: Some(excerpt),
            error: None,
            warnings: Vec::new(),
        },
        Err(err) => CommandResult {
            success: false,
            data: None,
            error: Some(map_source_error(err, &location.source_path)),
            warnings: Vec::new(),
        },
    }
}

fn map_source_error(err: SourceExcerptError, source_path: &str) -> CommandError {
    let recoverable = matches!(
        err.error_code,
        ErrorCode::SourceFileUnreadable | ErrorCode::LineRangeInvalid
    );
    CommandError {
        error_code: err.error_code,
        message: err.message,
        recoverable,
        details: None,
        source_path: Some(source_path.to_string()),
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
            line_range: crate::evidence::models::LineRange { start, end },
            symbol: None,
            summary: "test".to_string(),
            strength: EvidenceStrength::Direct,
        }
    }

    fn write_lines(dir: &std::path::Path, name: &str, count: usize) -> std::path::PathBuf {
        let path = dir.join(name);
        let content: String = (1..=count)
            .map(|i| format!("line {}\n", i))
            .collect();
        fs::write(&path, content).unwrap();
        path
    }

    // ─── command 测试 ────────────────────────────────────────────────

    #[test]
    fn cmd_excerpt_from_evidence_location_ok() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = write_lines(tmp.path(), "test.v", 10);
        let item = make_evidence_item(
            "EV-L0-000001",
            file_path.to_string_lossy().to_string(),
            1,
            3,
        );
        let _collection = make_evidence_collection(vec![item.clone()]);

        let location = SourceLocation {
            source_path: item.source_path,
            line_range: LineRange {
                start: item.line_range.start,
                end: item.line_range.end,
            },
            evidence_id: Some(item.evidence_id),
        };

        let result = get_source_excerpt(
            location,
            tmp.path().to_string_lossy().to_string(),
        );

        assert!(result.success);
        let excerpt = result.data.unwrap();
        assert_eq!(excerpt.lines.len(), 3);
        assert_eq!(excerpt.location.evidence_id, Some("EV-L0-000001".to_string()));
    }

    #[test]
    fn cmd_excerpt_location_ok() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = write_lines(tmp.path(), "test.v", 10);
        let location = SourceLocation {
            source_path: file_path.to_string_lossy().to_string(),
            line_range: LineRange { start: 2, end: 4 },
            evidence_id: None,
        };

        let result = get_source_excerpt(
            location,
            tmp.path().to_string_lossy().to_string(),
        );

        assert!(result.success);
        let excerpt = result.data.unwrap();
        assert_eq!(excerpt.lines.len(), 3);
        assert_eq!(excerpt.lines[0].content, "line 2");
    }

    #[test]
    fn cmd_excerpt_outside_root_not_allowed() {
        let tmp = tempfile::tempdir().unwrap();
        let location = SourceLocation {
            source_path: "/etc/passwd".to_string(),
            line_range: LineRange { start: 1, end: 1 },
            evidence_id: None,
        };

        let result = get_source_excerpt(
            location,
            tmp.path().to_string_lossy().to_string(),
        );

        assert!(!result.success);
        let err = result.error.unwrap();
        assert_eq!(err.error_code, ErrorCode::SourcePathNotAllowed);
        assert!(!err.recoverable);
    }

    #[test]
    fn cmd_excerpt_root_symlink_not_allowed() {
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

        let result = get_source_excerpt(
            location,
            link_root.to_string_lossy().to_string(),
        );

        assert!(!result.success);
        assert_eq!(result.error.unwrap().error_code, ErrorCode::SourcePathNotAllowed);
    }

    #[test]
    fn cmd_excerpt_parent_symlink_to_root_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("root");
        fs::create_dir(&root).unwrap();
        write_lines(&root, "real.v", 5);

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

        let result = get_source_excerpt(
            location,
            root.to_string_lossy().to_string(),
        );

        assert!(!result.success);
        assert_eq!(result.error.unwrap().error_code, ErrorCode::SourcePathNotAllowed);
    }

    #[test]
    fn cmd_excerpt_binary_file_unreadable() {
        let tmp = tempfile::tempdir().unwrap();
        let binary_path = tmp.path().join("binary.bin");
        fs::write(&binary_path, vec![0u8; 1024]).unwrap();

        let location = SourceLocation {
            source_path: binary_path.to_string_lossy().to_string(),
            line_range: LineRange { start: 1, end: 1 },
            evidence_id: None,
        };

        let result = get_source_excerpt(
            location,
            tmp.path().to_string_lossy().to_string(),
        );

        assert!(!result.success);
        assert_eq!(result.error.unwrap().error_code, ErrorCode::SourceFileUnreadable);
    }

    #[test]
    fn cmd_excerpt_non_utf8_file_unreadable() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("invalid.txt");
        fs::write(&path, vec![0xff, 0xfe, 0xfd]).unwrap();

        let location = SourceLocation {
            source_path: path.to_string_lossy().to_string(),
            line_range: LineRange { start: 1, end: 1 },
            evidence_id: None,
        };

        let result = get_source_excerpt(
            location,
            tmp.path().to_string_lossy().to_string(),
        );

        assert!(!result.success);
        assert_eq!(result.error.unwrap().error_code, ErrorCode::SourceFileUnreadable);
    }

    #[test]
    fn cmd_excerpt_too_large_file_unreadable() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("large.v");
        fs::write(&path, "x".repeat((5 * 1024 * 1024 + 1) as usize)).unwrap();

        let location = SourceLocation {
            source_path: path.to_string_lossy().to_string(),
            line_range: LineRange { start: 1, end: 1 },
            evidence_id: None,
        };

        let result = get_source_excerpt(
            location,
            tmp.path().to_string_lossy().to_string(),
        );

        assert!(!result.success);
        assert_eq!(result.error.unwrap().error_code, ErrorCode::SourceFileUnreadable);
    }

    #[test]
    fn cmd_excerpt_line_range_invalid() {
        let tmp = tempfile::tempdir().unwrap();
        let file_path = write_lines(tmp.path(), "test.v", 5);
        let location = SourceLocation {
            source_path: file_path.to_string_lossy().to_string(),
            line_range: LineRange { start: 1, end: 10 },
            evidence_id: None,
        };

        let result = get_source_excerpt(
            location,
            tmp.path().to_string_lossy().to_string(),
        );

        assert!(!result.success);
        assert_eq!(result.error.unwrap().error_code, ErrorCode::LineRangeInvalid);
    }
}
