use std::collections::HashSet;

use crate::evidence::models::{EvidenceCollection, EvidenceItem};

// ─── 结构化输出类型 ──────────────────────────────────────────────────

/// 结构化证据上下文条目 — 包含 evidence 的完整元信息
pub struct EvidenceContextItem {
    pub evidence_id: String,
    pub summary: String,
    pub symbol: Option<String>,
    /// 语言标识（snake_case，如 "python"、"verilog"）
    pub language: String,
    /// 来源类型（snake_case，如 "python_stage"、"rtl"）
    pub source_kind: String,
    /// 证据强度（snake_case，如 "direct"、"indirect"）
    pub strength: String,
}

/// 索引摘要
pub struct IndexSummary {
    pub path_count: usize,
    pub kind_count: usize,
    pub symbol_count: usize,
}

/// 统计摘要
pub struct StatsSummary {
    pub files_processed: u32,
    pub files_skipped: u32,
    pub total_items: u32,
}

// ─── 输入/输出结构 ──────────────────────────────────────────────────

/// ContextBuilder 输入（从 EvidenceCollection 派生的中间结构）
///
/// 设计理由：将 EvidenceCollection 的平铺数据重组为 LLM 可消费的结构，
/// 包括索引和 known_evidence_ids 用于后续 hallucination guard。
pub struct GeneratorInput {
    /// 阶段 ID
    pub stage_id: String,
    /// 所有 evidence items（按原始顺序）
    pub evidence_items: Vec<EvidenceItem>,
    /// 按文件分组的索引
    pub index_by_path: std::collections::HashMap<String, Vec<String>>,
    /// 按类型分组的索引
    pub index_by_kind: std::collections::HashMap<String, Vec<String>>,
    /// 按符号分组的索引
    pub index_by_symbol: std::collections::HashMap<String, Vec<String>>,
    /// 所有 evidence_id 集合（用于 existence check）
    pub known_evidence_ids: HashSet<String>,
}

/// ContextBuilder 输出（传给 Provider 和 Validator 的结构）
pub struct GeneratorOutput {
    /// Prompt（含 system prompt + user prompt）
    pub prompt: String,
    /// JSON schema（约束 LLM 输出格式）
    pub output_schema: serde_json::Value,
    /// 已知的 evidence_id 集合（传给 validator 用于 hallucination guard）
    pub known_evidence_ids: HashSet<String>,
    /// 结构化证据上下文条目
    pub evidence_context_items: Vec<EvidenceContextItem>,
    /// 索引摘要
    pub index_summary: IndexSummary,
    /// 统计摘要
    pub stats_summary: StatsSummary,
    /// 警告摘要（字符串列表）
    pub warnings_summary: Vec<String>,
}

// ─── 枚举序列化辅助 ─────────────────────────────────────────────────

/// 将 serde 枚举值转为 snake_case 字符串
fn enum_to_str<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default()
}

// ─── ContextBuilder ──────────────────────────────────────────────────

/// 确定性预打包层 — 将 EvidenceCollection 转换为 LLM 可消费的结构化输入
///
/// 这一层是**完全确定性**的，不涉及 LLM 调用。
pub struct ContextBuilder;

impl ContextBuilder {
    /// 从 EvidenceCollection 构建 LLM 输入
    pub fn build(collection: &EvidenceCollection) -> GeneratorOutput {
        let known_ids: HashSet<String> = collection
            .evidence_items
            .iter()
            .map(|item| item.evidence_id.clone())
            .collect();

        let evidence_context_items: Vec<EvidenceContextItem> = collection
            .evidence_items
            .iter()
            .map(|item| EvidenceContextItem {
                evidence_id: item.evidence_id.clone(),
                summary: item.summary.clone(),
                symbol: item.symbol.clone(),
                language: enum_to_str(&item.language),
                source_kind: enum_to_str(&item.source_kind),
                strength: enum_to_str(&item.strength),
            })
            .collect();

        let index_summary = IndexSummary {
            path_count: collection.index_by_path.len(),
            kind_count: collection.index_by_kind.len(),
            symbol_count: collection.index_by_symbol.len(),
        };

        let stats_summary = StatsSummary {
            files_processed: collection.stats.files_processed,
            files_skipped: collection.stats.files_skipped,
            total_items: collection.stats.total_items,
        };

        let warnings_summary: Vec<String> = collection
            .warnings
            .iter()
            .map(|w| format!("{:?}: {}", w.error_code, w.message))
            .collect();

        let prompt = Self::build_prompt(collection, &evidence_context_items);
        let schema = Self::build_output_schema();

        GeneratorOutput {
            prompt,
            output_schema: schema,
            known_evidence_ids: known_ids,
            evidence_context_items,
            index_summary,
            stats_summary,
            warnings_summary,
        }
    }

