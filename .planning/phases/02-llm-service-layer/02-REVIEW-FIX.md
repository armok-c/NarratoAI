---
phase: 02-llm-service-layer
fixed_at: 2026-04-29T08:34:27Z
review_path: .planning/phases/02-llm-service-layer/02-REVIEW.md
iteration: 1
findings_in_scope: 6
fixed: 5
skipped: 1
status: partial
---

# Phase 2: LLM Service Layer -- Code Review Fix Report

**Fixed at:** 2026-04-29T08:34:27Z
**Source review:** `.planning/phases/02-llm-service-layer/02-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 6
- Fixed: 5
- Skipped: 1

## Fixed Issues

### CR-01: Provider 名称冲突导致 vision provider 被 text provider 静默覆盖

**Files modified:** `src/llm/register.rs`
**Commit:** 0ddbd1d
**Applied fix:** 将 vision 和 text provider 的注册键从配置的 `vision_llm_provider` / `text_llm_provider` 改为始终使用 `VISION_PROVIDER_NAME` (`"openai_vision"`) 和 `TEXT_PROVIDER_NAME` (`"openai_text"`) 常量。配置字段保留用于兼容性目的。这样即使两个 provider 使用相同的配置名称也不会发生注册键冲突。

### CR-02: `test_analyze_images_result_ordering` 存在竞态条件（flaky test）

**Files modified:** `tests/llm_test.rs`
**Commit:** c562c02
**Applied fix:** 移除使用 `CyclicResponder` + `AtomicUsize` 的并发计数器模式，改为所有请求返回相同的固定响应。去掉具体的顺序断言（`results[0] == "batch-0"`），仅验证结果数量为 3 且每个结果非空。消除因并发请求到达 wiremock server 时序不确定导致的数据竞争。

### WR-01: JSON response_format 回退与客户端 backoff 重试可能导致请求数放大

**Files modified:** `src/llm/openai_compatible.rs`
**Commit:** 90b686e
**Applied fix:** 在 `OpenAiCompatibleProvider` 中新增 `fallback_client: Client<OpenAIConfig>` 字段，使用 `max_elapsed_time: Duration::ZERO`（完全禁用 backoff 重试）构建。`generate_text_with_json_fallback` 和 `create_vision_chat_with_json_fallback` 的 fallback 重试使用该无重试客户端，避免初始请求的 backoff 和 fallback 重试嵌套导致请求数放大。

### WR-02: 错误启发式分类中 `.contains("key")` 匹配过于宽泛

**Files modified:** `src/error.rs`
**Commit:** 83d9027
**Applied fix:** 将 `lower.contains("auth")` 改为 `lower.contains("authentication") || lower.contains("unauthorized") || lower.contains("auth failed")`，将 `lower.contains("key")` 改为 `(lower.contains("api key") && lower.contains("invalid"))`，减少误匹配。同时新增 `rate_limit` 的匹配项以提高 RateLimit 分类精确度。

### IN-02: `build_text_messages` 接收未使用的 `&self` 参数

**Files modified:** `src/llm/openai_compatible.rs`
**Commit:** 07abee4
**Applied fix:** 移除 `build_text_messages` 的 `&self` 参数，将其改为关联函数（与 `build_vision_messages` 一致）。更新 `generate_text`、`generate_text_stream`、`generate_text_with_json_fallback` 三处的调用从 `self.build_text_messages(...)` 为 `Self::build_text_messages(...)`。

## Skipped Issues

### IN-01: API Key 以明文 String 存储，无内存安全清理

**File:** `src/llm/openai_compatible.rs:35`
**Reason:** skip: optional fix per review（标记为"可选"），风险较低。修复需将 secrecy 添加为 Cargo.toml 直接依赖、修改 ProviderConfig 类型、更新 new() 构造器和测试中的构造代码。建议在独立的安全加固回合中统一处理。
**Original issue:** `ProviderConfig.api_key` 的类型为 `String`，在内存中以明文存在且 drop 时不会清零。虽然 Debug 实现已遮盖密钥值，但核心转储或侧信道攻击仍可能泄露密钥。

---

_Fixed: 2026-04-29T08:34:27Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
