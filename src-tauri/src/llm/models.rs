use serde::{Deserialize, Serialize};
use std::fmt;

/// LLM Provider 类别。
///
/// Batch A 仅实现 Mock/Fake；OpenAi/Anthropic 为占位，用于验证 no-network guard。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    /// 确定性本地 heuristic/mock，不调用网络。
    Mock,
    /// 可注入固定响应的 fake provider，仅用于测试与开发。
    Fake,
    /// OpenAI 兼容 provider（Batch A 占位，真实调用被禁用）。
    OpenAi,
    /// Anthropic 兼容 provider（Batch A 占位，真实调用被禁用）。
    Anthropic,
}

impl fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderKind::Mock => write!(f, "mock"),
            ProviderKind::Fake => write!(f, "fake"),
            ProviderKind::OpenAi => write!(f, "openai"),
            ProviderKind::Anthropic => write!(f, "anthropic"),
        }
    }
}

/// 网络访问模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkMode {
    /// 默认：禁止真实网络访问。
    Disabled,
    /// 仅允许通过代理/本地内网访问（后续扩展）。
    Proxy,
    /// 显式允许真实网络访问。
    Allow,
}

impl Default for NetworkMode {
    fn default() -> Self {
        NetworkMode::Disabled
    }
}

/// api_key 的安全包装。
///
/// - 不实现 `Display`，防止意外打印。
/// - 不实现 `Serialize`，防止写回日志/session。
/// - `Debug` 输出脱敏。
#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct ApiKey(String);

impl ApiKey {
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    pub fn expose_secret(&self) -> &str {
        &self.0
    }

    /// 返回脱敏后的展示字符串。
    pub fn masked(&self) -> String {
        let key = &self.0;
        if key.len() <= 8 {
            "***".to_string()
        } else {
            format!("{}...{}", &key[..4], &key[key.len() - 4..])
        }
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiKey([REDACTED])")
    }
}

/// Provider 能力声明。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProviderCapabilities {
    pub understanding: bool,
    pub qa: bool,
    pub structured_output: bool,
    pub max_context_tokens: u32,
}

impl Default for ProviderCapabilities {
    fn default() -> Self {
        Self {
            understanding: true,
            qa: true,
            structured_output: true,
            max_context_tokens: 8192,
        }
    }
}

/// Provider 运行状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderStatus {
    /// 使用 Mock/Fake 本地 provider。
    Mock,
    /// 真实 provider 已启用且成功调用。
    Real,
    /// 已启用真实 provider，但因失败/超时/取消/校验失败降级。
    Degraded,
    /// 证据不足，返回 unknown。
    Unknown,
}

/// Provider 配置。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProviderConfig {
    pub kind: ProviderKind,
    pub model: String,
    #[serde(skip_serializing)]
    pub api_key: Option<ApiKey>,
    pub base_url: Option<String>,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_retry_limit")]
    pub retry_limit: u32,
    #[serde(default = "default_rate_limit_per_min")]
    pub rate_limit_per_min: u32,
    #[serde(default)]
    pub network_mode: NetworkMode,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_timeout_ms() -> u64 {
    60_000
}

fn default_retry_limit() -> u32 {
    2
}

fn default_rate_limit_per_min() -> u32 {
    60
}

fn default_enabled() -> bool {
    false
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            kind: ProviderKind::Mock,
            model: "mock".to_string(),
            api_key: None,
            base_url: None,
            timeout_ms: default_timeout_ms(),
            retry_limit: default_retry_limit(),
            rate_limit_per_min: default_rate_limit_per_min(),
            network_mode: NetworkMode::default(),
            enabled: default_enabled(),
        }
    }
}

impl ProviderConfig {
    /// 构造默认 Mock 配置（no-network-by-default）。
    pub fn mock() -> Self {
        Self::default()
    }

    /// 构造一个 Fake provider 配置，用于测试。
    pub fn fake(model: impl Into<String>) -> Self {
        Self {
            kind: ProviderKind::Fake,
            model: model.into(),
            enabled: true,
            ..Self::default()
        }
    }

    /// 校验配置合法性。
    ///
    /// Batch A 规则：
    /// - 未启用时，任何真实 provider 都视为未配置。
    /// - OpenAi/Anthropic 必须提供 api_key。
    /// - 真实 provider 在 network_mode=Disabled 时仍然合法配置，但工厂层会拒绝调用。
    pub fn validate(&self) -> Result<(), LlmError> {
        if !self.enabled {
            return Err(LlmError::NotConfigured);
        }

        if self.model.trim().is_empty() {
            return Err(LlmError::InvalidConfig("model 不能为空".to_string()));
        }

        match self.kind {
            ProviderKind::Mock | ProviderKind::Fake => Ok(()),
            ProviderKind::OpenAi | ProviderKind::Anthropic => {
                if self.api_key.is_none() {
                    return Err(LlmError::MissingApiKey(self.kind));
                }
                Ok(())
            }
        }
    }

