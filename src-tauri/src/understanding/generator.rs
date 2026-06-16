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

// ─── P0-3 保守派生辅助 ──────────────────────────────────────────────

/// 保守派生产物 — 由 evidence symbol/excerpt 确定性派生的 IU 摘要片段。
///
/// 设计原则（诚实优先，不伪造）：
/// - 所有派生项必须绑定真实 evidence_id。
/// - 仅做基于 evidence 顺序与符号/关键词的保守识别，不做算法语义猜测。
/// - 证据不足以派生某类摘要时，对应字段保持空数组；view 层据此输出 empty_reason。
struct DerivedSummaries {
    signal_summaries: Vec<serde_json::Value>,
    interface_summaries: Vec<serde_json::Value>,
    processing_steps: Vec<serde_json::Value>,
}

/// 从单个 evidence context item 的摘要片段中保守识别信号名（用于 signal_summaries）。
///
/// 仅识别明确的方向性 token，不做语义推断。识别到则返回 (name, direction)。
fn extract_signal_from_excerpt(excerpt: &str) -> Vec<(String, Option<String>)> {
    let mut out: Vec<(String, Option<String>)> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    // RTL 端口：input/output <name> / wire/reg <name>
    // 仅匹配行首形式，避免在表达式中间误匹配。
    for raw_line in excerpt.lines() {
        let line = raw_line.trim_start();
        let lower = line.to_lowercase();
        // 跳过注释行
        if lower.starts_with("//") || lower.starts_with("/*") || lower.starts_with("*") {
            continue;
        }
        // input/output port
        if let Some(rest) = lower
            .strip_prefix("input ")
            .or_else(|| lower.strip_prefix("output "))
        {
            let direction = if lower.starts_with("input") {
                Some("input")
            } else {
                Some("output")
            };
            // 提取首个标识符
            if let Some(name) = first_identifier(rest) {
                let key = name.clone();
                if seen.insert(key) {
                    out.push((name, direction.map(|s| s.to_string())));
                }
            }
            continue;
        }
        // 时钟/复位信号关键字（仅当行内出现且非注释）。
        // 仅当行本身是 wire/reg 声明或端口声明时，提取其首个标识符作为时钟信号。
        if (lower.contains("clk") || lower.contains("clock"))
            && (lower.starts_with("wire ") || lower.starts_with("reg ") || lower.starts_with("input "))
        {
            if let Some(name) = first_identifier(&lower) {
                if name.contains("clk") || name.contains("clock") {
                    let key = name.clone();
                    if seen.insert(key) {
                        out.push((name, Some("input".to_string())));
                    }
                }
            }
        }
    }

    // 控制信号总数上限，避免噪声膨胀
    out.truncate(8);
    out
}

/// 提取字符串中第一个合法标识符（字母/下划线开头，含字母数字下划线）。
fn first_identifier(s: &str) -> Option<String> {
    let s = s.trim_start();
    let mut chars = s.char_indices().peekable();
    // 跳过前导非标识符字符（如 `[3:0]`、`wire`、`reg`、`signed`）
    let mut started = false;
    let mut start = 0usize;
    let mut end = 0usize;
    for (i, c) in chars.by_ref() {
        let is_ident_char = c.is_alphanumeric() || c == '_';
        let is_ident_start = c.is_alphabetic() || c == '_';
        if !started {
            if is_ident_start {
                started = true;
                start = i;
                end = i + c.len_utf8();
            }
            // 跳过类型关键字前缀（wire/reg/signed/logic/input/output），继续找下一个标识符
            // 简单处理：遇到标识符就记，但若它是已知类型关键字，则重置
        } else if is_ident_char {
            end = i + c.len_utf8();
        } else {
            break;
        }
    }
    if !started {
        return None;
    }
    let candidate = &s[start..end];
    // 跳过类型关键字，取其后的标识符
    let type_keywords = ["wire", "reg", "logic", "signed", "input", "output", "inout"];
    if type_keywords.contains(&candidate) {
        // 递归取剩余部分
        return first_identifier(&s[end..]);
    }
    Some(candidate.to_string())
}

