use crate::llm::models::{
    ChatRequest, ChatResponse, DegradedReason, LlmError, ProviderCapabilities, ProviderStatus,
    Usage,
};
use crate::llm::provider::LlmProvider;

/// 本地 Mock provider。
///
/// 不访问网络，返回固定 heuristic 响应，用于默认无配置场景。
pub struct MockProvider {
    model: String,
}

impl MockProvider {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
        }
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new("mock")
    }
}

impl LlmProvider for MockProvider {
    fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmError> {
        let last_user_message = request
            .messages
            .iter()
            .rev()
            .find(|m| matches!(m.role, crate::llm::models::ChatRole::User))
            .map(|m| m.content.as_str())
            .unwrap_or("");

        let content = if last_user_message.is_empty() {
            "当前没有可回答的用户问题。".to_string()
        } else {
            format!(
                "[mock] 已收到问题（{} 字），但 Batch A 尚未接入真实 LLM。",
                last_user_message.chars().count()
            )
        };

        Ok(ChatResponse {
            content,
            provider: self.provider_name().to_string(),
            model: self.model.clone(),
            is_degraded: true,
            degraded_reason: Some(DegradedReason::NotConfigured),
            citations: vec![],
            usage: Some(Usage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            }),
        })
    }

    fn provider_name(&self) -> &str {
        "mock"
    }

    fn model_id(&self) -> &str {
        &self.model
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            understanding: false,
            qa: false,
            structured_output: false,
            max_context_tokens: 0,
        }
    }
}

/// 当前运行状态快照。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MockProviderStatus {
    pub status: ProviderStatus,
}

impl MockProvider {
    pub fn status(&self) -> MockProviderStatus {
        MockProviderStatus {
            status: ProviderStatus::Mock,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::models::{ChatMessage, ChatRole};

    #[test]
    fn mock_provider_returns_degraded_response() {
        let provider = MockProvider::default();
        let request = ChatRequest {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: "hello".to_string(),
            }],
            system_prompt: None,
            temperature: None,
            max_tokens: None,
        };
        let response = provider.chat(&request).unwrap();
        assert!(response.is_degraded);
        assert_eq!(response.degraded_reason, Some(DegradedReason::NotConfigured));
        assert!(response.content.contains("[mock]"));
    }

    #[test]
    fn mock_provider_handles_empty_input() {
        let provider = MockProvider::default();
        let request = ChatRequest {
            messages: vec![],
            system_prompt: None,
            temperature: None,
            max_tokens: None,
        };
        let response = provider.chat(&request).unwrap();
        assert!(response.content.contains("没有可回答"));
    }

    #[test]
    fn mock_capabilities_declares_no_real_capabilities() {
        let provider = MockProvider::default();
        let caps = provider.capabilities();
        assert!(!caps.understanding);
        assert!(!caps.qa);
        assert!(!caps.structured_output);
    }
}
