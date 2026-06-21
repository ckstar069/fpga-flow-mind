use crate::llm::models::{
    ChatMessage, ChatRequest, ChatRole, DegradedReason, LlmError, NetworkMode,
    ProviderCapabilities, ProviderConfig, ProviderKind, ProviderStatus,
};
use crate::llm::provider::LlmProvider;
use crate::llm::real_provider::RealLlmProvider;
use crate::llm::status::{
    ProviderStatusResponse, ProviderTestConnectionResult, ProviderValidationResult,
};
use crate::llm::transport::{HttpTransport, LlmTransport};
use crate::models::enums::ErrorCode;
use crate::models::error::{CommandError, CommandResult};

fn capabilities_for(kind: ProviderKind) -> ProviderCapabilities {
    match kind {
        ProviderKind::Mock | ProviderKind::Fake => ProviderCapabilities {
            understanding: true,
            qa: true,
            structured_output: true,
            max_context_tokens: 0,
        },
        ProviderKind::OpenAi | ProviderKind::Anthropic => ProviderCapabilities::default(),
    }
}

#[tauri::command]
pub fn get_provider_status(config: ProviderConfig) -> CommandResult<ProviderStatusResponse> {
    if !config.enabled {
        return CommandResult {
            success: true,
            data: Some(ProviderStatusResponse {
                status: ProviderStatus::Mock,
                kind: config.kind,
                model: config.model,
                network_mode: config.network_mode,
                can_chat: false,
                degraded_reason: Some(DegradedReason::NotConfigured),
                capabilities: capabilities_for(config.kind),
            }),
            error: None,
            warnings: Vec::new(),
        };
    }

    match config.validate() {
        Ok(()) => {
            let is_local_provider = matches!(config.kind, ProviderKind::Mock | ProviderKind::Fake);
            let is_real_provider =
                matches!(config.kind, ProviderKind::OpenAi | ProviderKind::Anthropic);
            let real_network_allowed =
                config.network_mode == NetworkMode::Allow && is_real_provider;
            let can_chat = is_local_provider || real_network_allowed;

            let status = if is_local_provider {
                ProviderStatus::Mock
            } else if real_network_allowed {
                ProviderStatus::Real
            } else {
                ProviderStatus::Degraded
            };

            let degraded_reason = if is_local_provider || real_network_allowed {
                None
            } else {
                Some(DegradedReason::NetworkDisabled)
            };

            CommandResult {
                success: true,
                data: Some(ProviderStatusResponse {
                    status,
                    kind: config.kind,
                    model: config.model,
                    network_mode: config.network_mode,
                    can_chat,
                    degraded_reason,
                    capabilities: capabilities_for(config.kind),
                }),
                error: None,
                warnings: Vec::new(),
            }
        }
        Err(err) => {
            let message = err.to_string();
            CommandResult {
                success: false,
                data: Some(ProviderStatusResponse {
                    status: ProviderStatus::Degraded,
                    kind: config.kind,
                    model: config.model,
                    network_mode: config.network_mode,
                    can_chat: false,
                    degraded_reason: Some(map_llm_error_to_degraded_reason(&err)),
                    capabilities: capabilities_for(config.kind),
                }),
                error: Some(CommandError {
                    error_code: map_llm_error_to_error_code(&err),
                    message,
                    recoverable: true,
                    details: None,
                    source_path: None,
                }),
                warnings: Vec::new(),
            }
        }
    }
}

#[tauri::command]
pub fn validate_provider_config(config: ProviderConfig) -> CommandResult<ProviderValidationResult> {
    match config.validate() {
        Ok(()) => CommandResult {
            success: true,
            data: Some(ProviderValidationResult {
                valid: true,
                network_enabled: config.would_use_network(),
                issues: Vec::new(),
            }),
            error: None,
            warnings: Vec::new(),
        },
        Err(err) => {
            let message = err.to_string();
            CommandResult {
                success: false,
                data: Some(ProviderValidationResult {
                    valid: false,
                    network_enabled: false,
                    issues: vec![message.clone()],
                }),
                error: Some(CommandError {
                    error_code: map_llm_error_to_error_code(&err),
                    message,
                    recoverable: true,
                    details: None,
                    source_path: None,
                }),
                warnings: Vec::new(),
            }
        }
    }
}

