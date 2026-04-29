use std::path::PathBuf;
use std::sync::Arc;

use narratoai_core::error::LLMError;
use narratoai_core::llm::image_utils::image_to_base64_data_url;
use narratoai_core::llm::openai_compatible::{OpenAiCompatibleProvider, ProviderConfig};
use narratoai_core::llm::provider::LlmProvider;
use narratoai_core::llm::registry::Registry;
use narratoai_core::llm::types::LlmResponseFormat;
use narratoai_core::llm::test_utils::{create_test_jpeg_path, write_test_jpeg};
use tempfile::TempDir;
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path, body_string_contains};

use base64::Engine;

// ---------------------------------------------------------------------------
// 测试 1: Registry register / get / list_providers / ProviderNotFound
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_registry_register_and_get() {
    // Arrange
    let mut registry = Registry::new();
    let provider = create_test_provider("http://localhost:0/v1");

    // Act: register
    registry.register("test_provider", Arc::new(provider));

    // Assert: get existing provider
    let got = registry.get("test_provider");
    assert!(got.is_ok(), "应能获取已注册的 provider");

    // Assert: get non-existent provider
    let missing = registry.get("nonexistent");
    assert!(missing.is_err(), "不存在的 provider 应返回错误");
    match missing {
        Err(LLMError::ProviderNotFound(ref name)) => {
            assert_eq!(name, "nonexistent");
        }
        Err(e) => panic!("应为 ProviderNotFound, 得到: {}", e),
        Ok(_) => panic!("应失败"),
    }

    // Assert: list_providers
    let providers = registry.list_providers();
    assert!(providers.contains(&"test_provider".to_string()));
    assert_eq!(providers.len(), 1, "仅注册了一个 provider");
}

// ---------------------------------------------------------------------------
// 测试 2: image_to_base64_data_url
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_image_to_base64_data_url() {
    let dir = TempDir::new().expect("创建临时目录失败");
    let img_path = create_test_jpeg_path(dir.path());

    let result = image_to_base64_data_url(&img_path);
    assert!(result.is_ok(), "图片预处理应成功: {:?}", result.err());

    let data_url = result.unwrap();
    assert!(
        data_url.starts_with("data:image/jpeg;base64,"),
        "data URL 应以正确前缀开头: {}",
        &data_url[..30]
    );

    // 验证 base64 部分可解码
    let b64_part = data_url.strip_prefix("data:image/jpeg;base64,").unwrap();
    let decoded = base64::engine::general_purpose::STANDARD.decode(b64_part);
    assert!(decoded.is_ok(), "base64 部分应可解码: {:?}", decoded.err());
    assert!(!decoded.unwrap().is_empty(), "解码后的数据不应为空");
}

// ---------------------------------------------------------------------------
// 测试 3: Vision 请求消息构建（wiremock）
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_build_vision_message_structure() {
    let mock_server = MockServer::start().await;

    // Mock: expect POST /v1/chat/completions, return a simple response
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({
                "id": "test-vision-id",
                "object": "chat.completion",
                "created": 1234567890,
                "model": "test-model",
                "choices": [{
                    "index": 0,
                    "message": {
                        "content": "vision analysis result",
                        "role": "assistant"
                    },
                    "finish_reason": "stop"
                }]
            }),
        ))
        .expect(1) // 期望正好 1 次调用
        .mount(&mock_server)
        .await;

    let provider = create_test_provider(&mock_server.uri());
    let dir = TempDir::new().expect("创建临时目录失败");
    let img_path = create_test_jpeg_path(dir.path());

    let results = provider
        .analyze_images(&[img_path], "describe this image", None, Some(10), Some(1), None, None, None)
        .await;

    assert!(results.is_ok(), "analyze_images 应成功: {:?}", results.err());
    let results = results.unwrap();
    assert_eq!(results.len(), 1, "应为 1 个结果");
    assert_eq!(results[0], "vision analysis result");
}

// ---------------------------------------------------------------------------
// 测试 4: generate_text 成功路径（wiremock）
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_generate_text_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({
                "id": "test-text-id",
                "object": "chat.completion",
                "created": 1234567890,
                "model": "test-model",
                "choices": [{
                    "index": 0,
                    "message": {
                        "content": "Hello! How can I help you?",
                        "role": "assistant"
                    },
                    "finish_reason": "stop"
                }]
            }),
        ))
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = create_test_provider(&mock_server.uri());

    let result = provider.generate_text("Hello", None, None, None, None).await;
    assert!(result.is_ok(), "generate_text 应成功: {:?}", result.err());
    assert_eq!(result.unwrap(), "Hello! How can I help you?");
}

