/// Markdown 提取器 — 标题章节提取
///
/// 最小规则：
/// - 提取 `#` / `##` / `###` 标题
/// - symbol = 标题文本（去除前后空白）
/// - line_range.start = 标题行（1-based）
/// - line_range.end = 下一个同级或更高等级标题的前一行，或 EOF
/// - strength = Direct（标题关键字匹配是直接证据，章节范围是附加上下文）
/// - fenced code block（``` 或 ~~~）内的 `#` 不作为标题提取
/// - 无标题返回空列表

use super::{extract_lines_range, EvidenceExtractor};
use crate::evidence::models::{EvidenceStrength, LineRange, RawExtraction};

pub struct MarkdownExtractor;

/// 标题条目
struct HeadingEntry {
    /// 0-based 行索引
    line_idx: usize,
    /// 标题级别（1-3，对应 # 数量）
    level: usize,
    /// 标题文本
    title: String,
}

/// 检查是否为 fenced code block 的边界线（``` 或 ~~~，至少 3 个）
fn is_fence_line(line: &str) -> bool {
    let trimmed = line.trim();
    (trimmed.starts_with("```") || trimmed.starts_with("~~~"))
        && trimmed.len() >= 3
        && trimmed.chars().take(3).all(|c| c == trimmed.chars().next().unwrap())
}

/// 解析标题行，返回 (level, title_text) 或 None
fn parse_heading(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('#') {
        return None;
    }

    // 统计前导 # 数量
    let level = trimmed.chars().take_while(|c| *c == '#').count();
    if level == 0 || level > 6 {
        return None;
    }
    // 只提取 h1-h3
    if level > 3 {
        return None;
    }

    // # 后面必须跟空格或行尾
    let rest = &trimmed[level..];
    if !rest.is_empty() && !rest.starts_with(' ') && !rest.starts_with('\t') {
        return None;
    }

    let title = rest.trim_start().trim_end().to_string();
    if title.is_empty() {
        return None;
    }

    Some((level, title))
}

impl EvidenceExtractor for MarkdownExtractor {
    fn extract(&self, content: &str) -> Vec<RawExtraction> {
        if content.is_empty() {
            return vec![];
        }

        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return vec![];
        }

        let mut entries: Vec<HeadingEntry> = vec![];
        let mut in_code_block = false;

        for (i, line) in lines.iter().enumerate() {
            // 检测 fenced code block 边界
            if is_fence_line(line) {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            if let Some((level, title)) = parse_heading(line) {
                entries.push(HeadingEntry {
                    line_idx: i,
                    level,
                    title,
                });
            }
        }

        // 计算每个标题的边界
        let total_lines = lines.len();
        let mut results = vec![];

        for (idx, entry) in entries.iter().enumerate() {
            let start = (entry.line_idx + 1) as u32; // 1-based

            // 找下一个同级或更高级（level <= 当前）的标题
            let end_1based = entries[idx + 1..]
                .iter()
                .find(|e| e.level <= entry.level)
                .map(|e| e.line_idx as u32) // 0-based index = 1-based 前一行
                .unwrap_or(total_lines as u32);

            let raw_excerpt = extract_lines_range(content, start, end_1based);

            results.push(RawExtraction {
                symbol: Some(entry.title.clone()),
                line_range: LineRange { start, end: end_1based },
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
    fn md_01_single_h1() {
        let content = "# Project Title\n\nSome description\n\nMore text";
        let results = MarkdownExtractor.extract(content);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol.as_deref(), Some("Project Title"));
        assert_eq!(results[0].line_range, LineRange { start: 1, end: 5 });
        assert_eq!(results[0].strength, EvidenceStrength::Direct);
    }

    #[test]
    fn md_02_multi_level_headings() {
        let content = "# Title\n\nIntro\n\n## Section A\n\nContent A\n\n## Section B\n\nContent B\n\n# Another Title\n\nMore";
        let results = MarkdownExtractor.extract(content);
        assert_eq!(results.len(), 4);
        // # Title (level 1, idx 0): next level<=1 → # Another Title at idx=12 → end=12
        assert_eq!(results[0].symbol.as_deref(), Some("Title"));
        assert_eq!(results[0].line_range, LineRange { start: 1, end: 12 });
        // ## Section A (level 2, idx 4, 1-based=5): next level<=2 → ## Section B at idx=8 → end=8
        assert_eq!(results[1].symbol.as_deref(), Some("Section A"));
        assert_eq!(results[1].line_range, LineRange { start: 5, end: 8 });
        // ## Section B (level 2, idx 8, 1-based=9): next level<=2 → # Another Title at idx=12 → end=12
        assert_eq!(results[2].symbol.as_deref(), Some("Section B"));
        assert_eq!(results[2].line_range, LineRange { start: 9, end: 12 });
        // # Another Title (level 1, idx 12, 1-based=13): no next → EOF=15
        assert_eq!(results[3].symbol.as_deref(), Some("Another Title"));
        assert_eq!(results[3].line_range, LineRange { start: 13, end: 15 });
    }

    #[test]
    fn md_03_no_heading_returns_empty() {
        let content = "Just some text\nwithout any headings";
        let results = MarkdownExtractor.extract(content);
        assert!(results.is_empty());
    }

    #[test]
    fn md_04_code_block_heading_not_extracted() {
        let content = "# Real Title\n\n```python\n# This is a comment\nx = 1\n```\n\n## Section";
        let results = MarkdownExtractor.extract(content);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].symbol.as_deref(), Some("Real Title"));
        // # Real Title (level 1, line 1): next level<=1 → none → EOF=8
        assert_eq!(results[0].line_range, LineRange { start: 1, end: 8 });
        assert_eq!(results[1].symbol.as_deref(), Some("Section"));
        // ## Section (level 2, line 8)
        assert_eq!(results[1].line_range, LineRange { start: 8, end: 8 });
    }

    #[test]
    fn md_05_empty_content() {
        let results = MarkdownExtractor.extract("");
        assert!(results.is_empty());
    }
}
