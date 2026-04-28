# Phase 2: LLM Service Layer - Context

**Gathered:** 2026-04-28
**Status:** Ready for planning

<domain>
## Phase Boundary

实现 OpenAI 兼容协议的 Rust 异步 HTTP 客户端。通过 `provider/model_name` 格式路由请求到不同的 LLM API 端点。支持纯文本生成（非流式 + 流式 SSE）和视觉分析（base64 图片 + 文本）。含并发控制和统一错误重试机制。与 Phase 1 配置系统集成（读取 LLM 相关 TOML 段）。

覆盖需求：LLM-01, LLM-02, LLM-03, LLM-04, LLM-05, LLM-06, LLM-07

</domain>

<decisions>
## Implementation Decisions

### HTTP/SDK 策略
- **D-01:** 使用 `async-openai` crate（Rust 版 OpenAI SDK），不用 reqwest 手写。支持自定义 base_url、内置 streaming、retry。
- **D-02:** 按 provider 创建 async-openai Client 实例，`HashMap<String, Client<...>>` 缓存。每个 provider 有独立的 api_key + base_url + model_name。
- **D-03:** 两个独立入口：`generate_text()` 返回完整 `String`（非流式），`generate_text_stream()` 返回 `impl Stream<Item = Result<String>>`（流式 token 序列）。
- **D-04:** 单一 `LlmProvider` trait——包含 `generate_text()` + `generate_text_stream()` + `analyze_images()`。不分离 Vision/Text trait。底层都是 `/v1/chat/completions`，只是 message content 格式不同。

### Provider 架构
- **D-05:** 运行时 trait object 注册——`Registry` 持有 `HashMap<String, Arc<dyn LlmProvider>>`，对齐 Python 的动态 register 模式。
- **D-06:** 唯一具体 struct `OpenAiCompatibleProvider`，按不同 provider name（openai、deepseek、gemini 等）用不同配置创建多个实例并注册。
- **D-07:** 显式 `register_all_providers()` 函数在库初始化时调用，列举所有活跃 provider 并创建实例。
- **D-08:** provider 参数可选——`generate_text(prompt, provider=None)` 未指定时从 `config.app.vision_llm_provider` / `config.app.text_llm_provider` 读取默认值。

### 流式 SSE
- **D-09:** `generate_text_stream()` 返回 `impl Stream<Item = Result<String, LLMError>>`，每个 item 是从 `ChatCompletionChunk` 提取的 `delta.content` 文本片段。
- **D-10:** 流中途错误时立即终止流并返回 `Err`，不做自动重试或断点续传。
- **D-11:** 流式和非流式共享同一个 async-openai Client 实例和错误映射，但 `create()` vs `create_stream()` 分开调用路径。
- **D-12:** 流式功能始终编译包含，不放在 Cargo feature flag 后面。

### 重试与错误策略
- **D-13:** 使用 `async-openai` crate 的内置重试机制（通过 Client 的 `max_retries` 配置）。
- **D-14:** LLM 错误类型完全对齐 Python 的 9 种异常：`LLMError`(基础) + `ProviderNotFoundError` + `ConfigurationError` + `APICallError` + `ValidationError` + `ModelNotSupportedError` + `RateLimitError` + `AuthenticationError` + `ContentFilterError`。用 thiserror 派生。
- **D-15:** 默认值沿用 Python：`max_retries=3`，`text_timeout=180s`，`vision_timeout=120s`。从 TOML 配置读取。
- **D-16:** 透传 `[proxy]` 配置段的 HTTP/HTTPS 代理设置到 async-openai Client（通过底层 reqwest 的 proxy 方法）。

### Vision 批量并发控制
- **D-17:** 使用 `tokio::sync::Semaphore` 控制并发批次数，直接等价 Python 的 `asyncio.Semaphore`。
- **D-18:** 默认值沿用 Python：`batch_size=10`（每批最多 10 张图片），`max_concurrency=1`（一次只发一批）。

### response_format JSON 回退
- **D-19:** 实现与 Python 相同的回退逻辑——先尝试 `response_format={type:"json_object"}`，如果网关返回 400 且错误消息含 "response_format"，则自动重试（不带 response_format 参数，在 prompt 末尾追加 JSON 约束提示）。

