use std::collections::HashMap;
use std::time::Instant;

use crate::evidence::models::EvidenceCollection;
use crate::understanding::context_builder::ContextBuilder;
use crate::understanding::models::*;
use crate::understanding::schema_validator::SchemaValidator;

// ─── Provider trait ─────────────────────────────────────────────────

/// 理解生成 Provider 抽象
///
/// Phase 3 实现两个 provider：
/// - MockProvider：基于 known_evidence_ids 生成确定性 mock 输出
/// - ManualProvider：返回 NotConfigured，用于 degraded mode
pub trait UnderstandingProvider: Send + Sync {
    /// 调用生成，返回 ImplementationUnderstanding JSON
    fn generate(&self, input: &GeneratorOutput) -> Result<serde_json::Value, ProviderError>;
}

/// Provider 错误类型
#[derive(Debug)]
pub enum ProviderError {
    /// LLM 调用失败（Phase 3 不使用，预留）
    LlmCallFailed(String),
    /// LLM 返回格式错误（Phase 3 不使用，预留）
    InvalidFormat(String),
    /// LLM 超时（Phase 3 不使用，预留）
    Timeout,
    /// Provider 未配置（ManualProvider 默认行为）
    NotConfigured,
}

// ─── MockProvider ───────────────────────────────────────────────────

/// Mock provider — 基于 known_evidence_ids 生成确定性 mock 输出
///
/// 所有的 evidence_refs 仅使用传入的 known_evidence_ids，
/// 确保通过 SchemaValidator 的 hallucination guard。
pub struct MockProvider;

impl UnderstandingProvider for MockProvider {
    fn generate(&self, input: &GeneratorOutput) -> Result<serde_json::Value, ProviderError> {
        let stage_id = input
            .prompt
            .lines()
            .find(|l| l.starts_with("阶段 ID: "))
            .map(|l| l.trim_start_matches("阶段 ID: ").to_string())
            .unwrap_or_else(|| "L0".to_string());

        let evidence_count = input.known_evidence_ids.len();
        let ids: Vec<&String> = input.known_evidence_ids.iter().collect();

        // 构建确定性 mock ImplementationUnderstanding JSON
        let mut claims = Vec::new();
        let mut module_summaries = Vec::new();

        if !ids.is_empty() {
            // 为前 3 个 evidence（或全部）生成 claims
            let claim_count = ids.len().min(3);
            let categories = [
                "module_structure",
                "signal_definition",
                "data_processing",
            ];
            let confidences = ["confirmed", "supported", "inferred"];

            for i in 0..claim_count {
                let claim_id = format!("CL-{}-{:06}", stage_id, i + 1);
                let ev_id = ids[i].as_str();
                claims.push(serde_json::json!({
                    "claim_id": claim_id,
                    "category": categories[i % categories.len()],
                    "description": format!("基于证据 {} 的声明 {}", ev_id, i + 1),
                    "confidence": confidences[i % confidences.len()],
                    "evidence_refs": [{"evidence_id": ev_id}],
                    "has_evidence_gap": false
                }));

                // 为第一个 claim 生成模块摘要
                if i == 0 {
                    module_summaries.push(serde_json::json!({
                        "name": format!("module_{}", stage_id.to_lowercase()),
                        "description": format!("基于证据 {} 的模块", ev_id),
                        "evidence_refs": [{"evidence_id": ev_id}],
                        "confidence": "supported"
                    }));
                }
            }
        }

        // 如果 evidence 不足，添加 unknowns 和 gaps
        let mut unknowns = Vec::new();
        let mut evidence_gaps = Vec::new();

        if evidence_count < 2 {
            unknowns.push(serde_json::json!({
                "unknown_id": format!("UNK-{}-000001", stage_id),
                "description": "实现细节无法从现有证据推断",
                "related_evidence_refs": if !ids.is_empty() {
                    vec![serde_json::json!({"evidence_id": ids[0]})]
                } else {
                    vec![]
                },
                "reason": "证据数量不足以推断完整实现逻辑"
            }));
        }

        if evidence_count < 3 {
            evidence_gaps.push(serde_json::json!({
                "gap_id": format!("GAP-{}-000001", stage_id),
                "expected_evidence": "更多模块/信号/接口定义证据",
                "reason": "需要更完整的源码覆盖",
                "related_evidence_refs": if !ids.is_empty() {
                    vec![serde_json::json!({"evidence_id": ids[0]})]
                } else {
                    vec![]
                }
            }));
        }

        let short_summary = if ids.is_empty() {
            format!("阶段 {} 暂无充分证据生成完整理解", stage_id)
        } else {
            format!(
                "阶段 {} 包含 {} 条证据，生成了 {} 条声明",
                stage_id,
                evidence_count,
                claims.len()
            )
        };

        let detailed_summary = if ids.is_empty() {
            format!(
                "阶段 {} 当前无可用证据。建议补充源文件后重新收集。",
                stage_id
            )
        } else {
            format!(
                "基于 {} 条证据对阶段 {} 进行了结构化理解分析，识别出 {} 个声明、{} 个模块、{} 个未知项和 {} 个证据缺失。",
                evidence_count,
                stage_id,
                claims.len(),
                module_summaries.len(),
                unknowns.len(),
                evidence_gaps.len()
            )
        };

        // 构建统计
        let mut conf_map = serde_json::Map::new();
        let mut cat_map = serde_json::Map::new();
        for claim in &claims {
            let conf = claim
                .get("confidence")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let count = conf_map
                .get(conf)
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            conf_map.insert(conf.to_string(), serde_json::Value::from(count + 1));

            let cat = claim
                .get("category")
                .and_then(|v| v.as_str())
                .unwrap_or("other");
            let cat_count = cat_map
                .get(cat)
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            cat_map.insert(cat.to_string(), serde_json::Value::from(cat_count + 1));
        }

        Ok(serde_json::json!({
            "stage_id": stage_id,
            "version": "3.0.0",
            "summary": {
                "short": short_summary,
                "detailed": detailed_summary
            },
            "claims": claims,
            "module_summaries": module_summaries,
            "signal_summaries": [],
            "interface_summaries": [],
            "processing_steps": [],
            "unknowns": unknowns,
            "evidence_gaps": evidence_gaps,
            "generation_meta": {
                "provider": "mock",
                "generated_at": "2026-06-12T10:00:00Z",
                "input_evidence_count": evidence_count as u32,
                "generation_time_ms": 10u64,
                "is_degraded": false
            },
            "stats": {
                "total_claims": claims.len() as u32,
                "claims_by_confidence": conf_map,
                "claims_by_category": cat_map,
                "module_count": module_summaries.len() as u32,
                "signal_count": 0u32,
                "interface_count": 0u32,
                "processing_step_count": 0u32,
                "unknown_count": unknowns.len() as u32,
                "evidence_gap_count": evidence_gaps.len() as u32
            }
        }))
    }
}