// ---------------------------------------------------------------------------
// 测试 5: 流式 token 提取（wiremock - SSE）
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_generate_text_stream_token_extraction() {
    let mock_server = MockServer::start().await;

    // SSE format: data: {"choices":[{"delta":{"content":"..."}}]}
    let sse_body = "data: {\"id\":\"test-stream-id\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"test-model\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello\"},\"finish_reason\":null}]}\n\ndata: {\"id\":\"test-stream-id\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"test-model\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\" World\"},\"finish_reason\":null}]}\n\ndata: {\"id\":\"test-stream-id\",\"object\":\"chat.completion.chunk\",\"created\":1234567890,\"model\":\"test-model\",\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n\n";

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(sse_body)
                .insert_header("Content-Type", "text/event-stream"),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = create_test_provider(&mock_server.uri());

    let stream = provider
        .generate_text_stream("test", None, None, None)
        .await;
    assert!(stream.is_ok(), "generate_text_stream 应成功: {:?}", stream.err());

    let mut stream = stream.unwrap();
    use futures::StreamExt;
    let mut tokens = Vec::new();
    while let Some(item) = stream.next().await {
        match item {
            Ok(token) => tokens.push(token),
            Err(e) => panic!("流式 token 提取失败: {}", e),
        }
    }

    assert_eq!(tokens.join(""), "Hello World", "流式 token 应拼接为完整文本");
}

// ---------------------------------------------------------------------------
// 测试 6: JSON response_format 回退（wiremock）
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_json_response_format_fallback() {
    let mock_server = MockServer::start().await;

    // 首次请求（包含 response_format 字段）：返回 400，触发回退
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(body_string_contains("response_format"))
        .respond_with(ResponseTemplate::new(400).set_body_json(
            serde_json::json!({
                "error": {
                    "message": "response_format is not supported",
                    "type": "invalid_request_error"
                }
            }),
        ))
        .expect(1) // 首次请求恰好 1 次
        .mount(&mock_server)
        .await;

    // 回退重试请求（不含 response_format，prompt 含 JSON 指令）：返回成功
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({
                "id": "retry-success-id",
                "object": "chat.completion",
                "created": 1234567890,
                "model": "test-model",
                "choices": [{
                    "index": 0,
                    "message": {
                        "content": "{\"result\": \"success\"}",
                        "role": "assistant"
                    },
                    "finish_reason": "stop"
                }]
            }),
        ))
        .expect(1) // 回退重试恰好 1 次
        .mount(&mock_server)
        .await;

    let provider = create_test_provider(&mock_server.uri());

    // 传入 response_format=Json 触发 JSON 回退路径
    let result = provider
        .generate_text("test", None, None, None, Some(LlmResponseFormat::Json))
        .await;
    assert!(result.is_ok(), "JSON format 回退应成功: {:?}", result.err());
    assert_eq!(result.unwrap(), "{\"result\": \"success\"}");
}

/// 测试 7: OpenAI 错误映射（wiremock）
///
/// 验证 From<OpenAIError> 的 code 匹配和消息启发式判断逻辑。
/// 使用 code=invalid_api_key（401 → Permanent）、code=insufficient_quota（429 → Permanent）
/// 和通用 400 错误分别验证 Authentication、RateLimit 和 APICall 映射。
/// 测试 provider 使用 max_retries=0 禁用 async-openai 内置重试，避免 HTTP 错误响应挂起。
#[tokio::test]
async fn test_openai_error_mapping() {
    struct TestCase {
        status: u16,
        error_code: Option<&'static str>,
        error_type: &'static str,
        message: &'static str,
        expected: fn(&LLMError) -> bool,
    }

    fn is_authentication(e: &LLMError) -> bool {
        matches!(e, LLMError::Authentication(_))
    }
    fn is_rate_limit(e: &LLMError) -> bool {
        matches!(e, LLMError::RateLimit(_))
    }
    fn is_api_call(e: &LLMError) -> bool {
        matches!(e, LLMError::APICall(_))
    }

    let test_cases = vec![
        TestCase {
            status: 401,
            error_code: Some("invalid_api_key"),
            error_type: "authentication_error",
            message: "Invalid API key",
            expected: is_authentication,
        },
        TestCase {
            status: 429,
            error_code: Some("insufficient_quota"),
            error_type: "insufficient_quota",
            message: "Quota exceeded",
            expected: is_rate_limit,
        },
        TestCase {
            status: 400,
            error_code: None,
            error_type: "invalid_request_error",
            message: "Bad request",
            expected: is_api_call,
        },
    ];

    for tc in test_cases {
        let mock_server = MockServer::start().await;

        let mut error_body = serde_json::json!({
            "message": tc.message,
            "type": tc.error_type,
        });
        if let Some(code) = tc.error_code {
            error_body["code"] = serde_json::json!(code);
        }

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(
                ResponseTemplate::new(tc.status).set_body_json(serde_json::json!({
                    "error": error_body,
                })),
            )
            .mount(&mock_server)
            .await;

        let provider = create_test_provider(&mock_server.uri());
        let result = provider
            .generate_text("test", None, None, None, None)
            .await;

        assert!(
            result.is_err(),
            "HTTP {} 应返回错误",
            tc.status
        );
        let err = result.as_ref().unwrap_err();
        assert!(
            (tc.expected)(err),
            "HTTP {} 错误变体不匹配: {:?}",
            tc.status,
            result,
        );
    }
}