/// 从 evidence context items 保守派生 signal/interface/processing_step 摘要。
///
/// 规则（保守、可追溯）：
/// - **processing_steps**：对 Python 阶段的函数符号 evidence，按 evidence 顺序生成
///   step（confidence=inferred，绑定 evidence_id）。RTL 阶段不派生 step（避免把
///   硬件 module 当作算法步骤）。每个 step 仅表示“存在该处理单元”，不臆造调用关系。
/// - **signal_summaries**：从 RTL evidence 摘要中识别 input/output 端口与时钟/复位
///   关键字（confidence=inferred，绑定 evidence_id）。
/// - **interface_summaries**：保守不派生（需明确的接口契约证据，本轮不伪造）。
fn derive_conservative_summaries(
    _stage_id: &str,
    items: &[EvidenceContextItem],
) -> DerivedSummaries {
    let mut signal_summaries: Vec<serde_json::Value> = Vec::new();
    let mut processing_steps: Vec<serde_json::Value> = Vec::new();
    let mut sig_seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut step_seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut step_order: u32 = 1;

    for ctx in items {
        let ev_id = &ctx.evidence_id;
        let symbol = ctx.symbol.as_deref().unwrap_or("");
        let is_rtl = ctx.source_kind == "rtl";
        let is_python = ctx.source_kind == "python_stage";

        // ── processing_steps：仅 Python 函数/类符号，按顺序 ──
        // 跳过 dunder 与 __init__ 这类无算法语义的符号。
        if is_python && !symbol.is_empty() && !symbol.starts_with('_') {
            if step_seen.insert(symbol.to_string()) {
                processing_steps.push(serde_json::json!({
                    "name": symbol,
                    "description": format!("保守派生自证据 {} 的处理单元（按 evidence 顺序）", ev_id),
                    "order": step_order,
                    "evidence_refs": [{"evidence_id": ev_id}],
                    "confidence": "inferred"
                }));
                step_order += 1;
            }
        }

        // ── signal_summaries：RTL 端口与时钟/复位 ──
        if is_rtl {
            let sigs = extract_signal_from_excerpt(&ctx.summary);
            for (name, direction) in sigs {
                if sig_seen.insert(name.clone()) {
                    signal_summaries.push(serde_json::json!({
                        "name": name,
                        "description": format!("保守派生自 RTL 证据 {} 的信号", ev_id),
                        "direction": direction,
                        "evidence_refs": [{"evidence_id": ev_id}],
                        "confidence": "inferred"
                    }));
                }
            }
        }
    }

    // 控制处理步骤上限，避免过多噪声（保留 evidence 顺序前 N 个）
    const MAX_STEPS: usize = 12;
    if processing_steps.len() > MAX_STEPS {
        processing_steps.truncate(MAX_STEPS);
    }

    DerivedSummaries {
        signal_summaries,
        interface_summaries: vec![], // 本轮保守不派生 interface（需明确契约证据）
        processing_steps,
    }
}

// ─── MockProvider ───────────────────────────────────────────────────

/// Mock provider — 基于 known_evidence_ids 生成确定性 mock 输出
///
/// 所有的 evidence_refs 仅使用传入的 known_evidence_ids，
/// 确保通过 SchemaValidator 的 hallucination guard。
pub struct MockProvider;

