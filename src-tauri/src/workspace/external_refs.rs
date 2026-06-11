use std::collections::HashSet;
use std::path::Path;

/// 检测 `urban_wireless` 外部模块引用。
/// 扫描文件内容前 100 行，做简单字符串匹配。
/// 不做 AST 解析，不执行 Python import。
///
/// 匹配模式（不区分大小写）：
/// - `"from urban_wireless import"`
/// - `"import urban_wireless"`
/// - `"urban_wireless."`
/// - 路径字符串中包含 `"urban_wireless"`
pub fn detect_urban_wireless(path: &Path) -> Vec<String> {
    let content = match read_head(path, 100) {
        Some(c) => c,
        None => return Vec::new(),
    };

    let lower = content.to_lowercase();
    let mut refs = HashSet::new();

    if lower.contains("urban_wireless") {
        refs.insert("urban_wireless".to_string());
    }

    refs.into_iter().collect()
}

/// 读取文件前 `max_lines` 行。
fn read_head(path: &Path, max_lines: usize) -> Option<String> {
    use std::io::{BufRead, BufReader};

    let file = std::fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    let mut lines = Vec::with_capacity(max_lines);

    for line in reader.lines().take(max_lines) {
        match line {
            Ok(l) => lines.push(l),
            Err(_) => break,
        }
    }

    Some(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn detects_import_pattern() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("script.py");
        fs::write(&path, "from urban_wireless import channel_model\n").unwrap();

        let refs = detect_urban_wireless(&path);
        assert_eq!(refs, vec!["urban_wireless"]);
    }

    #[test]
    fn detects_dot_pattern() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("script.py");
        fs::write(&path, "urban_wireless.setup()\n").unwrap();

        let refs = detect_urban_wireless(&path);
        assert_eq!(refs, vec!["urban_wireless"]);
    }

    #[test]
    fn no_match_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("script.py");
        fs::write(&path, "print('hello')\n").unwrap();

        let refs = detect_urban_wireless(&path);
        assert!(refs.is_empty());
    }
}
