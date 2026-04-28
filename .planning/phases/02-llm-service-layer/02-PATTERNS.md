# Phase 2: LLM Service Layer - Pattern Map

**Mapped:** 2026-04-28
**Files analyzed:** 14 (10 new, 4 modified)
**Analogs found:** 13 / 14

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/llm/mod.rs` | config | N/A — module decl | `src/ffmpeg/mod.rs` | exact |
| `src/llm/provider.rs` | model | request-response | `src/ffmpeg/hwaccel.rs` (struct), Python `app/services/llm/base.py` (trait contract) | role-match |
| `src/llm/registry.rs` | service | request-response | `src/config/mod.rs` (cache+read pattern), Python `app/services/llm/manager.py` (register+lookup) | role-match |
| `src/llm/openai_compatible.rs` | service | request-response + streaming | `src/ffmpeg/command.rs` (async+error), Python `app/services/llm/openai_compatible_provider.py` (logic) | role-match |
| `src/llm/error.rs` | utility | N/A — error type | `src/error.rs` | exact |
| `src/llm/types.rs` | model | N/A — data types | `src/ffmpeg/hwaccel.rs` (struct+enum) | role-match |
| `src/llm/image_utils.rs` | utility | file-I/O | Python `app/services/llm/openai_compatible_provider.py` (image pipeline) | partial-match |
| `src/llm/register.rs` | utility | N/A — init | Python `app/services/llm/providers/__init__.py`, `src/config/mod.rs` (init pattern) | role-match |
| `tests/llm_test.rs` | test | N/A | `tests/config_test.rs` | exact |
| `src/llm/test_utils.rs` | test-utility | N/A | `tests/config_test.rs`, `tests/ffmpeg_test.rs` (helper fn pattern) | role-match |
| `src/lib.rs` | config | N/A — module decl | `src/lib.rs` (existing) | exact — modification only |
| `src/error.rs` | utility | N/A — error type | `src/error.rs` (existing) | exact — modification only |
| `Cargo.toml` | config | N/A | `Cargo.toml` (existing) | exact — modification only |
| `src/config/types.rs` | config | N/A | `src/config/types.rs` (existing) | exact — modification only |

## Pattern Assignments

### `src/llm/mod.rs` (config, module-declaration)

**Analog:** `src/ffmpeg/mod.rs`

**Imports pattern** (lines 1-3):
```rust
pub mod command;
pub mod probe;
pub mod hwaccel;
```

**Core pattern** (same file, line 1-3):
Module declaration exports submodules. `src/llm/mod.rs` should follow same pattern:
```rust
pub mod provider;
pub mod registry;
pub mod openai_compatible;
pub mod error;
pub mod types;
pub mod image_utils;
pub mod register;
```

---

### `src/llm/provider.rs` (model, trait-definition, request-response)

**Analog:** `src/ffmpeg/hwaccel.rs` (struct pattern), Python `app/services/llm/base.py` (trait contract)

**Rust struct pattern** (`src/ffmpeg/hwaccel.rs`, lines 7-19):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HwAccelProfile {
    HighPerformance,
    Compatibility,
    WindowsNvidia,
    MacosVideotoolbox,
    UniversalSoftware,
}
```

**Trait design** (Research.md lines 234-265, derived from Python `base.py`):
```rust
// LlmProvider trait — single trait, 3 methods, no separation of Vision/Text
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
```

**Error handling pattern** (from `src/error.rs`, thiserror convention, lines 4-20):
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SomeError {
    #[error("中文描述: {0}")]
    VariantName(String),
    // ...
}
```

---

### `src/llm/registry.rs` (service, request-response)

**Analog:** `src/config/mod.rs` (cache pattern), Python `app/services/llm/manager.py` (register+lookup logic)

**Cache pattern** (`src/config/mod.rs`, lines 5-11, 60-67):
```rust
use std::sync::Arc;
use crate::config::types::AppConfig;

pub struct ConfigManager {
    config: Arc<RwLock<AppConfig>>,
    // ...
}

pub fn config(&self) -> Arc<RwLock<AppConfig>> {
    Arc::clone(&self.config)
}
```

**Registry lookup pattern** (Research.md lines 267-286):
```rust
use std::collections::HashMap;

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

