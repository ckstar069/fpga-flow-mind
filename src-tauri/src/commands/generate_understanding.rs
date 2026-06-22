/// `generate_understanding` Tauri command
///
/// 完整链路：resolve_stage_context → EvidenceCollector → UnderstandingGenerator → ImplementationUnderstanding
///
/// 返回策略：
/// - resolve 失败 → 透传 CommandResult（success=false）
/// - stage_empty → success=true, degraded understanding（is_degraded=true）
/// - 空 evidence → success=true, MockProvider 生成含 unknowns/gaps 的理解
/// - provider 未配置 → success=true, degraded understanding
/// - 验证失败 → success=false, UnderstandingGenerationFailed error
/// - 正常 → success=true, data=Some(ImplementationUnderstanding)

use crate::evidence::collector::EvidenceCollector;
use crate::llm::{
    ChatMessage, ChatRequest, ChatRole, HttpTransport, LlmError, LlmProvider, NetworkMode,
    ProviderConfig, ProviderKind, RealLlmProvider,
};
use crate::models::enums::ErrorCode;
use crate::models::error::{CommandError, CommandResult, WorkspaceWarning};
use crate::understanding::context_builder::GeneratorOutput;
use crate::understanding::generator::{
    GeneratorError, MockProvider, ProviderError, UnderstandingGenerator, UnderstandingProvider,
};
use crate::understanding::models::ImplementationUnderstanding;

use super::select_stage::resolve_stage_context;

#[tauri::command]
pub fn generate_understanding(
    root_path: String,
    stage_id: String,
    provider_config: Option<ProviderConfig>,
) -> CommandResult<ImplementationUnderstanding> {
    // 1. 复用共享校验 + StageContext 构建
    let ctx_result = resolve_stage_context(&root_path, &stage_id);
    if !ctx_result.success {
        return CommandResult {
            success: false,
            data: None,
            error: ctx_result.error,
            warnings: ctx_result.warnings,
        };
    }

    let context = ctx_result.data.unwrap();

    // 2. 空阶段 → 仍然生成，但用空 evidence collection
    // （MockProvider 会生成含 unknowns/gaps 的理解）

    // 3. 收集 evidence（复用 Phase 2 逻辑）
    let mut collector = EvidenceCollector::new(&stage_id);
    let collection = collector.collect_from_stage_context(&context);

    // 4. 按显式运行态 provider 配置生成理解。默认路径保持 Mock/no-network。
    let result = match provider_config {
        Some(config) if should_use_real_llm(&config) => {
            let provider_label = format!("{}:{}", config.kind, config.model);
            let real_provider = RealLlmProvider::new(config, HttpTransport);
            let generator =
                UnderstandingGenerator::new(Box::new(LlmUnderstandingProvider::new(real_provider)));
            match generator.generate(&collection) {
                Ok(mut understanding) => {
                    understanding.generation_meta.provider = provider_label;
                    understanding.generation_meta.input_evidence_count =
                        collection.evidence_items.len() as u32;
                    Ok((understanding, Vec::new()))
                }
                Err(err) => {
                    let mut fallback_warnings = vec![WorkspaceWarning {
                        error_code: ErrorCode::UnderstandingGenerationFailed,
                        message: format!(
                            "真实 LLM 生成理解失败，已降级为本地 mock 理解：{}",
                            summarize_generator_error(&err)
                        ),
                        source_path: None,
                        related_stage_id: Some(stage_id.clone()),
                        recoverable: true,
                    }];
                    let mock_result = run_mock_generator(&collection).map(|mut understanding| {
                        understanding.generation_meta.provider =
                            "mock_fallback_after_real_llm".to_string();
                        understanding.generation_meta.is_degraded = true;
                        understanding
                    });
                    match mock_result {
                        Ok(understanding) => Ok((understanding, fallback_warnings)),
                        Err(mock_err) => {
                            fallback_warnings.push(WorkspaceWarning {
                                error_code: ErrorCode::UnderstandingGenerationFailed,
                                message: format!(
                                    "本地 fallback 也失败：{}",
                                    summarize_generator_error(&mock_err)
                                ),
                                source_path: None,
                                related_stage_id: Some(stage_id.clone()),
                                recoverable: false,
                            });
                            Err(mock_err)
                        }
                    }
                }
            }
        }
        Some(config) if config.enabled && matches!(config.kind, ProviderKind::OpenAi | ProviderKind::Anthropic) => {
            let mut warnings = Vec::new();
            let reason = if config.network_mode != NetworkMode::Allow {
                "真实网络未启用"
            } else {
                "provider 类型暂不支持"
            };
            warnings.push(WorkspaceWarning {
                error_code: ErrorCode::LlmNetworkDisabled,
                message: format!("{}，已使用本地 mock 理解", reason),
                source_path: None,
                related_stage_id: Some(stage_id.clone()),
                recoverable: true,
            });
            run_mock_generator(&collection).map(|understanding| (understanding, warnings))
        }
        _ => run_mock_generator(&collection).map(|understanding| (understanding, Vec::new())),
    };

    match result {
        Ok((understanding, warnings)) => CommandResult {
            success: true,
            data: Some(understanding),
            error: None,
            warnings,
        },
        Err(GeneratorError::ProviderError(e)) => CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::UnderstandingGenerationFailed,
                message: format!("Provider 错误: {:?}", e),
                recoverable: true,
                details: None,
                source_path: None,
            }),
            warnings: Vec::new(),
        },
        Err(GeneratorError::ValidationFailed(errors)) => CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::UnderstandingGenerationFailed,
                message: format!(
                    "Schema 验证失败: {}",
                    errors
                        .iter()
                        .map(|e| format!("{:?}", e))
                        .collect::<Vec<_>>()
                        .join("; ")
                ),
                recoverable: false,
                details: None,
                source_path: None,
            }),
            warnings: Vec::new(),
        },
        Err(GeneratorError::DeserializationError(e)) => CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::UnderstandingGenerationFailed,
                message: format!("反序列化失败: {}", e),
                recoverable: false,
                details: None,
                source_path: None,
            }),
            warnings: Vec::new(),
        },
    }
}

