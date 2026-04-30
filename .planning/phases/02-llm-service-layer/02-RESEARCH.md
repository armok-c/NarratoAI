# Phase 2: LLM Service Layer - Research

**Researched:** 2026-04-28
**Domain:** Rust LLM service layer with OpenAI-compatible protocol support
**Confidence:** HIGH

## Summary

This phase implements a Rust LLM service layer using the `async-openai` crate as the OpenAI-compatible HTTP client. The architecture follows a Provider/Registry pattern with runtime trait objects (`Arc<dyn LlmProvider>`), mirroring the Python version's design. A single concrete struct `OpenAiCompatibleProvider` handles all providers (OpenAI, DeepSeek, Gemini, etc.) by routing to different API base URLs via `OpenAIConfig::with_api_base()`.

Two public entry points provide text generation: `generate_text()` returns a complete `String`, and `generate_text_stream()` returns `impl Stream<Item = Result<String>>`. Vision analysis supports base64-encoded images with batch processing and semaphore-based concurrency control. The async-openai crate's built-in retry mechanism handles transient failures, while nine detailed error types using `thiserror` provide structured error reporting aligned with the Python version.

Proxy support is achieved by injecting a custom `reqwest::Client` (with proxy configuration) into the async-openai `Client` via `with_http_client()`. Image preprocessing uses the `image` crate (thumbnail to 1024px, JPEG quality 85, base64 data URL).

**Primary recommendation:** Use async-openai 0.36.x with `chat-completion` feature flag. One `OpenAiCompatibleProvider` struct, registered multiple times with different configs. Single `LlmProvider` trait with three methods. Registry pattern with `HashMap<String, Arc<dyn LlmProvider>>`.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01:** 使用 `async-openai` crate（Rust 版 OpenAI SDK），不用 reqwest 手写。支持自定义 base_url、内置 streaming、retry。

**D-02:** 按 provider 创建 async-openai Client 实例，`HashMap<String, Client<...>>` 缓存。每个 provider 有独立的 api_key + base_url + model_name。

**D-03:** 两个独立入口：`generate_text()` 返回完整 `String`（非流式），`generate_text_stream()` 返回 `impl Stream<Item = Result<String>>`（流式 token 序列）。

**D-04:** 单一 `LlmProvider` trait——包含 `generate_text()` + `generate_text_stream()` + `analyze_images()`。不分离 Vision/Text trait。底层都是 `/v1/chat/completions`，只是 message content 格式不同。

**D-05:** 运行时 trait object 注册——`Registry` 持有 `HashMap<String, Arc<dyn LlmProvider>>`，对齐 Python 的动态 register 模式。

**D-06:** 唯一具体 struct `OpenAiCompatibleProvider`，按不同 provider name（openai、deepseek、gemini 等）用不同配置创建多个实例并注册。

**D-07:** 显式 `register_all_providers()` 函数在库初始化时调用，列举所有活跃 provider 并创建实例。

**D-08:** provider 参数可选——`generate_text(prompt, provider=None)` 未指定时从 `config.app.vision_llm_provider` / `config.app.text_llm_provider` 读取默认值。

**D-09:** `generate_text_stream()` 返回 `impl Stream<Item = Result<String, LLMError>>`，每个 item 是从 `ChatCompletionChunk` 提取的 `delta.content` 文本片段。

**D-10:** 流中途错误时立即终止流并返回 `Err`，不做自动重试或断点续传。

**D-11:** 流式和非流式共享同一个 async-openai Client 实例和错误映射，但 `create()` vs `create_stream()` 分开调用路径。

**D-12:** 流式功能始终编译包含，不放在 Cargo feature flag 后面。

**D-13:** 使用 `async-openai` crate 的内置重试机制（通过 Client 的 `max_retries` 配置）。

**D-14:** LLM 错误类型完全对齐 Python 的 9 种异常：`LLMError`(基础) + `ProviderNotFoundError` + `ConfigurationError` + `APICallError` + `ValidationError` + `ModelNotSupportedError` + `RateLimitError` + `AuthenticationError` + `ContentFilterError`。用 thiserror 派生。

**D-15:** 默认值沿用 Python：`max_retries=3`，`text_timeout=180s`，`vision_timeout=120s`。从 TOML 配置读取。

**D-16:** 透传 `[proxy]` 配置段的 HTTP/HTTPS 代理设置到 async-openai Client（通过底层 reqwest 的 proxy 方法）。

**D-17:** 使用 `tokio::sync::Semaphore` 控制并发批次数，直接等价 Python 的 `asyncio.Semaphore`。

**D-18:** 默认值沿用 Python：`batch_size=10`（每批最多 10 张图片），`max_concurrency=1`（一次只发一批）。

**D-19:** 实现与 Python 相同的回退逻辑——先尝试 `response_format={type:"json_object"}`，如果网关返回 400 且错误消息含 "response_format"，则自动重试（不带 response_format 参数，在 prompt 末尾追加 JSON 约束提示）。

