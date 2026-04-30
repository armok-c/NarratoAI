# Phase 2: LLM Service Layer - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-28
**Phase:** 2-LLM Service Layer
**Areas discussed:** HTTP/SDK 策略, Provider 可扩展性, 流式 SSE 解析, 重试与错误策略, Vision 批量并发控制, response_format JSON 回退, Vision 图片预处理

---

## HTTP/SDK 策略

| Option | Description | Selected |
|--------|-------------|----------|
| async-openai crate | Rust 版 OpenAI SDK，支持自定义 base_url、streaming、retry | ✓ |
| reqwest 手写 | 直接构建 HTTP 请求，完全掌控但样板代码多 | |

**User's choice:** async-openai crate
**Notes:** 最接近 Python 的 AsyncOpenAI 路径，代码量少且支持自定义 base_url

---

### Client 实例管理

| Option | Description | Selected |
|--------|-------------|----------|
| 按 provider 创建 Client 实例 | HashMap<String, Client> 缓存，每个 provider 独立配置 | ✓ |
| 单 Client + 动态覆盖 | 一个全局 Client，每次请求覆盖配置 | |
| 每个请求新建 Client | 无连接池复用，性能差 | |

**User's choice:** 按 provider 创建 Client 实例
**Notes:** 最接近 Python 的 instance_cache 模式

---

### 流式入口设计

| Option | Description | Selected |
|--------|-------------|----------|
| 两个入口：stream + non-stream | generate_text() + generate_text_stream() | ✓ |
| 统一流式入口，内部收集 | 始终走 create_stream()，非流式收集全部 token | |
| 仅非流式 | 流式留到 Phase 10 | |

**User's choice:** 两个入口：stream + non-stream
**Notes:** 功能分离，调用方按需选择

---

### Trait 设计

| Option | Description | Selected |
|--------|-------------|----------|
| 单一 LlmProvider trait | 一个 trait 包含所有方法 | ✓ |
| 分离 Vision/Text trait | 完全对齐 Python 的两个抽象类 | |
| 不要 trait，直接 struct | 无抽象，当前只有一个 provider | |

**User's choice:** 单一 LlmProvider trait
**Notes:** 底层都是同一个 /v1/chat/completions，分离是过度抽象

---

## Provider 可扩展性

| Option | Description | Selected |
|--------|-------------|----------|
| 运行时 trait object 注册 | Registry<Arc<dyn LlmProvider>>，类似 Python | ✓ |
| 编译时 enum + match | Provider 类型硬编码，零运行时开销 | |
| 混合：enum + 动态构造 | 类型安全 + 配置驱动 | |

**User's choice:** 运行时 trait object 注册
**Notes:** 对齐 Python 的动态 register 模式

---

### Provider 类型建模

| Option | Description | Selected |
|--------|-------------|----------|
| 一个 struct 多次注册（不同配置） | OpenAiCompatibleProvider 用不同 api_key/base_url 注册 | ✓ |
| 每个 provider 独立 struct | DeepSeekProvider、GeminiProvider 各自实现 | |

**User's choice:** 一个 OpenAiCompatibleProvider 多次注册
**Notes:** 所有后端都走同一个 OpenAI 兼容协议

---

### Provider 发现机制

| Option | Description | Selected |
|--------|-------------|----------|
| 显式 register_all_providers() | 库初始化时调用注册函数 | ✓ |
| 配置扫描自动发现 | 扫描 TOML 中 vision_*_api_key 字段 | |
| TOML providers 段声明 | 新增 [providers] 结构化声明 | |

**User's choice:** 显式 register_all_providers()
**Notes:** 完全对齐 Python 模式，简单直接

---

### Provider 参数

| Option | Description | Selected |
|--------|-------------|----------|
| 可选 provider + 配置默认值 | 未指定时从 config.vision_llm_provider / text_llm_provider 读取 | ✓ |
| 始终显式指定 provider | 每个调用必须带 provider 参数 | |

**User's choice:** 可选 provider + 配置默认值
**Notes:** 对齐 Python，内部流水线基本都用默认 provider

---

## 流式 SSE 解析

| Option | Description | Selected |
|--------|-------------|----------|
| Stream<Item = Result<String>> | 每个 item 是 token 文本片段（从 delta.content 提取） | ✓ |
| Stream<Item = Result<ChatCompletionChunk>> | 完整 Chunk 结构体（含 finish_reason、usage） | |

**User's choice:** impl Stream<Item = Result<String>>
**Notes:** 简化调用方，只需拼接字符串

