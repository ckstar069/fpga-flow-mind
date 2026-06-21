use crate::llm::models::{LlmError, ProviderKind};
use std::fmt;
use std::time::Duration;

/// 脱敏字符串包装。
///
/// 用于 header 值中可能包含 api_key 的字段。
/// `Debug` 和 `Display` 均输出 `[REDACTED]`，防止意外泄露。
/// 通过 `expose_secret()` 获取原始值（仅供 transport 层构造 HTTP 请求时使用）。
#[derive(Clone)]
pub struct RedactedString(String);

impl RedactedString {
    /// 暴露原始字符串（仅限 transport 层使用）。
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for RedactedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl fmt::Display for RedactedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl From<String> for RedactedString {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for RedactedString {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// 传输层请求。
///
/// 由 `RequestBuilder` 构造，传递给 `LlmTransport::send`。
/// `Debug` 实现会脱敏所有 header 和 body，防止 api_key 泄露。
pub struct TransportRequest {
    pub request_id: String,
    pub provider: ProviderKind,
    pub model: String,
    pub base_url: Option<String>,
    pub headers: Vec<(String, RedactedString)>,
    pub body: String,
    pub timeout_ms: u64,
}

impl fmt::Debug for TransportRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TransportRequest")
            .field("request_id", &self.request_id)
            .field("provider", &self.provider)
            .field("model", &self.model)
            .field("base_url", &self.base_url)
            .field(
                "headers",
                &self
                    .headers
                    .iter()
                    .map(|(k, _v)| format!("{}: [REDACTED]", k))
                    .collect::<Vec<_>>(),
            )
            .field("body", &"[REDACTED]")
            .field("timeout_ms", &self.timeout_ms)
            .finish()
    }
}

/// 传输层响应。
pub struct TransportResponse {
    pub status_code: u16,
    pub body: String,
}

impl fmt::Debug for TransportResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TransportResponse")
            .field("status_code", &self.status_code)
            .field("body_len", &self.body.len())
            .finish()
    }
}

/// 可注入的 LLM 传输层 trait。
///
/// 测试中可使用 `FakeTransport`，默认产品路径使用 `NoNetworkTransport`。
pub trait LlmTransport: Send + Sync {
    fn send(
        &self,
        request: &TransportRequest,
        timeout_ms: u64,
    ) -> Result<TransportResponse, LlmError>;
}

/// 真实 HTTP 传输层。
///
/// 仅在调用方已经完成显式 opt-in（enabled + network_mode=allow + api_key）后使用。
/// 默认 provider 工厂仍使用 `NoNetworkTransport`，避免普通分析路径隐式联网。
pub struct HttpTransport;

impl LlmTransport for HttpTransport {
    fn send(
        &self,
        request: &TransportRequest,
        timeout_ms: u64,
    ) -> Result<TransportResponse, LlmError> {
        let base_url = request
            .base_url
            .as_deref()
            .ok_or_else(|| LlmError::InvalidConfig("缺少 base_url".to_string()))?;
        let url = chat_completions_url(base_url);
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_millis(timeout_ms.max(1)))
            .build()
            .map_err(|_err| LlmError::NetworkError("HTTP client 初始化失败".to_string()))?;

        let mut builder = client.post(url).body(request.body.clone());
        for (name, value) in &request.headers {
            let header_name = reqwest::header::HeaderName::from_bytes(name.as_bytes())
                .map_err(|_err| LlmError::InvalidConfig("HTTP header 名称无效".to_string()))?;
            let header_value = reqwest::header::HeaderValue::from_str(value.expose_secret())
                .map_err(|_err| LlmError::InvalidConfig("HTTP header 值无效".to_string()))?;
            builder = builder.header(header_name, header_value);
        }

        let response = builder
            .send()
            .map_err(|_err| LlmError::NetworkError("HTTP 请求失败".to_string()))?;
        let status_code = response.status().as_u16();
        let body = response
            .text()
            .map_err(|_err| LlmError::NetworkError("HTTP 响应读取失败".to_string()))?;

        Ok(TransportResponse { status_code, body })
    }
}

fn chat_completions_url(base_url: &str) -> String {
    format!("{}/chat/completions", base_url.trim_end_matches('/'))
}

/// 禁止网络的传输层。
///
/// 所有 `send` 调用均返回 `LlmError::NetworkDisabled`。
/// 这是默认产品路径中 `RealLlmProvider` 使用的传输层。
pub struct NoNetworkTransport;

impl LlmTransport for NoNetworkTransport {
    fn send(
        &self,
        _request: &TransportRequest,
        _timeout_ms: u64,
    ) -> Result<TransportResponse, LlmError> {
        Err(LlmError::NetworkDisabled)
    }
}

/// 可注入固定响应的 Fake 传输层。
///
/// 仅用于测试。可预设成功响应或错误，不发起真实网络请求。
pub struct FakeTransport {
    response: Result<TransportResponse, LlmError>,
}

impl FakeTransport {
    /// 创建一个总是返回给定响应的传输层。
    pub fn new(response: TransportResponse) -> Self {
        Self {
            response: Ok(response),
        }
    }