// ─── ManualProvider ─────────────────────────────────────────────────

/// Manual provider — 返回 NotConfigured，用于 degraded mode
///
/// Phase 3 编码阶段不实现手动 JSON 编辑。ManualProvider 的唯一作用
/// 是触发 UnderstandingGenerator 的 degraded mode 路径。
pub struct ManualProvider;

impl UnderstandingProvider for ManualProvider {
    fn generate(&self, _input: &GeneratorOutput) -> Result<serde_json::Value, ProviderError> {
        Err(ProviderError::NotConfigured)
    }
}

// ─── UnderstandingGenerator ─────────────────────────────────────────

/// Generator 错误类型
#[derive(Debug)]
pub enum GeneratorError {
    /// Provider 错误（非 NotConfigured）
    ProviderError(ProviderError),
    /// Schema 验证失败
    ValidationFailed(Vec<crate::understanding::schema_validator::ValidationError>),
    /// 反序列化失败
    DeserializationError(serde_json::Error),
}

/// 理解生成器 — 编排 ContextBuilder → Provider → SchemaValidator 完整流程
pub struct UnderstandingGenerator {
    provider: Box<dyn UnderstandingProvider>,
}

impl UnderstandingGenerator {
    /// 创建 generator 实例
    pub fn new(provider: Box<dyn UnderstandingProvider>) -> Self {
        Self { provider }
    }

