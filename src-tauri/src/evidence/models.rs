use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::models::enums::{ErrorCode, Language, SourceKind};

/// 证据强度枚举（完整定义，Phase 2 只使用 direct / indirect）
///
/// 不含 Unknown — 解析失败通过 warnings[] 表达，不产生 EvidenceItem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStrength {
    Direct,
    Indirect,
    Weak,
    Conflicting,
    Missing,
}

/// 行号范围（1-based，闭区间）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineRange {
    /// 起始行号，>= 1
    pub start: u32,
    /// 结束行号，>= start
    pub end: u32,
}

/// 单条证据项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    /// 全局唯一标识，格式 "EV-<stage_id>-<6位序号>"
    pub evidence_id: String,

    /// 源码文件绝对路径
    pub source_path: String,

    /// 语言，继承自 Phase 1 file_classifier
    pub language: Language,

    /// 来源类型，继承自 Phase 1 file_classifier
    pub source_kind: SourceKind,

    /// 行号范围（1-based，闭区间）
    pub line_range: LineRange,

    /// 符号名称（函数名/类名/module 名等）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,

    /// 代码片段或描述，最大 500 字符
    pub summary: String,

    /// 证据强度（evidence strength），不是 claim confidence
    pub strength: EvidenceStrength,
}

/// 证据收集警告（Phase 2 专用）
///
/// 设计理由：与 Phase 1 的 WorkspaceWarning 分开，因为：
/// 1. evidence 警告不需要 related_stage_id（evidence 始终在单个 stage 内）
/// 2. evidence 警告不需要 recoverable（所有 evidence 警告均为非致命，设计上保证）
/// 3. 更简洁、更聚焦
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceWarning {
    pub error_code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
}

/// 证据收集统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceStats {
    /// 处理的文件总数
    pub files_processed: u32,
    /// 跳过的文件数（二进制、不可读等）
    pub files_skipped: u32,
    /// evidence item 总数
    pub total_items: u32,
    /// 按 source_kind 分组的 item 计数
    pub items_by_kind: HashMap<String, u32>,
    /// 按 strength 分组的 item 计数
    pub items_by_strength: HashMap<String, u32>,
}

/// 证据集合（单阶段），对应 mvp-functional-contract.md 的 evidence_index.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceCollection {
    /// 阶段标识
    pub stage_id: String,

    /// 证据项列表
    pub evidence_items: Vec<EvidenceItem>,

    /// 按文件路径分组索引
    /// key = source_path，value = evidence_id[]
    pub index_by_path: HashMap<String, Vec<String>>,

    /// 按来源类型分组索引
    /// key = source_kind（snake_case 字符串），value = evidence_id[]
    pub index_by_kind: HashMap<String, Vec<String>>,

    /// 按符号名称反向索引
    /// key = symbol，value = evidence_id[]
    /// 仅包含 symbol 非 None 的 item
    pub index_by_symbol: HashMap<String, Vec<String>>,

    /// 收集过程中的非致命警告
    pub warnings: Vec<EvidenceWarning>,

    /// 收集统计
    pub stats: EvidenceStats,

    /// 产物格式版本
    pub version: String,
}

