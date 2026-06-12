/// Python 提取器 — def / class 关键字提取
///
/// 最小规则：
/// - 匹配行首或缩进后的 `def <name>(` → symbol=函数名, strength=Direct
/// - 匹配行首或缩进后的 `class <name>` → symbol=类名, strength=Direct
/// - 函数/类边界：基于缩进启发式，到下一个同级或更低缩进的 def/class 前一行，或 EOF
/// - 注释行（# 开头）中的 def/class 不提取
/// - 嵌套 def（方法/内部函数）会作为独立提取项返回
/// - 不做完整 Python AST，不做 async def（Phase 2 最小实现）

use super::{extract_lines_range, indent_level, is_valid_identifier, EvidenceExtractor};
use crate::evidence::models::{EvidenceStrength, LineRange, RawExtraction};

pub struct PythonExtractor;

/// def/class 条目
struct DefClassEntry {
    /// 0-based 行索引
    line_idx: usize,
    /// 符号名称
    symbol: String,
    /// 前导空白字符数
    indent: usize,
}

impl EvidenceExtractor for PythonExtractor {
    fn extract(&self, content: &str) -> Vec<RawExtraction> {
        if content.is_empty() {
            return vec![];
        }

        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return vec![];
        }

        // 第一遍：收集所有 def/class 条目
        let mut entries: Vec<DefClassEntry> = vec![];
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let indent = indent_level(line);

            // 匹配 def <name>(
            if let Some(rest) = trimmed.strip_prefix("def ") {
                if let Some(paren_pos) = rest.find('(') {
                    let name = rest[..paren_pos].trim();
                    if !name.is_empty() && is_valid_identifier(name) {
                        entries.push(DefClassEntry {
                            line_idx: i,
                            symbol: name.to_string(),
                            indent,
                        });
                        continue;
                    }
                }
            }

            // 匹配 class <name>
            if let Some(rest) = trimmed.strip_prefix("class ") {
                let name_end = rest
                    .find(|c: char| c == '(' || c == ':' || c == '\n')
                    .unwrap_or(rest.len());
                let name = rest[..name_end].trim();
                if !name.is_empty() && is_valid_identifier(name) {
                    entries.push(DefClassEntry {
                        line_idx: i,
                        symbol: name.to_string(),
                        indent,
                    });
                }
            }
        }

        // 第二遍：计算每个条目的边界并构建 RawExtraction
        let total_lines = lines.len();
        let mut results = vec![];

        for (idx, entry) in entries.iter().enumerate() {
            let start = (entry.line_idx + 1) as u32; // 1-based

            // 找下一个同级或更低缩进的 def/class
            let end_1based = entries[idx + 1..]
                .iter()
                .find(|e| e.indent <= entry.indent)
                .map(|e| e.line_idx as u32) // 0-based index = 1-based 行号的前一行
                .unwrap_or(total_lines as u32);

            let raw_excerpt = extract_lines_range(content, start, end_1based);

            results.push(RawExtraction {
                symbol: Some(entry.symbol.clone()),
                line_range: LineRange {
                    start,
                    end: end_1based,
                },
                raw_excerpt,
                strength: EvidenceStrength::Direct,
            });
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn py_01_single_def() {
        let content = "def foo():\n    pass";
        let results = PythonExtractor.extract(content);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol.as_deref(), Some("foo"));
        assert_eq!(results[0].line_range, LineRange { start: 1, end: 2 });
        assert_eq!(results[0].strength, EvidenceStrength::Direct);
        assert_eq!(results[0].raw_excerpt, content);
    }

    #[test]
    fn py_02_multiple_defs() {
        let content = "def foo():\n    x = 1\n    return x\n\ndef bar(a, b):\n    c = a + b\n    return c";
        let results = PythonExtractor.extract(content);
        assert_eq!(results.len(), 2);
        // foo: lines 1-3 (next def at line 5, indent=0, so end=4... wait)
        // entries: foo at idx=0 indent=0, bar at idx=4 indent=0
        // foo end = bar.line_idx = 4 (1-based) → range {1, 4}
        assert_eq!(results[0].symbol.as_deref(), Some("foo"));
        assert_eq!(results[0].line_range, LineRange { start: 1, end: 4 });
        assert_eq!(results[1].symbol.as_deref(), Some("bar"));
        assert_eq!(results[1].line_range, LineRange { start: 5, end: 7 });
    }

    #[test]
    fn py_03_class_definition() {
        let content = "class SignalProcessor:\n    def __init__(self, config):\n        self.config = config";
        let results = PythonExtractor.extract(content);
        assert_eq!(results.len(), 2);
        // class at idx=0 indent=0, __init__ at idx=1 indent=4
        // class end: next entry with indent<=0 → none → EOF=3
        assert_eq!(results[0].symbol.as_deref(), Some("SignalProcessor"));
        assert_eq!(results[0].line_range, LineRange { start: 1, end: 3 });
        // __init__ at indent=4, no next entry → EOF=3
        assert_eq!(results[1].symbol.as_deref(), Some("__init__"));
        assert_eq!(results[1].line_range, LineRange { start: 2, end: 3 });
    }

    #[test]
    fn py_04_nested_def() {
        // 嵌套 def：外层 range 包含内层，内层也作为独立提取项
        let content = "class Foo:\n    def bar(self):\n        def inner():\n            pass\n        return inner";
        let results = PythonExtractor.extract(content);
        assert_eq!(results.len(), 3);
        // class Foo: idx=0, indent=0 → EOF=5
        assert_eq!(results[0].symbol.as_deref(), Some("Foo"));
        assert_eq!(results[0].line_range, LineRange { start: 1, end: 5 });
        // def bar: idx=1, indent=4, next <=4 → inner at idx=2 indent=8 (skip, 8>4) → EOF=5
        assert_eq!(results[1].symbol.as_deref(), Some("bar"));
        assert_eq!(results[1].line_range, LineRange { start: 2, end: 5 });
        // def inner: idx=2, indent=8 → EOF=5
        assert_eq!(results[2].symbol.as_deref(), Some("inner"));
        assert_eq!(results[2].line_range, LineRange { start: 3, end: 5 });
    }

    #[test]
    fn py_05_comment_def_not_extracted() {
        let content = "# def commented():\n# class NotReal:\npass";
        let results = PythonExtractor.extract(content);
        assert!(results.is_empty(), "comment lines should not produce extractions");
    }

    #[test]
    fn py_06_empty_file() {
        let results = PythonExtractor.extract("");
        assert!(results.is_empty());
    }
}
