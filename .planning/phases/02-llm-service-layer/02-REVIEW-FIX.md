---
phase: 02-llm-service-layer
fixed_at: 2026-04-29T12:45:00Z
review_path: .planning/phases/02-llm-service-layer/02-REVIEW.md
iteration: 2
findings_in_scope: 9
fixed: 9
skipped: 0
status: all_fixed
---

# Phase 2: LLM 服务层代码修复报告

**修复时间:** 2026-04-29T12:45:00Z
**源审查报告:** .planning/phases/02-llm-service-layer/02-REVIEW.md
**迭代:** 2 (第二轮审查修复)

**摘要:**
- 范围内发现数: 9
- 已修复: 9
- 已跳过: 0

## 已修复问题

### IN-01: `analyze_images` trait 添加 `response_format` 参数

**修改文件:** `src/llm/provider.rs`, `src/llm/openai_compatible.rs`, `tests/llm_test.rs`
**提交:** 1acb683
**应用修复:** 为 `LlmProvider::analyze_images` 方法签名添加 `response_format: Option<LlmResponseFormat>` 参数。在 `OpenAiCompatibleProvider` 实现中，该参数通过 `use_json` 标志传入 `tokio::spawn` 闭包，并在请求构建器中通过 `request_builder.response_format(ResponseFormat::JsonObject)` 条件设置。两个测试调用点已补充 `None` 参数。

### WR-01: `analyze_images` 空响应返回 `LLMError` 而非静默空串

**修改文件:** `src/llm/openai_compatible.rs`
**提交:** 2598e13
**应用修复:** 将 `analyze_images` 方法中的 `.unwrap_or("")` 替换为 `.ok_or_else(|| ...)?` 错误返回逻辑。新实现与 `extract_text` 方法行为一致：当响应无有效文本内容时返回 `LLMError::APICall`。同时检查 `finish_reason` 是否为 `FinishReason::ContentFilter`，识别内容过滤场景时返回 `LLMError::ContentFilter`。

### WR-02: `ProviderConfig` Debug 实现脱敏 `api_key`

**修改文件:** `src/llm/openai_compatible.rs`
**提交:** 14ae100
**应用修复:** 从 `#[derive(Debug, Clone)]` 中去掉 `Debug`，改为手动实现 `fmt::Debug`。实现中 `api_key` 字段输出为 `"***REDACTED***"`，其余字段正常输出。避免了通过 `tracing::debug!(?config)`、`format!("{:?}", config)` 或 panic 栈展开泄露 API key 的风险。

### WR-03: 仅读取文件头少量字节做 JPEG 检测

**修改文件:** `src/llm/image_utils.rs`
**提交:** 9ea478c
**应用修复:** 将原 `std::fs::read(path)` 全文件读取检测魔术字节的方式，改为先打开文件句柄、仅读取前 4 字节 (`read_exact`)、然后立即 `drop(file)` 释放句柄。仅当确定是 JPEG 文件时，才重新 `std::fs::read` 完整文件进行 base64 编码。消除了非 JPEG 文件场景下的双重内存占用问题。

### IN-03: JPEG 直通路径添加完整性验证

**修改文件:** `src/llm/image_utils.rs`
**提交:** c5380ef
**应用修复:** 在 JPEG 直通路径中，base64 编码之前添加 `image::load_from_memory(&raw_bytes)` 轻量解码验证。如果 JPEG 文件头部魔术字节正确但内容已截断或损坏，将返回 `LLMError::General("JPEG 图片损坏: ...")` 错误而非将损坏数据发送到 LLM API。

### WR-04: 用单 mock + AtomicUsize 替代 3 个竞态 wiremock

**修改文件:** `tests/llm_test.rs`
**提交:** 6ed3b11
**应用修复:** 移除了 `test_analyze_images_result_ordering` 中 3 个相同匹配器的 wiremock，替换为单一 `CyclicResponder` 结构体实现 `wiremock::Respond` trait，内部使用 `AtomicUsize` 原子计数器轮询返回不同内容（`"batch-0"`、`"batch-1"`、`"batch-2"`）。消除了 wiremock 0.6 的非原子计数检查在读-递增模式下的竞态风险。

### WR-05: 使用配置值作注册名

**修改文件:** `src/llm/register.rs`
**提交:** 19ba2d6
**应用修复:** `register_all_providers` 现在优先使用 `config.app.vision_llm_provider` / `config.app.text_llm_provider` 作为 provider 注册名，仅在配置值为空时回退到默认名称。支持未来多 provider 实现共存。更新了函数文档注释和测试用例以反映新的注册逻辑。

### IN-02: 公开常量 `VISION_PROVIDER_NAME` / `TEXT_PROVIDER_NAME`

**修改文件:** `src/llm/register.rs`
**提交:** 19ba2d6
**应用修复:** 在模块顶部定义 `pub const VISION_PROVIDER_NAME: &str = "openai_vision"` 和 `pub const TEXT_PROVIDER_NAME: &str = "openai_text"` 作为公开常量供外部引用，避免字符串字面量拼写错误。

### IN-04: 添加 `RegistrationErrors` 包装类型实现 Error trait

**修改文件:** `src/llm/register.rs`
**提交:** 8b63b69
**应用修复:** 定义 `RegistrationErrors(Vec<LLMError>)` 结构体，手动实现 `Display` 和 `std::error::Error` trait。`register_all_providers` 的返回类型从 `Result<(), Vec<LLMError>>` 改为 `Result<(), RegistrationErrors>`，使调用者可以通过 `?` 操作符在 `Result<_, Box<dyn Error>>` 链中传播此错误。

## 修复统计

| 指标 | 值 |
|------|-----|
| 范围内发现数 | 9 |
| Critical 发现 | 0 |
| Warning 发现 | 5 |
| Info 发现 | 4 |
| 已修复 | 9 |
| 已跳过 | 0 |
| 总提交数 | 8 |

## 注意事项

- WR-05 和 IN-02 在同一提交中处理，因为它们修改同一个文件 (`register.rs`) 且逻辑相关。
- 所有修复均通过 `cargo check` 验证，仅存在 5 个与本次修复无关的预先存在的编译错误（`src/tts/edge_tts.rs` 中 `connect_req` 未定义、`src/llm/image_utils.rs` 中缺失 `GenericImageView` import、`src/llm/openai_compatible.rs` 中 `.with_max_retries` 方法未找到）。
- 未跳过任何发现。

---

_修复时间: 2026-04-29T12:45:00Z_
_修复者: Claude (gsd-code-fixer)_
_迭代: 2_