fn should_use_real_llm(config: &ProviderConfig) -> bool {
    config.enabled
        && config.kind == ProviderKind::OpenAi
        && config.network_mode == NetworkMode::Allow
        && config.api_key.is_some()
}

fn run_mock_generator(
    collection: &crate::evidence::models::EvidenceCollection,
) -> Result<ImplementationUnderstanding, GeneratorError> {
    let generator = UnderstandingGenerator::new(Box::new(MockProvider));
    generator.generate(collection)
}

struct LlmUnderstandingProvider<T: LlmProvider> {
    provider: T,
}

impl<T: LlmProvider> LlmUnderstandingProvider<T> {
    fn new(provider: T) -> Self {
        Self { provider }
    }
}

impl<T: LlmProvider> UnderstandingProvider for LlmUnderstandingProvider<T> {
    fn generate(&self, input: &GeneratorOutput) -> Result<serde_json::Value, ProviderError> {
        let schema = serde_json::to_string_pretty(&input.output_schema)
            .map_err(|_| ProviderError::InvalidFormat("输出 schema 序列化失败".to_string()))?;
        let request = ChatRequest {
            system_prompt: Some(real_understanding_system_prompt().to_string()),
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: format!(
                    "{}\n\n必须输出满足以下 JSON schema 的单个 JSON object，不要输出 Markdown，不要输出代码块，不要输出额外解释：\n{}",
                    input.prompt, schema
                ),
            }],
            temperature: Some(0.0),
            max_tokens: Some(8192),
        };

        let response = self
            .provider
            .chat(&request)
            .map_err(map_llm_error_to_provider_error)?;
        let mut value = parse_json_object_from_llm_content(&response.content)?;
        normalize_llm_understanding_json(&mut value, input);
        Ok(value)
    }
}

