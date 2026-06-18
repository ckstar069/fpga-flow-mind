use crate::llm::models::{LlmError, NetworkMode, ProviderConfig, ProviderKind};

/// 检查给定配置是否允许发起真实网络调用。
///
/// Batch A 规则：
/// - `NetworkMode::Disabled`（默认值）一律禁止真实网络调用。
/// - `NetworkMode::Allow` 仅对真实 provider 生效，但 Batch A 仍会返回
///   `LlmError::NotImplemented`，因为真实 HTTP 调用尚未实现。
/// - Mock/Fake 本地 provider 不经过此守卫。
pub fn check_network_allowed(config: &ProviderConfig) -> Result<(), LlmError> {
    if matches!(config.kind, ProviderKind::Mock | ProviderKind::Fake) {
        return Ok(());
    }

    if config.network_mode == NetworkMode::Disabled {
        return Err(LlmError::NetworkDisabled);
    }

    Ok(())
}

/// 返回一个说明当前网络策略的辅助字符串，用于日志与 UI 提示。
///
/// 注意：此函数不暴露 api_key 或任何敏感配置。
pub fn network_policy_summary(config: &ProviderConfig) -> String {
    match config.network_mode {
        NetworkMode::Disabled => "真实 LLM 网络调用已禁用（Batch A 默认）".to_string(),
        NetworkMode::Proxy => "真实 LLM 网络调用仅限代理/内网（尚未实现）".to_string(),
        NetworkMode::Allow => "真实 LLM 网络调用已显式允许（Batch A 仍会返回未实现）"
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::models::ApiKey;

    #[test]
    fn mock_config_bypasses_guard() {
        let cfg = ProviderConfig::default();
        assert!(check_network_allowed(&cfg).is_ok());
    }

    #[test]
    fn fake_config_bypasses_guard() {
        let cfg = ProviderConfig::fake("fake-model");
        assert!(check_network_allowed(&cfg).is_ok());
    }

    #[test]
    fn openai_disabled_network_is_blocked() {
        let cfg = ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            enabled: true,
            api_key: Some(ApiKey::new("sk-test")),
            ..ProviderConfig::default()
        };
        assert!(matches!(
            check_network_allowed(&cfg),
            Err(LlmError::NetworkDisabled)
        ));
    }

    #[test]
    fn openai_allowed_network_passes_guard_but_not_implemented() {
        let cfg = ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            enabled: true,
            api_key: Some(ApiKey::new("sk-test")),
            network_mode: NetworkMode::Allow,
            ..ProviderConfig::default()
        };
        assert!(check_network_allowed(&cfg).is_ok());
    }

    #[test]
    fn summary_does_not_leak_key() {
        let cfg = ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            enabled: true,
            api_key: Some(ApiKey::new("sk-secret-key")),
            ..ProviderConfig::default()
        };
        let summary = network_policy_summary(&cfg);
        assert!(!summary.contains("secret"));
        assert!(!summary.contains("sk-"));
    }
}