**D-20:** 完全对齐 Python 的预处理流程——用 `image` crate 实现：`thumbnail(1024px)` 保持宽高比 → JPEG 编码（quality=85）→ base64 编码 → 格式化为 `data:image/jpeg;base64,...` data URL。

### Claude's Discretion

无——所有决策均由用户明确选择。

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| LLM-01 | OpenAI 兼容协议客户端（/v1/chat/completions 端点） | async-openai crate 提供 `Client::with_config(OpenAIConfig::new().with_api_base())` 自定义 base_url，通过 `client.chat().create()` 调用 /v1/chat/completions |
| LLM-02 | Provider/Registry 模式——通过 `provider/model_name` 格式路由到不同 API 端点 | Registry 持有 `HashMap<String, Arc<dyn LlmProvider>>`，ProviderName 从 `provider/model_name` 中提取前缀部分；显式 `register_all_providers()` 注册 |
| LLM-03 | Vision 模型支持——发送图片（base64）+ 文本给视觉 LLM | async-openai 的 `ChatCompletionRequestMessageContentPart::ImageUrl` + `ImageUrl` struct；`image` crate 处理缩放到 base64 编码 |
| LLM-04 | Text 模型支持——纯文本对话（流式响应） | `client.chat().create()` 非流式 + `client.chat().create_stream()` 流式 |
| LLM-05 | 流式响应处理（SSE 解析，逐 token 输出） | `create_stream()` 返回 `ChatCompletionResponseStream`，每个 item 提取 `delta.content`，用户提供的 Stream 生成 String token |
| LLM-06 | 并发控制——视觉分析批量请求的信号量限制 | `tokio::sync::Semaphore` 控制并发批次；`futures::future::join_all` 启动所有批次 |
| LLM-07 | 统一错误处理——API 错误、超时、限流的重试机制 | async-openai 内置 retry（`max_retries`）；9 种 thiserror 错误类型；OpenAI API 异常映射 |