    /// 判断当前配置是否会尝试真实网络调用。
    pub fn would_use_network(&self) -> bool {
        self.enabled
            && matches!(self.kind, ProviderKind::OpenAi | ProviderKind::Anthropic)
            && self.network_mode == NetworkMode::Allow
    }
}

/// LLM 角色。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

/// 单条聊天消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

/// LLM 请求。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

/// Citation 输出模型。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub evidence_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    pub line_start: u32,
    pub line_end: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt_summary: Option<String>,
}

/// Token 用量。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// 降级原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DegradedReason {
    NetworkDisabled,
    NotConfigured,
    ProviderError,
    Cancelled,
    GroundingFailed,
    Unknown,
}

/// LLM 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub provider: String,
    pub model: String,
    pub is_degraded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degraded_reason: Option<DegradedReason>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub citations: Vec<Citation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

impl ChatResponse {
    pub fn unknown(provider: impl Into<String>, reason: DegradedReason) -> Self {
        Self {
            content: "根据当前证据无法确定。".to_string(),
            provider: provider.into(),
            model: "unknown".to_string(),
            is_degraded: true,
            degraded_reason: Some(reason),
            citations: vec![],
            usage: None,
        }
    }
}

/// LLM Provider 错误类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmError {
    /// 网络访问被禁用（no-network-by-default）。
    NetworkDisabled,
    /// 配置未启用或不完整。
    NotConfigured,
    /// 缺少 api_key。
    MissingApiKey(ProviderKind),
    /// 配置校验失败。
    InvalidConfig(String),
    /// Provider 调用失败。
    ProviderCallFailed(String),
    /// Batch A 占位：真实 provider 尚未实现。
    NotImplemented,
    /// 输入包含敏感数据且无法 redact。
    RedactionFailed(String),
    /// 输入为空或无效。
    InvalidInput(String),
}

impl fmt::Display for LlmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlmError::NetworkDisabled => {
                write!(f, "真实 LLM 调用尚未在 Batch A 启用（network_mode=Disabled）")
            }
            LlmError::NotConfigured => write!(f, "LLM Provider 未配置或未启用"),
            LlmError::MissingApiKey(kind) => write!(f, "Provider {} 需要提供 api_key", kind),
            LlmError::InvalidConfig(msg) => write!(f, "配置无效: {}", msg),
            LlmError::ProviderCallFailed(msg) => write!(f, "Provider 调用失败: {}", msg),
            LlmError::NotImplemented => write!(f, "真实 LLM provider 尚未实现"),
            LlmError::RedactionFailed(msg) => write!(f, "输入脱敏失败: {}", msg),
            LlmError::InvalidInput(msg) => write!(f, "输入无效: {}", msg),
        }
    }
}

impl std::error::Error for LlmError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_kind_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&ProviderKind::OpenAi).unwrap(),
            "\"openai\""
        );
        assert_eq!(
            serde_json::to_string(&ProviderKind::Anthropic).unwrap(),
            "\"anthropic\""
        );
    }

    #[test]
    fn api_key_redacted_in_debug() {
        let key = ApiKey::new("fake-key-used-only-in-unit-tests");
        let debug = format!("{:?}", key);
        assert!(!debug.contains("fake-key-used-only-in-unit-tests"));
        assert!(debug.contains("REDACTED"));
    }

    #[test]
    fn api_key_masked_display() {
        let key = ApiKey::new("fake-key-used-only-in-unit-tests");
        assert_eq!(key.masked(), "fake...ests");
    }

    #[test]
    fn default_config_is_mock_and_disabled() {
        let cfg = ProviderConfig::default();
        assert_eq!(cfg.kind, ProviderKind::Mock);
        assert!(!cfg.enabled);
        assert_eq!(cfg.network_mode, NetworkMode::Disabled);
        assert_eq!(cfg.timeout_ms, 60_000);
    }

    #[test]
    fn unenabled_config_fails_validation() {
        let cfg = ProviderConfig::default();
        assert!(matches!(cfg.validate(), Err(LlmError::NotConfigured)));
    }

    #[test]
    fn openai_without_key_fails_validation() {
        let cfg = ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            enabled: true,
            ..ProviderConfig::default()
        };
        assert!(matches!(cfg.validate(), Err(LlmError::MissingApiKey(_))));
    }

    #[test]
    fn openai_with_key_passes_validation() {
        let cfg = ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            enabled: true,
            api_key: Some(ApiKey::new("this-is-a-fake-key-for-tests")),
            ..ProviderConfig::default()
        };
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn would_use_network_only_when_allowed() {
        let mut cfg = ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            enabled: true,
            api_key: Some(ApiKey::new("this-is-a-fake-key-for-tests")),
            ..ProviderConfig::default()
        };
        assert!(!cfg.would_use_network());

        cfg.network_mode = NetworkMode::Allow;
        assert!(cfg.would_use_network());
    }
}