    /// 从 EvidenceCollection 生成 ImplementationUnderstanding
    ///
    /// 流程：
    /// 1. ContextBuilder::build — 确定性预打包
    /// 2. provider.generate — 生成 JSON
    /// 3. SchemaValidator::validate — 验证 + hallucination guard
    /// 4. 反序列化为 ImplementationUnderstanding
    ///
    /// 如果 provider 返回 NotConfigured，生成 degraded understanding。
    pub fn generate(
        &self,
        collection: &EvidenceCollection,
    ) -> Result<ImplementationUnderstanding, GeneratorError> {
        let start = Instant::now();

        // 1. 确定性预打包
        let generator_output = ContextBuilder::build(collection);
        let _elapsed_ms = start.elapsed().as_millis() as u64;

        // 2. 调用 provider
        let raw_output = match self.provider.generate(&generator_output) {
            Ok(v) => v,
            Err(ProviderError::NotConfigured) => {
                // degraded mode — 直接构建，跳过验证
                return Ok(Self::build_degraded_understanding(
                    collection,
                    start.elapsed().as_millis() as u64,
                ));
            }
            Err(e) => return Err(GeneratorError::ProviderError(e)),
        };

        // 3. Schema 验证
        let validation =
            SchemaValidator::validate(&raw_output, &generator_output.known_evidence_ids);

        if !validation.is_valid {
            return Err(GeneratorError::ValidationFailed(validation.errors));
        }

        // 4. 反序列化
        let understanding: ImplementationUnderstanding = serde_json::from_value(raw_output)
            .map_err(GeneratorError::DeserializationError)?;

        Ok(understanding)
    }

    /// 构建 degraded ImplementationUnderstanding
    ///
    /// 当 provider 未配置时生成，语义：
    /// - 不做任何 LLM 推断
    /// - 所有内容标注为 unknown
    /// - 不引用任何不存在的 evidence_id
    /// - 明确告知用户当前为降级模式
    fn build_degraded_understanding(
        collection: &EvidenceCollection,
        generation_time_ms: u64,
    ) -> ImplementationUnderstanding {
        let stage_id = &collection.stage_id;
        let evidence_count = collection.evidence_items.len() as u32;

        ImplementationUnderstanding {
            stage_id: stage_id.clone(),
            version: "3.0.0".to_string(),
            summary: StageSummary {
                short: format!(
                    "阶段 {} 当前未配置语义生成 Provider，无法生成结构化理解",
                    stage_id
                ),
                detailed: format!(
                    "阶段 {} 有 {} 条证据，但当前未配置语义生成 Provider。\
                     以下内容为降级模式自动生成，不包含任何语义推断。",
                    stage_id, evidence_count
                ),
            },
            claims: vec![],
            module_summaries: vec![],
            signal_summaries: vec![],
            interface_summaries: vec![],
            processing_steps: vec![],
            unknowns: vec![UnknownItem {
                unknown_id: format!("UNK-{}-000001", stage_id),
                description: "无法生成结构化理解".to_string(),
                related_evidence_refs: vec![],
                reason: "语义生成 Provider 未配置".to_string(),
            }],
            evidence_gaps: vec![EvidenceGap {
                gap_id: format!("GAP-{}-000001", stage_id),
                expected_evidence: "需要配置 LLM Provider 才能生成结构化理解".to_string(),
                reason: "当前为 degraded mode，无法执行语义分析".to_string(),
                related_evidence_refs: vec![],
            }],
            generation_meta: GenerationMeta {
                provider: "manual".to_string(),
                generated_at: chrono_dummy_timestamp(),
                input_evidence_count: evidence_count,
                generation_time_ms,
                is_degraded: true,
            },
            stats: UnderstandingStats {
                total_claims: 0,
                claims_by_confidence: HashMap::new(),
                claims_by_category: HashMap::new(),
                module_count: 0,
                signal_count: 0,
                interface_count: 0,
                processing_step_count: 0,
                unknown_count: 1,
                evidence_gap_count: 1,
            },
        }
    }
}

/// 生成确定性时间戳（避免 chrono 依赖）
fn chrono_dummy_timestamp() -> String {
    "2026-06-12T00:00:00Z".to_string()
}

// ─── GeneratorOutput 重导出 ─────────────────────────────────────────

