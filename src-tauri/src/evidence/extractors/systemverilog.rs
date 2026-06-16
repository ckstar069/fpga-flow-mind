/// SystemVerilog 提取器 — module / interface / package / class / port / signal / assign / always / instance / parameter 提取
///
/// 规则：
/// - 提取 module/endmodule, interface/endinterface, package/endpackage, class/endclass
/// - symbol = 块名称, strength = Direct
/// - 每种块类型独立按出现顺序配对（module 配 endmodule, interface 配 endinterface, etc.）
/// - 注释行中的关键字不提取
/// - 无 end 关键字时 range 到 EOF
/// - 不处理嵌套同名块（Phase 2 最小实现，实际 FPGA 代码极少嵌套同名块）
/// - port 声明: `input <type> <name>` / `output <type> <name>` / `inout <type> <name>` → symbol=端口名, Direct
/// - logic/wire/reg 声明 → symbol=信号名, Direct
/// - assign 语句 → symbol=赋值目标名, Direct
/// - always 块 → symbol=块标识, Direct
/// - 模块实例化 → symbol=实例名, Direct
/// - parameter/localparam → symbol=参数名, Direct
///
/// 配对算法：对每个 start 块，找到位于该 start 行之后第一个未使用的 end 关键字。
/// 这保证 line_range.start <= line_range.end。
/// 孤立 end 关键字（在第一个 start 之前）被自动跳过。
/// 多行块注释（`/* ... */`）内的关键字不被收集。

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


impl SystemVerilogExtractor {
    /// 提取行内端口声明
    fn extract_port(&self, line_idx: usize, trimmed: &str) -> Option<RawExtraction> {
        let lower = trimmed.to_lowercase();
        let direction = if lower.starts_with("input ") { "input" }
            else if lower.starts_with("output ") { "output" }
            else if lower.starts_with("inout ") { "inout" }
            else { return None; };

        let after_dir = trimmed[direction.len()..].trim();
        let after_type = if after_dir.to_lowercase().starts_with("reg ")
            || after_dir.to_lowercase().starts_with("wire ")
            || after_dir.to_lowercase().starts_with("logic ")
        {
            let space = after_dir.find(' ')?;
            after_dir[space + 1..].trim()
        } else {
            after_dir
        };

        let after_width = if after_type.starts_with('[') {
            let close = after_type.find(']')?;
            after_type[close + 1..].trim()
        } else {
            after_type
        };

        let name_end = after_width.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(after_width.len());
        let name = &after_width[..name_end];
        if name.is_empty() || name.chars().next()?.is_digit(10) { return None; }

        Some(RawExtraction {
            symbol: Some(name.to_string()),
            line_range: LineRange { start: (line_idx + 1) as u32, end: (line_idx + 1) as u32 },
            raw_excerpt: trimmed.to_string(),
            strength: EvidenceStrength::Direct,
        })
    }

    /// 提取 wire/reg/logic 声明（排除端口方向行）
    fn extract_signal_decl(&self, line_idx: usize, trimmed: &str) -> Option<RawExtraction> {
        let lower = trimmed.to_lowercase();
        if lower.starts_with("input ") || lower.starts_with("output ") || lower.starts_with("inout ") {
            return None;
        }

        let kw = if trimmed.starts_with("wire ") { "wire" }
            else if trimmed.starts_with("reg ") { "reg" }
            else if trimmed.starts_with("logic ") { "logic" }
            else { return None; };

        let after_kw = trimmed[kw.len()..].trim();
        let after_width = if after_kw.starts_with('[') {
            let close = after_kw.find(']')?;
            after_kw[close + 1..].trim()
        } else { after_kw };

        let name_end = after_width.find(|c: char| !c.is_alphanumeric() && c != '_' && c != '$').unwrap_or(after_width.len());
        let name = &after_width[..name_end];
        if name.is_empty() || name.chars().next()?.is_digit(10) { return None; }

        Some(RawExtraction {
            symbol: Some(name.to_string()),
            line_range: LineRange { start: (line_idx + 1) as u32, end: (line_idx + 1) as u32 },
            raw_excerpt: trimmed.to_string(),
            strength: EvidenceStrength::Direct,
        })
    }