fn real_understanding_system_prompt() -> &'static str {
    "你是 fpga-flow-mind 的真实 LLM 理解 provider。你只能基于用户消息里的 evidence 生成 ImplementationUnderstanding JSON。\
必须遵守：1. 只返回 JSON object；2. 不使用 Markdown/code fence；3. 所有非 unknown claim/summary/step 必须引用真实 evidence_id；\
4. 不编造 source path、line number 或 evidence_id；5. 无证据则写 unknown/evidence_gap；6. 不输出 PASS/HOLD/正确/错误/审计结论；\
7. 区分算法数据流与硬件时序，不得把普通 Python 函数顺序说成硬件时序。"
}

fn parse_json_object_from_llm_content(content: &str) -> Result<serde_json::Value, ProviderError> {
    let trimmed = content.trim();
    let candidate = if trimmed.starts_with("```") {
        let without_start = trimmed
            .lines()
            .skip(1)
            .collect::<Vec<_>>()
            .join("\n");
        without_start
            .trim_end()
            .strip_suffix("```")
            .unwrap_or(without_start.trim_end())
            .trim()
            .to_string()
    } else {
        trimmed.to_string()
    };

    let json_slice = if candidate.trim_start().starts_with('{') {
        candidate.trim()
    } else {
        let start = candidate.find('{').ok_or_else(|| {
            ProviderError::InvalidFormat("LLM 返回中未找到 JSON object".to_string())
        })?;
        let end = candidate.rfind('}').ok_or_else(|| {
            ProviderError::InvalidFormat("LLM 返回中未找到 JSON object 结束符".to_string())
        })?;
        &candidate[start..=end]
    };

    serde_json::from_str(json_slice)
        .map_err(|_| ProviderError::InvalidFormat("LLM 返回不是合法 JSON object".to_string()))
}

fn normalize_llm_understanding_json(value: &mut serde_json::Value, input: &GeneratorOutput) {
    {
        let Some(obj) = value.as_object_mut() else {
            return;
        };
        obj.insert(
            "stage_id".to_string(),
            serde_json::Value::String(input.stage_id.clone()),
        );
        obj.entry("version".to_string())
            .or_insert_with(|| serde_json::Value::String("3.0.0".to_string()));

        normalize_id_array(obj, "claims", "claim_id", "CL", &input.stage_id);
        normalize_id_array(obj, "unknowns", "unknown_id", "UNK", &input.stage_id);
        normalize_id_array(obj, "evidence_gaps", "gap_id", "GAP", &input.stage_id);
    }

    let stats = build_stats_from_value(value);
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "generation_meta".to_string(),
            serde_json::json!({
                "provider": "real_llm",
                "generated_at": chrono::Utc::now().to_rfc3339(),
                "input_evidence_count": input.ordered_evidence_ids.len() as u32,
                "generation_time_ms": 0,
                "is_degraded": false
            }),
        );
        obj.insert("stats".to_string(), stats);
    }
}

fn normalize_id_array(
    obj: &mut serde_json::Map<String, serde_json::Value>,
    array_field: &str,
    id_field: &str,
    prefix: &str,
    stage_id: &str,
) {
    let Some(items) = obj.get_mut(array_field).and_then(|v| v.as_array_mut()) else {
        return;
    };
    for (idx, item) in items.iter_mut().enumerate() {
        if let Some(item_obj) = item.as_object_mut() {
            item_obj.insert(
                id_field.to_string(),
                serde_json::Value::String(format!("{}-{}-{:06}", prefix, stage_id, idx + 1)),
            );
        }
    }
}

fn build_stats_from_value(value: &serde_json::Value) -> serde_json::Value {
    let claims = value
        .get("claims")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut by_confidence = serde_json::Map::new();
    let mut by_category = serde_json::Map::new();
    for claim in &claims {
        if let Some(confidence) = claim.get("confidence").and_then(|v| v.as_str()) {
            let current = by_confidence
                .get(confidence)
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            by_confidence.insert(confidence.to_string(), serde_json::json!(current + 1));
        }
        if let Some(category) = claim.get("category").and_then(|v| v.as_str()) {
            let current = by_category
                .get(category)
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            by_category.insert(category.to_string(), serde_json::json!(current + 1));
        }
    }
    serde_json::json!({
        "total_claims": claims.len() as u32,
        "claims_by_confidence": by_confidence,
        "claims_by_category": by_category,
        "module_count": array_len(value, "module_summaries"),
        "signal_count": array_len(value, "signal_summaries"),
        "interface_count": array_len(value, "interface_summaries"),
        "processing_step_count": array_len(value, "processing_steps"),
        "unknown_count": array_len(value, "unknowns"),
        "evidence_gap_count": array_len(value, "evidence_gaps")
    })
}