---

### `src/llm/openai_compatible.rs` (service, request-response + streaming)

**Analog:** `src/ffmpeg/command.rs` (async+spawn_blocking pattern), Python `app/services/llm/openai_compatible_provider.py` (business logic)

**Async method pattern** (`src/ffmpeg/command.rs`, lines 48-125):
```rust
pub async fn clip_video(
    input: &Path,
    output: &Path,
    start: f64,
    duration: f64,
    progress: Option<ProgressCallback>,
) -> Result<(), FFmpegError> {
    let input_path = input.to_string_lossy().to_string();
    let output_path = output.to_string_lossy().to_string();
    let progress = progress.map(Arc::new);
    // ... async body with tokio::task::spawn_blocking
}
```

**Streaming pattern** (Research.md lines 297-321):
```rust
use futures::StreamExt;

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

**Error mapping pattern** (Python `openai_compatible_provider.py`, lines 159-175, to be ported):
```python
try:
    response = await client.chat.completions.create(...)
except OpenAIAuthError as exc:
    raise AuthenticationError(str(exc))
except OpenAIRateLimitError as exc:
    raise RateLimitError(str(exc))
except OpenAIBadRequestError as exc:
    if _is_response_format_error(str(exc)):
        # JSON fallback retry logic
        ...
    raise APICallError(...)
except OpenAIAPIError as exc:
    raise APICallError(...)
```

**JSON response_format fallback** (Research.md lines 509-553):
```rust
async fn generate_text_with_json_fallback(
    client: &Client,
    mut request: CreateChatCompletionRequest,
    prompt: &str,
) -> Result<String, LLMError> {
    request.response_format = Some(ResponseFormat { type_: "json_object" });

    match client.chat().create(request.clone()).await {
        Ok(response) => Ok(extract_text(&response)),
        Err(OpenAIError::ApiError(api_err))
            if api_err.status_code == Some(400)
               && api_err.message.to_lowercase().contains("response_format") =>
        {
            request.response_format = None;
            // append JSON constraint to prompt
            ...
            let retry_response = client.chat().create(request).await?;
            Ok(extract_text(&retry_response))
        }
        Err(e) => Err(LLMError::from(e)),
    }
}
```

**Client creation with proxy** (Research.md lines 440-474):
```rust
use async_openai::{Client, config::OpenAIConfig};
use reqwest::{Proxy, Client as HttpClient};