### Vision 图片预处理
- **D-20:** 完全对齐 Python 的预处理流程——用 `image` crate 实现：`thumbnail(1024px)` 保持宽高比 → JPEG 编码（quality=85）→ base64 编码 → 格式化为 `data:image/jpeg;base64,...` data URL。

### Claude's Discretion
无——所有决策均由用户明确选择。

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Python 版 LLM 参考（代码级对齐）
- `app/services/llm/base.py` — VisionModelProvider / TextModelProvider 抽象基类和接口定义
- `app/services/llm/manager.py` — LLMServiceManager：工厂 + 实例缓存 + provider 注册
- `app/services/llm/openai_compatible_provider.py` — 唯一具体实现：vision 批量分析 + text 生成 + 流式（通过 AsyncOpenAI SDK）
- `app/services/llm/unified_service.py` — UnifiedLLMService 门面
- `app/services/llm/providers/__init__.py` — register_all_providers() 注册入口
- `app/services/llm/exceptions.py` — 9 种 LLM 领域异常类定义（错误类型对标模板）
- `app/config/config.py` — Python 版配置加载（LLM 配置段的字段命名约定 reference）
- `app/config/defaults.py` — normalize_openai_compatible_model_name() 和默认值逻辑

### 项目规划文档
- `.planning/PROJECT.md` — 项目约束（库 + Tauri 命令、文件存储、完全不复用 smartEdit）
- `.planning/REQUIREMENTS.md` — Phase 2 需求：LLM-01~07
- `.planning/ROADMAP.md` — Phase 2 定义和成功标准
- `.planning/phases/01-foundation/01-CONTEXT.md` — Phase 1 决策（crate 结构、tokio、tracing、thiserror、配置系统）

### 代码库映射
- `.planning/codebase/STACK.md` — Python 版依赖和 LLM provider 架构
- `.planning/codebase/ARCHITECTURE.md` — Python 版 LLM 服务架构（Provider/Registry/Manager + OpenAI 兼容协议）
- `.planning/codebase/INTEGRATIONS.md` — LLM 服务集成点和 proxy 支持

### Rust 依赖参考
- `Cargo.toml` — Phase 1 已定义的依赖（tokio、serde、tracing、thiserror、toml）

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/config/types.rs:AppSection` — LLM 配置字段已定义（vision_llm_provider、vision_openai_*、text_llm_provider、text_openai_*、llm_vision_timeout、llm_text_timeout、llm_max_retries）。Phase 2 直接读取这些字段。
- `src/config/types.rs:ProxySection` — 代理配置（proxy.http、proxy.https、proxy.enabled），透传给 async-openai Client。
- `src/error.rs` — thiserror 模式已建立，Phase 2 新增 `LLMError` enum 遵循相同约定（中文用户消息）。

### Established Patterns
- **模块级结构**：`src/llm/mod.rs` + 子模块。Phase 1 已有 `src/config/`、`src/ffmpeg/`、`src/error.rs` 作为结构参考。
- **thiserror 领域错误**：Phase 1 的 `ConfigError` 和 `FFmpegError` 使用 `#[derive(Error, Debug)]` + 中文 `#[error("...")]` 消息。LLMError 对齐此模式。
- **Arc<RwLock<AppConfig>>**：配置并发访问模式，Provider Registry 读取配置创建 Client 实例。
- **tokio::task::spawn_blocking**：Phase 1 用此模式包装 FFmpeg CLI。Phase 2 的 HTTP 调用是 async native，不需要 spawn_blocking。

### Integration Points
- `src/lib.rs` — 新增 `pub mod llm;`
- `src/config/types.rs` — LLM 配置段在此定义，Phase 2 可能需要扩展多个 provider 的配置字段
- `src/error.rs` — 新增 `LLMError` enum
- `Cargo.toml` — 新增 `async-openai`、`image`、`base64`、`tokio::sync::Semaphore`（tokio 已有）

</code_context>

<specifics>
## Specific Ideas

- 用户明确要求运行时 trait object 注册 + 显式 register_all_providers()，对齐 Python 的 Provider/Registry 模式
- 错误类型必须完全对齐 Python 的 9 种异常（1:1 映射）
- 图片预处理完全对齐 Python（thumbnail 1024px + JPEG quality 85 + base64 data URL）
- response_format JSON 回退逻辑必须实现（与 Python 行为一致）
- 代理支持必须透传到 async-openai Client

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 2-LLM Service Layer*
*Context gathered: 2026-04-28*
