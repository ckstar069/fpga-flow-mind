use crate::llm::models::{ChatResponse, LlmError, Usage};
use crate::llm::transport::TransportResponse;
use serde::Deserialize;

/// OpenAI-compatible response 的内部解析结构。
#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
    #[serde(default)]
    usage: Option<OpenAiUsage>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Deserialize)]
struct OpenAiMessage {
    content: String,
}

#[derive(Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// 解析 provider 的 HTTP 响应，生成 `ChatResponse`。
///
/// 支持 OpenAI-compatible chat completion JSON 格式。
/// Anthropic 格式留作骨架（当前与 OpenAI 兼容处理）。
pub struct ResponseParser;

impl ResponseParser {
    /// 解析传输层响应。
    ///
    /// - HTTP 4xx → `LlmError::AuthError`
    /// - HTTP 5xx → `LlmError::ProviderCallFailed`
    /// - JSON 非法 / 缺少 choices / 内容为空 → `LlmError::InvalidResponse`
    pub fn parse(
        &self,
        response: &TransportResponse,
        provider_name: &str,
        model: &str,
    ) -> Result<ChatResponse, LlmError> {
        // HTTP 状态码检查
        if response.status_code >= 500 {
            return Err(LlmError::ProviderCallFailed(format!(
                "HTTP {}",
                response.status_code
            )));
        }
        if response.status_code >= 400 {
            return Err(LlmError::AuthError(format!(
                "HTTP {}",
                response.status_code
            )));
        }

        // JSON 解析
        let parsed: OpenAiResponse = serde_json::from_str(&response.body).map_err(|e| {
            LlmError::InvalidResponse(format!("JSON 解析失败: {}", e))
        })?;

        // 提取 content
        let choice = parsed.choices.first().ok_or_else(|| {
            LlmError::InvalidResponse("响应中无 choices".to_string())
        })?;
        let content = choice.message.content.clone();

        if content.is_empty() {
            return Err(LlmError::InvalidResponse("响应内容为空".to_string()));
        }

        // 提取 usage
        let usage = parsed.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        Ok(ChatResponse {
            content,
            provider: provider_name.to_string(),
            model: model.to_string(),
            is_degraded: false,
            degraded_reason: None,
            citations: vec![],
            usage,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transport_response(status: u16, body: &str) -> TransportResponse {
        TransportResponse {
            status_code: status,
            body: body.to_string(),
        }
    }

    fn valid_body() -> &'static str {
        r#"{
            "choices": [
                {
                    "message": {
                        "content": "这是 FPGA 阶段 L0 的算法实现。"
                    },
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "prompt_tokens": 100,
                "completion_tokens": 50,
                "total_tokens": 150
            }
        }"#
    }

    #[test]
    fn parser_returns_content_and_usage() {
        let parser = ResponseParser;
        let resp = transport_response(200, valid_body());
        let result = parser.parse(&resp, "openai", "gpt-4").unwrap();

        assert_eq!(result.content, "这是 FPGA 阶段 L0 的算法实现。");
        assert_eq!(result.provider, "openai");
        assert_eq!(result.model, "gpt-4");
        assert!(!result.is_degraded);
        assert!(result.citations.is_empty());

        let usage = result.usage.unwrap();
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn parser_maps_http_500_error() {
        let parser = ResponseParser;
        let resp = transport_response(500, r#"{"error":"Internal Server Error"}"#);
        let result = parser.parse(&resp, "openai", "gpt-4");
        assert!(matches!(result, Err(LlmError::ProviderCallFailed(_))));
    }

    #[test]
    fn parser_maps_http_401_error() {
        let parser = ResponseParser;
        let resp = transport_response(401, r#"{"error":"Unauthorized"}"#);
        let result = parser.parse(&resp, "openai", "gpt-4");
        assert!(matches!(result, Err(LlmError::AuthError(_))));
    }

    #[test]
    fn parser_maps_http_403_error() {
        let parser = ResponseParser;
        let resp = transport_response(403, r#"{"error":"Forbidden"}"#);
        let result = parser.parse(&resp, "openai", "gpt-4");
        assert!(matches!(result, Err(LlmError::AuthError(_))));
    }

    #[test]
    fn parser_maps_http_429_error() {
        let parser = ResponseParser;
        let resp = transport_response(429, r#"{"error":"Rate Limited"}"#);
        let result = parser.parse(&resp, "openai", "gpt-4");
        assert!(matches!(result, Err(LlmError::AuthError(_))));
    }

    #[test]
    fn parser_maps_invalid_json() {
        let parser = ResponseParser;
        let resp = transport_response(200, "这不是 JSON");
        let result = parser.parse(&resp, "openai", "gpt-4");
        assert!(matches!(result, Err(LlmError::InvalidResponse(_))));
    }

    #[test]
    fn parser_maps_missing_choices() {
        let parser = ResponseParser;
        let resp =
            transport_response(200, r#"{"choices":[],"usage":null}"#);
        let result = parser.parse(&resp, "openai", "gpt-4");
        assert!(matches!(result, Err(LlmError::InvalidResponse(_))));
    }

    #[test]
    fn parser_maps_empty_content() {
        let parser = ResponseParser;
        let resp = transport_response(
            200,
            r#"{"choices":[{"message":{"content":""}}],"usage":null}"#,
        );
        let result = parser.parse(&resp, "openai", "gpt-4");
        assert!(matches!(result, Err(LlmError::InvalidResponse(_))));
    }

    #[test]
    fn parser_handles_missing_usage() {
        let parser = ResponseParser;
        let resp = transport_response(
            200,
            r#"{"choices":[{"message":{"content":"测试内容"}}]}"#,
        );
        let result = parser.parse(&resp, "openai", "gpt-4").unwrap();
        assert_eq!(result.content, "测试内容");
        assert!(result.usage.is_none());
    }
}
