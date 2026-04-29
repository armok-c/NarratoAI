use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use async_openai::{
    config::OpenAIConfig,
    error::OpenAIError,
    types::chat::{
        ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartImage,
        ChatCompletionRequestMessageContentPartText,
        ChatCompletionRequestSystemMessage, ChatCompletionRequestSystemMessageContent,
        ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
        ChatCompletionRequestUserMessageContentPart, CreateChatCompletionRequest,
        CreateChatCompletionRequestArgs, CreateChatCompletionResponse, ImageUrl,
        ResponseFormat,
    },
    Client,
};
use futures::stream::{Stream, StreamExt};
use tokio::sync::Semaphore;

use crate::error::LLMError;
use crate::llm::image_utils::image_to_base64_data_url;
use crate::llm::provider::LlmProvider;
use crate::llm::types::LlmResponseFormat;

/// Provider 配置参数
///
/// 用于构造 OpenAiCompatibleProvider 的配置结构体，避免 7 个位置参数的易错性。
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// API 密钥
    pub api_key: String,
    /// 模型名称
    pub model_name: String,
    /// API 基础 URL（如 https://api.openai.com/v1）
    pub base_url: String,
    /// 最大重试次数，传递给 OpenAI 客户端配置
    pub max_retries: u32,
    /// 请求超时秒数
    pub timeout_secs: u64,
    /// HTTP 代理 URL，None 表示不使用 HTTP 代理
    pub proxy_http: Option<String>,
    /// HTTPS 代理 URL，None 表示不使用 HTTPS 代理
    pub proxy_https: Option<String>,
}

/// OpenAI 兼容协议的 Provider 实现（D-06）
///
/// 支持任意符合 OpenAI /v1/chat/completions 协议的 API 网关。
/// 通过在构造函数中传入不同的 api_key / base_url / model_name 创建多个实例。
pub struct OpenAiCompatibleProvider {
    model_name: String,
    client: Client<OpenAIConfig>,
}

impl OpenAiCompatibleProvider {
    /// 使用 ProviderConfig 创建 OpenAiCompatibleProvider 实例
    pub fn new(cfg: ProviderConfig) -> Result<Self, LLMError> {
        let openai_config = OpenAIConfig::new()
            .with_api_key(&cfg.api_key)
            .with_api_base(cfg.base_url.trim_end_matches('/'))
            .with_max_retries(cfg.max_retries);

        let mut http_client_builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(cfg.timeout_secs));

        if let Some(ref proxy_url) = cfg.proxy_http {
            let proxy = reqwest::Proxy::http(proxy_url)
                .map_err(|e| LLMError::Configuration(format!("HTTP 代理配置失败: {}", e)))?;
            http_client_builder = http_client_builder.proxy(proxy);
        }

        if let Some(ref proxy_url) = cfg.proxy_https {
            let proxy = reqwest::Proxy::https(proxy_url)
                .map_err(|e| LLMError::Configuration(format!("HTTPS 代理配置失败: {}", e)))?;
            http_client_builder = http_client_builder.proxy(proxy);
        }

        let http_client = http_client_builder
            .build()
            .map_err(|e| LLMError::Configuration(format!("HTTP 客户端构建失败: {}", e)))?;

        let client = Client::with_config(openai_config)
            .with_http_client(http_client);