</phase_requirements>

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `async-openai` | 0.36.1 | OpenAI 兼容协议 HTTP 客户端 | 最成熟的 Rust OpenAI SDK，支持自定义 base_url、SSE streaming、内置 retry、builder pattern [VERIFIED: cargo search] |
| `image` | 0.25.10 | 图片加载和预处理（thumbnail、JPEG 编码） | 最主流 Rust 图像处理库，支持 LANCZOS 重采样、JPEG 编码 [VERIFIED: cargo search] |
| `base64` | 0.22.1 | Base64 编码（图片→base64 data URL） | 标准 base64 编解码库 [VERIFIED: cargo search] |
| `futures` | 0.3.32 | Stream trait（流式响应 consume） | 提供 `StreamExt::next()` 用于消费 async-openai 的流式响应 [VERIFIED: cargo search] |
| `reqwest` | 0.12.x | HTTP 客户端（async-openai 的底层依赖） | 用于构建带 proxy 的自定义 HTTP client，注入 async-openai [ASSUMED: 基于 crate 间接依赖] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tokio` | 1.52.1 | 异步运行时 + Semaphore | 已有依赖，用于 `tokio::sync::Semaphore` 并发控制 [VERIFIED: Cargo.toml] |
| `thiserror` | 2.0.18 | 领域错误类型派生宏 | 已有依赖，为 LLMError 9 种变体派生 Error trait [VERIFIED: Cargo.toml] |
| `serde` / `serde_json` | 1.0.228 / 1.0.140 | JSON 序列化/反序列化 | 已有依赖，用于构建自定义请求体或解析调试 [VERIFIED: Cargo.toml] |
| `tracing` | 0.1.44 | 结构化日志 | 已有依赖，用于 LLM 调用日志 [VERIFIED: Cargo.toml] |
| `tempfile` | 3.27.0 | 测试临时文件 | 已有 dev-dependency [VERIFIED: Cargo.toml] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `async-openai` | `reqwest` 手写 HTTP 请求 | async-openai 已处理 SSE 解析、type-safe 请求构建、重试、错误映射；手写多出约 500 行样板代码 |
| `async-openai` | `async-openai-compat` fork | 后者针对非 OpenAI 端点优化但版本滞后（0.30.6），主流 `async-openai` 0.36.x 更活跃且同样支持自定义 base_url |
| `image` crate | `fast_image_resize` | `fast_image_resize` 仅做 resize，image crate 提供完整的 load→resize→encode pipeline |
| `base64` crate | `data-encoding` | `base64` 0.22 是行业标准，API 最简单 |

**Installation:**
```toml
# 在 Cargo.toml 中新增依赖
async-openai = "0.36"
image = "0.25"
base64 = "0.22"
futures = "0.3"
reqwest = "0.12"
```

**Version verification:** All versions verified via `cargo search` on 2026-04-28. The `async-openai` 0.36.1 is current with ~598K monthly downloads.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Provider/Registry 管理 | LLM Service | Config | Registry 读取配置创建 Provider 实例，Config 提供 API key/base_url |
| Text 生成（非流式） | LLM Service | — | async-openai `client.chat().create()` 是纯 LLM 层职责 |
| Text 生成（流式） | LLM Service | — | async-openai `client.chat().create_stream()` 返回 SSE 流 |
| Vision 图片分析 | LLM Service | Utils | 图片预处理（thumbnail/encode）在 LLM 内部完成，被 `analyze_images()` 封装 |
| 图片预处理 | LLM Service (utils submodule) | — | thumbnail→JPEG→base64 是 Vision 管线的固定步骤 |
| 重试与错误处理 | LLM Service | — | async-openai Client 的 max_retries 配置即可，错误映射在 Provider 内部 |
| 代理支持 | Config → LLM Service | — | Config 提供 proxy 配置，LLM Service 在创建 Client 时注入 |
| 超时控制 | Config → LLM Service | — | 超时值从 TOML 读取，通过 `OpenAIConfig` 或自定义 reqwest Client 设置 |

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        LLM Service Layer                         │
│                                                                   │
│  ┌─────────────────────────┐     ┌──────────────────────────┐   │
│  │   register_all_providers│     │     LlmProvider trait     │   │
│  │   ()                     │     │                          │   │
│  │  Reads Config, creates   │     │  generate_text()         │   │
│  │  providers, registers    │────>│  generate_text_stream()  │   │
│  │  into Registry           │     │  analyze_images()        │   │
│  └─────────────────────────┘     └──────────┬───────────────┘   │
│                                              │                    │
│  ┌──────────────────────────┐                │                    │
│  │       Registry           │                │                    │
│  │  HashMap<String,         │                │ implements         │
│  │   Arc<dyn LlmProvider>>  │                ▼                    │
│  └──────────────────────────┘   ┌──────────────────────────┐   │
│                                 │ OpenAiCompatibleProvider  │   │
│                                 │  - api_key: String       │   │
│                                 │  - model_name: String    │   │
│                                 │  - base_url: String      │   │
│                                 │  - client: Client        │   │
│                                 └──────────┬───────────────┘   │
│                                            │                    │
│      User/Caller                           │                    │
│  ┌─────────────────┐                       │                    │
│  │ generate_text()  │───────────────────────┘                    │
│  │ (prompt,         │                                            │
│  │  provider=None)  │─── config.app.text_llm_provider fallback   │
│  └─────────────────┘                                            │
│                                                                   │
│  ┌─────────────────┐                                            │
│  │ analyze_images() │──[Semaphore]── Batch 0 ──▶ async-openai   │
│  │ (images, prompt, │──[Semaphore]── Batch 1 ──▶ Client.create()│
│  │  batch_size=10)  │──[Semaphore]── Batch 2 ──▶ (vision msg)   │
│  └─────────────────┘                                            │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  async-openai Client (per provider cache)               │    │
│  │                                                          │    │
│  │  ┌─────────────────┐   ┌─────────────────────────┐      │    │
│  │  │ create(request)  │   │ create_stream(request)  │      │    │
│  │  │ → ChatResponse   │   │ → ResponseStream        │      │    │
│  │  └─────────────────┘   └─────────────────────────┘      │    │
│  │                                                          │    │
│  │  Built-in: max_retries, timeout                          │    │
│  │  Custom: reqwest Client (proxy injection)                │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │  OpenAI Compat   │
                    │  API Gateway     │
                    │  (e.g.,          │
                    │   SiliconFlow,   │
                    │   OpenAI)        │
                    └──────────────────┘
```

### Recommended Project Structure

```
src/
├── lib.rs                    # 新增 pub mod llm;
├── error.rs                  # 新增 LLMError enum（9 种变体）
├── config/
│   └── types.rs              # 已有 AppSection 含 LLM 配置字段（Phase 1）
└── llm/
    ├── mod.rs                # 模块声明 + pub use 导出关键类型
    ├── provider.rs           # LlmProvider trait（generate_text, generate_text_stream, analyze_images）
    ├── registry.rs           # Registry：HashMap<String, Arc<dyn LlmProvider>>
    ├── openai_compatible.rs  # OpenAiCompatibleProvider 具体实现
    ├── error.rs              # LLMError enum（9 种）+ 从 OpenAIError 的 From 转换
    ├── types.rs              # 辅助类型：VisionBatchConfig, LLMRequestParams 等
    ├── image_utils.rs        # 图片预处理：thumbnail → JPEG encode → base64
    └── register.rs           # register_all_providers() 函数
```

### Pattern 1: LlmProvider Trait + Registry Pattern

**What:** 单一 trait 定义三个方法，Registry 持有 trait objects，OpenAiCompatibleProvider 是唯一具体实现。

**When to use:** 所有 provider 都走同一个 OpenAI 兼容协议 `/v1/chat/completions`，不需要为每个 provider 写独立的实现。