#[tauri::command]
pub fn test_provider_connection(
    config: ProviderConfig,
) -> CommandResult<ProviderTestConnectionResult> {
    if !config.enabled {
        return CommandResult {
            success: true,
            data: Some(ProviderTestConnectionResult {
                success: false,
                code: "not_enabled".to_string(),
                message: "Provider 未启用".to_string(),
            }),
            error: None,
            warnings: Vec::new(),
        };
    }

    if let Err(err) = config.validate() {
        let message = err.to_string();
        return CommandResult {
            success: false,
            data: Some(ProviderTestConnectionResult {
                success: false,
                code: "format_error".to_string(),
                message: message.clone(),
            }),
            error: Some(CommandError {
                error_code: map_llm_error_to_error_code(&err),
                message,
                recoverable: true,
                details: None,
                source_path: None,
            }),
            warnings: Vec::new(),
        };
    }

    if !config.would_use_network() {
        return CommandResult {
            success: true,
            data: Some(ProviderTestConnectionResult {
                success: false,
                code: "network_disabled".to_string(),
                message: "真实网络调用未启用；测试连接未发起请求".to_string(),
            }),
            error: None,
            warnings: Vec::new(),
        };
    }

    let data = test_provider_connection_with_transport(config, HttpTransport);
    CommandResult {
        success: true,
        data: Some(data),
        error: None,
        warnings: Vec::new(),
    }
}

fn test_provider_connection_with_transport<T: LlmTransport>(
    config: ProviderConfig,
    transport: T,
) -> ProviderTestConnectionResult {
    if config.kind != ProviderKind::OpenAi {
        return provider_test_result_from_error(&LlmError::NotImplemented);
    }

    let provider = RealLlmProvider::new(config, transport);
    match provider.chat(&connection_probe_request()) {
        Ok(_response) => ProviderTestConnectionResult {
            success: true,
            code: "connection_ok".to_string(),
            message: "连接成功：已发送最小 ping 请求，未发送项目源码、evidence 或 session 数据"
                .to_string(),
        },
        Err(err) => provider_test_result_from_error(&err),
    }
}

fn connection_probe_request() -> ChatRequest {
    ChatRequest {
        messages: vec![ChatMessage {
            role: ChatRole::User,
            content: "请用一句中文回复：连接测试。".to_string(),
        }],
        system_prompt: Some(
            "这是 fpga-flow-mind 的 LLM provider 连接测试；不得要求或依赖任何项目源码。"
                .to_string(),
        ),
        temperature: Some(0.0),
        max_tokens: Some(32),
    }
}

fn provider_test_result_from_error(err: &LlmError) -> ProviderTestConnectionResult {
    let (code, message) = match err {
        LlmError::NetworkDisabled => ("network_disabled", "真实网络调用未启用；测试连接未发起请求"),
        LlmError::NotConfigured => ("not_configured", "LLM Provider 未配置或未启用"),
        LlmError::MissingApiKey(_) => ("missing_api_key", "Provider 需要提供 API Key"),
        LlmError::InvalidConfig(_) => ("invalid_config", "Provider 配置无效"),
        LlmError::ProviderCallFailed(_) => ("provider_error", "Provider 服务端调用失败"),
        LlmError::NetworkError(_) => (
            "network_error",
            "网络连接失败，请检查 Base URL、代理或超时设置",
        ),
        LlmError::AuthError(_) => ("auth_error", "认证或授权失败，请检查 API Key 与模型权限"),
        LlmError::RateLimited(_) => ("rate_limited", "Provider 返回速率限制，请稍后重试"),
        LlmError::NotImplemented => ("not_implemented", "该 Provider 的真实连接测试尚未实现"),
        LlmError::InvalidResponse(_) => ("invalid_response", "Provider 响应格式无法解析"),
        LlmError::RedactionFailed(_) => ("redaction_failed", "输入脱敏失败，未发起可信连接"),
        LlmError::InvalidInput(_) => ("invalid_input", "连接测试输入无效"),
    };

    ProviderTestConnectionResult {
        success: false,
        code: code.to_string(),
        message: message.to_string(),
    }
}

fn map_llm_error_to_error_code(err: &crate::llm::models::LlmError) -> ErrorCode {
    use crate::llm::models::LlmError;
    match err {
        LlmError::NotConfigured => ErrorCode::LlmProviderNotConfigured,
        LlmError::NetworkDisabled => ErrorCode::LlmNetworkDisabled,
        LlmError::InvalidConfig(_) => ErrorCode::LlmInvalidConfig,
        LlmError::MissingApiKey(_) => ErrorCode::LlmInvalidConfig,
        _ => ErrorCode::LlmInvalidConfig,
    }
}