        Ok(Self {
            model_name: cfg.model_name,
            client,
        })
    }

    /// 构建文本生成的消息列表
    fn build_text_messages(
        &self,
        prompt: &str,
        system_prompt: Option<&str>,
    ) -> Vec<ChatCompletionRequestMessage> {
        let mut messages = Vec::new();

        if let Some(sp) = system_prompt {
            messages.push(ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessage {
                    content: ChatCompletionRequestSystemMessageContent::Text(sp.to_string()),
                    name: None,
                },
            ));
        }

        messages.push(ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text(prompt.to_string()),
                name: None,
            },
        ));

        messages
    }

    /// 从 chat completion 响应中提取文本内容
    fn extract_text(response: &CreateChatCompletionResponse) -> Result<String, LLMError> {
        response
            .choices
            .first()
            .and_then(|c| c.message.content.as_deref())
            .map(|s| s.to_string())
            .ok_or_else(|| LLMError::APICall("响应中没有有效文本内容".to_string()))
    }

    /// 带 JSON response_format 回退的文本生成（D-19）
    ///
    /// 某些网关不支持 response_format=json_object，会返回 400 错误。
    /// 此时回退到在 prompt 中追加 JSON 约束指示重新请求。
    ///
    /// 回退路径使用 builder API 重建请求而非 serde_json round-trip，
    /// 避免与 async-openai 的 serde 表示耦合。
    ///
    /// 注意：回退请求使用 build_text_messages() 重新构建消息列表，
    /// 该方法始终将 prompt 作为末条 user message，与原始请求结构一致。
    ///
    /// 限制：本方法仅支持单轮 system+user 对话模式。如果未来增加
    /// 多轮消息或其他角色（assistant、tool、function），
    /// 需要同时更新 build_text_messages() 和本回退路径。
    async fn generate_text_with_json_fallback(
        &self,
        request: CreateChatCompletionRequest,
        original_prompt: &str,
        system_prompt: Option<&str>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<CreateChatCompletionResponse, LLMError> {
        let result = self.client.chat().create(request).await;

        match result {
            Ok(response) => Ok(response),
            Err(OpenAIError::ApiError(api_err)) => {
                let msg_lower = api_err.message.to_lowercase();

                if msg_lower.contains("response_format") {
                    // 回退：使用 builder API 重建请求，避免 serde_json round-trip
                    let json_prompt = format!(
                        "{}\n\n请确保输出严格的JSON格式，不要包含任何其他文字或标记。",
                        original_prompt
                    );

                    let retry_messages = self.build_text_messages(&json_prompt, system_prompt);
                    let mut retry_builder = CreateChatCompletionRequestArgs::default();
                    retry_builder.model(&self.model_name);
                    retry_builder.messages(retry_messages);
                    if let Some(t) = temperature {
                        retry_builder.temperature(t);
                    }
                    if let Some(mt) = max_tokens {
                        retry_builder.max_tokens(mt);
                    }
                    let retry_request = retry_builder
                        .build()
                        .map_err(|e| LLMError::APICall(format!("请求重建失败: {}", e)))?;

                    let retry_response = self
                        .client
                        .chat()
                        .create(retry_request)
                        .await
                        .map_err(LLMError::from)?;

                    Ok(retry_response)
                } else {
                    Err(LLMError::from(OpenAIError::ApiError(api_err)))
                }
            }
            Err(e) => Err(LLMError::from(e)),
        }
    }
}

#[async_trait::async_trait]
impl LlmProvider for OpenAiCompatibleProvider {
    async fn generate_text(
        &self,
        prompt: &str,
        system_prompt: Option<&str>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        response_format: Option<LlmResponseFormat>,
    ) -> Result<String, LLMError> {
        let messages = self.build_text_messages(prompt, system_prompt);

        let mut request_builder = CreateChatCompletionRequestArgs::default();
        request_builder.model(&self.model_name);
        request_builder.messages(messages);

        if let Some(t) = temperature {
            request_builder.temperature(t);
        }

        if let Some(mt) = max_tokens {
            request_builder.max_tokens(mt);
        }

        // 处理 response_format
        let use_json = matches!(response_format, Some(LlmResponseFormat::Json));
        if use_json {
            request_builder.response_format(ResponseFormat::JsonObject);
        }

        let request = request_builder
            .build()
            .map_err(|e| LLMError::Configuration(format!("请求构建失败: {}", e)))?;

        let response = if use_json {
            self.generate_text_with_json_fallback(request, prompt, system_prompt, temperature, max_tokens)
                .await?
        } else {
            self.client
                .chat()
                .create(request)
                .await
                .map_err(LLMError::from)?
        };

        Self::extract_text(&response)
    }