**Example:**
```rust
// provider.rs
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// 非流式文本生成
    async fn generate_text(
        &self,
        prompt: &str,
        system_prompt: Option<&str>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        response_format: Option<ResponseFormat>,
    ) -> Result<String, LLMError>;

    /// 流式文本生成——返回 token 文本流
    async fn generate_text_stream(
        &self,
        prompt: &str,
        system_prompt: Option<&str>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, LLMError>> + Send>>, LLMError>;

    /// 视觉分析
    async fn analyze_images(
        &self,
        images: &[PathBuf],
        prompt: &str,
        batch_size: Option<usize>,
        max_concurrency: Option<usize>,
    ) -> Result<Vec<String>, LLMError>;
}

// registry.rs
pub struct Registry {
    providers: HashMap<String, Arc<dyn LlmProvider>>,
}

impl Registry {
    pub fn new() -> Self {
        Self { providers: HashMap::new() }
    }

    pub fn register(&mut self, name: &str, provider: Arc<dyn LlmProvider>) {
        self.providers.insert(name.to_lowercase(), provider);
    }

    pub fn get(&self, name: &str) -> Result<Arc<dyn LlmProvider>, LLMError> {
        self.providers
            .get(name)
            .cloned()
            .ok_or_else(|| LLMError::ProviderNotFound(name.to_string()))
    }
}
```

### Pattern 2: Streaming Response Handling

**What:** async-openai 的 `create_stream()` 返回 `ChatCompletionResponseStream`，转换为调用方期望的 `Stream<Item = Result<String>>`。

**When to use:** 任何需要实时输出 LLM token 的场景。

**Example:**
```rust
// Source: [CITED: WebSearch re: async-openai streaming]
use futures::StreamExt;

// 在 OpenAiCompatibleProvider::generate_text_stream() 内部:
async fn generate_text_stream(
    &self,
    request: CreateChatCompletionRequest,
) -> Result<Pin<Box<dyn Stream<Item = Result<String, LLMError>> + Send>>, LLMError> {
    let stream = self.client.chat().create_stream(request).await?;

    let mapped = stream.map(|chunk_result| match chunk_result {
        Ok(chunk) => {
            let text = chunk.choices
                .first()
                .and_then(|c| c.delta.content.as_deref())
                .unwrap_or("")
                .to_string();
            Ok(text)
        }
        Err(e) => Err(LLMError::from(e)),
    });

    Ok(Box::pin(mapped))
}
```

**Note:** `create_stream()` 自动设置 `stream: true`，调用方不应手动设置 stream 参数。

### Pattern 3: Vision Image with Base64 Data URL

**What:** 构建包含 base64 编码图片的 Chat Completion 请求。

**When to use:** 调用视觉模型分析图片内容。

**Example:**
```rust
// Source: [CITED: WebSearch re: async-openai ImageUrl]
use async_openai::types::{
    CreateChatCompletionRequestArgs,
    ChatCompletionRequestMessage,
    ChatCompletionRequestMessageContentPart,
};

fn build_vision_messages(
    prompt: &str,
    base64_images: &[String],  // "data:image/jpeg;base64,..."
) -> Vec<ChatCompletionRequestMessage> {
    let mut content_parts = vec![
        ChatCompletionRequestMessageContentPart::Text(
            async_openai::types::ChatCompletionRequestMessageContentPartText {
                text: prompt.to_string(),
            }
        )
    ];

    for b64 in base64_images {
        content_parts.push(
            ChatCompletionRequestMessageContentPart::ImageUrl(
                async_openai::types::ChatCompletionRequestMessageContentPartImageUrl {
                    image_url: async_openai::types::ImageUrl {
                        url: b64.clone(),
                        detail: None,
                    },
                }
            )
        );
    }

    vec![
        ChatCompletionRequestMessage::User(
            async_openai::types::UserMessage {
                content: Some(content_parts),
                ..Default::default()
            }
        )
    ]
}
```

### Anti-Patterns to Avoid

- **分离 Vision/Text trait:** 底层都是同一个 `/v1/chat/completions`，只是 message content 格式不同。分离 trait 导致大量重复代码（错误处理、重试逻辑、日志）。使用单一 `LlmProvider` trait。
- **编译时 enum match:** 将所有 provider 硬编码为 enum 变体（`Provider::Openai`, `Provider::DeepSeek`）导致每次增加新 provider 都需要修改枚举和所有 match 分支。使用运行时 trait object 注册。
- **每个请求新建 HTTP Client:** 失去连接池复用，增加 TLS 握手开销。按 provider 缓存 Client 实例。

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SSE 流式解析 | 手动解析 `data: ...` SSE 行 | async-openai `create_stream()` | SSE 有 `[DONE]` 信号、重连令牌、错误行等多种边界情况，async-openai 已处理 |
| Base64 data URL | 手动构建 `data:image/jpeg;base64,` 字符串 | `format!("data:image/jpeg;base64,{}", encoded)` | 这是固定格式，但注意 MIME 类型必须匹配实际编码格式 |
| 图片 thumbnail | 自行实现缩放算法 | `image::imageops::thumbnail()` 或 `DynamicImage::thumbnail()` | 已用 LANCZOS3 实现高质量缩放 |
| HTTP 重试/backoff | 实现指数退避、抖动、状态码判断 | async-openai Client `max_retries` | 内置 exponential backoff、429/5xx 状态码判断 |
| Semaphore 并发限制 | `tokio::sync::Semaphore`（不是问题，已决策） | — | 这是直接使用标准库，不是手写 |