    /// 提取 assign 语句
    fn extract_assign(&self, line_idx: usize, trimmed: &str) -> Option<RawExtraction> {
        if !trimmed.starts_with("assign ") { return None; }
        let after = trimmed[7..].trim();
        if let Some(eq_pos) = after.find(" = ") {
            let name = after[..eq_pos].trim();
            if !name.is_empty() && !name.chars().next()?.is_digit(10) {
                return Some(RawExtraction {
                    symbol: Some(name.to_string()),
                    line_range: LineRange { start: (line_idx + 1) as u32, end: (line_idx + 1) as u32 },
                    raw_excerpt: trimmed.to_string(),
                    strength: EvidenceStrength::Direct,
                });
            }
        }
        None
    }

    /// 提取 always 块头
    fn extract_always(&self, line_idx: usize, trimmed: &str) -> Option<RawExtraction> {
        let lower = trimmed.to_lowercase();
        let symbol = if lower.starts_with("always_ff ") || lower.starts_with("always_ff(") {
            "always_ff"
        } else if lower.starts_with("always_comb") {
            "always_comb"
        } else if lower.starts_with("always_latch") {
            "always_latch"
        } else if lower.starts_with("always @") || lower.starts_with("always@") {
            "always"
        } else { return None; };

        Some(RawExtraction {
            symbol: Some(symbol.to_string()),
            line_range: LineRange { start: (line_idx + 1) as u32, end: (line_idx + 1) as u32 },
            raw_excerpt: trimmed.to_string(),
            strength: EvidenceStrength::Direct,
        })
    }

    /// 提取模块实例化（同 Verilog 逻辑）
    fn extract_instance(&self, line_idx: usize, trimmed: &str) -> Option<RawExtraction> {
        if trimmed.starts_with("module ") || trimmed.starts_with("endmodule")
            || trimmed.starts_with("interface ") || trimmed.starts_with("endinterface")
            || trimmed.starts_with("package ") || trimmed.starts_with("endpackage")
            || trimmed.starts_with("class ") || trimmed.starts_with("endclass")
            || trimmed.starts_with("input ") || trimmed.starts_with("output ")
            || trimmed.starts_with("inout ") || trimmed.starts_with("wire ")
            || trimmed.starts_with("reg ") || trimmed.starts_with("logic ")
            || trimmed.starts_with("assign ") || trimmed.starts_with("always")
            || trimmed.starts_with("parameter ") || trimmed.starts_with("localparam ")
            || trimmed.starts_with("initial ") || trimmed.starts_with("//") || trimmed.starts_with("/*")
        { return None; }

        if !trimmed.contains('(') { return None; }
        if trimmed.starts_with("if ") || trimmed.starts_with("for ") || trimmed.starts_with("while ")
            || trimmed.starts_with("case ") || trimmed.starts_with("begin") || trimmed.starts_with("end")
            || trimmed.starts_with("else") || trimmed.starts_with("repeat") || trimmed.starts_with("forever")
            || trimmed.starts_with("function") || trimmed.starts_with("task ")
            || trimmed.starts_with("generate") || trimmed.starts_with("endgenerate")
        { return None; }

        let words: Vec<&str> = trimmed.split_whitespace().collect();
        if words.len() < 2 { return None; }
        let inst_idx = if words.get(1).map_or(false, |w| w.starts_with('#')) { 2 } else { 1 };
        let inst_word = words.get(inst_idx)?;
        let inst_name = inst_word.trim_end_matches(|c: char| c == '(' || c == ',');
        if inst_name.is_empty() || inst_name.chars().next()?.is_digit(10) { return None; }
        if !inst_name.chars().all(|c| c.is_alphanumeric() || c == '_') { return None; }

        Some(RawExtraction {
            symbol: Some(inst_name.to_string()),
            line_range: LineRange { start: (line_idx + 1) as u32, end: (line_idx + 1) as u32 },
            raw_excerpt: trimmed.to_string(),
            strength: EvidenceStrength::Direct,
        })
    }

