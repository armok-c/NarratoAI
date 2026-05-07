use std::path::PathBuf;
use std::pin::Pin;
use futures::stream::Stream;
use tokio_util::sync::CancellationToken;
use crate::error::LLMError;
use crate::llm::types::LlmResponseFormat;

/// LLM Provider 统一 trait
///
/// 单一 trait 包含三个方法——文本生成（非流式）、文本生成（流式）、图片视觉分析。
/// 底层都是 `/v1/chat/completions` 端点，只是 content 格式不同（D-04）。
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    /// 非流式文本生成
    ///
    /// 返回完整响应文本。当 response_format 为 Some(Json) 时，
    /// 实现应尝试 D-19 JSON 回退逻辑。
    async fn generate_text(
        &self,
        prompt: &str,
        system_prompt: Option<&str>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        response_format: Option<LlmResponseFormat>,
    ) -> Result<String, LLMError>;

    /// 流式文本生成
    ///
    /// 返回 token 文本流。每个 item 是从 ChatCompletionChunk delta.content 提取的 String 片段（D-09）。
    /// 流中途错误立即终止并返回 Err，不做重试（D-10）。
    async fn generate_text_stream(
        &self,
        prompt: &str,
        system_prompt: Option<&str>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, LLMError>> + Send>>, LLMError>;

    /// 视觉分析
    ///
    /// 接收图片文件路径列表 + 文本 prompt。内部完成图片预处理（thumbnail→JPEG→base64→data URL），
    /// 按 batch_size 分批，使用 Semaphore 控制并发（D-17, D-18），每批发送 vision 消息给 LLM。
    /// 返回按批次顺序排列的分析结果列表。
    async fn analyze_images(
        &self,
        images: &[PathBuf],
        prompt: &str,
        system_prompt: Option<&str>,
        batch_size: Option<usize>,
        max_concurrency: Option<usize>,
        response_format: Option<LlmResponseFormat>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        cancel: Option<CancellationToken>,
    ) -> Result<Vec<String>, LLMError>;
}
