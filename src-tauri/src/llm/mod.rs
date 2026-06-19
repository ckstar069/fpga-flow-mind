pub mod fake_provider;
pub mod mock_provider;
pub mod models;
pub mod no_network_guard;
pub mod provider;
pub mod real_provider;
pub mod request_builder;
pub mod response_parser;
pub mod transport;

pub use fake_provider::FakeProvider;
pub use mock_provider::MockProvider;
pub use models::*;
pub use no_network_guard::{check_network_allowed, network_policy_summary};
pub use provider::{BoxedLlmProvider, LlmProvider};
pub use real_provider::RealLlmProvider;
pub use request_builder::RequestBuilder;
pub use response_parser::ResponseParser;
pub use transport::{
    FakeTransport, LlmTransport, NoNetworkTransport, RedactedString, TransportRequest,
    TransportResponse,
};

/// 根据配置创建对应 provider 实例。
///
/// Batch B 行为：
/// - Mock/Fake 直接返回本地 provider。
/// - OpenAi/Anthropic 在 `NetworkMode::Disabled` 时返回 `LlmError::NetworkDisabled`。
/// - OpenAi + `NetworkMode::Allow` → `RealLlmProvider` + `NoNetworkTransport`
///   （Batch B 默认使用 NoNetworkTransport，真实 HTTP transport 留待后续注入）。
/// - Anthropic + `NetworkMode::Allow` → `LlmError::NotImplemented`（Batch B 骨架）。
pub fn create_provider(
    config: &crate::llm::models::ProviderConfig,
) -> Result<BoxedLlmProvider, crate::llm::models::LlmError> {
    use crate::llm::models::{LlmError, NetworkMode, ProviderKind};

    config.validate()?;

    match config.kind {
        ProviderKind::Mock => Ok(Box::new(MockProvider::new(config.model.clone()))),
        ProviderKind::Fake => Ok(Box::new(
            FakeProvider::new("[fake] 默认响应").with_model(config.model.clone()),
        )),
        ProviderKind::OpenAi | ProviderKind::Anthropic => {
            check_network_allowed(config)?;
            // network_mode 已确认为 Allow（Disabled 已被 check_network_allowed 拦截）
            if config.network_mode == NetworkMode::Allow {
                match config.kind {
                    ProviderKind::OpenAi => {
                        // Batch B 默认使用 NoNetworkTransport；
                        // 真实 HTTP transport 在 Batch E 手工验收或显式 smoke 测试中注入。
                        Ok(Box::new(RealLlmProvider::new(
                            config.clone(),
                            NoNetworkTransport,
                        )))
                    }
                    ProviderKind::Anthropic => {
                        // Anthropic 在 Batch B 保留为骨架
                        Err(LlmError::NotImplemented)
                    }
                    _ => unreachable!(),
                }
            } else {
                // 理论上不会到达（check_network_allowed 已拦截 Disabled/Proxy）
                Err(LlmError::NetworkDisabled)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::models::{ApiKey, ChatMessage, ChatRole, ChatRequest};

    #[test]
    fn create_mock_provider() {
        let mut cfg = ProviderConfig::default();
        cfg.enabled = true;
        let provider = create_provider(&cfg).unwrap();
        assert_eq!(provider.provider_name(), "mock");
    }

    #[test]
    fn create_fake_provider() {
        let cfg = ProviderConfig::fake("my-fake");
        let provider = create_provider(&cfg).unwrap();
        assert_eq!(provider.provider_name(), "fake");
        assert_eq!(provider.model_id(), "my-fake");
    }

    #[test]
    fn create_openai_default_blocked() {
        let cfg = ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            enabled: true,
            api_key: Some(ApiKey::new("this-is-a-fake-key-for-tests")),
            ..ProviderConfig::default()
        };
        assert!(matches!(
            create_provider(&cfg),
            Err(LlmError::NetworkDisabled)
        ));
    }

    #[test]
    fn create_openai_allowed_creates_real_provider() {
        let cfg = ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            enabled: true,
            api_key: Some(ApiKey::new("this-is-a-fake-key-for-tests")),
            network_mode: NetworkMode::Allow,
            ..ProviderConfig::default()
        };
        let provider = create_provider(&cfg).unwrap();
        assert_eq!(provider.provider_name(), "openai");
        assert_eq!(provider.model_id(), "gpt-4");

        // 默认使用 NoNetworkTransport，chat() 返回 NetworkDisabled
        let request = ChatRequest {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: "hello".to_string(),
            }],
            system_prompt: None,
            temperature: None,
            max_tokens: None,
        };
        let result = provider.chat(&request);
        assert!(matches!(result, Err(LlmError::NetworkDisabled)));
    }

    #[test]
    fn create_anthropic_allowed_not_implemented() {
        let cfg = ProviderConfig {
            kind: ProviderKind::Anthropic,
            model: "claude-3".to_string(),
            enabled: true,
            api_key: Some(ApiKey::new("this-is-a-fake-key-for-tests")),
            network_mode: NetworkMode::Allow,
            ..ProviderConfig::default()
        };
        assert!(matches!(
            create_provider(&cfg),
            Err(LlmError::NotImplemented)
        ));
    }

    #[test]
    fn create_disabled_config_fails() {
        let cfg = ProviderConfig::default();
        assert!(matches!(
            create_provider(&cfg),
            Err(LlmError::NotConfigured)
        ));
    }

    /// Smoke 测试：验证显式 env 配合同意下，`create_provider` 可创建真实 provider。
    ///
    /// 本测试标记为 `#[ignore]`，默认不运行。
    /// 运行方式：
    /// ```bash
    /// FPGA_FLOW_LLM_SMOKE=1 cargo test --lib real_smoke_requires_env_and_allow -- --ignored
    /// ```
    ///
    /// Batch B 骨架阶段，即使 env 存在，`chat()` 仍返回 `NetworkDisabled`
    /// （因为默认使用 `NoNetworkTransport`）。
    #[test]
    #[ignore]
    fn real_smoke_requires_env_and_allow() {
        // 未设置显式 env 时直接跳过
        let Ok(smoke) = std::env::var("FPGA_FLOW_LLM_SMOKE") else {
            return;
        };
        if smoke.is_empty() {
            return;
        }

        // 仅在此测试中读取 env（默认测试路径不读 env）
        let api_key_str = std::env::var("FPGA_FLOW_LLM_API_KEY").unwrap_or_default();
        if api_key_str.is_empty() {
            return;
        }

        let cfg = ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            enabled: true,
            api_key: Some(ApiKey::new(api_key_str)),
            network_mode: NetworkMode::Allow,
            ..ProviderConfig::default()
        };

        // 断言 opt-in gate 工作：create_provider 可成功返回 RealLlmProvider
        let provider = create_provider(&cfg).unwrap();
        assert_eq!(provider.provider_name(), "openai");

        // Batch B 骨架：chat() 使用 NoNetworkTransport，返回 NetworkDisabled
        let request = ChatRequest {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: "smoke test".to_string(),
            }],
            system_prompt: None,
            temperature: None,
            max_tokens: None,
        };
        let result = provider.chat(&request);
        assert!(matches!(result, Err(LlmError::NetworkDisabled)));
    }
}