    /// 提取参数声明
    fn extract_param(&self, line_idx: usize, trimmed: &str) -> Option<RawExtraction> {
        let stripped = if trimmed.starts_with("parameter ") { trimmed[9..].trim() }
            else if trimmed.starts_with("localparam ") { trimmed[10..].trim() }
            else { return None; };

        let after_type = if stripped.starts_with("integer ") || stripped.starts_with("real ")
            || stripped.starts_with("time ") || stripped.starts_with("realtime ")
        {
            let space = stripped.find(' ')?;
            stripped[space + 1..].trim()
        } else { stripped };

        let name_end = after_type.find(|c: char| c == '=' || c == ',' || c == ')' || c == ' ' || c == '\t')
            .unwrap_or(after_type.len());
        let name = after_type[..name_end].trim();
        if name.is_empty() || name.chars().next()?.is_digit(10) { return None; }

        Some(RawExtraction {
            symbol: Some(name.to_string()),
            line_range: LineRange { start: (line_idx + 1) as u32, end: (line_idx + 1) as u32 },
            raw_excerpt: trimmed.to_string(),
            strength: EvidenceStrength::Direct,
        })
    }
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
    let mut in_block_comment = false;

    for (i, line) in lines.iter().enumerate() {
        if in_block_comment {
            if line.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }
        if is_hdl_comment_line(line) {
            let trimmed = line.trim_start();
            if trimmed.starts_with("/*") && !line.contains("*/") {
                in_block_comment = true;
            }
            continue;
        }

        let stripped = strip_line_comment(line);
        let trimmed = stripped.trim();

        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            let rest = rest.trim_start();
            let name_end = rest
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(rest.len());
            let name = &rest[..name_end];
            if !name.is_empty() {
                starts.push(BlockEntry { line_idx: i, name: name.to_string() });
                continue;
            }
        }

        if trimmed.starts_with(end_kw) {
            let after = trimmed.strip_prefix(end_kw).unwrap_or("");
            if after.is_empty() || after.starts_with(' ') || after.starts_with('\t') || after.starts_with("//") {
                ends.push(i);
            }
        }
    }

    let total_lines = lines.len();
    let mut results = vec![];
    let mut end_cursor: usize = 0;

    for block in &starts {
        let start_1based = (block.line_idx + 1) as u32;
        let end_0based = ends[end_cursor..]
            .iter()
            .position(|&line| line > block.line_idx)
            .map(|pos| { let abs = end_cursor + pos; end_cursor = abs + 1; ends[abs] })
            .unwrap_or(total_lines - 1);
        let end_1based = ((end_0based + 1) as u32).max(start_1based);
        let raw_excerpt = extract_lines_range(content, start_1based, end_1based);

        results.push(RawExtraction {
            symbol: Some(block.name.clone()),
            line_range: LineRange { start: start_1based, end: end_1based },
            raw_excerpt,
            strength: EvidenceStrength::Direct,
        });
    }

    results
}