impl UnderstandingProvider for MockProvider {
    fn generate(&self, input: &GeneratorOutput) -> Result<serde_json::Value, ProviderError> {
        // 直接从 input.stage_id 获取，不解析 prompt 文案
        let stage_id = &input.stage_id;
        let evidence_count = input.evidence_context_items.len();

        // 使用 ordered_evidence_ids（确定性顺序，来自 evidence_items Vec）
        // 构建确定性 mock ImplementationUnderstanding JSON
        let mut claims = Vec::new();
        let mut module_summaries = Vec::new();

        if !input.evidence_context_items.is_empty() {
            // 为前 3 个 evidence（或全部）生成 claims
            let claim_count = evidence_count.min(3);
            let categories = [
                "module_structure",
                "signal_definition",
                "data_processing",
            ];
            let confidences = ["confirmed", "supported", "inferred"];

            for i in 0..claim_count {
                let ctx = &input.evidence_context_items[i];
                let ev_id = &ctx.evidence_id;
                let claim_id = format!("CL-{}-{:06}", stage_id, i + 1);
                let desc = if let Some(sym) = &ctx.symbol {
                    format!("基于证据 {} [{}] 的声明 {}", ev_id, sym, i + 1)
                } else {
                    format!("基于证据 {} 的声明 {}", ev_id, i + 1)
                };
                claims.push(serde_json::json!({
                    "claim_id": claim_id,
                    "category": categories[i % categories.len()],
                    "description": desc,
                    "confidence": confidences[i % confidences.len()],
                    "evidence_refs": [{"evidence_id": ev_id}],
                    "has_evidence_gap": false
                }));

                // 为第一个 claim 生成模块摘要
                if i == 0 {
                    let module_name = ctx.symbol.as_deref().unwrap_or("unknown");
                    module_summaries.push(serde_json::json!({
                        "name": format!("module_{}", module_name),
                        "description": format!("基于证据 {} 的模块", ev_id),
                        "evidence_refs": [{"evidence_id": ev_id}],
                        "confidence": "supported"
                    }));
                }
            }
        }

        // ── P0-3 保守派生：从 evidence symbol/excerpt 派生 signals / interfaces /
        //    processing_steps。全部绑定 evidence_id，不做算法语义猜测，仅按 evidence
        //    顺序产出可追溯节点。证据不足的字段保持空数组（由 view 层输出 empty_reason）。
        let derived = derive_conservative_summaries(stage_id, &input.evidence_context_items);
        let signal_summaries = derived.signal_summaries;
        let interface_summaries = derived.interface_summaries;
        let processing_steps = derived.processing_steps;

        // 如果 evidence 不足，添加 unknowns 和 gaps
        let first_ev_id = input.ordered_evidence_ids.first();
        let mut unknowns = Vec::new();
        let mut evidence_gaps = Vec::new();

        if evidence_count < 2 {
            unknowns.push(serde_json::json!({
                "unknown_id": format!("UNK-{}-000001", stage_id),
                "description": "实现细节无法从现有证据推断",
                "related_evidence_refs": match first_ev_id {
                    Some(id) => vec![serde_json::json!({"evidence_id": id})],
                    None => vec![],
                },
                "reason": "证据数量不足以推断完整实现逻辑"
            }));
        }

        if evidence_count < 3 {
            evidence_gaps.push(serde_json::json!({
                "gap_id": format!("GAP-{}-000001", stage_id),
                "expected_evidence": "更多模块/信号/接口定义证据",
                "reason": "需要更完整的源码覆盖",
                "related_evidence_refs": match first_ev_id {
                    Some(id) => vec![serde_json::json!({"evidence_id": id})],
                    None => vec![],
                }
            }));
        }

        let short_summary = if evidence_count == 0 {
            format!("阶段 {} 暂无充分证据生成完整理解", stage_id)
        } else {
            format!(
                "阶段 {} 包含 {} 条证据，生成了 {} 条声明",
                stage_id,
                evidence_count,
                claims.len()
            )
        };

        let detailed_summary = if evidence_count == 0 {
            format!(
                "阶段 {} 当前无可用证据。建议补充源文件后重新收集。",
                stage_id
            )
        } else {
            format!(
                "基于 {} 条证据对阶段 {} 进行了结构化理解分析，识别出 {} 个声明、{} 个模块、{} 个信号、{} 个接口、{} 个处理步骤、{} 个未知项和 {} 个证据缺失。",
                evidence_count,
                stage_id,
                claims.len(),
                module_summaries.len(),
                signal_summaries.len(),
                interface_summaries.len(),
                processing_steps.len(),
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
            "signal_summaries": signal_summaries,
            "interface_summaries": interface_summaries,
            "processing_steps": processing_steps,
            "unknowns": unknowns,
            "evidence_gaps": evidence_gaps,
            "generation_meta": {
                "provider": "mock",
                "generated_at": chrono::Utc::now().to_rfc3339(),
                "input_evidence_count": evidence_count as u32,
                "generation_time_ms": 10u64,
                "is_degraded": false
            },
            "stats": {
                "total_claims": claims.len() as u32,
                "claims_by_confidence": conf_map,
                "claims_by_category": cat_map,
                "module_count": module_summaries.len() as u32,
                "signal_count": signal_summaries.len() as u32,
                "interface_count": interface_summaries.len() as u32,
                "processing_step_count": processing_steps.len() as u32,
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
                generated_at: chrono::Utc::now().to_rfc3339(),
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

// ─── GeneratorOutput 重导出 ─────────────────────────────────────────

// GeneratorOutput, EvidenceContextItem 等定义在 context_builder.rs，这里重新导出
pub use crate::understanding::context_builder::{EvidenceContextItem, GeneratorOutput, IndexSummary, StatsSummary};

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

    // ─── gen_08: MockProvider 输出确定性 ──────────────────────────

    /// 同一 EvidenceCollection 连续两次 generate，claims/evidence_refs 顺序完全一致
    #[test]
    fn gen_08_mock_provider_deterministic() {
        let items = vec![
            make_item("EV-L0-000001", Some("mod_a"), "模块 A"),
            make_item("EV-L0-000002", Some("mod_b"), "模块 B"),
            make_item("EV-L0-000003", Some("mod_c"), "模块 C"),
        ];
        let collection = make_collection("L0", items);

        let generator = UnderstandingGenerator::new(Box::new(MockProvider));

        let result1 = generator.generate(&collection).unwrap();
        let result2 = generator.generate(&collection).unwrap();

        // Stage/structure equality
        assert_eq!(result1.stage_id, result2.stage_id);
        assert_eq!(result1.version, result2.version);
        assert_eq!(result1.claims.len(), result2.claims.len());

        // claims 顺序一致性
        for (i, (c1, c2)) in result1
            .claims
            .iter()
            .zip(result2.claims.iter())
            .enumerate()
        {
            assert_eq!(
                c1.claim_id, c2.claim_id,
                "claim {} id 应一致: {} vs {}",
                i, c1.claim_id, c2.claim_id
            );
            assert_eq!(
                c1.evidence_refs.len(),
                c2.evidence_refs.len(),
                "claim {} evidence_refs 数量应一致",
                i
            );
            for (j, (r1, r2)) in c1
                .evidence_refs
                .iter()
                .zip(c2.evidence_refs.iter())
                .enumerate()
            {
                assert_eq!(
                    r1.evidence_id, r2.evidence_id,
                    "claim {} ref {} 应一致: {} vs {}",
                    i, j, r1.evidence_id, r2.evidence_id
                );
            }
        }

        // 序列化后 JSON 字符串一致（忽略 generated_at，因为使用实时 chrono）
        // 我们将 generated_at 字段归一化后再比较
        let json1 = serde_json::to_value(&result1).unwrap();
        let json2 = serde_json::to_value(&result2).unwrap();

        // 检查 claims 数组完全相同
        let claims1 = json1.get("claims").unwrap();
        let claims2 = json2.get("claims").unwrap();
        assert_eq!(claims1, claims2, "claims JSON 应完全一致");

        // 检查 module_summaries 完全相同
        let mods1 = json1.get("module_summaries").unwrap();
        let mods2 = json2.get("module_summaries").unwrap();
        assert_eq!(mods1, mods2, "module_summaries JSON 应完全一致");

        // 检查 unknowns / evidence_gaps 完全相同
        assert_eq!(
            json1.get("unknowns").unwrap(),
            json2.get("unknowns").unwrap(),
            "unknowns 应一致"
        );
        assert_eq!(
            json1.get("evidence_gaps").unwrap(),
            json2.get("evidence_gaps").unwrap(),
            "evidence_gaps 应一致"
        );
    }

    // ─── gen_09: MockProvider 使用 input.stage_id，不解析 prompt ──

    /// 自定义 GeneratorOutput（prompt 不含 "阶段 ID:"）
    fn make_generator_output(stage_id: &str, ids: Vec<String>, items: Vec<EvidenceContextItem>) -> GeneratorOutput {
        GeneratorOutput {
            stage_id: stage_id.to_string(),
            prompt: "这是一个不含阶段 ID: 前缀的 prompt 文案".to_string(),
            output_schema: serde_json::json!({}),
            known_evidence_ids: ids.iter().cloned().collect(),
            ordered_evidence_ids: ids,
            evidence_context_items: items,
            index_summary: IndexSummary {
                path_count: 0,
                kind_count: 0,
                symbol_count: 0,
            },
            stats_summary: StatsSummary {
                files_processed: 0,
                files_skipped: 0,
                total_items: 0,
            },
            warnings_summary: vec![],
        }
    }

    #[test]
    fn gen_09_mock_provider_uses_input_stage_id() {
        let items = vec![EvidenceContextItem {
            evidence_id: "EV-Z9-000001".to_string(),
            summary: "测试证据".to_string(),
            symbol: Some("test_mod".to_string()),
            language: "verilog".to_string(),
            source_kind: "rtl".to_string(),
            strength: "direct".to_string(),
        }];

        // prompt 不含 "阶段 ID:" 且 stage_id 是 "Z9"（非默认 "L0"）
        let output = make_generator_output(
            "Z9",
            vec!["EV-Z9-000001".to_string()],
            items,
        );

        let provider = MockProvider;
        let result = provider.generate(&output).unwrap();

        assert_eq!(
            result.get("stage_id").and_then(|v| v.as_str()).unwrap(),
            "Z9",
            "stage_id 应从 input.stage_id 获取，不是解析 prompt"
        );

        // claims 的 claim_id 前缀应基于 stage_id
        let claims = result.get("claims").unwrap().as_array().unwrap();
        assert!(
            claims[0].get("claim_id").unwrap().as_str().unwrap().starts_with("CL-Z9-"),
            "claim_id 应包含 Z9 前缀"
        );
    }

    // ─── gen_10: generated_at 使用 chrono 真实时间 ──────────────────

    #[test]
    fn gen_10_generated_at_uses_chrono() {
        let items = vec![make_item("EV-L0-000001", Some("mod_a"), "模块 A")];
        let collection = make_collection("L0", items);

        let generator = UnderstandingGenerator::new(Box::new(MockProvider));
        let understanding = generator.generate(&collection).unwrap();

        let ts = &understanding.generation_meta.generated_at;
        assert!(!ts.is_empty(), "generated_at 不应为空");
        assert!(
            ts.contains('T'),
            "generated_at 应为 ISO 8601 格式含 'T': {}",
            ts
        );
        // 不应是 hardcoded 值
        assert_ne!(
            ts, "2026-06-12T10:00:00Z",
            "generated_at 不应是 hardcoded 值"
        );
        assert_ne!(
            ts, "2026-06-12T00:00:00Z",
            "generated_at 不应是旧 dummy 值"
        );
    }

    // ─── gen_11: degraded mode 也使用 chrono ───────────────────────

    #[test]
    fn gen_11_degraded_uses_chrono() {
        let items = vec![make_item("EV-L0-000001", Some("mod_a"), "模块 A")];
        let collection = make_collection("L0", items);

        let generator = UnderstandingGenerator::new(Box::new(ManualProvider));
        let understanding = generator.generate(&collection).unwrap();

        assert!(understanding.generation_meta.is_degraded);
        let ts = &understanding.generation_meta.generated_at;
        assert!(!ts.is_empty());
        assert!(ts.contains('T'), "degraded generated_at 应为 ISO 8601: {}", ts);
        assert_ne!(ts, "2026-06-12T00:00:00Z", "degraded 不应使用旧 dummy 值");
    }

    // ─── gen_12 ~ gen_16: P0-3 保守派生测试 ─────────────────────────

    fn make_item_with_kind(
        id: &str,
        symbol: Option<&str>,
        summary: &str,
        source_kind: SourceKind,
        language: Language,
    ) -> EvidenceItem {
        EvidenceItem {
            evidence_id: id.to_string(),
            source_path: "/tmp/test".to_string(),
            language,
            source_kind,
            line_range: LineRange { start: 1, end: 5 },
            symbol: symbol.map(|s| s.to_string()),
            summary: summary.to_string(),
            strength: EvidenceStrength::Direct,
        }
    }

    /// gen_12: Python 函数符号 evidence → processing_steps（按 evidence 顺序）
    #[test]
    fn gen_12_python_symbols_derive_processing_steps() {
        let items = vec![
            make_item_with_kind("EV-L0-000001", Some("load_samples"), "def load_samples():", SourceKind::PythonStage, Language::Python),
            make_item_with_kind("EV-L0-000002", Some("correlate"), "def correlate(rx):", SourceKind::PythonStage, Language::Python),
            make_item_with_kind("EV-L0-000003", Some("detect_peak"), "def detect_peak(corr):", SourceKind::PythonStage, Language::Python),
        ];
        let collection = make_collection("L0", items);
        let generator = UnderstandingGenerator::new(Box::new(MockProvider));
        let iu = generator.generate(&collection).unwrap();

        assert_eq!(iu.processing_steps.len(), 3, "3 个 Python 函数应派生 3 个 step");
        assert_eq!(iu.processing_steps[0].name, "load_samples");
        assert_eq!(iu.processing_steps[0].order, 1);
        assert_eq!(iu.processing_steps[1].name, "correlate");
        assert_eq!(iu.processing_steps[1].order, 2);
        assert_eq!(iu.processing_steps[2].name, "detect_peak");
        assert_eq!(iu.processing_steps[2].order, 3);
        // 每个 step 必须绑定 evidence_id
        for step in &iu.processing_steps {
            assert!(!step.evidence_refs.is_empty(), "step {} 必须绑定 evidence", step.name);
            assert!(step.evidence_refs[0].evidence_id.starts_with("EV-"));
        }
        // 置信度应为 inferred（保守派生）
        assert_eq!(iu.processing_steps[0].confidence, ClaimConfidence::Inferred);
        // stats 反映派生
        assert_eq!(iu.stats.processing_step_count, 3);
    }

    /// gen_13: dunder / 下划线开头符号不派生 step
    #[test]
    fn gen_13_dunder_symbols_not_derived() {
        let items = vec![
            make_item_with_kind("EV-L0-000001", Some("__init__"), "def __init__(self):", SourceKind::PythonStage, Language::Python),
            make_item_with_kind("EV-L0-000002", Some("_private"), "def _private():", SourceKind::PythonStage, Language::Python),
            make_item_with_kind("EV-L0-000003", Some("public_fn"), "def public_fn():", SourceKind::PythonStage, Language::Python),
        ];
        let collection = make_collection("L0", items);
        let iu = UnderstandingGenerator::new(Box::new(MockProvider)).generate(&collection).unwrap();

        assert_eq!(iu.processing_steps.len(), 1, "仅 public_fn 应派生 step");
        assert_eq!(iu.processing_steps[0].name, "public_fn");
    }

    /// gen_14: RTL evidence 派生 signals（input/output 端口 + clk）
    #[test]
    fn gen_14_rtl_derives_signals() {
        let rtl_summary = "module coarse_sync(\n    input clk,\n    input rst_n,\n    input [11:0] rx_data,\n    output [11:0] peak_idx\n);";
        let items = vec![make_item_with_kind(
            "EV-RTL-000001",
            Some("coarse_sync"),
            rtl_summary,
            SourceKind::Rtl,
            Language::Verilog,
        )];
        let collection = make_collection("RTL", items);
        let iu = UnderstandingGenerator::new(Box::new(MockProvider)).generate(&collection).unwrap();

        // 应识别出多个信号（clk/rst_n/rx_data/peak_idx）
        assert!(!iu.signal_summaries.is_empty(), "RTL evidence 应派生 signals");
        let names: Vec<&str> = iu.signal_summaries.iter().map(|s| s.name.as_str()).collect();
        assert!(names.iter().any(|n| n.contains("clk")), "应识别 clk，实际: {:?}", names);
        assert!(names.iter().any(|n| n.contains("rx_data") || n.contains("data")), "应识别数据端口");
        // 每个信号绑定 evidence_id
        for sig in &iu.signal_summaries {
            assert!(!sig.evidence_refs.is_empty(), "信号 {} 必须绑定 evidence", sig.name);
        }
        // RTL 不派生 processing_steps（不把 module 当算法步骤）
        assert!(iu.processing_steps.is_empty(), "RTL 不应派生 processing_steps");
    }

    /// gen_15: 空 evidence collection 不派生任何摘要
    #[test]
    fn gen_15_empty_evidence_no_derivation() {
        let collection = make_collection("L0", vec![]);
        let iu = UnderstandingGenerator::new(Box::new(MockProvider)).generate(&collection).unwrap();

        assert!(iu.processing_steps.is_empty());
        assert!(iu.signal_summaries.is_empty());
        assert!(iu.interface_summaries.is_empty());
    }

    /// gen_16: 派生项全部引用真实 evidence_id（hallucination guard）
    #[test]
    fn gen_16_derived_refs_all_real() {
        let items = vec![
            make_item_with_kind("EV-L0-000001", Some("foo"), "def foo():", SourceKind::PythonStage, Language::Python),
            make_item_with_kind("EV-L0-000002", Some("bar"), "def bar():", SourceKind::PythonStage, Language::Python),
        ];
        let collection = make_collection("L0", items);
        let iu = UnderstandingGenerator::new(Box::new(MockProvider)).generate(&collection).unwrap();

        let known: HashSet<String> = collection.evidence_items.iter().map(|i| i.evidence_id.clone()).collect();
        for step in &iu.processing_steps {
            for r in &step.evidence_refs {
                assert!(known.contains(&r.evidence_id), "step {} 引用未知 evidence_id: {}", step.name, r.evidence_id);
            }
        }
        for sig in &iu.signal_summaries {
            for r in &sig.evidence_refs {
                assert!(known.contains(&r.evidence_id), "signal {} 引用未知 evidence_id: {}", sig.name, r.evidence_id);
            }
        }
    }

    // ─── P0-3 真实项目只读验证 harness（#[ignore]，手动 --ignored 触发） ────
    //
    // 直接读取真实项目 /Users/ckstar/Repo/znxt_ofdm/fpga_project_coarse_sync 源码
    // （只读：std::fs::read，绝不写入），跑 collect → understanding → views →
    // quality，打印每阶段 nodes/edges/issues 统计。不创建临时目录，不写目标项目。

    use crate::evidence::collector::EvidenceCollector;
    use crate::models::stage_context::{StageContext, StageFile};
    use crate::quality::view_evaluator::{ViewEvaluator, ViewEvaluatorInput};
    use crate::views::generator::ViewGraphGenerator;

    fn stage_files_for(dir: &str, stage_id: &str) -> Vec<StageFile> {
        let base = format!(
            "/Users/ckstar/Repo/znxt_ofdm/fpga_project_coarse_sync/{}",
            dir
        );
        let mut files = Vec::new();
        collect_source_files_recursive(std::path::Path::new(&base), &mut files);
        // 调试输出（测试日志可见）
        eprintln!("[harness] stage={} dir={} files={}", stage_id, dir, files.len());
        files
    }

    fn collect_source_files_recursive(base: &std::path::Path, files: &mut Vec<StageFile>) {
        let entries = match std::fs::read_dir(base) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // 跳过噪声目录
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if matches!(name, "__pycache__" | ".git" | ".claude" | "node_modules" | "target") {
                    continue;
                }
                collect_source_files_recursive(&path, files);
                continue;
            }
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let (language, source_kind) = match ext {
                    "py" => (Language::Python, SourceKind::PythonStage),
                    "v" => (Language::Verilog, SourceKind::Rtl),
                    "sv" => (Language::SystemVerilog, SourceKind::Rtl),
                    _ => continue,
                };
                files.push(StageFile {
                    source_path: path.to_string_lossy().to_string(),
                    language,
                    source_kind,
                    size_bytes: std::fs::metadata(&path).ok().map(|m| m.len()),
                });
            }
        }
    }

    fn run_stage_pipeline(stage_id: &str, dir: &str) -> String {
        let files = stage_files_for(dir, stage_id);
        let stage_context = StageContext {
            stage_id: stage_id.to_string(),
            source_path: dir.to_string(),
            files,
            external_deps: vec![],
            upstream_refs: vec![],
            error_code: None,
        };
        let mut collector = EvidenceCollector::new(stage_id);
        let collection = collector.collect_from_stage_context(&stage_context);
        let ev_count = collection.evidence_items.len();

        let iu = UnderstandingGenerator::new(Box::new(MockProvider)).generate(&collection).unwrap();
        let graphs = ViewGraphGenerator::generate_all(&iu);

        let ev_id_set: HashSet<String> = collection.evidence_items.iter().map(|i| i.evidence_id.clone()).collect();
        let cl_id_set: HashSet<String> = iu.claims.iter().map(|c| c.claim_id.clone()).collect();

        let mut report_lines = vec![format!(
            "[{}] evidence={} claims={} steps={} signals={} modules={}",
            stage_id, ev_count, iu.claims.len(), iu.processing_steps.len(), iu.signal_summaries.len(), iu.module_summaries.len()
        )];

        for graph in &graphs {
            let vt = format!("{:?}", graph.view_type).to_lowercase();
            let (_rpt, issues) = ViewEvaluator::evaluate(&ViewEvaluatorInput {
                sample_id: stage_id,
                stage_id,
                view: graph,
                evidence_id_set: &ev_id_set,
                claim_id_set: &cl_id_set,
            });
            let empty_medium = issues.iter().filter(|i| {
                i.kind == crate::quality::models::QualityIssueKind::EmptyOrUnhelpfulView
                    && i.severity == crate::quality::models::QualitySeverity::Medium
            }).count();
            let empty_low = issues.iter().filter(|i| {
                i.kind == crate::quality::models::QualityIssueKind::EmptyOrUnhelpfulView
                    && i.severity == crate::quality::models::QualitySeverity::Low
            }).count();
            report_lines.push(format!(
                "  {} nodes={} edges={} empty_reason={} | empty_medium={} empty_low={}",
                vt,
                graph.nodes.len(),
                graph.edges.len(),
                graph.meta.empty_reason.is_some(),
                empty_medium,
                empty_low,
            ));
        }
        report_lines.join("\n")
    }

    #[test]
    #[ignore]
    fn p03_real_project_readonly_baseline() {
        // 只读：仅 std::fs::read_dir / read / metadata，不写目标项目。
        let l0 = run_stage_pipeline("L0", "src/python_model/L0_external");
        let l1 = run_stage_pipeline("L1", "src/python_model/L1_prototype");
        let rtl = run_stage_pipeline("RTL", "src/verilog_model/rtl");
        eprintln!("\n===== P0-3 真实项目只读基线 =====\n{}\n\n{}\n\n{}\n========================", l0, l1, rtl);
    }
}
