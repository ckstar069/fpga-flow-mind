use crate::llm::models::{ChatRequest, LlmError, ProviderConfig, ProviderKind};
use crate::llm::transport::{RedactedString, TransportRequest};
use std::time::{SystemTime, UNIX_EPOCH};

/// 构建 provider-neutral 的 `TransportRequest`。
///
/// 负责：
/// 1. 校验配置合法性（已启用、有 api_key）
/// 2. 构造 OpenAI-compatible JSON body
/// 3. 设置 Authorization header（值使用 RedactedString 防泄露）
/// 4. 拒绝 Mock/Fake provider（RequestBuilder 仅用于真实 provider）
pub struct RequestBuilder {
    config: ProviderConfig,
    request_id: String,
}

impl RequestBuilder {
    pub fn new(config: &ProviderConfig) -> Self {
        let id = format!(
            "req-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        Self {
            config: config.clone(),
            request_id: id,
        }
    }

    pub fn with_request_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = id.into();
        self
    }

    /// 根据 `ChatRequest` 构建 `TransportRequest`。
    ///
    /// 返回错误的情况：
    /// - 配置为 Mock/Fake（RequestBuilder 只用于真实 provider）
    /// - 配置未启用或缺少 api_key
    /// - body JSON 序列化失败
    pub fn build(&self, chat_request: &ChatRequest) -> Result<TransportRequest, LlmError> {
        // 拒绝 Mock/Fake
        if matches!(self.config.kind, ProviderKind::Mock | ProviderKind::Fake) {
            return Err(LlmError::InvalidConfig(
                "RequestBuilder 只用于真实 provider（OpenAi/Anthropic）".to_string(),
            ));
        }

        // 校验配置（enabled、model 非空、api_key 存在）
        self.config.validate()?;

        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or(LlmError::MissingApiKey(self.config.kind))?;

        // 构建 JSON body（OpenAI-compatible）
        let mut body_map = serde_json::Map::new();
        body_map.insert(
            "model".to_string(),
            serde_json::Value::String(self.config.model.clone()),
        );

        let messages: Vec<serde_json::Value> = chat_request
            .messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content,
                })
            })
            .collect();
        body_map.insert("messages".to_string(), serde_json::Value::Array(messages));

        if let Some(temp) = chat_request.temperature {
            body_map.insert(
                "temperature".to_string(),
                serde_json::json!(temp),
            );
        }
        if let Some(mt) = chat_request.max_tokens {
            body_map.insert("max_tokens".to_string(), serde_json::json!(mt));
        }

        let body = serde_json::to_string(&body_map)
            .map_err(|e| LlmError::InvalidConfig(format!("JSON 序列化失败: {}", e)))?;

        // 默认 base_url
        let base_url = self.config.base_url.clone().unwrap_or_else(|| match self.config.kind {
            ProviderKind::OpenAi => "https://api.openai.com/v1".to_string(),
            ProviderKind::Anthropic => "https://api.anthropic.com/v1".to_string(),
            _ => unreachable!(),
        });

        // 构造 header（Authorization 值使用 RedactedString 防泄露）
        let auth_value = format!("Bearer {}", api_key.expose_secret());
        let headers = vec![
            (
                "authorization".to_string(),
                RedactedString::from(auth_value),
            ),
            (
                "content-type".to_string(),
                RedactedString::from("application/json"),
            ),
        ];

        Ok(TransportRequest {
            request_id: self.request_id.clone(),
            provider: self.config.kind,
            model: self.config.model.clone(),
            base_url: Some(base_url),
            headers,
            body,
            timeout_ms: self.config.timeout_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::models::{ApiKey, ChatMessage, ChatRole, NetworkMode};

    fn openai_config_with_key() -> ProviderConfig {
        ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            enabled: true,
            api_key: Some(ApiKey::new("this-is-a-fake-key-for-tests")),
            base_url: None,
            timeout_ms: 30_000,
            retry_limit: 2,
            rate_limit_per_min: 60,
            network_mode: NetworkMode::Allow,
        }
    }

    fn simple_chat_request() -> ChatRequest {
        ChatRequest {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: "你好".to_string(),
            }],
            system_prompt: None,
            temperature: None,
            max_tokens: None,
        }
    }

    #[test]
    fn request_builder_rejects_mock_config() {
        let cfg = ProviderConfig::mock();
        let builder = RequestBuilder::new(&cfg);
        let result = builder.build(&simple_chat_request());
        assert!(matches!(result, Err(LlmError::InvalidConfig(_))));
    }

    #[test]
    fn request_builder_rejects_fake_config() {
        let cfg = ProviderConfig::fake("fake-model");
        let builder = RequestBuilder::new(&cfg);
        let result = builder.build(&simple_chat_request());
        assert!(matches!(result, Err(LlmError::InvalidConfig(_))));
    }

    #[test]
    fn request_builder_requires_api_key() {
        let cfg = ProviderConfig {
            kind: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            enabled: true,
            api_key: None,
            ..ProviderConfig::default()
        };
        let builder = RequestBuilder::new(&cfg);
        let result = builder.build(&simple_chat_request());
        assert!(matches!(result, Err(LlmError::MissingApiKey(_))));
    }

    #[test]
    fn request_builder_produces_openai_request() {
        let cfg = openai_config_with_key();
        let builder = RequestBuilder::new(&cfg).with_request_id("test-req-001");
        let request = ChatRequest {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: "解释一下这段代码".to_string(),
            }],
            system_prompt: Some("你是一个 FPGA 专家".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(1024),
        };
        let transport_req = builder.build(&request).unwrap();

        assert_eq!(transport_req.request_id, "test-req-001");
        assert_eq!(transport_req.provider, ProviderKind::OpenAi);
        assert_eq!(transport_req.model, "gpt-4");
        assert_eq!(
            transport_req.base_url,
            Some("https://api.openai.com/v1".to_string())
        );
        assert_eq!(transport_req.timeout_ms, 30_000);

        // 验证 header
        assert_eq!(transport_req.headers.len(), 2);
        assert_eq!(transport_req.headers[0].0, "authorization");
        assert_eq!(transport_req.headers[1].0, "content-type");

        // 验证 body 结构
        let body: serde_json::Value =
            serde_json::from_str(&transport_req.body).expect("body 应为合法 JSON");
        assert_eq!(body["model"], "gpt-4");
        assert_eq!(body["messages"][0]["role"], "user");
        assert_eq!(body["messages"][0]["content"], "解释一下这段代码");
        assert!((body["temperature"].as_f64().unwrap() - 0.7).abs() < 0.001);
        assert_eq!(body["max_tokens"], 1024);

        // 验证 system_prompt 在当前设计中不单独作为 message 发送
        // （Batch B 默认不处理 system_prompt，留待后续扩展）
    }

    #[test]
    fn request_builder_redacts_authorization_in_debug() {
        let cfg = openai_config_with_key();
        let builder = RequestBuilder::new(&cfg);
        let transport_req = builder.build(&simple_chat_request()).unwrap();
        let debug = format!("{:?}", transport_req);

        // api_key 不应出现在 Debug 输出中
        assert!(!debug.contains("this-is-a-fake-key-for-tests"));
        // Authorization header value 不应出现
        assert!(!debug.contains("Bearer"));
        // header 名称可能出现，但值已脱敏
        assert!(debug.contains("[REDACTED]"));
    }

    #[test]
    fn request_body_does_not_contain_key() {
        let cfg = openai_config_with_key();
        let builder = RequestBuilder::new(&cfg);
        let transport_req = builder.build(&simple_chat_request()).unwrap();

        // body 中不应出现 api_key
        assert!(!transport_req.body.contains("this-is-a-fake-key-for-tests"));
        // body 中不应出现 "api_key" 或 "authorization"
        assert!(!transport_req.body.to_lowercase().contains("api_key"));
    }

    #[test]
    fn request_builder_uses_custom_base_url() {
        let mut cfg = openai_config_with_key();
        cfg.base_url = Some("https://custom-proxy.example.com/v1".to_string());
        let builder = RequestBuilder::new(&cfg);
        let transport_req = builder.build(&simple_chat_request()).unwrap();
        assert_eq!(
            transport_req.base_url,
            Some("https://custom-proxy.example.com/v1".to_string())
        );
    }
}
