/// Evidence extractors — 按语言类型的内容分析器
///
/// 每个提取器实现 `EvidenceExtractor` trait，接收文件内容，返回 `Vec<RawExtraction>`。
/// ID 分配和 summary 截断在 collector 层统一处理。
///
/// Trait 设计选择：`extract(&self, content: &str)` 只接收内容字符串，
/// 不接收 source_path/language/source_kind。
/// 理由：提取器是纯内容分析器。dispatch 路由由 `extract_by_language` 处理，
/// EvidenceItem 的完整元数据由 collector 层填充。
/// 这使得提取器无状态、纯函数、易于用纯字符串测试。

use crate::models::enums::{Language, SourceKind};
use super::excerpt::count_lines;
use super::models::{EvidenceStrength, LineRange, RawExtraction};

mod python;
mod verilog;
mod systemverilog;
mod markdown;
mod config;

pub use python::PythonExtractor;
pub use verilog::VerilogExtractor;
pub use systemverilog::SystemVerilogExtractor;
pub use markdown::MarkdownExtractor;
pub use config::ConfigExtractor;

/// Evidence extractor trait
///
/// 所有提取器均为无状态 unit struct。
/// `extract` 接收文件全文，返回原始提取结果。
pub trait EvidenceExtractor {
    fn extract(&self, content: &str) -> Vec<RawExtraction>;
}

/// 按语言和来源类型分派到对应提取器
///
/// 优先级：SourceKind::Config → ConfigExtractor（TCL/XDC 文件语言为 Unknown）。
/// 然后按 Language 分派到专用提取器。
/// 其余类型（Text/Json/Yaml/Toml/Unknown）→ FallbackExtractor（整文件级 indirect）。
pub fn extract_by_language(
    language: Language,
    source_kind: SourceKind,
    content: &str,
) -> Vec<RawExtraction> {
    // Config source_kind 优先（TCL/XDC 文件被 Phase 1 分类为 SourceKind::Config / Language::Unknown）
    if source_kind == SourceKind::Config {
        return ConfigExtractor.extract(content);
    }

    match language {
        Language::Python => PythonExtractor.extract(content),
        Language::Verilog => VerilogExtractor.extract(content),
        Language::SystemVerilog => SystemVerilogExtractor.extract(content),
        Language::Markdown => MarkdownExtractor.extract(content),
        // Text / Json / Yaml / Toml / Unknown → 整文件级 indirect
        _ => FallbackExtractor.extract(content),
    }
}

/// Fallback 提取器（未专门支持的语言）
///
/// 返回单条整文件级提取，strength=Indirect。
/// 空内容返回空 vec。
struct FallbackExtractor;

impl EvidenceExtractor for FallbackExtractor {
    fn extract(&self, content: &str) -> Vec<RawExtraction> {
        if content.is_empty() {
            return vec![];
        }
        let total = count_lines(content);
        if total == 0 {
            return vec![];
        }
        vec![RawExtraction {
            symbol: None,
            line_range: LineRange {
                start: 1,
                end: total as u32,
            },
            raw_excerpt: content.to_string(),
            strength: EvidenceStrength::Indirect,
        }]
    }
}

/// 提取 content 中 start..=end 行的文本（1-based 闭区间）
pub(super) fn extract_lines_range(content: &str, start: u32, end: u32) -> String {
    content
        .lines()
        .skip((start - 1) as usize)
        .take((end - start + 1) as usize)
        .collect::<Vec<_>>()
        .join("\n")
}

/// 检查字符串是否为合法标识符（字母/下划线开头，含字母/数字/下划线）
pub(super) fn is_valid_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_')
        && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// 计算 line 的前导空白字符数（空格和 tab 各算 1）
pub(super) fn indent_level(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ' || *c == '\t').count()
}

/// 检查行是否为 Verilog/SystemVerilog 注释行（// 或 /* 开头）
pub(super) fn is_hdl_comment_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("//") || trimmed.starts_with("/*")
}

/// 去除行内 // 注释后的内容
pub(super) fn strip_line_comment(line: &str) -> &str {
    match line.find("//") {
        Some(pos) => &line[..pos],
        None => line,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Fallback 提取器测试 ─────────────────────────────────────

    #[test]
    fn dispatch_01_fallback_unknown_language() {
        let content = "some text\nline 2\nline 3";
        let results = extract_by_language(Language::Unknown, SourceKind::Doc, content);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].strength, EvidenceStrength::Indirect);
        assert!(results[0].symbol.is_none());
        assert_eq!(results[0].line_range, LineRange { start: 1, end: 3 });
    }

    #[test]
    fn dispatch_02_fallback_text_language() {
        let content = "hello world";
        let results = extract_by_language(Language::Text, SourceKind::Doc, content);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].strength, EvidenceStrength::Indirect);
        assert_eq!(results[0].line_range, LineRange { start: 1, end: 1 });
    }

    #[test]
    fn dispatch_03_fallback_empty() {
        let results = extract_by_language(Language::Unknown, SourceKind::Doc, "");
        assert!(results.is_empty());
    }

    #[test]
    fn dispatch_04_config_source_kind_priority() {
        // SourceKind::Config 优先于 Language，即使 Language 是 Unknown
        let content = "proc run {}";
        let results = extract_by_language(Language::Unknown, SourceKind::Config, content);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol.as_deref(), Some("run"));
    }

    #[test]
    fn dispatch_05_python_dispatch() {
        let content = "def foo():\n    pass";
        let results = extract_by_language(Language::Python, SourceKind::PythonStage, content);
        assert!(!results.is_empty());
        assert_eq!(results[0].symbol.as_deref(), Some("foo"));
    }

    #[test]
    fn dispatch_06_verilog_dispatch() {
        let content = "module top();\nendmodule";
        let results = extract_by_language(Language::Verilog, SourceKind::Rtl, content);
        assert!(!results.is_empty());
        assert_eq!(results[0].symbol.as_deref(), Some("top"));
    }

    #[test]
    fn dispatch_07_systemverilog_dispatch() {
        let content = "module sv_top();\nendmodule";
        let results = extract_by_language(Language::SystemVerilog, SourceKind::Rtl, content);
        assert!(!results.is_empty());
        assert_eq!(results[0].symbol.as_deref(), Some("sv_top"));
    }

    #[test]
    fn dispatch_08_markdown_dispatch() {
        let content = "# Title\nSome text";
        let results = extract_by_language(Language::Markdown, SourceKind::Doc, content);
        assert!(!results.is_empty());
        assert_eq!(results[0].symbol.as_deref(), Some("Title"));
    }

    // ─── extract_lines_range 辅助函数测试 ────────────────────────

    #[test]
    fn helper_extract_lines_range_full() {
        let content = "line1\nline2\nline3\nline4\nline5";
        assert_eq!(extract_lines_range(content, 1, 5), content);
    }

    #[test]
    fn helper_extract_lines_range_partial() {
        let content = "line1\nline2\nline3\nline4\nline5";
        assert_eq!(extract_lines_range(content, 2, 4), "line2\nline3\nline4");
    }

    #[test]
    fn helper_extract_lines_range_single() {
        let content = "line1\nline2\nline3";
        assert_eq!(extract_lines_range(content, 2, 2), "line2");
    }

    // ─── is_valid_identifier 测试 ─────────────────────────────────

    #[test]
    fn helper_valid_identifier() {
        assert!(is_valid_identifier("foo"));
        assert!(is_valid_identifier("_bar"));
        assert!(is_valid_identifier("MyClass"));
        assert!(is_valid_identifier("__init__"));
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("123"));
        assert!(!is_valid_identifier("1abc"));
    }
}