// ---------------------------------------------------------------------------
// 测试 8: analyze_images 结果顺序（wiremock）
// ---------------------------------------------------------------------------
use std::sync::atomic::{AtomicUsize, Ordering};

struct CyclicResponder {
    counter: AtomicUsize,
}

impl wiremock::Respond for CyclicResponder {
    fn respond(&self, _request: &wiremock::Request) -> wiremock::ResponseTemplate {
        let idx = self.counter.fetch_add(1, Ordering::SeqCst);
        wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": format!("resp-{}", idx),
            "object": "chat.completion",
            "created": 1234567890,
            "model": "test-model",
            "choices": [{
                "index": 0,
                "message": {
                    "content": format!("batch-{}", idx),
                    "role": "assistant"
                },
                "finish_reason": "stop"
            }]
        }))
    }
}

#[tokio::test]
async fn test_analyze_images_result_ordering() {
    let mock_server = MockServer::start().await;

    // 使用单一 mock + AtomicUsize 轮询返回不同内容，消除 3 个相同 matcher 的竞态风险
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(CyclicResponder { counter: AtomicUsize::new(0) })
        .expect(3)  // 期望正好 3 次调用
        .mount(&mock_server)
        .await;

    let provider = create_test_provider(&mock_server.uri());
    let dir = TempDir::new().expect("创建临时目录失败");

    // 生成 3 个测试图片
    let img1 = create_test_jpeg_path(dir.path());
    let img2_path = dir.path().join("test_image2.jpg");
    write_test_jpeg(&img2_path).expect("写入测试图片失败");
    let img3_path = dir.path().join("test_image3.jpg");
    write_test_jpeg(&img3_path).expect("写入测试图片失败");

    let images = vec![img1, img2_path, img3_path];

    let results = provider
        .analyze_images(&images, "describe", None, Some(1), Some(2), None, None, None)
        .await;

    assert!(results.is_ok(), "analyze_images 应成功: {:?}", results.err());
    let results = results.unwrap();
    assert_eq!(results.len(), 3, "应为 3 个结果");
    assert_eq!(results[0], "batch-0", "结果 0 应保持原始顺序");
    assert_eq!(results[1], "batch-1", "结果 1 应保持原始顺序");
    assert_eq!(results[2], "batch-2", "结果 2 应保持原始顺序");
}

// ---------------------------------------------------------------------------
// 辅助函数：创建指向指定 base_url 的测试 provider
// ---------------------------------------------------------------------------
fn create_test_provider(base_url: &str) -> OpenAiCompatibleProvider {
    // provider 构造函数需要 /v1 路径；wiremock 的 uri() 返回类似 http://127.0.0.1:PORT
    let api_base = format!("{}/v1", base_url.trim_end_matches('/'));
    OpenAiCompatibleProvider::new(ProviderConfig {
        api_key: "test-key".to_string(),
        model_name: "test-model".to_string(),
        base_url: api_base,
        max_retries: 0,
        timeout_secs: 30,
        proxy_http: None,
        proxy_https: None,
    })
    .expect("测试 provider 创建失败")
}

// ---------------------------------------------------------------------------
// (测试辅助函数由 narratoai_core::llm::test_utils 提供)
// ---------------------------------------------------------------------------

