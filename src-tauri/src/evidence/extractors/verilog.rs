/// Verilog 提取器 — module / port / signal / assign / always / instance / parameter 提取
///
/// 规则：
/// - module/endmodule 配对提取（现有逻辑）
/// - port 声明: `input <width> <name>` / `output <width> <name>` / `inout <width> <name>` → symbol=端口名, Direct
/// - wire/reg 声明: `wire <name>` / `reg <name>` / `logic <name>` → symbol=信号名, Direct
/// - assign 语句: `assign <name> = <expr>` → symbol=赋值目标名, Direct
/// - always 块: `always @(...)` / `always_ff @(...)` / `always_comb` → symbol=块标识, Direct
/// - 模块实例化: `<module_type> <inst_name> (...)` → symbol=实例名, Direct
/// - 参数: `parameter <name> = <value>` / `localparam <name> = <value>` → symbol=参数名, Direct
/// - 注释行中的关键字不提取
/// - 多行块注释（`/* ... */`）内的关键字不被收集

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

impl VerilogExtractor {
    /// 判断行是否在 module 体内（行 index 在某个 module 的 [idx+1, endmodule_idx) 区间内）
    fn is_inside_module(line_idx: usize, modules: &[ModuleEntry], endmodules: &[usize]) -> bool {
        for (i, m) in modules.iter().enumerate() {
            if line_idx <= m.line_idx {
                continue;
            }
            let end = endmodules.get(i).copied().unwrap_or(usize::MAX);
            if line_idx < end {
                return true;
            }
        }
        false
    }

    /// 提取行内端口声明：input/output/inout [reg/wire] <width> <name>
    fn extract_port(&self, line_idx: usize, trimmed: &str) -> Option<RawExtraction> {
        let lower = trimmed.to_lowercase();
        let direction = if lower.starts_with("input ") { "input" }
            else if lower.starts_with("output ") { "output" }
            else if lower.starts_with("inout ") { "inout" }
            else { return None; };

        // After direction keyword, skip optional reg/wire, then optional width, then identifier
        let after_dir = trimmed[direction.len()..].trim();

        // Skip optional "reg" or "wire" or "logic" type specifier after direction
        let after_type = if after_dir.to_lowercase().starts_with("reg ")
            || after_dir.to_lowercase().starts_with("wire ")
            || after_dir.to_lowercase().starts_with("logic ")
        {
            let space = after_dir.find(' ')?;
            after_dir[space + 1..].trim()
        } else {
            after_dir
        };

        // Skip width like [3:0] or [11:0]
        let after_width = if after_type.starts_with('[') {
            let close = after_type.find(']')?;
            after_type[close + 1..].trim()
        } else {
            after_type
        };

        // First identifier is the port name
        let name_end = after_width.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(after_width.len());
        let name = &after_width[..name_end];
        if name.is_empty() || name.chars().next()?.is_digit(10) {
            return None;
        }

        Some(RawExtraction {
            symbol: Some(name.to_string()),
            line_range: LineRange { start: (line_idx + 1) as u32, end: (line_idx + 1) as u32 },
            raw_excerpt: trimmed.to_string(),
            strength: EvidenceStrength::Direct,
        })
    }

    /// 提取 wire/reg/logic 声明（排除端口方向前缀的行）
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
        // Skip width like [7:0]
        let after_width = if after_kw.starts_with('[') {
            let close = after_kw.find(']')?;
            after_kw[close + 1..].trim()
        } else {
            after_kw
        };

        // First identifier (up to =, ;, comma, whitespace)
        let name_end = after_width.find(|c: char| !c.is_alphanumeric() && c != '_' && c != '$')
            .unwrap_or(after_width.len());
        let name = &after_width[..name_end];
        if name.is_empty() || name.chars().next()?.is_digit(10) {
            return None;
        }