impl EvidenceExtractor for SystemVerilogExtractor {
    fn extract(&self, content: &str) -> Vec<RawExtraction> {
        if content.is_empty() { return vec![]; }

        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() { return vec![]; }

        // ── Pass 1: 块级提取（module/interface/package/class） ──
        let mut results = vec![];
        results.extend(find_blocks(content, "module", "endmodule"));
        results.extend(find_blocks(content, "interface", "endinterface"));
        results.extend(find_blocks(content, "package", "endpackage"));
        results.extend(find_blocks(content, "class", "endclass"));

        // ── 从块级结果构建行级边界索引 ──
        struct Range { start_idx: usize, end_idx: usize }
        let mut block_ranges: Vec<Range> = vec![];
        for r in &results {
            if let Some(sym) = &r.symbol {
                for (i, line) in lines.iter().enumerate() {
                    let trimmed = strip_line_comment(line).trim();
                    if trimmed.contains(sym) && !trimmed.starts_with("//") {
                        let start_idx = i;
                        let end_idx = r.line_range.end as usize;
                        if end_idx > start_idx {
                            block_ranges.push(Range { start_idx, end_idx: end_idx.min(lines.len()) });
                        }
                        break;
                    }
                }
            }
        }

        // ── Pass 2: 行级提取 ──
        let mut in_block_comment = false;
        let mut line_results: Vec<RawExtraction> = vec![];

        for (i, line) in lines.iter().enumerate() {
            if in_block_comment {
                if line.contains("*/") { in_block_comment = false; }
                continue;
            }
            if is_hdl_comment_line(line) {
                let trimmed = line.trim_start();
                if trimmed.starts_with("/*") && !line.contains("*/") { in_block_comment = true; }
                continue;
            }

            let trimmed_raw = strip_line_comment(line);
            let trimmed = trimmed_raw.trim();
            if trimmed.is_empty() { continue; }

            // Skip block header/footer lines
            let skip_kws = ["module ", "endmodule", "interface ", "endinterface",
                "package ", "endpackage", "class ", "endclass"];
            if skip_kws.iter().any(|kw| trimmed.starts_with(kw)) { continue; }

            // Check if inside any block
            let inside_any = block_ranges.iter().any(|r| i > r.start_idx && i < r.end_idx);

            // Only extract line-level items if there are blocks and we're inside one
            if !block_ranges.is_empty() && !inside_any { continue; }

            let mut extracted: Option<RawExtraction> = None;
            extracted = extracted.or_else(|| self.extract_port(i, trimmed));
            extracted = extracted.or_else(|| self.extract_param(i, trimmed));
            extracted = extracted.or_else(|| self.extract_always(i, trimmed));
            extracted = extracted.or_else(|| self.extract_assign(i, trimmed));
            extracted = extracted.or_else(|| self.extract_signal_decl(i, trimmed));
            extracted = extracted.or_else(|| self.extract_instance(i, trimmed));

            if let Some(ext) = extracted { line_results.push(ext); }
        }

        // ── 合并 ──
        results.extend(line_results);
        results.sort_by_key(|r| r.line_range.start);
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
    fn sv_01_module() {
        let content = "module top(\n    input logic clk\n);\nendmodule";
        let results = SystemVerilogExtractor.extract(content);
        // Now extracts module + port/signal inside
        assert!(results.len() >= 1, "应有至少 1 项");
        assert!(results.iter().any(|r| r.symbol.as_deref() == Some("top")));
        assert_eq!(results[0].line_range, LineRange { start: 1, end: 4 });
        assert_eq!(results[0].strength, EvidenceStrength::Direct);
        assert_valid_ranges(&results);
    }

    #[test]
    fn sv_02_interface() {
        let content = "interface bus_if(\n    logic clk\n);\nendinterface";
        let results = SystemVerilogExtractor.extract(content);
        // interface(bus_if) + signal(clk) = 2 items
        assert!(results.len() >= 2, "应有 interface + signal, 实际 {}", results.len());
        assert!(results.iter().any(|r| r.symbol.as_deref() == Some("bus_if")), "应包含 bus_if");
        assert_valid_ranges(&results);
    }

    #[test]
    fn sv_03_package() {
        let content = "package pkg;\n    parameter WIDTH = 8;\nendpackage";
        let results = SystemVerilogExtractor.extract(content);
        // package(pkg) + param(WIDTH) = 2 items
        assert!(results.len() >= 2, "应有 package + param, 实际 {}", results.len());
        assert!(results.iter().any(|r| r.symbol.as_deref() == Some("pkg")), "应包含 pkg");
        assert!(results.iter().any(|r| r.symbol.as_deref() == Some("WIDTH")), "应包含 WIDTH");
        assert_valid_ranges(&results);
    }

    #[test]
    fn sv_04_class() {
        let content = "class Packet;\n    logic [7:0] data;\nendclass";
        let results = SystemVerilogExtractor.extract(content);
        // class(Packet) + signal(data) = 2 items
        assert!(results.len() >= 2, "应有 class + signal, 实际 {}", results.len());
        assert!(results.iter().any(|r| r.symbol.as_deref() == Some("Packet")), "应包含 Packet");
        assert!(results.iter().any(|r| r.symbol.as_deref() == Some("data")), "应包含 data");
        assert_valid_ranges(&results);
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
        // block results + line-level (param(W), port(clk), signal(clk))
        let block_symbols: Vec<&str> = results.iter().filter_map(|r| r.symbol.as_deref()).collect();
        assert!(block_symbols.contains(&"my_pkg"), "应包含 my_pkg");
        assert!(block_symbols.contains(&"alu"), "应包含 alu");
        assert!(block_symbols.contains(&"bus"), "应包含 bus");
        assert!(block_symbols.contains(&"W"), "应包含 parameter W");
        assert_valid_ranges(&results);
    }

    #[test]
    fn sv_06_orphan_endinterface_before_real_interface() {
        // 孤立 endinterface 在文件开头，后面有真实 interface/endinterface
        let content = "endinterface\n\ninterface my_bus(\n    logic clk\n);\nendinterface";
        let results = SystemVerilogExtractor.extract(content);
        // interface(my_bus) + signal(clk) = 2 items
        assert!(results.len() >= 2, "应有 interface + signal, 实际 {}", results.len());
        assert!(results.iter().any(|r| r.symbol.as_deref() == Some("my_bus")), "应包含 my_bus");
        assert_valid_ranges(&results);
    }

    #[test]
    fn sv_07_block_comment_with_endmodule_before_real_module() {
        // 块注释内 endmodule 不应影响后面真实 module 的配对
        let content = "/*\n  endmodule\n*/\nmodule real_mod();\nendmodule";
        let results = SystemVerilogExtractor.extract(content);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol.as_deref(), Some("real_mod"));
        // module at idx 3 (1-based=4), endmodule at idx 4 (1-based=5)
        assert_eq!(results[0].line_range, LineRange { start: 4, end: 5 });
        assert_valid_ranges(&results);
    }