    /// 创建一个总是返回给定错误的传输层。
    pub fn new_error(err: LlmError) -> Self {
        Self { response: Err(err) }
    }
}

impl LlmTransport for FakeTransport {
    fn send(
        &self,
        _request: &TransportRequest,
        _timeout_ms: u64,
    ) -> Result<TransportResponse, LlmError> {
        match &self.response {
            Ok(resp) => Ok(TransportResponse {
                status_code: resp.status_code,
                body: resp.body.clone(),
            }),
            Err(e) => Err(e.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacted_string_debug_is_safe() {
        let s = RedactedString::from("super-secret-value");
        let debug = format!("{:?}", s);
        assert!(!debug.contains("super-secret-value"));
        assert!(debug.contains("REDACTED"));
    }

    #[test]
    fn redacted_string_display_is_safe() {
        let s = RedactedString::from("super-secret-value");
        let display = format!("{}", s);
        assert!(!display.contains("super-secret-value"));
        assert!(display.contains("REDACTED"));
    }

    #[test]
    fn redacted_string_expose_secret_returns_original() {
        let s = RedactedString::from("super-secret-value");
        assert_eq!(s.expose_secret(), "super-secret-value");
    }

    #[test]
    fn redacted_string_from_string_and_str() {
        let s1 = RedactedString::from("hello".to_string());
        let s2 = RedactedString::from("world");
        assert_eq!(s1.expose_secret(), "hello");
        assert_eq!(s2.expose_secret(), "world");
    }

    #[test]
    fn transport_request_debug_redacts_headers_and_body() {
        let req = TransportRequest {
            request_id: "req-1".to_string(),
            provider: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            base_url: Some("https://api.openai.com/v1".to_string()),
            headers: vec![
                (
                    "authorization".to_string(),
                    RedactedString::from("Bearer this-is-a-fake-key-for-transport-tests"),
                ),
                (
                    "content-type".to_string(),
                    RedactedString::from("application/json"),
                ),
            ],
            body: r#"{"model":"gpt-4","messages":[]}"#.to_string(),
            timeout_ms: 30000,
        };
        let debug = format!("{:?}", req);
        // header 已脱敏
        assert!(!debug.contains("this-is-a-fake-key-for-transport-tests"));
        assert!(!debug.contains("Bearer"));
        // body 已脱敏
        assert!(!debug.contains(r#""model""#));
        assert!(debug.contains("[REDACTED]"));
        // 非敏感字段仍然可见
        assert!(debug.contains("req-1"));
        assert!(debug.contains("openai"));
    }

    #[test]
    fn transport_response_debug_shows_status_not_body() {
        let resp = TransportResponse {
            status_code: 200,
            body: "secret body content".to_string(),
        };
        let debug = format!("{:?}", resp);
        assert!(debug.contains("200"));
        assert!(!debug.contains("secret body content"));
        assert!(debug.contains("body_len"));
    }

    #[test]
    fn no_network_transport_always_blocked() {
        let transport = NoNetworkTransport;
        let req = TransportRequest {
            request_id: "test".to_string(),
            provider: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            base_url: None,
            headers: vec![],
            body: "{}".to_string(),
            timeout_ms: 1000,
        };
        let result = transport.send(&req, 1000);
        assert!(matches!(result, Err(LlmError::NetworkDisabled)));
    }

    #[test]
    fn fake_transport_returns_preset_response() {
        let expected = TransportResponse {
            status_code: 200,
            body: r#"{"choices":[{"message":{"content":"hello"}}]}"#.to_string(),
        };
        let transport = FakeTransport::new(TransportResponse {
            status_code: 200,
            body: r#"{"choices":[{"message":{"content":"hello"}}]}"#.to_string(),
        });
        let req = TransportRequest {
            request_id: "test".to_string(),
            provider: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            base_url: None,
            headers: vec![],
            body: "{}".to_string(),
            timeout_ms: 1000,
        };
        let result = transport.send(&req, 1000).unwrap();
        assert_eq!(result.status_code, 200);
        assert_eq!(result.body, expected.body);
    }

    #[test]
    fn fake_transport_returns_preset_error() {
        let transport =
            FakeTransport::new_error(LlmError::NetworkError("模拟网络故障".to_string()));
        let req = TransportRequest {
            request_id: "test".to_string(),
            provider: ProviderKind::OpenAi,
            model: "gpt-4".to_string(),
            base_url: None,
            headers: vec![],
            body: "{}".to_string(),
            timeout_ms: 1000,
        };
        let result = transport.send(&req, 1000);
        assert!(matches!(result, Err(LlmError::NetworkError(_))));
    }

    #[test]
    fn chat_completions_url_appends_path_and_trims_slash() {
        assert_eq!(
            chat_completions_url("https://api.deepseek.com"),
            "https://api.deepseek.com/chat/completions"
        );
        assert_eq!(
            chat_completions_url("https://api.deepseek.com/"),
            "https://api.deepseek.com/chat/completions"
        );
    }
}
