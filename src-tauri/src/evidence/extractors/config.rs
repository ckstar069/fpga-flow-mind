/// Config 提取器 — TCL / XDC 约束文件提取
///
/// 最小规则：
/// - `proc <name>` → symbol=name, strength=Direct
/// - `create_clock` → symbol="create_clock", strength=Indirect
/// - `set_property` → symbol="set_property", strength=Indirect
/// - `set_input_delay` → symbol="set_input_delay", strength=Indirect
/// - `set_output_delay` → symbol="set_output_delay", strength=Indirect
/// - line_range = 单行（start == end）
/// - 注释行（# 开头）不提取
/// - 普通 `set` 变量赋值不提取（Phase 2 最小实现不做）
/// - 空文件或无匹配返回空列表
///
/// Strength 选择理由：
/// - proc 定义是直接的结构性证据 → Direct
/// - 约束命令（create_clock 等）是间接证据，需结合上下文判断其含义 → Indirect
/// （与设计文档 phase-2-evidence-collector-design.md §5.5 一致）

use super::EvidenceExtractor;
use crate::evidence::models::{EvidenceStrength, LineRange, RawExtraction};

/// 已知的约束命令列表
const CONSTRAINT_COMMANDS: &[&str] = &[
    "create_clock",
    "set_property",
    "set_input_delay",
    "set_output_delay",
];

pub struct ConfigExtractor;

/// 匹配结果
struct MatchResult {
    symbol: String,
    strength: EvidenceStrength,
}

/// 尝试从行中提取 proc 定义
fn match_proc(line: &str) -> Option<MatchResult> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix("proc ")?;
    let rest = rest.trim_start();
    // proc 名到第一个空白、{ 或行尾
    let name_end = rest
        .find(|c: char| c.is_whitespace() || c == '{' || c == '(')
        .unwrap_or(rest.len());
    let name = &rest[..name_end];
    if name.is_empty() {
        return None;
    }
    Some(MatchResult {
        symbol: name.to_string(),
        strength: EvidenceStrength::Direct,
    })
}

/// 尝试从行中匹配约束命令
fn match_constraint(line: &str) -> Option<MatchResult> {
    let trimmed = line.trim_start();
    for &cmd in CONSTRAINT_COMMANDS {
        if let Some(rest) = trimmed.strip_prefix(cmd) {
            // 命令后必须跟空白或行尾（避免部分匹配如 create_clock_xxx）
            if rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t') || rest.starts_with('-') {
                return Some(MatchResult {
                    symbol: cmd.to_string(),
                    strength: EvidenceStrength::Indirect,
                });
            }
        }
    }
    None
}

impl EvidenceExtractor for ConfigExtractor {
    fn extract(&self, content: &str) -> Vec<RawExtraction> {
        if content.is_empty() {
            return vec![];
        }

        let mut results = vec![];

        for (i, line) in content.lines().enumerate() {
            let line_num = (i + 1) as u32; // 1-based

            // 跳过空行
            if line.trim().is_empty() {
                continue;
            }

            // 跳过注释行（# 开头）
            if line.trim_start().starts_with('#') {
                continue;
            }

            // 尝试匹配 proc
            if let Some(m) = match_proc(line) {
                results.push(RawExtraction {
                    symbol: Some(m.symbol),
                    line_range: LineRange {
                        start: line_num,
                        end: line_num,
                    },
                    raw_excerpt: line.to_string(),
                    strength: m.strength,
                });
                continue;
            }

            // 尝试匹配约束命令
            if let Some(m) = match_constraint(line) {
                results.push(RawExtraction {
                    symbol: Some(m.symbol),
                    line_range: LineRange {
                        start: line_num,
                        end: line_num,
                    },
                    raw_excerpt: line.to_string(),
                    strength: m.strength,
                });
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cfg_01_proc_direct() {
        let content = "proc build {\n    puts \"building\"\n}";
        let results = ConfigExtractor.extract(content);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol.as_deref(), Some("build"));
        assert_eq!(results[0].line_range, LineRange { start: 1, end: 1 });
        assert_eq!(results[0].strength, EvidenceStrength::Direct);
        assert_eq!(results[0].raw_excerpt, "proc build {");
    }

    #[test]
    fn cfg_02_create_clock_indirect() {
        let content = "create_clock -period 10 [get_ports clk]";
        let results = ConfigExtractor.extract(content);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol.as_deref(), Some("create_clock"));
        assert_eq!(results[0].strength, EvidenceStrength::Indirect);
    }

    #[test]
    fn cfg_03_set_property_indirect() {
        let content = "set_property -dict { IOSTANDARD LVCMOS33 } [get_ports led]";
        let results = ConfigExtractor.extract(content);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol.as_deref(), Some("set_property"));
        assert_eq!(results[0].strength, EvidenceStrength::Indirect);
    }

    #[test]
    fn cfg_04_comment_line_not_extracted() {
        let content = "# proc skipped\n# create_clock -period 10\nset_property IOSTANDARD LVCMOS33 [get_ports led]";
        let results = ConfigExtractor.extract(content);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol.as_deref(), Some("set_property"));
        assert_eq!(results[0].line_range, LineRange { start: 3, end: 3 });
    }

    #[test]
    fn cfg_05_multiple_constraints() {
        let content = "\
create_clock -period 10 [get_ports clk]
set_input_delay -clock clk 5 [get_ports data_in]
set_output_delay -clock clk 3 [get_ports data_out]";
        let results = ConfigExtractor.extract(content);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].symbol.as_deref(), Some("create_clock"));
        assert_eq!(results[1].symbol.as_deref(), Some("set_input_delay"));
        assert_eq!(results[2].symbol.as_deref(), Some("set_output_delay"));
        // 全部 Indirect
        assert!(results.iter().all(|r| r.strength == EvidenceStrength::Indirect));
    }

    #[test]
    fn cfg_06_empty_file() {
        let results = ConfigExtractor.extract("");
        assert!(results.is_empty());
    }
}