        let start_1based = (line_idx + 1) as u32;
        Some(RawExtraction {
            symbol: Some(name.to_string()),
            line_range: LineRange { start: start_1based, end: start_1based },
            raw_excerpt: trimmed.to_string(),
            strength: EvidenceStrength::Direct,
        })
    }

    /// 提取 assign 语句
    fn extract_assign(&self, line_idx: usize, trimmed: &str) -> Option<RawExtraction> {
        if !trimmed.starts_with("assign ") {
            return None;
        }
        let after_assign = trimmed[7..].trim();
        // assign <name> = <expr>
        if let Some(eq_pos) = after_assign.find(" = ") {
            let name = after_assign[..eq_pos].trim();
            if !name.is_empty() && !name.chars().next()?.is_digit(10) {
                let start_1based = (line_idx + 1) as u32;
                return Some(RawExtraction {
                    symbol: Some(name.to_string()),
                    line_range: LineRange { start: start_1based, end: start_1based },
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
        } else {
            return None;
        };

        Some(RawExtraction {
            symbol: Some(symbol.to_string()),
            line_range: LineRange { start: (line_idx + 1) as u32, end: (line_idx + 1) as u32 },
            raw_excerpt: trimmed.to_string(),
            strength: EvidenceStrength::Direct,
        })
    }

    /// 提取模块实例化: <mod_type> <inst_name>(<ports>);
    /// 保守匹配：module 体内的行，包含 '(' 和 ')'，实例名是第一个非关键字非参数覆盖的标识符
    fn extract_instance(&self, line_idx: usize, trimmed: &str) -> Option<RawExtraction> {
        if trimmed.starts_with("module ") || trimmed.starts_with("endmodule")
            || trimmed.starts_with("input ") || trimmed.starts_with("output ")
            || trimmed.starts_with("inout ") || trimmed.starts_with("wire ")
            || trimmed.starts_with("reg ") || trimmed.starts_with("logic ")
            || trimmed.starts_with("assign ") || trimmed.starts_with("always")
            || trimmed.starts_with("parameter ") || trimmed.starts_with("localparam ")
            || trimmed.starts_with("initial ") || trimmed.starts_with("//")
            || trimmed.starts_with("/*")
        {
            return None;
        }

        // Must contain '(' indicating port connections start (ports may span multiple lines)
        if !trimmed.contains('(') {
            return None;
        }

        // Skip leading keywords: if/for/while/case/begin/end/else/repeat/etc.
        if trimmed.starts_with("if ") || trimmed.starts_with("for ") || trimmed.starts_with("while ")
            || trimmed.starts_with("case ") || trimmed.starts_with("begin") || trimmed.starts_with("end")
            || trimmed.starts_with("else") || trimmed.starts_with("repeat") || trimmed.starts_with("forever")
            || trimmed.starts_with("function") || trimmed.starts_with("task ")
            || trimmed.starts_with("generate") || trimmed.starts_with("endgenerate")
        {
            return None;
        }

        // Split by whitespace and look for instance name (first identifier after module type,
        // skipping optional #(...) parameter override)
        let words: Vec<&str> = trimmed.split_whitespace().collect();
        if words.len() < 2 {
            return None;
        }

        // If word[1] starts with '#', it's a parameter override; instance name is word[2]
        let inst_idx = if words.get(1).map_or(false, |w| w.starts_with('#')) { 2 } else { 1 };

        let inst_word = words.get(inst_idx)?;
        let inst_name = inst_word.trim_end_matches(|c: char| c == '(' || c == ',');

        if inst_name.is_empty() || inst_name.chars().next()?.is_digit(10) {
            return None;
        }

        // Instance name should be a valid identifier
        if !inst_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return None;
        }

        Some(RawExtraction {
            symbol: Some(inst_name.to_string()),
            line_range: LineRange { start: (line_idx + 1) as u32, end: (line_idx + 1) as u32 },
            raw_excerpt: trimmed.to_string(),
            strength: EvidenceStrength::Direct,
        })
    }

    /// 提取参数声明: parameter <name> = <value> 或 localparam <name> = <value>
    fn extract_param(&self, line_idx: usize, trimmed: &str) -> Option<RawExtraction> {
        let stripped = if trimmed.starts_with("parameter ") {
            trimmed[9..].trim()
        } else if trimmed.starts_with("localparam ") {
            trimmed[10..].trim()
        } else {
            return None;
        };

        // parameter type <name> = <value> 或 parameter <name> = <value>
        // Skip optional type keyword
        let after_type = if stripped.starts_with("integer ") || stripped.starts_with("real ")
            || stripped.starts_with("time ") || stripped.starts_with("realtime ")
        {
            // Find space after type keyword
            let space = stripped.find(' ')?;
            stripped[space + 1..].trim()
        } else {
            stripped
        };

        // Get name before '='
        let name_end = after_type.find(|c: char| c == '=' || c == ',' || c == ')' || c == ' ' || c == '\t')
            .unwrap_or(after_type.len());
        let name = after_type[..name_end].trim();
        if name.is_empty() || name.chars().next()?.is_digit(10) {
            return None;
        }

        Some(RawExtraction {
            symbol: Some(name.to_string()),
            line_range: LineRange { start: (line_idx + 1) as u32, end: (line_idx + 1) as u32 },
            raw_excerpt: trimmed.to_string(),
            strength: EvidenceStrength::Direct,
        })
    }
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

        // ── Pass 1: 收集 module/endmodule 配对 ──
        let mut modules: Vec<ModuleEntry> = vec![];
        let mut endmodules: Vec<usize> = vec![]; // 0-based line indices
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

            if let Some(rest) = trimmed.strip_prefix("module ") {
                let rest = rest.trim_start();
                let name_end = rest
                    .find(|c: char| !c.is_alphanumeric() && c != '_')
                    .unwrap_or(rest.len());
                let name = &rest[..name_end];
                if !name.is_empty() {
                    modules.push(ModuleEntry { line_idx: i, name: name.to_string() });
                }
                continue;
            }

            if trimmed.starts_with("endmodule") {
                let after = trimmed.strip_prefix("endmodule").unwrap_or("");
                if after.is_empty() || after.starts_with(' ') || after.starts_with('\t') || after.starts_with("//") {
                    endmodules.push(i);
                }
            }
        }

        // ── Pass 2: 构建 module-level RawExtraction ──
        let total_lines = lines.len();
        let mut results = vec![];
        let mut end_cursor: usize = 0;

        for module in &modules {
            let start = (module.line_idx + 1) as u32;
            let end_0based = endmodules[end_cursor..]
                .iter()
                .position(|&line| line > module.line_idx)
                .map(|pos| { let abs = end_cursor + pos; end_cursor = abs + 1; endmodules[abs] })
                .unwrap_or(total_lines - 1);
            let end_1based = ((end_0based + 1) as u32).max(start);
            let raw_excerpt = extract_lines_range(content, start, end_1based);

            results.push(RawExtraction {
                symbol: Some(module.name.clone()),
                line_range: LineRange { start, end: end_1based },
                raw_excerpt,
                strength: EvidenceStrength::Direct,
            });
        }

        // ── Pass 3: 行级提取（port / signal / assign / always / instance / parameter） ──
        let mut line_results: Vec<RawExtraction> = vec![];
        in_block_comment = false;

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

            // Skip module/endmodule definitions themselves (already extracted)
            let trimmed_raw = strip_line_comment(line);
            let trimmed = trimmed_raw.trim();

            // Skip empty lines, module/endmodule lines
            if trimmed.is_empty() || trimmed.starts_with("module ") || trimmed.starts_with("endmodule") {
                continue;
            }

            // Extract in priority order
            let mut extracted: Option<RawExtraction> = None;

            // Only extract line-level items within a module (or if no module at all, extract everything)
            // Only extract line-level items when there are modules (context matters).
            let inside = !modules.is_empty() && Self::is_inside_module(i, &modules, &endmodules);

            if inside {
                // Try each extractor in priority order
                extracted = extracted.or_else(|| self.extract_port(i, trimmed));
                extracted = extracted.or_else(|| self.extract_param(i, trimmed));
                extracted = extracted.or_else(|| self.extract_always(i, trimmed));
                extracted = extracted.or_else(|| self.extract_assign(i, trimmed));
                extracted = extracted.or_else(|| self.extract_signal_decl(i, trimmed));
                extracted = extracted.or_else(|| self.extract_instance(i, trimmed));
            }

            if let Some(ext) = extracted {
                line_results.push(ext);
            }
        }

        // ── 合并结果：module 优先，行级在后，按行号排序 ──
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
    fn vlg_01_single_module() {
        let content = "module top(\n    input clk,\n    output data\n);\nwire x;\nassign data = x;\nendmodule";
        let results = VerilogExtractor.extract(content);
        // Now extracts module(top) + port(clk) + port(data) + wire(x) = 4 items
        // "assign data = x" is for "data" but "data" is already a port → no additional assign symbol extracted
        // "data" shows up as both port and assign target, assign extraction returns "data" which collides
        // Actually "data" shows up twice - once as port, once as assign target. That's fine because they have different line ranges.
        // But wait, "data" appears as port on line 3 and as assign target on line 6. Both have the same symbol "data".
        // Let me count: module(top) line 1, port(clk) line 2, port(data) line 3, wire(x) line 5, assign(data) line 6 = 5 items
        assert!(results.len() >= 4, "应有至少 module+port+wire, 实际 {}", results.len());
        let symbols: Vec<&str> = results.iter().filter_map(|r| r.symbol.as_deref()).collect();
        assert!(symbols.contains(&"top"), "应包含 module top");
        assert!(symbols.contains(&"clk"), "应包含 port clk");
        assert!(symbols.contains(&"data"), "应包含 port/assign data");
        assert!(symbols.contains(&"x"), "应包含 wire x");
        assert_valid_ranges(&results);
    }

    #[test]
    fn vlg_02_multiple_modules() {
        let content = "module alu(\n    input a\n);\nendmodule\n\nmodule top(\n    input clk\n);\nendmodule";
        let results = VerilogExtractor.extract(content);
        // module(alu) + port(a) + module(top) + port(clk) = 4
        assert!(results.len() >= 4, "应有至少 4 项, 实际 {}", results.len());
        let symbols: Vec<&str> = results.iter().filter_map(|r| r.symbol.as_deref()).collect();
        assert!(symbols.contains(&"alu"));
        assert!(symbols.contains(&"a"), "应包含 port a");
        assert!(symbols.contains(&"top"));
        assert!(symbols.contains(&"clk"), "应包含 port clk");
        assert_valid_ranges(&results);
    }

    #[test]
    fn vlg_03_no_endmodule() {
        // 无 endmodule → range 到 EOF，仍标 Direct
        let content = "module incomplete(\n    input clk\n);";
        let results = VerilogExtractor.extract(content);
        // module(incomplete) + port(clk) = 2
        assert!(results.len() >= 2, "应有 module + port, 实际 {}", results.len());
        assert!(results.iter().any(|r| r.symbol.as_deref() == Some("incomplete")));
        assert!(results.iter().any(|r| r.symbol.as_deref() == Some("clk")));
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

    // ─── P1: 新增 HDL 行级提取测试 ──────────────────────────────────

    /// vlg_08: 端口提取 — input/output/inout
    #[test]
    fn vlg_08_port_extraction() {
        let content = "\
module top(
    input clk,
    input rst_n,
    input [11:0] rx_data,
    output reg [11:0] peak_idx,
    inout sda
);
endmodule
";
        let results = VerilogExtractor.extract(content);
        let port_names: Vec<&str> = results.iter()
            .filter_map(|r| r.symbol.as_deref())
            .filter(|s| *s == "clk" || *s == "rst_n" || *s == "rx_data" || *s == "peak_idx" || *s == "sda")
            .collect();
        assert!(port_names.contains(&"clk"), "应提取 clk 端口");
        assert!(port_names.contains(&"rst_n"), "应提取 rst_n 端口");
        assert!(port_names.contains(&"rx_data"), "应提取 rx_data 端口");
        assert!(port_names.contains(&"peak_idx"), "应提取 peak_idx 端口");
        assert!(port_names.contains(&"sda"), "应提取 sda inout 端口");
        assert_valid_ranges(&results);
    }

    /// vlg_09: wire/reg/logic 信号声明提取
    #[test]
    fn vlg_09_signal_decl_extraction() {
        let content = "\
module top(
    input clk
);
    wire [7:0] data_bus;
    reg  [3:0] counter;
    logic enable;
    wire clk_out;
