use crate::llm::models::{ChatRequest, ChatResponse, LlmError, ProviderCapabilities, ProviderConfig};
use crate::llm::provider::LlmProvider;
use crate::llm::request_builder::RequestBuilder;
use crate::llm::response_parser::ResponseParser;
use crate::llm::transport::LlmTransport;

/// 真实 LLM Provider，通过可注入的 `LlmTransport` 发起调用。
///
/// # 安全设计
///
/// - Batch B 默认产品路径使用 `NoNetworkTransport`，所有调用返回 `NetworkDisabled`。
/// - 真实 HTTP transport 仅在显式注入时可用（测试 / 手工 smoke）。
/// - 实现有界重试：仅对 `ProviderCallFailed`（5xx）和 `NetworkError` 重试，
///   `AuthError`（4xx）和 `InvalidResponse` 不重试。
/// - 重试使用固定 10ms 退避，次数受 `config.retry_limit` 控制。
pub struct RealLlmProvider<T: LlmTransport> {
    config: ProviderConfig,
    transport: T,
}

impl<T: LlmTransport> RealLlmProvider<T> {
    pub fn new(config: ProviderConfig, transport: T) -> Self {
        Self { config, transport }
    }
}

impl<T: LlmTransport> LlmProvider for RealLlmProvider<T> {
    fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, LlmError> {
        let mut last_error: Option<LlmError> = None;

        for attempt in 0..=self.config.retry_limit {
            if attempt > 0 {
                // 固定 10ms 退避（测试友好，不引入真实 sleep 依赖）
                std::thread::sleep(std::time::Duration::from_millis(10));
            }

            // 1. 构建 TransportRequest
            let transport_request = match RequestBuilder::new(&self.config).build(request) {
                Ok(req) => req,
                Err(e) => return Err(e), // 配置错误不重试
            };

            // 2. 通过 transport 发送
            let transport_response = match self
                .transport
                .send(&transport_request, self.config.timeout_ms)
            {
                Ok(resp) => resp,
                Err(e) => {
                    // 仅对可重试错误重试
                    match &e {
                        LlmError::ProviderCallFailed(_) | LlmError::NetworkError(_) => {
                            last_error = Some(e);
                            continue;
                        }
                        _ => return Err(e),
                    }
                }
            };

            // 3. 解析响应
            match ResponseParser.parse(
                &transport_response,
                self.provider_name(),
                self.model_id(),
            ) {
                Ok(chat_response) => return Ok(chat_response),
                Err(e) => {
                    // InvalidResponse 不重试；AuthError 不重试
                    match &e {
                        LlmError::InvalidResponse(_) | LlmError::AuthError(_) => {
                            return Err(e);
                        }
                        _ => {
                            last_error = Some(e);
                            continue;
                        }
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| LlmError::ProviderCallFailed("所有重试均已耗尽".to_string())))
    }

    fn provider_name(&self) -> &str {
        match self.config.kind {
            crate::llm::models::ProviderKind::OpenAi => "openai",
            crate::llm::models::ProviderKind::Anthropic => "anthropic",
            _ => "real",
        }
    }

    fn model_id(&self) -> &str {
        &self.config.model
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            understanding: true,
            qa: true,
            structured_output: true,
            max_context_tokens: 128_000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::models::{
        ApiKey, ChatMessage, ChatRole, NetworkMode, ProviderKind,
    };
    use crate::llm::transport::{FakeTransport, NoNetworkTransport, TransportRequest, TransportResponse};

    fn openai_config() -> ProviderConfig {
        ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            enabled: true,
            api_key: Some(ApiKey::new("this-is-a-fake-key-for-tests")),
            base_url: None,
            timeout_ms: 30_000,
            retry_limit: 1,
            rate_limit_per_min: 60,
            network_mode: NetworkMode::Allow,
        }
    }

    fn make_request() -> ChatRequest {
        ChatRequest {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: "什么是 FPGA？".to_string(),
            }],
            system_prompt: None,
            temperature: None,
            max_tokens: None,
        }
    }