    async fn generate_text_stream(
        &self,
        prompt: &str,
        system_prompt: Option<&str>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, LLMError>> + Send>>, LLMError> {
        let messages = self.build_text_messages(prompt, system_prompt);

        let mut request_builder = CreateChatCompletionRequestArgs::default();
        request_builder.model(&self.model_name);
        request_builder.messages(messages);

        if let Some(t) = temperature {
            request_builder.temperature(t);
        }

        if let Some(mt) = max_tokens {
            request_builder.max_tokens(mt);
        }

        let request = request_builder
            .build()
            .map_err(|e| LLMError::Configuration(format!("请求构建失败: {}", e)))?;

        let stream = self
            .client
            .chat()
            .create_stream(request)
            .await
            .map_err(LLMError::from)?;

        let mapped = stream.map(|chunk| match chunk {
            Ok(chunk) => {
                let content = chunk
                    .choices
                    .first()
                    .and_then(|c| c.delta.content.as_deref())
                    .unwrap_or("")
                    .to_string();
                Ok(content)
            }
            Err(e) => Err(LLMError::from(e)),
        });

        Ok(Box::pin(mapped))
    }

    async fn analyze_images(
        &self,
        images: &[PathBuf],
        prompt: &str,
        system_prompt: Option<&str>,
        batch_size: Option<usize>,
        max_concurrency: Option<usize>,
        response_format: Option<LlmResponseFormat>,
    ) -> Result<Vec<String>, LLMError> {
        // 预处理所有图片为 base64 data URL
        let data_urls: Vec<String> = images
            .iter()
            .map(|p| image_to_base64_data_url(p))
            .collect::<Result<Vec<_>, _>>()?;

        if data_urls.is_empty() {
            return Ok(Vec::new());
        }

        let batch_size = batch_size.unwrap_or(10).max(1);
        let max_concurrency = max_concurrency.unwrap_or(1);
        let bounded_concurrency = max_concurrency.max(1);

        // 将 data_urls 分片
        let chunks: Vec<Vec<String>> = data_urls.chunks(batch_size).map(|c| c.to_vec()).collect();

        let total_batches = chunks.len();
        let semaphore = Arc::new(Semaphore::new(bounded_concurrency));

        let mut handles = Vec::with_capacity(total_batches);

        for (batch_idx, batch) in chunks.into_iter().enumerate() {
            let sem_clone = semaphore.clone();
            let client_clone = self.client.clone();
            let prompt_owned = prompt.to_string();
            let system_prompt_owned = system_prompt.map(|s| s.to_string());
            let model_name = self.model_name.clone();
            let use_json = matches!(response_format, Some(LlmResponseFormat::Json));

            handles.push(tokio::spawn(async move {
                let _permit = sem_clone.acquire_owned().await.map_err(|_| {
                    LLMError::General("信号量获取失败".to_string())
                })?;

                // 构建包含图片和文本的 vision 消息
                let mut content_parts: Vec<ChatCompletionRequestUserMessageContentPart> =
                    Vec::with_capacity(1 + batch.len());

                content_parts.push(
                    ChatCompletionRequestUserMessageContentPart::Text(
                        ChatCompletionRequestMessageContentPartText {
                            text: prompt_owned,
                        },
                    ),
                );

                for b64 in &batch {
                    content_parts.push(
                        ChatCompletionRequestUserMessageContentPart::ImageUrl(
                            ChatCompletionRequestMessageContentPartImage {
                                image_url: ImageUrl {
                                    url: b64.clone(),
                                    detail: None,
                                },
                            },
                        ),
                    );
                }

                let mut messages: Vec<ChatCompletionRequestMessage> = Vec::with_capacity(2);
                if let Some(sp) = system_prompt_owned {
                    messages.push(ChatCompletionRequestMessage::System(
                        ChatCompletionRequestSystemMessage {
                            content: ChatCompletionRequestSystemMessageContent::Text(sp),
                            name: None,
                        },
                    ));
                }
                messages.push(ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessage {
                        content: ChatCompletionRequestUserMessageContent::Array(content_parts),
                        name: None,
                    },
                ));

                let mut request_builder = CreateChatCompletionRequestArgs::default();
                request_builder.model(&model_name);
                request_builder.messages(messages);
                if use_json {
                    request_builder.response_format(ResponseFormat::JsonObject);
                }
                let request = request_builder.build().map_err(|e| {
                    LLMError::Configuration(format!("请求构建失败: {}", e))
                })?;

                let response = client_clone
                    .chat()
                    .create(request)
                    .await
                    .map_err(LLMError::from)?;

                let text = response
                    .choices
                    .first()
                    .and_then(|c| c.message.content.as_deref())
                    .unwrap_or("")
                    .to_string();

                Ok::<_, LLMError>((batch_idx, text))
            }));
        }

        let results = futures::future::join_all(handles).await;

        let mut sorted_results: Vec<(usize, String)> = Vec::with_capacity(total_batches);
        for result in results {
            match result {
                Ok(Ok((idx, text))) => sorted_results.push((idx, text)),
                Ok(Err(e)) => return Err(e),
                Err(join_err) => {
                    return Err(LLMError::General(format!(
                        "任务执行失败: {}",
                        join_err
                    )));
                }
            }
        }

        sorted_results.sort_by_key(|(idx, _)| *idx);
        Ok(sorted_results.into_iter().map(|(_, text)| text).collect())
    }
}