endmodule
";
        let results = VerilogExtractor.extract(content);
        let sig_names: Vec<&str> = results.iter()
            .filter_map(|r| r.symbol.as_deref())
            .filter(|s| *s == "data_bus" || *s == "counter" || *s == "enable" || *s == "clk_out")
            .collect();
        assert!(sig_names.contains(&"data_bus"), "应提取 wire data_bus");
        assert!(sig_names.contains(&"counter"), "应提取 reg counter");
        assert!(sig_names.contains(&"enable"), "应提取 logic enable");
        assert!(sig_names.contains(&"clk_out"), "应提取 wire clk_out");
        assert_valid_ranges(&results);
    }

    /// vlg_10: assign 语句提取
    #[test]
    fn vlg_10_assign_extraction() {
        let content = "\
module top(
    input a, b,
    output sum
);
    assign sum = a ^ b;
    assign carry = a & b;
endmodule
";
        let results = VerilogExtractor.extract(content);
        let assign_names: Vec<&str> = results.iter()
            .filter_map(|r| r.symbol.as_deref())
            .filter(|s| *s == "sum" || *s == "carry")
            .collect();
        assert!(assign_names.contains(&"sum"), "应提取 assign sum");
        assert!(assign_names.contains(&"carry"), "应提取 assign carry");
        assert_valid_ranges(&results);
    }

    /// vlg_11: always 块提取
    #[test]
    fn vlg_11_always_extraction() {
        let content = "\
module top(input clk, input rst_n, output reg q);
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n)
            q <= 1'b0;
        else
            q <= ~q;
    end
    always_ff @(posedge clk) begin
        q <= ~q;
    end
    always_comb begin
        q = ~q;
    end