    // ─── P1: 新增 SV 行级提取测试 ─────────────────────────────────

    /// sv_08: 端口提取
    #[test]
    fn sv_08_port_extraction() {
        let content = "\
module sv_top(
    input logic clk,
    input logic rst_n,
    input [7:0] data_in,
    output logic [15:0] result
);
endmodule
";
        let results = SystemVerilogExtractor.extract(content);
        let port_names: Vec<&str> = results.iter()
            .filter_map(|r| r.symbol.as_deref())
            .filter(|s| *s == "clk" || *s == "rst_n" || *s == "data_in" || *s == "result")
            .collect();
        assert!(port_names.contains(&"clk"), "应提取 clk 端口");
        assert!(port_names.contains(&"rst_n"), "应提取 rst_n 端口");
        assert!(port_names.contains(&"data_in"), "应提取 data_in 端口");
        assert!(port_names.contains(&"result"), "应提取 result 端口");
        assert_valid_ranges(&results);
    }

    /// sv_09: logic/wire/reg 信号声明
    #[test]
    fn sv_09_signal_decl_extraction() {
        let content = "\
module top(input clk);
    logic [7:0] data_bus;
    reg  [3:0] counter;
    wire enable;
endmodule
";
        let results = SystemVerilogExtractor.extract(content);
        let sig_names: Vec<&str> = results.iter()
            .filter_map(|r| r.symbol.as_deref())
            .filter(|s| *s == "data_bus" || *s == "counter" || *s == "enable")
            .collect();
        assert!(sig_names.contains(&"data_bus"), "应提取 logic data_bus");
        assert!(sig_names.contains(&"counter"), "应提取 reg counter");
        assert!(sig_names.contains(&"enable"), "应提取 wire enable");
        assert_valid_ranges(&results);
    }

    /// sv_10: always_ff/always_comb 提取
    #[test]
    fn sv_10_always_extraction() {
        let content = "\
module top(input logic clk, input logic rst_n, output logic q);
    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) q <= '0;
        else q <= ~q;
    end
    always_comb begin
        q = ~q;
    end