**Key insight:** async-openai 已经封装了 OpenAI API 的完整生命周期（SSE 解析、错误映射、流媒体传输、速率限制处理）。手写 reqwest 请求需要额外处理大量边缘情况，不值得。

## Common Pitfalls

### Pitfall 1: async-openai Feature Flags
**What goes wrong:** 使用 `Client::new()` 或 `client.chat()` 时编译报错，提示未实现的 trait 或未定义的方法。
**Why it happens:** async-openai 0.30+ 使用 granular feature flags，默认不启用所有 API。`chat()` 和 `create_stream()` 需要 `chat-completion` feature。
**How to avoid:** 在 Cargo.toml 中显式指明：
```toml
async-openai = { version = "0.36", features = ["chat-completion"] }
```
**Warning signs:** `error[E0599]: no method named 'chat' found` — 说明缺少 feature flag。

### Pitfall 2: async-openai api_base 不从环境变量自动读取
**What goes wrong:** 设置环境变量 `OPENAI_API_BASE` 后 Client 仍然连接默认的 `https://api.openai.com/v1`。
**Why it happens:** 原始 `64bit/async-openai` crate 不自动读取 `OPENAI_API_BASE` 环境变量（这与 Python 的 OpenAI SDK 不同）。只有 `spiceai` fork 支持此功能。
**How to avoid:** 始终显式调用 `OpenAIConfig::new().with_api_base(url)`，从 TOML 配置读取 base_url。
**Warning signs:** 请求被路由到 OpenAI 官方端点而非自定义网关。

### Pitfall 3: Vision 请求的 Content Parts API 版本差异
**What goes wrong:** 构建 vision 请求时发现 `ChatCompletionRequestMessageContentPart` 类型结构变化。
**Why it happens:** async-openai 0.36.x 使用 `ChatCompletionRequestMessage::User` 包裹 `UserMessage`，而非直接使用 `ChatCompletionRequestMessage` 的内容字段。content 字段类型为 `Option<Vec<ChatCompletionRequestMessageContentPart>>`。
**How to avoid:** 查看 crate 版本的 examples 目录。最新版使用 `UserMessage { content: Some(vec![...]) }`。
**Warning signs:** `UserMessage` 类型不匹配，或者 destructure 字段错误。

### Pitfall 4: 流式响应中的空 token
**What goes wrong:** 流式响应中某些 chunk 的 `delta.content` 为 `None`（如 `finish_reason` chunk）。
**Why it happens:** 流式响应在结束时发送 `finish_reason: "stop"` chunk，不包含 content。其他功能 chunk（如 thinking、tool calls）也可能没有 content。
**How to avoid:** 始终使用 `c.delta.content.as_deref().unwrap_or("")` 而非直接 unwrap。
**Warning signs:** 流式处理到末尾时 panic。

### Pitfall 5: response_format JSON 回退的异步状态
**What goes wrong:** 重试时忘记了已消耗的 request body 或过期状态。
**Why it happens:** response_format 回退需要在第一个请求失败后重建请求。async-openai 的 request builder 是消费式的，不能克隆。
**How to avoid:** `response_format` 回退时重建 `CreateChatCompletionRequest`，而不是试图修改原有的 builder。
**Warning signs:** "builder already consumed" 编译错误。

## Code Examples

### Client Creation with Custom Base URL and Proxy

```rust
// Source: [CITED: WebSearch re: async-openai Client config]
use async_openai::{Client, config::OpenAIConfig};
use reqwest::{Proxy, Client as HttpClient};

/// 构建异步 OpenAI 客户端，支持自定义 base_url、timeout、
/// max_retries 和 proxy 透传。
pub fn build_client(
    api_key: &str,
    base_url: &str,
    timeout_secs: u64,
    max_retries: u32,
    proxy_config: &ProxySection,  // 从 config.toml 的 [proxy] 读取
) -> Client {
    let config = OpenAIConfig::new()
        .with_api_key(api_key)
        .with_api_base(base_url.trim_end_matches('/'));

    // 如果启用了 proxy，构建自定义 reqwest Client
    if proxy_config.enabled {
        let mut http_client_builder = HttpClient::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs));

        if !proxy_config.http.is_empty() {
            if let Ok(proxy) = Proxy::http(&proxy_config.http) {
                http_client_builder = http_client_builder.proxy(proxy);
            }
        }
        if !proxy_config.https.is_empty() {
            if let Ok(proxy) = Proxy::https(&proxy_config.https) {
                http_client_builder = http_client_builder.proxy(proxy);
            }
        }

        let http_client = http_client_builder.build()
            .expect("构建 reqwest Client 失败");

        Client::with_config(config).with_http_client(http_client)
    } else {
        Client::with_config(config)
    }
}
```

