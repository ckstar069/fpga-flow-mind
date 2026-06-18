use crate::llm::models::{
    ChatRequest, ChatResponse, Citation, DegradedReason, LlmError, ProviderCapabilities,
    Usage,
};
use crate::llm::provider::LlmProvider;

/// 可注入固定响应的 Fake provider。
///
/// 仅用于测试与开发，不访问网络。可预设返回内容、citation、usage 与降级状态。
pub struct FakeProvider {
    model: String,
    response_template: String,
    citations: Vec<Citation>,
    usage: Option<Usage>,
    degraded: bool,
    degraded_reason: Option<DegradedReason>,
}

impl FakeProvider {
    pub fn new(response: impl Into<String>) -> Self {
        Self {
            model: "fake".to_string(),
            response_template: response.into(),
            citations: vec![],
            usage: None,
            degraded: false,
            degraded_reason: None,
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_citations(mut self, citations: Vec<Citation>) -> Self {
        self.citations = citations;
        self
    }

    pub fn with_usage(mut self, usage: Usage) -> Self {
        self.usage = Some(usage);
        self
    }

    pub fn with_degraded(mut self, reason: DegradedReason) -> Self {
        self.degraded = true;
        self.degraded_reason = Some(reason);
        self
    }
}

impl Default for FakeProvider {
    fn default() -> Self {
        Self::new("[fake] 固定响应")
    }
}

impl LlmProvider for FakeProvider {
    fn chat(&self, _request: &ChatRequest) -> Result<ChatResponse, LlmError> {
        Ok(ChatResponse {
            content: self.response_template.clone(),
            provider: self.provider_name().to_string(),
            model: self.model.clone(),
            is_degraded: self.degraded,
            degraded_reason: self.degraded_reason,
            citations: self.citations.clone(),
            usage: self.usage,
        })
    }

    fn provider_name(&self) -> &str {
        "fake"
    }

    fn model_id(&self) -> &str {
        &self.model
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            understanding: true,
            qa: true,
            structured_output: true,
            max_context_tokens: 8192,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_provider_returns_injected_response() {
        let provider = FakeProvider::new("injected answer");
        let request = ChatRequest {
            messages: vec![],
            system_prompt: None,
            temperature: None,
            max_tokens: None,
        };
        let response = provider.chat(&request).unwrap();
        assert_eq!(response.content, "injected answer");
        assert!(!response.is_degraded);
    }

    #[test]
    fn fake_provider_can_simulate_degraded() {
        let provider =
            FakeProvider::new("unknown").with_degraded(DegradedReason::NetworkDisabled);
        let request = ChatRequest {
            messages: vec![],
            system_prompt: None,
            temperature: None,
            max_tokens: None,
        };
        let response = provider.chat(&request).unwrap();
        assert!(response.is_degraded);
        assert_eq!(response.degraded_reason, Some(DegradedReason::NetworkDisabled));
    }

    #[test]
    fn fake_provider_preserves_citations_and_usage() {
        let citation = Citation {
            evidence_id: "EV-L0-000001".to_string(),
            source_path: Some("/tmp/test.py".to_string()),
            line_start: 10,
            line_end: 12,
            excerpt_summary: Some("test".to_string()),
        };
        let usage = Usage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        };
        let provider = FakeProvider::new("with evidence")
            .with_citations(vec![citation.clone()])
            .with_usage(usage);
        let request = ChatRequest {
            messages: vec![],
            system_prompt: None,
            temperature: None,
            max_tokens: None,
        };
        let response = provider.chat(&request).unwrap();
        assert_eq!(response.citations.len(), 1);
        assert_eq!(response.usage, Some(usage));
    }
}
