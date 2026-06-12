use std::collections::HashSet;

use crate::evidence::models::{EvidenceCollection, EvidenceItem};

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
}

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

        let prompt = Self::build_prompt(collection);
        let schema = Self::build_output_schema();

        GeneratorOutput {
            prompt,
            output_schema: schema,
            known_evidence_ids: known_ids,
        }
    }

    /// 构建 prompt — system prompt + user prompt + 输出 schema
    fn build_prompt(collection: &EvidenceCollection) -> String {
        let mut parts = Vec::new();

        // System prompt
        parts.push("你是一个 FPGA 实现理解助手。你的任务是基于提供的 evidence 生成结构化理解。".to_string());
        parts.push("".to_string());
        parts.push("约束：".to_string());
        parts.push("1. 每条 claim 必须引用 evidence_id".to_string());
        parts.push("2. evidence_id 必须在提供的 evidence 列表中真实存在".to_string());
        parts.push("3. 无法推断的内容标注为 unknown".to_string());
        parts.push("4. 缺失的 evidence 标注为 evidence_gap".to_string());
        parts.push("5. 不使用\"正确/错误\"、\"PASS/HOLD\"等审计用语".to_string());
        parts.push("6. confidence 语义：confirmed（充分证据）、supported（有证据需辅助推断）、inferred（有限证据）、unknown（证据不足）、conflicting（证据矛盾）".to_string());

        // User prompt — evidence 摘要
        parts.push("".to_string());
        parts.push(format!("阶段 ID: {}", collection.stage_id));
        parts.push(format!("证据总数: {}", collection.evidence_items.len()));

        if collection.evidence_items.is_empty() {
            parts.push("".to_string());
            parts.push("无可用证据。".to_string());
        } else {
            for item in &collection.evidence_items {
                let symbol_str = match &item.symbol {
                    Some(s) => format!(" [{}]", s),
                    None => String::new(),
                };
                parts.push("".to_string());
                parts.push(format!(
                    "- {}{}: {}",
                    item.evidence_id, symbol_str, item.summary
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
        let required = output.output_schema.get("required").unwrap().as_array().unwrap();
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
}
