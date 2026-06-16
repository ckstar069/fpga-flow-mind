/// Python 提取器 — def / class / import / constant / dataclass / return-type / call-site 提取
///
/// 规则：
/// - 匹配行首或缩进后的 `def <name>(` → symbol=函数名, strength=Direct
/// - 匹配行首或缩进后的 `class <name>` → symbol=类名, strength=Direct
/// - 函数/类边界：基于缩进启发式，到下一个同级或更低缩进的 def/class 前一行，或 EOF
/// - 注释行（# 开头）中的关键字不提取
/// - 嵌套 def（方法/内部函数）会作为独立提取项返回
/// - `import X` / `from Y import Z` → symbol=模块/标识符, strength=Direct
/// - 模块级 `NAME = value`（纯大写或下划线前缀，indent=0）→ symbol=常量名, strength=Direct
/// - `@dataclass` 类的字段声明 `name: type[ = default]` → symbol=字段名, strength=Direct
/// - `def fn() -> ReturnType` 中的返回类型 → 独立提取项, strength=Direct
/// - `self.field = expr` 在 __init__ 内的类字段 → symbol=字段名, strength=Indirect
/// - 函数体内的关键调用 `func(...)` 或 `obj.method(...)` → symbol=被调用名, strength=Indirect
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

impl PythonExtractor {
    /// 提取 def/class 条目并计算边界
    fn extract_defs_and_classes(&self, content: &str, lines: &[&str]) -> Vec<RawExtraction> {
        let total_lines = lines.len();
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

        let mut results = vec![];
        for (idx, entry) in entries.iter().enumerate() {
            let start = (entry.line_idx + 1) as u32; // 1-based

            let end_1based = entries[idx + 1..]
                .iter()
                .find(|e| e.indent <= entry.indent)
                .map(|e| e.line_idx as u32)
                .unwrap_or(total_lines as u32);

            let raw_excerpt = extract_lines_range(content, start, end_1based);

            results.push(RawExtraction {
                symbol: Some(entry.symbol.clone()),
                line_range: LineRange { start, end: end_1based },
                raw_excerpt,
                strength: EvidenceStrength::Direct,
            });
        }

        results
    }

    /// 提取 import 语句
    ///
    /// - `import X` → symbol=X, 行范围, Direct
    /// - `from Y import Z` → symbol=Z, 行范围, Direct
    fn extract_imports(&self, lines: &[&str]) -> Vec<RawExtraction> {
        let mut results = vec![];
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let start_1based = (i + 1) as u32;

            // from X import Y
            if let Some(rest) = trimmed.strip_prefix("from ") {
                if let Some(import_pos) = rest.find(" import ") {
                    let module = rest[..import_pos].trim();
                    let after_import = rest[import_pos + 8..].trim();
                    if !module.is_empty() && !after_import.is_empty() {
                        // from X import Y, Z → extract each imported symbol
                        for tok in after_import.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                            // Handle "from X import Y as Z" → keep Z
                            let name = tok.split_whitespace()
                                .last()
                                .unwrap_or(tok)
                                .to_string();
                            if is_valid_identifier(&name) {
                                results.push(RawExtraction {
                                    symbol: Some(name),
                                    line_range: LineRange { start: start_1based, end: start_1based },
                                    raw_excerpt: line.to_string(),
                                    strength: EvidenceStrength::Direct,
                                });
                            }
                        }
                        continue;
                    }
                }
            }

