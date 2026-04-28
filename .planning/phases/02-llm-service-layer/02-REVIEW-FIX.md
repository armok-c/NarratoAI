---
phase: 02-llm-service-layer
fixed_at: "2026-04-28T22:45:00+08:00"
review_path: .planning/phases/02-llm-service-layer/02-REVIEW.md
fixed_from_branch: fix-02-review-197809
findings_in_scope: 9
fixed: 9
skipped: 0
fix_scope: critical_warning
iteration: 1
status: all_fixed
---

# Phase 02: LLM Service Layer -- Code Review Fix Report

**Fixed at:** 2026-04-28T22:45:00+08:00
**Source review:** `.planning/phases/02-llm-service-layer/02-REVIEW.md`
**Fix scope:** critical + warning (9 findings)
**Iteration:** 1

## Summary

| Metric | Count |
|--------|-------|
| Findings in scope | 9 (3 critical, 6 warning) |
| Fixed | 9 |
| Skipped | 0 |

## Fixed Issues

### CR-01: Missing `#[derive(Default)]` on 8 config section structs

**Files modified:** `src/config/types.rs`
**Commit:** `2e167fb`
**Applied fix:** Added `#[derive(Default)]` to 8 structs: `AppSection`, `UiSection`, `TencentSection`, `SoulVoiceSection`, `TtsQwenSection`, `IndexTTS2Section`, `DoubaoTTSSection`, and `FramesSection`. All contain only primitive types that already implement `Default`, making this safe.

### CR-02: `reqwest = "0.13"` incompatible with async-openai's reqwest 0.12.x

**Files modified:** `Cargo.toml`
**Commit:** `db778ce`
**Applied fix:** Changed `reqwest = "0.13"` to `reqwest = "0.12"` to match async-openai 0.36.x's dependency. This avoids duplicate reqwest versions in the dependency tree and prevents type mismatch errors between `reqwest::Client` from different semver-incompatible versions.

### CR-03: `test_json_response_format_fallback` never triggers the fallback path

**Files modified:** `tests/llm_test.rs`
**Commit:** `7e19f10`
**Applied fix:** Restructured test to use a single mock server with body-content-based differentiation. The first request (containing `response_format` in body) returns 400, triggering the `generate_text_with_json_fallback` method. The retry (without `response_format`, with appended JSON instruction) returns 200. Added `LlmResponseFormat` import and `body_string_contains` wiremock matcher import. The test now actually exercises the fallback code path.

### WR-01: `From<OpenAIError>` maps all errors lossily

**Files modified:** `src/error.rs`
**Commit:** `9870190`
**Applied fix:** Replaced blanket `LLMError::APICall(err.to_string())` conversion with match on `OpenAIError::ApiError.status_code`: `Some(401)` maps to `LLMError::Authentication`, `Some(429)` maps to `LLMError::RateLimit`, and all other cases fall through to `LLMError::APICall`. This makes the `Authentication` and `RateLimit` variants reachable and eliminates dead code.

### WR-02: Provider registration failures silently swallowed

**Files modified:** `src/llm/register.rs`
**Commit:** `7937543`
**Applied fix:** Changed both vision and text provider registration from `if let Ok(provider) = ... { } else { tracing::warn!(...) }` to `match OpenAiCompatibleProvider::new(...) { Ok(provider) => ..., Err(e) => tracing::error!("...: {}", e) }`. This preserves the error details from provider construction failures and upgrades the log severity from `warn!` to `error!`.

### WR-03: `api_key` and `base_url` retained in memory with `#[allow(dead_code)]`

**Files modified:** `src/llm/openai_compatible.rs`
**Commit:** `6b21783`
**Applied fix:** Removed `api_key` and `base_url` fields from `OpenAiCompatibleProvider` struct, along with the `#[allow(dead_code)]` annotations. The API key is passed to `OpenAIConfig::with_api_key()` during construction and never read again. Storing it in the struct increased memory disclosure attack surface for no benefit.

### WR-04: `extract_text` silently returns empty string for null/missing content

**Files modified:** `src/llm/openai_compatible.rs`
**Commit:** `680834d`
**Applied fix:** Changed `extract_text` return type from `String` to `Result<String, LLMError>`. Replaced `.unwrap_or("")` with `.ok_or_else(|| LLMError::APICall(...))`. Updated the single call site in `generate_text` to return the error directly. Callers now get a proper error instead of silently receiving an empty string when the API returns an unexpected response structure.

### WR-05: Image processing errors misclassified as `LLMError::Configuration`

**Files modified:** `src/llm/image_utils.rs`
**Commit:** `6ad48a3`
**Applied fix:** Changed both `image::open()` failure and JPEG encoding failure from `LLMError::Configuration(...)` to `LLMError::General(...)`. These are runtime input/data errors (missing file, corrupt image, unsupported format), not configuration problems. Using `Configuration` was misleading during debugging as it directed users to check `config.toml`.

### WR-06: `test_openai_error_mapping` permanently `#[ignore]` and `max_retries` not threaded

**Files modified:** `src/llm/openai_compatible.rs`, `tests/llm_test.rs`
**Commit:** `3dace1a`
**Applied fix:** Created `OpenAIClientConfig` with `RetryConfig { max_retries: _max_retries, .. }` and passed it to both `Client::build()` calls instead of `Default::default()`. Removed `#[ignore]` from the error mapping test since max_retries=0 now allows the test to complete quickly without waiting for exponential backoff. Added `OpenAIClientConfig` and `RetryConfig` imports.

---

_Fixed: 2026-04-28T22:45:00+08:00_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