endmodule
";
        let results = VerilogExtractor.extract(content);
        let always_symbols: Vec<&str> = results.iter()
            .filter_map(|r| r.symbol.as_deref())
            .filter(|s| *s == "always" || *s == "always_ff" || *s == "always_comb")
            .collect();
        assert!(always_symbols.contains(&"always"), "应提取 always @(...)");
        assert!(always_symbols.contains(&"always_ff"), "应提取 always_ff");
        assert!(always_symbols.contains(&"always_comb"), "应提取 always_comb");
        assert_valid_ranges(&results);
    }

    /// vlg_12: 模块实例化提取
    #[test]
    fn vlg_12_instance_extraction() {
        let content = "\
module top(input clk, input [7:0] data);
    adder u_adder (
        .a(data),
        .b(data),
        .sum(sum)
    );
    multiplier #(.WIDTH(8)) u_mult (
        .in(data),
        .out(result)
    );
endmodule
";
        let results = VerilogExtractor.extract(content);
        let inst_names: Vec<&str> = results.iter()
            .filter_map(|r| r.symbol.as_deref())
            .filter(|s| *s == "u_adder" || *s == "u_mult")
            .collect();
        assert!(inst_names.contains(&"u_adder"), "应提取 u_adder 实例");
        assert!(inst_names.contains(&"u_mult"), "应提取 u_mult 实例");
        assert_valid_ranges(&results);
    }

    /// vlg_13: 参数提取 — parameter/localparam
    #[test]
    fn vlg_13_parameter_extraction() {
        let content = "\
module top();
    parameter WIDTH = 8;
    localparam DEPTH = 64;
    parameter integer ADDR_WIDTH = 4;
endmodule
";
        let results = VerilogExtractor.extract(content);
        let param_names: Vec<&str> = results.iter()
            .filter_map(|r| r.symbol.as_deref())
            .filter(|s| *s == "WIDTH" || *s == "DEPTH" || *s == "ADDR_WIDTH")
            .collect();
        assert!(param_names.contains(&"WIDTH"), "应提取 parameter WIDTH");
        assert!(param_names.contains(&"DEPTH"), "应提取 localparam DEPTH");
        assert!(param_names.contains(&"ADDR_WIDTH"), "应提取 parameter integer ADDR_WIDTH");
        assert_valid_ranges(&results);
    }

    /// vlg_14: 综合测试 — 多种提取共存，line_range 合法
    #[test]
    fn vlg_14_comprehensive_extraction() {
        let content = "\
module coarse_sync (
    input clk,
    input rst_n,
    input [11:0] rx_data,
    output reg [11:0] peak_idx,
    output reg [11:0] energy
);
    parameter THRESHOLD = 1000;
    localparam WINDOW = 64;

    wire [11:0] corr_out;
    reg  [11:0] acc;

    assign energy = acc;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n)
            acc <= 0;
        else
            acc <= acc + rx_data;
    end

    correlator #(.WIDTH(12)) u_corr (
        .data(rx_data),
        .result(corr_out)
    );
endmodule
";
        let results = VerilogExtractor.extract(content);
        // Module + ports + signals + assign + always + instance + param
        assert!(results.len() >= 10, "应有至少 10 项提取，实际 {}", results.len());

        let symbols: Vec<&str> = results.iter().filter_map(|r| r.symbol.as_deref()).collect();
        assert!(symbols.contains(&"coarse_sync"), "应提取 module");
        assert!(symbols.contains(&"clk"), "应提取 clk port");
        assert!(symbols.contains(&"rx_data"), "应提取 rx_data port");
        assert!(symbols.contains(&"THRESHOLD"), "应提取 parameter");
        assert!(symbols.contains(&"WINDOW"), "应提取 localparam");
        assert!(symbols.contains(&"corr_out"), "应提取 wire");
        assert!(symbols.contains(&"acc"), "应提取 reg");
        assert!(symbols.contains(&"energy"), "应提取 assign");
        assert!(symbols.contains(&"always"), "应提取 always");
        assert!(symbols.contains(&"u_corr"), "应提取实例");
        assert_valid_ranges(&results);
    }
}
