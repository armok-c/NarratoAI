---
phase: 02-llm-service-layer
fixed_at: 2026-04-29T11:30:00Z
review_path: .planning/phases/02-llm-service-layer/02-REVIEW.md
iteration: 1
findings_in_scope: 10
fixed: 9
skipped: 1
status: all_fixed
---

# Phase 02: LLM Service Layer -- Code Review Fix Report

**Fixed at:** 2026-04-29T11:30:00Z
**Source review:** .planning/phases/02-llm-service-layer/02-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 10
- Fixed: 9
- Skipped: 1

## Fixed Issues

### CR-01: Missing `use base64::Engine` import causes compilation failure

**Files modified:** `tests/llm_test.rs`
**Commit:** 0ec7ba5
**Applied fix:** Added `use base64::Engine;` import and replaced fully-qualified `base64::Engine::decode(...)` call with method syntax `base64::engine::general_purpose::STANDARD.decode(b64_part)`.

### CR-02: `max_retries` constructor parameter silently ignored

**Files modified:** `src/llm/openai_compatible.rs`
**Commit:** 49d7ac4
**Applied fix:** Removed the `let _ = max_retries;` suppressor line and wired the parameter to `.with_max_retries(max_retries)` on `OpenAIConfig`. Updated doc comment to reflect active usage.

### WR-01: Image thumbnail algorithm does not match claimed LANCZOS quality

**Files modified:** `src/llm/image_utils.rs`
**Commit:** 713545c
**Applied fix:** Replaced `image::DynamicImage::thumbnail()` (nearest-neighbor/pixel-averaging) with `resize()` using `image::imageops::FilterType::Lanczos3`, matching PIL's `LANCZOS` quality. Updated pipeline comment accordingly.

### WR-02: Fragile JSON round-trip in `response_format` fallback

**Files modified:** `src/llm/openai_compatible.rs`
**Commit:** 6eedbed
**Applied fix:** Replaced `serde_json` round-trip manipulation with fresh request construction via `CreateChatCompletionRequestArgs` builder. The function now accepts `system_prompt`, `temperature`, and `max_tokens` parameters to rebuild the request cleanly. This also eliminates the unnecessary `request.clone()` on the success path (addressing IN-04).

### WR-03: Duplicate test helper functions (`write_test_jpeg` / `create_test_jpeg_path`)

**Files modified:** `tests/llm_test.rs`
**Commit:** 152665b
**Applied fix:** Removed duplicate `write_test_jpeg` and `create_test_jpeg_path` functions from integration tests. Added import from shared `narratoai_core::llm::test_utils`.

### WR-04: Dual TLS backends increase binary size and build time

**Files modified:** `Cargo.toml`
**Commit:** 378fd9a
**Applied fix:** Changed `tokio-tungstenite` feature from `native-tls` to `rustls-tls` to align with `reqwest`'s TLS backend. The direct `native-tls = "0.2"` dependency is kept because it is used by `src/tts/edge_tts.rs`.

### IN-01: Dead code -- `mock_chat_response` function never called

**Files modified:** `src/llm/test_utils.rs`
**Commit:** 71187fe
**Applied fix:** Removed the unused `mock_chat_response` function which also constructed deprecated fields (`function_call`, `system_fingerprint`).

### IN-02: Provider registration failures silently swallowed

**Files modified:** `src/llm/register.rs`
**Commit:** d5083c2
**Applied fix:** Changed `register_all_providers` return type from `()` to `Result<(), Vec<LLMError>>`. Errors are still logged but now also accumulated and returned, allowing callers to detect partial registration failures at init time.

### IN-03: Client-level timeouts may not be detected as timeouts

**Files modified:** `src/error.rs`
**Commit:** 4cca1a5
**Applied fix:** Added a heuristic string check `e.to_string().to_lowercase().contains("timeout")` alongside `e.is_timeout()` to catch response timeouts (which `is_timeout()` does not detect) in addition to connect timeouts.

## Skipped Issues

### IN-04: Unnecessary request clone on success path in JSON fallback

**File:** `src/llm/openai_compatible.rs:138`
**Reason:** already fixed by WR-02
**Original issue:** `self.client.chat().create(request.clone()).await` clones the request on every call, but the clone is only needed in the error/fallback path. The WR-02 fix replaced the serde_json round-trip with builder API, which consumes `request` by value on the success path and builds a fresh request for the fallback -- eliminating the clone entirely.

---

_Fixed: 2026-04-29T11:30:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