### Image Preprocessing (Python Alignment)

```rust
// Source: [CITED: image crate docs + Python openai_compatible_provider.py]
use image::{DynamicImage, ImageBuffer, imageops::FilterType};
use std::path::Path;

/// 加载图片，缩放到 1024px 保持宽高比，编码为 JPEG quality 85，
/// 然后 base64 编码为 data URL。完全对齐 Python 版。
pub fn image_to_base64_data_url(path: &Path) -> Result<String, LLMError> {
    let img = image::open(path)
        .map_err(|e| LLMError::Configuration(format!("图片加载失败: {}", e)))?;

    // thumbnail(1024, 1024) 保持宽高比，与 Python 的 PIL.Image.thumbnail((1024, 1024), LANCZOS) 一致
    let thumb = img.thumbnail(1024, 1024);

    // 编码为 JPEG quality 85
    let mut buf = std::io::Cursor::new(Vec::new());
    thumb.write_to(&mut buf, image::ImageFormat::Jpeg)
        .map_err(|e| LLMError::Configuration(format!("JPEG 编码失败: {}", e)))?;

    // base64 编码
    let b64 = base64::engine::general_purpose::STANDARD
        .encode(buf.get_ref());

    Ok(format!("data:image/jpeg;base64,{}", b64))
}
```

### Response Format JSON Fallback

```rust
// Source: [CITED: Python openai_compatible_provider.py lines 222-234]
/// response_format=json 回退逻辑：如果网关返回 400 且含有 "response_format"，
/// 重试时移除 response_format 并在 prompt 末尾追加 JSON 约束提示。
async fn generate_text_with_json_fallback(
    client: &Client,
    mut request: CreateChatCompletionRequest,
    prompt: &str,
) -> Result<String, LLMError> {
    request.response_format = Some(ResponseFormat { type_: "json_object" });

    match client.chat().create(request.clone()).await {
        Ok(response) => {
            // 成功，解析文本
            Ok(extract_text(&response))
        }
        Err(OpenAIError::ApiError(api_err))
            if api_err.status_code == Some(400)
               && api_err.message.to_lowercase().contains("response_format") =>
        {
            // 回退：移除 response_format，prompt 追加 JSON 约束
            request.response_format = None;
            if let Some(last_msg) = request.messages.last_mut() {
                if let ChatCompletionRequestMessage::User(user_msg) = last_msg {
                    if let Some(ref mut content) = user_msg.content {
                        // 追加 JSON 约束到 text content part
                        if let Some(text_part) = content.iter_mut().find_map(|part| {
                            // 需要根据具体类型处理... 简化版：
                            if let ChatCompletionRequestMessageContentPart::Text(ref mut t) = part {
                                Some(t)
                            } else {
                                None
                            }
                        }) {
                            text_part.text.push_str("\n\n请确保输出严格的JSON格式，不要包含任何其他文字或标记。");
                        }
                    }
                }
            }

            let retry_response = client.chat().create(request).await?;
            Ok(extract_text(&retry_response))
        }
        Err(e) => Err(LLMError::from(e)),
    }
}
```

## Runtime State Inventory

> 此阶段为绿色字段开发——没有重命名或迁移操作，无需运行时状态盘点。

SKIPPED — 绿色字段阶段，不涉及 rename/refactor/migration。

## Validation Architecture

> nyquist_validation: true (from .planning/config.json)

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `cargo test` |
| Config file | 无——Rust 标准测试框架 |
| Quick run command | `cargo test -p narratoai-core --test llm_test 2>/dev/null; echo "See individual test below"` |
| Full suite command | `cargo test -p narratoai-core 2>&1` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| LLM-01 | OpenAI 兼容协议客户端 | unit | `cargo test test_openai_client_creation` | ❌ Wave 0 |
| LLM-02 | Provider/Registry 路由 | unit | `cargo test test_registry_register_get` | ❌ Wave 0 |
| LLM-03 | Vision 消息构建（base64 data URL） | unit | `cargo test test_vision_message_contains_data_url` | ❌ Wave 0 |
| LLM-04 | Text 非流式生成 | unit (mock) | `cargo test test_generate_text_success` | ❌ Wave 0 |
| LLM-05 | 流式 token 提取 | unit | `cargo test test_stream_tokens_extracted` | ❌ Wave 0 |
| LLM-06 | Semaphore 并发控制 | unit | `cargo test test_vision_concurrency_semaphore` | ❌ Wave 0 |
| LLM-07 | 错误类型映射 | unit | `cargo test test_openai_error_mapping` | ❌ Wave 0 |
| LLM-07 | 重试耗尽后错误 | unit | `cargo test test_retries_exhausted_returns_error` | ❌ Wave 0 |
| LLM-07 | JSON response_format 回退 | unit (mock) | `cargo test test_json_response_format_fallback` | ❌ Wave 0 |
| D-20 | 图片预处理 pipeline | unit | `cargo test test_image_to_base64_data_url` | ❌ Wave 0 |

