/// SystemVerilog 提取器 — module / interface / package / class 提取
///
/// 最小规则：
/// - 提取 module/endmodule, interface/endinterface, package/endpackage, class/endclass
/// - symbol = 块名称, strength = Direct
/// - 每种块类型独立按出现顺序配对（module 配 endmodule, interface 配 endinterface, etc.）
/// - 注释行中的关键字不提取
/// - 无 end 关键字时 range 到 EOF
/// - 不处理嵌套同名块（Phase 2 最小实现，实际 FPGA 代码极少嵌套同名块）

use super::{extract_lines_range, is_hdl_comment_line, strip_line_comment, EvidenceExtractor};
use crate::evidence::models::{EvidenceStrength, LineRange, RawExtraction};

pub struct SystemVerilogExtractor;

/// 通用块类型
struct BlockEntry {
    /// 0-based 行索引
    line_idx: usize,
    /// 块名称
    name: String,
}

/// 从 content 中提取指定 start_kw / end_kw 对应的所有块
fn find_blocks(content: &str, start_kw: &str, end_kw: &str) -> Vec<RawExtraction> {
    if content.is_empty() {
        return vec![];
    }

    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return vec![];
    }

    let prefix = format!("{} ", start_kw);
    let mut starts: Vec<BlockEntry> = vec![];
    let mut ends: Vec<usize> = vec![];

    for (i, line) in lines.iter().enumerate() {
        if is_hdl_comment_line(line) {
            continue;
        }

        let stripped = strip_line_comment(line);
        let trimmed = stripped.trim();

        // 匹配 start_kw <name>
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            let rest = rest.trim_start();
            let name_end = rest
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(rest.len());
            let name = &rest[..name_end];
            if !name.is_empty() {
                starts.push(BlockEntry {
                    line_idx: i,
                    name: name.to_string(),
                });
                continue;
            }
        }

        // 匹配 end_kw
        if trimmed.starts_with(end_kw) {
            let after = trimmed.strip_prefix(end_kw).unwrap_or("");
            if after.is_empty()
                || after.starts_with(' ')
                || after.starts_with('\t')
                || after.starts_with("//")
            {
                ends.push(i);
            }
        }
    }

    let total_lines = lines.len();
    let mut results = vec![];

    for (idx, block) in starts.iter().enumerate() {
        let start_1based = (block.line_idx + 1) as u32;

        let end_0based = if idx < ends.len() {
            ends[idx]
        } else {
            total_lines - 1
        };
        let end_1based = (end_0based + 1) as u32;

        let raw_excerpt = extract_lines_range(content, start_1based, end_1based);

        results.push(RawExtraction {
            symbol: Some(block.name.clone()),
            line_range: LineRange {
                start: start_1based,
                end: end_1based,
            },
            raw_excerpt,
            strength: EvidenceStrength::Direct,
        });
    }

    results
}

impl EvidenceExtractor for SystemVerilogExtractor {
    fn extract(&self, content: &str) -> Vec<RawExtraction> {
        let mut results = vec![];
        results.extend(find_blocks(content, "module", "endmodule"));
        results.extend(find_blocks(content, "interface", "endinterface"));
        results.extend(find_blocks(content, "package", "endpackage"));
        results.extend(find_blocks(content, "class", "endclass"));
        // 按行号排序，保证结果有序
        results.sort_by_key(|r| r.line_range.start);
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sv_01_module() {
        let content = "module top(\n    input logic clk\n);\nendmodule";
        let results = SystemVerilogExtractor.extract(content);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol.as_deref(), Some("top"));
        assert_eq!(results[0].line_range, LineRange { start: 1, end: 4 });
        assert_eq!(results[0].strength, EvidenceStrength::Direct);
    }

    #[test]
    fn sv_02_interface() {
        let content = "interface bus_if(\n    logic clk\n);\nendinterface";
        let results = SystemVerilogExtractor.extract(content);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol.as_deref(), Some("bus_if"));
        assert_eq!(results[0].line_range, LineRange { start: 1, end: 4 });
    }

    #[test]
    fn sv_03_package() {
        let content = "package pkg;\n    parameter WIDTH = 8;\nendpackage";
        let results = SystemVerilogExtractor.extract(content);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol.as_deref(), Some("pkg"));
        assert_eq!(results[0].line_range, LineRange { start: 1, end: 3 });
    }

    #[test]
    fn sv_04_class() {
        let content = "class Packet;\n    logic [7:0] data;\nendclass";
        let results = SystemVerilogExtractor.extract(content);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol.as_deref(), Some("Packet"));
        assert_eq!(results[0].line_range, LineRange { start: 1, end: 3 });
    }

    #[test]
    fn sv_05_multiple_blocks_and_comments() {
        let content = "\
package my_pkg;
    parameter W = 8;
endpackage

// module should_be_skipped

module alu(
    input logic clk
);
endmodule

interface bus;
    logic clk;
endinterface";
        let results = SystemVerilogExtractor.extract(content);
        // 结果按行号排序
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].symbol.as_deref(), Some("my_pkg"));
        assert_eq!(results[0].line_range, LineRange { start: 1, end: 3 });
        assert_eq!(results[1].symbol.as_deref(), Some("alu"));
        assert_eq!(results[1].line_range, LineRange { start: 7, end: 10 });
        assert_eq!(results[2].symbol.as_deref(), Some("bus"));
        assert_eq!(results[2].line_range, LineRange { start: 12, end: 14 });
    }
}