---

### 流中途错误

| Option | Description | Selected |
|--------|-------------|----------|
| 终止流并返回错误 | Stream 遇 error 立即终止 | ✓ |
| 自动重试并断点续传 | 用相同参数重试，跳过已收到的 token | |
| 忽略错误，继续发空 token | 丢弃错误 token | |

**User's choice:** 终止流并返回错误
**Notes:** 最简实现，调用方自行处理已收到的部分 token

---

### 流式/非流式共享

| Option | Description | Selected |
|--------|-------------|----------|
| 共享 Client + 分开调用路径 | 同一个 Client，create() vs create_stream() | ✓ |
| 完全独立代码路径 | 各自 client 构建、错误处理 | |

**User's choice:** 共享 Client + 配置，分开调用路径
**Notes:** 共享错误映射、provider 解析

---

### Feature flag

| Option | Description | Selected |
|--------|-------------|----------|
| 不要 feature flag | 始终包含流式支持 | ✓ |
| 加 feature flag | 按需启用，减小体积 | |

**User's choice:** 不要 feature flag，始终包含
**Notes:** 流式和非流式共享 Client，不需要条件编译

---

## 重试与错误策略

| Option | Description | Selected |
|--------|-------------|----------|
| async-openai 内置重试 | 利用 Client 的 max_retries 配置 | ✓ |
| 业务层 backoff crate | 手动实现重试循环，精确控制 | |

**User's choice:** async-openai 内置重试
**Notes:** Python 就是这条路，最简——一行配置

---

### 错误类型粒度

| Option | Description | Selected |
|--------|-------------|----------|
| 完全对齐 Python 的 9 种 | LLMError + 8 种子类型（1:1 映射） | ✓ |
| 精简 4-5 种核心错误 | 只保留最常见的 | |
| 仅 2-3 种 | Error + Auth + Api | |

**User's choice:** 完全对齐 Python 的 9 种
**Notes:** 完整性最高，所有异常都有精确类型

---

### 默认值

| Option | Description | Selected |
|--------|-------------|----------|
| 沿用 Python 默认值 | max_retries=3, text_timeout=180s, vision_timeout=120s | ✓ |
| 更保守的默认值 | max_retries=2, text_timeout=120s, vision_timeout=90s | |

**User's choice:** 沿用 Python 的默认值
**Notes:** 与 Python 保持一致，配置字段名也不变

---

### 代理支持

| Option | Description | Selected |
|--------|-------------|----------|
| 透传 proxy 到 Client | 从 config.proxy 读取，注入 async-openai Client | ✓ |
| 留到 Phase 11 | 代理统一后续处理 | |

**User's choice:** 透传 proxy 配置到 async-openai Client
**Notes:** 对齐 Python 的 proxy 支持

---

## Vision 批量并发控制

| Option | Description | Selected |
|--------|-------------|----------|
| tokio::sync::Semaphore | 直接等价 Python 的 asyncio.Semaphore | ✓ |
| StreamExt::buffer_unordered | N 个 batch 同时启动，限制 pending 数量 | |

**User's choice:** tokio::sync::Semaphore
**Notes:** 精确控制并发数，行为可预期

---

### 默认参数

| Option | Description | Selected |
|--------|-------------|----------|
| 沿用 Python | batch_size=10, max_concurrency=1 | ✓ |
| 放宽并发 | max_concurrency=3 | |

**User's choice:** 沿用 Python 默认值
**Notes:** Python 已验证足够稳定

---

## response_format JSON 回退

| Option | Description | Selected |
|--------|-------------|----------|
| 实现同样的回退 | 先 response_format=json，失败后 prompt 约束 JSON | ✓ |
| 不做回退 | 直接返回错误给调用方 | |
| 仅 prompt 约束 | 始终在 prompt 追加 JSON 约束 | |

**User's choice:** 实现同样的回退
**Notes:** 完全对齐 Python，最大化兼容性

---

## Vision 图片预处理

| Option | Description | Selected |
|--------|-------------|----------|
| 完全对齐 Python | thumbnail 1024px + JPEG quality 85 + base64 data URL | ✓ |
| 仅 base64，不缩放 | 原始分辨率直接发送 | |

**User's choice:** 完全对齐 Python 的预处理
**Notes:** 减少 API 负载，加速 LLM 处理

---

## Claude's Discretion

None — all decisions were made by the user.

## Deferred Ideas

None — discussion stayed within phase scope