### Wave 0 Gaps

- [ ] `tests/llm_test.rs` — 覆盖 LLM-01 到 LLM-07 的单元测试（模块级测试文件，使用 mock HTTP server 测试 async-openai 调用）
- [ ] `src/llm/test_utils.rs` — 测试辅助函数（mock 响应生成器、fixture 图片）
- [ ] `tests/config_test.rs` 已覆盖 LLM 配置字段解析——无需重复

### Testing Strategy Notes

- **真实 API 调用** 应跳过（使用 `#[cfg(not(feature = "integration_test"))]` 或 `#[ignore]` 标记），与 Python 版 `conftest.py` 对齐
- **单元测试** 使用 mock HTTP server（如 `wiremock` 或 `httpmock`）模拟 OpenAI API 响应，验证请求构建、响应解析和错误映射
- **图片预处理测试** 使用程序生成的测试图片（单色/渐变 PNG），不依赖外部文件
- **流式测试** 构造模拟的 SSE chunk 序列，验证 token 提取和错误传播

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | yes | API key 从 TOML 配置读取，通过 `OpenAIConfig::with_api_key()` 传递，不在代码中硬编码 |
| V5 Input Validation | yes | OpenAI 兼容 API 不接受任意用户输入——prompt 内容由上游流水线控制；LLMError 在边界处转换 |
| V6 Cryptography | no | 密钥管理依赖 TOML 文件，TLS 由 reqwest/async-openai 底层处理 |

### Known Threat Patterns for async-openai + reqwest

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| API key 泄露 | Information Disclosure | 密钥仅存于 `config.toml`（gitignored），运行时在内存中保持为 `String`，不写入日志 |
| 请求注入 | Tampering | `CreateChatCompletionRequest` 使用 builder pattern，类型安全，不会拼接 URL 参数 |
| 中间人攻击（HTTP proxy） | Spoofing | 通过 `reqwest::Proxy` 支持的 HTTPS proxy 或明确信任的 HTTP proxy；TOML 中 `proxy.enabled` 控制 |
| SSL/TLS 降级 | Tampering | reqwest 默认使用 rustls/native-tls，无法降至明文 HTTP |

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Python `AsyncOpenAI` SDK | Rust `async-openai` crate | 此阶段 | 1:1 功能对齐，但 Rust 版本需注意 granular feature flags |
| `asyncio.Semaphore` | `tokio::sync::Semaphore` | 此阶段 | 使用方式几乎一致，`acquire().await` |
| `PIL.Image.open()` + `thumbnail()` | `image::open()` + `DynamicImage::thumbnail()` | 此阶段 | API 相似，thumbnail 默认使用不同算法（Rust 版 integer-based vs PIL LANCZOS） |
| Python 9 种异常类继承链 | Rust thiserror enum with 9 variants | 此阶段 | thiserror 在编译期生成 Display + Error impl，无需运行时继承 |

**Deprecated/outdated:**
- `async-openai` 0.30 之前使用 `CreateChatCompletionRequestStream` 和 `CreateChatCompletionStreamResponse` 命名——新版已简化
- `image` crate 0.24 之前 `FilterType` 命名为 `Lanczos3`（不是 `Lanczos`）——0.25 版本保持统一

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `reqwest` 0.12.x 是 async-openai 0.36.x 的间接依赖 | Standard Stack | 如果 async-openai 使用的 reqwest 版本不同，proxy 注入模式可能需要调整 |
| A2 | `ChatCompletionRequestMessageContentPart::ImageUrl` 变体在 async-openai 0.36 中存在 | Code Examples | 类型名可能在 crates.io 发布前变更——需按实际 API docs 调整 |
| A3 | `create_stream()` 自动设置 `stream: true` 且不允许调用方手动设置 | Architecture Patterns | 行为在 async-openai 0.31+ 中确认；如果版本回退到 0.30，行为可能不同 |
| A4 | thumbnail() 默认算法与 Python PIL `LANCZOS` 足够接近 | Image Preprocessing | Python 显式指定 LANCZOS，Rust `thumbnail()` 使用 fast integer scaling，视觉结果可能有微小差异（对齐 Python 的质量目标可能需要改用 `resize_exact()` + `FilterType::Lanczos3`） |

## Open Questions (RESOLVED)