    /// 构建 prompt — system prompt + user prompt（含语言/来源/强度）
    fn build_prompt(
        collection: &EvidenceCollection,
        context_items: &[EvidenceContextItem],
    ) -> String {
        let mut parts = Vec::new();

        // System prompt
        parts.push(
            "你是一个 FPGA 实现理解助手。你的任务是基于提供的 evidence 生成结构化理解。"
                .to_string(),
        );
        parts.push(String::new());
        parts.push("约束：".to_string());
        parts.push("1. 每条 claim 必须引用 evidence_id".to_string());
        parts.push("2. evidence_id 必须在提供的 evidence 列表中真实存在".to_string());
        parts.push("3. 无法推断的内容标注为 unknown".to_string());
        parts.push("4. 缺失的 evidence 标注为 evidence_gap".to_string());
        parts.push("5. 不使用\"正确/错误\"、\"PASS/HOLD\"等审计用语".to_string());
        parts.push(
            "6. confidence 语义：confirmed（充分证据）、supported（有证据需辅助推断）、inferred（有限证据）、unknown（证据不足）、conflicting（证据矛盾）"
                .to_string(),
        );

        // User prompt — 阶段信息
        parts.push(String::new());
        parts.push(format!("阶段 ID: {}", collection.stage_id));
        parts.push(format!("证据总数: {}", collection.evidence_items.len()));

        // 证据条目（含 language, source_kind, strength）
        if context_items.is_empty() {
            parts.push(String::new());
            parts.push("无可用证据。".to_string());
            parts.push(String::new());
            parts.push("提示：在证据完全缺失时，应：".to_string());
            parts.push("1. 不生成任何 claim".to_string());
            parts.push("2. 在 unknowns 中标注无法推断的内容".to_string());
            parts.push("3. 在 evidence_gaps 中标注期望的证据类型".to_string());
        } else {
            for item in context_items {
                let symbol_str = match &item.symbol {
                    Some(s) => format!(" [{}]", s),
                    None => String::new(),
                };
                parts.push(String::new());
                parts.push(format!(
                    "- {}{} ({}, {}, {}): {}",
                    item.evidence_id,
                    symbol_str,
                    item.language,
                    item.source_kind,
                    item.strength,
                    item.summary
                ));
            }
        }

        parts.join("\n")
    }