            // import X
            if let Some(rest) = trimmed.strip_prefix("import ") {
                // import X, Y, Z → extract each module
                for tok in rest.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                    // Handle "import X.Y.Z" → use last segment
                    // Handle "import X as Y" → use Y
                    let name = tok.split_whitespace()
                        .last()
                        .unwrap_or(tok)
                        .trim()
                        .to_string();
                    if !name.is_empty() {
                        results.push(RawExtraction {
                            symbol: Some(name),
                            line_range: LineRange { start: start_1based, end: start_1based },
                            raw_excerpt: line.to_string(),
                            strength: EvidenceStrength::Direct,
                        });
                    }
                }
            }
        }
        results
    }

    /// 提取模块级顶层常量（全大写或下划线前缀，indent=0）
    ///
    /// 规则：
    /// - `NAME = expr` 或 `_name = expr` 在 indent=0
    /// - 不考虑多行表达式
    fn extract_constants(&self, lines: &[&str]) -> Vec<RawExtraction> {
        let mut results = vec![];
        let mut in_function_or_class = 0usize; // indent level if inside, 0 if not

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let indent = indent_level(line);

            // Track entering def/class at indent 0
            if indent == 0 {
                if trimmed.starts_with("def ") || trimmed.starts_with("class ") {
                    in_function_or_class = 0; // reset, we're at top level
                    continue;
                }
            }

            // Skip if inside a function or class (indent > 0 and we're past the def/class line)
            if indent > 0 && in_function_or_class > 0 {
                // Check if we've returned to outer level
                if indent < in_function_or_class {
                    // Keep going, we're just at a less indented level
                }
                continue;
            }

            // Only indent=0 for constants
            if indent > 0 {
                continue;
            }

            // Match NAME = <value> or _name = <value>
            if let Some(eq_pos) = trimmed.find('=') {
                let name_part = trimmed[..eq_pos].trim();
                // Name must be an identifier and not start with def/class/import/return
                if !name_part.starts_with("def ") && !name_part.starts_with("class ")
                    && !name_part.starts_with("import ") && !name_part.starts_with("return ")
                    && !name_part.starts_with("from ") && !name_part.starts_with("if ")
                    && !name_part.starts_with("elif ") && !name_part.starts_with("else:")
                    && !name_part.starts_with("for ") && !name_part.starts_with("while ")
                    && !name_part.starts_with("with ") && !name_part.starts_with("try:")
                    && !name_part.starts_with("except") && !name_part.starts_with("raise")
                {
                    // Accept constants: UPPER_CASE or _prefix
                    let name = name_part.to_string();
                    if is_valid_identifier(&name)
                        && (name.chars().all(|c| c.is_uppercase() || c == '_' || c.is_digit(10))
                            || name.starts_with('_'))
                    {
                        let start_1based = (i + 1) as u32;
                        results.push(RawExtraction {
                            symbol: Some(name),
                            line_range: LineRange { start: start_1based, end: start_1based },
                            raw_excerpt: line.to_string(),
                            strength: EvidenceStrength::Direct,
                        });
                    }
                }
            }
        }
        results
    }

    /// 提取 @dataclass 类的字段声明
    ///
    /// 匹配 `name: type` 或 `name: type = default` 在 @dataclass 类体内
    fn extract_dataclass_fields(&self, lines: &[&str]) -> Vec<RawExtraction> {
        let mut results = vec![];
        let mut in_dataclass = false;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();

            // Detect @dataclass decorator
            if trimmed.starts_with("@dataclass") {
                in_dataclass = true;
                continue;
            }

            if !in_dataclass {
                continue;
            }

            let indent = indent_level(line);

            if trimmed.starts_with("class ") {
                // This is the class line after @dataclass
                // Reset tracking — the class body is indented
                continue;
            }

            if indent <= 0 {
                // Back to top level → no longer in dataclass
                in_dataclass = false;
                continue;
            }

            // Track indent of class body (should be > 0)
            if indent > 0 && !trimmed.starts_with('#') {
                // Check for field declaration: name: type[ = default]
                if let Some(colon_pos) = trimmed.find(':') {
                    let name_part = trimmed[..colon_pos].trim();
                    // Must be a valid identifier (not def/class/return/import/etc.)
                    if is_valid_identifier(name_part)
                        && !name_part.starts_with("def ")
                        && !name_part.starts_with("class ")
                        && !name_part.starts_with("return ")
                    {
                        let start_1based = (i + 1) as u32;
                        results.push(RawExtraction {
                            symbol: Some(name_part.to_string()),
                            line_range: LineRange { start: start_1based, end: start_1based },
                            raw_excerpt: line.to_string(),
                            strength: EvidenceStrength::Direct,
                        });
                    }
                }
            }
        }
        results
    }

    /// 提取函数返回类型注释（-> Type）
    ///
    /// 规则：在 def line 中匹配 `-> <type>:` 模式
    fn extract_return_types(&self, lines: &[&str]) -> Vec<RawExtraction> {
        let mut results = vec![];
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if !trimmed.starts_with("def ") || trimmed.starts_with('#') {
                continue;
            }

            // Match -> Type in the def line
            if let Some(arrow_pos) = trimmed.find("->") {
                let after_arrow = trimmed[arrow_pos + 2..].trim();
                if let Some(colon_pos) = after_arrow.find(':') {
                    let ret_type = after_arrow[..colon_pos].trim();
                    if !ret_type.is_empty() {
                        let start_1based = (i + 1) as u32;
                        // Return type as child evidence of the function
                        // Only use last identifier for clean symbol
                        let clean_symbol = ret_type.split_whitespace()
                            .last()
                            .unwrap_or(ret_type)
                            .trim_end_matches('>')
                            .to_string();
                        if is_valid_identifier(&clean_symbol) || clean_symbol.contains("List") || clean_symbol.contains("Dict") || clean_symbol.contains("Optional") {
                            results.push(RawExtraction {
                                symbol: Some(clean_symbol),
                                line_range: LineRange { start: start_1based, end: start_1based },
                                raw_excerpt: format!("{} -> {}", trimmed.trim_end_matches(':'), ret_type),
                                strength: EvidenceStrength::Direct,
                            });
                        }
                    }
                }
            }
        }
        results
    }

    /// 提取 __init__ 方法内的 self.field = value 类字段赋值
    ///
    /// 规则：在类方法体内（非顶层 indent=0），匹配 `self.<name> = <expr>`
    fn extract_self_fields(&self, lines: &[&str]) -> Vec<RawExtraction> {
        let mut results = vec![];
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if !trimmed.starts_with("self.") || !trimmed.contains(" = ") {
                continue;
            }
            // Extract field name: self.<name> =
            let after_self = &trimmed["self.".len()..];
            if let Some(eq_pos) = after_self.find(" = ") {
                let field_name = after_self[..eq_pos].trim();
                if is_valid_identifier(field_name) && !field_name.starts_with('_') {
                    let start_1based = (i + 1) as u32;
                    results.push(RawExtraction {
                        symbol: Some(field_name.to_string()),
                        line_range: LineRange { start: start_1based, end: start_1based },
                        raw_excerpt: trimmed.to_string(),
                        strength: EvidenceStrength::Indirect,
                    });
                }
            }
        }
        results
    }

    /// 提取函数体内的关键调用（保守：仅匹配 `func(...)` 或 `obj.method(...)` 这些可识别的调用站）
    ///
    /// 规则：
    /// - 在非顶层 indent>0 行中，匹配 `identifier(` 或 `identifier.identifier(`
    /// - 跳过 def/class/if/for/while/return 等关键字
    fn extract_call_sites(&self, lines: &[&str]) -> Vec<RawExtraction> {
        let mut results = vec![];
        let mut seen: std::collections::HashSet<(usize, String)> = std::collections::HashSet::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let indent = indent_level(line);
            if indent == 0 {
                continue; // skip top-level (def/class/import lines)
            }

            // Don't match def, class, if, for, while, return, import, from, raise
            if trimmed.starts_with("def ") || trimmed.starts_with("class ")
                || trimmed.starts_with("if ") || trimmed.starts_with("elif ")
                || trimmed.starts_with("for ") || trimmed.starts_with("while ")
                || trimmed.starts_with("return ") || trimmed.starts_with("import ")
                || trimmed.starts_with("from ") || trimmed.starts_with("raise ")
                || trimmed.starts_with("try:") || trimmed.starts_with("except")
                || trimmed.starts_with("with ") || trimmed.starts_with("else:")
            {
                continue;
            }

            // Find function call patterns: name(...) or obj.name(...)
            // Match: identifier( or identifier.identifier(
            let mut scan_pos = 0;
            let s = trimmed.as_bytes();
            while scan_pos < s.len() {
                // Look for '('
                if let Some(paren_pos) = trimmed[scan_pos..].find('(') {
                    let before_paren = trimmed[scan_pos..scan_pos + paren_pos].trim_end();
                    // Extract the call expression before '('
                    // Find the start of the identifier (last word before paren)
                    // Match: `name(` or `obj.name(` or `self.method(`
                    if let Some(last_call) = before_paren.split(|c: char| c == ' ' || c == '\t' || c == ',' || c == ';')
                        .filter(|s| !s.is_empty())
                        .last()
                    {
                        // Clean up: remove trailing whitespace/newlines
                        let call_name = last_call.trim_end();
                        if is_valid_identifier(call_name) || call_name.contains('.') {
                            // Extract the last identifier part of call_name
                            let last_seg = call_name.rsplit('.').next().unwrap_or(call_name);
                            if is_valid_identifier(last_seg)
                                && !last_seg.starts_with('_')
                                && last_seg.len() > 1
                            {
                                let key = (i, last_seg.to_string());
                                if seen.insert(key) {
                                    let start_1based = (i + 1) as u32;
                                    results.push(RawExtraction {
                                        symbol: Some(last_seg.to_string()),
                                        line_range: LineRange { start: start_1based, end: start_1based },
                                        raw_excerpt: trimmed.to_string(),
                                        strength: EvidenceStrength::Indirect,
                                    });
                                }
                            }
                        }
                    }
                    scan_pos += paren_pos + 1;
                } else {
                    break;
                }
            }
        }
        results
    }
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

        let mut results = Vec::new();

        // Pass 1: def/class extraction (existing)
        results.extend(self.extract_defs_and_classes(content, &lines));

        // Pass 2: import extraction
        results.extend(self.extract_imports(&lines));

        // Pass 3: top-level constants
        results.extend(self.extract_constants(&lines));

        // Pass 4: dataclass fields (inside @dataclass classes)
        results.extend(self.extract_dataclass_fields(&lines));

        // Pass 5: return type annotations
        results.extend(self.extract_return_types(&lines));

        // Pass 6: self.field assignments in __init__
        results.extend(self.extract_self_fields(&lines));

        // Pass 7: key call-sites
        results.extend(self.extract_call_sites(&lines));

        // Sort by line number for stable deterministic order
        results.sort_by_key(|r| r.line_range.start);

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
        // Now also extracts self.config as indirect evidence
        assert!(results.len() >= 2, "至少应有 class + __init__，实际 {}", results.len());
        // class at idx=0 indent=0, __init__ at idx=1 indent=4
        let class_result = results.iter().find(|r| r.symbol.as_deref() == Some("SignalProcessor"));
        assert!(class_result.is_some(), "应提取 SignalProcessor");
        assert_eq!(class_result.unwrap().line_range, LineRange { start: 1, end: 3 });
        let init_result = results.iter().find(|r| r.symbol.as_deref() == Some("__init__"));
        assert!(init_result.is_some(), "应提取 __init__");
        assert_eq!(init_result.unwrap().line_range, LineRange { start: 2, end: 3 });
        // self.config extracted as indirect evidence
        let config_result = results.iter().find(|r| r.symbol.as_deref() == Some("config") && r.strength == EvidenceStrength::Indirect);
        assert!(config_result.is_some(), "应提取 self.config");
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
        // "pass" has no def/class, so only top-level constant analysis runs
        // "pass" doesn't match any extraction rule, so results should be empty
        let def_class_results: Vec<_> = results.iter().filter(|r| r.strength == EvidenceStrength::Direct).collect();
        assert!(def_class_results.is_empty(), "comment lines should not produce def/class extractions");
    }

    #[test]
    fn py_06_empty_file() {
        let results = PythonExtractor.extract("");
        assert!(results.is_empty());
    }

    // ─── P1: 新增提取类别的测试 ────────────────────────────────────

    /// py_07: import X 提取
    #[test]
    fn py_07_import_extraction() {
        let content = "import numpy\nimport torch.nn as nn\nfrom math import sqrt, cos\n";
        let results = PythonExtractor.extract(content);
        let imports: Vec<_> = results.iter()
            .filter(|r| r.strength == EvidenceStrength::Direct
                && (r.symbol.as_deref() == Some("numpy")
                    || r.symbol.as_deref() == Some("nn")
                    || r.symbol.as_deref() == Some("sqrt")
                    || r.symbol.as_deref() == Some("cos")))
            .collect();
        assert!(imports.len() >= 4, "应提取 4 个 import 符号, 实际: {:?}", imports.iter().map(|r| r.symbol.as_deref()).collect::<Vec<_>>());
        for r in &results {
            assert_eq!(r.line_range.start, r.line_range.end, "import 应为单行");
        }
    }

    /// py_08: 顶层常量提取
    #[test]
    fn py_08_constant_extraction() {
        let content = "SAMPLE_RATE = 48000\nFFT_SIZE = 1024\n_internal = \"secret\"\nnot_constant = 42\n";
        let results = PythonExtractor.extract(content);
        let const_names: Vec<&str> = results.iter()
            .filter(|r| r.strength == EvidenceStrength::Direct
                && (r.symbol.as_deref() == Some("SAMPLE_RATE")
                    || r.symbol.as_deref() == Some("FFT_SIZE")
                    || r.symbol.as_deref() == Some("_internal")))
            .map(|r| r.symbol.as_deref().unwrap())
            .collect();
        assert!(const_names.contains(&"SAMPLE_RATE"), "应提取 SAMPLE_RATE");
        assert!(const_names.contains(&"FFT_SIZE"), "应提取 FFT_SIZE");
        assert!(const_names.contains(&"_internal"), "应提取 _internal");
        assert!(!const_names.contains(&"not_constant"), "not_constant 不应提取（非全大写非下划线前缀）");
    }

    /// py_09: @dataclass 字段提取
    #[test]
    fn py_09_dataclass_field_extraction() {
        let content = "\
@dataclass
class Config:
    sample_rate: int
    fft_size: int = 1024
    enable_debug: bool = False
";
        let results = PythonExtractor.extract(content);
        let field_names: Vec<&str> = results.iter()
            .filter_map(|r| r.symbol.as_deref())
            .filter(|s| *s == "sample_rate" || *s == "fft_size" || *s == "enable_debug")
            .collect();
        assert!(field_names.contains(&"sample_rate"), "应提取 sample_rate");
        assert!(field_names.contains(&"fft_size"), "应提取 fft_size");
        assert!(field_names.contains(&"enable_debug"), "应提取 enable_debug");
    }

    /// py_10: 返回类型提取
    #[test]
    fn py_10_return_type_extraction() {
        let content = "def compute() -> float:\n    return 1.0\n\ndef process(data: list) -> Optional[list]:\n    return data\n";
        let results = PythonExtractor.extract(content);
        let ret_types: Vec<&str> = results.iter()
            .filter_map(|r| r.symbol.as_deref())
            .filter(|s| *s == "float" || *s == "list" || s.contains("Optional"))
            .collect();
        assert!(ret_types.contains(&"float"), "应提取 float 返回类型");
        assert!(!ret_types.is_empty(), "应至少提取一个返回类型");
    }

    /// py_11: self.field 提取（类字段）
    #[test]
    fn py_11_self_field_extraction() {
        let content = "\
class Processor:
    def __init__(self):
        self.config = {}
        self.buffer = []
        self._private = 0
";
        let results = PythonExtractor.extract(content);
        let field_names: Vec<&str> = results.iter()
            .filter(|r| r.strength == EvidenceStrength::Indirect)
            .filter_map(|r| r.symbol.as_deref())
            .collect();
        assert!(field_names.contains(&"config"), "应提取 config 字段");
        assert!(field_names.contains(&"buffer"), "应提取 buffer 字段");
        assert!(!field_names.contains(&"_private"), "私有字段 _private 不应提取");
    }

    /// py_12: 函数内关键调用提取
    #[test]
    fn py_12_call_site_extraction() {
        let content = "\
def main():
    data = load_samples()
    result = correlate(data)
    peak = detect_peak(result)
    return peak
";
        let results = PythonExtractor.extract(content);
        let calls: Vec<&str> = results.iter()
            .filter(|r| r.strength == EvidenceStrength::Indirect)
            .filter_map(|r| r.symbol.as_deref())
            .collect();
        assert!(calls.contains(&"load_samples"), "应提取 load_samples 调用");
        assert!(calls.contains(&"correlate"), "应提取 correlate 调用");
        assert!(calls.contains(&"detect_peak"), "应提取 detect_peak 调用");
    }

    /// py_13: 组合提取 — 真实代码段的完整提取
    #[test]
    fn py_13_comprehensive_extraction() {
        let content = "\
import numpy as np
from math import sqrt

SAMPLE_RATE = 48000
FFT_SIZE = 1024

@dataclass
class SyncConfig:
    threshold: float
    window_size: int = 64

class CoarseSync:
    def __init__(self, config: SyncConfig):
        self.config = config
        self.buffer = []

    def correlate(self, samples):
        corr = np.convolve(samples, samples)
        return corr

    def detect_peak(self, corr) -> int:
        peak = np.argmax(corr)
        return peak
";
        let results = PythonExtractor.extract(content);
        let symbols: Vec<&str> = results.iter().filter_map(|r| r.symbol.as_deref()).collect();

        // def/class
        assert!(symbols.contains(&"CoarseSync"), "应提取类名");
        assert!(symbols.contains(&"__init__"), "应提取 __init__");
        assert!(symbols.contains(&"correlate"), "应提取 correlate");

        // imports
        assert!(symbols.contains(&"np"), "应提取 import np");

        // constants
        assert!(symbols.contains(&"SAMPLE_RATE"), "应提取常量");

        // dataclass fields
        assert!(symbols.contains(&"threshold"), "应提取 dataclass 字段");
        assert!(symbols.contains(&"window_size"), "应提取 dataclass 字段");

        // self fields
        assert!(symbols.contains(&"config"), "应提取 self.config");
        assert!(symbols.contains(&"buffer"), "应提取 self.buffer");

        // return type
        assert!(symbols.contains(&"int"), "应提取 int 返回类型");

        // call sites
        assert!(symbols.contains(&"convolve"), "应提取 np.convolve 调用");
        assert!(symbols.contains(&"argmax"), "应提取 np.argmax 调用");

        // All line_ranges valid
        for r in &results {
            assert!(r.line_range.start >= 1, "start >= 1: {:?}", r);
            assert!(r.line_range.start <= r.line_range.end, "start <= end: {:?}", r);
        }
    }
}
