/// Verilog 提取器 — module / endmodule 提取
///
/// 最小规则：
/// - 提取 `module <name>` 到对应 `endmodule`
/// - symbol = module 名, strength = Direct
/// - 不提取 assign/input/output 为独立 evidence item（Phase 2 最小实现不做）
/// - 无 endmodule 时 range 到 EOF，仍标 Direct（module 声明本身是直接证据）
/// - 注释行（// 或 /* 开头）中的 module 不提取
/// - 只有 assign 无 module 返回空列表
///
/// 配对算法：对每个 module，找到位于该 module 行之后第一个未使用的 endmodule。
/// 这保证 line_range.start <= line_range.end。
/// 孤立 endmodule（在第一个 module 之前）被自动跳过。
/// 多行块注释（`/* ... */`）内的 endmodule 不被收集。

use super::{extract_lines_range, is_hdl_comment_line, strip_line_comment, EvidenceExtractor};
use crate::evidence::models::{EvidenceStrength, LineRange, RawExtraction};

pub struct VerilogExtractor;

/// module 起始条目
struct ModuleEntry {
    /// 0-based 行索引
    line_idx: usize,
    /// module 名
    name: String,
}

impl EvidenceExtractor for VerilogExtractor {
    fn extract(&self, content: &str) -> Vec<RawExtraction> {
        if content.is_empty() {
            return vec![];
        }

        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return vec![];
        }

        let mut modules: Vec<ModuleEntry> = vec![];
        let mut endmodules: Vec<usize> = vec![]; // 0-based line indices
        let mut in_block_comment = false;

        for (i, line) in lines.iter().enumerate() {
            // 多行块注释跟踪
            if in_block_comment {
                if line.contains("*/") {
                    in_block_comment = false;
                }
                continue;
            }
            if is_hdl_comment_line(line) {
                // 单行 /* ... */ 不进入块注释状态
                let trimmed = line.trim_start();
                if trimmed.starts_with("/*") && !line.contains("*/") {
                    in_block_comment = true;
                }
                continue;
            }

            let stripped = strip_line_comment(line);
            let trimmed = stripped.trim();

            // 匹配 module <name>
            if let Some(rest) = trimmed.strip_prefix("module ") {
                let rest = rest.trim_start();
                // module 名到第一个非标识符字符
                let name_end = rest
                    .find(|c: char| !c.is_alphanumeric() && c != '_')
                    .unwrap_or(rest.len());
                let name = &rest[..name_end];
                if !name.is_empty() {
                    modules.push(ModuleEntry {
                        line_idx: i,
                        name: name.to_string(),
                    });
                    continue;
                }
            }

            // 匹配 endmodule
            if trimmed.starts_with("endmodule") {
                let after = trimmed.strip_prefix("endmodule").unwrap_or("");
                // endmodule 后应该跟空白、注释或行尾
                if after.is_empty()
                    || after.starts_with(' ')
                    || after.starts_with('\t')
                    || after.starts_with("//")
                {
                    endmodules.push(i);
                }
            }
        }

        let total_lines = lines.len();
        let mut results = vec![];

        // 游标式配对：对每个 module，找其之后第一个未使用的 endmodule
        let mut end_cursor: usize = 0;

        for module in &modules {
            let start = (module.line_idx + 1) as u32; // 1-based

            // 找第一个 line_idx > module.line_idx 的未使用 endmodule
            let end_0based = endmodules[end_cursor..]
                .iter()
                .position(|&line| line > module.line_idx)
                .map(|pos| {
                    let abs = end_cursor + pos;
                    end_cursor = abs + 1;
                    endmodules[abs]
                })
                .unwrap_or(total_lines - 1);

            // end_1based 至少等于 start（安全兜底）
            let end_1based = ((end_0based + 1) as u32).max(start);

            let raw_excerpt = extract_lines_range(content, start, end_1based);

            results.push(RawExtraction {
                symbol: Some(module.name.clone()),
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

    /// 辅助：断言所有 extraction 的 line_range 合法
    fn assert_valid_ranges(results: &[RawExtraction]) {
        for r in results {
            assert!(
                r.line_range.start <= r.line_range.end,
                "illegal range: start={} > end={} for symbol {:?}",
                r.line_range.start,
                r.line_range.end,
                r.symbol,
            );
        }
    }

    #[test]
    fn vlg_01_single_module() {
        let content = "module top(\n    input clk,\n    output data\n);\nwire x;\nassign data = x;\nendmodule";
        let results = VerilogExtractor.extract(content);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol.as_deref(), Some("top"));
        assert_eq!(results[0].line_range, LineRange { start: 1, end: 7 });
        assert_eq!(results[0].strength, EvidenceStrength::Direct);
        assert_valid_ranges(&results);
    }

    #[test]
    fn vlg_02_multiple_modules() {
        let content = "module alu(\n    input a\n);\nendmodule\n\nmodule top(\n    input clk\n);\nendmodule";
        let results = VerilogExtractor.extract(content);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].symbol.as_deref(), Some("alu"));
        assert_eq!(results[0].line_range, LineRange { start: 1, end: 4 });
        assert_eq!(results[1].symbol.as_deref(), Some("top"));
        assert_eq!(results[1].line_range, LineRange { start: 6, end: 9 });
        assert_valid_ranges(&results);
    }

    #[test]
    fn vlg_03_no_endmodule() {
        // 无 endmodule → range 到 EOF，仍标 Direct
        let content = "module incomplete(\n    input clk\n);";
        let results = VerilogExtractor.extract(content);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol.as_deref(), Some("incomplete"));
        assert_eq!(results[0].line_range, LineRange { start: 1, end: 3 });
        assert_eq!(results[0].strength, EvidenceStrength::Direct);
        assert_valid_ranges(&results);
    }

    #[test]
    fn vlg_04_comment_module_not_extracted() {
        let content = "// module fake_top(\n// );\n// endmodule\nmodule real_one();\nendmodule";
        let results = VerilogExtractor.extract(content);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol.as_deref(), Some("real_one"));
        assert_eq!(results[0].line_range, LineRange { start: 4, end: 5 });
        assert_valid_ranges(&results);
    }

    #[test]
    fn vlg_05_only_assign_no_module() {
        let content = "assign x = 1;\nassign y = 2;";
        let results = VerilogExtractor.extract(content);
        assert!(results.is_empty(), "only assign, no module → empty");
    }

    #[test]
    fn vlg_06_block_comment_with_endmodule_before_real_module() {
        // 块注释内 endmodule 不应影响后面真实 module 的配对
        let content = "/*\n  endmodule\n*/\nmodule real_one();\nendmodule";
        let results = VerilogExtractor.extract(content);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol.as_deref(), Some("real_one"));
        // module at idx 3 (1-based=4), endmodule at idx 4 (1-based=5)
        assert_eq!(results[0].line_range, LineRange { start: 4, end: 5 });
        assert_valid_ranges(&results);
    }

    #[test]
    fn vlg_07_orphan_endmodule_before_real_module() {
        // 文件开头有孤立 endmodule，后面有真实 module/endmodule
        let content = "endmodule\n\nmodule foo(\n    input clk\n);\nendmodule";
        let results = VerilogExtractor.extract(content);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol.as_deref(), Some("foo"));
        // module at idx 2 (1-based=3), endmodule at idx 5 (1-based=6)
        assert_eq!(results[0].line_range, LineRange { start: 3, end: 6 });
        assert_valid_ranges(&results);
    }
}
