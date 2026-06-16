/// Evidence Collector 核心收集器
///
/// 遍历 `StageContext.files`，分派到各提取器，组装 `EvidenceCollection`。
/// 本模块不接 Tauri command，不做 command 层错误处理。
/// 单个文件失败不阻断整个收集流程。
///
/// 文件处理流程：
/// 1. 预检（目录/存在性/大小/二进制/编码）
/// 2. 分派到 `extract_by_language`
/// 3. 验证 line_range、分配 ID、截断 summary
///
/// 只允许的文件系统操作：`metadata`、`read`（只读）。

use std::collections::HashMap;
use std::path::Path;

use crate::models::enums::{ErrorCode, SourceKind};
use crate::models::stage_context::{StageContext, StageFile};

use super::excerpt::truncate_summary;
use super::extractors::extract_by_language;
use super::id_generator::EvidenceIdGenerator;
use super::index_builder::build_indexes;
use super::models::{
    EvidenceCollection, EvidenceItem, EvidenceStats, EvidenceStrength, EvidenceWarning,
};

/// 文件大小上限：5 MB
const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024;

pub struct EvidenceCollector {
    stage_id: String,
    id_generator: EvidenceIdGenerator,
}

impl EvidenceCollector {
    pub fn new(stage_id: &str) -> Self {
        Self {
            stage_id: stage_id.to_string(),
            id_generator: EvidenceIdGenerator::new(stage_id),
        }
    }

    /// 从 StageContext 收集 evidence
    pub fn collect_from_stage_context(
        &mut self,
        stage_context: &StageContext,
    ) -> EvidenceCollection {
        let mut items: Vec<EvidenceItem> = vec![];
        let mut warnings: Vec<EvidenceWarning> = vec![];
        let mut files_processed: u32 = 0;
        let mut files_skipped: u32 = 0;

        for file in &stage_context.files {
            match self.process_file(file, &mut warnings) {
                Ok(file_items) => {
                    items.extend(file_items);
                    files_processed += 1;
                }
                Err(()) => {
                    files_skipped += 1;
                }
            }
        }

        let indexes = build_indexes(&items);
        let stats = build_stats(&items, files_processed, files_skipped);

        EvidenceCollection {
            stage_id: self.stage_id.clone(),
            evidence_items: items,
            index_by_path: indexes.index_by_path,
            index_by_kind: indexes.index_by_kind,
            index_by_symbol: indexes.index_by_symbol,
            warnings,
            stats,
            version: "1.0.0".to_string(),
        }
    }

    /// 处理单个文件
    ///
    /// 返回 Ok(items) 表示文件成功处理（可能 0 items）。
    /// 返回 Err(()) 表示文件被跳过，调用方负责 files_skipped++。
    /// 失败原因已推入 warnings。
    fn process_file(
        &mut self,
        file: &StageFile,
        warnings: &mut Vec<EvidenceWarning>,
    ) -> Result<Vec<EvidenceItem>, ()> {
        let path = Path::new(&file.source_path);

        // 跳过目录
        if path.is_dir() {
            warnings.push(make_warning(
                ErrorCode::FileUnreadable,
                &format!("路径是目录，跳过: {}", file.source_path),
                &file.source_path,
            ));
            return Err(());
        }

        // 检查存在性
        if !path.exists() {
            warnings.push(make_warning(
                ErrorCode::FileUnreadable,
                &format!("文件不存在: {}", file.source_path),
                &file.source_path,
            ));
            return Err(());
        }

        // 获取文件元数据
        let metadata = match path.metadata() {
            Ok(m) => m,
            Err(_) => {
                warnings.push(make_warning(
                    ErrorCode::FileUnreadable,
                    &format!("无法读取文件元数据: {}", file.source_path),
                    &file.source_path,
                ));
                return Err(());
            }
        };

        // 大小检查
        if metadata.len() > MAX_FILE_SIZE {
            warnings.push(make_warning(
                ErrorCode::FileTooLarge,
                &format!(
                    "文件过大 ({} bytes > {} bytes): {}",
                    metadata.len(),
                    MAX_FILE_SIZE,
                    file.source_path
                ),
                &file.source_path,
            ));
            return Err(());
        }

        // 读取文件字节
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(_) => {
                warnings.push(make_warning(
                    ErrorCode::FileUnreadable,
                    &format!("无法读取文件: {}", file.source_path),
                    &file.source_path,
                ));
                return Err(());
            }
        };