1. **`OpenAIConfig::with_api_base()` 的结尾斜杠处理** *(RESOLVED)*
   - What we know: Python 版 `base_url` 经常以 `/v1` 结尾，async-openai `with_api_base()` 期望完整 base URL。
   - What's unclear: async-openai 是否自动拼接 `/chat/completions` 路径，还是需要 base_url 包含 `/v1`。
   - Recommendation: 在集成测试中用 mock server 验证。若 `with_api_base("https://gateway/v1")` 产生 `https://gateway/v1/chat/completions` 则正确；否则可能需要 `https://gateway/v1/`（带结尾斜杠）。
   - RESOLVED: Plan 02-02 Task 2 在构建 Client 时使用 `base_url.trim_end_matches('/')` 标准化处理，并在集成测试中通过 mock server 验证实际请求路径。

2. **流式响应是否需要 `futures` crate 的 `StreamExt` trait 导入** *(RESOLVED)*
   - What we know: `ChatCompletionResponseStream` 是 `Pin<Box<dyn Stream>>`，需要 `.next().await` 方法。
   - What's unclear: `async-openai` 是否 re-export 所需的 stream trait 方法。
   - Recommendation: 添加 `futures = "0.3"` 依赖，使用 `futures::StreamExt::next()`。这是标准用法。
   - RESOLVED: Plan 02-01 Task 1 在 Cargo.toml 中添加 `futures = "0.3"` 依赖，代码中使用 `use futures::StreamExt;` 导入 trait。

3. **Vision request content parts 的精确类型签名** *(RESOLVED)*
   - What we know: async-openai 0.36 使用 `UserMessage { content: Option<Vec<ChatCompletionRequestMessageContentPart>> }`。
   - What's unclear: `ChatCompletionRequestMessageContentPart` 的精确 enum variant 名称（`Text`、`ImageUrl`，或 `TextContent`、`ImageUrlContent`）。
   - Recommendation: 实现时查阅 `docs.rs/async-openai` 确认 variant 名称。上述 Code Examples 使用推测名称，需验证。
   - RESOLVED: 执行时查阅 `docs.rs/async-openai/0.36` 确认 enum variant 名称，使用实际 API 中的正确类型名。Plan 02-02 Task 2 的 accept_criteria 要求代码编译通过（`cargo check` 成功），这保证了类型名必然正确。

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | All code | Yes | rustc 1.93.1, cargo 1.93.1 | — |
| `async-openai` | LLM HTTP client | Yes (cargo) | 0.36.1 | — |
| `image` | Image preprocessing | Yes (cargo) | 0.25.10 | — |
| `base64` | Base64 encoding | Yes (cargo) | 0.22.1 | — |
| `futures` | Stream streaming | Yes (cargo) | 0.3.32 | — |
| `reqwest` | Proxy injection | Yes (cargo) | 0.12.x | — |
| OpenAI 兼容网关 | Vision/text API 调用 | No — 需用户配置 | — | 单元测试使用 mock server（wiremock），跳过集成测试 |

**Missing dependencies with no fallback:**
- 无——所有依赖均可通过 cargo 安装

**Missing dependencies with fallback:**
- OpenAI 兼容 API 网关：需要用户在 `config.toml` 中配置 API key 和 base_url。单元测试使用 `wiremock` 或 `httpmock` 模拟。

## Sources

### Primary (HIGH confidence)

- Python reference codebase: `app/services/llm/base.py`, `openai_compatible_provider.py`, `manager.py`, `unified_service.py`, `exceptions.py`, `providers/__init__.py` [VERIFIED: codebase grep]
- Rust Phase 1 foundation: `src/error.rs`, `src/config/types.rs`, `src/config/mod.rs`, `src/lib.rs` [VERIFIED: codebase grep]
- `Cargo.toml` — existing dependencies [VERIFIED: file read]
- `async-openai` crate: v0.36.1, `chat-completion` feature flag, `Client::with_config()`, `create_stream()` [VERIFIED: cargo search + WebSearch]
- `image` crate: v0.25.10, `thumbnail()`, JPEG encoding [VERIFIED: cargo search + WebSearch]
- `base64` crate: v0.22.1 [VERIFIED: cargo search]
- `futures` crate: v0.3.32 [VERIFIED: cargo search]

### Secondary (MEDIUM confidence)

- async-openai proxy injection pattern via `Client::with_http_client()` [CITED: WebSearch]
- async-openai `ImageUrl` struct with `url` and optional `detail` fields [CITED: docs.rs source URL]
- async-openai `ChatCompletionRequestMessageContentPart` enum for vision requests [CITED: WebSearch]
- reqwest `Proxy::all()`, `Proxy::http()`, `Proxy::https()` APIs [CITED: WebSearch]

### Tertiary (LOW confidence)

- None — all claims verified via registered tool sources or direct codebase read.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all crate versions verified via `cargo search`
- Architecture: HIGH — patterns directly derived from Python reference + user decisions
- Pitfalls: HIGH — based on known async-openai version differences and type evolution
- Code examples: MEDIUM — vision content part types are inferred from WebSearch; precise API shape needs verification against docs.rs

**Research date:** 2026-04-28
**Valid until:** 2026-05-28 (30 days for stable crates; async-openai moves quickly)