pub fn build_client(
    api_key: &str,
    base_url: &str,
    timeout_secs: u64,
    max_retries: u32,
    proxy_config: &ProxySection,
) -> Client {
    let config = OpenAIConfig::new()
        .with_api_key(api_key)
        .with_api_base(base_url.trim_end_matches('/'));

    if proxy_config.enabled {
        let mut http_client_builder = HttpClient::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs));
        // ... proxy setup ...
        let http_client = http_client_builder.build().expect("构建 reqwest Client 失败");
        Client::with_config(config).with_http_client(http_client)
    } else {
        Client::with_config(config)
    }
}
```

**Vision batch + Semaphore** (Python `openai_compatible_provider.py` lines 95-128):
```python
bounded_concurrency = max(1, int(max_concurrency))
semaphore = asyncio.Semaphore(bounded_concurrency)
batches = [
    (index // batch_size, processed_images[index : index + batch_size])
    for index in range(0, len(processed_images), batch_size)
]

async def run_batch(batch_index, batch):
    async with semaphore:
        result = await self._analyze_batch(batch, prompt)
        return batch_index, result

completed = await asyncio.gather(*(run_batch(i, b) for i, b in batches))
completed.sort(key=lambda item: item[0])
```
Port to Rust with `tokio::sync::Semaphore`:
```rust
use tokio::sync::Semaphore;
let semaphore = Arc::new(Semaphore::new(max_concurrency));
let mut handles = vec![];
for (index, batch) in batches.iter().enumerate() {
    let permit = semaphore.clone().acquire_owned().await?;
    handles.push(tokio::spawn(async move {
        let _permit = permit;
        analyze_batch(batch).await
    }));
}
```

---

### `src/llm/error.rs` (utility, error-type)

**Analog:** `src/error.rs` (exact match — same pattern)

**thiserror enum pattern** (`src/error.rs`, lines 1-46):
```rust
use thiserror::Error;

/// [领域]错误
#[derive(Error, Debug)]
pub enum SomeDomainError {
    #[error("中文描述: {0}")]
    VariantA(String),

    #[error("中文描述: {0}")]
    VariantB(String),
    // ...
}

/// From trait conversion for external error types
impl From<ExternalError> for SomeDomainError {
    fn from(err: ExternalError) -> Self {
        SomeDomainError::Variant(err.to_string())
    }
}
```

**Test pattern** (`src/error.rs`, lines 48-121):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_some_error_message_chinese() {
        let err = SomeDomainError::Variant("test".into());
        let msg = err.to_string();
        assert!(msg.contains("中文描述"), "消息应包含中文: {}", msg);
    }
}
```

**LLMError should define 9 variants** (from Python `app/services/llm/exceptions.py`):
```rust
#[derive(Error, Debug)]
pub enum LLMError {
    #[error("LLM 调用失败: {0}")]
    General(String),                      // LLMError base

    #[error("未找到大模型供应商: {0}")]
    ProviderNotFound(String),             // ProviderNotFoundError

    #[error("配置错误: {0}")]
    Configuration(String),                // ConfigurationError

    #[error("API 调用失败: {0}")]
    APICall(String),                      // APICallError

    #[error("输出验证失败: {0}")]
    Validation(String),                   // ValidationError

    #[error("不支持的模型: {0}")]
    ModelNotSupported(String),            // ModelNotSupportedError

    #[error("API 速率限制: {0}")]
    RateLimit(String),                    // RateLimitError

    #[error("API 认证失败: {0}")]
    Authentication(String),               // AuthenticationError

    #[error("内容被安全过滤器阻止: {0}")]
    ContentFilter(String),                // ContentFilterError
}
```

---

### `src/llm/types.rs` (model, data-types)

**Analog:** `src/ffmpeg/hwaccel.rs` (struct+enum pattern)

**Struct pattern** (`src/ffmpeg/hwaccel.rs`, lines 22-33):
```rust
#[derive(Debug, Clone)]
pub struct FFmpegProfileInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub hwaccel_enabled: bool,
    pub hwaccel_type: Option<&'static str>,
    pub encoder: &'static str,
    pub quality_preset: &'static str,
    pub pixel_format: &'static str,
    pub additional_args: &'static [&'static str],
    pub compatibility_level: u8,
}
```

Expected `src/llm/types.rs` pattern:
```rust
use async_openai::types::ResponseFormat;

/// ResponseFormat option for LLM requests
#[derive(Debug, Clone)]
pub enum LlmResponseFormat {
    Text,
    Json,
}

/// Configuration for vision batch processing
#[derive(Debug, Clone)]
pub struct VisionBatchConfig {
    pub batch_size: usize,
    pub max_concurrency: usize,
}

impl Default for VisionBatchConfig {
    fn default() -> Self {
        Self { batch_size: 10, max_concurrency: 1 }
    }
}
```

---

### `src/llm/image_utils.rs` (utility, file-I/O)

**Analog:** Python `app/services/llm/openai_compatible_provider.py` (image pipeline)

**Image preprocessing** (Research.md lines 479-503):
```rust
use image::{DynamicImage, imageops::FilterType};
use std::path::Path;

/// 加载图片，缩放到 1024px 保持宽高比，编码为 JPEG quality 85，
/// 然后 base64 编码为 data URL。完全对齐 Python 版。
pub fn image_to_base64_data_url(path: &Path) -> Result<String, LLMError> {
    let img = image::open(path)
        .map_err(|e| LLMError::Configuration(format!("图片加载失败: {}", e)))?;

    // thumbnail(1024, 1024) 保持宽高比
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

**Python reference** (`openai_compatible_provider.py`, lines 177-180):
```python
def _image_to_base64(self, img: PIL.Image.Image) -> str:
    img_buffer = io.BytesIO()
    img.save(img_buffer, format="JPEG", quality=85)
    return base64.b64encode(img_buffer.getvalue()).decode("utf-8")
```

---

### `src/llm/register.rs` (utility, initialization)

**Analog:** Python `app/services/llm/providers/__init__.py`, `src/config/mod.rs` (init pattern)

**Python register pattern** (`providers/__init__.py`, lines 11-33):
```python
def register_all_providers():
    from ..manager import LLMServiceManager

    from ..openai_compatible_provider import (
        OpenAICompatibleVisionProvider,
        OpenAICompatibleTextProvider,
    )

    LLMServiceManager.register_vision_provider('openai', OpenAICompatibleVisionProvider)
    LLMServiceManager.register_text_provider('openai', OpenAICompatibleTextProvider)
```

Expected `src/llm/register.rs` pattern:
```rust
use std::sync::Arc;
use crate::config::types::AppConfig;
use crate::llm::registry::Registry;
use crate::llm::openai_compatible::OpenAiCompatibleProvider;

/// 注册所有活跃 provider。在库初始化时调用。
/// 从配置读取每个 provider 的 api_key / base_url / model_name，
/// 创建 OpenAiCompatibleProvider 实例并注册到 Registry。
pub fn register_all_providers(config: &AppConfig, registry: &mut Registry) {
    // Register vision provider(s)
    if !config.app.vision_openai_api_key.is_empty() {
        let provider = OpenAiCompatibleProvider::new(
            &config.app.vision_openai_api_key,
            &config.app.vision_openai_model_name,
            &config.app.vision_openai_base_url,
        );
        registry.register("openai_vision", Arc::new(provider));
    }

    // Register text provider(s)
    if !config.app.text_openai_api_key.is_empty() {
        let provider = OpenAiCompatibleProvider::new(
            &config.app.text_openai_api_key,
            &config.app.text_openai_model_name,
            &config.app.text_openai_base_url,
        );
        registry.register("openai_text", Arc::new(provider));
    }
}
```

---

### `tests/llm_test.rs` (test, N/A)

**Analog:** `tests/config_test.rs` (exact match — integration test file)

**Integration test pattern** (`tests/config_test.rs`, lines 1-8, 10-60):
```rust
use std::path::Path;
use std::time::Duration;

use narratoai_core::config::ConfigManager;
use narratoai_core::config::types::AppConfig;
use narratoai_core::error::ConfigError;
use tempfile::TempDir;

#[test]
fn test_load_example_config() {
    // Arrange
    let config_path = ...;
    // Act
    let config_manager = ConfigManager::load(&config_path).expect("...");
    // Assert
    assert_eq!(config.app.project_version, "0.7.8");
}
```

**Test helper fn pattern** (`tests/ffmpeg_test.rs`, lines 14-35):
```rust
fn find_ffmpeg() -> PathBuf {
    if let Ok(output) = Command::new("ffmpeg").arg("-version").output() {
        if output.status.success() {
            return PathBuf::from("ffmpeg");
        }
    }
    let cached = ffmpeg_sidecar::paths::ffmpeg_path();
    if cached.exists() {
        return cached;
    }
    panic!("FFmpeg 不可用。请安装 FFmpeg...");
}
```

---

### `src/lib.rs` (config, module-decl — MODIFICATION)

**Analog:** Existing `src/lib.rs` (lines 1-3):
```rust
pub mod config;
pub mod ffmpeg;
pub mod error;
```

Add new line:
```rust
pub mod llm;
```

---

### `src/error.rs` (utility, error-type — MODIFICATION)

**Analog:** Existing `src/error.rs` (lines 1-46), add `LLMError` enum at the end:
```rust
/// LLM 服务领域错误
#[derive(Error, Debug)]
pub enum LLMError {
    #[error("LLM 调用失败: {0}")]
    General(String),

    #[error("未找到大模型供应商: {0}")]
    ProviderNotFound(String),

    #[error("配置错误: {0}")]
    Configuration(String),

    #[error("API 调用失败: {0}")]
    APICall(String),

    #[error("输出验证失败: {0}")]
    Validation(String),

    #[error("不支持的模型: {0}")]
    ModelNotSupported(String),

    #[error("API 速率限制: {0}")]
    RateLimit(String),

    #[error("API 认证失败: {0}")]
    Authentication(String),

    #[error("内容被安全过滤器阻止: {0}")]
    ContentFilter(String),
}

/// From<async_openai::error::OpenAIError> for LLMError
impl From<async_openai::error::OpenAIError> for LLMError {
    fn from(err: async_openai::error::OpenAIError) -> Self {
        // Map OpenAIError variants to appropriate LLMError variants
        LLMError::APICall(err.to_string())
    }
}
```

---

### `Cargo.toml` (config — MODIFICATION)

**Analog:** Existing `Cargo.toml` (lines 6-18), add:
```toml
async-openai = { version = "0.36", features = ["chat-completion"] }
image = "0.25"
base64 = "0.22"
futures = "0.3"
reqwest = "0.12"
```

**Pattern** (existing convention, lines 7-9):
```toml
tokio = { version = "1.52.1", features = ["full"] }
serde = { version = "1.0.228", features = ["derive"] }
```

---

### `src/config/types.rs` (config — MODIFICATION)

**Analog:** Existing `src/config/types.rs`, lines 29-59 (`AppSection`).

Currently `AppSection` has single vision/text provider fields. Phase 2 uses these fields directly, so minimal modification needed. The Rust `AppSection` already matches the TOML structure for `vision_llm_provider`, `text_llm_provider`, etc.

If multi-provider support is needed, pattern would follow the existing section struct pattern (e.g., `ProxySection`, lines 182-191):
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ProviderConfig {
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub model_name: String,
    #[serde(default)]
    pub base_url: String,
}
```

## Shared Patterns

### Domain Error (thiserror)
**Source:** `src/error.rs` (entire file, lines 1-46)
**Apply to:** `src/llm/error.rs`, `src/error.rs` (LLMError addition)

Pattern: `#[derive(Error, Debug)]` enum with Chinese `#[error("...")]` messages.
See `ConfigError` (lines 4-20) and `FFmpegError` (lines 22-39) for exact conventions.

### Module Structure
**Source:** `src/ffmpeg/mod.rs` (line 1-3), `src/config/mod.rs` (lines 1-3)
**Apply to:** `src/llm/mod.rs`

Declare submodules at top via `pub mod submodule;` lines, one per file.

### Init/Registration
**Source:** `src/config/mod.rs` (lines 15-68, `ConfigManager::load`)
**Apply to:** `src/llm/register.rs`

Use `from`-style constructor pattern: `fn load(path: &Path) -> Result<Self, Error>` or the functional `register_all_providers(config, registry)` approach.

### Async + Error Handling
**Source:** `src/ffmpeg/command.rs` (lines 48-125, `clip_video`)
**Apply to:** `src/llm/openai_compatible.rs`

```rust
pub async fn some_operation(...) -> Result<ReturnType, DomainError> {
    // 1. Clone/copy inputs before async block
    // 2. Call async operation
    // 3. Map errors with ? operator or match
    // 4. Return Ok(result)
}
```

### Integration Test Structure
**Source:** `tests/config_test.rs` (entire file), `tests/ffmpeg_test.rs` (entire file)
**Apply to:** `tests/llm_test.rs`

- Use `use narratoai_core::module::...;` imports
- `#[test]` for sync tests, `#[tokio::test]` for async tests
- Use `tempfile::TempDir` for file I/O tests
- Pattern: Arrange -> Act -> Assert with Chinese assertion messages
- Skip integration tests that require real API keys (mark `#[ignore]`)

### Error Type Test Pattern
**Source:** `src/error.rs` (lines 48-121)
**Apply to:** `src/llm/error.rs`, `src/error.rs` (LLMError addition)

```rust
#[test]
fn test_llm_error_message_chinese() {
    let err = LLMError::General("test".into());
    let msg = err.to_string();
    assert!(msg.contains("LLM 调用失败"), "消息应包含中文: {}", msg);
}
```

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `src/llm/image_utils.rs` | utility | file-I/O | No existing Rust file has image processing pipeline. Python `openai_compatible_provider.py` lines 177-180 + RESEARCH.md code examples provide the pattern. |

## Metadata

**Analog search scope:** `src/` (all Rust source), `tests/` (integration tests), `app/services/llm/` (Python reference)
**Files scanned:** 14 Rust files, 5 Python LLM reference files
**Pattern extraction date:** 2026-04-28