fn array_len(value: &serde_json::Value, field: &str) -> u32 {
    value
        .get(field)
        .and_then(|v| v.as_array())
        .map(|arr| arr.len() as u32)
        .unwrap_or(0)
}

fn map_llm_error_to_provider_error(err: LlmError) -> ProviderError {
    match err {
        LlmError::NetworkDisabled | LlmError::NotConfigured | LlmError::MissingApiKey(_) => {
            ProviderError::NotConfigured
        }
        LlmError::InvalidResponse(_) => {
            ProviderError::InvalidFormat("Provider 响应格式无法解析".to_string())
        }
        LlmError::NetworkError(_)
        | LlmError::ProviderCallFailed(_)
        | LlmError::AuthError(_)
        | LlmError::RateLimited(_)
        | LlmError::InvalidConfig(_)
        | LlmError::NotImplemented
        | LlmError::RedactionFailed(_)
        | LlmError::InvalidInput(_) => {
            ProviderError::LlmCallFailed("Provider 调用失败或配置不可用".to_string())
        }
    }
}

fn summarize_generator_error(err: &GeneratorError) -> String {
    match err {
        GeneratorError::ProviderError(provider_err) => match provider_err {
            ProviderError::LlmCallFailed(_) => "provider_call_failed".to_string(),
            ProviderError::InvalidFormat(_) => "invalid_provider_output".to_string(),
            ProviderError::Timeout => "timeout".to_string(),
            ProviderError::NotConfigured => "provider_not_configured".to_string(),
        },
        GeneratorError::ValidationFailed(errors) => {
            let summary = errors
                .iter()
                .take(3)
                .map(|e| format!("{:?}", e))
                .collect::<Vec<_>>()
                .join("; ");
            format!("schema_validation_failed: {}", summary)
        }
        GeneratorError::DeserializationError(_) => "deserialization_failed".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::ApiKey;
    use crate::models::enums::ErrorCode;

    /// 辅助：创建临时目录并写入文件
    fn touch(root: &std::path::Path, rel: &str, content: &[u8]) {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, content).unwrap();
    }

    // ─── und_01: 正常场景 — Python 项目端到端 ────────────────────────

    #[test]
    fn und_01_available_stage_generates() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top.py", b"def top():\n    pass\n");
        touch(root, "L0/helper.py", b"def helper():\n    return 1\n");

        let result = generate_understanding(
            root.to_str().unwrap().to_string(),
            "L0".to_string(),
            None,
        );
        assert!(result.success, "可用阶段应成功");
        assert!(result.data.is_some(), "应有 data");
        assert!(result.error.is_none(), "不应有 error: {:?}", result.error);

        let understanding = result.data.unwrap();
        assert_eq!(understanding.stage_id, "L0");
        assert_eq!(understanding.version, "3.0.0");
        assert!(!understanding.claims.is_empty(), "应有至少 1 条 claim");
        assert!(!understanding.generation_meta.is_degraded);
    }

    // ─── und_02: Verilog 阶段端到端 ──────────────────────────────────

    #[test]
    fn und_02_verilog_stage_generates() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(
            root,
            "rtl/top.v",
            b"module top(\n    input clk\n);\nendmodule\n",
        );

        let result = generate_understanding(
            root.to_str().unwrap().to_string(),
            "RTL".to_string(),
            None,
        );
        assert!(result.success, "Verilog 阶段应成功");
        assert!(result.data.is_some());

        let understanding = result.data.unwrap();
        assert_eq!(understanding.stage_id, "RTL");
        assert!(!understanding.claims.is_empty());
    }

    // ─── und_03: 空阶段 → MockProvider 生成含 unknowns/gaps ─────────

    #[test]
    fn und_03_empty_stage_generates_with_gaps() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir(root.join("L0")).unwrap();

        let result = generate_understanding(
            root.to_str().unwrap().to_string(),
            "L0".to_string(),
            None,
        );

        // 空阶段 → 仍然生成 understanding（MockProvider 对空 collection 生成 unknowns/gaps）
        assert!(result.success, "空阶段应返回 success=true");
        assert!(
            result.data.is_some(),
            "应有 data（含 unknowns/gaps）"
        );

        let understanding = result.data.unwrap();
        assert_eq!(understanding.stage_id, "L0");
        assert!(understanding.claims.is_empty(), "空阶段不应有 claims");
        assert!(
            !understanding.unknowns.is_empty() || !understanding.evidence_gaps.is_empty(),
            "空阶段应有 unknowns 或 evidence_gaps"
        );
    }

    // ─── und_04: 无效路径失败 ────────────────────────────────────────

    #[test]
    fn und_04_invalid_root_fails() {
        let result = generate_understanding("/does/not/exist".to_string(), "L0".to_string(), None);
        assert!(!result.success);
        assert_eq!(
            result.error.as_ref().unwrap().error_code,
            ErrorCode::PathNotFound
        );
    }

    // ─── und_05: 不存在的阶段失败 ────────────────────────────────────

    #[test]
    fn und_05_nonexistent_stage_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top.py", b"def top(): pass\n");

        let result = generate_understanding(
            root.to_str().unwrap().to_string(),
            "L9".to_string(),
            None,
        );
        assert!(!result.success);
        assert_eq!(
            result.error.as_ref().unwrap().error_code,
            ErrorCode::NotDirectory
        );
    }

    // ─── und_06: 目标项目只读验证 ────────────────────────────────────

    #[test]
    fn und_06_target_project_readonly() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top.py", b"def process():\n    pass\n");

        let before = std::fs::read_to_string(root.join("L0/top.py")).unwrap();

        let _result = generate_understanding(
            root.to_str().unwrap().to_string(),
            "L0".to_string(),
            None,
        );

        let after = std::fs::read_to_string(root.join("L0/top.py")).unwrap();
        assert_eq!(before, after, "generate_understanding 不应修改目标文件");
    }

    // ─── und_07: 空 stage_id 失败 ────────────────────────────────────

    #[test]
    fn und_07_empty_stage_id_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top.py", b"def top(): pass\n");

        let result = generate_understanding(
            root.to_str().unwrap().to_string(),
            "".to_string(),
            None,
        );
        assert!(!result.success);
    }

    #[test]
    fn und_08_real_provider_disabled_falls_back_to_mock_without_network() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top.py", b"def correlate(samples):\n    return samples\n");
        let cfg = ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "deepseek-chat".to_string(),
            api_key: Some(ApiKey::new("this-is-a-fake-key-for-tests")),
            network_mode: NetworkMode::Disabled,
            enabled: true,
            ..ProviderConfig::default()
        };

        let result = generate_understanding(
            root.to_str().unwrap().to_string(),
            "L0".to_string(),
            Some(cfg),
        );

        assert!(result.success, "禁用网络时应安全降级而不是失败");
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.message.contains("本地 mock 理解"))
        );
        let understanding = result.data.unwrap();
        assert_eq!(understanding.generation_meta.provider, "mock");
    }

    #[test]
    fn und_09_parse_json_object_accepts_fenced_llm_output() {
        let value = parse_json_object_from_llm_content(
            "```json\n{\"stage_id\":\"L0\",\"version\":\"3.0.0\"}\n```",
        )
        .unwrap();
        assert_eq!(value["stage_id"], "L0");
        assert_eq!(value["version"], "3.0.0");
    }

    // ─── und_08: E2E 多阶段完整 pipeline ────────────────────────────

    #[test]
    fn und_10_e2e_multi_stage_pipeline() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let root_str = root.to_str().unwrap();

        // 构建夹具
        touch(
            root,
            "L0/top.py",
            br#""""Top-level signal processing module."""
def process_signal(data, sample_rate):
    normalized = normalize(data)
    return normalized

def normalize(data):
    max_val = max(abs(data))
    return [x / max_val for x in data]
"#,
        );
        touch(
            root,
            "RTL/top.v",
            b"module top(\n    input wire clk\n);\nendmodule\n",
        );

        // collect 前快照
        let l0_before = std::fs::read_to_string(root.join("L0/top.py")).unwrap();
        let rtl_before = std::fs::read_to_string(root.join("RTL/top.v")).unwrap();

        // L0 生成理解
        let l0_result =
            generate_understanding(root_str.to_string(), "L0".to_string(), None);
        assert!(l0_result.success, "L0 应成功");
        let l0_understanding = l0_result.data.unwrap();
        assert_eq!(l0_understanding.stage_id, "L0");
        assert!(!l0_understanding.claims.is_empty());
        assert!(
            l0_understanding.stats.total_claims > 0,
            "L0 应有统计 claims"
        );

        // RTL 生成理解
        let rtl_result =
            generate_understanding(root_str.to_string(), "RTL".to_string(), None);
        assert!(rtl_result.success, "RTL 应成功");
        let rtl_understanding = rtl_result.data.unwrap();
        assert_eq!(rtl_understanding.stage_id, "RTL");
        assert!(!rtl_understanding.claims.is_empty());

        // 只读验证
        assert_eq!(
            l0_before,
            std::fs::read_to_string(root.join("L0/top.py")).unwrap(),
            "L0 文件不应被修改"
        );
        assert_eq!(
            rtl_before,
            std::fs::read_to_string(root.join("RTL/top.v")).unwrap(),
            "RTL 文件不应被修改"
        );
    }

    #[test]
    #[ignore = "需要显式设置 FPGA_FLOW_LLM_SMOKE=1 和 FPGA_FLOW_LLM_API_KEY"]
    fn und_11_real_llm_generate_understanding_smoke() {
        if std::env::var("FPGA_FLOW_LLM_SMOKE").ok().as_deref() != Some("1") {
            eprintln!("skipping real LLM smoke: FPGA_FLOW_LLM_SMOKE != 1");
            return;
        }
        let Ok(api_key) = std::env::var("FPGA_FLOW_LLM_API_KEY") else {
            eprintln!("skipping real LLM smoke: FPGA_FLOW_LLM_API_KEY missing");
            return;
        };
        let model = std::env::var("FPGA_FLOW_LLM_MODEL")
            .unwrap_or_else(|_| "deepseek-chat".to_string());
        let base_url = std::env::var("FPGA_FLOW_LLM_BASE_URL")
            .unwrap_or_else(|_| "https://api.deepseek.com".to_string());

        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(
            root,
            "L0/coarse_sync.py",
            br#"
def correlate(samples, delay):
    return [samples[i] * samples[i - delay].conjugate() for i in range(delay, len(samples))]

def energy(samples, window):
    return [sum(abs(x) ** 2 for x in samples[i:i + window]) for i in range(len(samples) - window)]

def detect_peak(metric):
    return max(range(len(metric)), key=lambda idx: metric[idx])
"#,
        );

        let result = generate_understanding(
            root.to_str().unwrap().to_string(),
            "L0".to_string(),
            Some(ProviderConfig {
                kind: ProviderKind::OpenAi,
                model,
                api_key: Some(ApiKey::new(api_key)),
                base_url: Some(base_url),
                timeout_ms: 60_000,
                retry_limit: 1,
                rate_limit_per_min: 60,
                network_mode: NetworkMode::Allow,
                enabled: true,
            }),
        );

        assert!(result.success, "真实 LLM smoke 应成功: {:?}", result.error);
        let understanding = result.data.unwrap();
        assert!(
            understanding.generation_meta.provider.contains("deepseek")
                || understanding.generation_meta.provider.starts_with("openai:"),
            "应使用真实 provider 或 OpenAI-compatible provider 标签: {}",
            understanding.generation_meta.provider
        );
        assert!(
            !understanding.generation_meta.is_degraded,
            "真实 LLM smoke 不应降级到 mock fallback"
        );
        assert!(!understanding.claims.is_empty(), "真实 LLM 应生成 claims");
    }
}
