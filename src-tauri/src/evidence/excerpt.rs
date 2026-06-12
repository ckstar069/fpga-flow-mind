/// 摘要提取与截断工具
///
/// 提供纯函数用于截断 evidence summary 和生成文件级摘要。
/// 所有截断按 char 边界操作，不破坏 UTF-8。

/// 截断后缀："...(已截断，共 N 行)"
const TRUNCATE_SUFFIX_TEMPLATE: &str = "...(已截断，共 ";
const TRUNCATE_SUFFIX_END: &str = " 行)";

/// 文件级摘要后缀："...(共 N 行)"
const FILE_SUFFIX_TEMPLATE: &str = "...(共 ";
const FILE_SUFFIX_END: &str = " 行)";

/// 计算 input 中的行数（按 '\n' 计数，空字符串为 0 行）
pub fn count_lines(input: &str) -> usize {
    if input.is_empty() {
        return 0;
    }
    input.lines().count()
}

/// 截断 summary 文本
///
/// 如果 input 长度 <= max_chars，返回 (原文, false)。
/// 否则截取前 keep_chars 个字符（按 char 边界），追加 "...(已截断，共 N 行)" 后缀，
/// 返回 (截断结果, true)。
///
/// total_lines 用于后缀中的行数显示。
pub fn truncate_summary(
    input: &str,
    max_chars: usize,
    keep_chars: usize,
    total_lines: usize,
) -> (String, bool) {
    if input.len() <= max_chars {
        return (input.to_string(), false);
    }

    let truncated: String = input.chars().take(keep_chars).collect();
    let suffix = format!(
        "{}{}{}{}",
        TRUNCATE_SUFFIX_TEMPLATE, total_lines, TRUNCATE_SUFFIX_END,
        if total_lines > 1 { "" } else { "" }
    );
    (format!("{}{}", truncated, suffix), true)
}

/// 生成整文件级摘要
///
/// 取前 preview_chars 个字符作为预览，超出时追加 "...(共 N 行)"。
/// 不保存全文。
pub fn make_file_level_summary(
    input: &str,
    preview_chars: usize,
    total_lines: usize,
) -> (String, bool) {
    if input.chars().count() <= preview_chars {
        return (input.to_string(), false);
    }

    let preview: String = input.chars().take(preview_chars).collect();
    let suffix = format!("{}{}{}", FILE_SUFFIX_TEMPLATE, total_lines, FILE_SUFFIX_END);
    (format!("{}{}", preview, suffix), true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excerpt_01_short_text_no_truncation() {
        let input = "short text";
        let (result, truncated) = truncate_summary(input, 500, 400, 1);
        assert!(!truncated);
        assert_eq!(result, input);
    }

    #[test]
    fn excerpt_02_exact_max_no_truncation() {
        let input: String = "a".repeat(500);
        let (result, truncated) = truncate_summary(&input, 500, 400, 10);
        assert!(!truncated);
        assert_eq!(result.len(), 500);
    }

    #[test]
    fn excerpt_03_over_max_truncates() {
        let input: String = "a".repeat(600);
        let (result, truncated) = truncate_summary(&input, 500, 400, 20);
        assert!(truncated);
        assert!(result.starts_with(&"a".repeat(400)));
        assert!(result.contains("已截断"));
        assert!(result.contains("20 行"));
    }

    #[test]
    fn excerpt_04_multibyte_char_safe() {
        // 中文字符每个 3 字节 UTF-8
        let input: String = "你".repeat(200); // 200 chars, 600 bytes
        let (result, truncated) = truncate_summary(&input, 100, 80, 10);
        assert!(truncated);
        // 验证不 panic 且结果为有效 UTF-8
        assert!(result.is_char_boundary(result.len()));
        // 验证截断部分只包含原始字符
        let truncated_part: String = result.chars().take(80).collect();
        assert!(truncated_part.chars().all(|c| c == '你'));
        // 验证后缀包含行数信息
        assert!(result.contains("已截断"));
    }

    #[test]
    fn excerpt_05_empty_string() {
        let (result, truncated) = truncate_summary("", 500, 400, 0);
        assert!(!truncated);
        assert_eq!(result, "");
    }

    #[test]
    fn excerpt_06_file_level_summary_short() {
        let input = "hello world";
        let (result, truncated) = make_file_level_summary(input, 200, 5);
        assert!(!truncated);
        assert_eq!(result, input);
    }

    #[test]
    fn excerpt_07_file_level_summary_long() {
        let input: String = "x".repeat(500);
        let (result, truncated) = make_file_level_summary(&input, 200, 50);
        assert!(truncated);
        assert!(result.starts_with(&"x".repeat(200)));
        assert!(result.contains("共 50 行"));
    }

    #[test]
    fn excerpt_08_file_level_summary_empty() {
        let (result, truncated) = make_file_level_summary("", 200, 0);
        assert!(!truncated);
        assert_eq!(result, "");
    }

    #[test]
    fn excerpt_09_count_lines() {
        assert_eq!(count_lines(""), 0);
        assert_eq!(count_lines("one line"), 1);
        assert_eq!(count_lines("line1\nline2\nline3"), 3);
        assert_eq!(count_lines("line1\nline2\n"), 2); // Rust lines() 不计入末尾换行后的空行
        assert_eq!(count_lines("\n\n"), 2); // 两个换行 = 2 个空行
    }

    #[test]
    fn excerpt_10_multiline_with_cjk() {
        let input = "第一行\n第二行内容\n第三行";
        let lines = count_lines(input);
        assert_eq!(lines, 3);

        let input_long: String = "第一行内容很长\n".repeat(50); // 50 lines
        let (result, truncated) = make_file_level_summary(&input_long, 100, 50);
        assert!(truncated);
        assert!(result.contains("共 50 行"));
    }
}