    fn success_response_body() -> String {
        r#"{
            "choices": [
                {
                    "message": {
                        "content": "FPGA 是现场可编程门阵列。"
                    }
                }
            ],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        }"#
        .to_string()
    }

    #[test]
    fn real_provider_with_fake_transport_returns_response() {
        let config = openai_config();
        let transport = FakeTransport::new(TransportResponse {
            status_code: 200,
            body: success_response_body(),
        });
        let provider = RealLlmProvider::new(config, transport);

        let response = provider.chat(&make_request()).unwrap();
        assert_eq!(response.content, "FPGA 是现场可编程门阵列。");
        assert_eq!(response.provider, "openai");
        assert_eq!(response.model, "gpt-4");
        assert!(!response.is_degraded);
        assert_eq!(response.usage.unwrap().total_tokens, 15);
    }

    #[test]
    fn real_provider_retries_then_succeeds() {
        let config = openai_config(); // retry_limit = 1，即最多 2 次尝试

        // 第一次失败（网络错误），第二次成功
        struct RetryTransport {
            call_count: std::sync::atomic::AtomicU32,
            success_body: String,
        }
        impl LlmTransport for RetryTransport {
            fn send(
                &self,
                _request: &TransportRequest,
                _timeout_ms: u64,
            ) -> Result<TransportResponse, LlmError> {
                let count = self
                    .call_count
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if count == 0 {
                    Err(LlmError::NetworkError("第一次调用失败".to_string()))
                } else {
                    Ok(TransportResponse {
                        status_code: 200,
                        body: self.success_body.clone(),
                    })
                }
            }
        }

        let transport = RetryTransport {
            call_count: std::sync::atomic::AtomicU32::new(0),
            success_body: success_response_body(),
        };
        let provider = RealLlmProvider::new(config, transport);
        let response = provider.chat(&make_request()).unwrap();

        assert_eq!(response.content, "FPGA 是现场可编程门阵列。");
    }

    #[test]
    fn real_provider_does_not_retry_auth_error() {
        let config = openai_config();
        let transport = FakeTransport::new(TransportResponse {
            status_code: 401,
            body: r#"{"error":"Unauthorized"}"#.to_string(),
        });
        let provider = RealLlmProvider::new(config, transport);

        let result = provider.chat(&make_request());
        assert!(matches!(result, Err(LlmError::AuthError(_))));
    }

    #[test]
    fn real_provider_does_not_retry_invalid_response() {
        let config = openai_config();
        let transport = FakeTransport::new(TransportResponse {
            status_code: 200,
            body: "not json at all".to_string(),
        });
        let provider = RealLlmProvider::new(config, transport);

        let result = provider.chat(&make_request());
        assert!(matches!(result, Err(LlmError::InvalidResponse(_))));
    }

    #[test]
    fn real_provider_exhausts_retries_and_fails() {
        let mut config = openai_config();
        config.retry_limit = 2; // 最多 3 次尝试

        let transport =
            FakeTransport::new_error(LlmError::NetworkError("持续网络故障".to_string()));
        let provider = RealLlmProvider::new(config, transport);

        let result = provider.chat(&make_request());
        assert!(matches!(result, Err(LlmError::NetworkError(_))));
    }

    #[test]
    fn real_provider_no_network_transport_is_blocked() {
        let config = openai_config();
        let transport = NoNetworkTransport;
        let provider = RealLlmProvider::new(config, transport);

        let result = provider.chat(&make_request());
        assert!(matches!(result, Err(LlmError::NetworkDisabled)));
    }

    #[test]
    fn real_provider_capabilities_declare_full() {
        let config = openai_config();
        let transport = FakeTransport::new(TransportResponse {
            status_code: 200,
            body: success_response_body(),
        });
        let provider = RealLlmProvider::new(config, transport);
        let caps = provider.capabilities();
        assert!(caps.understanding);
        assert!(caps.qa);
        assert!(caps.structured_output);
        assert!(caps.max_context_tokens > 0);
    }
}