endmodule
";
        let results = SystemVerilogExtractor.extract(content);
        let always_symbols: Vec<&str> = results.iter()
            .filter_map(|r| r.symbol.as_deref())
            .filter(|s| *s == "always_ff" || *s == "always_comb")
            .collect();
        assert!(always_symbols.contains(&"always_ff"), "应提取 always_ff");
        assert!(always_symbols.contains(&"always_comb"), "应提取 always_comb");
        assert_valid_ranges(&results);
    }

    /// sv_11: 参数提取
    #[test]
    fn sv_11_parameter_extraction() {
        let content = "\
module top();
    parameter WIDTH = 8;
    localparam DEPTH = 64;
endmodule
";
        let results = SystemVerilogExtractor.extract(content);
        let param_names: Vec<&str> = results.iter()
            .filter_map(|r| r.symbol.as_deref())
            .filter(|s| *s == "WIDTH" || *s == "DEPTH")
            .collect();
        assert!(param_names.contains(&"WIDTH"), "应提取 parameter WIDTH");
        assert!(param_names.contains(&"DEPTH"), "应提取 localparam DEPTH");
        assert_valid_ranges(&results);
    }

    /// sv_12: 模块实例化提取
    #[test]
    fn sv_12_instance_extraction() {
        let content = "\
module top(input clk, input [7:0] data);
    adder #(.WIDTH(8)) u_adder (
        .a(data),
        .b(data),
        .sum(sum)
    );
endmodule
";
        let results = SystemVerilogExtractor.extract(content);
        let inst_names: Vec<&str> = results.iter()
            .filter_map(|r| r.symbol.as_deref())
            .filter(|s| *s == "u_adder")
            .collect();
        assert!(inst_names.contains(&"u_adder"), "应提取 u_adder 实例");
        assert_valid_ranges(&results);
    }

    /// sv_13: 综合测试
    #[test]
    fn sv_13_comprehensive_extraction() {
        let content = "\
module coarse_sync (
    input logic clk,
    input logic rst_n,
    input [11:0] rx_data,
    output logic [11:0] peak
);
    parameter THRESHOLD = 1000;

    logic [11:0] corr;
    logic valid;

    assign peak = corr;

    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) corr <= '0;
        else corr <= corr + rx_data;
    end

    correlator #(.WIDTH(12)) u_corr (
        .data(rx_data),
        .result(corr)
    );
endmodule
";
        let results = SystemVerilogExtractor.extract(content);
        let symbols: Vec<&str> = results.iter().filter_map(|r| r.symbol.as_deref()).collect();
        assert!(symbols.contains(&"coarse_sync"), "应提取 module");
        assert!(symbols.contains(&"clk"), "应提取 clk");
        assert!(symbols.contains(&"rx_data"), "应提取 rx_data");
        assert!(symbols.contains(&"THRESHOLD"), "应提取 parameter");
        assert!(symbols.contains(&"corr"), "应提取 logic corr");
        assert!(symbols.contains(&"valid"), "应提取 logic valid");
        assert!(symbols.contains(&"peak"), "应提取 assign peak");
        assert!(symbols.contains(&"always_ff"), "应提取 always_ff");
        assert!(symbols.contains(&"u_corr"), "应提取实例");
        assert_valid_ranges(&results);
    }
}