    /// 返回 ImplementationUnderstanding 的 JSON schema
    ///
    /// 使用 serde_json::json! 硬编码，不引入 JSON Schema crate。
    fn build_output_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "required": ["stage_id", "version", "summary", "claims", "stats", "generation_meta"],
            "properties": {
                "stage_id": { "type": "string" },
                "version": { "type": "string" },
                "summary": {
                    "type": "object",
                    "required": ["short", "detailed"],
                    "properties": {
                        "short": { "type": "string" },
                        "detailed": { "type": "string" }
                    }
                },
                "claims": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["claim_id", "category", "description", "confidence", "evidence_refs", "has_evidence_gap"],
                        "properties": {
                            "claim_id": { "type": "string" },
                            "category": { "type": "string" },
                            "description": { "type": "string" },
                            "confidence": { "type": "string" },
                            "evidence_refs": { "type": "array" },
                            "has_evidence_gap": { "type": "boolean" }
                        }
                    }
                },
                "module_summaries": { "type": "array" },
                "signal_summaries": { "type": "array" },
                "interface_summaries": { "type": "array" },
                "processing_steps": { "type": "array" },
                "unknowns": { "type": "array" },
                "evidence_gaps": { "type": "array" },
                "generation_meta": { "type": "object" },
                "stats": { "type": "object" }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::models::{EvidenceStats, EvidenceStrength, LineRange};
    use crate::models::enums::{Language, SourceKind};
    use std::collections::HashMap;

    /// 构建测试用的 EvidenceCollection
    fn make_collection(stage_id: &str, items: Vec<EvidenceItem>) -> EvidenceCollection {
        let known_ids: Vec<String> = items.iter().map(|i| i.evidence_id.clone()).collect();
        EvidenceCollection {
            stage_id: stage_id.to_string(),
            evidence_items: items,
            index_by_path: HashMap::new(),
            index_by_kind: HashMap::new(),
            index_by_symbol: HashMap::new(),
            warnings: vec![],
            stats: EvidenceStats {
                files_processed: 1,
                files_skipped: 0,
                total_items: known_ids.len() as u32,
                items_by_kind: HashMap::new(),
                items_by_strength: HashMap::new(),
            },
            version: "1.0.0".to_string(),
        }
    }

    fn make_item(id: &str, symbol: Option<&str>, summary: &str) -> EvidenceItem {
        EvidenceItem {
            evidence_id: id.to_string(),
            source_path: "/tmp/test.py".to_string(),
            language: Language::Python,
            source_kind: SourceKind::PythonStage,
            line_range: LineRange { start: 1, end: 5 },
            symbol: symbol.map(|s| s.to_string()),
            summary: summary.to_string(),
            strength: EvidenceStrength::Direct,
        }
    }

    // ─── ctx_01 ~ ctx_05: 原有测试 ──────────────────────────────────

    #[test]
    fn ctx_01_normal_collection_prompt_contains_evidence() {
        let items = vec![
            make_item("EV-L0-000001", Some("mod_a"), "模块 A 定义"),
            make_item("EV-L0-000002", Some("mod_b"), "模块 B 定义"),
            make_item("EV-L0-000003", Some("fn_c"), "函数 C 实现"),
            make_item("EV-L0-000004", None, "文件级描述"),
            make_item("EV-L0-000005", Some("mod_e"), "模块 E 定义"),
        ];
        let collection = make_collection("L0", items);
        let output = ContextBuilder::build(&collection);

        // prompt 包含所有 evidence_id
        assert!(output.prompt.contains("EV-L0-000001"), "must contain EV-L0-000001");
        assert!(output.prompt.contains("EV-L0-000002"), "must contain EV-L0-000002");
        assert!(output.prompt.contains("EV-L0-000003"), "must contain EV-L0-000003");
        assert!(output.prompt.contains("EV-L0-000004"), "must contain EV-L0-000004");
        assert!(output.prompt.contains("EV-L0-000005"), "must contain EV-L0-000005");

        // prompt 包含 symbol
        assert!(output.prompt.contains("mod_a"), "must contain symbol mod_a");
        assert!(output.prompt.contains("mod_b"), "must contain symbol mod_b");
        assert!(output.prompt.contains("fn_c"), "must contain symbol fn_c");

        // prompt 包含 summary
        assert!(output.prompt.contains("模块 A 定义"), "must contain summary");
        assert!(output.prompt.contains("文件级描述"), "must contain summary");
    }

    #[test]
    fn ctx_02_empty_collection_prompt() {
        let collection = make_collection("L0", vec![]);
        let output = ContextBuilder::build(&collection);

        assert!(
            output.prompt.contains("无可用证据"),
            "empty collection prompt must indicate no evidence"
        );
        assert!(
            output.prompt.contains("证据总数: 0"),
            "must show evidence count 0"
        );
    }

    #[test]
    fn ctx_03_known_evidence_ids_consistent() {
        let items = vec![
            make_item("EV-L0-000001", None, "a"),
            make_item("EV-L0-000002", None, "b"),
            make_item("EV-L0-000003", None, "c"),
            make_item("EV-L0-000004", None, "d"),
            make_item("EV-L0-000005", None, "e"),
        ];
        let collection = make_collection("L0", items);
        let output = ContextBuilder::build(&collection);

        assert_eq!(output.known_evidence_ids.len(), 5);
        assert!(output.known_evidence_ids.contains("EV-L0-000001"));
        assert!(output.known_evidence_ids.contains("EV-L0-000003"));
        assert!(output.known_evidence_ids.contains("EV-L0-000005"));
    }

    #[test]
    fn ctx_04_output_schema_valid_json() {
        let collection = make_collection("L0", vec![]);
        let output = ContextBuilder::build(&collection);

        // schema 是合法 JSON
        assert!(output.output_schema.is_object(), "schema must be a JSON object");

        // 包含 ImplementationUnderstanding 关键字段
        let props = output.output_schema.get("properties").unwrap();
        assert!(props.get("stage_id").is_some(), "schema must have stage_id");
        assert!(props.get("version").is_some(), "schema must have version");
        assert!(props.get("summary").is_some(), "schema must have summary");
        assert!(props.get("claims").is_some(), "schema must have claims");
        assert!(props.get("stats").is_some(), "schema must have stats");
        assert!(props.get("generation_meta").is_some(), "schema must have generation_meta");

        // required 字段
        let required = output
            .output_schema
            .get("required")
            .unwrap()
            .as_array()
            .unwrap();
        assert!(required.iter().any(|r| r == "stage_id"));
        assert!(required.iter().any(|r| r == "claims"));
    }

    #[test]
    fn ctx_05_prompt_contains_constraints() {
        let collection = make_collection("L0", vec![]);
        let output = ContextBuilder::build(&collection);

        assert!(
            output.prompt.contains("evidence_id"),
            "system prompt must mention evidence_id"
        );
        assert!(
            output.prompt.contains("confidence"),
            "system prompt must mention confidence"
        );
        assert!(
            output.prompt.contains("unknown"),
            "system prompt must mention unknown"
        );
    }

    // ─── ctx_06 ~ ctx_08: 新增测试 ──────────────────────────────────

    /// ctx_06: prompt 包含每个 evidence 的 language/source_kind/strength
    #[test]
    fn ctx_06_prompt_includes_language_source_kind_strength() {
        let items = vec![
            make_item("EV-L0-000001", Some("mod_a"), "模块 A"),
        ];
        let collection = make_collection("L0", items);
        let output = ContextBuilder::build(&collection);

        assert!(
            output.prompt.contains("python"),
            "prompt must contain language 'python'"
        );
        assert!(
            output.prompt.contains("python_stage"),
            "prompt must contain source_kind 'python_stage'"
        );
        assert!(
            output.prompt.contains("direct"),
            "prompt must contain strength 'direct'"
        );
    }

    /// ctx_07: evidence_context_items 包含完整的结构化元信息
    #[test]
    fn ctx_07_evidence_context_items_structured() {
        let items = vec![
            make_item("EV-L0-000001", Some("mod_a"), "模块 A"),
            make_item("EV-L0-000002", None, "文件描述"),
        ];
        let collection = make_collection("L0", items);
        let output = ContextBuilder::build(&collection);

        assert_eq!(output.evidence_context_items.len(), 2);

        let item0 = &output.evidence_context_items[0];
        assert_eq!(item0.evidence_id, "EV-L0-000001");
        assert_eq!(item0.symbol, Some("mod_a".to_string()));
        assert_eq!(item0.language, "python");
        assert_eq!(item0.source_kind, "python_stage");
        assert_eq!(item0.strength, "direct");
        assert_eq!(item0.summary, "模块 A");

        let item1 = &output.evidence_context_items[1];
        assert_eq!(item1.evidence_id, "EV-L0-000002");
        assert_eq!(item1.symbol, None);
        assert_eq!(item1.summary, "文件描述");
    }

    /// ctx_08: 空集合 prompt 包含 gap/unknown 提示 + stats/index 正确
    #[test]
    fn ctx_08_empty_collection_includes_hints() {
        let collection = make_collection("L0", vec![]);
        let output = ContextBuilder::build(&collection);

        // prompt 包含 gap/unknown 提示
        assert!(
            output.prompt.contains("unknowns"),
            "must hint about unknowns"
        );
        assert!(
            output.prompt.contains("evidence_gaps"),
            "must hint about evidence_gaps"
        );

        // evidence_context_items 为空
        assert!(output.evidence_context_items.is_empty());

        // stats_summary 反映空集合
        assert_eq!(output.stats_summary.total_items, 0);
        assert_eq!(output.stats_summary.files_processed, 1);

        // index_summary 全零
        assert_eq!(output.index_summary.path_count, 0);
        assert_eq!(output.index_summary.kind_count, 0);
        assert_eq!(output.index_summary.symbol_count, 0);

        // warnings_summary 为空
        assert!(output.warnings_summary.is_empty());
    }
}
