use crate::llm::models::{ChatRequest, ChatResponse, LlmError, ProviderCapabilities};

/// 通用 LLM Provider trait。
///
/// 设计为同步接口，与现有 `UnderstandingProvider` / `GroundedQaProvider` 保持一致，
/// Batch A 不引入 async runtime 依赖。
pub trait LlmProvider: Send + Sync {
    /// 发起一次非流式聊天/补全调用。
    fn chat(&self,
        request: &ChatRequest,
    ) -> Result<ChatResponse, LlmError>;

    /// Provider 显示名称。
    fn provider_name(&self) -> &str;

    /// 当前模型 id。
    fn model_id(&self) -> &str;

    /// 能力声明。
    fn capabilities(&self) -> ProviderCapabilities;
}

/// 将 `Box<dyn LlmProvider>` 作为通用 provider 句柄。
pub type BoxedLlmProvider = Box<dyn LlmProvider>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::models::{ChatMessage, ChatRole};

    struct DummyProvider;

    impl LlmProvider for DummyProvider {
        fn chat(&self, _request: &ChatRequest) -> Result<ChatResponse, LlmError> {
            Ok(ChatResponse {
                content: "dummy".to_string(),
                provider: "dummy".to_string(),
                model: "dummy-model".to_string(),
                is_degraded: false,
                degraded_reason: None,
                citations: vec![],
                usage: None,
            })
        }

        fn provider_name(&self) -> &str {
            "dummy"
        }

        fn model_id(&self) -> &str {
            "dummy-model"
        }

        fn capabilities(&self) -> ProviderCapabilities {
            ProviderCapabilities::default()
        }
    }

    #[test]
    fn boxed_provider_can_be_used() {
        let provider: BoxedLlmProvider = Box::new(DummyProvider);
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
        assert_eq!(response.content, "dummy");
        assert_eq!(provider.provider_name(), "dummy");
    }
}