// GeneratorOutput 定义在 context_builder.rs，这里重新导出以便 provider 使用
pub use crate::understanding::context_builder::GeneratorOutput;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::models::{EvidenceItem, EvidenceStats, EvidenceStrength, LineRange};
    use crate::models::enums::{Language, SourceKind};
    use std::collections::{HashMap, HashSet};

    /// 构建测试用的 EvidenceCollection
    fn make_collection(stage_id: &str, items: Vec<EvidenceItem>) -> EvidenceCollection {
        let total = items.len() as u32;
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
                total_items: total,
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

    // ─── gen_01: MockProvider 正常生成通过 SchemaValidator ───────────

    #[test]
    fn gen_01_mock_provider_valid_output() {
        let items = vec![
            make_item("EV-L0-000001", Some("mod_a"), "模块 A"),
            make_item("EV-L0-000002", Some("mod_b"), "模块 B"),
            make_item("EV-L0-000003", None, "文件描述"),
        ];
        let collection = make_collection("L0", items);

        let generator = UnderstandingGenerator::new(Box::new(MockProvider));
        let result = generator.generate(&collection);

        assert!(result.is_ok(), "MockProvider 应生成合法输出: {:?}", result.err());
        let understanding = result.unwrap();

        assert_eq!(understanding.stage_id, "L0");
        assert_eq!(understanding.version, "3.0.0");
        assert!(!understanding.summary.short.is_empty());
        assert!(!understanding.claims.is_empty(), "应有至少 1 条 claim");
        assert!(!understanding.generation_meta.is_degraded);
    }

    // ─── gen_02: MockProvider 不引用未知 evidence_id ─────────────────

    #[test]
    fn gen_02_mock_provider_no_unknown_ids() {
        let items = vec![
            make_item("EV-L0-000001", Some("mod_a"), "模块 A"),
        ];
        let collection = make_collection("L0", items);

        let generator = UnderstandingGenerator::new(Box::new(MockProvider));
        let understanding = generator.generate(&collection).unwrap();

        let known_ids: HashSet<String> = collection
            .evidence_items
            .iter()
            .map(|i| i.evidence_id.clone())
            .collect();

        // 检查所有 evidence_refs
        for claim in &understanding.claims {
            for r in &claim.evidence_refs {
                assert!(
                    known_ids.contains(&r.evidence_id),
                    "claim {} 引用了未知 evidence_id: {}",
                    claim.claim_id,
                    r.evidence_id
                );
            }
        }
        for unk in &understanding.unknowns {
            for r in &unk.related_evidence_refs {
                assert!(
                    known_ids.contains(&r.evidence_id),
                    "unknown {} 引用了未知 evidence_id: {}",
                    unk.unknown_id,
                    r.evidence_id
                );
            }
        }
        for gap in &understanding.evidence_gaps {
            for r in &gap.related_evidence_refs {
                assert!(
                    known_ids.contains(&r.evidence_id),
                    "gap {} 引用了未知 evidence_id: {}",
                    gap.gap_id,
                    r.evidence_id
                );
            }
        }
    }

    // ─── gen_03: ManualProvider 返回 degraded understanding ──────────

    #[test]
    fn gen_03_manual_provider_degraded() {
        let items = vec![make_item("EV-L0-000001", Some("mod_a"), "模块 A")];
        let collection = make_collection("L0", items);

        let generator = UnderstandingGenerator::new(Box::new(ManualProvider));
        let understanding = generator.generate(&collection).unwrap();

        assert!(understanding.generation_meta.is_degraded);
        assert_eq!(understanding.generation_meta.provider, "manual");
        assert!(
            understanding.summary.short.contains("未配置"),
            "degraded summary 应说明未配置: {}",
            understanding.summary.short
        );
        assert!(understanding.claims.is_empty(), "degraded 不应有 claims");
        assert!(!understanding.unknowns.is_empty(), "degraded 应有 unknowns");
        assert!(
            !understanding.evidence_gaps.is_empty(),
            "degraded 应有 evidence_gaps"
        );
        // degraded 不引用任何 evidence_id
        for unk in &understanding.unknowns {
            assert!(
                unk.related_evidence_refs.is_empty(),
                "degraded unknown 不应引用 evidence"
            );
        }
    }

    // ─── gen_04: 空 evidence collection → MockProvider 仍成功 ────────

    #[test]
    fn gen_04_empty_collection_mock() {
        let collection = make_collection("L0", vec![]);
        let generator = UnderstandingGenerator::new(Box::new(MockProvider));
        let result = generator.generate(&collection);

        assert!(
            result.is_ok(),
            "空 collection 不应 panic: {:?}",
            result.err()
        );
        let understanding = result.unwrap();
        assert_eq!(understanding.stage_id, "L0");
        assert!(
            !understanding.unknowns.is_empty() || !understanding.evidence_gaps.is_empty(),
            "空 collection 应有 unknowns 或 gaps"
        );
    }

    // ─── gen_05: 空 evidence collection → ManualProvider degraded ────

    #[test]
    fn gen_05_empty_collection_degraded() {
        let collection = make_collection("L0", vec![]);
        let generator = UnderstandingGenerator::new(Box::new(ManualProvider));
        let understanding = generator.generate(&collection).unwrap();

        assert!(understanding.generation_meta.is_degraded);
        assert!(understanding.claims.is_empty());
        assert_eq!(understanding.generation_meta.input_evidence_count, 0);
    }

    // ─── gen_06: BadProvider 返回非法 JSON → ValidationFailed ────────

    struct BadProvider;

    impl UnderstandingProvider for BadProvider {
        fn generate(&self, _input: &GeneratorOutput) -> Result<serde_json::Value, ProviderError> {
            Ok(serde_json::json!({"not": "valid"}))
        }
    }

    #[test]
    fn gen_06_bad_provider_validation_fails() {
        let items = vec![make_item("EV-L0-000001", Some("mod_a"), "模块 A")];
        let collection = make_collection("L0", items);

        let generator = UnderstandingGenerator::new(Box::new(BadProvider));
        let result = generator.generate(&collection);

        assert!(result.is_err(), "BadProvider 应失败");
        match result.unwrap_err() {
            GeneratorError::ValidationFailed(errors) => {
                assert!(!errors.is_empty(), "应有验证错误");
            }
            other => panic!("预期 ValidationFailed，实际: {:?}", other),
        }
    }

    // ─── gen_07: FakeIdProvider 引用不存在的 ID → ValidationFailed ──

    struct FakeIdProvider;

    impl UnderstandingProvider for FakeIdProvider {
        fn generate(&self, _input: &GeneratorOutput) -> Result<serde_json::Value, ProviderError> {
            Ok(serde_json::json!({
                "stage_id": "L0",
                "version": "3.0.0",
                "summary": {"short": "test", "detailed": "test"},
                "claims": [{
                    "claim_id": "CL-L0-000001",
                    "category": "module_structure",
                    "description": "fake claim",
                    "confidence": "confirmed",
                    "evidence_refs": [{"evidence_id": "EV-FAKE-999999"}],
                    "has_evidence_gap": false
                }],
                "module_summaries": [],
                "signal_summaries": [],
                "interface_summaries": [],
                "processing_steps": [],
                "unknowns": [],
                "evidence_gaps": [],
                "generation_meta": {
                    "provider": "fake",
                    "generated_at": "2026-06-12T10:00:00Z",
                    "input_evidence_count": 1,
                    "generation_time_ms": 10,
                    "is_degraded": false
                },
                "stats": {
                    "total_claims": 1,
                    "claims_by_confidence": {},
                    "claims_by_category": {},
                    "module_count": 0,
                    "signal_count": 0,
                    "interface_count": 0,
                    "processing_step_count": 0,
                    "unknown_count": 0,
                    "evidence_gap_count": 0
                }
            }))
        }
    }

    #[test]
    fn gen_07_fake_id_provider_fails() {
        let items = vec![make_item("EV-L0-000001", Some("mod_a"), "模块 A")];
        let collection = make_collection("L0", items);

        let generator = UnderstandingGenerator::new(Box::new(FakeIdProvider));
        let result = generator.generate(&collection);

        assert!(result.is_err(), "引用假 ID 应失败");
        match result.unwrap_err() {
            GeneratorError::ValidationFailed(errors) => {
                assert!(
                    errors.iter().any(|e| matches!(
                        e,
                        crate::understanding::schema_validator::ValidationError::UnknownEvidenceId { .. }
                    )),
                    "应有 UnknownEvidenceId 错误: {:?}",
                    errors
                );
            }
            other => panic!("预期 ValidationFailed，实际: {:?}", other),
        }
    }
}