/// 原始提取结果（提取器中间产物，ID 和 summary 截断在 collector 层统一处理）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawExtraction {
    /// 符号名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// 行号范围
    pub line_range: LineRange,
    /// 原始代码片段（可能超过 500 字符，由 excerpt 模块截断）
    pub raw_excerpt: String,
    /// 证据强度
    pub strength: EvidenceStrength,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_strength_no_unknown() {
        let values = [
            EvidenceStrength::Direct,
            EvidenceStrength::Indirect,
            EvidenceStrength::Weak,
            EvidenceStrength::Conflicting,
            EvidenceStrength::Missing,
        ];
        let json = serde_json::to_string(&values).unwrap();
        assert!(!json.contains("unknown"), "EvidenceStrength must not contain unknown: {}", json);
    }

    #[test]
    fn evidence_strength_serde_roundtrip() {
        for strength in [
            EvidenceStrength::Direct,
            EvidenceStrength::Indirect,
            EvidenceStrength::Weak,
            EvidenceStrength::Conflicting,
            EvidenceStrength::Missing,
        ] {
            let json = serde_json::to_string(&strength).unwrap();
            let back: EvidenceStrength = serde_json::from_str(&json).unwrap();
            assert_eq!(strength, back, "roundtrip failed for {}", json);
        }
    }

    #[test]
    fn evidence_strength_snake_case() {
        assert_eq!(
            serde_json::to_string(&EvidenceStrength::Direct).unwrap(),
            "\"direct\""
        );
        assert_eq!(
            serde_json::to_string(&EvidenceStrength::Indirect).unwrap(),
            "\"indirect\""
        );
        assert_eq!(
            serde_json::to_string(&EvidenceStrength::Conflicting).unwrap(),
            "\"conflicting\""
        );
    }

    #[test]
    fn line_range_serializes_to_object() {
        let lr = LineRange { start: 1, end: 10 };
        let json = serde_json::to_string(&lr).unwrap();
        assert!(json.contains("\"start\":1"));
        assert!(json.contains("\"end\":10"));
    }

    #[test]
    fn evidence_item_fields_present() {
        let item = EvidenceItem {
            evidence_id: "EV-L0-000001".to_string(),
            source_path: "/tmp/test.py".to_string(),
            language: Language::Python,
            source_kind: SourceKind::PythonStage,
            line_range: LineRange { start: 1, end: 5 },
            symbol: Some("foo".to_string()),
            summary: "def foo(): pass".to_string(),
            strength: EvidenceStrength::Direct,
        };
        let json = serde_json::to_string(&item).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("evidence_id").is_some());
        assert!(parsed.get("source_path").is_some());
        assert!(parsed.get("language").is_some());
        assert!(parsed.get("source_kind").is_some());
        assert!(parsed.get("line_range").is_some());
        assert!(parsed.get("symbol").is_some());
        assert!(parsed.get("summary").is_some());
        assert!(parsed.get("strength").is_some());
        // 不应出现 confidence 字段
        assert!(parsed.get("confidence").is_none(), "EvidenceItem must not have confidence field");
    }

    #[test]
    fn evidence_item_symbol_none_skipped() {
        let item = EvidenceItem {
            evidence_id: "EV-L0-000001".to_string(),
            source_path: "/tmp/test.py".to_string(),
            language: Language::Python,
            source_kind: SourceKind::PythonStage,
            line_range: LineRange { start: 1, end: 5 },
            symbol: None,
            summary: "...".to_string(),
            strength: EvidenceStrength::Indirect,
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(!json.contains("\"symbol\""), "symbol=None should be skipped: {}", json);
    }

    #[test]
    fn evidence_collection_roundtrip() {
        let collection = EvidenceCollection {
            stage_id: "L0".to_string(),
            evidence_items: vec![],
            index_by_path: HashMap::new(),
            index_by_kind: HashMap::new(),
            index_by_symbol: HashMap::new(),
            warnings: vec![],
            stats: EvidenceStats {
                files_processed: 0,
                files_skipped: 0,
                total_items: 0,
                items_by_kind: HashMap::new(),
                items_by_strength: HashMap::new(),
            },
            version: "1.0.0".to_string(),
        };
        let json = serde_json::to_string(&collection).unwrap();
        let back: EvidenceCollection = serde_json::from_str(&json).unwrap();
        assert_eq!(collection.stage_id, back.stage_id);
        assert_eq!(collection.version, back.version);
    }

    #[test]
    fn evidence_stats_has_items_by_strength() {
        let mut stats = EvidenceStats {
            files_processed: 3,
            files_skipped: 1,
            total_items: 10,
            items_by_kind: HashMap::new(),
            items_by_strength: HashMap::new(),
        };
        stats.items_by_strength.insert("direct".to_string(), 7);
        stats.items_by_strength.insert("indirect".to_string(), 3);
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("items_by_strength"), "must use items_by_strength, not items_by_confidence: {}", json);
        assert!(!json.contains("items_by_confidence"), "must not have items_by_confidence: {}", json);
    }
}
