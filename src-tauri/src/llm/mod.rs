pub mod fake_provider;
pub mod mock_provider;
pub mod models;
pub mod no_network_guard;
pub mod provider;

pub use fake_provider::FakeProvider;
pub use mock_provider::MockProvider;
pub use models::*;
pub use no_network_guard::{check_network_allowed, network_policy_summary};
pub use provider::{BoxedLlmProvider, LlmProvider};

/// 根据配置创建对应 provider 实例。
///
/// Batch A 行为：
/// - Mock/Fake 直接返回本地 provider。
/// - OpenAi/Anthropic 在 `NetworkMode::Disabled` 时返回 `LlmError::NetworkDisabled`。
/// - 任何允许网络的真实 provider 返回 `LlmError::NotImplemented`，因为 Batch A
///   不实现真实 HTTP 调用。
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
            if config.network_mode == NetworkMode::Allow {
                Err(LlmError::NotImplemented)
            } else {
                Err(LlmError::NetworkDisabled)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::models::ApiKey;

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
    fn create_openai_allowed_not_implemented() {
        let cfg = ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
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
}
