/// Evidence 索引构建器
///
/// 从 `Vec<EvidenceItem>` 构建三组 HashMap 索引：
/// - index_by_path: key = source_path
/// - index_by_kind: key = source_kind (snake_case)
/// - index_by_symbol: key = symbol（仅 symbol.is_some() 的 item）
///
/// 设计约束：
/// - 覆盖所有 EvidenceItem（path/kind），不丢失
/// - 保持 evidence_items 原始顺序，不重排
/// - 同一 key 对应多个 evidence_id 时全部保留

use std::collections::HashMap;

use crate::models::enums::SourceKind;

use super::models::EvidenceItem;

/// 证据索引集合
#[derive(Debug, Clone)]
pub struct EvidenceIndexes {
    /// key = source_path，value = evidence_id[]
    pub index_by_path: HashMap<String, Vec<String>>,
    /// key = source_kind (snake_case)，value = evidence_id[]
    pub index_by_kind: HashMap<String, Vec<String>>,
    /// key = symbol，value = evidence_id[]（仅 symbol 非 None 的 item）
    pub index_by_symbol: HashMap<String, Vec<String>>,
}

/// 从 EvidenceItem 列表构建三组索引
pub fn build_indexes(items: &[EvidenceItem]) -> EvidenceIndexes {
    let mut by_path: HashMap<String, Vec<String>> = HashMap::new();
    let mut by_kind: HashMap<String, Vec<String>> = HashMap::new();
    let mut by_symbol: HashMap<String, Vec<String>> = HashMap::new();

    for item in items {
        by_path
            .entry(item.source_path.clone())
            .or_default()
            .push(item.evidence_id.clone());

        by_kind
            .entry(source_kind_key(&item.source_kind))
            .or_default()
            .push(item.evidence_id.clone());

        if let Some(ref sym) = item.symbol {
            by_symbol
                .entry(sym.clone())
                .or_default()
                .push(item.evidence_id.clone());
        }
    }

    EvidenceIndexes {
        index_by_path: by_path,
        index_by_kind: by_kind,
        index_by_symbol: by_symbol,
    }
}

/// SourceKind → snake_case 字符串键
fn source_kind_key(kind: &SourceKind) -> String {
    serde_json::to_string(kind)
        .unwrap()
        .trim_matches('"')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::models::{EvidenceStrength, LineRange};
    use crate::models::enums::Language;

    fn make_item(id: &str, path: &str, kind: SourceKind, symbol: Option<&str>) -> EvidenceItem {
        EvidenceItem {
            evidence_id: id.to_string(),
            source_path: path.to_string(),
            language: Language::Python,
            source_kind: kind,
            line_range: LineRange { start: 1, end: 1 },
            symbol: symbol.map(|s| s.to_string()),
            summary: "test".to_string(),
            strength: EvidenceStrength::Direct,
        }
    }

    #[test]
    fn idx_01_empty_input_returns_empty_indexes() {
        let indexes = build_indexes(&[]);
        assert!(indexes.index_by_path.is_empty());
        assert!(indexes.index_by_kind.is_empty());
        assert!(indexes.index_by_symbol.is_empty());
    }

    #[test]
    fn idx_02_same_file_multiple_items() {
        let items = vec![
            make_item("EV-L0-000001", "/tmp/a.py", SourceKind::PythonStage, Some("foo")),
            make_item("EV-L0-000002", "/tmp/a.py", SourceKind::PythonStage, Some("bar")),
        ];
        let indexes = build_indexes(&items);
        assert_eq!(indexes.index_by_path.len(), 1);
        let ids = indexes.index_by_path.get("/tmp/a.py").unwrap();
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0], "EV-L0-000001");
        assert_eq!(ids[1], "EV-L0-000002");
    }

    #[test]
    fn idx_03_same_source_kind_multiple_items() {
        let items = vec![
            make_item("EV-RTL-000001", "/tmp/a.v", SourceKind::Rtl, Some("mod_a")),
            make_item("EV-RTL-000002", "/tmp/b.v", SourceKind::Rtl, Some("mod_b")),
        ];
        let indexes = build_indexes(&items);
        assert_eq!(indexes.index_by_kind.len(), 1);
        let rtl = indexes.index_by_kind.get("rtl").unwrap();
        assert_eq!(rtl.len(), 2);
    }

    #[test]
    fn idx_04_symbol_and_no_symbol_mix() {
        let items = vec![
            make_item("EV-L0-000001", "/tmp/a.py", SourceKind::PythonStage, Some("foo")),
            make_item("EV-L0-000002", "/tmp/b.txt", SourceKind::Doc, None),
        ];
        let indexes = build_indexes(&items);
        // index_by_path 和 index_by_kind 覆盖所有 item
        assert_eq!(indexes.index_by_path.len(), 2);
        assert_eq!(indexes.index_by_kind.len(), 2);
        // index_by_symbol 只包含有 symbol 的 item
        assert_eq!(indexes.index_by_symbol.len(), 1);
        assert!(indexes.index_by_symbol.contains_key("foo"));
    }

    #[test]
    fn idx_05_duplicate_symbol_preserved() {
        let items = vec![
            make_item("EV-L0-000001", "/tmp/a.py", SourceKind::PythonStage, Some("process")),
            make_item("EV-L0-000002", "/tmp/b.py", SourceKind::PythonStage, Some("process")),
        ];
        let indexes = build_indexes(&items);
        let ids = indexes.index_by_symbol.get("process").unwrap();
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0], "EV-L0-000001");
        assert_eq!(ids[1], "EV-L0-000002");
    }

    #[test]
    fn idx_06_every_id_in_path_and_kind() {
        let items = vec![
            make_item("EV-L0-000001", "/tmp/a.py", SourceKind::PythonStage, Some("foo")),
            make_item("EV-RTL-000001", "/tmp/b.v", SourceKind::Rtl, Some("bar")),
            make_item("EV-DOC-000001", "/tmp/c.txt", SourceKind::Doc, None),
        ];
        let indexes = build_indexes(&items);

        let all_path_ids: Vec<&String> = indexes.index_by_path.values().flatten().collect();
        let all_kind_ids: Vec<&String> = indexes.index_by_kind.values().flatten().collect();

        assert_eq!(all_path_ids.len(), 3);
        assert_eq!(all_kind_ids.len(), 3);

        for item in &items {
            assert!(
                all_path_ids.contains(&&item.evidence_id),
                "evidence_id {} missing from index_by_path",
                item.evidence_id
            );
            assert!(
                all_kind_ids.contains(&&item.evidence_id),
                "evidence_id {} missing from index_by_kind",
                item.evidence_id
            );
        }
    }
}