fn map_llm_error_to_degraded_reason(err: &crate::llm::models::LlmError) -> DegradedReason {
    use crate::llm::models::LlmError;
    match err {
        LlmError::NetworkDisabled => DegradedReason::NetworkDisabled,
        LlmError::NotConfigured => DegradedReason::NotConfigured,
        LlmError::MissingApiKey(_) => DegradedReason::NotConfigured,
        _ => DegradedReason::ProviderError,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::models::ApiKey;
    use crate::llm::transport::{FakeTransport, TransportResponse};

    #[test]
    fn provider_status_default_mock_disabled_no_network() {
        let config = ProviderConfig::default();
        let result = get_provider_status(config);
        assert!(result.success);
        let status = result.data.unwrap();
        assert_eq!(status.status, ProviderStatus::Mock);
        assert_eq!(status.kind, ProviderKind::Mock);
        assert!(!status.can_chat);
        assert_eq!(status.degraded_reason, Some(DegradedReason::NotConfigured));
        assert_eq!(status.network_mode, NetworkMode::Disabled);
    }

    #[test]
    fn validate_provider_config_redacts_api_key() {
        let config = ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            enabled: true,
            api_key: Some(ApiKey::new("this-is-a-fake-key-for-tests")),
            network_mode: NetworkMode::Disabled,
            ..ProviderConfig::default()
        };
        let result = validate_provider_config(config);
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("this-is-a-fake-key-for-tests"));
    }

    #[test]
    fn test_connection_without_network_returns_network_disabled() {
        let config = ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            enabled: true,
            api_key: Some(ApiKey::new("this-is-a-fake-key-for-tests")),
            network_mode: NetworkMode::Disabled,
            ..ProviderConfig::default()
        };
        let result = test_provider_connection(config);
        assert!(result.success);
        let data = result.data.unwrap();
        assert!(!data.success);
        assert_eq!(data.code, "network_disabled");
    }

    #[test]
    fn provider_status_command_does_not_return_api_key() {
        let config = ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            enabled: true,
            api_key: Some(ApiKey::new("this-is-a-fake-key-for-tests")),
            network_mode: NetworkMode::Disabled,
            ..ProviderConfig::default()
        };
        let result = get_provider_status(config);
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("this-is-a-fake-key-for-tests"));
        assert!(!json.contains("api_key"));
    }

    #[test]
    fn validate_provider_config_does_not_persist_api_key() {
        let config = ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            enabled: true,
            api_key: Some(ApiKey::new("this-is-a-fake-key-for-tests")),
            network_mode: NetworkMode::Disabled,
            ..ProviderConfig::default()
        };
        let result = validate_provider_config(config);
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("this-is-a-fake-key-for-tests"));
        assert!(!json.contains("api_key"));
        assert!(result.success);
        assert!(result.data.unwrap().valid);
    }

    #[test]
    fn test_connection_without_explicit_network_returns_disabled() {
        let config = ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            enabled: true,
            api_key: Some(ApiKey::new("this-is-a-fake-key-for-tests")),
            network_mode: NetworkMode::Disabled,
            ..ProviderConfig::default()
        };
        let result = test_provider_connection(config);
        assert!(result.success);
        let data = result.data.unwrap();
        assert!(!data.success);
        assert_eq!(data.code, "network_disabled");

        let config_not_enabled = ProviderConfig::default();
        let result = test_provider_connection(config_not_enabled);
        assert!(result.success);
        let data = result.data.unwrap();
        assert!(!data.success);
        assert_eq!(data.code, "not_enabled");
    }

    #[test]
    fn command_result_redacts_sensitive_fields() {
        let config = ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            enabled: true,
            api_key: Some(ApiKey::new("this-is-a-fake-key-for-tests")),
            network_mode: NetworkMode::Disabled,
            ..ProviderConfig::default()
        };
        let status_json = serde_json::to_string(&get_provider_status(config.clone())).unwrap();
        let validate_json =
            serde_json::to_string(&validate_provider_config(config.clone())).unwrap();
        let test_json = serde_json::to_string(&test_provider_connection(config)).unwrap();

        for json in [&status_json, &validate_json, &test_json] {
            assert!(
                !json.contains("this-is-a-fake-key-for-tests"),
                "json must not contain plaintext key"
            );
            assert!(
                !json.contains("api_key"),
                "json must not contain api_key field"
            );
            assert!(
                !json.contains("Authorization"),
                "json must not contain Authorization header"
            );
            assert!(
                !json.contains("Bearer"),
                "json must not contain Bearer token"
            );
        }
    }

    #[test]
    fn command_does_not_read_env_key() {
        let config = ProviderConfig::default();
        let result = get_provider_status(config);
        assert!(result.success);
        assert!(
            !result.data.unwrap().can_chat,
            "默认配置不得触发真实 provider"
        );
    }

    #[test]
    fn test_connection_allowed_openai_uses_transport() {
        let config = ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "deepseek-chat".to_string(),
            enabled: true,
            api_key: Some(ApiKey::new("this-is-a-fake-key-for-tests")),
            base_url: Some("https://api.deepseek.com".to_string()),
            network_mode: NetworkMode::Allow,
            ..ProviderConfig::default()
        };
        let transport = FakeTransport::new(TransportResponse {
            status_code: 200,
            body: r#"{"choices":[{"message":{"content":"连接测试成功"}}]}"#.to_string(),
        });

        let data = test_provider_connection_with_transport(config, transport);
        assert!(data.success);
        assert_eq!(data.code, "connection_ok");
        assert!(data.message.contains("未发送项目源码"));
    }

    #[test]
    fn test_connection_allowed_openai_maps_auth_error_without_key() {
        let key = "this-is-a-fake-key-for-tests";
        let config = ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "deepseek-chat".to_string(),
            enabled: true,
            api_key: Some(ApiKey::new(key)),
            base_url: Some("https://api.deepseek.com".to_string()),
            network_mode: NetworkMode::Allow,
            ..ProviderConfig::default()
        };
        let transport =
            FakeTransport::new_error(LlmError::AuthError(format!("fake key {} rejected", key)));

        let data = test_provider_connection_with_transport(config, transport);
        let json = serde_json::to_string(&data).unwrap();
        assert!(!data.success);
        assert_eq!(data.code, "auth_error");
        assert!(!json.contains(key));
        assert!(!json.contains("Bearer"));
        assert!(!json.contains("api_key"));
    }

    #[test]
    fn test_connection_anthropic_allowed_returns_not_implemented() {
        let config = ProviderConfig {
            kind: ProviderKind::Anthropic,
            model: "claude-compatible".to_string(),
            enabled: true,
            api_key: Some(ApiKey::new("this-is-a-fake-key-for-tests")),
            network_mode: NetworkMode::Allow,
            ..ProviderConfig::default()
        };
        let result = test_provider_connection(config);
        assert!(result.success);
        let data = result.data.unwrap();
        assert!(!data.success);
        assert_eq!(data.code, "not_implemented");
    }

    #[test]
    #[ignore]
    fn test_connection_deepseek_real_smoke() {
        let Ok(smoke) = std::env::var("FPGA_FLOW_LLM_SMOKE") else {
            return;
        };
        if smoke != "1" {
            return;
        }

        let api_key = std::env::var("FPGA_FLOW_LLM_API_KEY").unwrap_or_default();
        if api_key.is_empty() {
            return;
        }

        let config = ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: std::env::var("FPGA_FLOW_LLM_MODEL")
                .unwrap_or_else(|_| "deepseek-chat".to_string()),
            enabled: true,
            api_key: Some(ApiKey::new(api_key.clone())),
            base_url: Some(
                std::env::var("FPGA_FLOW_LLM_BASE_URL")
                    .unwrap_or_else(|_| "https://api.deepseek.com".to_string()),
            ),
            timeout_ms: 30_000,
            retry_limit: 0,
            rate_limit_per_min: 60,
            network_mode: NetworkMode::Allow,
        };

        let result = test_provider_connection(config);
        let json = serde_json::to_string(&result).unwrap();
        assert!(result.success);
        assert!(result.data.unwrap().success);
        assert!(!json.contains(&api_key));
        assert!(!json.contains("Authorization"));
        assert!(!json.contains("Bearer"));
        assert!(!json.contains("api_key"));
    }

    #[test]
    fn real_provider_status_requires_explicit_enabled_and_network_allow() {
        let mut config = ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            enabled: true,
            api_key: Some(ApiKey::new("this-is-a-fake-key-for-tests")),
            network_mode: NetworkMode::Allow,
            ..ProviderConfig::default()
        };
        let result = get_provider_status(config.clone());
        assert!(result.success);
        let status = result.data.unwrap();
        assert_eq!(status.status, ProviderStatus::Real);
        assert!(status.can_chat);

        config.network_mode = NetworkMode::Disabled;
        let result = get_provider_status(config);
        assert!(result.success);
        let status = result.data.unwrap();
        assert_eq!(status.status, ProviderStatus::Degraded);
        assert!(!status.can_chat);
        assert_eq!(
            status.degraded_reason,
            Some(DegradedReason::NetworkDisabled)
        );
    }

    #[test]
    fn enabled_mock_status_remains_mock() {
        let config = ProviderConfig {
            kind: ProviderKind::Mock,
            model: "mock-model".to_string(),
            enabled: true,
            network_mode: NetworkMode::Allow,
            ..ProviderConfig::default()
        };

        let result = get_provider_status(config);
        assert!(result.success);
        let status = result.data.unwrap();
        assert_eq!(status.status, ProviderStatus::Mock);
        assert!(status.can_chat);
        assert_eq!(status.degraded_reason, None);
    }

    #[test]
    fn grounding_status_maps_unvalidated_to_degraded_or_unknown() {
        let config = ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            enabled: true,
            api_key: None,
            network_mode: NetworkMode::Allow,
            ..ProviderConfig::default()
        };
        let result = get_provider_status(config);
        assert!(!result.success);
        let status = result.data.unwrap();
        assert_eq!(status.status, ProviderStatus::Degraded);
        assert_eq!(status.degraded_reason, Some(DegradedReason::NotConfigured));
    }
}