        // 二进制检测（NUL 字节）
        if bytes.contains(&0) {
            warnings.push(make_warning(
                ErrorCode::BinaryFileSkipped,
                &format!("二进制文件，跳过: {}", file.source_path),
                &file.source_path,
            ));
            return Err(());
        }

        // UTF-8 解码
        let content = match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => {
                warnings.push(make_warning(
                    ErrorCode::NonUtf8FileSkipped,
                    &format!("非 UTF-8 文件，跳过: {}", file.source_path),
                    &file.source_path,
                ));
                return Err(());
            }
        };

        // 分派提取
        let extractions = extract_by_language(file.language, file.source_kind, &content);

        // 转换为 EvidenceItem
        let mut items = vec![];
        for raw in extractions {
            // 验证 line_range 合法性
            if raw.line_range.start < 1 || raw.line_range.start > raw.line_range.end {
                warnings.push(make_warning(
                    ErrorCode::EvidenceCollectionFailed,
                    &format!(
                        "非法 line_range [{}, {}]，跳过提取结果: {}",
                        raw.line_range.start,
                        raw.line_range.end,
                        raw.symbol.as_deref().unwrap_or("<anonymous>")
                    ),
                    &file.source_path,
                ));
                continue;
            }

            let evidence_id = self.id_generator.next_id();
            let line_count = (raw.line_range.end - raw.line_range.start + 1) as usize;
            let (summary, was_truncated) = truncate_summary(&raw.raw_excerpt, line_count);

            if was_truncated {
                warnings.push(make_warning(
                    ErrorCode::SourceExcerptTruncated,
                    &format!(
                        "summary 截断 (evidence_id={}): {}",
                        evidence_id, file.source_path
                    ),
                    &file.source_path,
                ));
            }

            items.push(EvidenceItem {
                evidence_id,
                source_path: file.source_path.clone(),
                language: file.language,
                source_kind: file.source_kind,
                line_range: raw.line_range,
                symbol: raw.symbol,
                summary,
                strength: raw.strength,
            });
        }

        Ok(items)
    }
}

/// 构建收集统计
fn build_stats(items: &[EvidenceItem], files_processed: u32, files_skipped: u32) -> EvidenceStats {
    let mut items_by_kind: HashMap<String, u32> = HashMap::new();
    let mut items_by_strength: HashMap<String, u32> = HashMap::new();

    for item in items {
        *items_by_kind
            .entry(kind_key(&item.source_kind))
            .or_insert(0) += 1;
        *items_by_strength
            .entry(strength_key(&item.strength))
            .or_insert(0) += 1;
    }

    EvidenceStats {
        files_processed,
        files_skipped,
        total_items: items.len() as u32,
        items_by_kind,
        items_by_strength,
    }
}

/// 构造 EvidenceWarning
fn make_warning(error_code: ErrorCode, message: &str, source_path: &str) -> EvidenceWarning {
    EvidenceWarning {
        error_code,
        message: message.to_string(),
        source_path: Some(source_path.to_string()),
    }
}

/// SourceKind → snake_case 字符串
fn kind_key(kind: &SourceKind) -> String {
    serde_json::to_string(kind)
        .unwrap()
        .trim_matches('"')
        .to_string()
}

/// EvidenceStrength → snake_case 字符串
fn strength_key(strength: &EvidenceStrength) -> String {
    serde_json::to_string(strength)
        .unwrap()
        .trim_matches('"')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::enums::Language;
    use std::collections::HashSet;
    use std::path::PathBuf;

    // ─── 测试辅助 ──────────────────────────────────────────────────

    /// 测试临时目录（Drop 时自动清理）
    struct TestDir(PathBuf);

    impl TestDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir()
                .join("fpga-flow-mind-collector-test")
                .join(name);
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        /// 写入文件，返回绝对路径字符串
        fn write(&self, name: &str, content: &[u8]) -> String {
            let file_path = self.0.join(name);
            std::fs::write(&file_path, content).unwrap();
            file_path.to_string_lossy().to_string()
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// 创建测试用 StageContext
    fn make_stage(files: Vec<StageFile>) -> StageContext {
        StageContext {
            stage_id: "L0".to_string(),
            source_path: "/tmp".to_string(),
            files,
            external_deps: vec![],
            upstream_refs: vec![],
            error_code: None,
        }
    }

    fn make_file(path: &str, lang: Language, kind: SourceKind) -> StageFile {
        StageFile {
            source_path: path.to_string(),
            language: lang,
            source_kind: kind,
            size_bytes: None,
        }
    }

    // ─── 正常提取测试 ──────────────────────────────────────────────

    #[test]
    fn col_01_python_stage() {
        let dir = TestDir::new("col_01");
        let py_path = dir.write("main.py", b"def foo():\n    pass\n\ndef bar():\n    x = 1\n");

        let ctx = make_stage(vec![make_file(&py_path, Language::Python, SourceKind::PythonStage)]);
        let mut collector = EvidenceCollector::new("L0");
        let collection = collector.collect_from_stage_context(&ctx);

        assert_eq!(collection.evidence_items.len(), 2);
        assert_eq!(collection.evidence_items[0].symbol.as_deref(), Some("foo"));
        assert_eq!(collection.evidence_items[1].symbol.as_deref(), Some("bar"));
        assert_eq!(collection.stats.files_processed, 1);
        assert_eq!(collection.stats.files_skipped, 0);
        assert_eq!(collection.stats.total_items, 2);
        // 索引完整性
        assert!(!collection.index_by_path.is_empty());
        assert!(!collection.index_by_kind.is_empty());
        assert_eq!(collection.index_by_symbol.len(), 2);
        // stats 分组
        assert_eq!(collection.stats.items_by_kind.get("python_stage"), Some(&2));
        assert_eq!(collection.stats.items_by_strength.get("direct"), Some(&2));
    }

    #[test]
    fn col_02_verilog_stage() {
        let dir = TestDir::new("col_02");
        let v_path = dir.write("top.v", b"module top(\n    input clk\n);\nendmodule");

        let ctx = make_stage(vec![make_file(&v_path, Language::Verilog, SourceKind::Rtl)]);
        let mut collector = EvidenceCollector::new("RTL");
        let collection = collector.collect_from_stage_context(&ctx);

        // P1: module + port extraction → 2 items
        assert!(collection.evidence_items.len() >= 2, "应有 module + port, 实际 {}", collection.evidence_items.len());
        assert!(collection.evidence_items.iter().any(|i| i.symbol.as_deref() == Some("top")), "应包含 module top");
        assert!(collection.evidence_items.iter().any(|i| i.symbol.as_deref() == Some("clk")), "应包含 port clk");
        assert_eq!(collection.stats.files_processed, 1);
        assert_eq!(collection.stats.items_by_kind.get("rtl"), Some(&collection.stats.total_items));
    }

    #[test]
    fn col_03_markdown_file() {
        let dir = TestDir::new("col_03");
        let md_path = dir.write(
            "readme.md",
            b"# Project\n\nIntro text\n\n## Setup\n\nSteps here",
        );

        let ctx = make_stage(vec![make_file(&md_path, Language::Markdown, SourceKind::Doc)]);
        let mut collector = EvidenceCollector::new("DOC");
        let collection = collector.collect_from_stage_context(&ctx);

        assert!(
            collection.evidence_items.len() >= 2,
            "至少 2 个标题，实际 {}",
            collection.evidence_items.len()
        );
        assert_eq!(collection.stats.files_processed, 1);
        assert_eq!(collection.stats.items_by_kind.get("doc"), Some(&collection.stats.total_items));
    }

    #[test]
    fn col_04_config_file() {
        let dir = TestDir::new("col_04");
        let xdc_path = dir.write(
            "constraints.xdc",
            b"create_clock -period 10 [get_ports clk]\nproc build {}",
        );

        let ctx = make_stage(vec![make_file(&xdc_path, Language::Unknown, SourceKind::Config)]);
        let mut collector = EvidenceCollector::new("CFG");
        let collection = collector.collect_from_stage_context(&ctx);

        assert_eq!(collection.evidence_items.len(), 2);
        let symbols: Vec<_> = collection
            .evidence_items
            .iter()
            .map(|i| i.symbol.as_deref())
            .collect();
        assert!(symbols.contains(&Some("build")));
        assert!(symbols.contains(&Some("create_clock")));
        // strength 分组
        assert_eq!(collection.stats.items_by_strength.get("direct"), Some(&1));
        assert_eq!(collection.stats.items_by_strength.get("indirect"), Some(&1));
    }

    #[test]
    fn col_05_empty_file() {
        let dir = TestDir::new("col_05");
        let py_path = dir.write("empty.py", b"");

        let ctx = make_stage(vec![make_file(&py_path, Language::Python, SourceKind::PythonStage)]);
        let mut collector = EvidenceCollector::new("L0");
        let collection = collector.collect_from_stage_context(&ctx);

        assert_eq!(collection.evidence_items.len(), 0);
        assert_eq!(collection.stats.files_processed, 1, "空文件也算 processed");
        assert_eq!(collection.stats.total_items, 0);
    }

    #[test]
    fn col_06_plain_python_no_def() {
        let dir = TestDir::new("col_06");
        let py_path = dir.write("script.py", b"x = 1\ny = 2\nprint(x + y)\n");

        let ctx = make_stage(vec![make_file(&py_path, Language::Python, SourceKind::PythonStage)]);
        let mut collector = EvidenceCollector::new("L0");
        let collection = collector.collect_from_stage_context(&ctx);

        assert_eq!(collection.evidence_items.len(), 0, "无 def/class → 0 item");
        assert_eq!(collection.stats.files_processed, 1);
    }

    // ─── 跳过场景测试 ──────────────────────────────────────────────

    #[test]
    fn col_07_large_file_skipped() {
        let dir = TestDir::new("col_07");
        let large = vec![b'a'; (MAX_FILE_SIZE + 1) as usize];
        let path = dir.write("big.v", &large);

        let ctx = make_stage(vec![make_file(&path, Language::Verilog, SourceKind::Rtl)]);
        let mut collector = EvidenceCollector::new("RTL");
        let collection = collector.collect_from_stage_context(&ctx);

        assert_eq!(collection.stats.files_skipped, 1);
        assert_eq!(collection.stats.files_processed, 0);
        assert!(
            collection
                .warnings
                .iter()
                .any(|w| w.error_code == ErrorCode::FileTooLarge),
            "应有 file_too_large warning"
        );
    }

    #[test]
    fn col_08_binary_file_skipped() {
        let dir = TestDir::new("col_08");
        let bin_path = dir.write("binary.bin", &[0x00, 0x01, 0x02, 0x03]);

        let ctx = make_stage(vec![make_file(
            &bin_path,
            Language::Unknown,
            SourceKind::ExternalModule,
        )]);
        let mut collector = EvidenceCollector::new("L0");
        let collection = collector.collect_from_stage_context(&ctx);

        assert_eq!(collection.stats.files_skipped, 1);
        assert!(
            collection
                .warnings
                .iter()
                .any(|w| w.error_code == ErrorCode::BinaryFileSkipped),
            "应有 binary_file_skipped warning"
        );
    }

    #[test]
    fn col_09_non_utf8_file_skipped() {
        let dir = TestDir::new("col_09");
        // 无 NUL 但非合法 UTF-8
        let path = dir.write("bad.txt", &[0xFF, 0xFE, 0xFD]);

        let ctx = make_stage(vec![make_file(&path, Language::Text, SourceKind::Doc)]);
        let mut collector = EvidenceCollector::new("L0");
        let collection = collector.collect_from_stage_context(&ctx);

        assert_eq!(collection.stats.files_skipped, 1);
        assert!(
            collection
                .warnings
                .iter()
                .any(|w| w.error_code == ErrorCode::NonUtf8FileSkipped),
            "应有 non_utf8_file_skipped warning"
        );
    }

    // ─── summary 截断测试 ─────────────────────────────────────────

    #[test]
    fn col_10_summary_truncated() {
        let dir = TestDir::new("col_10");
        // Python 函数，raw_excerpt > 500 字符
        let long_var: String = "x".repeat(600);
        let content = format!("def long_fn():\n    var = \"{}\"\n    pass\n", long_var);
        let py_path = dir.write("long.py", content.as_bytes());

        let ctx = make_stage(vec![make_file(&py_path, Language::Python, SourceKind::PythonStage)]);
        let mut collector = EvidenceCollector::new("L0");
        let collection = collector.collect_from_stage_context(&ctx);

        assert!(
            !collection.evidence_items.is_empty(),
            "应有至少 1 个 item"
        );
        assert!(
            collection
                .warnings
                .iter()
                .any(|w| w.error_code == ErrorCode::SourceExcerptTruncated),
            "应有截断 warning"
        );
        let item = &collection.evidence_items[0];
        assert!(
            item.summary.contains("已截断"),
            "summary 应包含截断标记: {}",
            item.summary
        );
    }

    // ─── ID 格式与唯一性 ─────────────────────────────────────────

    #[test]
    fn col_11_id_format_uniqueness() {
        let dir = TestDir::new("col_11");
        let py1 = dir.write("a.py", b"def alpha():\n    pass\n");
        let py2 = dir.write("b.py", b"def beta():\n    pass\n");

        let ctx = make_stage(vec![
            make_file(&py1, Language::Python, SourceKind::PythonStage),
            make_file(&py2, Language::Python, SourceKind::PythonStage),
        ]);
        let mut collector = EvidenceCollector::new("L0");
        let collection = collector.collect_from_stage_context(&ctx);

        assert_eq!(collection.evidence_items.len(), 2);

        // ID 格式检查
        for item in &collection.evidence_items {
            assert!(
                item.evidence_id.starts_with("EV-L0-"),
                "ID 前缀错误: {}",
                item.evidence_id
            );
            let suffix = &item.evidence_id[6..]; // "EV-L0-".len() == 6
            assert_eq!(suffix.len(), 6, "ID 后缀长度应为 6: {}", item.evidence_id);
            assert!(
                suffix.chars().all(|c| c.is_ascii_digit()),
                "ID 后缀应为纯数字: {}",
                item.evidence_id
            );
        }

        // ID 唯一性
        let ids: HashSet<String> = collection
            .evidence_items
            .iter()
            .map(|i| i.evidence_id.clone())
            .collect();
        assert_eq!(ids.len(), collection.evidence_items.len(), "ID 应唯一");
    }

    // ─── line_range 合法性 + 索引覆盖率 ──────────────────────────

    #[test]
    fn col_12_line_range_and_index_coverage() {
        let dir = TestDir::new("col_12");
        let py = dir.write("code.py", b"def foo():\n    pass\n\ndef bar():\n    x = 1\n");
        let md = dir.write("doc.md", b"# Title\n\nContent\n\n## Section\n\nMore");

        let ctx = make_stage(vec![
            make_file(&py, Language::Python, SourceKind::PythonStage),
            make_file(&md, Language::Markdown, SourceKind::Doc),
        ]);
        let mut collector = EvidenceCollector::new("TEST");
        let collection = collector.collect_from_stage_context(&ctx);

        // line_range 合法性
        for item in &collection.evidence_items {
            assert!(
                item.line_range.start >= 1,
                "line_range.start < 1: item={}",
                item.evidence_id
            );
            assert!(
                item.line_range.start <= item.line_range.end,
                "line_range.start > end: item={}, start={}, end={}",
                item.evidence_id,
                item.line_range.start,
                item.line_range.end,
            );
        }

        // index_by_path 覆盖所有 item
        for item in &collection.evidence_items {
            let path_ids = collection.index_by_path.get(&item.source_path);
            assert!(
                path_ids.is_some(),
                "index_by_path 缺少 {}",
                item.source_path
            );
            assert!(
                path_ids.unwrap().contains(&item.evidence_id),
                "index_by_path[{}] 缺少 {}",
                item.source_path,
                item.evidence_id,
            );
        }

        // index_by_kind 覆盖所有 item
        let all_kind_ids: Vec<&String> = collection.index_by_kind.values().flatten().collect();
        for item in &collection.evidence_items {
            assert!(
                all_kind_ids.contains(&&item.evidence_id),
                "index_by_kind 缺少 {}",
                item.evidence_id,
            );
        }

        assert_eq!(collection.stats.files_processed, 2);
        assert!(
            collection.evidence_items.len() >= 3,
            "至少 2 def + 1 heading，实际 {}",
            collection.evidence_items.len()
        );
    }
}
